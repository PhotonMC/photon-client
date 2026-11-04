//! The name of the player last signed in, kept as a plain file beside `links.txt` so a website
//! sign-in question can name them at once instead of waiting for the account services. It is a
//! hint, not an account: no token is in it, and what signs the website in is still the resumed
//! account, which must carry the name that was shown.

use std::path::PathBuf;

fn path() -> Option<PathBuf> {
    let cache = photon_content::Cache::user_level().ok()?;
    Some(cache.root().parent()?.join("signed_in.txt"))
}

/// Whether `text` can be a player name: what the question prints must be one word of the
/// characters a name has.
fn valid(text: &str) -> bool {
    (1..=16).contains(&text.len()) && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The name kept, if there is a valid one.
pub(crate) fn read() -> Option<String> {
    let text = std::fs::read_to_string(path()?).ok()?;
    let name = text.trim();
    valid(name).then(|| name.to_owned())
}

/// Keep `name` as the signed-in player. A failure to write only costs the speed-up.
pub(crate) fn write(name: &str) {
    if let (Some(path), true) = (path(), valid(name)) {
        let _ = std::fs::write(path, name);
    }
}

/// Forget the name: nobody is signed in.
pub(crate) fn clear() {
    if let Some(path) = path() {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::valid;

    #[test]
    fn only_a_player_name_is_shown() {
        assert!(valid("Miate_1"));
        assert!(!valid(""));
        assert!(!valid("two words"));
        assert!(!valid("12345678901234567"));
        assert!(!valid("%s\n"));
    }
}
