use super::*;

#[test]
fn entity_sleeping_position_follows_java_optional_block_pos_metadata() {
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
    let entity = world.entity_at(37).expect("default sleeping entity");
    assert_eq!(entity.sleeping_pos(), None);
    assert!(!entity.is_sleeping());

    let bed = voxel_core::binary::BlockPos::new(-12, 90, 37);
    let mut sleeping_metadata = vec![
        crate::ENTITY_SLEEPING_POS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
        1,
    ];
    voxel_core::binary::write_blockpos(bed, &mut sleeping_metadata);
    sleeping_metadata.extend([3, 8, 1, 255]);
    assert!(world.apply_entity_metadata(37, &sleeping_metadata));
    let entity = world.entity_at(37).expect("sleeping entity");
    assert_eq!(entity.sleeping_pos(), Some(bed));
    assert!(entity.is_sleeping());
    assert!(entity.is_custom_name_visible());

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("sleeping entity after partial metadata")
            .sleeping_pos(),
        Some(bed)
    );

    assert!(world.apply_entity_metadata(37, &[14, 11, 0, 255]));
    assert_eq!(
        world.entity_at(37).expect("awake entity").sleeping_pos(),
        None
    );

    let mut wrong_serializer = vec![14, 10, 1];
    voxel_core::binary::write_blockpos(bed, &mut wrong_serializer);
    wrong_serializer.push(255);
    assert!(world.apply_entity_metadata(37, &wrong_serializer));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after wrong serializer")
            .sleeping_pos(),
        None
    );

    assert!(world.apply_entity_metadata(37, &[14, 11, 1, 0, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after truncated sleeping position")
            .sleeping_pos(),
        None
    );
}

#[test]
fn entity_living_flags_follow_java_byte_metadata_masks() {
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
    let entity = world.entity_at(37).expect("default living flags entity");
    assert_eq!(entity.living_flags(), 0);
    assert!(!entity.is_using_item());
    assert!(!entity.used_item_is_off_hand());
    assert!(!entity.is_auto_spin_attack());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_LIVING_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b111,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("all living flags entity");
    assert_eq!(entity.living_flags(), 0b111);
    assert!(entity.is_using_item());
    assert!(entity.used_item_is_off_hand());
    assert!(entity.is_auto_spin_attack());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_LIVING_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b010,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("off-hand living flags entity");
    assert_eq!(entity.living_flags(), 0b010);
    assert!(!entity.is_using_item());
    assert!(entity.used_item_is_off_hand());
    assert!(!entity.is_auto_spin_attack());

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    let entity = world
        .entity_at(37)
        .expect("living flags after partial metadata");
    assert_eq!(entity.living_flags(), 0b010);
    assert!(entity.is_no_gravity());
}

#[test]
fn mob_flags_follow_java_byte_metadata_masks_and_preserve_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 70,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default mob flags entity");
    assert_eq!(entity.mob_flags(), 0);
    assert!(!entity.is_no_ai());
    assert!(!entity.is_left_handed());
    assert!(!entity.is_aggressive());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_MOB_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b111,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("all mob flags entity");
    assert_eq!(entity.mob_flags(), 0b111);
    assert!(entity.is_no_ai());
    assert!(entity.is_left_handed());
    assert!(entity.is_aggressive());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_MOB_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b001,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("no-ai mob flags entity");
    assert_eq!(entity.mob_flags(), 0b001);
    assert!(entity.is_no_ai());
    assert!(!entity.is_left_handed());
    assert!(!entity.is_aggressive());

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    let entity = world
        .entity_at(37)
        .expect("mob flags after partial metadata");
    assert_eq!(entity.mob_flags(), 0b001);
    assert!(entity.is_no_ai());
    assert!(entity.is_no_gravity());

    assert!(world.apply_entity_metadata(37, &[15, 8, 0, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::mob_flags),
        Some(0b001)
    );
}

#[test]
fn entity_pose_follows_java_pose_metadata_and_maps_unknown_ids_to_standing() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::pose),
        Some(crate::ClientEntityPose::Standing)
    );

    let expected_poses = [
        crate::ClientEntityPose::Standing,
        crate::ClientEntityPose::FallFlying,
        crate::ClientEntityPose::Sleeping,
        crate::ClientEntityPose::Swimming,
        crate::ClientEntityPose::SpinAttack,
        crate::ClientEntityPose::Crouching,
        crate::ClientEntityPose::LongJumping,
        crate::ClientEntityPose::Dying,
        crate::ClientEntityPose::Croaking,
        crate::ClientEntityPose::UsingTongue,
        crate::ClientEntityPose::Sitting,
        crate::ClientEntityPose::Roaring,
        crate::ClientEntityPose::Sniffing,
        crate::ClientEntityPose::Emerging,
        crate::ClientEntityPose::Digging,
        crate::ClientEntityPose::Sliding,
        crate::ClientEntityPose::Shooting,
        crate::ClientEntityPose::Inhaling,
    ];
    for (id, expected) in expected_poses.into_iter().enumerate() {
        let mut metadata = vec![
            crate::ENTITY_POSE_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_POSE_ID as u8,
        ];
        voxel_core::wire::write_varint(id as i32, &mut metadata);
        metadata.push(255);
        assert!(world.apply_entity_metadata(37, &metadata));
        let entity = world.entity_at(37).expect("entity with pose");
        assert_eq!(entity.pose(), expected);
        assert_eq!(entity.pose().id(), id as u8);
        assert_eq!(
            entity.player_dimensions(),
            Some(expected.player_dimensions())
        );
    }

    let combined_metadata = [0, 0, 64, 5, 8, 1, 6, 20, 10, 255];
    assert!(world.apply_entity_metadata(37, &combined_metadata));
    let entity = world.entity_at(37).expect("combined pose entity");
    assert_eq!(entity.pose(), crate::ClientEntityPose::Sitting);
    assert!(entity.is_glowing());
    assert!(entity.is_no_gravity());

    assert!(world.apply_entity_metadata(37, &[6, 20, 99, 255]));
    let entity = world.entity_at(37).expect("fallback pose entity");
    assert_eq!(entity.pose(), crate::ClientEntityPose::Standing);
    assert_eq!(
        entity.player_dimensions(),
        Some(crate::ClientEntityPose::Standing.player_dimensions())
    );
    assert!(entity.is_glowing());
    assert!(entity.is_no_gravity());
}

