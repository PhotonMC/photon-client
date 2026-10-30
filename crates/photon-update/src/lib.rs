//! Photon's self-update, first part: deciding whether a release may be trusted.
//!
//! A release is described by a small text manifest (`version`, `archive`, `sha256`, one
//! `key=value` per line) and an Ed25519 signature over the manifest's exact bytes. The only public
//! key the client trusts is [`RELEASE_KEY`]; nothing is downloaded or applied from a manifest
//! whose signature does not verify, and an archive is applied only if its SHA-256 matches the
//! signed manifest. No private key is ever part of the repository.

use std::fs;
use std::path::{Path, PathBuf};

use ring::signature::{ED25519, UnparsedPublicKey};
use sha2::{Digest, Sha256};

/// The public key releases are signed with, as 32 bytes. `None` until the owner generates the real
/// key, which makes every update refuse: an unset key is never replaced by "trust anything".
pub const RELEASE_KEY: Option<[u8; 32]> = None;

/// Why a release was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// No release key is set in this build.
    NoKey,
    /// The signature is not 64 bytes of hex.
    BadSignature,
    /// The signature does not match the manifest under the key.
    Untrusted,
    /// The manifest is not UTF-8 or lacks or repeats a field.
    BadManifest(&'static str),
    /// The archive's SHA-256 differs from the signed one.
    WrongArchive,
    /// The release is not newer than the running version.
    NotNewer,
    /// The archive name is not a plain file name.
    BadName,
    /// The pending folder could not be written.
    Io(String),
}

/// What a verified manifest says.
#[derive(Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub archive: String,
    pub sha256: [u8; 32],
}

/// The manifest `bytes` as a release, if `signature_hex` is a valid signature of exactly these
/// bytes under `key`.
pub fn verify_manifest(
    key: Option<&[u8; 32]>,
    bytes: &[u8],
    signature_hex: &str,
) -> Result<Release, Error> {
    let key = key.ok_or(Error::NoKey)?;
    let signature = hex(signature_hex.trim()).ok_or(Error::BadSignature)?;
    UnparsedPublicKey::new(&ED25519, key)
        .verify(bytes, &signature)
        .map_err(|_| Error::Untrusted)?;
    parse(bytes)
}

/// Whether `archive` is the file the signed `release` names by hash.
pub fn check_archive(release: &Release, archive: &[u8]) -> Result<(), Error> {
    if Sha256::digest(archive).as_slice() == release.sha256 {
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
    key: Option<&[u8; 32]>,
    current: &str,
    (manifest, signature_hex): (&[u8], &str),
    archive: &[u8],
) -> Result<Staged, Error> {
    let release = verify_manifest(key, manifest, signature_hex)?;
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
pub fn staged(pending: &Path, key: Option<&[u8; 32]>) -> Result<Staged, Error> {
    let io = |error: std::io::Error| Error::Io(error.to_string());
    let manifest = fs::read(pending.join("manifest")).map_err(io)?;
    let signature = fs::read_to_string(pending.join("manifest.sig")).map_err(io)?;
    let release = verify_manifest(key, &manifest, &signature)?;
    let name = Path::new(&release.archive);
    if name.file_name() != Some(name.as_os_str()) {
        return Err(Error::BadName);
    }
    let archive = pending.join(&release.archive);
    check_archive(&release, &fs::read(&archive).map_err(io)?)?;
    Ok(Staged { release, archive })
}

/// What a release host may send, as upper bounds.
const MANIFEST_LIMIT: u64 = 4096;
const SIGNATURE_LIMIT: u64 = 256;
const ARCHIVE_LIMIT: u64 = 512 * 1024 * 1024;

/// Asks `base` (a folder URL holding `manifest`, `manifest.sig` and the archive it names) for a
/// release newer than `current`, and stages it in `pending`. `None` is "up to date". Only
/// `https://` is used, except a loopback host (for tests); a failed check changes nothing.
pub fn check(
    base: &str,
    key: Option<&[u8; 32]>,
    current: &str,
    pending: &Path,
) -> Result<Option<Staged>, Error> {
    let base = base.trim_end_matches('/');
    let secure = base.starts_with("https://");
    if !secure && !base.starts_with("http://127.0.0.1:") {
        return Err(Error::Io("the release host must be https".to_owned()));
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(secure)
        .timeout_connect(Some(std::time::Duration::from_secs(30)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(900)))
        .build()
        .into();
    let get = |name: &str, limit: u64| -> Result<Vec<u8>, Error> {
        agent
            .get(&format!("{base}/{name}"))
            .call()
            .map_err(|error| Error::Io(error.to_string()))?
            .body_mut()
            .with_config()
            .limit(limit)
            .read_to_vec()
            .map_err(|error| Error::Io(error.to_string()))
    };
    let manifest = get("manifest", MANIFEST_LIMIT)?;
    let signature = String::from_utf8_lossy(&get("manifest.sig", SIGNATURE_LIMIT)?).into_owned();
    let release = verify_manifest(key, &manifest, &signature)?;
    if !is_newer(current, &release.version) {
        return Ok(None);
    }
    let archive = get(&release.archive, ARCHIVE_LIMIT)?;
    stage(pending, key, current, (&manifest, &signature), &archive).map(Some)
}

fn numbers(version: &str) -> Option<Vec<u64>> {
    version.split('.').map(|part| part.parse().ok()).collect()
}

fn parse(bytes: &[u8]) -> Result<Release, Error> {
    let text = std::str::from_utf8(bytes).map_err(|_| Error::BadManifest("not UTF-8"))?;
    let (mut version, mut archive, mut sha256) = (None, None, None);
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let (name, value) = line
            .split_once('=')
            .ok_or(Error::BadManifest("a line without ="))?;
        let slot = match name.trim() {
            "version" => &mut version,
            "archive" => &mut archive,
            "sha256" => &mut sha256,
            _ => return Err(Error::BadManifest("unknown field")),
        };
        if slot.replace(value.trim().to_owned()).is_some() {
            return Err(Error::BadManifest("repeated field"));
        }
    }
    let sha256 = hex(&sha256.ok_or(Error::BadManifest("no sha256"))?)
        .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
        .ok_or(Error::BadManifest("sha256 is not 32 bytes of hex"))?;
    Ok(Release {
        version: version.ok_or(Error::BadManifest("no version"))?,
        archive: archive.ok_or(Error::BadManifest("no archive"))?,
        sha256,
    })
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

#[cfg(test)]
mod tests;
