use super::*;

#[test]
fn entity_teleport_applies_relative_delta_motion_and_rotation() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 1,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        movement: (1.0, 2.0, 3.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let relative_flags = ENTITY_RELATIVE_X_FLAG
        | ENTITY_RELATIVE_Y_FLAG
        | ENTITY_RELATIVE_Z_FLAG
        | ENTITY_RELATIVE_Y_ROT_FLAG
        | ENTITY_RELATIVE_X_ROT_FLAG
        | ENTITY_RELATIVE_DELTA_X_FLAG
        | ENTITY_RELATIVE_DELTA_Y_FLAG
        | ENTITY_RELATIVE_DELTA_Z_FLAG;
    assert!(world.apply_entity_teleport(EntityTeleportInput {
        entity_id: 37,
        x: 2.0,
        y: 3.0,
        z: 4.0,
        delta_x: 0.25,
        delta_y: -0.5,
        delta_z: 0.75,
        y_rot: 15.0,
        x_rot: 10.0,
        relative_flags,
    }));
    let entity = world.entity_at(37).expect("relative teleport entity");
    assert_eq!(entity.position(), (12.0, 67.0, 24.0));
    assert_eq!(entity.rotation(), (15.0, 10.0, 0.0));
    assert_eq!(entity.motion(), (1.25, 1.5, 3.75));

    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(3, 4),
        type_id: 1,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        movement: (1.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let rotate_flags = ENTITY_RELATIVE_Y_ROT_FLAG
        | ENTITY_RELATIVE_DELTA_X_FLAG
        | ENTITY_RELATIVE_DELTA_Z_FLAG
        | ENTITY_RELATIVE_ROTATE_DELTA_FLAG;
    assert!(world.apply_entity_teleport(EntityTeleportInput {
        entity_id: 38,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        delta_x: 0.0,
        delta_y: 0.0,
        delta_z: 0.0,
        y_rot: 90.0,
        x_rot: 0.0,
        relative_flags: rotate_flags,
    }));
    let entity = world.entity_at(38).expect("rotated teleport entity");
    let radians = (f64::from(-90.0_f32) * (std::f64::consts::PI / 180.0)) as f32;
    let expected_x = f64::from(mth_cos_f64(f64::from(radians)));
    let expected_z = -f64::from(mth_sin_f64(f64::from(radians)));
    assert_eq!(entity.rotation(), (90.0, 0.0, 0.0));
    assert_eq!(entity.motion(), (expected_x, 0.0, expected_z));

    assert!(world.apply_entity(EntityInput {
        entity_id: 39,
        uuid: Uuid::new(5, 6),
        type_id: 1,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        movement: (0.5, 0.25, -0.75),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_teleport(EntityTeleportInput {
        entity_id: 39,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        delta_x: f64::NAN,
        delta_y: 0.0,
        delta_z: 0.0,
        y_rot: 0.0,
        x_rot: 0.0,
        relative_flags: ENTITY_RELATIVE_DELTA_X_FLAG,
    }));
    assert_eq!(
        world
            .entity_at(39)
            .expect("nonfinite teleport entity")
            .motion(),
        (0.5, 0.25, -0.75)
    );
}

#[test]
fn remote_player_position_sync_snaps_when_distance_exceeds_java_threshold() {
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
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.0,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (350.0, -20.0, 350.0),
        data: 0,
    }));

    assert!(world.apply_entity_position_and_rotation(
        EntityPositionInput {
            entity_id: 37,
            x: 66.0,
            y: 80.0,
            z: -35.5,
            relative_flags: 0,
        },
        EntityRotationInput {
            entity_id: 37,
            y_rot: Some(10.0),
            x_rot: Some(20.0),
            y_head_rot: None,
            relative_y_rot: false,
            relative_x_rot: false,
        },
        EntityPositionUpdateMode::PositionSync,
    ));
    let entity = world.entity_at(37).expect("snapped remote player");
    assert_eq!(entity.position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.previous_position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.rotation(), (10.0, 20.0, 350.0));
    assert_eq!(entity.previous_rotation(), (10.0, 20.0, 350.0));
    assert_eq!(entity.interpolation_steps(), 0);
    assert_eq!(entity.interpolation_target_position(), None);
    assert_eq!(entity.interpolation_target_rotation(), None);

    world.tick();
    let entity = world.entity_at(37).expect("snapped remote player tick");
    assert_eq!(entity.position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.previous_position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.rotation(), (10.0, 20.0, 350.0));

    assert!(world.apply_entity_position_and_rotation(
        EntityPositionInput {
            entity_id: 37,
            x: 130.0,
            y: 80.0,
            z: -35.5,
            relative_flags: 0,
        },
        EntityRotationInput {
            entity_id: 37,
            y_rot: Some(30.0),
            x_rot: Some(10.0),
            y_head_rot: None,
            relative_y_rot: false,
            relative_x_rot: false,
        },
        EntityPositionUpdateMode::PositionSync,
    ));
    let entity = world.entity_at(37).expect("threshold remote player");
    assert_eq!(entity.position(), (66.0, 80.0, -35.5));
    assert_eq!(
        entity.interpolation_target_position(),
        Some((130.0, 80.0, -35.5))
    );
    assert_eq!(entity.interpolation_target_rotation(), Some((30.0, 10.0)));
    assert_eq!(entity.interpolation_steps(), 3);

    assert!(world.apply_entity_position_and_rotation(
        EntityPositionInput {
            entity_id: 37,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            relative_flags: 0b111,
        },
        EntityRotationInput {
            entity_id: 37,
            y_rot: Some(20.0),
            x_rot: Some(100.0),
            y_head_rot: None,
            relative_y_rot: true,
            relative_x_rot: true,
        },
        EntityPositionUpdateMode::Teleport,
    ));
    let entity = world.entity_at(37).expect("relative teleport target");
    assert_eq!(
        entity.interpolation_target_position(),
        Some((130.0, 80.0, -35.5))
    );
    assert_eq!(entity.interpolation_target_rotation(), Some((50.0, 90.0)));
}

