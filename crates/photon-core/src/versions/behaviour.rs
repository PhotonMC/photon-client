//! Behaviour questions whose answer depends on the version the session plays. Every one reads
//! the active version, so a process holds one version at a time.

use crate::version::Version;

/// Whether a negative element count in a tag list is refused, because the list is sized from
/// the count (26.3; before it the loop simply reads nothing).
pub fn negative_tag_counts_fail() -> bool {
    !Version::active_before_26_3()
}

/// Whether the game has the improved transparency option and its debug key (new in 26.3).
pub fn has_improved_transparency() -> bool {
    !Version::active_before_26_3()
}

/// Whether the name tag text is the fixed half-transparent white instead of a colour derived
/// from the background opacity (26.2 submits a constant; 26.3 derives it).
pub fn name_tag_text_is_constant() -> bool {
    Version::active_before_26_3()
}

/// Whether the custom statistic exists in the active version (the straw bed one is new in 26.3).
pub fn custom_stat_exists(name: &str) -> bool {
    !name.ends_with(":sleep_in_straw_bed") || !Version::active_before_26_3()
}

/// Whether keypad digits also count as digits for tab shortcuts (new in 26.3).
pub fn keypad_digits_choose_tabs() -> bool {
    !Version::active_before_26_3()
}

/// Whether a subtitle shows the closest place it was heard at (26.2) instead of the latest one
/// within range (26.3).
pub fn subtitles_show_closest() -> bool {
    Version::active_before_26_3()
}

/// Whether the brewing stand screen uses fixed totals (400 ticks, 20 fuel) instead of the ones the
/// server sends (26.3 sends them as data slots 2 and 3).
pub fn brewing_totals_are_fixed() -> bool {
    Version::active_before_26_3()
}

/// Whether a unifont glyph advances by half its width in whole pixels (26.3) instead of exactly
/// half (26.2).
pub fn unihex_advance_is_whole() -> bool {
    !Version::active_before_26_3()
}

/// Whether a shulker box tooltip line names the count before the item (26.3) or the item before
/// the count (26.2).
pub fn container_line_count_first() -> bool {
    !Version::active_before_26_3()
}

/// The number of command argument parser types the active version registers: 26.3 added five
/// after the template and heightmap parsers, moving the UUID one from 56 to 61.
pub fn command_parser_count() -> i32 {
    if Version::active_before_26_3() {
        57
    } else {
        62
    }
}

/// The id of the UUID command argument parser (the last one registered).
pub fn command_uuid_parser_id() -> i32 {
    command_parser_count() - 1
}

/// Whether the level events for a splashed potion and a waxed block sound here as well as showing
/// their particles (26.2); in 26.3 the sounds are their own events.
pub fn splash_and_wax_events_sound() -> bool {
    Version::active_before_26_3()
}

/// Whether the client swings for a click that mounts a minecart or opens a grown wandering
/// trader's trades (26.2); in 26.3 only the server's answer swings.
pub fn mount_and_trade_clicks_swing() -> bool {
    Version::active_before_26_3()
}

/// Whether a camel consumes a click it has no use for (26.2); 26.3 lets it go on to the held item.
pub fn camel_consumes_idle_click() -> bool {
    Version::active_before_26_3()
}

/// Whether a block being broken sounds its hit every fourth tick of the breaking (26.2); 26.3
/// sounds it once when the breaking starts.
pub fn hit_sound_repeats() -> bool {
    Version::active_before_26_3()
}

/// Whether an attack on an entity in creative waits out the next block break (26.3).
pub fn creative_attack_delays_breaking() -> bool {
    !Version::active_before_26_3()
}

/// Whether the atmosphere's fog colour is worked out on packed 8-bit colours (26.2); 26.3 keeps
/// float vectors.
pub fn fog_colour_is_packed() -> bool {
    Version::active_before_26_3()
}

/// Whether a fishing bobber falls by 0.03 as a double (26.2); 26.3 widens the float 0.03.
pub fn bobber_gravity_is_double() -> bool {
    Version::active_before_26_3()
}

/// Whether the dragon head's outline is half a pixel taller than the other heads' (26.3).
pub fn dragon_head_outline_is_tall() -> bool {
    !Version::active_before_26_3()
}

/// Whether the resin brick wall is declared solid whatever its shape (26.3).
pub fn resin_brick_wall_is_forced_solid() -> bool {
    !Version::active_before_26_3()
}

/// Whether `blocksMotion` is plain solidity (26.3); 26.2 says no for cobweb and bamboo sapling.
pub fn blocks_motion_is_solidity() -> bool {
    !Version::active_before_26_3()
}

/// Whether the motion blocking heightmap leaves out signs, cobweb and bamboo sapling by its own
/// list (26.3); 26.2 counts whatever blocks motion.
pub fn motion_heightmap_has_own_list() -> bool {
    !Version::active_before_26_3()
}

/// Whether a block can block the view only at the camera's near plane (26.3: stairs and slabs
/// the player's eye sits inside); 26.2 has no such rule, a block blocks the view or it does not.
pub fn blocks_view_at_near_plane() -> bool {
    !Version::active_before_26_3()
}

/// Whether `Mth.length(float, float)` sums its squares as floats (26.3); 26.2 sums them as doubles.
pub fn length_sums_squares_as_floats() -> bool {
    !Version::active_before_26_3()
}

