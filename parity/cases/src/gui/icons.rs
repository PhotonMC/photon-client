//! The server icons the ping cases carry: a gradient PNG made without any
//! asset, as a data URL. Its pixel data is stored, not compressed, so the
//! bytes do not depend on the zlib that wrote them.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(data: &[u8]) -> String {
    let mut out = String::new();
    for group in data.chunks(3) {
        let bits =
            group.iter().enumerate().fold(0u32, |bits, (index, byte)| bits | (u32::from(*byte) << (16 - 8 * index)));
        for index in 0..=group.len() {
            out.push(char::from(ALPHABET[((bits >> (18 - 6 * index)) & 63) as usize]));
        }
        out.extend(std::iter::repeat_n('=', 3 - group.len()));
    }
    out
}

/// `data` as the data URL a server status carries its icon in.
pub fn favicon(data: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64(data))
}

fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut crc = flate2::Crc::new();
    crc.update(kind);
    crc.update(data);
    let mut out = (data.len() as u32).to_be_bytes().to_vec();
    out.extend(kind);
    out.extend(data);
    out.extend(crc.sum().to_be_bytes());
    out
}

/// `data` as a zlib stream of stored blocks.
fn stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut blocks = data.chunks(0xffff).peekable();
    while let Some(block) = blocks.next() {
        let length = block.len() as u16;
        out.push(u8::from(blocks.peek().is_none()));
        out.extend(length.to_le_bytes());
        out.extend((!length).to_le_bytes());
        out.extend(block);
    }
    let (mut low, mut high) = (1u32, 0u32);
    for byte in data {
        low = (low + u32::from(*byte)) % 65521;
        high = (high + low) % 65521;
    }
    out.extend(((high << 16) | low).to_be_bytes());
    out
}

/// A valid RGB PNG of a gradient.
pub fn png(width: u32, height: u32, seed: u32) -> Vec<u8> {
    let mut rows = Vec::new();
    for y in 0..height {
        rows.push(0);
        for x in 0..width {
            rows.extend([((x * 4 + seed) % 256) as u8, ((y * 4) % 256) as u8, ((x + y + seed) % 256) as u8]);
        }
    }
    let mut header = Vec::new();
    header.extend(width.to_be_bytes());
    header.extend(height.to_be_bytes());
    header.extend([8, 2, 0, 0, 0]);
    let compressed = stored(&rows);
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    out.extend(chunk(b"IHDR", &header));
    out.extend(chunk(b"IDAT", &compressed));
    out.extend(chunk(b"IEND", &[]));
    out
}
