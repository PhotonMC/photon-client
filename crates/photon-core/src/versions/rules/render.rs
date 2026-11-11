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
        /// A humanoid tips its head toward the swimming pose right after setting its look,
        /// before the arms are posed and the swing reads the head (26.2); 26.3 tips it with the
        /// swim, after the swing and the crouch.
        swim_head_tips_before_arms: before V26_3,
        /// An arm stays out of the swim stroke only while a swing of it has run (attack time
        /// above zero) in 26.2; 26.3 asks only whether a swing of that arm is current.
        swim_arm_skips_swing_with_progress: before V26_3,
        /// A spear pose on one arm leaves the other arm unposed, as a bow does (26.2); 26.3 poses
        /// both arms.
        spear_pose_affects_offhand_pose: before V26_3,
        /// A spear held in the right arm is raised by the main hand's use and one held in the left
        /// arm by the off hand's, whichever arm is the main one (26.2); 26.3 maps the hand using
        /// the item to its arm.
        spear_use_hand_is_fixed_by_arm: before V26_3,
        /// A zombie-like model holds its arms out after the whole humanoid pose (crouch, bob and
        /// swim included), and bobs them again; it skips that only when the swing is a stab
        /// (26.2). 26.3 does it inside the attack animation, skipped when a spear is held.
        zombie_arms_follow_whole_pose: before V26_3,
        /// A drowned model swims with the humanoid stroke first and its own stroke on top, after
        /// holding a thrown trident up again (26.2); 26.3 swaps the whole stroke for its own.
        drowned_swim_adds_to_humanoid_swim: before V26_3,
        /// The box an entity is culled by is drawn between its last and current position at the
        /// frame's partial tick (26.3); 26.2 culls by the current box alone.
        culling_box_interpolates: since V26_3,
        /// The enchantment glint is a pass of its own over the model (`pipeline/glint`); from
        /// 26.3 it is a switch of the pipeline that draws the model itself.
        glint_is_own_pass: before V26_3,
    }
}
