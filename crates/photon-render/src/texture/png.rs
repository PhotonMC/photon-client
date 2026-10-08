//! PNG decoding to RGBA8, matching what the reference gets back from stb.
//!
//! The reference calls `stbi_load_from_memory` asking for four components, so
//! every colour type arrives as RGBA and 16-bit samples are cut to their high
//! byte. This decoder reproduces that result rather than a general-purpose PNG
//! reader's.

use std::io::Read as _;

/// Why a PNG could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PngError {
    /// The file does not start with the PNG signature.
    Signature,
    /// A chunk was truncated or its length was impossible.
    Truncated,
    /// A chunk's CRC did not match its contents.
    Checksum,
    /// The header described a combination this decoder does not handle.
    Unsupported(&'static str),
    /// The compressed image data could not be inflated.
    Inflate,
    /// A filter byte outside 0..=4.
    Filter(u8),
    /// A palette index with no palette entry.
    Palette,
}

/// A decoded image, row-major RGBA8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImage {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Row-major RGBA8 samples, four bytes per pixel.
    pub rgba: Vec<u8>,
}

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Decode a PNG the way the reference's loader does.
pub fn decode_rgba(bytes: &[u8]) -> Result<DecodedImage, PngError> {
    if bytes.len() < SIGNATURE.len() || bytes[..SIGNATURE.len()] != SIGNATURE {
        return Err(PngError::Signature);
    }
    let mut cursor = SIGNATURE.len();
    let mut header: Option<Header> = None;
    let mut palette: Vec<[u8; 3]> = Vec::new();
    let mut transparency: Vec<u8> = Vec::new();
    let mut compressed: Vec<u8> = Vec::new();

    while cursor + 8 <= bytes.len() {
        let length = u32::from_be_bytes(
            bytes[cursor..cursor + 4]
                .try_into()
                .map_err(|_| PngError::Truncated)?,
        ) as usize;
        let kind: [u8; 4] = bytes[cursor + 4..cursor + 8]
            .try_into()
            .map_err(|_| PngError::Truncated)?;
        let body_start = cursor + 8;
        let body_end = body_start.checked_add(length).ok_or(PngError::Truncated)?;
        if body_end + 4 > bytes.len() {
            return Err(PngError::Truncated);
        }
        let body = &bytes[body_start..body_end];
        let stored = u32::from_be_bytes(
            bytes[body_end..body_end + 4]
                .try_into()
                .map_err(|_| PngError::Truncated)?,
        );
        if crc32(&kind, body) != stored {
            return Err(PngError::Checksum);
        }

        match &kind {
            b"IHDR" => header = Some(Header::parse(body)?),
            b"PLTE" => {
                if !body.len().is_multiple_of(3) {
                    return Err(PngError::Truncated);
                }
                palette = body.as_chunks::<3>().0.to_vec();
            }
            b"tRNS" => transparency = body.to_vec(),
            b"IDAT" => compressed.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        cursor = body_end + 4;
    }

    let header = header.ok_or(PngError::Truncated)?;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(compressed.as_slice())
        .read_to_end(&mut raw)
        .map_err(|_| PngError::Inflate)?;

    let samples = header.samples_per_pixel();
    let bits_per_pixel = samples * usize::from(header.bit_depth);
    // Rows are padded to a byte boundary, and the filter works on whole bytes,
    // so a sub-byte pixel still borrows from one byte back.
    let stride = (bits_per_pixel * header.width as usize).div_ceil(8);
    let bytes_per_pixel = bits_per_pixel.div_ceil(8);
    let unfiltered = unfilter(&raw, stride, bytes_per_pixel, header.height as usize)?;

    let bytes_per_sample = usize::from(header.bit_depth).div_ceil(8);
    let expanded_stride = samples * bytes_per_sample * header.width as usize;
    let mut rgba = Vec::with_capacity(header.width as usize * header.height as usize * 4);
    let mut expanded = vec![0_u8; expanded_stride];
    for row in unfiltered.chunks_exact(stride) {
        header.expand_row(row, &mut expanded);
        for pixel in expanded.chunks_exact(samples * bytes_per_sample) {
            rgba.extend_from_slice(&header.to_rgba(pixel, &palette, &transparency)?);
        }
    }

    Ok(DecodedImage {
        width: header.width,
        height: header.height,
        rgba,
    })
}

#[derive(Debug, Clone, Copy)]
struct Header {
    width: u32,
    height: u32,
    bit_depth: u8,
    colour_type: u8,
}

impl Header {
    fn parse(body: &[u8]) -> Result<Self, PngError> {
        if body.len() < 13 {
            return Err(PngError::Truncated);
        }
        let width = u32::from_be_bytes(body[0..4].try_into().map_err(|_| PngError::Truncated)?);
        let height = u32::from_be_bytes(body[4..8].try_into().map_err(|_| PngError::Truncated)?);
        let bit_depth = body[8];
        let colour_type = body[9];
        if body[10] != 0 {
            return Err(PngError::Unsupported("compression method"));
        }
        if body[11] != 0 {
            return Err(PngError::Unsupported("filter method"));
        }
        if body[12] != 0 {
            return Err(PngError::Unsupported("interlacing"));
        }
        if !matches!(bit_depth, 1 | 2 | 4 | 8 | 16) {
            return Err(PngError::Unsupported("bit depth"));
        }
        if !matches!(colour_type, 0 | 2 | 3 | 4 | 6) {
            return Err(PngError::Unsupported("colour type"));
        }
        // Only greyscale and palette may pack several samples into a byte.
        if bit_depth < 8 && !matches!(colour_type, 0 | 3) {
            return Err(PngError::Unsupported("sub-byte colour type"));
        }
        if width == 0 || height == 0 {
            return Err(PngError::Unsupported("zero dimension"));
        }
        Ok(Self {
            width,
            height,
            bit_depth,
            colour_type,
        })
    }

    /// Widen one row to a byte per sample, keeping the raw sample values.
    ///
    /// Sub-byte samples are unpacked most significant bits first and left
    /// unscaled: the transparency key is written at the original depth, so the
    /// comparison has to happen before any scaling.
    fn expand_row(self, row: &[u8], out: &mut [u8]) {
        if self.bit_depth >= 8 {
            out.copy_from_slice(&row[..out.len()]);
            return;
        }
        let depth = u32::from(self.bit_depth);
        let mask = (1_u16 << depth) - 1;
        for (index, slot) in out.iter_mut().enumerate() {
            let bit = index as u32 * depth;
            let byte = row[(bit / 8) as usize];
            let shift = 8 - depth - (bit % 8);
            *slot = ((u16::from(byte) >> shift) & mask) as u8;
        }
    }

    /// Scale a raw sample up to the full 8-bit range, as stb does for the
    /// depths that do not already fill a byte.
    fn scale_sample(self, raw: u16) -> u8 {
        match self.bit_depth {
            1 => (raw as u8).wrapping_mul(0xFF),
            2 => (raw as u8).wrapping_mul(0x55),
            4 => (raw as u8).wrapping_mul(0x11),
            16 => (raw >> 8) as u8,
            _ => raw as u8,
        }
    }

    /// One raw sample, at whatever width the header declared.
    fn raw_sample(self, pixel: &[u8], index: usize) -> u16 {
        if self.bit_depth == 16 {
            u16::from_be_bytes([pixel[index * 2], pixel[index * 2 + 1]])
        } else {
            u16::from(pixel[index])
        }
    }

    fn samples_per_pixel(self) -> usize {
        match self.colour_type {
            0 | 3 => 1,
            2 => 3,
            4 => 2,
            _ => 4,
        }
    }

    /// One pixel, widened to RGBA8 the way stb does for four requested
    /// components: grey replicates across the colour channels, missing alpha
    /// becomes opaque, and 16-bit samples keep only their high byte.
    fn to_rgba(
        self,
        pixel: &[u8],
        palette: &[[u8; 3]],
        transparency: &[u8],
    ) -> Result<[u8; 4], PngError> {
        Ok(match self.colour_type {
            0 => {
                let raw = self.raw_sample(pixel, 0);
                let grey = self.scale_sample(raw);
                // A greyscale image may nominate one sample value as fully
                // transparent; the key is stored at the source bit depth.
                let key = key_sample(transparency, 0);
                let alpha = if key == Some(raw) { 0 } else { 255 };
                [grey, grey, grey, alpha]
            }
            2 => {
                let raw = [
                    self.raw_sample(pixel, 0),
                    self.raw_sample(pixel, 1),
                    self.raw_sample(pixel, 2),
                ];
                let keys = [
                    key_sample(transparency, 0),
                    key_sample(transparency, 1),
                    key_sample(transparency, 2),
                ];
                let transparent = keys
                    .iter()
                    .zip(raw)
                    .all(|(key, sample)| *key == Some(sample));
                [
                    self.scale_sample(raw[0]),
                    self.scale_sample(raw[1]),
                    self.scale_sample(raw[2]),
                    if transparent { 0 } else { 255 },
                ]
            }
            3 => {
                let index = usize::from(pixel[0]);
                let entry = palette.get(index).ok_or(PngError::Palette)?;
                let alpha = transparency.get(index).copied().unwrap_or(255);
                [entry[0], entry[1], entry[2], alpha]
            }
            4 => {
                let grey = self.scale_sample(self.raw_sample(pixel, 0));
                [
                    grey,
                    grey,
                    grey,
                    self.scale_sample(self.raw_sample(pixel, 1)),
                ]
            }
            _ => [
                self.scale_sample(self.raw_sample(pixel, 0)),
                self.scale_sample(self.raw_sample(pixel, 1)),
                self.scale_sample(self.raw_sample(pixel, 2)),
                self.scale_sample(self.raw_sample(pixel, 3)),
            ],
        })
    }
}

/// Reverse the per-row filters, in place across the image.
fn unfilter(
    raw: &[u8],
    stride: usize,
    bytes_per_pixel: usize,
    height: usize,
) -> Result<Vec<u8>, PngError> {
    if raw.len() < height * (stride + 1) {
        return Err(PngError::Truncated);
    }
    let mut out = vec![0_u8; stride * height];
    for row in 0..height {
        let filter = raw[row * (stride + 1)];
        let source = &raw[row * (stride + 1) + 1..row * (stride + 1) + 1 + stride];
        for index in 0..stride {
            let left = if index >= bytes_per_pixel {
                out[row * stride + index - bytes_per_pixel]
            } else {
                0
            };
            let up = if row > 0 {
                out[(row - 1) * stride + index]
            } else {
                0
            };
            let up_left = if row > 0 && index >= bytes_per_pixel {
                out[(row - 1) * stride + index - bytes_per_pixel]
            } else {
                0
            };
            let value = source[index];
            out[row * stride + index] = match filter {
                0 => value,
                1 => value.wrapping_add(left),
                2 => value.wrapping_add(up),
                3 => value.wrapping_add(((u16::from(left) + u16::from(up)) / 2) as u8),
                4 => value.wrapping_add(paeth(left, up, up_left)),
                other => return Err(PngError::Filter(other)),
            };
        }
    }
    Ok(out)
}

/// The PNG Paeth predictor, ties going to the left neighbour then above.
fn paeth(left: u8, up: u8, up_left: u8) -> u8 {
    let estimate = i16::from(left) + i16::from(up) - i16::from(up_left);
    let to_left = (estimate - i16::from(left)).abs();
    let to_up = (estimate - i16::from(up)).abs();
    let to_up_left = (estimate - i16::from(up_left)).abs();
    if to_left <= to_up && to_left <= to_up_left {
        left
    } else if to_up <= to_up_left {
        up
    } else {
        up_left
    }
}

/// CRC-32 over a chunk's type and body, as PNG specifies.
fn crc32(kind: &[u8; 4], body: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for byte in kind.iter().chain(body) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// One transparency key from a `tRNS` chunk, which stores 16-bit big-endian
/// values whatever the image's bit depth is.
fn key_sample(transparency: &[u8], index: usize) -> Option<u16> {
    let at = index * 2;
    let bytes = transparency.get(at..at + 2)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}
