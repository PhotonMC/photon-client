use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn passenger_tick_uses_ride_tick_motion_reset_before_projectile_tick() {
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
    for (entity_id, type_id, motion) in [
        (37, 71, (0.0, 0.0, 0.0)),
        (38, VANILLA_SNOWBALL_ENTITY_TYPE_ID, (0.125, 0.0, 0.0)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: motion,
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    world.tick();

    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(|entity| entity.tick_count()),
        Some(1)
    );
    let passenger = world
        .entities()
        .at(EntityId(38))
        .expect("snowball passenger");
    assert_eq!(passenger.tick_count(), 1);
    assert_eq!(passenger.motion().0, 0.0);
    assert_eq!(passenger.motion().2, 0.0);
    assert_eq!(
        passenger.motion().1,
        -SNOWBALL_GRAVITY * f64::from(0.99_f32)
    );
    assert_eq!(passenger.position().0, 2.0);
}

#[test]
fn passenger_tick_positions_snowball_on_regular_boat_seat() {
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
    for (entity_id, type_id, position, motion) in [
        (37, 89, (2.0, 2.0, 2.0), (0.0, 0.0, 0.0)),
        (
            38,
            VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            (2.0, 12.0, 2.0),
            (0.125, 0.0, 0.0),
        ),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: position.0,
            y: position.1,
            z: position.2,
            movement: motion,
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    world.tick();

    let passenger = world
        .entities()
        .at(EntityId(38))
        .expect("snowball passenger");
    assert_eq!(passenger.previous_position(), (2.0, 12.0, 2.0));
    assert_eq!(passenger.position(), (2.0, 2.1875, 2.0));
    assert_eq!(passenger.motion().0, 0.0);
    assert_eq!(
        passenger.motion().1,
        -SNOWBALL_GRAVITY * f64::from(0.99_f32)
    );
}

#[test]
fn passenger_tick_positions_multiple_snowballs_by_boat_order() {
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
    for (entity_id, type_id, position, rotation) in [
        (37, 89, (2.0, 2.0, 2.0), (90.0, 0.0, 0.0)),
        (
            38,
            VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            (2.0, 12.0, 2.0),
            (0.0, 0.0, 0.0),
        ),
        (
            39,
            VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            (2.0, 12.0, 2.0),
            (0.0, 0.0, 0.0),
        ),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: position.0,
            y: position.1,
            z: position.2,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38), EntityId(39)]));

    world.tick();

    let first = world
        .entities()
        .at(EntityId(38))
        .expect("first snowball passenger");
    let second = world
        .entities()
        .at(EntityId(39))
        .expect("second snowball passenger");
    assert!((first.position().0 - 1.8).abs() < 1e-6);
    assert!((first.position().1 - 2.1875).abs() < 1e-6);
    assert!((first.position().2 - 2.0).abs() < 1e-6);
    assert!((second.position().0 - 2.6).abs() < 1e-6);
    assert!((second.position().1 - 2.1875).abs() < 1e-6);
    assert!((second.position().2 - 2.0).abs() < 1e-6);
}

#[test]
fn passenger_tick_positions_all_vanilla_boat_geometry_rows() {
    let expected_chest_offset = f64::from(0.15_f32);
    let expected_raft_height = f64::from(0.5625_f32 * 0.8888889_f32);
    let expected_geometry = [
        (0, 0.1875, 0.0),
        (1, 0.1875, expected_chest_offset),
        (8, expected_raft_height, expected_chest_offset),
        (9, expected_raft_height, 0.0),
        (12, 0.1875, 0.0),
        (13, 0.1875, expected_chest_offset),
        (23, 0.1875, 0.0),
        (24, 0.1875, expected_chest_offset),
        (33, 0.1875, 0.0),
        (34, 0.1875, expected_chest_offset),
        (74, 0.1875, 0.0),
        (75, 0.1875, expected_chest_offset),
        (81, 0.1875, 0.0),
        (82, 0.1875, expected_chest_offset),
        (89, 0.1875, 0.0),
        (90, 0.1875, expected_chest_offset),
        (94, 0.1875, 0.0),
        (95, 0.1875, expected_chest_offset),
        (125, 0.1875, 0.0),
        (126, 0.1875, expected_chest_offset),
    ];
    for (index, &(vehicle_type_id, ride_height, passenger_z_offset)) in
        expected_geometry.iter().enumerate()
    {
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
        let vehicle_id = 1000 + index as i32;
        let passenger_id = 2000 + index as i32;
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(vehicle_id),
            uuid: Uuid::new(vehicle_id as u64, 0),
            type_id: vehicle_type_id,
            x: 2.0,
            y: 10.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(passenger_id),
            uuid: Uuid::new(passenger_id as u64, 0),
            type_id: VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            x: 2.0,
            y: 12.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity_passengers(EntityId(vehicle_id), &[EntityId(passenger_id)]));

        world.tick();

        let passenger = world
            .entities()
            .at(EntityId(passenger_id))
            .expect("boat geometry passenger");
        assert_eq!(
            passenger.position(),
            (2.0, 10.0 + ride_height, 2.0 + passenger_z_offset)
        );
    }
}

