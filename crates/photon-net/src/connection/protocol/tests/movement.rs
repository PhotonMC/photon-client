use super::*;

#[test]
fn entity_motion_parser_matches_java_id_and_packed_vector_order() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(37, &mut payload);
    payload.push(0);

    assert_eq!(
        parse_set_entity_motion_packet(&payload).expect("entity motion"),
        super::super::EntityMotionObservation {
            entity_id: 37,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            applied_to_world: false,
            applied_motion: None,
        }
    );

    payload.push(0);
    assert!(parse_set_entity_motion_packet(&payload).is_err());
}

#[test]
fn entity_head_rotation_parser_matches_java_packed_degrees() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(37, &mut payload);
    payload.push(64);

    assert_eq!(
        parse_rotate_head_packet(&payload).expect("rotate head"),
        super::super::EntityHeadRotationObservation {
            entity_id: 37,
            y_head_rot: 90.0,
            applied_to_world: false,
            applied_head_rotation: None,
        }
    );

    payload.push(0);
    assert!(parse_rotate_head_packet(&payload).is_err());
}

#[test]
fn entity_teleport_parser_matches_java_position_move_rotation_order() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(42, &mut payload);
    for value in [10.5_f64, 20.25, -30.75, 0.125, -0.25, 0.5] {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    payload.extend_from_slice(&45.0_f32.to_be_bytes());
    payload.extend_from_slice(&(-15.0_f32).to_be_bytes());
    payload.extend_from_slice(&0b101_i32.to_be_bytes());
    payload.push(1);

    assert_eq!(
        parse_entity_teleport_packet(&payload).expect("entity teleport"),
        super::super::EntityTeleportObservation {
            entity_id: 42,
            x: 10.5,
            y: 20.25,
            z: -30.75,
            delta_x: 0.125,
            delta_y: -0.25,
            delta_z: 0.5,
            y_rot: 45.0,
            x_rot: -15.0,
            relative_flags: 0b101,
            on_ground: true,
            motion_before: None,
            applied_to_world: false,
            applied_on_ground: None,
            applied_position: None,
            applied_rotation: None,
            applied_motion: None,
        }
    );

    payload.push(0);
    assert!(parse_entity_teleport_packet(&payload).is_err());
}

#[test]
fn entity_position_sync_parser_matches_java_position_move_rotation_without_mask() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(-7, &mut payload);
    for value in [1.5_f64, 2.25, -3.75, 0.0, 0.125, -0.5] {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    payload.extend_from_slice(&180.0_f32.to_be_bytes());
    payload.extend_from_slice(&(-30.0_f32).to_be_bytes());
    payload.push(0);

    assert_eq!(
        parse_entity_position_sync_packet(&payload).expect("entity position sync"),
        super::super::EntityPositionSyncObservation {
            entity_id: -7,
            x: 1.5,
            y: 2.25,
            z: -3.75,
            delta_x: 0.0,
            delta_y: 0.125,
            delta_z: -0.5,
            y_rot: 180.0,
            x_rot: -30.0,
            on_ground: false,
            applied_to_world: false,
            applied_on_ground: None,
            applied_position: None,
            applied_rotation: None,
        }
    );

    payload.push(0);
    assert!(parse_entity_position_sync_packet(&payload).is_err());
}

#[test]
fn move_entity_parsers_match_java_position_and_rotation_variants() {
    let mut pos_payload = Vec::new();
    photon_core::wire::write_varint(42, &mut pos_payload);
    for value in [256_i16, -512, 1024] {
        pos_payload.extend_from_slice(&value.to_be_bytes());
    }
    pos_payload.push(1);
    assert_eq!(
        parse_move_entity_pos_packet(&pos_payload).expect("move entity position"),
        super::super::EntityMoveObservation {
            entity_id: 42,
            delta_x: 256,
            delta_y: -512,
            delta_z: 1024,
            y_rot: None,
            x_rot: None,
            has_position: true,
            has_rotation: false,
            on_ground: true,
            applied_to_world: false,
            applied_on_ground: None,
            applied_position: None,
            applied_rotation: None,
        }
    );

    let mut pos_rot_payload = pos_payload.clone();
    pos_rot_payload.insert(pos_rot_payload.len() - 1, 64);
    pos_rot_payload.insert(pos_rot_payload.len() - 1, 192);
    assert_eq!(
        parse_move_entity_pos_rot_packet(&pos_rot_payload).expect("move entity position rotation"),
        super::super::EntityMoveObservation {
            entity_id: 42,
            delta_x: 256,
            delta_y: -512,
            delta_z: 1024,
            y_rot: Some(90.0),
            x_rot: Some(-90.0),
            has_position: true,
            has_rotation: true,
            on_ground: true,
            applied_to_world: false,
            applied_on_ground: None,
            applied_position: None,
            applied_rotation: None,
        }
    );

    let mut rot_payload = Vec::new();
    photon_core::wire::write_varint(42, &mut rot_payload);
    rot_payload.extend_from_slice(&[64, 192, 0]);
    assert_eq!(
        parse_move_entity_rot_packet(&rot_payload).expect("move entity rotation"),
        super::super::EntityMoveObservation {
            entity_id: 42,
            delta_x: 0,
            delta_y: 0,
            delta_z: 0,
            y_rot: Some(90.0),
            x_rot: Some(-90.0),
            has_position: false,
            has_rotation: true,
            on_ground: false,
            applied_to_world: false,
            applied_on_ground: None,
            applied_position: None,
            applied_rotation: None,
        }
    );

    pos_payload.push(0);
    assert!(parse_move_entity_pos_packet(&pos_payload).is_err());
}

#[test]
fn minecart_steps_parser_reads_doubles_packed_turns_and_a_float_weight() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(9, &mut payload);
    photon_core::wire::write_varint(1, &mut payload);
    for value in [1.5_f64, 64.1, -2.0, 0.25, 0.0, -0.5] {
        payload.extend(value.to_be_bytes());
    }
    payload.extend([64, 0xe0]);
    payload.extend(0.75_f32.to_be_bytes());

    let (entity_id, steps) = parse_move_minecart_packet(&payload).expect("minecart steps");
    assert_eq!(entity_id, 9);
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].position, (1.5, 64.1, -2.0));
    assert_eq!(steps[0].movement, (0.25, 0.0, -0.5));
    assert_eq!((steps[0].y_rot, steps[0].x_rot), (90.0, -45.0));
    assert_eq!(steps[0].weight, 0.75);

    payload.push(0);
    assert!(parse_move_minecart_packet(&payload).is_err());
}
