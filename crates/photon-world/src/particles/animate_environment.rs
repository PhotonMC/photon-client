//! The part of `animateTick` that reads the environment attributes: the biomes' ambient particles,
//! and the creaking heart's idle sound while the creaking is about.

use photon_core::ids::BlockStateId;
use photon_core::random::RandomSource;
use photon_data::block::classes::BlockClass;
use photon_data::block::registry::Block;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::local_sound;
use super::animate_falling::in_tag;
use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;

type Pos = (i32, i32, i32);

impl ParticleEngine {
    /// The biome's `AMBIENT_PARTICLES` at a block the level picked, for a block that is not a full
    /// cube: each particle by its chance, at a place in the block drawn from the level's random.
    pub(super) fn ambient_particles<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        pos: Pos,
        context: &mut LevelEventContext<'_, F>,
    ) {
        if crate::block::state_or_air(state).collision_full_block {
            return;
        }
        for ambient in context.ambience.ambient_particles(context.world, pos) {
            if context.level_random.next_float() > ambient.probability {
                continue;
            }
            let random = &mut *context.level_random;
            let at = (
                f64::from(pos.0) + random.next_double(),
                f64::from(pos.1) + random.next_double(),
                f64::from(pos.2) + random.next_double(),
            );
            let name = ambient
                .particle
                .strip_prefix("minecraft:")
                .unwrap_or(&ambient.particle);
            self.level_add(context, name, at, (0.0, 0.0, 0.0));
        }
    }

    /// `Block.animateTick` of the blocks that ask the environment; false for another block.
    pub(super) fn environment_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (state, block): (BlockStateId, &'static Block),
        pos: Pos,
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) -> bool {
        if !block.class.is(BlockClass::CreakingHeartBlock) {
            return false;
        }
        if context.ambience.creaking_active()
            && crate::block::rules::property(state, "creaking_heart_state") != Some("uprooted")
            && random.next_int_bound(16) == Ok(0)
            && is_surrounded_by_logs(context, pos)
        {
            let at = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
            sounds.push(local_sound(
                (
                    "minecraft:block.creaking_heart.idle",
                    LocalSoundSource::Blocks,
                ),
                (1.0, 1.0),
                at,
                context,
            ));
        }
        true
    }
}

/// `CreakingHeartBlock.isSurroundedByLogs`: pale oak logs on all six sides.
fn is_surrounded_by_logs<F: FnMut() -> RandomSource>(
    context: &LevelEventContext<'_, F>,
    (x, y, z): Pos,
) -> bool {
    photon_core::direction::Direction::stream().all(|direction| {
        let (dx, dy, dz) = direction.step();
        let neighbour = context.world.block_state_or_air(x + dx, y + dy, z + dz);
        in_tag(context.world, neighbour, "minecraft:pale_oak_logs")
    })
}
