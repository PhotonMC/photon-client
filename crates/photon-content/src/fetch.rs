//! Hash-checked downloads: a file lands under its name only after its size and SHA-1 are the
//! promised ones.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use crate::endpoints::Endpoints;
use crate::error::{ContentError, io_error};
use crate::hash::{Hasher, Sha1Hex};

const ATTEMPTS: u32 = 3;

/// What a download must turn out to be.
pub(crate) struct Want<'a> {
    pub(crate) sha1: &'a Sha1Hex,
    /// The promised size; without one, `limit` caps what is accepted.
    pub(crate) size: Option<u64>,
    pub(crate) limit: u64,
}

pub(crate) fn agent(endpoints: &Endpoints) -> ureq::Agent {
    ureq::Agent::config_builder()
        .https_only(endpoints.is_secure())
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_recv_response(Some(Duration::from_secs(60)))
        .timeout_recv_body(Some(Duration::from_secs(900)))
        .build()
        .into()
}

/// Whether `dest` already holds the wanted file. A size is trusted (the file was renamed into
/// place after its hash was checked); without one the content is hashed.
fn present(dest: &Path, want: &Want<'_>) -> bool {
    let Ok(meta) = fs::metadata(dest) else {
        return false;
    };
    match want.size {
        Some(size) => meta.is_file() && meta.len() == size,
        None => meta.is_file() && Sha1Hex::of_file(dest).is_ok_and(|have| &have == want.sha1),
    }
}

/// One attempt: the body of `url` into `partial`, hashed on the way.
fn download(
    agent: &ureq::Agent,
    url: &str,
    partial: &Path,
    want: &Want<'_>,
) -> Result<(), ContentError> {
    let limit = want.size.unwrap_or(want.limit);
    let mut response = agent
        .get(url)
        .call()
        .map_err(|error| ContentError::Network(format!("{}: {error}", short(url))))?;
    let mut out = File::create(partial).map_err(|error| io_error(partial, &error))?;
    let mut reader = response
        .body_mut()
        .as_reader()
        .take(limit.saturating_add(1));
    let mut hasher = Hasher::new();
    let mut block = vec![0u8; 1 << 16];
    let mut total = 0u64;
    loop {
        let read = reader
            .read(&mut block)
            .map_err(|error| ContentError::Network(format!("{}: {error}", short(url))))?;
        if read == 0 {
            break;
        }
        total += read as u64;
        hasher.update(&block[..read]);
        out.write_all(&block[..read])
            .map_err(|error| io_error(partial, &error))?;
    }
    drop(out);
    if total > limit || want.size.is_some_and(|size| size != total) {
        return Err(ContentError::Mismatch(format!("size of {}", short(url))));
    }
    if &hasher.finish() != want.sha1 {
        return Err(ContentError::Mismatch(format!("hash of {}", short(url))));
    }
    Ok(())
}

/// `url` without a query, for messages.
fn short(url: &str) -> String {
    url.split('?')
        .next()
        .unwrap_or(url)
        .chars()
        .take(120)
        .collect()
}

/// Download `url` to `dest` unless the wanted file is already there; whether it downloaded.
/// The URL must already have passed the endpoints' check.
pub(crate) fn fetch(
    endpoints: &Endpoints,
    agent: &ureq::Agent,
    url: &str,
    dest: &Path,
    want: &Want<'_>,
) -> Result<bool, ContentError> {
    if present(dest, want) {
        return Ok(false);
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error(parent, &error))?;
    }
    let mut partial = dest.as_os_str().to_owned();
    partial.push(".part");
    let partial = Path::new(&partial);
    let mut last = ContentError::Network(short(url));
    for attempt in 0..ATTEMPTS {
        match download(agent, url, partial, want) {
            Ok(()) => {
                fs::rename(partial, dest).map_err(|error| io_error(dest, &error))?;
                return Ok(true);
            }
            Err(error) => last = error,
        }
        let _ = fs::remove_file(partial);
        std::thread::sleep(endpoints.retry_delay * (attempt + 1));
    }
    Err(last)
}

/// The body of `url` as text, up to `limit` bytes.
pub(crate) fn fetch_text(
    agent: &ureq::Agent,
    url: &str,
    limit: u64,
) -> Result<String, ContentError> {
    let mut response = agent
        .get(url)
        .call()
        .map_err(|error| ContentError::Network(format!("{}: {error}", short(url))))?;
    let mut text = String::new();
    response
        .body_mut()
        .as_reader()
        .take(limit.saturating_add(1))
        .read_to_string(&mut text)
        .map_err(|error| ContentError::Network(format!("{}: {error}", short(url))))?;
    if text.len() as u64 > limit {
        return Err(ContentError::Mismatch(format!("size of {}", short(url))));
    }
    Ok(text)
}
