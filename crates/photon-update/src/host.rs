//! Asking the release host (GitHub Releases) for a newer release and storing it. Everything it
//! sends is checked before it is used: only https (a loopback host for tests), a few redirects,
//! an upper bound on every body, the signed manifest, and the archive against the signed size and
//! hash. A failed check changes nothing on disk.

use std::io::Read;
use std::path::Path;
use std::time::Duration;

use crate::{Error, Release, Staged, is_newer, stage, verify_manifest};

/// Where releases are published; `latest` is the newest release that is not a pre-release.
pub const RELEASE_HOST: &str = "https://github.com/PhotonMC/photon-client/releases/latest/download";

/// The target triple this build is for, which names its files on the host.
pub const PLATFORM: &str = env!("PHOTON_TARGET");

/// What a release host may send, as upper bounds.
const MANIFEST_LIMIT: u64 = 4096;
const SIGNATURE_LIMIT: u64 = 1024;
const ARCHIVE_LIMIT: u64 = 512 * 1024 * 1024;
/// Redirects followed per request (`releases/latest/download` answers with one or two).
const REDIRECTS: u32 = 5;

/// A newer release the host offers: its verified manifest, ready to be downloaded.
#[derive(Debug, PartialEq, Eq)]
pub struct Offer {
    pub release: Release,
    manifest: Vec<u8>,
    signature: String,
}

fn agent(host: &str) -> Result<ureq::Agent, Error> {
    let secure = host.starts_with("https://");
    if !secure && !host.starts_with("http://127.0.0.1:") {
        return Err(Error::Io("the release host must be https".to_owned()));
    }
    Ok(ureq::Agent::config_builder()
        .https_only(secure)
        .max_redirects(REDIRECTS)
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_recv_body(Some(Duration::from_secs(900)))
        .build()
        .into())
}

fn failed(error: ureq::Error) -> Error {
    match error {
        ureq::Error::StatusCode(404) => Error::NotFound,
        other => Error::Io(other.to_string()),
    }
}

fn get(agent: &ureq::Agent, url: &str, limit: u64) -> Result<Vec<u8>, Error> {
    agent
        .get(url)
        .call()
        .map_err(failed)?
        .body_mut()
        .with_config()
        .limit(limit)
        .read_to_vec()
        .map_err(failed)
}

/// Asks `host` for the manifest of `platform`; the release it describes if it is newer than
/// `current`, and `None` for "up to date".
pub fn offer(
    host: &str,
    keys: &[[u8; 32]],
    platform: &str,
    current: &str,
) -> Result<Option<Offer>, Error> {
    // Without a key nothing could be verified, so nothing is asked.
    if keys.is_empty() {
        return Err(Error::NoKey);
    }
    let host = host.trim_end_matches('/');
    let agent = agent(host)?;
    let manifest = get(
        &agent,
        &format!("{host}/photon-{platform}.manifest"),
        MANIFEST_LIMIT,
    )?;
    let signature = get(
        &agent,
        &format!("{host}/photon-{platform}.manifest.sig"),
        SIGNATURE_LIMIT,
    )?;
    let signature = String::from_utf8_lossy(&signature).into_owned();
    let release = verify_manifest(keys, &manifest, &signature)?;
    if !is_newer(current, &release.version) {
        return Ok(None);
    }
    Ok(Some(Offer {
        release,
        manifest,
        signature,
    }))
}

/// Downloads the archive of an offer and stages it in `pending`. `progress` is called with the
/// bytes read so far and the signed total.
pub fn download(
    host: &str,
    keys: &[[u8; 32]],
    offer: &Offer,
    (current, pending): (&str, &Path),
    progress: &mut dyn FnMut(u64, u64),
) -> Result<Staged, Error> {
    let host = host.trim_end_matches('/');
    let agent = agent(host)?;
    let total = offer.release.size.min(ARCHIVE_LIMIT);
    let mut body = agent
        .get(&format!("{host}/{}", offer.release.archive))
        .call()
        .map_err(failed)?
        .into_body();
    // One byte more than signed, so a longer body is read far enough to be refused as wrong.
    let mut reader = body
        .with_config()
        .limit(ARCHIVE_LIMIT + 1)
        .reader()
        .take(total + 1);
    let mut archive = Vec::with_capacity(usize::try_from(total).unwrap_or(0));
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let read = reader
            .read(&mut chunk)
            .map_err(|error| Error::Io(error.to_string()))?;
        if read == 0 {
            break;
        }
        archive.extend_from_slice(&chunk[..read]);
        progress(archive.len() as u64, total);
    }
    stage(
        pending,
        keys,
        current,
        (&offer.manifest, &offer.signature),
        &archive,
    )
}

/// [`offer`] and [`download`] in one: the staged release, or `None` when `host` has nothing
/// newer than `current`.
pub fn check(
    host: &str,
    keys: &[[u8; 32]],
    platform: &str,
    (current, pending): (&str, &Path),
) -> Result<Option<Staged>, Error> {
    let Some(offer) = offer(host, keys, platform, current)? else {
        return Ok(None);
    };
    download(host, keys, &offer, (current, pending), &mut |_, _| {}).map(Some)
}
