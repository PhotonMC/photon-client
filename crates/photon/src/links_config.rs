//! Where the backend and the website are: the player's own `PHOTON_BACKEND` and `PHOTON_WEBSITE`
//! first, then the `links.txt` beside the content cache (`backend=` and `website=` lines), so a
//! client the browser starts, which has no environment of its own, still knows them. There is no
//! built-in address, and a link never names one: the code it carries is only spent where the
//! player's own settings say.

use std::path::PathBuf;

use photon_net::backend::{Backend, BackendError};

/// The value for `key`: the environment's when it is not empty, else the file's line.
fn resolve(environment: Option<&str>, file: Option<&str>, key: &str) -> Option<String> {
    if let Some(value) = environment.filter(|value| !value.is_empty()) {
        return Some(value.to_owned());
    }
    file?.lines().find_map(|line| {
        let (name, value) = line.split_once('=')?;
        let value = value.trim();
        (name.trim() == key && !value.is_empty()).then(|| value.to_owned())
    })
}

fn file_text() -> Option<String> {
    let cache = photon_content::Cache::user_level().ok()?;
    let path: PathBuf = cache.root().parent()?.join("links.txt");
    std::fs::read_to_string(path).ok()
}

fn setting(variable: &str, key: &str) -> Option<String> {
    let environment = std::env::var(variable).ok();
    resolve(environment.as_deref(), file_text().as_deref(), key)
}

/// The backend the player named; none keeps the client as it is. An address that is not secure is
/// an error, whichever source named it.
pub(crate) fn backend() -> Option<Result<Backend, BackendError>> {
    setting("PHOTON_BACKEND", "backend").map(|address| Backend::new(&address))
}

/// The website the player named, to be checked where it is used.
pub(crate) fn website() -> Option<String> {
    setting("PHOTON_WEBSITE", "website")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "# links\n\nbackend = https://backend.example\nwebsite=https://site.example\nunknown=1\nnonsense\n";

    #[test]
    fn the_environment_overrides_the_file() {
        let got = resolve(Some("http://127.0.0.1:1"), Some(FILE), "backend");
        assert_eq!(got.as_deref(), Some("http://127.0.0.1:1"));
    }

    #[test]
    fn the_file_is_used_when_the_environment_is_absent_or_empty() {
        for environment in [None, Some("")] {
            let got = resolve(environment, Some(FILE), "website");
            assert_eq!(got.as_deref(), Some("https://site.example"));
        }
    }

    #[test]
    fn blank_unknown_and_empty_lines_are_ignored() {
        assert_eq!(resolve(None, Some(FILE), "missing"), None);
        assert_eq!(resolve(None, Some("backend=\n"), "backend"), None);
        assert_eq!(resolve(None, None, "backend"), None);
    }

    #[test]
    fn an_insecure_address_from_the_file_is_refused() {
        let file = resolve(None, Some("backend=http://backend.example\n"), "backend").unwrap();
        assert!(matches!(Backend::new(&file), Err(BackendError::Insecure)));
    }
}