#[test]
fn entity_ticks_frozen_follows_java_int_metadata_and_derived_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 1,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default freeze entity");
    assert_eq!(entity.ticks_frozen(), 0);
    assert_eq!(entity.percent_frozen(), 0.0);
    assert!(!entity.is_freezing());
    assert!(!entity.is_fully_frozen());

    let mut partial_metadata = vec![
        crate::ENTITY_TICKS_FROZEN_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(70, &mut partial_metadata);
    partial_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &partial_metadata));
    let entity = world.entity_at(37).expect("partially frozen entity");
    assert_eq!(entity.ticks_frozen(), 70);
    assert_eq!(entity.percent_frozen(), 0.5);
    assert!(entity.is_freezing());
    assert!(!entity.is_fully_frozen());

    let mut combined_metadata = vec![0, 0, 64, 6, 20, 10, 7, 1];
    voxel_core::wire::write_varint(140, &mut combined_metadata);
    combined_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &combined_metadata));
    let entity = world.entity_at(37).expect("fully frozen entity");
    assert_eq!(entity.ticks_frozen(), 140);
    assert_eq!(entity.percent_frozen(), 1.0);
    assert!(entity.is_freezing());
    assert!(entity.is_fully_frozen());
    assert_eq!(entity.pose(), crate::ClientEntityPose::Sitting);
    assert!(entity.is_glowing());

    let mut negative_metadata = vec![
        crate::ENTITY_TICKS_FROZEN_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(-1, &mut negative_metadata);
    negative_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &negative_metadata));
    let entity = world.entity_at(37).expect("negative freeze entity");
    assert_eq!(entity.ticks_frozen(), -1);
    assert_eq!(entity.percent_frozen(), -1.0 / 140.0);
    assert!(!entity.is_freezing());
    assert!(!entity.is_fully_frozen());
    assert_eq!(entity.pose(), crate::ClientEntityPose::Sitting);
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

