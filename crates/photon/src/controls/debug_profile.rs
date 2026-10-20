//! `debug-profile.json` of the game directory (`DebugScreenEntryList.load` and `save`).

use photon_gui::hud::debug::{EntryList, parse_profile, profile_text};

use super::*;

fn debug_profile_path() -> std::path::PathBuf {
    game_directory().join("debug-profile.json")
}

fn write_debug_profile(list: &EntryList) {
    let _ = std::fs::create_dir_all(game_directory());
    let _ = std::fs::write(
        debug_profile_path(),
        profile_text(list, photon_game::options::DATA_VERSION),
    );
}

/// The entries the file sets: the default profile when there is no file (not written), and when the
/// file cannot be read (written then).
pub(super) fn load_debug_entries() -> EntryList {
    let Ok(text) = std::fs::read_to_string(debug_profile_path()) else {
        return EntryList::default();
    };
    parse_profile(&text).unwrap_or_else(|| {
        let list = EntryList::default();
        write_debug_profile(&list);
        list
    })
}

impl Controls {
    /// `DebugScreenEntryList.save`, whether or not a status changed (the profile buttons).
    pub(super) fn save_debug_profile_now(&mut self) {
        self.debug.entries.take_changed();
        write_debug_profile(&self.debug.entries);
    }

    /// `DebugScreenEntryList.save`, after a status was set.
    pub(super) fn save_debug_profile(&mut self) {
        if self.debug.entries.take_changed() {
            write_debug_profile(&self.debug.entries);
        }
    }
}