/// Whether a boat clamps every passenger the same way (26.2): body turned to the boat, yaw held
/// within a quarter turn, previous yaw moved along, head set to the yaw. 26.3 treats mobs apart.
pub fn boat_clamp_is_uniform() -> bool {
    Version::active_before_26_3()
}

/// The data version the active version's files carry (`world_version` of its `version.json`).
pub fn data_version() -> i32 {
    if Version::active_before_26_3() {
        4903
    } else {
        5023
    }
}

/// Whether the debug entries are those of 26.2 (no `player_speed`; `post_effect` for 26.3's
/// `post_effects`).
pub fn debug_entries_before_26_3() -> bool {
    Version::active_before_26_3()
}

/// Whether the fps debug line ends with the display's refresh rate (new in 26.3).
pub fn fps_line_shows_refresh_rate() -> bool {
    !Version::active_before_26_3()
}

/// Whether the debug footer writes the fps chart in lower case (26.3; 26.2 writes `FPS`).
pub fn debug_footer_fps_lower_case() -> bool {
    !Version::active_before_26_3()
}

/// Whether the performance debug entry has the terrain rendering line (new in 26.3).
pub fn debug_shows_terrain_rendering() -> bool {
    !Version::active_before_26_3()
}

/// Whether the system specs debug entry names its runtime and has the window line (26.3; 26.2
/// says `Java` and has no window line).
pub fn debug_system_specs_have_window() -> bool {
    !Version::active_before_26_3()
}

/// Whether the debug options screen has the GUI scale button (`debugGuiScale`, new in 26.3).
pub fn debug_options_have_gui_scale() -> bool {
    !Version::active_before_26_3()
}

/// Whether the debug options screen's category titles are focusable text widgets, written bold
/// and underlined (26.3; in 26.2 they are plain text drawn five pixels down and take no focus).
pub fn debug_option_categories_styled() -> bool {
    !Version::active_before_26_3()
}

/// Whether an enum read by ordinal takes the first constant when the id is out of range
/// (26.3's `OutOfBoundsStrategy.ZERO`; before it the read fails).
pub fn out_of_range_enum_ids_read_as_first() -> bool {
    !Version::active_before_26_3()
}

/// Whether a bed item sets its block with its own flags (26: no neighbour update; before 26.3,
/// whose bed item has no override and uses the block item's).
pub fn bed_item_sets_own_flags() -> bool {
    Version::active_before_26_3()
}

/// Whether every tooltip leaves a gap after its first line, not only an item's (before 26.3).
pub fn every_tooltip_gaps_after_first_line() -> bool {
    Version::active_before_26_3()
}

/// The resource pack format `(major, minor)` of the game the session plays (26.1 shares 26.2's
/// until its jar is read).
pub fn resource_pack_format() -> (i32, i32) {
    if Version::active_before_26_3() {
        (88, 0)
    } else {
        (97, 1)
    }
}

thread_local! {
    /// The version of the assets this thread reads, when that is not the version being played
    /// (a bake of another version's blocks ahead of a join).
    static ASSET_VERSION: std::cell::Cell<Option<Version>> = const { std::cell::Cell::new(None) };
}

/// Run `read` with the assets of `version` as the ones this thread reads, whatever version is
/// being played.
pub fn with_asset_version<R>(version: Option<Version>, read: impl FnOnce() -> R) -> R {
    let before = ASSET_VERSION.with(|cell| cell.replace(version));
    let result = read();
    ASSET_VERSION.with(|cell| cell.set(before));
    result
}

/// Whether block model elements carry the boolean `shade` (before 26.3, which replaced it with
/// `shade_direction_override`). Follows the assets being read, else the version played.
pub fn model_shade_is_boolean() -> bool {
    ASSET_VERSION
        .with(std::cell::Cell::get)
        .map_or_else(Version::active_before_26_3, |version| {
            version < Version::V26_3
        })
}

/// The release name the session plays, as the servers and the debug screen show it.
pub fn game_version_name() -> &'static str {
    Version::active().name()
}

/// Whether the serverbound swing carries a hand and the player action numbers skip the
/// direction change (before 26.3).
pub fn punch_is_swing() -> bool {
    Version::active_before_26_3()
}

/// Whether `PoseStack.Pose.rotate(axis, angle)` goes through a quaternion (before 26.3).
pub fn pose_rotates_by_quaternion() -> bool {
    Version::active_before_26_3()
}

/// Whether the client leaves a dropped item's speed as it was on a collision, because it does
/// not simulate those entities (before 26.3).
pub fn item_collision_keeps_speed() -> bool {
    Version::active_before_26_3()
}

/// Whether swings use the older timing rules (before 26.3).
pub fn legacy_swing() -> bool {
    Version::active_before_26_3()
}

/// Whether using a shelf answers with a plain success, which swings (before 26.3).
pub fn shelf_use_swings() -> bool {
    Version::active_before_26_3()
}

/// Whether the brewing stand's ingredient slot takes `PotionBrewing`'s own list (before 26.3).
pub fn brewing_ingredients_are_hard_coded() -> bool {
    Version::active_before_26_3()
}

/// Whether the spruce leaves exist as the tinted-particle class of the older versions.
pub fn spruce_leaves_are_tinted() -> bool {
    Version::active_before_26_3()
}
