//! A small PNG encoder for synthetic images: every row filter can be chosen
//! so the decoder's filters are all exercised.

use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;

/// Rows of raw (unfiltered) sample bytes and how to store them.
pub struct Png<'a> {
    pub width: u32,
    pub colour_type: u8,
    pub bit_depth: u8,
    pub rows: &'a [Vec<u8>],
    pub palette: Option<&'a [u8]>,
    pub transparency: Option<&'a [u8]>,
    /// The filter type of each row; all `None` filters when empty.
    pub filters: &'a [u8],
}

fn chunk(kind: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut crc = flate2::Crc::new();
    crc.update(kind);
    crc.update(payload);
    let mut out = (payload.len() as u32).to_be_bytes().to_vec();
    out.extend(kind);
    out.extend(payload);
    out.extend(crc.sum().to_be_bytes());
    out
}

fn paeth(left: i32, up: i32, up_left: i32) -> i32 {
    let estimate = left + up - up_left;
    let (to_left, to_up) = ((estimate - left).abs(), (estimate - up).abs());
    let to_up_left = (estimate - up_left).abs();
    if to_left <= to_up && to_left <= to_up_left {
        left
    } else if to_up <= to_up_left {
        up
    } else {
        up_left
    }
}

/// `row` filtered with `kind` against `previous`.
fn filter(kind: u8, row: &[u8], previous: &[u8], sample: usize) -> Vec<u8> {
    let mut line = Vec::with_capacity(row.len());
    for (index, value) in row.iter().enumerate() {
        let back = |bytes: &[u8]| {
            if index >= sample { i32::from(bytes[index - sample]) } else { 0 }
        };
        let (left, up, up_left) = (back(row), i32::from(previous[index]), back(previous));
        let predicted = match kind {
            0 => 0,
            1 => left,
            2 => up,
            3 => (left + up) / 2,
            _ => paeth(left, up, up_left),
        };
        line.push((i32::from(*value) - predicted).rem_euclid(256) as u8);
    }
    line
}

impl Png<'_> {
    /// The PNG file bytes.
    pub fn encode(&self) -> Vec<u8> {
        let height = self.rows.len() as u32;
        let mut header = self.width.to_be_bytes().to_vec();
        header.extend(height.to_be_bytes());
        header.extend([self.bit_depth, self.colour_type, 0, 0, 0]);
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        out.extend(chunk(b"IHDR", &header));
        if let Some(palette) = self.palette {
            out.extend(chunk(b"PLTE", palette));
        }
        if let Some(transparency) = self.transparency {
            out.extend(chunk(b"tRNS", transparency));
        }
        out.extend(chunk(b"IDAT", &compress(&self.filtered())));
        out.extend(chunk(b"IEND", b""));
        out
    }

    fn filtered(&self) -> Vec<u8> {
        let channels = match self.colour_type {
            2 => 3,
            4 => 2,
            6 => 4,
            _ => 1,
        };
        // Sample width in bytes for Sub/Average/Paeth; sub-byte depths use 1.
        let sample = (usize::from(self.bit_depth) * channels / 8).max(1);
        let mut raw = Vec::new();
        let mut previous = vec![0; self.rows.first().map_or(0, Vec::len)];
        for (index, row) in self.rows.iter().enumerate() {
            let kind = self.filters.get(index).copied().unwrap_or(0);
            raw.push(kind);
            raw.extend(filter(kind, row, &previous, sample));
            previous.clone_from(row);
        }
        raw
    }
}

fn compress(raw: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(raw).expect("writing to memory cannot fail");
    encoder.finish().expect("writing to memory cannot fail")
}

/// An 8-bit RGBA PNG with unfiltered rows, as hex.
pub fn rgba(width: u32, rows: &[Vec<u8>]) -> String {
    let png = Png { width, colour_type: 6, bit_depth: 8, rows, palette: None, transparency: None, filters: &[] };
    crate::bits::hex(&png.encode())
}
