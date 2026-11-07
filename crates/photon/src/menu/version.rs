//! The game version button of the home screen: the version the next join plays, picked inside
//! the client ([ADR 0013](../../../../docs/adr/0013-multi-version-content-model.md)).

use photon_core::version::Version;
use photon_net::connection::{choose_version, chosen_version};

use super::{Client, Menu, Screen};
use std::sync::atomic::{AtomicBool, Ordering};

/// A join of a picked version is under way: the window keeps showing the last frame the player
/// saw (the home screen) until the new game's first world frame, so a switch shows no
/// connecting or loading screens ([ADR 0013](../../../../docs/adr/0013-multi-version-content-model.md)).
static HOLDING: AtomicBool = AtomicBool::new(false);

/// Whether a join of a picked version is waiting for its first world frame.
pub(crate) fn holding() -> bool {
    HOLDING.load(Ordering::Relaxed)
}

/// The join's wait is over (the world shows, or the join failed).
pub(crate) fn release() {
    HOLDING.store(false, Ordering::Relaxed);
}

/// Start holding the frame when the player picked a version (or `PHOTON_HOLD_JOIN` asks, for
/// scripted runs, which have no picker).
pub(super) fn hold_for_join() {
    let wanted = chosen_version().is_some() || std::env::var_os("PHOTON_HOLD_JOIN").is_some();
    HOLDING.store(wanted, Ordering::Relaxed);
}

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
    /// The frame's polls of the services behind the menus: credits, friends, directory, links and
    /// sign-in.
    pub(super) fn poll_services(&mut self, client: &mut Client, size: (i32, i32)) {
        self.credits_ticked(size);
        self.friends_account_polled();
        self.directory_polled();
        self.link_arrived(client);
        self.link_polled(size);
        self.signin_polled(size);
        self.account_polled(size, client);
        self.friends_ticked(client);
    }

    /// Whether the window keeps its last frame: a picked version's join is under way.
    pub(super) fn holds_frame(&self) -> bool {
        if !matches!(self.screen, Screen::Connecting(..)) {
            release();
        }
        holding()
    }

    /// A press on the version button: the next choice, shown on the button.
    pub(super) fn cycle_version(&mut self) {
        let next = after(chosen_version(), &offered());
        choose_version(next);
        // The picked version's content is baked while the player is still on the menu.
        if let Some(version) = next {
            crate::play::prefetch::start(version);
        }
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
