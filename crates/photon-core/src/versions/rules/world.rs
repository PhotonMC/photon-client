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
        /// A picked-up entity that is silent makes no pickup sound (26.3); before it always did.
        silent_pickups_make_no_sound: since V26_3,
        /// A silent firework rocket bursts without the blast and twinkle sounds (26.3); 26.2 played
        /// them whatever its flags.
        silent_rockets_make_no_sound: since V26_3,
        /// A silent lightning bolt plays neither its thunder nor its impact on its first tick (26.3);
        /// 26.2 played them whatever its flags.
        silent_lightning_makes_no_sound: since V26_3,
        /// A pickup whose taker the level does not hold is flown to the local player (26.2); in
        /// 26.3 there is no pickup animation then.
        unknown_taker_is_the_local_player: before V26_3,
        /// The server tells of the effects of a block broken without a sound (event 2014), of a
        /// block's teleport (2015, 2016, 2017 and 2018) and of a block broken from a face (2019
        /// and 2020) with level events (26.3); a 26.2 client has no such events.
        destroy_and_teleport_level_events: since V26_3,
        /// A boat above a bubble column tells the one who steers it with entity events 71 (drag
        /// down) and 72 (launch up), which set its motion there (26.3); 26.2 does not tell.
        boat_bubble_column_events: since V26_3,
        /// A glide cannot be started in lava either (26.3: `isInLiquid`); 26.2 refuses only in
        /// water.
        glide_start_refused_in_lava: since V26_3,
        /// The height the camera and the eyes-in-fluid test read of a source fluid is a whole
        /// block when a block with a sturdy underside is over it (26.3: `getHeightForCamera`);
        /// 26.2 reads the fluid's height.
        camera_height_fills_under_ceiling: since V26_3,
        /// A fluid's flow is stopped by the blocks of the `blocks_fluid_flow` tag the server sends
        /// (26.3: the motion blocking blocks and the signs); 26.2 asks `blocksMotion`, which is
        /// false for cobweb and bamboo sapling.
        fluid_flow_is_stopped_by_tag: since V26_3,
        /// A sleeper lies at the height the bed gives (26.3: the top of its shape, and nowhere
        /// when the block is no bed); 26.2 puts it a fixed 0.6875 over the block.
        sleep_height_comes_from_the_bed: since V26_3,
    }
}
