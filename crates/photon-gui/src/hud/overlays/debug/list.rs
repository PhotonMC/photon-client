//! The statuses of the debug screen's entries (`DebugScreenEntryList`): which are shown always,
//! only while F3's overlay is up, or never, and the sorted list of those on now.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    AlwaysOn,
    InOverlay,
    Never,
}

/// The default profile (`DebugScreenEntries.PROFILES`): the entries of its map, all in the overlay.
const DEFAULT_PROFILE: [&str; 9] = [
    "3d_crosshair",
    "game_version",
    "tps",
    "fps",
    "memory",
    "system_specs",
    "player_position",
    "player_section_position",
    "simple_performance_impactors",
];

/// `DebugScreenEntry.isAllowed`: the entries that stay with reduced debug info (the version,
/// the performance lines and the section-relative position); the position's coordinates and the
/// 3D crosshair do not.
fn allowed(id: &str, reduced_debug_info: bool) -> bool {
    !reduced_debug_info
        || matches!(
            id,
            "game_version"
                | "player_section_position"
                | "fps"
                | "tps"
                | "memory"
                | "system_specs"
                | "simple_performance_impactors"
        )
}

#[derive(Debug, Clone)]
pub struct EntryList {
    statuses: BTreeMap<&'static str, Status>,
    overlay_visible: bool,
}

impl Default for EntryList {
    fn default() -> Self {
        Self {
            statuses: DEFAULT_PROFILE
                .iter()
                .map(|id| (*id, Status::InOverlay))
                .collect(),
            overlay_visible: false,
        }
    }
}

impl EntryList {
    pub fn is_overlay_visible(&self) -> bool {
        self.overlay_visible
    }

    /// `toggleDebugOverlay`.
    pub fn toggle_overlay(&mut self) {
        self.overlay_visible = !self.overlay_visible;
    }

    pub fn set_overlay_visible(&mut self, visible: bool) {
        self.overlay_visible = visible;
    }

    /// `getCurrentlyEnabled`: the ids shown now, sorted by their paths as `Identifier`'s natural
    /// order does (the map is sorted by them).
    pub fn enabled(&self, reduced_debug_info: bool) -> Vec<&'static str> {
        self.statuses
            .iter()
            .filter(|(id, status)| {
                (**status == Status::AlwaysOn
                    || self.overlay_visible && **status == Status::InOverlay)
                    && allowed(id, reduced_debug_info)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// `showDebugScreen`: the overlay is up or an entry is on.
    pub fn any_shown(&self, reduced_debug_info: bool) -> bool {
        self.overlay_visible || !self.enabled(reduced_debug_info).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_profile_shows_only_in_the_overlay_and_sorted() {
        let mut list = EntryList::default();
        assert!(list.enabled(false).is_empty());
        list.toggle_overlay();
        assert_eq!(
            list.enabled(false),
            [
                "3d_crosshair",
                "fps",
                "game_version",
                "memory",
                "player_position",
                "player_section_position",
                "simple_performance_impactors",
                "system_specs",
                "tps"
            ]
        );
    }

    #[test]
    fn reduced_debug_info_drops_the_position_and_the_crosshair() {
        let mut list = EntryList::default();
        list.toggle_overlay();
        assert_eq!(
            list.enabled(true),
            [
                "fps",
                "game_version",
                "memory",
                "player_section_position",
                "simple_performance_impactors",
                "system_specs",
                "tps"
            ]
        );
    }
}
