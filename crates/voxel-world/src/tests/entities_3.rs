use super::*;

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
            entity_id: 37,
            uuid: Uuid::new(1, 37),
            type_id: 89,
            x: 12.5,
            y: 8.0,
            z: -4.25,
            movement: (0.25, 0.5, -0.75),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        world.set_local_player_id(38);
        assert!(world.apply_entity_passengers(37, &[38]));
        if no_gravity {
            assert!(world.apply_entity_metadata(37, &[ENTITY_NO_GRAVITY_DATA_ID, 8, 1, 255]));
        }

        world.tick();

        let boat = world.entity_at(37).expect("air boat");
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
        entity_id: 37,
        uuid: Uuid::new(1, 37),
        type_id: 89,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.25, 0.5, -0.75),
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
    assert!(world.apply_entity_passengers(37, &[38]));
    assert!(world.set_boat_input(37, BoatInput::new(false, true, false, false)));

    world.tick();

    let boat = world.entity_at(37).expect("remote boat");
    assert_eq!(boat.position(), (2.0, 12.0, 2.0));
    assert_eq!(boat.motion(), (0.0, 0.0, 0.0));
    assert_eq!(boat.rotation(), (0.0, 0.0, 0.0));
    assert_eq!(boat.boat_delta_rotation(), 0.0);
    assert_eq!(boat.paddle_state(), (false, false));
}

#[test]
fn entity_shared_flags_follow_java_byte_metadata_and_preserve_unspecified_values() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 120,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let all_defined_flags = 0b1111_1011;
    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_SHARED_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            all_defined_flags,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("entity with shared flags");
    assert_eq!(entity.shared_flags(), all_defined_flags);
    assert!(entity.is_on_fire());
    assert!(entity.is_shift_key_down());
    assert!(entity.is_sprinting());
    assert!(entity.is_swimming());
    assert!(entity.is_invisible());
    assert!(entity.is_glowing());
    assert!(entity.is_fall_flying());

    assert!(world.apply_entity_metadata(37, &[1, 0, 7, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::shared_flags),
        Some(all_defined_flags)
    );

    assert!(world.apply_entity_metadata(37, &[0, 0, 0, 255]));
    let entity = world.entity_at(37).expect("cleared shared flags");
    assert_eq!(entity.shared_flags(), 0);
    assert!(!entity.is_on_fire());
    assert!(!entity.is_shift_key_down());
    assert!(!entity.is_sprinting());
    assert!(!entity.is_swimming());
    assert!(!entity.is_invisible());
    assert!(!entity.is_glowing());
    assert!(!entity.is_fall_flying());
}

#[test]
fn entity_air_supply_follows_java_int_metadata_and_preserves_shared_flags() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 120,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::air_supply),
        Some(300)
    );

    let mut air_metadata = vec![
        crate::ENTITY_AIR_SUPPLY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(250, &mut air_metadata);
    air_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &air_metadata));
    let entity = world.entity_at(37).expect("entity with air supply");
    assert_eq!(entity.air_supply(), 250);
    assert_eq!(entity.shared_flags(), 0);

    let mut combined_metadata = vec![
        crate::ENTITY_SHARED_FLAGS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        64,
        crate::ENTITY_AIR_SUPPLY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(-1, &mut combined_metadata);
    combined_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &combined_metadata));
    let entity = world.entity_at(37).expect("entity with combined metadata");
    assert_eq!(entity.air_supply(), -1);
    assert_eq!(entity.shared_flags(), 64);
    assert!(entity.is_glowing());

    assert!(world.apply_entity_metadata(37, &[0, 0, 0, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::air_supply),
        Some(-1)
    );
}

#[test]
fn entity_no_gravity_follows_java_boolean_metadata_and_preserves_air_supply() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 78,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let mut no_gravity_metadata = vec![
        crate::ENTITY_NO_GRAVITY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        1,
        255,
    ];
    assert!(world.apply_entity_metadata(37, &no_gravity_metadata));
    let entity = world.entity_at(37).expect("no-gravity entity");
    assert!(entity.is_no_gravity());
    assert_eq!(entity.air_supply(), 300);

    no_gravity_metadata = vec![
        crate::ENTITY_SHARED_FLAGS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        64,
        crate::ENTITY_NO_GRAVITY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        0,
        255,
    ];
    assert!(world.apply_entity_metadata(37, &no_gravity_metadata));
    let entity = world.entity_at(37).expect("cleared no-gravity entity");
    assert!(!entity.is_no_gravity());
    assert_eq!(entity.shared_flags(), 64);
    assert!(entity.is_glowing());

    let mut air_metadata = vec![1, 1];
    voxel_core::wire::write_varint(250, &mut air_metadata);
    air_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &air_metadata));
    let entity = world.entity_at(37).expect("air supply entity");
    assert_eq!(entity.air_supply(), 250);
    assert!(!entity.is_no_gravity());
}

