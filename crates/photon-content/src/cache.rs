//! The content-addressed cache: where the downloaded files live and how they are named.
//!
//! ```text
//! <root>/objects/<ab>/<sha1>     asset objects, the client jar and every other blob, by digest
//! <root>/indexes/<id>.json       the asset index of the version
//! <root>/versions/<id>.json      the version metadata
//! <root>/jar/<jar sha1>/         the jar's resources (`assets/`, `data/`), without classes
//! ```

use std::path::{Path, PathBuf};

use crate::error::ContentError;
use crate::hash::Sha1Hex;

/// A cache directory.
#[derive(Debug, Clone)]
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    /// A cache in `root`, which need not exist yet.
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The user-level cache: `$XDG_DATA_HOME/photon/content` (else `~/.local/share`),
    /// `%LOCALAPPDATA%\photon\content`, or `~/Library/Application Support/photon/content`.
    pub fn user_level() -> Result<Self, ContentError> {
        let base = user_data_directory(&|name| std::env::var_os(name))
            .ok_or(ContentError::NoCacheDirectory)?;
        Ok(Self::at(base.join("photon").join("content")))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Where the blob with this digest is kept.
    pub fn object(&self, digest: &Sha1Hex) -> PathBuf {
        self.root
            .join("objects")
            .join(digest.fan_out())
            .join(digest.as_str())
    }

    /// Where the asset index of `id` is kept.
    pub fn index(&self, id: &Id) -> PathBuf {
        self.root.join("indexes").join(format!("{}.json", id.0))
    }

    /// Where the version metadata of `id` is kept.
    pub fn version(&self, id: &Id) -> PathBuf {
        self.root.join("versions").join(format!("{}.json", id.0))
    }

    /// Where the resources of the jar with this digest are extracted.
    pub fn jar_tree(&self, jar: &Sha1Hex) -> PathBuf {
        self.root.join("jar").join(jar.as_str())
    }
}

/// A version or index id, safe as a file name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id(String);

impl Id {
    /// `text` when it is only letters, digits, `.`, `_` and `-` (and not `..`).
    pub fn parse(text: &str) -> Result<Self, ContentError> {
        let valid = !text.is_empty()
            && text.len() <= 64
            && text != ".."
            && text != "."
            && text
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
        if valid {
            Ok(Self(text.to_owned()))
        } else {
            Err(ContentError::Metadata(format!(
                "bad id: {}",
                text.chars().take(48).collect::<String>()
            )))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The per-user data directory, from an environment lookup (so that tests need no environment).
fn user_data_directory(var: &dyn Fn(&str) -> Option<std::ffi::OsString>) -> Option<PathBuf> {
    let absolute = |name: &str| var(name).map(PathBuf::from).filter(|p| p.is_absolute());
    if cfg!(windows) {
        return absolute("LOCALAPPDATA");
    }
    if cfg!(target_os = "macos") {
        return absolute("HOME").map(|home| home.join("Library/Application Support"));
    }
    absolute("XDG_DATA_HOME").or_else(|| absolute("HOME").map(|home| home.join(".local/share")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blobs_are_filed_by_their_digest() {
        let cache = Cache::at("/c");
        let digest = Sha1Hex::of_bytes(b"x");
        let path = cache.object(&digest);
        assert!(path.starts_with(format!("/c/objects/{}", digest.fan_out())));
        assert!(path.ends_with(digest.as_str()));
    }

    #[test]
    fn ids_cannot_leave_their_folder() {
        assert!(Id::parse("26.2").is_ok());
        assert!(Id::parse("32").is_ok());
        for bad in ["", "..", "../x", "a/b", "a\\b", "a b", "x\0"] {
            assert!(Id::parse(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_user_directory_follows_the_platform() {
        let env = |pairs: &'static [(&str, &str)]| {
            move |name: &str| {
                pairs
                    .iter()
                    .find(|(key, _)| *key == name)
                    .map(|(_, value)| std::ffi::OsString::from(value))
            }
        };
        if cfg!(all(unix, not(target_os = "macos"))) {
            let both = env(&[("XDG_DATA_HOME", "/x"), ("HOME", "/h")]);
            assert_eq!(user_data_directory(&both), Some(PathBuf::from("/x")));
            let home = env(&[("HOME", "/h")]);
            assert_eq!(
                user_data_directory(&home),
                Some(PathBuf::from("/h/.local/share"))
            );
            let relative = env(&[("XDG_DATA_HOME", "x"), ("HOME", "/h")]);
            assert_eq!(
                user_data_directory(&relative),
                Some(PathBuf::from("/h/.local/share"))
            );
        }
        assert_eq!(user_data_directory(&env(&[])), None);
    }
}
