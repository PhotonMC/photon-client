use super::super::*;

#[test]
fn passenger_tick_clamps_remote_player_rotation_to_boat_heading() {
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
    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(38, 0),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (140.0, 0.0, 140.0),
        data: 0,
    }));
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    let passenger = world.entities().at(38).expect("remote player passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 105.0));
    assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 140.0));
    assert_eq!(passenger.body_y_rotation(), 0.0);
    assert_eq!(passenger.previous_body_y_rotation(), 140.0);
}

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

    let passenger = world.entities().at(38).expect("Breeze passenger");
    assert_eq!(passenger.rotation(), (170.0, 0.0, 170.0));
    assert_eq!(passenger.previous_rotation(), (170.0, 0.0, 170.0));
}

#[test]
fn passenger_tick_clamps_boat_rotation_for_generic_item_passenger() {
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
    for (entity_id, type_id, rotation) in [(37, 89, (0.0, 0.0, 0.0)), (38, 71, (170.0, 0.0, 170.0))]
    {
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

    let passenger = world.entities().at(38).expect("generic boat passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 170.0));
    assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 170.0));
    assert_eq!(passenger.body_y_rotation(), 170.0);
}

#[test]
fn passenger_tick_applies_animal_living_rotation_before_capacity_offset() {
    for (vehicle_type_id, expected_rotation, expected_body) in
        [(89, 105.0_f32, 0.0_f32), (90, 195.0_f32, 90.0_f32)]
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
        for (entity_id, type_id, rotation) in [
            (37, vehicle_type_id, (0.0, 0.0, 0.0)),
            (38, VANILLA_COW_ENTITY_TYPE_ID, (170.0, 0.0, 170.0)),
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

        let passenger = world.entities().at(38).expect("Cow passenger");
        assert_eq!(passenger.rotation(), (105.0, 0.0, expected_rotation));
        assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 170.0));
        assert_eq!(passenger.body_y_rotation(), expected_body);
        assert_eq!(passenger.previous_body_y_rotation(), 170.0);
    }
}

#[test]
fn passenger_turn_calls_boat_on_passenger_turned() {
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
        (38, VANILLA_PLAYER_ENTITY_TYPE_ID, (168.75, 0.0, 168.75)),
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

    assert!(world.turn_entity(38, -100.0, 0.0));

    let passenger = world.entities().at(38).expect("Player passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 105.0));
    assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 168.75));
    assert_eq!(passenger.body_y_rotation(), 0.0);
    assert_eq!(passenger.previous_body_y_rotation(), 168.75);
    assert_eq!(passenger.position(), (2.0, 2.0, 2.0));
}

#[test]
fn passenger_turn_clamps_breeze_despite_boat_turn_exemption() {
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
        (38, VANILLA_BREEZE_ENTITY_TYPE_ID, (168.75, 0.0, 168.75)),
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

    assert!(world.turn_entity(38, -100.0, 0.0));

    let passenger = world.entities().at(38).expect("Breeze passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 105.0));
    assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 168.75));
    assert_eq!(passenger.body_y_rotation(), 0.0);
    assert_eq!(passenger.previous_body_y_rotation(), 168.75);
}

#[test]
fn boat_control_delta_rotation_reaches_passenger_tick() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let (mut blocks, biomes) = section(0, 0);
    blocks[12 * 256 + 2 * 16 + 2] = 1;
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    for (entity_id, type_id) in [(37, 89), (38, 151), (39, VANILLA_PLAYER_ENTITY_TYPE_ID)] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(1, entity_id as u64),
            type_id,
            x: 2.0,
            y: 12.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    world.set_local_player_id(39);
    assert!(world.apply_entity_passengers(37, &[39, 38]));
    assert!(world.set_boat_input(37, BoatInput::new(false, true, false, false)));

    world.tick();

    let passenger = world.entities().at(38).expect("living boat passenger");
    assert_eq!(passenger.rotation(), (1.0, 0.0, 1.0));
    assert_eq!(passenger.previous_rotation(), (0.0, 0.0, 0.0));
    assert_eq!(passenger.body_y_rotation(), 1.0);
    // The boat moved as it ticked, and the passenger is carried to where it now stands.
    let boat = world.entities().at(37).expect("boat").position();
    let radians = f64::from(-(std::f32::consts::PI / 180.0_f32));
    let passenger_z_offset = f64::from(-0.6_f32);
    let expected_position = (
        boat.0 + passenger_z_offset * f64::from(mth_sin_f64(radians)),
        boat.1 + 0.1875,
        boat.2 + passenger_z_offset * f64::from(mth_cos_f64(radians)),
    );
    assert_eq!(passenger.position(), expected_position);
}

#[test]
fn boat_authority_uses_first_passenger_only() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let (mut blocks, biomes) = section(0, 0);
    blocks[12 * 256 + 2 * 16 + 2] = 1;
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 37),
        type_id: 89,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(1, 38),
        type_id: 151,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: 39,
        uuid: Uuid::new(1, 39),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.set_local_player_id(39);
    assert!(world.apply_entity_passengers(37, &[38, 39]));
    assert!(world.set_boat_input(37, BoatInput::new(false, true, false, false)));

    world.tick();

    let boat = world.entities().at(37).expect("boat");
    assert_eq!(boat.position(), (2.0, 12.0, 2.0));
    assert_eq!(boat.motion(), (0.0, 0.0, 0.0));
    assert_eq!(boat.rotation(), (0.0, 0.0, 0.0));
    assert_eq!(boat.boat_delta_rotation(), 0.0);
    assert_eq!(boat.paddle_state(), (false, false));
}

#[test]
fn passenger_turn_clamps_every_vanilla_living_entity_body_and_head() {
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
    for (index, &type_id) in VANILLA_LIVING_ENTITY_TYPE_IDS.iter().enumerate() {
        let entity_id = 100 + index as i32;
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (168.75, 0.0, 168.75),
            data: 0,
        }));
    }
    for index in 0..VANILLA_LIVING_ENTITY_TYPE_IDS.len() {
        let passenger_id = 100 + index as i32;
        assert!(world.apply_entity_passengers(37, &[passenger_id]));
        assert!(world.turn_entity(passenger_id, -100.0, 0.0));
        let passenger = world.entities().at(passenger_id).expect("living passenger");
        assert_eq!(
            passenger.rotation(),
            (
                VANILLA_BOAT_PASSENGER_MAX_ROTATION,
                0.0,
                VANILLA_BOAT_PASSENGER_MAX_ROTATION,
            )
        );
        assert_eq!(
            passenger.previous_rotation(),
            (VANILLA_BOAT_PASSENGER_MAX_ROTATION, 0.0, 168.75)
        );
        assert_eq!(passenger.body_y_rotation(), 0.0);
    }
}