#[test]
fn ticking_snowballs_applies_java_free_flight_gravity_drag_and_history() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_SNOWBALL_ENTITY_TYPE_ID,
        x: 2.0,
        y: 1.0,
        z: 2.0,
        movement: (0.25, 0.5, 0.25),
        rotation: (10.0, -20.0, 30.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(3, 4),
        type_id: VANILLA_SNOWBALL_ENTITY_TYPE_ID,
        x: 4.0,
        y: 1.0,
        z: 2.0,
        movement: (0.0, 0.0, 1.0),
        rotation: (45.0, 30.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_metadata(
        38,
        &[
            ENTITY_NO_GRAVITY_DATA_ID,
            ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            1,
            255,
        ],
    ));

    world.tick();

    let air_drag = f64::from(0.99_f32);
    let expected_motion = (
        0.25 * air_drag,
        (0.5 - SNOWBALL_GRAVITY) * air_drag,
        0.25 * air_drag,
    );
    let entity = world.entity_at(37).expect("gravity snowball");
    assert_eq!(entity.previous_position(), (2.0, 1.0, 2.0));
    assert_eq!(entity.motion(), expected_motion);
    assert_eq!(
        entity.position(),
        (
            2.0 + expected_motion.0,
            1.0 + expected_motion.1,
            2.0 + expected_motion.2
        )
    );
    assert_eq!(entity.tick_count(), 1);

    let entity = world.entity_at(38).expect("no-gravity snowball");
    assert_eq!(entity.motion(), (0.0, 0.0, air_drag));
    assert_eq!(entity.position(), (4.0, 1.0, 2.0 + air_drag));
    assert_eq!(entity.previous_position(), (4.0, 1.0, 2.0));
    assert_eq!(entity.rotation(), (36.0, 24.0, 0.0));
}

#[test]
fn ticking_arrows_applies_java_free_flight_inertia_gravity_and_history() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_ARROW_ENTITY_TYPE_ID,
        x: 2.0,
        y: 1.0,
        z: 2.0,
        movement: (0.25, 0.5, 0.25),
        rotation: (10.0, -20.0, 30.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(3, 4),
        type_id: VANILLA_ARROW_ENTITY_TYPE_ID,
        x: 4.0,
        y: 1.0,
        z: 2.0,
        movement: (0.0, 0.0, 1.0),
        rotation: (45.0, 30.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_metadata(
        38,
        &[
            ENTITY_NO_GRAVITY_DATA_ID,
            ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            1,
            255,
        ],
    ));

    world.tick();

    let air_drag = f64::from(0.99_f32);
    let expected_motion = (0.25 * air_drag, 0.5 * air_drag - 0.05, 0.25 * air_drag);
    let entity = world.entity_at(37).expect("gravity arrow");
    assert_eq!(entity.previous_position(), (2.0, 1.0, 2.0));
    assert_eq!(entity.motion(), expected_motion);
    assert_eq!(entity.position(), (2.0 + 0.25, 1.0 + 0.5, 2.0 + 0.25));
    assert_eq!(entity.tick_count(), 1);

    let entity = world.entity_at(38).expect("no-gravity arrow");
    assert_eq!(entity.motion(), (0.0, 0.0, air_drag));
    assert_eq!(entity.position(), (4.0, 1.0, 3.0));
    assert_eq!(entity.previous_position(), (4.0, 1.0, 2.0));
    assert_eq!(entity.rotation(), (36.0, 24.0, 0.0));
}

#[test]
fn experience_orb_first_client_tick_then_applies_gravity_drag_and_motion() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID,
        x: 2.0,
        y: 10.0,
        z: 2.0,
        movement: (0.125, 0.25, -0.125),
        rotation: (10.0, 20.0, 30.0),
        data: 1,
    }));

    world.tick();

    let entity = world
        .entity_at(37)
        .expect("experience orb after first tick");
    assert_eq!(entity.position(), (2.0, 10.0, 2.0));
    assert_eq!(entity.motion(), (0.125, 0.25, -0.125));
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(entity.tick_count(), 1);

    world.tick();

    let gravity_motion = (0.125, 0.25 - 0.03, -0.125);
    let air_drag = f64::from(0.98_f32);
    let entity = world
        .entity_at(37)
        .expect("experience orb after second tick");
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(
        entity.position(),
        (
            2.0 + gravity_motion.0,
            10.0 + gravity_motion.1,
            2.0 + gravity_motion.2,
        )
    );
    assert_eq!(
        entity.motion(),
        (
            gravity_motion.0 * air_drag,
            gravity_motion.1 * air_drag,
            gravity_motion.2 * air_drag,
        )
    );
    assert_eq!(entity.tick_count(), 2);
}

