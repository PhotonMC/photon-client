use super::super::*;

#[test]
fn player_entities_keep_ticking_after_their_chunk_unloads() {
    let mut world = ClientWorld::default();
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
        entity_id: 42,
        uuid: Uuid::new(5, 6),
        type_id: crate::VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(
        world
            .entities()
            .at(42)
            .is_some_and(|entity| { entity.is_always_ticking() && entity.is_ticking() })
    );

    assert!(world.unload_chunk(0, -3));
    assert!(
        world
            .entities()
            .at(42)
            .is_some_and(|entity| { entity.is_always_ticking() && entity.is_ticking() })
    );
    world.tick();
    assert_eq!(
        world.entities().at(42).map(crate::ClientEntity::tick_count),
        Some(1)
    );
}

#[test]
fn spawner_state_decodes_java_update_tag_and_defaults() {
    use voxel_core::nbt::{Field, FieldValue};

    let mut tag = Vec::new();
    voxel_core::nbt::write_nbt(
        Some(&[
            Field {
                key: "Delay".to_owned(),
                value: FieldValue::Int(65_535),
            },
            Field {
                key: "MaxSpawnDelay".to_owned(),
                value: FieldValue::Short(640),
            },
            Field {
                key: "MinSpawnDelay".to_owned(),
                value: FieldValue::Short(120),
            },
            Field {
                key: "SpawnCount".to_owned(),
                value: FieldValue::Str("invalid numeric tag".to_owned()),
            },
        ]),
        &mut tag,
    );

    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::default();
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
    assert_eq!(
        world.apply_block_entities(
            0,
            0,
            [BlockEntityInput {
                x: 1,
                y: -63,
                z: 2,
                type_id: 9,
                tag: Some(&tag),
            }],
        ),
        1
    );

    let state = world
        .block_entity_at(1, -63, 2)
        .and_then(crate::ClientBlockEntity::spawner_state)
        .expect("spawner tag should be retained as semantic state");
    assert_eq!(state.delay(), -1);
    assert_eq!(state.min_spawn_delay(), 120);
    assert_eq!(state.max_spawn_delay(), 640);
    assert_eq!(state.spawn_count(), 4);
    assert_eq!(state.max_nearby_entities(), 6);
    assert_eq!(state.required_player_range(), 16);
    assert_eq!(state.spawn_range(), 4);

    let non_spawner = world.block_entity_at(1, -63, 2).unwrap();
    assert_eq!(non_spawner.type_id, 9);
}

