//! Getting a mod: from the cache when it is there, else from the address the server named,
//! checked against the hash before it is kept.

use std::io::Read;
use std::time::Duration;

use super::cache::{CacheError, ModCache, hash_of};
use super::manifest::ModEntry;
use super::sync::Reason;

const TIMEOUT: Duration = Duration::from_secs(20);
/// The bytes read between two looks at whether the file is still wanted.
const CHUNK: usize = 32 << 10;

#[derive(Debug)]
pub enum FetchError {
    /// The address did not answer, or answered with an error.
    Unavailable,
    /// The bytes are not the hash the manifest named.
    Mismatch,
    /// More or fewer bytes than the manifest said.
    Size,
    /// Nobody wants the file any more; the fetch stopped early.
    Cancelled,
    Cache(CacheError),
}

impl FetchError {
    /// The word the hello carries for it.
    pub fn reason(&self) -> Reason {
        match self {
            Self::Unavailable | Self::Cancelled | Self::Cache(CacheError::Io(_)) => Reason::Fetch,
            Self::Mismatch | Self::Cache(CacheError::Mismatch) => Reason::Hash,
            Self::Size => Reason::Size,
        }
    }
}

/// The bytes of `entry`: from `cache` without any network when the hash is there, else fetched,
/// verified and kept. Blocking; the caller runs it off the game's thread.
pub fn ensure(cache: &ModCache, entry: &ModEntry) -> Result<Vec<u8>, FetchError> {
    ensure_while(cache, entry, &|| true)
}

/// [`ensure`] that stops with [`FetchError::Cancelled`] as soon as `wanted` says nobody needs
/// the file: before the request and between the chunks of the body, so a manifest that was
/// replaced wastes at most one chunk.
pub fn ensure_while(
    cache: &ModCache,
    entry: &ModEntry,
    wanted: &dyn Fn() -> bool,
) -> Result<Vec<u8>, FetchError> {
    if let Some(bytes) = cache.get(&entry.hash) {
        return Ok(bytes);
    }
    if !wanted() {
        return Err(FetchError::Cancelled);
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
    let mut body = response.body_mut().as_reader().take(entry.size + 1);
    let mut chunk = [0u8; CHUNK];
    loop {
        if !wanted() {
            return Err(FetchError::Cancelled);
        }
        match body.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => bytes.extend_from_slice(&chunk[..read]),
            Err(_) => return Err(FetchError::Unavailable),
        }
    }
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

    /// A server that sends `total` bytes slowly, in pieces, so a client can stop in the middle.
    fn serve_slowly(total: usize) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let url = format!("http://{}/m.wasm", listener.local_addr().expect("addr"));
        std::thread::spawn(move || {
            let Some(Ok(mut stream)) = listener.incoming().next() else {
                return;
            };
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let head =
                format!("HTTP/1.1 200 OK\r\nContent-Length: {total}\r\nConnection: close\r\n\r\n");
            let _ = stream.write_all(head.as_bytes());
            for _ in 0..total / 65536 {
                if stream.write_all(&[1u8; 65536]).is_err() {
                    return;
                }
                std::thread::sleep(Duration::from_millis(30));
            }
        });
        url
    }

    #[test]
    fn a_file_nobody_wants_is_not_requested_and_not_kept() {
        let (url, count) = serve(b"module".to_vec(), 1);
        let (cache, wanted) = (cache("unwanted"), entry(&url, b"module"));
        let result = ensure_while(&cache, &wanted, &|| false);
        assert!(matches!(result, Err(FetchError::Cancelled)));
        assert_eq!(count.load(Ordering::SeqCst), 0);
        assert!(cache.get(&wanted.hash).is_none());
    }

    #[test]
    fn a_fetch_stops_in_the_middle_of_the_body_when_the_file_stops_being_wanted() {
        let total = 1 << 20;
        let url = serve_slowly(total);
        let mut wanted = entry(&url, &vec![1u8; total]);
        wanted.size = total as u64;
        let looks = AtomicUsize::new(0);
        // Wanted for the request and the first chunks, then not any more.
        let still = || looks.fetch_add(1, Ordering::SeqCst) < 4;
        let cache = cache("midway");
        let result = ensure_while(&cache, &wanted, &still);
        assert!(matches!(result, Err(FetchError::Cancelled)));
        // The whole body is 32 chunks; it stopped long before.
        assert!(looks.load(Ordering::SeqCst) <= 8);
        assert!(cache.get(&wanted.hash).is_none());
    }

    #[test]
    fn a_cached_file_is_returned_even_if_nobody_asks_for_it_any_more() {
        let (cache, wanted) = (
            cache("cached-unwanted"),
            entry("http://127.0.0.1:9/x", b"m"),
        );
        cache.put(&wanted.hash, b"m").expect("put");
        assert_eq!(ensure_while(&cache, &wanted, &|| false).expect("hit"), b"m");
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
