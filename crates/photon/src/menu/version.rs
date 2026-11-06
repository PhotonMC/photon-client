//! The game version button of the home screen: the version the next join plays, picked inside
//! the client ([ADR 0013](../../../../docs/adr/0013-multi-version-content-model.md)).

use photon_core::version::Version;
use photon_net::connection::{choose_version, chosen_version};

use super::{Menu, Screen};

/// The versions the button offers, after automatic: the newest first, those whose assets are on
/// disk.
fn offered() -> Vec<Version> {
    Version::ALL
        .iter()
        .rev()
        .copied()
        .filter(|version| photon_core::content_root::assets_for(*version).is_some())
        .collect()
}

/// The choice after `current`: automatic, then each offered version, then automatic again.
fn after(current: Option<Version>, offered: &[Version]) -> Option<Version> {
    match current {
        None => offered.first().copied(),
        Some(version) => {
            let at = offered.iter().position(|offered| *offered == version)?;
            offered.get(at + 1).copied()
        }
    }
}

/// The text of the version button: the picked version, or automatic.
pub(super) fn label(lang: &photon_game::text::Lang) -> String {
    match chosen_version() {
        Some(version) => version.name().to_owned(),
        None => lang.text("photon.home.version.auto"),
    }
}

impl Menu {
    /// A press on the version button: the next choice, shown on the button.
    pub(super) fn cycle_version(&mut self) {
        choose_version(after(chosen_version(), &offered()));
        let label = label(self.gui.lang());
        let font = self.gui.text_assets().0;
        if let Screen::Home(home) = &mut self.screen {
            home.set_game_version(&label, font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_button_walks_automatic_then_each_version_then_automatic() {
        let offered = [Version::V26_3, Version::V26_2];
        assert_eq!(after(None, &offered), Some(Version::V26_3));
        assert_eq!(after(Some(Version::V26_3), &offered), Some(Version::V26_2));
        assert_eq!(after(Some(Version::V26_2), &offered), None);
        assert_eq!(after(None, &[]), None);
    }
}
