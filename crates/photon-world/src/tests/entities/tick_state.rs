use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn tick_counters_follow_local_player_and_entity_ticking_eligibility() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::named("minecraft:snowball"),
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(0)
    );
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::tick_count),
        Some(0)
    );

    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(1)
    );
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::tick_count),
        Some(1)
    );

    assert!(world.unload_chunk(0, -3));
    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(2)
    );
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::tick_count),
        Some(1)
    );

    world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(3)
    );
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::tick_count),
        Some(2)
    );

    assert!(world.recreate_local_player(0));
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(0)
    );
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(1)
    );
}

#[test]
fn tick_captures_previous_transform_for_position_and_rotation_interpolation() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
    assert!(
        world.apply_local_player_position((10.0, 64.0, -2.0), (0.0, 0.0, 0.0), (350.0, -20.0),)
    );
    assert_eq!(
        world
            .local_player()
            .map(crate::ClientPlayerState::previous_position),
        Some((10.0, 64.0, -2.0))
    );
    assert_eq!(
        world
            .local_player()
            .map(crate::ClientPlayerState::previous_rotation),
        Some((350.0, -20.0))
    );

    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::named("minecraft:snowball"),
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (350.0, -20.0, 350.0),
        data: 0,
    }));

    world.tick();
    assert_eq!(
        world
            .local_player()
            .map(crate::ClientPlayerState::previous_position),
        Some((10.0, 64.0, -2.0))
    );
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::previous_position),
        Some((1.25, 80.0, -35.5))
    );

    assert!(world.apply_local_player_position((14.0, 68.0, 6.0), (0.0, 0.0, 0.0), (10.0, 20.0),));
    let player = world.local_player().expect("local player transform");
    assert_eq!(player.previous_position(), (10.0, 64.0, -2.0));
    assert_eq!(player.interpolated_position(0.5), (12.0, 66.0, 2.0));
    assert_eq!(player.previous_rotation(), (350.0, -20.0));
    assert_eq!(player.interpolated_rotation(0.5), (360.0, 0.0));
    assert_eq!(player.interpolated_rotation(1.0), (10.0, 20.0));

    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: EntityId(37),
        x: 3.0,
        y: 82.0,
        z: -34.5,
        relative_flags: 0,
    }));
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: EntityId(37),
        y_rot: Some(10.0),
        x_rot: Some(20.0),
        y_head_rot: Some(10.0),
        relative_y_rot: false,
        relative_x_rot: false,
    }));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("remote entity transform");
    assert_eq!(entity.previous_position(), (1.25, 80.0, -35.5));
    assert_eq!(entity.interpolated_position(0.5), (2.125, 81.0, -35.0));
    assert_eq!(entity.previous_rotation(), (350.0, -20.0, 350.0));
    assert_eq!(entity.interpolated_rotation(0.5), (360.0, 0.0, 360.0));

    assert!(world.unload_chunk(0, -3));
    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: EntityId(37),
        x: 4.0,
        y: 82.0,
        z: -34.5,
        relative_flags: 0,
    }));
    world.tick();
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::previous_position),
        Some((1.25, 80.0, -35.5))
    );

    world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    world.tick();
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::previous_position),
        Some((4.0, 82.0, -34.5))
    );
}

#[test]
fn tick_counter_is_independent_of_chunk_storage() {
    let mut world = ClientWorld::default();
    assert_eq!(world.game_time(), 0);
    world.tick();
    world.tick();
    assert_eq!(world.game_time(), 2);
    assert_eq!(world.loaded_chunk_count(), 0);
}
