use super::*;

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
