use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn air_boat_float_applies_java_drag_gravity_and_position_step() {
    for (name, no_gravity, expected_motion_y, expected_position_y) in [
        ("falling", false, 0.46_f64, 8.46_f64),
        ("no_gravity", true, 0.5_f64, 8.5_f64),
    ] {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            0,
            -1,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(37),
            uuid: Uuid::new(1, 37),
            type_id: photon_core::ids::EntityTypeId(89),
            x: 12.5,
            y: 8.0,
            z: -4.25,
            movement: (0.25, 0.5, -0.75),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        world.set_local_player_id(EntityId(38));
        assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));
        if no_gravity {
            assert!(
                world.apply_entity_metadata(EntityId(37), &[ENTITY_NO_GRAVITY_DATA_ID, 8, 1, 255])
            );
        }

        world.tick();

        let boat = world.entities().at(EntityId(37)).expect("air boat");
        assert_eq!(boat.motion().0.to_bits(), 0x3fcc_cccc_c000_0000, "{name}");
        assert_eq!(
            boat.motion().1.to_bits(),
            expected_motion_y.to_bits(),
            "{name}"
        );
        assert_eq!(boat.motion().2.to_bits(), 0xbfe5_9999_9000_0000, "{name}");
        assert_eq!(boat.position().0.to_bits(), 0x4029_7333_3300_0000, "{name}");
        assert_eq!(
            boat.position().1.to_bits(),
            expected_position_y.to_bits(),
            "{name}"
        );
        assert_eq!(boat.position().2.to_bits(), 0xc013_b333_3200_0000, "{name}");
    }
}

#[test]
fn remote_boat_tick_does_not_apply_client_control_or_motion() {
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
        uuid: Uuid::new(1, 37),
        type_id: photon_core::ids::EntityTypeId(89),
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.25, 0.5, -0.75),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(38),
        uuid: Uuid::new(1, 38),
        type_id: photon_core::ids::EntityTypeId(151),
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));
    assert!(world.set_boat_input(EntityId(37), BoatInput::new(false, true, false, false)));

    world.tick();

    let boat = world.entities().at(EntityId(37)).expect("remote boat");
    assert_eq!(boat.position(), (2.0, 12.0, 2.0));
    assert_eq!(boat.motion(), (0.0, 0.0, 0.0));
    assert_eq!(boat.rotation(), (0.0, 0.0, 0.0));
    assert_eq!(boat.boat_delta_rotation(), 0.0);
    assert_eq!(boat.paddle_state(), (false, false));
}

#[test]
fn a_turn_through_the_half_circle_is_drawn_the_short_way_round() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 37),
        type_id: photon_core::ids::EntityTypeId(89),
        x: 0.0,
        y: 0.0,
        z: 0.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (179.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entities_mut().at_mut(EntityId(37)).expect("the boat");
    entity.old_y_rot = photon_core::java_float::JavaF32::new(179.0);
    entity.y_rot = photon_core::java_float::JavaF32::new(-179.0);
    // Halfway between 179 and -179 is the far side (180), not the front (0).
    assert!((entity.y_rot_at(0.5).abs() - 180.0).abs() < 1.0e-3);
}

#[test]
fn a_passenger_of_an_armor_stand_follows_it() {
    let stand = photon_data::entity_type::id("minecraft:armor_stand").expect("the armor stand");
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
    for (id, y) in [(37, 5.0), (38, 5.0)] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(id),
            uuid: Uuid::new(1, u64::from(id as u32)),
            type_id: stand,
            x: 0.0,
            y,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));
    let vehicle = world.entities_mut().at_mut(EntityId(37)).expect("vehicle");
    vehicle.set_position((0.0, 9.0, 0.0));
    world.tick();
    let rider = world.entities().at(EntityId(38)).expect("rider");
    // An armor stand is 1.975 blocks tall (`EntityType.ARMOR_STAND`).
    assert_eq!(rider.position().1, 9.0 + f64::from(1.975_f32));
}

#[test]
fn a_boat_the_local_player_steers_ignores_the_servers_moves() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 37),
        type_id: photon_core::ids::EntityTypeId(89),
        x: 12.5,
        y: 8.0,
        z: -4.25,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.set_local_player_id(EntityId(38));
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));
    world.apply_entity_move(EntityMoveInput {
        entity_id: EntityId(37),
        delta_x: 4_096,
        delta_y: 0,
        delta_z: 0,
        has_position: true,
        y_rot: None,
        x_rot: None,
    });
    let boat = world.entities().at(EntityId(37)).expect("the boat");
    assert_eq!(boat.interpolation_position(), (12.5, 8.0, -4.25));
}