#[test]
fn ticking_primed_tnt_applies_gravity_move_and_air_drag_in_order() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PRIMED_TNT_ENTITY_TYPE_ID,
        x: 2.0,
        y: 10.0,
        z: 2.0,
        movement: (0.25, 0.5, -0.125),
        rotation: (10.0, 20.0, 30.0),
        data: 0,
    }));

    world.tick();

    let gravity_motion = (0.25, 0.5 - VANILLA_PRIMED_TNT_GRAVITY, -0.125);
    let air_drag = f64::from(VANILLA_PRIMED_TNT_AIR_FRICTION);
    let entity = world.entity_at(37).expect("primed TNT after tick");
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(
        entity.position(),
        (
            2.0 + gravity_motion.0,
            10.0 + gravity_motion.1,
            2.0 + gravity_motion.2,
        )
    );
    assert_eq!(
        entity.motion(),
        (
            gravity_motion.0 * air_drag,
            gravity_motion.1 * air_drag,
            gravity_motion.2 * air_drag,
        )
    );
    assert_eq!(entity.rotation(), (10.0, 20.0, 30.0));
    assert_eq!(entity.tick_count(), 1);
}

#[test]
fn ticking_falling_blocks_applies_gravity_and_air_drag_after_move() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID,
        x: 2.0,
        y: 10.0,
        z: 2.0,
        movement: (0.25, 0.5, -0.125),
        rotation: (10.0, 20.0, 30.0),
        data: 0,
    }));

    world.tick();

    let gravity_motion = (0.25, 0.5 - VANILLA_FALLING_BLOCK_GRAVITY, -0.125);
    let entity = world.entity_at(37).expect("falling block after tick");
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(
        entity.position(),
        (
            2.0 + gravity_motion.0,
            10.0 + gravity_motion.1,
            2.0 + gravity_motion.2,
        )
    );
    let air_drag = f64::from(VANILLA_FALLING_BLOCK_AIR_FRICTION);
    assert_eq!(
        entity.motion(),
        (
            gravity_motion.0 * air_drag,
            gravity_motion.1 * air_drag,
            gravity_motion.2 * air_drag,
        )
    );
    assert_eq!(entity.rotation(), (10.0, 20.0, 30.0));
    assert_eq!(entity.tick_count(), 1);
}

#[test]
fn ticking_firework_rockets_accelerates_and_rotates_from_free_flight_motion() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID,
        x: 2.0,
        y: 10.0,
        z: 2.0,
        movement: (0.25, 0.5, -0.125),
        rotation: (10.0, 20.0, 30.0),
        data: 0,
    }));

    world.tick();

    let movement = (
        0.25 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
        0.5 + VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION,
        -0.125 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
    );
    let horizontal_distance = (movement.0 * movement.0 + movement.2 * movement.2).sqrt();
    let target_x_rotation =
        (mth_atan2(movement.1, horizontal_distance) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
    let target_y_rotation =
        (mth_atan2(movement.0, movement.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
    let expected_rotation = (
        projectile_lerp_rotation_f32(10.0, target_y_rotation),
        projectile_lerp_rotation_f32(20.0, target_x_rotation),
        30.0,
    );
    let entity = world.entity_at(37).expect("firework rocket after tick");
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(
        entity.position(),
        (2.0 + movement.0, 10.0 + movement.1, 2.0 + movement.2,)
    );
    assert_eq!(entity.motion(), movement);
    assert_eq!(entity.rotation(), expected_rotation);
    assert_eq!(entity.tick_count(), 1);
}

#[test]
fn tick_counters_follow_local_player_and_entity_ticking_eligibility() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
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
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(0)
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::tick_count),
        Some(0)
    );

    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(1)
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::tick_count),
        Some(1)
    );

    assert!(world.unload_chunk(0, -3));
    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(2)
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::tick_count),
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
        world.entity_at(37).map(crate::ClientEntity::tick_count),
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
    world.set_local_player_id(42);
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
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 120,
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
            .entity_at(37)
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
        entity_id: 37,
        x: 3.0,
        y: 82.0,
        z: -34.5,
        relative_flags: 0,
    }));
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: 37,
        y_rot: Some(10.0),
        x_rot: Some(20.0),
        y_head_rot: Some(10.0),
        relative_y_rot: false,
        relative_x_rot: false,
    }));
    let entity = world.entity_at(37).expect("remote entity transform");
    assert_eq!(entity.previous_position(), (1.25, 80.0, -35.5));
    assert_eq!(entity.interpolated_position(0.5), (2.125, 81.0, -35.0));
    assert_eq!(entity.previous_rotation(), (350.0, -20.0, 350.0));
    assert_eq!(entity.interpolated_rotation(0.5), (360.0, 0.0, 360.0));

    assert!(world.unload_chunk(0, -3));
    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: 37,
        x: 4.0,
        y: 82.0,
        z: -34.5,
        relative_flags: 0,
    }));
    world.tick();
    assert_eq!(
        world
            .entity_at(37)
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
            .entity_at(37)
            .map(crate::ClientEntity::previous_position),
        Some((4.0, 82.0, -34.5))
    );
}

