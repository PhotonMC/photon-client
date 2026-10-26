use super::super::*;
use crate::{DimensionBounds, SectionInput};
use photon_core::ids::EntityId;

/// A world with a stone floor whose top is at y = 64, and a player the server added standing on
/// it at the middle of a block.
fn runner(data: u8) -> ClientWorld {
    let stone = photon_data::block::registry::default_state_named("minecraft:stone").0;
    let mut floor = vec![0; 4_096];
    floor[15 * 256..].fill(stone);
    let empty = vec![0; 4_096];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 128));
    world.replace_chunk(
        0,
        0,
        (0..8).map(|section| SectionInput {
            non_empty_block_count: if section == 3 { 256 } else { 0 },
            fluid_count: 0,
            block_states: if section == 3 { &floor } else { &empty },
            biomes: &[0; 64],
        }),
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(9),
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 8.5,
        y: 64.0,
        z: 8.5,
        movement: (0.2, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_metadata(
        EntityId(9),
        &[
            crate::ENTITY_SHARED_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            data,
            255,
        ],
    ));
    world
}

/// `Entity.baseTick`: a player the server says is sprinting kicks up a flake of the block under
/// its feet each tick, like the local player; one that only walks, or crouches, does not.
#[test]
fn a_sprinting_player_the_server_added_kicks_up_flakes_of_the_block_underfoot() {
    let mut world = runner(1 << 3);
    world.tick_entities();
    let flakes = world.take_effect_particles();
    assert_eq!(flakes.len(), 1);
    assert_eq!(
        flakes[0].type_id,
        photon_data::particle_type::id("block").expect("the block particle")
    );
    assert!((flakes[0].at.1 - 64.1).abs() < 1e-9);
    assert!((flakes[0].at.0 - 8.5).abs() <= 0.3 && (flakes[0].at.2 - 8.5).abs() <= 0.3);

    // Walking only: no flakes.
    let mut walking = runner(0);
    walking.tick_entities();
    assert!(walking.take_effect_particles().is_empty());

    // Sprinting but crouching (pose 5): no flakes.
    let mut crouching = runner(1 << 3);
    assert!(crouching.apply_entity_metadata(
        EntityId(9),
        &[
            crate::ENTITY_POSE_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_POSE_ID as u8,
            5,
            255,
        ],
    ));
    crouching.tick_entities();
    assert!(crouching.take_effect_particles().is_empty());
}
