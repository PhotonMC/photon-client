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
        /// A colour attribute (sky, fog, cloud, sunrise, light colours) is a packed 8-bit int
        /// through every blend, keyframe lerp and modifier, each channel truncated (26.2); 26.3
        /// holds float vectors. Not the fog colour's own working (`fog_colour_is_packed`), which is
        /// another class. Proved by the `movement/sky-flash` rows between keyframes and in weather.
        attribute_colours_are_packed: before V26_3,
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
        /// The glint of an enchanted armor piece with a trim is drawn after the trim, over every
        /// layer, as a pass of its own (26.3). Without it (26.2), and in 26.3 for a piece with no
        /// trim, the glint comes right after the first layer and the layers after it and the trim
        /// are drawn over it. Source-derived (`EquipmentLayerRenderer`), no golden.
        trimmed_armor_glint_follows_trim: since V26_3,
        /// The fog colour's brightening (night vision, water vision) scales the channels that are
        /// not the brightest by one over the brightest channel held to at least 0.07 (26.2); 26.3
        /// scales every channel by one over the brightest. Proved by the `movement/fog-data` rows
        /// with dim fog.
        fog_brighten_clamps_scale: before V26_3,
        /// An ambient-occlusion corner lets light through when the block beyond it does not
        /// block the view or does not dampen light (26.2); 26.3 asks whether it is not drawn as a
        /// solid cube or does not dampen light. Ice, slime and shulker boxes differ. Proved by the
        /// `movement/block-lighting` ring rows.
        ao_corner_asks_view_blocking: before V26_3,
        /// A section's fade length is set each time it is compiled and is none for a section
        /// near the camera or one that was empty before; a section found empty is never stamped
        /// (26.2). 26.3 fades every section over the option's length from the time it is first
        /// found empty or meshed. Source-derived, no golden (a frame clock decides it).
        section_fade_is_set_per_compile: before V26_3,
        /// The world border's wall is drawn 0.01 inside the border on every side (26.3); its
        /// distance checks keep the true border. Proved by the `math/shape/border` quad rows,
        /// which the oracle lays out from the real renderer's constant.
        border_wall_is_inset: since V26_3,
        /// The forcefield texture the world border is drawn with has four mip levels and a linear
        /// minification filter (26.3); 26.2 has the one level. Source-derived, no golden.
        forcefield_has_mipmaps: since V26_3,
        /// An armor trim's picture is the pattern's grey picture recoloured by a palette when it
        /// is first drawn, and an equipment asset's `trim_overrides` choose the picture and
        /// palette (26.3); 26.2 draws a sprite of the `armor_trims` atlas, named by the material's
        /// suffix, and the material's `override_armor_assets` choose that suffix per armor.
        /// Source-derived, no golden: the oracle proves the names only.
        armor_trims_are_recoloured_on_demand: since V26_3,
        /// The biome noise (the temperature above the snow line, the frozen biome's cold patches,
        /// the swamp's grass colour) hands back a float, and the frozen patches sum their three
        /// octaves in floats (26.3); 26.2 hands back doubles and sums them in doubles. A noise
        /// value that lies close to a threshold can fall on either side. Proved by the
        /// `movement/biome-noise` rows at positions where the two disagree.
        biome_noise_is_float: since V26_3,
        /// The destroy overlay of a block entity's opaque model is drawn with the opaque models,
        /// before every translucent one, so the translucent pattern layers of a cracked banner
        /// lie over the crack (26.3); 26.2 draws every overlay after all translucent models, over
        /// the layers. Source-derived, no golden: the draw log does not carry submit order.
        crumbling_is_drawn_with_solids: since V26_3,
    }
}
