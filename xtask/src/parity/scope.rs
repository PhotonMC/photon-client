//! Which parity cases a version is compared on.
//!
//! Only the newest version is compared on every case. For an older version the menus (main
//! menu, options screens, server list and the like) are out of scope: they behave as the newest
//! version for every version, so only the in-game cases are compared.

/// The gui case ids that are out of scope for older versions: exact ids, or prefixes when they
/// end in `/`.
const MENU_CASES: [&str; 30] = [
    "gui/accessibility-options-screen",
    "gui/chat-options-keys",
    "gui/chat-options-screen",
    "gui/connect-screen",
    "gui/controls-keys",
    "gui/controls-screen",
    "gui/credits-screen",
    "gui/font-options-screen",
    "gui/game-rules-screen",
    "gui/join-multiplayer-screen",
    "gui/key-binds-screen",
    "gui/key-mappings",
    "gui/language-screen",
    "gui/loading-overlay",
    "gui/mouse-settings-keys",
    "gui/mouse-settings-screen",
    "gui/online-options-screen",
    "gui/options-screen",
    "gui/pack-selection-screen",
    "gui/panorama-view/",
    "gui/restrictions-screen",
    "gui/server-form",
    "gui/server-list-icons",
    "gui/server-list-pings",
    "gui/skin-customization-screen",
    "gui/sound-options-screen",
    "gui/video-keys",
    "gui/video-settings-screen",
    "gui/world-options-screen",
    "gui/friends-overlay",
];

/// Whether `case_id` is compared for a version; `newest` is whether that version is the newest.
pub fn in_scope(case_id: &str, newest: bool) -> bool {
    newest
        || !MENU_CASES.iter().any(|menu| {
            if menu.ends_with('/') {
                case_id.starts_with(menu)
            } else {
                case_id == *menu
            }
        })
}

#[cfg(test)]
mod tests {
    use super::in_scope;

    #[test]
    fn the_newest_version_keeps_every_case() {
        assert!(in_scope("gui/options-screen", true));
        assert!(in_scope("gui/panorama-view/0", true));
    }

    #[test]
    fn older_versions_drop_the_menus_only() {
        assert!(!in_scope("gui/options-screen", false));
        assert!(!in_scope("gui/panorama-view/0", false));
        assert!(!in_scope("gui/friends-overlay", false));
        assert!(in_scope("gui/hotbar", false));
        assert!(in_scope("movement/walk", false));
    }
}