#[test]
fn entities_are_queryable_and_replaced_by_entity_id() {
    let mut world = ClientWorld::default();
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
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 120,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.125, -0.25, 0.5),
        rotation: (12.5, -30.0, 45.0),
        data: 6,
    }));
    assert_eq!(world.entities().count(), 1);
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::position),
        Some((1.25, 80.0, -35.5))
    );
    assert_eq!(
        world.entities().at(37).map(|entity| entity.type_id),
        Some(120)
    );
    assert_eq!(world.entities().at(37).map(|entity| entity.data), Some(6));
    assert_eq!(
        world.entities().at(37).map(|entity| entity.uuid),
        Some(Uuid::new(1, 2))
    );
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::motion),
        Some((0.125, -0.25, 0.5))
    );
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::rotation),
        Some((12.5, -30.0, 45.0))
    );
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::on_ground),
        Some(false)
    );
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::is_ticking),
        Some(true)
    );
    assert!(world.unload_chunk(0, -3));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::is_ticking),
        Some(false)
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
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::is_ticking),
        Some(true)
    );
    assert_eq!(world.entities().metadata(37), None);
    assert!(world.apply_entity_metadata(37, &[1, 0, 255]));
    assert_eq!(world.entities().metadata(37), Some(&[1, 0, 255][..]));
    assert!(!world.apply_entity_metadata(999, &[255]));
    assert!(world.entities_mut().apply_motion(EntityMotionInput {
        entity_id: 37,
        x: -0.5,
        y: 0.25,
        z: 0.75,
    }));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::motion),
        Some((-0.5, 0.25, 0.75))
    );
    assert!(!world.entities_mut().apply_motion(EntityMotionInput {
        entity_id: 999,
        x: 1.0,
        y: 2.0,
        z: 3.0,
    }));
    assert!(world.apply_entity_ground_state(37, true));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::on_ground),
        Some(true)
    );
    assert!(world.apply_entity_ground_state(37, false));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::on_ground),
        Some(false)
    );
    assert!(!world.apply_entity_ground_state(999, true));
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: 37,
        y_rot: Some(90.0),
        x_rot: Some(-15.0),
        y_head_rot: Some(135.0),
        relative_y_rot: false,
        relative_x_rot: false,
    }));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::rotation),
        Some((90.0, -15.0, 135.0))
    );
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: 37,
        y_rot: Some(45.0),
        x_rot: Some(10.0),
        y_head_rot: None,
        relative_y_rot: true,
        relative_x_rot: true,
    }));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::rotation),
        Some((135.0, -5.0, 135.0))
    );
    assert!(!world.apply_entity_rotation(EntityRotationInput {
        entity_id: 999,
        y_rot: Some(1.0),
        x_rot: Some(2.0),
        y_head_rot: Some(3.0),
        relative_y_rot: false,
        relative_x_rot: false,
    }));

    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: 37,
        x: 10.0,
        y: 20.0,
        z: 30.0,
        relative_flags: 0,
    }));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::position),
        Some((10.0, 20.0, 30.0))
    );
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::is_ticking),
        Some(false)
    );
    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: 37,
        x: 0.5,
        y: -1.0,
        z: 2.0,
        relative_flags: 0b111,
    }));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::position),
        Some((10.5, 19.0, 32.0))
    );
    assert!(!world.apply_entity_position(EntityPositionInput {
        entity_id: 999,
        x: 1.0,
        y: 2.0,
        z: 3.0,
        relative_flags: 0,
    }));
    assert!(world.apply_entity_move(EntityMoveInput {
        entity_id: 37,
        delta_x: 2_048,
        delta_y: -4_096,
        delta_z: 0,
        has_position: true,
        y_rot: None,
        x_rot: None,
    }));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::position),
        Some((11.0, 18.0, 32.0))
    );
    assert!(world.apply_entity_move(EntityMoveInput {
        entity_id: 37,
        delta_x: 7,
        delta_y: 8,
        delta_z: 9,
        has_position: false,
        y_rot: None,
        x_rot: None,
    }));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::position),
        Some((11.0, 18.0, 32.0))
    );
    world.replace_chunk(
        1,
        2,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity_move(EntityMoveInput {
        entity_id: 37,
        delta_x: 32_767,
        delta_y: 0,
        delta_z: 0,
        has_position: true,
        y_rot: None,
        x_rot: None,
    }));
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::is_ticking),
        Some(true)
    );

    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(3, 4),
        type_id: 71,
        x: -4.0,
        y: 65.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(world.entities().count(), 1);
    assert_eq!(
        world.entities().at(37).map(|entity| entity.type_id),
        Some(71)
    );
    assert_eq!(
        world.entities().at(37).map(|entity| entity.uuid),
        Some(Uuid::new(3, 4))
    );
    assert_eq!(
        world.entities().at(37).map(crate::ClientEntity::position),
        Some((-4.0, 65.0, 2.0))
    );
    assert_eq!(world.entities().metadata(37), None);
    assert!(world.remove_entity(37));
    assert_eq!(world.entities().count(), 0);
    assert!(!world.remove_entity(37));
}

#[test]
fn entity_passenger_state_replaces_previous_vehicle_and_cleans_relationships() {
    let mut world = ClientWorld::default();
    for entity_id in [37, 38, 39, 40] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id: 71,
            x: 0.0,
            y: 64.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    world.set_local_player_id(42);

    assert!(world.apply_entity_passengers(39, &[38]));
    assert_eq!(world.entity_passengers(39), Some(&[38][..]));
    assert_eq!(world.entity_vehicle_id(38), Some(39));

    assert!(world.apply_entity_passengers(37, &[38, 39, 999, 38, 42]));
    assert_eq!(world.entity_passengers(37), Some(&[38, 39, 42][..]));
    assert_eq!(world.entity_passengers(39), Some(&[][..]));
    assert_eq!(world.entity_vehicle_id(38), Some(37));
    assert_eq!(world.entity_vehicle_id(39), Some(37));
    assert_eq!(world.entity_vehicle_id(42), Some(37));
    assert!(world.apply_entity_passengers(39, &[37]));
    assert_eq!(world.entity_passengers(39), Some(&[][..]));
    assert_eq!(world.entity_passengers(37), Some(&[38, 39, 42][..]));

    assert!(world.apply_entity_passengers(37, &[]));
    assert_eq!(world.entity_passengers(37), Some(&[][..]));
    assert_eq!(world.entity_vehicle_id(38), None);
    assert_eq!(world.entity_vehicle_id(39), None);
    assert_eq!(world.entity_vehicle_id(42), None);
    assert!(!world.apply_entity_passengers(999, &[38]));

    assert!(world.apply_entity_passengers(37, &[38]));
    assert!(world.remove_entity(37));
    assert_eq!(world.entity_vehicle_id(38), None);
    assert_eq!(world.entities().at(37), None);
}
