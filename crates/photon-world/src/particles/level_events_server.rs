//! The level events a 26.3 server sends for effects a 26.2 client made on its own: a block's
//! crumbs without its sound (2014), a block teleporting (2015, 2016, 2017) and an enderman doing
//! so (2018), and a block broken from a face (2019, 2020; in `level_events_more`).

use photon_core::binary::BlockPos;
use photon_core::ids::BlockStateId;
use photon_core::random::RandomSource;

use super::super::local_player::LocalSound;
use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;

/// `EntityType.ENDERMAN`'s width and height.
const ENDERMAN_WIDTH: f32 = 0.6;
const ENDERMAN_HEIGHT: f32 = 2.9;

/// `BlockUtil.unpackDifferenceInPosition`: the place a packed difference leads to from `pos`, each
/// axis kept in a byte with its radius added.
fn unpack_difference(pos: (i32, i32, i32), packed: i32, radii: (i32, i32, i32)) -> (i32, i32, i32) {
    (
        pos.0 + ((packed >> 16 & 0xFF) - radii.0),
        pos.1 + ((packed >> 8 & 0xFF) - radii.1),
        pos.2 + ((packed & 0xFF) - radii.2),
    )
}

/// `Mth.lerp(double, double, double)`.
fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

impl ParticleEngine {
    /// Events 2014 to 2020 as a 26.3 server sends them; a 26.2 client has no such events.
    pub(super) fn server_effect_event<F: FnMut() -> RandomSource>(
        &mut self,
        (event, pos, data): (i32, (i32, i32, i32), i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        if !photon_core::versions::rules()
            .world
            .destroy_and_teleport_level_events
        {
            return;
        }
        match event {
            2014 => {
                let at = BlockPos::new(pos.0, pos.1, pos.2);
                let state = (at, BlockStateId(data));
                self.add_destroy_block_effect(context.world, state, &mut context.particle_random);
            }
            2015 => self.teleport_specks(pos, unpack_difference(pos, data, (16, 8, 16)), context),
            2016 => self.teleport_specks(pos, unpack_difference(pos, data, (8, 8, 8)), context),
            2017 => {
                self.teleport_specks(pos, unpack_difference(pos, data, (127, 127, 127)), context);
            }
            2018 => {
                self.enderman_specks(pos, unpack_difference(pos, data, (127, 127, 127)), context);
            }
            _ => self.breaking_block_effects((event, pos, data), context, sounds),
        }
    }

    /// `LevelEventHandler.singleBlockTeleportationParticles`: 128 portal specks strewn along the
    /// way from the place the block went to, within a block's width of it.
    fn teleport_specks<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        to: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..128 {
            let random = &mut *context.level_random;
            let along = random.next_double();
            let push = (
                f64::from((random.next_float() - 0.5) * 0.2),
                f64::from((random.next_float() - 0.5) * 0.2),
                f64::from((random.next_float() - 0.5) * 0.2),
            );
            let x =
                lerp(along, f64::from(to.0), f64::from(pos.0)) + (random.next_double() - 0.5) + 0.5;
            let y = lerp(along, f64::from(to.1), f64::from(pos.1)) + random.next_double() - 0.5;
            let z =
                lerp(along, f64::from(to.2), f64::from(pos.2)) + (random.next_double() - 0.5) + 0.5;
            self.level_add(context, "portal", (x, y, z), push);
        }
    }

    /// Event 2018: 128 portal specks along an enderman's way, scattered about the size of its box.
    fn enderman_specks<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        to: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..128 {
            let random = &mut *context.level_random;
            let along = random.next_double();
            let push = (
                f64::from((random.next_float() - 0.5) * 0.2),
                f64::from((random.next_float() - 0.5) * 0.2),
                f64::from((random.next_float() - 0.5) * 0.2),
            );
            let width = f64::from(ENDERMAN_WIDTH);
            let x = lerp(along, f64::from(to.0), f64::from(pos.0))
                + (random.next_double() - 0.5) * width * 2.0;
            let y = lerp(along, f64::from(to.1), f64::from(pos.1))
                + random.next_double() * f64::from(ENDERMAN_HEIGHT);
            let z = lerp(along, f64::from(to.2), f64::from(pos.2))
                + (random.next_double() - 0.5) * width * 2.0;
            self.level_add(context, "portal", (x, y, z), push);
        }
    }
}
