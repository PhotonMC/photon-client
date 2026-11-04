//! Getting the file: `https` (or this machine), no redirects, a size cap.

use std::io::Read;
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

/// The bytes at `url`. Blocking; the caller runs it off the game's thread.
pub fn fetch(url: &str) -> Result<Vec<u8>, VideoError> {
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
    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(MAX_VIDEO_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| VideoError::Unavailable)?;
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
