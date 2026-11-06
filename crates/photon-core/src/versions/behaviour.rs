//! Behaviour questions whose answer depends on the version the session plays. Every one reads
//! the active version, so a process holds one version at a time.

use crate::version::Version;

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
