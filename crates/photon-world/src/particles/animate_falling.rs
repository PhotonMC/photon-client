//! `FallingBlock.animateTick`: dust sifting out of a falling block with room below it, and the
//! desert's sand sounds (`AmbientDesertBlockSoundsPlayer.playAmbientSandSounds`).

use photon_core::ids::BlockStateId;
use photon_core::random::RandomSource;

use super::super::ClientWorld;
use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::local_sound;
use super::engine::{ParticleData, ParticleEngine};
use super::level_events::LevelEventContext;

/// `FallingBlock.isFree`: air, fire, a fluid, or a block that gives way.
pub(super) fn is_free(world: &ClientWorld, state: BlockStateId) -> bool {
    let info = crate::block::state_or_air(state);
    let block = info.block();
    block.air
        || block.liquid
        || block.replaceable
        || world
            .block_state_registry
            .tags()
            .contains(usize::from(info.block), "minecraft:fire")
}

/// The state at a block, air where nothing is loaded.
pub(super) fn state_at(world: &ClientWorld, (x, y, z): (i32, i32, i32)) -> BlockStateId {
    world.block_state_or_air(x, y, z)
}

pub(super) fn in_tag(world: &ClientWorld, state: BlockStateId, tag: &str) -> bool {
    let info = crate::block::state_or_air(state);
    world
        .block_state_registry
        .tags()
        .contains(usize::from(info.block), tag)
}

/// `Heightmap.Types.WORLD_SURFACE`: one above the highest block that is not air in the column.
pub(super) fn world_surface(world: &ClientWorld, x: i32, z: i32) -> i32 {
    let bounds = world.dimension();
    let top = bounds.min_y + bounds.height - 1;
    (bounds.min_y..=top)
        .rev()
        .find(|&y| {
            !crate::block::state_or_air(state_at(world, (x, y, z)))
                .block()
                .air
        })
        .map_or(bounds.min_y, |y| y + 1)
}

/// `columnContainsTriggeringBlock`: sand under open sky in the column, near the block's height or
/// within ten blocks of six above it.
fn column_triggers(world: &ClientWorld, (x, y, z): (i32, i32, i32)) -> bool {
    let triggers = |state| {
        in_tag(
            world,
            state,
            "minecraft:triggers_ambient_desert_sand_block_sounds",
        )
    };
    let air = |state: BlockStateId| crate::block::state_or_air(state).block().air;
    let surface = world_surface(world, x, z) - 1;
    if (surface - y).abs() > 5 {
        let mut above = state_at(world, (x, y + 6, z));
        let mut at = y + 5;
        for _ in 0..10 {
            let current = state_at(world, (x, at, z));
            if air(above) && triggers(current) {
                return true;
            }
            above = current;
            at -= 1;
        }
        false
    } else {
        air(state_at(world, (x, surface + 1, z))) && triggers(state_at(world, (x, surface, z)))
    }
}

/// `shouldPlayAmbientSandSound`: three of the four columns eight blocks off have sand under sky.
fn plays_sand_sound(world: &ClientWorld, (x, y, z): (i32, i32, i32)) -> bool {
    let mut found = 0;
    // `Direction.Plane.HORIZONTAL`: north, east, south, west.
    for (checked, (dx, dz)) in [(0, -8), (8, 0), (0, 8), (-8, 0)].into_iter().enumerate() {
        if column_triggers(world, (x + dx, y, z + dz)) {
            if found >= 3 {
                return true;
            }
            found += 1;
        }
        if 4 - (checked as i32 + 1) + found < 3 {
            return false;
        }
    }
    false
}

impl ParticleEngine {
    /// `FallingBlock.animateTick`, and sand's sounds after it.
    pub(super) fn falling_block_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (state, sand): (BlockStateId, bool),
        (x, y, z): (i32, i32, i32),
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) {
        let world = context.world;
        if random.next_int_bound(16) == Ok(0) && is_free(world, state_at(world, (x, y - 1, z))) {
            // `ParticleUtils.spawnParticleBelow`.
            let at = (
                f64::from(x) + random.next_double(),
                f64::from(y) - 0.05,
                f64::from(z) + random.next_double(),
            );
            self.level_add_data(context, "falling_dust", ParticleData::Block(state), at);
        }
        let above = crate::block::state_or_air(state_at(world, (x, y + 1, z)));
        if sand
            && above.block().name == "minecraft:air"
            && random.next_int_bound(2100) == Ok(0)
            && plays_sand_sound(world, (x, y, z))
        {
            let event = ("minecraft:block.sand.idle", LocalSoundSource::Ambient);
            let at = (f64::from(x), f64::from(y), f64::from(z));
            sounds.push(local_sound(event, (1.0, 1.0), at, context));
        }
    }
}