#[test]
fn remote_player_position_interpolation_follows_java_three_step_handler() {
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

    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: 37,
        x: 4.0,
        y: 82.0,
        z: -34.5,
        relative_flags: 0,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::position),
        Some((1.0, 80.0, -35.5))
    );
    assert_eq!(
        world
            .entity_at(37)
            .and_then(crate::ClientEntity::interpolation_target_position),
        Some((4.0, 82.0, -34.5))
    );
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: 37,
        y_rot: Some(10.0),
        x_rot: Some(20.0),
        y_head_rot: None,
        relative_y_rot: false,
        relative_x_rot: false,
    }));
    assert_eq!(
        world
            .entity_at(37)
            .and_then(crate::ClientEntity::interpolation_target_rotation),
        Some((10.0, 20.0))
    );

    world.tick();
    let first = world.entity_at(37).expect("first interpolation step");
    assert_eq!(first.interpolation_steps(), 2);
    assert_eq!(
        first.position(),
        (2.0, 80.66666666666667, -35.166666666666664)
    );
    assert_eq!(first.rotation(), (356.666_66_f32, -6.6666665_f32, 350.0));

    world.tick();
    let second = world.entity_at(37).expect("second interpolation step");
    assert_eq!(second.interpolation_steps(), 1);
    assert_eq!(
        second.position(),
        (3.0, 81.33333333333334, -34.83333333333333)
    );
    assert_eq!(second.rotation(), (3.333313_f32, 6.666667_f32, 350.0));

    world.tick();
    let third = world.entity_at(37).expect("final interpolation step");
    assert_eq!(third.interpolation_steps(), 0);
    assert_eq!(third.position(), (4.0, 82.0, -34.5));
    assert_eq!(third.rotation(), (10.0, 20.0, 350.0));
}

#[test]
fn remote_player_motion_packet_interpolates_for_three_ticks() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.0,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));

    let target = (0.3, -0.6, 0.9);
    assert!(world.apply_entity_motion(EntityMotionInput {
        entity_id: 37,
        x: target.0,
        y: target.1,
        z: target.2,
    }));
    let entity = world.entity_at(37).expect("remote player motion target");
    assert_eq!(entity.motion(), (0.0, 0.0, 0.0));
    assert_eq!(entity.motion_interpolation_target(), Some(target));
    assert_eq!(entity.motion_interpolation_steps(), 3);

    let first = (target.0 / 3.0, target.1 / 3.0, target.2 / 3.0);
    world.tick();
    let entity = world
        .entity_at(37)
        .expect("first remote player motion step");
    assert_eq!(entity.motion(), first);
    assert_eq!(entity.motion_interpolation_steps(), 2);

    let second = (
        first.0 + (target.0 - first.0) / 2.0,
        first.1 + (target.1 - first.1) / 2.0,
        first.2 + (target.2 - first.2) / 2.0,
    );
    world.tick();
    let entity = world
        .entity_at(37)
        .expect("second remote player motion step");
    assert_eq!(entity.motion(), second);
    assert_eq!(entity.motion_interpolation_steps(), 1);

    let third = (
        second.0 + (target.0 - second.0),
        second.1 + (target.1 - second.1),
        second.2 + (target.2 - second.2),
    );
    world.tick();
    let entity = world
        .entity_at(37)
        .expect("final remote player motion step");
    assert_eq!(entity.motion(), third);
    assert_eq!(entity.motion(), target);
    assert_eq!(entity.motion_interpolation_target(), None);
    assert_eq!(entity.motion_interpolation_steps(), 0);
}

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
