//! Photon's self-update, first part: deciding whether a release may be trusted.
//!
//! A release is described by a small text manifest (`version`, `archive`, `sha256`, one
//! `key=value` per line) and Ed25519 signatures over the manifest's exact bytes, one per line of the
//! signature file. The public keys the client trusts are [`RELEASE_KEYS`]; nothing is downloaded or
//! applied from a manifest none of whose signatures verifies under one of them, and an archive is
//! applied only if its SHA-256 matches the signed manifest. No private key is ever part of the
//! repository.

use std::fs;
use std::path::{Path, PathBuf};

use ring::signature::{ED25519, UnparsedPublicKey};
use sha2::{Digest, Sha256};

/// The public keys releases are signed with, 32 bytes each. Empty until the owner generates the
/// real key, which makes every update refuse: an empty list is never replaced by "trust anything".
/// A key is rotated by a release signed with the old key whose build lists the old and the new
/// one (ADR 0015).
pub const RELEASE_KEYS: &[[u8; 32]] = &[];

/// Why a release was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// No release key is set in this build.
    NoKey,
    /// No line of the signature file is 64 bytes of hex.
    BadSignature,
    /// No signature matches the manifest under any trusted key.
    Untrusted,
    /// The manifest is not UTF-8 or lacks or repeats a field.
    BadManifest(&'static str),
    /// The archive's SHA-256 differs from the signed one.
    WrongArchive,
    /// The release is not newer than the running version.
    NotNewer,
    /// The archive name is not a plain file name.
    BadName,
    /// The host has no such file (no release yet, or a newer one replaced the archive).
    NotFound,
    /// The pending folder could not be written or the host could not be reached.
    Io(String),
}

/// What a verified manifest says.
#[derive(Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub archive: String,
    pub sha256: [u8; 32],
    /// The archive's size in bytes, signed so a body of another size is refused unread.
    pub size: u64,
    /// One line of release notes, empty when the manifest has none.
    pub notes: String,
    /// The protocol numbers the release can join servers with, empty when the manifest names none.
    pub protocols: Vec<u32>,
}

/// The manifest `bytes` as a release, if a line of `signatures` is a valid signature of exactly
/// these bytes under one of `keys`.
pub fn verify_manifest(
    keys: &[[u8; 32]],
    bytes: &[u8],
    signatures: &str,
) -> Result<Release, Error> {
    if keys.is_empty() {
        return Err(Error::NoKey);
    }
    let signatures: Vec<Vec<u8>> = signatures
        .lines()
        .filter_map(|line| hex(line.trim()))
        .filter(|signature| signature.len() == 64)
        .collect();
    if signatures.is_empty() {
        return Err(Error::BadSignature);
    }
    let trusted = keys.iter().any(|key| {
        let key = UnparsedPublicKey::new(&ED25519, key);
        signatures
            .iter()
            .any(|signature| key.verify(bytes, signature).is_ok())
    });
    if !trusted {
        return Err(Error::Untrusted);
    }
    parse(bytes)
}

/// Whether `archive` is the file the signed `release` names by hash.
pub fn check_archive(release: &Release, archive: &[u8]) -> Result<(), Error> {
    if archive.len() as u64 == release.size && Sha256::digest(archive).as_slice() == release.sha256
    {
        Ok(())
    } else {
        Err(Error::WrongArchive)
    }
}

/// Whether `offered` is a higher dotted number than `current` (`1.2.10` beats `1.2.9`); a version
/// that is not dotted numbers is never newer.
pub fn is_newer(current: &str, offered: &str) -> bool {
    match (numbers(current), numbers(offered)) {
        (Some(current), Some(offered)) => offered > current,
        _ => false,
    }
}

/// A release that was checked and stored, to be applied at the next start.
#[derive(Debug, PartialEq, Eq)]
pub struct Staged {
    pub release: Release,
    pub archive: PathBuf,
}

