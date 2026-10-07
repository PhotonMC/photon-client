use super::*;

#[test]
fn passenger_tick_leaves_breeze_rotation_unclamped() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    for (entity_id, type_id, rotation) in [
        (37, 89, (0.0, 0.0, 0.0)),
        (38, VANILLA_BREEZE_ENTITY_TYPE_ID, (170.0, 0.0, 170.0)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    let passenger = world.entity_at(38).expect("Breeze passenger");
    assert_eq!(passenger.rotation(), (170.0, 0.0, 170.0));
    assert_eq!(passenger.previous_rotation(), (170.0, 0.0, 170.0));
}

#[test]
fn passenger_tick_positions_local_player_from_vehicle_attachment() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(37, 0),
        type_id: 89,
        x: 2.0,
        y: 2.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.set_local_player_id(38);
    assert!(world.apply_entity_metadata(37, &[ENTITY_NO_GRAVITY_DATA_ID, 8, 1, 255]));
    assert!(world.apply_local_player_position((2.0, 12.0, 2.0), (0.0, 0.0, 0.0), (0.0, 0.0),));
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    let player = world.local_player().expect("local player passenger");
    assert_eq!(player.previous_position(), (2.0, 12.0, 2.0));
    assert_eq!(player.position(), (2.0, 1.5875, 2.0));
}
