//! The `unihex` glyph provider (`UnihexProvider`): the unifont bitmaps, 16 rows of 8, 16, 24 or 32
//! columns per codepoint, read from the `.hex` files of a zip archive. A glyph is as wide as its
//! ink, or as the provider's size override for its range says, and advances half its width plus
//! one.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Read;
use std::sync::{Arc, Mutex, OnceLock};

use photon_core::json::JsonObject;

/// One unifont glyph: the columns it spans in its bitmap, `left` to `right` inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnihexGlyph {
    pub left: i32,
    pub right: i32,
}

impl UnihexGlyph {
    /// `Glyph.width`: the pixels across, at twice the resolution of the other glyphs.
    pub fn width(&self) -> i32 {
        self.right - self.left + 1
    }

    /// `Glyph.info().getAdvance`.
    pub fn advance(&self) -> f32 {
        self.width() as f32 / 2.0 + 1.0
    }
}

/// A size override: the glyphs of `from..=to` span `left..=right` whatever they are drawn with.
#[derive(Debug, Clone, Copy)]
pub struct OverrideRange {
    from: u32,
    to: u32,
    glyph: UnihexGlyph,
}

/// `Definition.sizeOverrides`; none if a range is malformed (the provider then fails to load).
pub fn overrides(provider: &JsonObject) -> Option<Vec<OverrideRange>> {
    let Some(ranges) = provider.array("size_overrides") else {
        return Some(Vec::new());
    };
    ranges
        .iter()
        .map(|range| {
            let photon_core::json::JsonValue::Object(range) = range else {
                return None;
            };
            let (from, to) = (single(&range.text("from")?)?, single(&range.text("to")?)?);
            // `OverrideRange.CODEC`: a range of one codepoint is invalid too.
            (from < to).then_some(())?;
            let glyph = UnihexGlyph {
                left: range.i32("left")?,
                right: range.i32("right")?,
            };
            Some(OverrideRange { from, to, glyph })
        })
        .collect()
}

/// `ExtraCodecs.CODEPOINT`: a string of exactly one codepoint.
fn single(text: &str) -> Option<u32> {
    let mut chars = text.chars();
    let first = chars.next()?;
    chars.next().is_none().then_some(u32::from(first))
}

/// What the hex files say of each codepoint: the columns its ink spans.
type Table = HashMap<u32, UnihexGlyph>;

/// The glyphs of a hex archive under `ranges`: a codepoint in a range takes the first range's
/// columns, the others the span of their ink.
pub fn glyphs(archive: &[u8], ranges: &[OverrideRange]) -> Option<Vec<(u32, UnihexGlyph)>> {
    let table = table(archive)?;
    Some(
        table
            .iter()
            .map(|(&codepoint, &ink)| {
                let glyph = ranges
                    .iter()
                    .find(|range| (range.from..=range.to).contains(&codepoint))
                    .map_or(ink, |range| range.glyph);
                (codepoint, glyph)
            })
            .collect(),
    )
}

/// The parsed archive, kept: the unifont is several megabytes and every font that includes it
/// would read it again.
fn table(archive: &[u8]) -> Option<Arc<Table>> {
    static CACHE: OnceLock<Mutex<HashMap<u64, Arc<Table>>>> = OnceLock::new();
    let mut hasher = DefaultHasher::new();
    archive.hash(&mut hasher);
    let key = hasher.finish();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(table) = cache.lock().ok()?.get(&key) {
        return Some(Arc::clone(table));
    }
    let table = Arc::new(parse_archive(archive)?);
    cache.lock().ok()?.insert(key, Arc::clone(&table));
    Some(table)
}

/// `Definition.loadData`: every `.hex` entry, in archive order; a later line of a codepoint
/// replaces an earlier one.
fn parse_archive(archive: &[u8]) -> Option<Table> {
    let mut table = Table::new();
    for (name, contents) in zip_entries(archive)? {
        if name.ends_with(".hex") {
            read_hex(&contents, &mut table)?;
        }
    }
    Some(table)
}

/// `readFromStream`: `<4 to 6 hex digits>:<32, 64, 96 or 128 hex digits>` per line. Anything else
/// fails the whole provider, as does a lower-case digit.
fn read_hex(contents: &[u8], table: &mut Table) -> Option<()> {
    let mut rest = contents;
    while !rest.is_empty() {
        let colon = rest.iter().position(|&byte| byte == b':')?;
        let digits = &rest[..colon];
        if !(4..=6).contains(&digits.len()) {
            return None;
        }
        let codepoint = digits
            .iter()
            .try_fold(0u32, |value, &digit| Some(value << 4 | hex(digit)?))?;
        // The array behind `CodepointMap` ends at the last codepoint.
        (codepoint <= 0x10FFFF).then_some(())?;
        rest = &rest[colon + 1..];
        let end = rest
            .iter()
            .position(|&byte| byte == b'\n')
            .unwrap_or(rest.len());
        table.insert(codepoint, ink(&rest[..end])?);
        rest = rest.get(end + 1..).unwrap_or(&[]);
    }
    Some(())
}

