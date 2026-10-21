//! PNG encoding of a decoded image, for the sprites the client makes itself (a recoloured
//! picture the atlas then reads like any other file).

use std::io::Write as _;

use super::png::DecodedImage;

/// One chunk: length, kind, data and the CRC of kind and data.
fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    out.extend(kind);
    out.extend(data);
    let mut crc = flate2::Crc::new();
    crc.update(kind);
    crc.update(data);
    out.extend(crc.sum().to_be_bytes());
}

/// The image as an RGBA8 PNG with no filtering.
pub fn encode_rgba(image: &DecodedImage) -> Vec<u8> {
    let row = image.width as usize * 4;
    let mut raw = Vec::with_capacity(image.rgba.len() + image.height as usize);
    for line in image.rgba.chunks(row.max(1)) {
        raw.push(0);
        raw.extend(line);
    }
    let mut zlib = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    zlib.write_all(&raw).expect("a vector takes every write");
    let compressed = zlib.finish().expect("a vector takes every write");
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut header = Vec::with_capacity(13);
    header.extend(image.width.to_be_bytes());
    header.extend(image.height.to_be_bytes());
    header.extend([8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &compressed);
    chunk(&mut out, b"IEND", &[]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_encoded_image_decodes_to_itself() {
        let image = DecodedImage {
            width: 3,
            height: 2,
            rgba: (0..24).map(|byte| byte * 9).collect(),
        };
        let decoded = super::super::png::decode_rgba(&encode_rgba(&image)).expect("a PNG");
        assert_eq!(decoded, image);
    }
}
