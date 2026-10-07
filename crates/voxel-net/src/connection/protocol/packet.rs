use super::*;

pub(crate) fn encode_packet_frame(
    body: &[u8],
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    if body.len() > MAX_PACKET_UNCOMPRESSED_LENGTH {
        return Err(ConnectionError::Frame(FrameError::PacketTooLarge));
    }

    let mut payload = Vec::new();
    match compression_threshold {
        Some(threshold) if body.len() < threshold => {
            wire::write_varint(0, &mut payload);
            payload.extend_from_slice(body);
        }
        Some(_) => {
            wire::write_varint(
                i32::try_from(body.len())
                    .map_err(|_| ConnectionError::Frame(FrameError::PacketTooLarge))?,
                &mut payload,
            );
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body)?;
            payload.extend_from_slice(
                &encoder
                    .finish()
                    .map_err(|error| ConnectionError::Compression(error.to_string()))?,
            );
        }
        None => payload.extend_from_slice(body),
    }
    frame::prepend(&payload).map_err(ConnectionError::from)
}

pub(crate) fn decode_packet_frame(
    frame: &[u8],
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    let Some(_) = compression_threshold else {
        return Ok(frame.to_vec());
    };

    let mut input = frame;
    let uncompressed_length = wire::read_varint(&mut input)?;
    if uncompressed_length < 0 {
        return Err(ConnectionError::Compression(
            "negative uncompressed length".to_owned(),
        ));
    }
    if uncompressed_length == 0 {
        return Ok(input.to_vec());
    }

    let uncompressed_length = usize::try_from(uncompressed_length)
        .map_err(|_| ConnectionError::Compression("invalid uncompressed length".to_owned()))?;
    if uncompressed_length > MAX_PACKET_UNCOMPRESSED_LENGTH {
        return Err(ConnectionError::Compression(
            "uncompressed packet exceeds 8 MiB".to_owned(),
        ));
    }

    let decoder = ZlibDecoder::new(input);
    let mut output = Vec::with_capacity(uncompressed_length);
    decoder
        .take((uncompressed_length as u64) + 1)
        .read_to_end(&mut output)?;
    if output.len() != uncompressed_length {
        return Err(ConnectionError::Compression(format!(
            "declared {uncompressed_length} bytes, decoded {}",
            output.len()
        )));
    }
    Ok(output)
}

pub(crate) fn split_packet(body: &[u8]) -> Result<(i32, &[u8]), ConnectionError> {
    let mut input = body;
    let packet_id = wire::read_varint(&mut input)?;
    Ok((packet_id, input))
}
