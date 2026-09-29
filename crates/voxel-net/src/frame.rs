//! The Java 21-bit packet frame boundary used before packet decoding.

use voxel_core::wire;

pub const MAX_VARINT21_BYTES: usize = 3;
pub const MAX_FRAME_LENGTH: usize = (1 << 21) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    LengthTooWide,
    ZeroLength,
    PacketTooLarge,
}

pub fn prepend(body: &[u8]) -> Result<Vec<u8>, FrameError> {
    if body.len() > MAX_FRAME_LENGTH {
        return Err(FrameError::PacketTooLarge);
    }

    let mut framed = Vec::with_capacity(wire::varint_size(body.len() as i32) + body.len());
    wire::write_varint(body.len() as i32, &mut framed);
    framed.extend_from_slice(body);
    Ok(framed)
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Varint21FrameDecoder {
    pending: Vec<u8>,
    consumed: usize,
}

impl Varint21FrameDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one input chunk and perform one Java decoder invocation.
    ///
    /// An incomplete header or body is retained for the next invocation. A
    /// successful invocation removes exactly one frame, matching the decoder's
    /// single `out.add` call; callers can invoke this again to drain more data.
    pub fn feed(&mut self, chunk: &[u8]) -> Result<Option<Vec<u8>>, FrameError> {
        self.pending.extend_from_slice(chunk);

        let mut header_length = None;
        for index in 0..MAX_VARINT21_BYTES {
            let Some(&byte) = self.pending.get(index) else {
                return Ok(None);
            };
            if byte & 0x80 == 0 {
                header_length = Some(index + 1);
                break;
            }
        }

        let Some(header_length) = header_length else {
            self.consume_prefix(MAX_VARINT21_BYTES);
            return Err(FrameError::LengthTooWide);
        };

        let mut header = &self.pending[..header_length];
        let length =
            wire::read_varint(&mut header).expect("a terminated three-byte VarInt must decode");
        if length == 0 {
            self.consume_prefix(header_length);
            return Err(FrameError::ZeroLength);
        }

        let length = length as usize;
        if self.pending.len() - header_length < length {
            return Ok(None);
        }

        let end = header_length + length;
        let frame = self.pending[header_length..end].to_vec();
        self.consume_prefix(end);
        Ok(Some(frame))
    }

    pub fn consumed(&self) -> usize {
        self.consumed
    }

    pub fn remaining(&self) -> &[u8] {
        &self.pending
    }

    fn consume_prefix(&mut self, length: usize) {
        self.pending.drain(..length);
        self.consumed += length;
    }
}
