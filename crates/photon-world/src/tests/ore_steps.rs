use super::*;
use crate::local_player::LocalPlayer;
use crate::{ClientPlayerGameMode, LevelEvent};
use photon_core::binary::BlockPos;
use photon_core::ids::BlockStateId;

fn ore(lit: &str) -> BlockStateId {
    let ore = photon_data::block::registry::block("minecraft:redstone_ore")
        .unwrap()
        .default_state_id();
    crate::block::rules::with_property(ore, "lit", lit).unwrap()
}

fn world_with_ore(lit: &str) -> ClientWorld {
    let mut blocks = vec![0; 4_096];
    blocks[256 + 16 + 1] = ore(lit).0;
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    assert!(world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ));
    world
}

fn stepped(count: usize) -> LocalPlayer {
    let mut player = LocalPlayer::new(
        photon_core::ids::EntityId(1),
        Vec3::new(1.5, 2.0, 1.5),
        0.0,
        0.0,
        ClientPlayerGameMode::Survival,
        true,
    );
    player
        .trace
        .stepped_on
        .extend(std::iter::repeat_n(BlockPos { x: 1, y: 1, z: 1 }, count));
    player
}

/// `RedStoneOreBlock.stepOn`: a step lights an unlit ore, and every step (a push repeats them)
/// gives off its dust.
#[test]
fn a_step_lights_an_ore_and_sparks_each_time() {
    let mut world = world_with_ore("false");
    world.apply_player_steps(&mut stepped(2));
    assert_eq!(world.block_state_at(1, 1, 1), Some(ore("true")));
    let sparks = LevelEvent::OreSparks { pos: (1, 1, 1) };
    assert_eq!(world.take_level_events(), vec![sparks.clone(), sparks]);
}

/// A lit ore stays lit and still sparks.
#[test]
fn a_lit_ore_only_sparks() {
    let mut world = world_with_ore("true");
    world.apply_player_steps(&mut stepped(1));
    assert_eq!(world.block_state_at(1, 1, 1), Some(ore("true")));
    assert_eq!(world.take_level_events().len(), 1);
}