#[test]
fn attribute_effective_value_matches_java_operation_order_and_sanitization() {
    let attribute = ClientAttributeState::from_wire(
        26,
        0.25,
        vec![
            ClientAttributeModifierState::from_wire("minecraft:oracle_add".to_owned(), 0.1, 0),
            ClientAttributeModifierState::from_wire("minecraft:oracle_base".to_owned(), 0.2, 1),
            ClientAttributeModifierState::from_wire("minecraft:oracle_total".to_owned(), 0.5, 2),
        ],
    );
    assert_eq!(
        attribute.effective_value(0.0, 1024.0).to_bits(),
        0x3fe4_28f5_c28f_5c29
    );

    let unknown_operation = ClientAttributeState::from_wire(
        26,
        1.0,
        vec![ClientAttributeModifierState::from_wire(
            "minecraft:oracle_unknown".to_owned(),
            0.5,
            255,
        )],
    );
    assert_eq!(unknown_operation.effective_value(0.0, 1024.0), 1.5);

    let clamped = ClientAttributeState::from_wire(26, 2048.0, Vec::new());
    assert_eq!(clamped.effective_value(0.0, 1024.0), 1024.0);

    let nan = ClientAttributeState::from_wire(26, f64::NAN, Vec::new());
    assert_eq!(nan.effective_value(0.0, 1024.0), 0.0);
}

