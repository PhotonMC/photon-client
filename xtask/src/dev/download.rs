//! Hash-checked downloads from Mojang's distribution.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::Duration;

use sha1::{Digest, Sha1};
use ureq::Agent;

const ATTEMPTS: u64 = 3;

fn agent() -> &'static Agent {
    static AGENT: OnceLock<Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(60)))
            .timeout_recv_response(Some(Duration::from_secs(60)))
            .timeout_recv_body(Some(Duration::from_secs(900)))
            .build()
            .into()
    })
}

pub fn sha1_of(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut digest = Sha1::new();
    let mut block = vec![0u8; 1 << 20];
    loop {
        let read = file
            .read(&mut block)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if read == 0 {
            return Ok(format!("{:x}", digest.finalize()));
        }
        digest.update(&block[..read]);
    }
}

fn download(url: &str, partial: &Path, sha1: Option<&str>) -> Result<(), String> {
    let mut response = agent()
        .get(url)
        .call()
        .map_err(|error| format!("{url}: {error}"))?;
    let mut out =
        File::create(partial).map_err(|error| format!("{}: {error}", partial.display()))?;
    io::copy(&mut response.body_mut().as_reader(), &mut out)
        .map_err(|error| format!("{url}: {error}"))?;
    drop(out);
    match sha1 {
        Some(want) if sha1_of(partial)? != want => Err(format!("hash mismatch for {url}")),
        _ => Ok(()),
    }
}

/// Downloads `url` to `path` unless a file with the right hash is already
/// there; returns whether it downloaded.
pub fn fetch(url: &str, path: &Path, sha1: Option<&str>) -> Result<bool, String> {
    let present =
        path.is_file() && sha1.is_none_or(|want| sha1_of(path).is_ok_and(|have| have == want));
    if present {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let mut partial = path.as_os_str().to_owned();
    partial.push(".part");
    let partial = Path::new(&partial);
    let mut last = String::new();
    for attempt in 0..ATTEMPTS {
        match download(url, partial, sha1) {
            Ok(()) => {
                fs::rename(partial, path)
                    .map_err(|error| format!("{}: {error}", path.display()))?;
                return Ok(true);
            }
            Err(error) => last = error,
        }
        sleep(Duration::from_secs(1 + attempt));
    }
    Err(last)
}

pub fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    let mut response = agent()
        .get(url)
        .call()
        .map_err(|error| format!("{url}: {error}"))?;
    let mut text = String::new();
    response
        .body_mut()
        .as_reader()
        .read_to_string(&mut text)
        .map_err(|error| format!("{url}: {error}"))?;
    serde_json::from_str(&text).map_err(|error| format!("{url}: {error}"))
}
