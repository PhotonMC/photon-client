//! `DragonEggBlock.teleport` on a client (before 26.3): up to a thousand random places near the
//! egg are tried with the level's random source, and the first fit gets the portal specks.

use photon_core::random::RandomSource;
use photon_data::block::registry;

use super::ParticleEngine;
use super::level_events::LevelEventContext;
use crate::*;

/// How many places the egg tries before it gives up.
const TRIES: u32 = 1000;

impl ParticleEngine {
    /// The egg at `pos` looks for a place to go; the specks fly along the way when it finds one.
    pub(super) fn egg_teleport<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..TRIES {
            let random = &mut *context.level_random;
            let mut offset = |bound: i32| -> i32 {
                let plus = random.next_int_bound(bound).unwrap_or(0);
                plus - random.next_int_bound(bound).unwrap_or(0)
            };
            let (dx, dy, dz) = (offset(16), offset(8), offset(16));
            let test = (pos.0 + dx, pos.1 + dy, pos.2 + dz);
            if may_land(context.world, test) {
                self.teleport_specks(pos, test, context);
                return;
            }
        }
    }
}

/// Air at the place, something below it, the place inside the border and the build height.
fn may_land(world: &ClientWorld, at: (i32, i32, i32)) -> bool {
    let air = |x, y, z| {
        registry::block_for_state(world.block_state_or_air(x, y, z))
            .is_none_or(|(block, _)| block.air)
    };
    let border = world.world_border();
    let (x, z) = (f64::from(at.0), f64::from(at.2));
    let bounds = world.dimension();
    air(at.0, at.1, at.2)
        && !air(at.0, at.1 - 1, at.2)
        && x >= border.min_x(0.0)
        && x < border.max_x(0.0)
        && z >= border.min_z(0.0)
        && z < border.max_z(0.0)
        && at.1 >= bounds.min_y
        && at.1 < bounds.min_y + bounds.height
}