#[test]
fn passenger_tick_positions_remote_player_from_vehicle_attachment() {
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
    for (entity_id, type_id, position) in [
        (37, 89, (2.0, 2.0, 2.0)),
        (38, VANILLA_PLAYER_ENTITY_TYPE_ID, (2.0, 12.0, 2.0)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: position.0,
            y: position.1,
            z: position.2,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    world.tick();

    let passenger = world
        .entities()
        .at(EntityId(38))
        .expect("remote player passenger");
    assert_eq!(passenger.previous_position(), (2.0, 12.0, 2.0));
    assert_eq!(passenger.position(), (2.0, 1.5875, 2.0));
}

#[test]
fn passenger_tick_positions_generic_entity_vehicle_attachment() {
    for (passenger_id, passenger_type_id, expected_y) in [
        (38, 71, 2.1875_f64),
        (39, 151, 2.1875_f64),
        (40, 124, 1.875_f64),
    ] {
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
            entity_id: EntityId(37),
            uuid: Uuid::new(37, 0),
            type_id: 89,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(passenger_id),
            uuid: Uuid::new(passenger_id as u64, 0),
            type_id: passenger_type_id,
            x: 2.0,
            y: 12.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(passenger_id)]));

        world.tick();

        let passenger = world
            .entities()
            .at(EntityId(passenger_id))
            .expect("generic vehicle passenger");
        assert_eq!(passenger.previous_position(), (2.0, 12.0, 2.0));
        assert_eq!(passenger.position(), (2.0, expected_y, 2.0));
    }
}

#[test]
fn passenger_tick_uses_zombie_baby_vehicle_attachment_metadata() {
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
        entity_id: EntityId(37),
        uuid: Uuid::new(37, 0),
        type_id: 89,
        x: 2.0,
        y: 2.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(38),
        uuid: Uuid::new(38, 0),
        type_id: 151,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    world.tick();
    assert_eq!(
        world
            .entities()
            .at(EntityId(38))
            .expect("adult zombie")
            .position(),
        (2.0, 2.1875, 2.0)
    );

    assert!(world.apply_entity_metadata(EntityId(38), &[16, 8, 1, 255]));
    world.tick();
    let baby = world.entities().at(EntityId(38)).expect("baby zombie");
    assert!(baby.is_baby());
    assert_eq!(baby.position(), (2.0, 2.0, 2.0));

    assert!(world.apply_entity_metadata(EntityId(38), &[16, 8, 0, 255]));
    world.tick();
    let adult = world
        .entities()
        .at(EntityId(38))
        .expect("adult zombie after reset");
    assert!(!adult.is_baby());
    assert_eq!(adult.position(), (2.0, 2.1875, 2.0));
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
        entity_id: EntityId(37),
        uuid: Uuid::new(37, 0),
        type_id: 89,
        x: 2.0,
        y: 2.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.set_local_player_id(EntityId(38));
    assert!(world.apply_entity_metadata(EntityId(37), &[ENTITY_NO_GRAVITY_DATA_ID, 8, 1, 255]));
    assert!(world.apply_local_player_position((2.0, 12.0, 2.0), (0.0, 0.0, 0.0), (0.0, 0.0),));
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    world.tick();

    let player = world.local_player().expect("local player passenger");
    assert_eq!(player.previous_position(), (2.0, 12.0, 2.0));
    assert_eq!(player.position(), (2.0, 1.5875, 2.0));
}
