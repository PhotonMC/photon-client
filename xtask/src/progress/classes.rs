//! Class hierarchy read from the reference's client jar.
//!
//! A decompiler's output can differ between machines (formatting, nested classes), so counts that
//! depend on the hierarchy come from the class files, which do not.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const ABSTRACT: u16 = 0x0400;
const INTERFACE: u16 = 0x0200;

/// What a class file's header says of its class.
#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub parent: Option<String>,
    pub access: u16,
}

impl Header {
    /// Whether the class can be instantiated by name: not abstract, not an interface.
    pub fn concrete(&self) -> bool {
        self.access & (ABSTRACT | INTERFACE) == 0
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take(&mut self, count: usize) -> Option<&[u8]> {
        let out = self.bytes.get(self.at..self.at + count)?;
        self.at += count;
        Some(out)
    }

    fn u16(&mut self) -> Option<u16> {
        let bytes = self.take(2)?;
        Some(u16::from_be_bytes([bytes[0], bytes[1]]))
    }
}

/// The header of a class file: its name, its parent and its access flags.
pub fn header(bytes: &[u8]) -> Option<Header> {
    let mut reader = Reader { bytes, at: 8 };
    let count = usize::from(reader.u16()?);
    // Utf8 text and Class name indices, by constant pool index.
    let mut text = BTreeMap::new();
    let mut classes = BTreeMap::new();
    let mut index = 1;
    while index < count {
        let tag = *reader.take(1)?.first()?;
        match tag {
            1 => {
                let length = usize::from(reader.u16()?);
                let value = String::from_utf8_lossy(reader.take(length)?).into_owned();
                text.insert(index, value);
            }
            7 => {
                classes.insert(index, usize::from(reader.u16()?));
            }
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => {
                reader.take(4)?;
            }
            5 | 6 => {
                reader.take(8)?;
                index += 1;
            }
            8 | 16 | 19 | 20 => {
                reader.take(2)?;
            }
            15 => {
                reader.take(3)?;
            }
            _ => return None,
        }
        index += 1;
    }
    let access = reader.u16()?;
    let this = usize::from(reader.u16()?);
    let parent = usize::from(reader.u16()?);
    let name_of = |class: usize| text.get(classes.get(&class)?).cloned();
    Some(Header {
        name: name_of(this)?,
        parent: if parent == 0 { None } else { name_of(parent) },
        access,
    })
}

/// The headers of the classes below `prefix` (`net/minecraft/...`) in the jar.
pub fn headers(jar: &Path, prefix: &str) -> Result<Vec<Header>, String> {
    let file = File::open(jar).map_err(|error| format!("{}: {error}", jar.display()))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
    let mut out = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let name = entry.name().map_err(|error| error.to_string())?;
        if !(name.starts_with(prefix) && name.ends_with(".class")) {
            continue;
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        out.extend(header(&bytes));
    }
    Ok(out)
}

/// The concrete classes among `headers` that extend `root`, directly or through other classes of
/// `headers`, leaving out anonymous ones (`Outer$1`).
pub fn concrete_subclasses<'a>(headers: &'a [Header], root: &str) -> Vec<&'a Header> {
    let parents: BTreeMap<&str, Option<&str>> = headers
        .iter()
        .map(|header| (header.name.as_str(), header.parent.as_deref()))
        .collect();
    let extends_root = |name: &str| {
        let mut at = name;
        for _ in 0..32 {
            match parents.get(at) {
                Some(Some(parent)) if *parent == root => return true,
                Some(Some(parent)) => at = parent,
                _ => return false,
            }
        }
        false
    };
    headers
        .iter()
        .filter(|header| {
            header.concrete() && !anonymous(&header.name) && extends_root(&header.name)
        })
        .collect()
}

fn anonymous(name: &str) -> bool {
    name.rsplit('$')
        .next()
        .is_some_and(|last| name.contains('$') && last.chars().all(|c| c.is_ascii_digit()))
}

/// The simple name: after the last `/` and `$`.
pub fn simple_name(name: &str) -> &str {
    name.rsplit(['/', '$']).next().unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A class file with only a constant pool of Utf8 and Class entries and a header.
    fn class(name: &str, parent: Option<&str>, access: u16) -> Vec<u8> {
        let mut out = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 65];
        let mut names = vec![name];
        names.extend(parent);
        let mut pool = Vec::new();
        for (index, text) in names.iter().enumerate() {
            pool.push(1);
            pool.extend((text.len() as u16).to_be_bytes());
            pool.extend(text.as_bytes());
            let _ = index;
        }
        for index in 0..names.len() {
            pool.push(7);
            pool.extend(((index + 1) as u16).to_be_bytes());
        }
        out.extend(((names.len() * 2 + 1) as u16).to_be_bytes());
        out.extend(pool);
        out.extend(access.to_be_bytes());
        out.extend(((names.len() + 1) as u16).to_be_bytes());
        let parent_class = if parent.is_some() { names.len() + 2 } else { 0 };
        out.extend((parent_class as u16).to_be_bytes());
        out
    }

    #[test]
    fn reads_name_parent_and_access() {
        let bytes = class("a/B", Some("a/Screen"), ABSTRACT);
        let header = header(&bytes).unwrap();
        assert_eq!(header.name, "a/B");
        assert_eq!(header.parent.as_deref(), Some("a/Screen"));
        assert!(!header.concrete());
        assert!(super::header(&bytes[..12]).is_none());
    }

    #[test]
    fn finds_concrete_subclasses_through_abstract_ones() {
        let make = |name: &str, parent: &str, access| Header {
            name: name.to_owned(),
            parent: Some(parent.to_owned()),
            access,
        };
        let all = vec![
            make("a/Base", "a/Screen", ABSTRACT),
            make("a/One", "a/Base", 0),
            make("a/Other", "java/lang/Object", 0),
            make("a/One$1", "a/Base", 0),
            make("a/One$Inner", "a/Screen", 0),
        ];
        let names: Vec<&str> = concrete_subclasses(&all, "a/Screen")
            .iter()
            .map(|header| header.name.as_str())
            .collect();
        assert_eq!(names, ["a/One", "a/One$Inner"]);
        assert_eq!(simple_name("a/One$Inner"), "Inner");
    }
}
