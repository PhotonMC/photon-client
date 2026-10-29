//! What can go wrong between the pinned game version and the files on disk.

use std::fmt;

/// A failure of the content download. The texts name files and hosts, never a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    /// A hash that is not 40 lowercase hexadecimal digits.
    BadHash(String),
    /// A URL outside the hosts the game's content is fetched from.
    ForeignHost(String),
    /// The server did not answer or answered with an error.
    Network(String),
    /// A download whose size or hash is not the one promised.
    Mismatch(String),
    /// A file could not be read or written.
    Io(String),
    /// Metadata that is not the shape it should be.
    Metadata(String),
    /// A jar member that would land outside the extraction folder.
    UnsafePath(String),
    /// The user-level directory cannot be told.
    NoCacheDirectory,
}

impl fmt::Display for ContentError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadHash(text) => write!(out, "not a SHA-1 hash: {text}"),
            Self::ForeignHost(url) => {
                write!(out, "not a host the game's content comes from: {url}")
            }
            Self::Network(text) => write!(out, "download failed: {text}"),
            Self::Mismatch(text) => write!(out, "download does not match: {text}"),
            Self::Io(text) => write!(out, "file error: {text}"),
            Self::Metadata(text) => write!(out, "unexpected metadata: {text}"),
            Self::UnsafePath(name) => write!(out, "jar member outside the folder: {name}"),
            Self::NoCacheDirectory => write!(out, "no user-level directory for the game's content"),
        }
    }
}

impl std::error::Error for ContentError {}

/// `path: error` as an I/O failure.
pub(crate) fn io_error(path: &std::path::Path, error: &impl fmt::Display) -> ContentError {
    ContentError::Io(format!("{}: {error}", path.display()))
}
