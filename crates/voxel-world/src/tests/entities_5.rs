use super::*;

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
