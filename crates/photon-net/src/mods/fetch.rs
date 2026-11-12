//! Getting a mod: from the cache when it is there, else from the address the server named,
//! checked against the hash before it is kept.

use std::io::Read;
use std::time::Duration;

use super::cache::{CacheError, ModCache, hash_of};
use super::manifest::ModEntry;
use super::sync::Reason;

const TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug)]
pub enum FetchError {
    /// The address did not answer, or answered with an error.
    Unavailable,
    /// The bytes are not the hash the manifest named.
    Mismatch,
    /// More or fewer bytes than the manifest said.
    Size,
    Cache(CacheError),
}

impl FetchError {
    /// The word the hello carries for it.
    pub fn reason(&self) -> Reason {
        match self {
            Self::Unavailable | Self::Cache(CacheError::Io(_)) => Reason::Fetch,
            Self::Mismatch | Self::Cache(CacheError::Mismatch) => Reason::Hash,
            Self::Size => Reason::Size,
        }
    }
}

/// The bytes of `entry`: from `cache` without any network when the hash is there, else fetched,
/// verified and kept. Blocking; the caller runs it off the game's thread.
pub fn ensure(cache: &ModCache, entry: &ModEntry) -> Result<Vec<u8>, FetchError> {
    if let Some(bytes) = cache.get(&entry.hash) {
        return Ok(bytes);
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .http_status_as_error(true)
        .max_redirects(0)
        .build()
        .into();
    let mut response = agent
        .get(&entry.url)
        .call()
        .map_err(|_| FetchError::Unavailable)?;
    let mut bytes = Vec::new();
    // One byte over the size the manifest promised is already a lie.
    response
        .body_mut()
        .as_reader()
        .take(entry.size + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| FetchError::Unavailable)?;
    if bytes.len() as u64 != entry.size {
        return Err(FetchError::Size);
    }
    if hash_of(&bytes) != entry.hash {
        return Err(FetchError::Mismatch);
    }
    cache.put(&entry.hash, &bytes).map_err(FetchError::Cache)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// A server on this machine that answers every request with `body`, and counts them. It stops
    /// when the returned flag handle is dropped by the test ending the process; connections after
    /// `max` are refused by closing the listener.
    fn serve(body: Vec<u8>, max: usize) -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let url = format!("http://{}/m.wasm", listener.local_addr().expect("addr"));
        let count = Arc::new(AtomicUsize::new(0));
        let served = Arc::clone(&count);
        std::thread::spawn(move || {
            for stream in listener.incoming().take(max) {
                let Ok(mut stream) = stream else { return };
                let mut request = [0u8; 1024];
                let _ = stream.read(&mut request);
                served.fetch_add(1, Ordering::SeqCst);
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        (url, count)
    }

    fn entry(url: &str, bytes: &[u8]) -> ModEntry {
        ModEntry {
            id: "m".to_owned(),
            hash: hash_of(bytes),
            size: bytes.len() as u64,
            url: url.to_owned(),
            caps: Vec::new(),
        }
    }

    fn cache(name: &str) -> ModCache {
        let dir = std::env::temp_dir().join(format!("photon-fetch-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        ModCache::new(dir)
    }

    #[test]
    fn the_second_join_reads_the_cache_and_asks_nobody() {
        let (url, count) = serve(b"module-one".to_vec(), 1);
        let (cache, wanted) = (cache("hit"), entry(&url, b"module-one"));
        assert_eq!(ensure(&cache, &wanted).expect("first"), b"module-one");
        assert_eq!(count.load(Ordering::SeqCst), 1);
        // The server took its one request and is gone; the cache answers.
        assert_eq!(ensure(&cache, &wanted).expect("second"), b"module-one");
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn an_update_is_a_new_hash_and_is_fetched() {
        let (old_url, _) = serve(b"v1".to_vec(), 1);
        let (new_url, new_count) = serve(b"v2-longer".to_vec(), 1);
        let cache = cache("update");
        ensure(&cache, &entry(&old_url, b"v1")).expect("v1");
        let bytes = ensure(&cache, &entry(&new_url, b"v2-longer")).expect("v2");
        assert_eq!(bytes, b"v2-longer");
        assert_eq!(new_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn bytes_that_are_not_the_hash_are_refused_and_not_kept() {
        let (url, _) = serve(b"tampered!".to_vec(), 1);
        let cache = cache("mismatch");
        let wanted = entry(&url, b"original!");
        assert!(matches!(ensure(&cache, &wanted), Err(FetchError::Mismatch)));
        assert!(cache.get(&wanted.hash).is_none());
    }

    #[test]
    fn a_body_longer_than_promised_is_refused() {
        let (url, _) = serve(vec![7; 100], 1);
        let mut wanted = entry(&url, &[7; 100]);
        wanted.size = 10;
        assert!(matches!(
            ensure(&cache("long"), &wanted),
            Err(FetchError::Size)
        ));
    }
}
