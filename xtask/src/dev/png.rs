//! A binary PPM frame as a PNG, so a screenshot opens anywhere.

use std::fs;
use std::io::Write;
use std::path::Path;

use flate2::Compression;
use flate2::Crc;
use flate2::write::ZlibEncoder;

/// `(width, height, rgb bytes)` of a `P6` image with 8-bit channels.
fn parse_ppm(bytes: &[u8]) -> Result<(u32, u32, &[u8]), String> {
    let mut fields = Vec::new();
    let mut at = 0;
    while fields.len() < 4 {
        while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        let start = at;
        while bytes
            .get(at)
            .is_some_and(|byte| !byte.is_ascii_whitespace())
        {
            at += 1;
        }
        if start == at {
            return Err("truncated PPM header".to_owned());
        }
        fields.push(String::from_utf8_lossy(&bytes[start..at]).into_owned());
    }
    at += 1;
    if fields[0] != "P6" || fields[3] != "255" {
        return Err("only 8-bit P6 images are supported".to_owned());
    }
    let size = |text: &String| text.parse::<u32>().map_err(|_| "bad PPM size".to_owned());
    let (width, height) = (size(&fields[1])?, size(&fields[2])?);
    let data = bytes.get(at..).ok_or("truncated PPM data")?;
    if data.len() < width as usize * height as usize * 3 {
        return Err("truncated PPM data".to_owned());
    }
    Ok((width, height, data))
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    out.extend(kind);
    out.extend(data);
    let mut crc = Crc::new();
    crc.update(kind);
    crc.update(data);
    out.extend(crc.sum().to_be_bytes());
}

pub fn ppm_to_png(ppm: &Path, png: &Path) -> Result<(), String> {
    let bytes = fs::read(ppm).map_err(|error| format!("{}: {error}", ppm.display()))?;
    let (width, height, rgb) = parse_ppm(&bytes)?;
    let mut scanlines = Vec::with_capacity((width as usize * 3 + 1) * height as usize);
    for row in rgb.chunks_exact(width as usize * 3).take(height as usize) {
        scanlines.push(0);
        scanlines.extend(row);
    }
    let mut zlib = ZlibEncoder::new(Vec::new(), Compression::default());
    zlib.write_all(&scanlines)
        .map_err(|error| error.to_string())?;
    let compressed = zlib.finish().map_err(|error| error.to_string())?;
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend(width.to_be_bytes());
    header.extend(height.to_be_bytes());
    header.extend([8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &compressed);
    chunk(&mut out, b"IEND", &[]);
    fs::write(png, out).map_err(|error| format!("{}: {error}", png.display()))
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use flate2::read::ZlibDecoder;

    use super::*;

    /// The chunks of a PNG as `(kind, data)`, with their CRCs checked.
    fn chunks(png: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let mut at = 8;
        let mut found = Vec::new();
        while at < png.len() {
            let length =
                u32::from_be_bytes([png[at], png[at + 1], png[at + 2], png[at + 3]]) as usize;
            let kind: [u8; 4] = png[at + 4..at + 8].try_into().expect("four bytes");
            let data = png[at + 8..at + 8 + length].to_vec();
            let stored = u32::from_be_bytes(
                png[at + 8 + length..at + 12 + length]
                    .try_into()
                    .expect("four bytes"),
            );
            let mut crc = Crc::new();
            crc.update(&kind);
            crc.update(&data);
            assert_eq!(crc.sum(), stored, "bad CRC");
            found.push((kind, data));
            at += 12 + length;
        }
        found
    }

    #[test]
    fn a_ppm_becomes_a_png_with_the_same_pixels() {
        let dir = std::env::temp_dir().join(format!("photon-png-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        let (ppm, png) = (dir.join("frame.ppm"), dir.join("frame.png"));
        let pixels = [255, 0, 0, 0, 255, 0, 0, 0, 255, 10, 20, 30];
        fs::write(&ppm, [b"P6\n2 2\n255\n".as_slice(), &pixels].concat()).expect("write ppm");
        ppm_to_png(&ppm, &png).expect("convert");
        let chunks = chunks(&fs::read(&png).expect("read png"));
        assert_eq!(
            chunks.iter().map(|(kind, _)| *kind).collect::<Vec<_>>(),
            [*b"IHDR", *b"IDAT", *b"IEND"]
        );
        assert_eq!(chunks[0].1, [0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0, 0, 0]);
        let mut raw = Vec::new();
        ZlibDecoder::new(chunks[1].1.as_slice())
            .read_to_end(&mut raw)
            .expect("inflate");
        assert_eq!(raw, [0, 255, 0, 0, 0, 255, 0, 0, 0, 0, 255, 10, 20, 30]);
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn a_truncated_frame_is_refused() {
        assert!(parse_ppm(b"P6\n4 4\n255\n\x01\x02").is_err());
        assert!(parse_ppm(b"P5\n1 1\n255\n\x01").is_err());
    }
}
