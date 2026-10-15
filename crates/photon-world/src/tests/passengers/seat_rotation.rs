use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn passenger_tick_applies_animal_rotation_offset_by_entity_parity_at_capacity() {
    for (animal_id, expected_offset) in [(38, 90.0_f32), (39, 270.0_f32)] {
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
            (animal_id, VANILLA_COW_ENTITY_TYPE_ID, (170.0, 0.0, 170.0)),
            (41, VANILLA_SNOWBALL_ENTITY_TYPE_ID, (0.0, 0.0, 0.0)),
        ] {
            assert!(world.apply_entity(EntityInput {
                entity_id: EntityId(entity_id),
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
        assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(animal_id), EntityId(41)]));

        world.tick();

        let passenger = world
            .entities()
            .at(EntityId(animal_id))
            .expect("Cow passenger");
        assert_eq!(passenger.rotation(), (105.0, 0.0, 105.0 + expected_offset));
        assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 170.0));
        assert_eq!(passenger.body_y_rotation(), expected_offset);
        assert_eq!(passenger.previous_body_y_rotation(), 170.0);
    }
}

#[test]
fn passenger_tick_positions_animal_on_full_boat_seat_offset() {
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
        (38, VANILLA_COW_ENTITY_TYPE_ID, (2.0, 12.0, 2.0)),
        (41, VANILLA_SNOWBALL_ENTITY_TYPE_ID, (2.0, 12.0, 2.0)),
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
    assert!(world.apply_entity_passengers(EntityId(37), &[EntityId(38), EntityId(41)]));

    world.tick();

    let passenger = world.entities().at(EntityId(38)).expect("Cow passenger");
    assert_eq!(
        passenger.position(),
        (2.0, 2.1875, 2.0 + f64::from(0.2_f32 + 0.2_f32))
    );
}
