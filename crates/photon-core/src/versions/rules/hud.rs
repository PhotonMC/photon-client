//! The HUD and the screens the game opens while playing.

use crate::version::Version;

rules! {
    /// The rules of the HUD and its screens (see the module).
    HudRules {
        /// Keypad digits also count as digits for tab shortcuts (new in 26.3).
        keypad_digits_choose_tabs: since V26_3,
        /// The cartography table marks a map with paper as unable to grow when it is not an
        /// extendable map (the item tag) or is at scale 4 or more (26.3); 26.2 looks at the scale
        /// alone. Proved by the `gui/container` `cartography-*` rows.
        cartography_scaling_needs_extendable_map: since V26_3,
        /// The game mode switcher (F3+F4) shows the buried treasure map for adventure mode
        /// (26.3); 26.2 shows the plain map.
        switcher_adventure_icon_is_the_treasure_map: since V26_3,
        /// A subtitle shows the closest place it was heard at (26.2) instead of the latest one
        /// within range (26.3).
        subtitles_show_closest: before V26_3,
        /// The brewing stand screen uses fixed totals (400 ticks, 20 fuel) instead of the ones the
        /// server sends (26.3 sends them as data slots 2 and 3).
        brewing_totals_are_fixed: before V26_3,
        /// The debug entries are those of 26.2 (no `player_speed`; `post_effect` for 26.3's
        /// `post_effects`).
        debug_entries_are_the_older_set: before V26_3,
        /// The fps debug line ends with the display's refresh rate (new in 26.3).
        fps_line_shows_refresh_rate: since V26_3,
        /// The debug footer writes the fps chart in lower case (26.3; 26.2 writes `FPS`).
        debug_footer_fps_lower_case: since V26_3,
        /// The performance debug entry has the terrain rendering line (new in 26.3).
        debug_shows_terrain_rendering: since V26_3,
        /// The system specs debug entry names its runtime and has the window line (26.3; 26.2 says
        /// `Java` and has no window line).
        debug_system_specs_have_window: since V26_3,
        /// The debug options screen has the GUI scale button (`debugGuiScale`, new in 26.3).
        debug_options_have_gui_scale: since V26_3,
        /// The debug options screen's category titles are focusable text widgets, written bold and
        /// underlined (26.3; in 26.2 they are plain text drawn five pixels down and take no focus).
        debug_option_categories_styled: since V26_3,
    }
}
