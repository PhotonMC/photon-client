//! Where the game's own assets are (the folder that holds `assets/`: the jar's resources). The
//! development runs name it with `PHOTON_ASSETS`; an installed game has it resolved once at
//! start-up from the content cache. Readers ask here, so the two cannot disagree.

use std::env::VarError;
use std::sync::OnceLock;

static INSTALLED: OnceLock<String> = OnceLock::new();

/// The assets folder: `PHOTON_ASSETS` when it is set, else the one the start-up resolved. It
/// has the type of the environment lookup it replaces.
pub fn assets() -> Result<String, VarError> {
    std::env::var("PHOTON_ASSETS").or_else(|error| INSTALLED.get().cloned().ok_or(error))
}

/// Record the assets folder of the installed game. The first call wins; whether this one did.
pub fn set_installed_assets(root: String) -> bool {
    INSTALLED.set(root).is_ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_installed_folder_answers_when_the_environment_does_not() {
        if std::env::var_os("PHOTON_ASSETS").is_some() {
            assert_eq!(super::assets(), std::env::var("PHOTON_ASSETS"));
            return;
        }
        assert!(super::assets().is_err());
        assert!(super::set_installed_assets("/cache/jar/assets".to_owned()));
        assert_eq!(super::assets().as_deref(), Ok("/cache/jar/assets"));
        assert!(!super::set_installed_assets("/other".to_owned()));
    }
}
