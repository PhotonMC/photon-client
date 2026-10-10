//! `Screenshot.grab`: the frame written to `screenshots/` of the game directory under the date it
//! was taken, and the chat message that says where.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::Write;

/// A date and time of day, as `yyyy-MM-dd_HH.mm.ss` writes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: i64,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl DateTime {
    /// The moment `seconds` after the epoch, in UTC.
    pub fn from_unix(seconds: i64) -> Self {
        let days = seconds.div_euclid(86_400);
        let in_day = seconds.rem_euclid(86_400);
        // The civil date of a day number (days from 1970-01-01), shifted to start in March.
        let shifted = days + 719_468;
        let era = shifted.div_euclid(146_097);
        let day_of_era = shifted.rem_euclid(146_097);
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_index = (5 * day_of_year + 2) / 153;
        let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
        let month = if month_index < 10 {
            month_index + 3
        } else {
            month_index - 9
        } as u32;
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        Self {
            year,
            month,
            day,
            hour: (in_day / 3600) as u32,
            minute: (in_day % 3600 / 60) as u32,
            second: (in_day % 60) as u32,
        }
    }

    /// The time on the wall clock: UTC moved by the offset the system's `date` reports, or UTC
    /// where there is none to ask.
    pub fn now() -> Self {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs() as i64);
        Self::from_unix(seconds + local_offset().unwrap_or(0))
    }

    /// `Util.getFilenameFormattedDateTime`.
    pub fn file_name(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}_{:02}.{:02}.{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

/// The local zone's offset from UTC in seconds, from `date +%z` (`+0300`, `-0930`).
fn local_offset() -> Option<i64> {
    let output = std::process::Command::new("date")
        .arg("+%z")
        .output()
        .ok()?;
    parse_offset(String::from_utf8(output.stdout).ok()?.trim())
}

fn parse_offset(text: &str) -> Option<i64> {
    let (sign, digits) = match text.split_at_checked(1)? {
        ("+", digits) => (1, digits),
        ("-", digits) => (-1, digits),
        _ => return None,
    };
    if digits.len() != 4 {
        return None;
    }
    let hours: i64 = digits[..2].parse().ok()?;
    let minutes: i64 = digits[2..].parse().ok()?;
    Some(sign * (hours * 3600 + minutes * 60))
}

/// `Screenshot.getFile`: the date's name, with a count from two on when it is taken.
fn free_file(directory: &Path, name: &str) -> PathBuf {
    let mut count = 1;
    loop {
        let suffix = if count == 1 {
            String::new()
        } else {
            format!("_{count}")
        };
        let file = directory.join(format!("{name}{suffix}.png"));
        if !file.exists() {
            return file;
        }
        count += 1;
    }
}

fn chunk(out: &mut Vec<u8>, tag: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(tag);
    out.extend_from_slice(body);
    let mut crc = flate2::Crc::new();
    crc.update(tag);
    crc.update(body);
    out.extend_from_slice(&crc.sum().to_be_bytes());
}

/// An opaque RGBA8 image as a PNG file.
pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut raw = Vec::with_capacity((width as usize * 3 + 1) * height as usize);
    for row in rgba.chunks_exact(width as usize * 4) {
        raw.push(0);
        for pixel in row.as_chunks::<4>().0 {
            raw.extend_from_slice(&pixel[..3]);
        }
    }
    let mut zlib = ZlibEncoder::new(Vec::new(), Compression::default());
    zlib.write_all(&raw)?;
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    // Eight bits a channel, colour without alpha, the one method, filter and interlace.
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, b"IHDR", &header);
    chunk(&mut png, b"IDAT", &zlib.finish()?);
    chunk(&mut png, b"IEND", &[]);
    Ok(png)
}

/// Write the frame to `screenshots/` of the game directory; where it went.
pub fn save(
    game_directory: &Path,
    (width, height, rgba): (u32, u32, &[u8]),
    when: DateTime,
) -> std::io::Result<PathBuf> {
    let directory = game_directory.join("screenshots");
    std::fs::create_dir_all(&directory)?;
    let file = free_file(&directory, &when.file_name());
    std::fs::write(&file, encode_png(width, height, rgba)?)?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_unix_time_is_a_date_and_a_time_of_day() {
        // 2026-10-10T09:05:32Z.
        let when = DateTime::from_unix(1_791_623_132);
        assert_eq!(when.file_name(), "2026-10-10_09.05.32");
        assert_eq!(DateTime::from_unix(0).file_name(), "1970-01-01_00.00.00");
        // The leap day.
        assert_eq!(
            DateTime::from_unix(951_825_600).file_name(),
            "2000-02-29_12.00.00"
        );
    }

    #[test]
    fn the_zone_offset_is_read_from_the_date_format() {
        assert_eq!(parse_offset("+0300"), Some(10_800));
        assert_eq!(parse_offset("-0930"), Some(-34_200));
        assert_eq!(parse_offset("UTC"), None);
    }

    #[test]
    fn a_taken_name_gets_a_count() {
        let directory = std::env::temp_dir().join(format!("photon-shot-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let first = free_file(&directory, "2026-10-10_09.05.32");
        assert!(first.ends_with("2026-10-10_09.05.32.png"));
        std::fs::write(&first, []).unwrap();
        assert!(
            free_file(&directory, "2026-10-10_09.05.32").ends_with("2026-10-10_09.05.32_2.png")
        );
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn a_frame_is_a_png() {
        let png = encode_png(2, 1, &[255, 0, 0, 255, 0, 255, 0, 255]).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
    }
}
