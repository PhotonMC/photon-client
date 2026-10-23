//! Fetching a pack over HTTP(S) into the cache (`DownloadQueue` and `HttpUtil.downloadFile`).
//!
//! A pack is kept at `<cache>/<pack id>/<sha1>`. A file already there with the hash the server
//! named is used as it is; otherwise the pack is downloaded, hashed as it arrives and checked
//! against the hash. Without a hash the file is named for the hash it turned out to have.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use photon_core::binary::Uuid;
use sha1::{Digest, Sha1};

use super::cache;
use super::manager::DownloadRequest;
use super::source::{PackFetcher, PackUser};

/// The largest pack that is taken (`DownloadedPackSource`'s limit, in bytes).
pub const MAX_PACK_BYTES: u64 = 262_144_000;

/// The pack format of the client's resources, for the header the servers see.
const PACK_FORMAT: &str = "88.0";

/// The game's version name and id, as the servers see them.
const VERSION: &str = crate::resources::KnownPack::VANILLA_VERSION;

/// Downloads packs into a cache folder.
pub struct HttpFetcher {
    cache_dir: PathBuf,
    agent: ureq::Agent,
}

impl HttpFetcher {
    /// A fetcher keeping its files under `cache_dir`, which is made (and trimmed) now.
    pub fn new(cache_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&cache_dir)?;
        cache::vacuum(&cache_dir, cache::MAX_KEPT_PACKS);
        let agent = ureq::Agent::config_builder()
            .timeout_connect(Some(std::time::Duration::from_secs(30)))
            .build()
            .into();
        Ok(Self { cache_dir, agent })
    }

    fn download(&self, request: &DownloadRequest, user: &PackUser) -> Result<PathBuf, String> {
        let folder = self.cache_dir.join(request.id.to_string_java());
        if let Some(hash) = &request.hash
            && let Some(cached) = reuse_cached(&folder.join(hash), hash)
        {
            return Ok(cached);
        }
        let response = self
            .agent
            .get(&request.url)
            .header("X-Minecraft-Username", &user.name)
            .header("X-Minecraft-UUID", undashed(user.id))
            .header("X-Minecraft-Version", VERSION)
            .header("X-Minecraft-Version-ID", VERSION)
            .header("X-Minecraft-Pack-Format", PACK_FORMAT)
            .header("User-Agent", format!("Minecraft Java/{VERSION}"))
            .call()
            .map_err(|error| error.to_string())?;
        let announced = response.body().content_length();
        if let Some(size) = announced.filter(|size| *size > MAX_PACK_BYTES) {
            return Err(format!(
                "file size is bigger than maximum allowed (file is {size}, limit is {MAX_PACK_BYTES})"
            ));
        }
        std::fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
        let mut body = response.into_body().into_reader();
        match &request.hash {
            Some(hash) => fetch_named(&mut body, &folder.join(hash), hash),
            None => fetch_unnamed(&mut body, &folder),
        }
    }
}

impl PackFetcher for HttpFetcher {
    fn fetch(&self, request: &DownloadRequest, user: &PackUser) -> Result<PathBuf, String> {
        self.download(request, user)
    }
}

fn undashed(id: Uuid) -> String {
    format!("{:016x}{:016x}", id.most, id.least)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The SHA-1 of a file, as 40 lowercase hex digits.
fn hash_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            return Ok(hex(&hasher.finalize()));
        }
        hasher.update(&buffer[..read]);
    }
}

/// A file kept from before that has the hash asked for is used again and counts as used now; one
/// that does not is removed.
fn reuse_cached(path: &Path, hash: &str) -> Option<PathBuf> {
    match hash_file(path) {
        Ok(actual) if actual == hash => {
            log::info!("returning cached file since actual hash matches requested");
            if let Ok(file) = std::fs::File::options().write(true).open(path)
                && let Err(error) = file.set_modified(std::time::SystemTime::now())
            {
                log::warn!(
                    "failed to update modification time of {}: {error}",
                    path.display()
                );
            }
            Some(path.to_path_buf())
        }
        Ok(_) => {
            log::warn!("existing file {} had a mismatched hash", path.display());
            let _ = std::fs::remove_file(path);
            None
        }
        Err(_) => None,
    }
}

/// The body written to `path`, and its SHA-1; more than [`MAX_PACK_BYTES`] is an error.
fn save_hashed(body: &mut impl Read, path: &Path) -> Result<String, String> {
    let written = write_hashed(body, path);
    if written.is_err() {
        let _ = std::fs::remove_file(path);
    }
    written
}

fn write_hashed(body: &mut impl Read, path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::create(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 8192];
    let mut total = 0_u64;
    loop {
        let read = body.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            return Ok(hex(&hasher.finalize()));
        }
        total += read as u64;
        if total > MAX_PACK_BYTES {
            return Err(format!(
                "file size was bigger than maximum allowed (got >= {total}, limit was {MAX_PACK_BYTES})"
            ));
        }
        hasher.update(&buffer[..read]);
        file.write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
    }
}

/// A pack whose hash is known: it is saved under the hash and must have it.
fn fetch_named(body: &mut impl Read, target: &Path, hash: &str) -> Result<PathBuf, String> {
    let actual = save_hashed(body, target)?;
    if actual == hash {
        return Ok(target.to_path_buf());
    }
    let _ = std::fs::remove_file(target);
    Err(format!(
        "hash of downloaded file ({actual}) did not match requested ({hash})"
    ))
}

/// A pack whose hash is not known: it is saved aside and named for the hash it has.
fn fetch_unnamed(body: &mut impl Read, folder: &Path) -> Result<PathBuf, String> {
    let temporary = folder.join(format!("download{}.tmp", std::process::id()));
    let actual = save_hashed(body, &temporary)?;
    let target = folder.join(&actual);
    let result = if hash_file(&target).is_ok_and(|existing| existing == actual) {
        std::fs::remove_file(&temporary)
    } else {
        std::fs::rename(&temporary, &target)
    };
    result.map_err(|error| error.to_string())?;
    Ok(target)
}