fn hex(digit: u8) -> Option<u32> {
    match digit {
        b'0'..=b'9' => Some(u32::from(digit - b'0')),
        b'A'..=b'F' => Some(u32::from(digit - b'A') + 10),
        _ => None,
    }
}

/// `LineData.calculateWidth`: the columns the ink of 16 rows of `digits` spans, in the 32 bits the
/// rows are widened to; none for a bitmap that is not 8, 16, 24 or 32 columns wide.
fn ink(digits: &[u8]) -> Option<UnihexGlyph> {
    let columns = match digits.len() {
        32 => 8,
        64 => 16,
        96 => 24,
        128 => 32,
        _ => return None,
    };
    let per_row = digits.len() / 16;
    let mut mask = 0u32;
    for row in digits.chunks(per_row) {
        let value = row
            .iter()
            .try_fold(0u32, |value, &digit| Some(value << 4 | hex(digit)?))?;
        mask |= value << (32 - columns);
    }
    Some(if mask == 0 {
        UnihexGlyph {
            left: 0,
            right: columns,
        }
    } else {
        UnihexGlyph {
            left: mask.leading_zeros() as i32,
            right: 32 - mask.trailing_zeros() as i32 - 1,
        }
    })
}

/// The files of a zip archive, by their central directory: stored or deflated.
fn zip_entries(archive: &[u8]) -> Option<Vec<(String, Vec<u8>)>> {
    let u16_at = |at: usize| -> Option<usize> {
        Some(usize::from(u16::from_le_bytes(
            archive.get(at..at + 2)?.try_into().ok()?,
        )))
    };
    let u32_at = |at: usize| -> Option<usize> {
        Some(u32::from_le_bytes(archive.get(at..at + 4)?.try_into().ok()?) as usize)
    };
    let end = (0..archive.len().checked_sub(22)? + 1)
        .rev()
        .find(|&at| archive[at..].starts_with(b"PK\x05\x06"))?;
    let mut entry = u32_at(end + 16)?;
    let mut files = Vec::new();
    for _ in 0..u16_at(end + 10)? {
        (archive.get(entry..entry + 4)? == b"PK\x01\x02").then_some(())?;
        let (method, size) = (u16_at(entry + 10)?, u32_at(entry + 20)?);
        let (name_length, extra, comment) = (
            u16_at(entry + 28)?,
            u16_at(entry + 30)?,
            u16_at(entry + 32)?,
        );
        let name = String::from_utf8_lossy(archive.get(entry + 46..entry + 46 + name_length)?);
        let local = u32_at(entry + 42)?;
        let data = local + 30 + u16_at(local + 26)? + u16_at(local + 28)?;
        let raw = archive.get(data..data + size)?;
        let mut contents = Vec::new();
        match method {
            0 => contents.extend_from_slice(raw),
            8 => {
                flate2::read::DeflateDecoder::new(raw)
                    .read_to_end(&mut contents)
                    .ok()?;
            }
            _ => return None,
        }
        files.push((name.into_owned(), contents));
        entry += 46 + name_length + extra + comment;
    }
    Some(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ink_spans_the_set_columns() {
        // Row 0 sets columns 2 and 5 of 8 (0x24), the rest are empty.
        let digits = format!("24{}", "00".repeat(15));
        assert_eq!(
            ink(digits.as_bytes()),
            Some(UnihexGlyph { left: 2, right: 5 })
        );
    }

    #[test]
    fn an_empty_bitmap_spans_its_whole_width() {
        let digits = "0".repeat(64);
        assert_eq!(
            ink(digits.as_bytes()),
            Some(UnihexGlyph { left: 0, right: 16 })
        );
    }

    #[test]
    fn wide_bitmaps_keep_their_columns() {
        let wide = format!("FFFFFF{}", "0".repeat(90));
        assert_eq!(
            ink(wide.as_bytes()),
            Some(UnihexGlyph { left: 0, right: 23 })
        );
        let widest = format!("{}00000001", "0".repeat(120));
        assert_eq!(
            ink(widest.as_bytes()),
            Some(UnihexGlyph {
                left: 31,
                right: 31
            })
        );
    }

    #[test]
    fn a_lower_case_digit_fails_the_file() {
        let line = format!("0041:{}\n", "a0".repeat(16));
        assert!(read_hex(line.as_bytes(), &mut Table::new()).is_none());
    }
}