#[test]
fn entity_silent_follows_java_boolean_metadata_and_preserves_other_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 5,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(!world.entity_at(37).expect("entity").is_silent());

    let metadata = [4, 8, 1, 255];
    assert!(world.apply_entity_metadata(37, &metadata));
    let entity = world.entity_at(37).expect("silent entity");
    assert!(entity.is_silent());
    assert!(!entity.is_no_gravity());
    assert_eq!(entity.air_supply(), 300);

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    let entity = world.entity_at(37).expect("silent no-gravity entity");
    assert!(entity.is_silent());
    assert!(entity.is_no_gravity());

    assert!(world.apply_entity_metadata(37, &[4, 8, 0, 255]));
    let entity = world.entity_at(37).expect("audible entity");
    assert!(!entity.is_silent());
    assert!(entity.is_no_gravity());
}

#[test]
fn entity_custom_name_visibility_follows_java_boolean_metadata() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 5,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default name visibility entity");
    assert!(!entity.is_custom_name_visible());
    assert!(!entity.should_show_name());

    let mut visible_metadata = vec![
        2,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
        1,
    ];
    voxel_core::nbt::write_any_nbt(
        Some(&voxel_core::nbt::FieldValue::Str("Display name".to_owned())),
        &mut visible_metadata,
    );
    visible_metadata.extend([
        crate::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        1,
        255,
    ]);
    assert!(world.apply_entity_metadata(37, &visible_metadata));
    let entity = world.entity_at(37).expect("visible name entity");
    assert!(entity.is_custom_name_visible());
    assert!(entity.should_show_name());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            0,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("hidden name entity");
    assert!(!entity.is_custom_name_visible());
    assert!(!entity.should_show_name());
}

#[test]
fn entity_custom_name_presence_follows_java_optional_component_metadata() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 5,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default custom-name entity");
    assert!(!entity.has_custom_name());
    assert!(!entity.is_custom_name_visible());

    let mut custom_name_metadata = vec![
        crate::ENTITY_CUSTOM_NAME_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
        1,
    ];
    voxel_core::nbt::write_any_nbt(
        Some(&voxel_core::nbt::FieldValue::Str("Display name".to_owned())),
        &mut custom_name_metadata,
    );
    custom_name_metadata.extend([
        crate::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        1,
        255,
    ]);
    assert!(world.apply_entity_metadata(37, &custom_name_metadata));
    let entity = world.entity_at(37).expect("named entity");
    assert!(entity.has_custom_name());
    assert!(entity.is_custom_name_visible());
    assert!(entity.should_show_name());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_CUSTOM_NAME_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
            0,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("unnamed entity");
    assert!(!entity.has_custom_name());
    assert!(entity.is_custom_name_visible());
    assert!(entity.should_show_name());
}

#[test]
fn entity_health_follows_java_float_metadata_and_preserves_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 54,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world.entity_at(37).and_then(crate::ClientEntity::health),
        None
    );

    let mut health_metadata = vec![
        8,
        crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        0,
        crate::ENTITY_HEALTH_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
    ];
    health_metadata.extend_from_slice(&7.5_f32.to_be_bytes());
    health_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &health_metadata));
    let entity = world.entity_at(37).expect("living entity health");
    assert_eq!(entity.health(), Some(7.5));
    assert_eq!(entity.shared_flags(), 0);

    let mut zero_health_metadata = vec![
        crate::ENTITY_HEALTH_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
    ];
    zero_health_metadata.extend_from_slice(&0.0_f32.to_be_bytes());
    zero_health_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &zero_health_metadata));
    assert_eq!(
        world.entity_at(37).and_then(crate::ClientEntity::health),
        Some(0.0)
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    let entity = world.entity_at(37).expect("health after partial metadata");
    assert_eq!(entity.health(), Some(0.0));
    assert!(entity.is_no_gravity());
}
