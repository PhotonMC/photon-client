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

/// Whether the quit shortcuts option exists (new in 26.3).
pub fn has_quit_shortcuts_option() -> bool {
    !Version::active_before_26_3()
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

/// Whether option list headers are a left aligned string widget, a line and 4 tall (before 26.3).
pub fn option_headers_are_plain() -> bool {
    Version::active_before_26_3()
}
