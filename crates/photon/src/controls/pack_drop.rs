//! Files dropped on the Resource Packs screen (`PackSelectionScreen.onFilesDrop`): the player is
//! asked, and the paths that are packs are copied into the pack folder.

use std::path::{Path, PathBuf};

use photon_assets::pack_directory;
use photon_game::text::Component;
use photon_gui::screen::confirm::ConfirmBackground;

use super::unattended::ConfirmTarget;
use super::*;

/// `PackDetector`: a directory with a regular `pack.mcmeta`, or a regular `.zip` file. Links are
/// followed by the metadata calls, like the reference's validated paths.
fn is_pack(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if meta.is_dir() {
        return std::fs::metadata(path.join("pack.mcmeta")).is_ok_and(|meta| meta.is_file());
    }
    meta.is_file()
        && path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
}

/// `PackSelectionScreen.copyPacks` for one pack: a file, or a directory with everything under it.
fn copy_pack(from: &Path, folder: &Path) -> std::io::Result<()> {
    let Some(name) = from.file_name() else {
        return Ok(());
    };
    copy_tree(from, &folder.join(name))
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    if std::fs::metadata(from)?.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else {
        std::fs::copy(from, to)?;
    }
    Ok(())
}

/// The dropped paths that are packs are copied into `folder`; the ones that are not, and the
/// failures, are returned.
pub(super) fn copy_dropped(paths: &[PathBuf], folder: &Path) -> (Vec<PathBuf>, Vec<String>) {
    let (packs, leftovers): (Vec<_>, Vec<_>) =
        paths.iter().cloned().partition(|path| is_pack(path));
    let mut failures = Vec::new();
    let made = if packs.is_empty() {
        Ok(())
    } else {
        std::fs::create_dir_all(folder)
    };
    failures.extend(made.err().map(|error| error.to_string()));
    for pack in &packs {
        if let Err(error) = copy_pack(pack, folder) {
            failures.push(error.to_string());
        }
    }
    (leftovers, failures)
}

impl Controls {
    /// `onFilesDrop`: a confirmation naming the files.
    pub(super) fn ask_to_copy_packs(
        &mut self,
        paths: Vec<PathBuf>,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        let names: Vec<String> = paths
            .iter()
            .filter_map(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .collect();
        let screen = ConfirmScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            &Component::translatable("pack.dropConfirm"),
            &Component::literal(names.join(", ")),
            &Component::translatable("gui.yes"),
            &Component::translatable("gui.no"),
        )
        .with_background(ConfirmBackground::Menu {
            in_world: !self.title_host,
        });
        self.link_confirm = Some((screen, ConfirmTarget::Drop(paths)));
    }

    /// The player agreed: the packs are copied and the screen lists the folder again.
    pub(super) fn copy_confirmed_packs(&mut self, paths: &[PathBuf], assets: &ScreenAssets) {
        let folder = pack_directory(&game_directory());
        let (leftovers, failures) = copy_dropped(paths, &folder);
        for path in &leftovers {
            log::warn!("Path {} does not seem like pack", path.display());
        }
        for failure in &failures {
            log::warn!("Failed to copy the dropped packs: {failure}");
        }
        self.reload_pack_screen(assets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("photon-drop-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn zips_and_directories_with_metadata_are_copied() {
        let source = temp("source");
        let folder = temp("folder").join("resourcepacks");
        std::fs::write(source.join("Fancy.zip"), b"zip").unwrap();
        std::fs::write(source.join("notes.txt"), b"text").unwrap();
        let tree = source.join("Tree");
        std::fs::create_dir_all(tree.join("assets/minecraft")).unwrap();
        std::fs::write(tree.join("pack.mcmeta"), b"{}").unwrap();
        std::fs::write(tree.join("assets/minecraft/a.txt"), b"a").unwrap();
        let bare = source.join("Bare");
        std::fs::create_dir_all(&bare).unwrap();
        let paths = vec![
            source.join("Fancy.zip"),
            source.join("notes.txt"),
            tree,
            bare.clone(),
        ];
        let (leftovers, failures) = copy_dropped(&paths, &folder);
        assert!(failures.is_empty());
        assert_eq!(leftovers, vec![source.join("notes.txt"), bare]);
        assert!(folder.join("Fancy.zip").is_file());
        assert!(folder.join("Tree/pack.mcmeta").is_file());
        assert!(folder.join("Tree/assets/minecraft/a.txt").is_file());
        assert!(!folder.join("notes.txt").exists());
    }
}
