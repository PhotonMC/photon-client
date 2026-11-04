//! Getting the file: `https` (or this machine), no redirects, a size cap.

use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::VideoError;

/// The largest file taken, in bytes.
pub const MAX_VIDEO_BYTES: u64 = 32 << 20;

const TIMEOUT: Duration = Duration::from_secs(60);

/// Whether `url` may be fetched: `https`, or `http` to this machine.
fn allowed(url: &str) -> bool {
    url.starts_with("https://")
        || ["http://127.0.0.1", "http://localhost", "http://[::1]"]
            .iter()
            .any(|prefix| {
                url.strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with([':', '/']) || rest.is_empty())
            })
}

/// The bytes at `url`, read in chunks so that `stop` ends the download at once. Blocking; the
/// caller runs it off the game's thread.
pub fn fetch(url: &str, stop: &AtomicBool) -> Result<Vec<u8>, VideoError> {
    if !allowed(url) {
        return Err(VideoError::Unavailable);
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .http_status_as_error(true)
        .max_redirects(0)
        .build()
        .into();
    let mut response = agent.get(url).call().map_err(|_| VideoError::Unavailable)?;
    let mut reader = response.body_mut().as_reader().take(MAX_VIDEO_BYTES + 1);
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 64 << 10];
    loop {
        if stop.load(Ordering::Relaxed) {
            return Err(VideoError::Unavailable);
        }
        let read = reader
            .read(&mut chunk)
            .map_err(|_| VideoError::Unavailable)?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    if bytes.len() as u64 > MAX_VIDEO_BYTES {
        return Err(VideoError::TooBig);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_and_this_machine_are_allowed() {
        assert!(allowed("https://example.com/a.webm"));
        assert!(allowed("http://127.0.0.1:8765/a.webm"));
        assert!(allowed("http://localhost/a.webm"));
        assert!(!allowed("http://example.com/a.webm"));
        assert!(!allowed("http://127.0.0.1.evil.com/a.webm"));
        assert!(!allowed("file:///etc/passwd"));
        assert!(!allowed("ftp://127.0.0.1/a"));
    }
}
