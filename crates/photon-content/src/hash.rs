//! SHA-1 digests as the metadata writes them: 40 lowercase hexadecimal digits. A value of
//! [`Sha1Hex`] is safe to put in a path, which downloaded text is not.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha1::{Digest, Sha1};

use crate::error::{ContentError, io_error};

/// A checked SHA-1 digest.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sha1Hex(String);

impl Sha1Hex {
    /// The digest `text` names, or why it is none.
    pub fn parse(text: &str) -> Result<Self, ContentError> {
        let valid =
            text.len() == 40 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        if valid {
            Ok(Self(text.to_owned()))
        } else {
            Err(ContentError::BadHash(text.chars().take(48).collect()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The two digits the object store files it under.
    pub(crate) fn fan_out(&self) -> &str {
        &self.0[..2]
    }

    /// The digest of `bytes`.
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Self(format!("{:x}", Sha1::digest(bytes)))
    }

    /// The digest of a file's content.
    pub fn of_file(path: &Path) -> Result<Self, ContentError> {
        let mut file = File::open(path).map_err(|error| io_error(path, &error))?;
        let mut digest = Sha1::new();
        let mut block = vec![0u8; 1 << 20];
        loop {
            let read = file
                .read(&mut block)
                .map_err(|error| io_error(path, &error))?;
            if read == 0 {
                return Ok(Self(format!("{:x}", digest.finalize())));
            }
            digest.update(&block[..read]);
        }
    }
}

/// A running digest, for hashing a download as it is written.
pub(crate) struct Hasher(Sha1);

impl Hasher {
    pub(crate) fn new() -> Self {
        Self(Sha1::new())
    }

    pub(crate) fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    pub(crate) fn finish(self) -> Sha1Hex {
        Sha1Hex(format!("{:x}", self.0.finalize()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_forty_lowercase_hex_digits_are_a_digest() {
        let good = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
        assert_eq!(Sha1Hex::parse(good).unwrap().as_str(), good);
        assert_eq!(Sha1Hex::of_bytes(b""), Sha1Hex::parse(good).unwrap());
        for bad in [
            "",
            "da39",
            &good.to_uppercase(),
            &format!("{good}0"),
            "../../etc/passwd/aaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "da39a3ee5e6b4b0d3255bfef95601890afd8070g",
        ] {
            assert!(
                matches!(Sha1Hex::parse(bad), Err(ContentError::BadHash(_))),
                "{bad}"
            );
        }
    }
}