#[test]
fn an_open_containers_last_slots_are_the_players_inventory() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let stone = ClientItemStackState::from_wire(7, 1, vec![], vec![]);
    let dirt = ClientItemStackState::from_wire(3, 10, vec![], vec![]);
    let mut items = vec![None; 63];
    // A main inventory slot (menu 9) and the first hotbar slot (menu 36),
    // which a three row chest lists at 27 and 54.
    items[27] = Some(stone.clone());
    items[54] = Some(dirt.clone());
    assert!(world.apply_local_player_open_menu(7, 2, &[10, 0, 0, 0]));
    assert!(world.apply_local_player_container_content(7, 1, &items, None));
    let player = world.local_player().expect("player");
    assert_eq!(player.inventory_slot(9), Some(&stone));
    assert_eq!(player.inventory_slot(36), Some(&dirt));

    // A click in the container's menu writes through both ways.
    let crate::OpenContainerMenu {
        mut slots, carried, ..
    } = world.local_player_container_menu().expect("menu");
    slots[54] = None;
    world.set_local_player_container_menu(&slots, carried);
    assert!(
        world
            .local_player()
            .expect("player")
            .inventory_slot(36)
            .is_none()
    );

    // And the server changing the inventory shows in the container's menu.
    assert!(world.apply_local_player_inventory_slot(1, Some(&dirt)));
    let crate::OpenContainerMenu { slots, .. } = world.local_player_container_menu().expect("menu");
    assert_eq!(slots[55].as_ref(), Some(&dirt));
}

#[test]
fn vanilla_menu_registry_slot_counts_cover_all_builtin_menu_types() {
    let expected = [
        (0, 9),
        (1, 18),
        (2, 27),
        (3, 36),
        (4, 45),
        (5, 54),
        (6, 9),
        (7, 10),
        (8, 3),
        (9, 1),
        (10, 3),
        (11, 5),
        (12, 10),
        (13, 2),
        (14, 3),
        (15, 3),
        (16, 5),
        (17, 1),
        (18, 4),
        (19, 3),
        (20, 27),
        (21, 4),
        (22, 3),
        (23, 3),
        (24, 2),
    ];

    for (menu_type_id, slot_count) in expected {
        assert_eq!(menu_top_slot_count(menu_type_id), Some(slot_count));
    }
    assert_eq!(menu_top_slot_count(-1), None);
    assert_eq!(menu_top_slot_count(25), None);
}

#[test]
fn changing_dimension_keeps_local_player_identity_until_respawn_replacement() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(17);
    let input = DimensionInput {
        key: "minecraft:the_nether",
        dimension_type_id: 3,
        seed: 776,
        is_debug: false,
        is_flat: false,
        sea_level: 32,
    };

    assert!(world.replace_dimension_with_environment(
        input,
        DimensionBounds::new(0, 256),
        DimensionEnvironment::for_builtin_dimension(input.key),
    ));
    assert_eq!(world.local_player().map(|player| player.id()), Some(17));
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(0)
    );
    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(world.local_player().map(|player| player.id()), Some(17));
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(1)
    );
}

#[test]
fn changing_dimension_replaces_world_storage_and_builtin_bounds() {
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
    assert!(world.apply_entity(EntityInput {
        entity_id: 9,
        uuid: Uuid::new(5, 6),
        type_id: 71,
        x: 0.0,
        y: 64.0,
        z: 0.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.tick();

    let input = DimensionInput {
        key: "minecraft:the_nether",
        dimension_type_id: 1,
        seed: 776,
        is_debug: false,
        is_flat: false,
        sea_level: 32,
    };
    assert!(world.replace_dimension_with_environment(
        input,
        DimensionBounds::for_builtin_dimension(input.key),
        DimensionEnvironment::for_builtin_dimension(input.key),
    ));
    assert_eq!(world.dimension_key(), "minecraft:the_nether");
    assert_eq!(world.dimension(), DimensionBounds::new(0, 256));
    assert_eq!(world.dimension_type_id(), 1);
    assert_eq!(world.seed(), 776);
    assert_eq!(world.sea_level(), 32);
    assert_eq!(world.loaded_chunk_count(), 0);
    assert_eq!(world.loaded_entity_count(), 0);
    assert_eq!(world.game_time(), 0);

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
    assert!(!world.replace_dimension_with_environment(
        input,
        DimensionBounds::for_builtin_dimension(input.key),
        DimensionEnvironment::for_builtin_dimension(input.key),
    ));
    assert_eq!(world.loaded_chunk_count(), 1);
    assert_eq!(world.loaded_entity_count(), 0);
}
