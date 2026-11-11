//! The level's rules: blocks, entities, movement, particles and their sounds.

use crate::version::Version;

rules! {
    /// The level's rules (see the module).
    WorldRules {
        /// The level events for a splashed potion and a waxed block sound here as well as showing
        /// their particles (26.2); in 26.3 the sounds are their own events.
        splash_and_wax_events_sound: before V26_3,
        /// A fishing bobber falls by 0.03 as a double (26.2); 26.3 widens the float 0.03.
        bobber_gravity_is_double: before V26_3,
        /// The dragon head's outline is half a pixel taller than the other heads' (26.3).
        dragon_head_outline_is_tall: since V26_3,
        /// The resin brick wall is declared solid whatever its shape (26.3).
        resin_brick_wall_is_forced_solid: since V26_3,
        /// `blocksMotion` is plain solidity (26.3); 26.2 says no for cobweb and bamboo sapling.
        blocks_motion_is_solidity: since V26_3,
        /// The motion blocking heightmap leaves out signs, cobweb and bamboo sapling by its own
        /// list (26.3); 26.2 counts whatever blocks motion.
        motion_heightmap_has_own_list: since V26_3,
        /// A block can block the view only at the camera's near plane (26.3: stairs and slabs the
        /// player's eye sits inside); 26.2 has no such rule, a block blocks the view or it does
        /// not.
        blocks_view_at_near_plane: since V26_3,
        /// `Mth.length(float, float)` sums its squares as floats (26.3); 26.2 sums them as doubles.
        length_sums_squares_as_floats: since V26_3,
        /// A boat clamps every passenger the same way (26.2): body turned to the boat, yaw held
        /// within a quarter turn, previous yaw moved along, head set to the yaw. 26.3 treats mobs
        /// apart.
        boat_clamp_is_uniform: before V26_3,
        /// A bed item sets its block with its own flags (26: no neighbour update; before 26.3,
        /// whose bed item has no override and uses the block item's).
        bed_item_sets_own_flags: before V26_3,
        /// The client leaves a dropped item's speed as it was on a collision, because it does not
        /// simulate those entities (before 26.3).
        item_collision_keeps_speed: before V26_3,
        /// `Quaternionf.slerp` works out its first weight in double precision and narrows it
        /// (JOML 1.10.8, before 26.3); JOML 1.10.9 stays in single precision.
        slerp_weight_in_double: before V26_3,
        /// A position easing steps in the common tick, before the entity's own tick (26.3); 26.2
        /// steps it inside the tick of a living entity (`aiStep`, and the dragon's own), so what
        /// the tick does before that point, a mob's motion fade and the dragon's flight history,
        /// sees the easing and the position as they were when the tick began.
        easing_steps_in_common_tick: since V26_3,
        /// A slime block on a moving piston and a bubble column set the motion of every entity the
        /// client holds (26.2); 26.3 sets only that of the entities the client simulates.
        block_motion_reaches_all_entities: before V26_3,
        /// A fall that is exactly as fast as gravity pulls in a tick does not bounce off a springy
        /// block (26.3: the speed is compared with `<=`); 26.2 lets it bounce (`<`).
        bounce_stops_at_gravity_speed: since V26_3,
        /// The local player's own tick pushes it out of the blocks around it even while it rides
        /// a vehicle (26.2); from 26.3 a passenger is left alone.
        passenger_is_pushed_out_of_blocks: before V26_3,
        /// Swings use the older timing rules (before 26.3).
        legacy_swing: before V26_3,
        /// Using a shelf answers with a plain success, which swings (before 26.3).
        shelf_use_swings: before V26_3,
    }
}
