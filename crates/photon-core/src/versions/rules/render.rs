//! How things are drawn.

use crate::version::Version;

rules! {
    /// The rules of drawing (see the module).
    RenderRules {
        /// The name tag text is the fixed half-transparent white instead of a colour derived from
        /// the background opacity (26.2 submits a constant; 26.3 derives it).
        name_tag_text_is_constant: before V26_3,
        /// A unifont glyph advances by half its width in whole pixels (26.3) instead of exactly
        /// half (26.2).
        unihex_advance_is_whole: since V26_3,
        /// The atmosphere's fog colour is worked out on packed 8-bit colours (26.2); 26.3 keeps
        /// float vectors.
        fog_colour_is_packed: before V26_3,
        /// Every tooltip leaves a gap after its first line, not only an item's (before 26.3).
        every_tooltip_gaps_after_first_line: before V26_3,
        /// `PoseStack.Pose.rotate(axis, angle)` goes through a quaternion (before 26.3).
        pose_rotates_by_quaternion: before V26_3,
    }
}
