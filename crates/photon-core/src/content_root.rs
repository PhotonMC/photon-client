//! Where the game's own assets are (the folder that holds `assets/`: the jar's resources). The
//! development runs name it with `PHOTON_ASSETS`; an installed game has it resolved once at
//! start-up from the content cache. Readers ask here, so the two cannot disagree.

use std::env::VarError;
use std::sync::OnceLock;

static INSTALLED: OnceLock<String> = OnceLock::new();

/// The folder of the version the player switched to, which readers see before the start-up one.
static SELECTED: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// The folder the start-up resolved: `PHOTON_ASSETS` when it is set, else the installed game's.
fn start_up() -> Result<String, VarError> {
    std::env::var("PHOTON_ASSETS").or_else(|error| INSTALLED.get().cloned().ok_or(error))
}

/// The assets folder: the one of the selected version, else the start-up one. It has the type
/// of the environment lookup it replaces.
pub fn assets() -> Result<String, VarError> {
    if let Some(folder) = SELECTED.read().ok().and_then(|selected| selected.clone()) {
        return Ok(folder);
    }
    start_up()
}

/// Make the assets of `version` the ones every reader sees, for a switch of the game version
/// between play sessions. Whether its assets are on disk; when they are not, nothing changes.
pub fn select_version(version: crate::version::Version) -> bool {
    let Some(folder) = assets_for_start_up(version) else {
        return false;
    };
    if let Ok(mut selected) = SELECTED.write() {
        *selected = Some(folder.to_string_lossy().into_owned());
    }
    true
}

/// [`assets_for`], looked up from the start-up folder so a selection does not move the search.
fn assets_for_start_up(version: crate::version::Version) -> Option<std::path::PathBuf> {
    assets_beside(std::path::PathBuf::from(start_up().ok()?), version)
}

/// Record the assets folder of the installed game. The first call wins; whether this one did.
pub fn set_installed_assets(root: String) -> bool {
    INSTALLED.set(root).is_ok()
}

/// The assets folder of one game version, found beside the folder the start-up names: a sibling
/// `jar-<version>` or `jar` whose `version.json` carries that version's name. The newest
/// version falls back to that folder itself, as an installed game keeps only that one. `None`
/// when the version's assets are not on disk, so the version cannot be played.
pub fn assets_for(version: crate::version::Version) -> Option<std::path::PathBuf> {
    assets_for_start_up(version)
}

/// [`assets_for`] with the named folder given.
fn assets_beside(
    base: std::path::PathBuf,
    version: crate::version::Version,
) -> Option<std::path::PathBuf> {
    let container = base.parent()?.parent()?;
    let candidates = [
        container
            .join(format!("jar-{}", version.name()))
            .join("assets"),
        container.join("jar").join("assets"),
        base.clone(),
    ];
    let found = candidates
        .into_iter()
        .find(|folder| folder_version(folder).as_deref() == Some(version.name()));
    found.or_else(|| version.is_newest().then_some(base))
}

/// The release name in the `version.json` beside an `assets` folder.
fn folder_version(assets: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(assets.parent()?.join("version.json")).ok()?;
    let rest = text.split("\"id\"").nth(1)?;
    let rest = rest.split_once(':')?.1.trim_start().strip_prefix('"')?;
    Some(rest.split('"').next()?.to_owned())
}

#[cfg(test)]
mod tests {
    use crate::version::Version;

    fn game(container: &std::path::Path, folder: &str, name: &str) {
        let root = container.join(folder);
        std::fs::create_dir_all(root.join("assets")).expect("assets");
        std::fs::write(
            root.join("version.json"),
            format!("{{\n  \"id\": \"{name}\"\n}}"),
        )
        .expect("json");
    }

    #[test]
    fn each_version_finds_its_own_folder_beside_the_named_one() {
        let dir = std::env::temp_dir().join(format!("photon-assets-for-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        game(&dir, "jar", "26.2");
        game(&dir, "jar-26.3", "26.3");
        let base = dir.join("jar/assets");
        assert_eq!(
            super::assets_beside(base.clone(), Version::V26_2),
            Some(base.clone())
        );
        assert_eq!(
            super::assets_beside(base.clone(), Version::V26_3),
            Some(dir.join("jar-26.3/assets"))
        );
        assert_eq!(super::assets_beside(base, Version::V26_1), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

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
