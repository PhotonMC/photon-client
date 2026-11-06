use super::super::*;
use crate::entity::{vanilla_max_head_y_rot, vanilla_passenger_is_mob};
use crate::entity_type_groups::entity_type_id as ty;
use photon_core::ids::{EntityId, EntityTypeId};

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
        entity_id: EntityId(37),
        uuid: Uuid::new(37, 0),
        type_id: photon_data::entity_type::named("minecraft:oak_boat"),
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
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (140.0, 0.0, 140.0),
        data: 0,
    }));
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    world.tick();

    let passenger = world
        .entities()
        .at(EntityId(38))
        .expect("remote player passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 140.0));
    assert_eq!(passenger.previous_rotation(), (140.0, 0.0, 140.0));
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
        (37, ty("minecraft:oak_boat"), (0.0, 0.0, 0.0)),
        (38, VANILLA_BREEZE_ENTITY_TYPE_ID.0, (170.0, 0.0, 170.0)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, 0),
            type_id: EntityTypeId(type_id),
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    world.tick();

    let passenger = world.entities().at(EntityId(38)).expect("Breeze passenger");
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
    for (entity_id, type_id, rotation) in [
        (37, ty("minecraft:oak_boat"), (0.0, 0.0, 0.0)),
        (38, ty("minecraft:item"), (170.0, 0.0, 170.0)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, 0),
            type_id: EntityTypeId(type_id),
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    world.tick();

    let passenger = world
        .entities()
        .at(EntityId(38))
        .expect("generic boat passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 170.0));
    assert_eq!(passenger.previous_rotation(), (170.0, 0.0, 170.0));
    assert_eq!(passenger.body_y_rotation(), 170.0);
}

#[test]
fn passenger_tick_applies_animal_living_rotation_before_capacity_offset() {
    for (vehicle_type_id, expected_yaw, expected_head, expected_body) in [
        (ty("minecraft:oak_boat"), 0.0_f32, 75.0_f32, 0.0_f32),
        (
            ty("minecraft:oak_chest_boat"),
            90.0_f32,
            165.0_f32,
            90.0_f32,
        ),
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
        for (entity_id, type_id, rotation) in [
            (37, vehicle_type_id, (0.0, 0.0, 0.0)),
            (38, VANILLA_COW_ENTITY_TYPE_ID.0, (170.0, 0.0, 170.0)),
        ] {
            assert!(world.apply_entity(EntityInput {
                entity_id: EntityId(entity_id),
                uuid: Uuid::new(entity_id as u64, 0),
                type_id: EntityTypeId(type_id),
                x: 2.0,
                y: 2.0,
                z: 2.0,
                movement: (0.0, 0.0, 0.0),
                rotation,
                data: 0,
            }));
        }
        assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

        world.tick();

        let passenger = world.entities().at(EntityId(38)).expect("Cow passenger");
        assert_eq!(passenger.rotation(), (expected_yaw, 0.0, expected_head));
        assert_eq!(passenger.previous_rotation(), (170.0, 0.0, 170.0));
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
        (37, ty("minecraft:oak_boat"), (0.0, 0.0, 0.0)),
        (38, VANILLA_PLAYER_ENTITY_TYPE_ID.0, (168.75, 0.0, 168.75)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, 0),
            type_id: EntityTypeId(type_id),
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    assert!(world.turn_entity(EntityId(38), -100.0, 0.0));

    let passenger = world.entities().at(EntityId(38)).expect("Player passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 168.75));
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
        (37, ty("minecraft:oak_boat"), (0.0, 0.0, 0.0)),
        (38, VANILLA_BREEZE_ENTITY_TYPE_ID.0, (168.75, 0.0, 168.75)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, 0),
            type_id: EntityTypeId(type_id),
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38)]));

    assert!(world.turn_entity(EntityId(38), -100.0, 0.0));

    let passenger = world.entities().at(EntityId(38)).expect("Breeze passenger");
    assert_eq!(passenger.rotation(), (0.0, 0.0, 30.0));
    assert_eq!(passenger.previous_rotation(), (153.75, 0.0, 168.75));
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
    for (entity_id, type_id) in [
        (37, ty("minecraft:oak_boat")),
        (38, ty("minecraft:zombie")),
        (39, VANILLA_PLAYER_ENTITY_TYPE_ID.0),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(1, entity_id as u64),
            type_id: EntityTypeId(type_id),
            x: 2.0,
            y: 12.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    world.set_local_player_id(EntityId(39));
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(39), EntityId(38)]));
    assert!(world.set_boat_input(EntityId(37), BoatInput::new(false, true, false, false)));

    world.tick();

    let passenger = world
        .entities()
        .at(EntityId(38))
        .expect("living boat passenger");
    assert_eq!(passenger.rotation(), (1.0, 0.0, 0.0));
    assert_eq!(passenger.previous_rotation(), (0.0, 0.0, 0.0));
    assert_eq!(passenger.body_y_rotation(), 1.0);
    // The boat moved as it ticked, and the passenger is carried to where it now stands.
    let boat = world.entities().at(EntityId(37)).expect("boat").position();
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
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 37),
        type_id: photon_data::entity_type::named("minecraft:oak_boat"),
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(38),
        uuid: Uuid::new(1, 38),
        type_id: photon_data::entity_type::named("minecraft:zombie"),
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(39),
        uuid: Uuid::new(1, 39),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.set_local_player_id(EntityId(39));
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38), EntityId(39)]));
    assert!(world.set_boat_input(EntityId(37), BoatInput::new(false, true, false, false)));

    world.tick();

    let boat = world.entities().at(EntityId(37)).expect("boat");
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
        entity_id: EntityId(37),
        uuid: Uuid::new(37, 0),
        type_id: photon_data::entity_type::named("minecraft:oak_boat"),
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
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, 0),
            type_id: EntityTypeId(type_id),
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (168.75, 0.0, 168.75),
            data: 0,
        }));
    }
    for (index, &raw_type_id) in VANILLA_LIVING_ENTITY_TYPE_IDS.iter().enumerate() {
        let passenger_id = 100 + index as i32;
        assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(passenger_id)]));
        assert!(world.turn_entity(EntityId(passenger_id), -100.0, 0.0));
        let passenger = world
            .entities()
            .at(EntityId(passenger_id))
            .expect("living passenger");
        // A mob's yaw becomes its body's and its head is held to its own limit; anyone else
        // keeps the head and has the yaw held within a quarter turn.
        let type_id = EntityTypeId(raw_type_id);
        if vanilla_passenger_is_mob(type_id) {
            let limit = vanilla_max_head_y_rot(type_id);
            assert_eq!(passenger.rotation(), (0.0, 0.0, limit.min(168.75)));
            assert_eq!(passenger.previous_rotation(), (153.75, 0.0, 168.75));
        } else {
            assert_eq!(
                passenger.rotation(),
                (VANILLA_BOAT_PASSENGER_MAX_ROTATION, 0.0, 168.75)
            );
            assert_eq!(
                passenger.previous_rotation(),
                (VANILLA_BOAT_PASSENGER_MAX_ROTATION, 0.0, 168.75)
            );
        }
        assert_eq!(passenger.body_y_rotation(), 0.0);
    }
}