/// Checks the signed manifest and the archive against it, then stores the archive in `pending`
/// (replacing any earlier one). Nothing is written unless every check passed, the release is newer
/// than `current`, and the archive name is a plain file name.
pub fn stage(
    pending: &Path,
    keys: &[[u8; 32]],
    current: &str,
    (manifest, signature_hex): (&[u8], &str),
    archive: &[u8],
) -> Result<Staged, Error> {
    let release = verify_manifest(keys, manifest, signature_hex)?;
    if !is_newer(current, &release.version) {
        return Err(Error::NotNewer);
    }
    check_archive(&release, archive)?;
    let name = Path::new(&release.archive);
    if name.file_name() != Some(name.as_os_str()) || release.archive.starts_with('.') {
        return Err(Error::BadName);
    }
    let io = |error: std::io::Error| Error::Io(error.to_string());
    let staging = pending.with_extension("tmp");
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(io)?;
    fs::write(staging.join(&release.archive), archive).map_err(io)?;
    fs::write(staging.join("manifest"), manifest).map_err(io)?;
    fs::write(staging.join("manifest.sig"), signature_hex).map_err(io)?;
    let _ = fs::remove_dir_all(pending);
    fs::rename(&staging, pending).map_err(io)?;
    Ok(Staged {
        archive: pending.join(&release.archive),
        release,
    })
}

/// The release stored in `pending`, checked again against the key and its archive's hash (the
/// folder is user-writable, so nothing in it is trusted because it is there).
pub fn staged(pending: &Path, keys: &[[u8; 32]]) -> Result<Staged, Error> {
    let io = |error: std::io::Error| Error::Io(error.to_string());
    let manifest = fs::read(pending.join("manifest")).map_err(io)?;
    let signature = fs::read_to_string(pending.join("manifest.sig")).map_err(io)?;
    let release = verify_manifest(keys, &manifest, &signature)?;
    let name = Path::new(&release.archive);
    if name.file_name() != Some(name.as_os_str()) {
        return Err(Error::BadName);
    }
    let archive = pending.join(&release.archive);
    check_archive(&release, &fs::read(&archive).map_err(io)?)?;
    Ok(Staged { release, archive })
}

fn numbers(version: &str) -> Option<Vec<u64>> {
    version.split('.').map(|part| part.parse().ok()).collect()
}

/// The fields a manifest names, in the order [`fields`] returns their values.
const FIELDS: [&str; 6] = ["version", "archive", "sha256", "size", "notes", "protocols"];

/// The value of each of [`FIELDS`] the manifest text has.
fn fields(bytes: &[u8]) -> Result<[Option<String>; 6], Error> {
    let text = std::str::from_utf8(bytes).map_err(|_| Error::BadManifest("not UTF-8"))?;
    let mut values: [Option<String>; 6] = Default::default();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let (name, value) = line
            .split_once('=')
            .ok_or(Error::BadManifest("a line without ="))?;
        let name = name.trim();
        if !is_name(name) {
            return Err(Error::BadManifest("a malformed field name"));
        }
        // Fields a later release adds are ignored, so an older client keeps reading it.
        let Some(at) = FIELDS.iter().position(|known| *known == name) else {
            continue;
        };
        if values[at].replace(value.trim().to_owned()).is_some() {
            return Err(Error::BadManifest("repeated field"));
        }
    }
    Ok(values)
}

fn parse(bytes: &[u8]) -> Result<Release, Error> {
    let [version, archive, sha256, size, notes, protocols] = fields(bytes)?;
    let sha256 = hex(&sha256.ok_or(Error::BadManifest("no sha256"))?)
        .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
        .ok_or(Error::BadManifest("sha256 is not 32 bytes of hex"))?;
    let size = size
        .ok_or(Error::BadManifest("no size"))?
        .parse()
        .map_err(|_| Error::BadManifest("size is not a number"))?;
    let protocols = protocol_list(protocols.as_deref())?;
    Ok(Release {
        version: version.ok_or(Error::BadManifest("no version"))?,
        archive: archive.ok_or(Error::BadManifest("no archive"))?,
        sha256,
        size,
        notes: notes.unwrap_or_default(),
        protocols,
    })
}

/// The protocol numbers of a comma-separated list, none for a manifest without one.
fn protocol_list(list: Option<&str>) -> Result<Vec<u32>, Error> {
    let Some(list) = list else {
        return Ok(Vec::new());
    };
    list.split(',')
        .map(|part| part.trim().parse())
        .collect::<Result<_, _>>()
        .map_err(|_| Error::BadManifest("protocols are not numbers"))
}

/// Whether `name` is a field name: lowercase letters, digits and underscores.
fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&text[at..at + 2], 16).ok())
        .collect()
}

mod host;
mod swap;
pub use host::{Offer, PLATFORM, RELEASE_HOST, check, download, offer};
pub use swap::{Applied, apply};

#[cfg(test)]
mod host_tests;
#[cfg(test)]
pub(crate) mod tests;
