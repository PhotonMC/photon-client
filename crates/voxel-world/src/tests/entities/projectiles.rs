use super::super::*;

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
