use super::super::*;
use photon_core::ids::EntityId;

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
        entity_id: EntityId(37),
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
        entity_id: EntityId(37),
        x: 4.0,
        y: 82.0,
        z: -34.5,
        relative_flags: 0,
    }));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::position),
        Some((1.0, 80.0, -35.5))
    );
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .and_then(crate::ClientEntity::interpolation_target_position),
        Some((4.0, 82.0, -34.5))
    );
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: EntityId(37),
        y_rot: Some(10.0),
        x_rot: Some(20.0),
        y_head_rot: None,
        relative_y_rot: false,
        relative_x_rot: false,
    }));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .and_then(crate::ClientEntity::interpolation_target_rotation),
        Some((10.0, 20.0))
    );

    world.tick();
    let first = world
        .entities()
        .at(EntityId(37))
        .expect("first interpolation step");
    assert_eq!(first.interpolation_steps(), 2);
    assert_eq!(
        first.position(),
        (2.0, 80.66666666666667, -35.166666666666664)
    );
    assert_eq!(first.rotation(), (356.666_66_f32, -6.6666665_f32, 350.0));

    world.tick();
    let second = world
        .entities()
        .at(EntityId(37))
        .expect("second interpolation step");
    assert_eq!(second.interpolation_steps(), 1);
    assert_eq!(
        second.position(),
        (3.0, 81.33333333333334, -34.83333333333333)
    );
    assert_eq!(second.rotation(), (3.333313_f32, 6.666667_f32, 350.0));

    world.tick();
    let third = world
        .entities()
        .at(EntityId(37))
        .expect("final interpolation step");
    assert_eq!(third.interpolation_steps(), 0);
    assert_eq!(third.position(), (4.0, 82.0, -34.5));
    assert_eq!(third.rotation(), (10.0, 20.0, 350.0));
}

#[test]
fn remote_player_motion_packet_interpolates_for_three_ticks() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
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
    assert!(world.entities_mut().apply_motion(EntityMotionInput {
        entity_id: EntityId(37),
        x: target.0,
        y: target.1,
        z: target.2,
    }));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("remote player motion target");
    assert_eq!(entity.motion(), (0.0, 0.0, 0.0));
    assert_eq!(entity.motion_interpolation_target(), Some(target));
    assert_eq!(entity.motion_interpolation_steps(), 3);

    let first = (target.0 / 3.0, target.1 / 3.0, target.2 / 3.0);
    world.tick();
    let entity = world
        .entities()
        .at(EntityId(37))
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
        .entities()
        .at(EntityId(37))
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
        .entities()
        .at(EntityId(37))
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
        entity_id: EntityId(37),
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
        entity_id: EntityId(37),
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
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("relative teleport entity");
    assert_eq!(entity.position(), (12.0, 67.0, 24.0));
    assert_eq!(entity.rotation(), (15.0, 10.0, 0.0));
    assert_eq!(entity.motion(), (1.25, 1.5, 3.75));

    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(38),
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
        entity_id: EntityId(38),
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
    let entity = world
        .entities()
        .at(EntityId(38))
        .expect("rotated teleport entity");
    let radians = (f64::from(-90.0_f32) * (std::f64::consts::PI / 180.0)) as f32;
    let expected_x = f64::from(mth_cos_f64(f64::from(radians)));
    let expected_z = -f64::from(mth_sin_f64(f64::from(radians)));
    assert_eq!(entity.rotation(), (90.0, 0.0, 0.0));
    assert_eq!(entity.motion(), (expected_x, 0.0, expected_z));

    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(39),
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
        entity_id: EntityId(39),
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
            .entities()
            .at(EntityId(39))
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
        entity_id: EntityId(37),
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
            entity_id: EntityId(37),
            x: 66.0,
            y: 80.0,
            z: -35.5,
            relative_flags: 0,
        },
        EntityRotationInput {
            entity_id: EntityId(37),
            y_rot: Some(10.0),
            x_rot: Some(20.0),
            y_head_rot: None,
            relative_y_rot: false,
            relative_x_rot: false,
        },
        EntityPositionUpdateMode::PositionSync,
    ));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("snapped remote player");
    assert_eq!(entity.position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.previous_position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.rotation(), (10.0, 20.0, 350.0));
    assert_eq!(entity.previous_rotation(), (10.0, 20.0, 350.0));
    assert_eq!(entity.interpolation_steps(), 0);
    assert_eq!(entity.interpolation_target_position(), None);
    assert_eq!(entity.interpolation_target_rotation(), None);

    world.tick();
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("snapped remote player tick");
    assert_eq!(entity.position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.previous_position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.rotation(), (10.0, 20.0, 350.0));

    assert!(world.apply_entity_position_and_rotation(
        EntityPositionInput {
            entity_id: EntityId(37),
            x: 130.0,
            y: 80.0,
            z: -35.5,
            relative_flags: 0,
        },
        EntityRotationInput {
            entity_id: EntityId(37),
            y_rot: Some(30.0),
            x_rot: Some(10.0),
            y_head_rot: None,
            relative_y_rot: false,
            relative_x_rot: false,
        },
        EntityPositionUpdateMode::PositionSync,
    ));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("threshold remote player");
    assert_eq!(entity.position(), (66.0, 80.0, -35.5));
    assert_eq!(
        entity.interpolation_target_position(),
        Some((130.0, 80.0, -35.5))
    );
    assert_eq!(entity.interpolation_target_rotation(), Some((30.0, 10.0)));
    assert_eq!(entity.interpolation_steps(), 3);

    assert!(world.apply_entity_position_and_rotation(
        EntityPositionInput {
            entity_id: EntityId(37),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            relative_flags: 0b111,
        },
        EntityRotationInput {
            entity_id: EntityId(37),
            y_rot: Some(20.0),
            x_rot: Some(100.0),
            y_head_rot: None,
            relative_y_rot: true,
            relative_x_rot: true,
        },
        EntityPositionUpdateMode::Teleport,
    ));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("relative teleport target");
    assert_eq!(
        entity.interpolation_target_position(),
        Some((130.0, 80.0, -35.5))
    );
    assert_eq!(entity.interpolation_target_rotation(), Some((50.0, 90.0)));
}
