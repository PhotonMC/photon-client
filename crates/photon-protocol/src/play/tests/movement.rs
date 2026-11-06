use super::super::movement_steps::*;
use super::*;

#[test]
fn entity_motion_parser_matches_java_id_and_packed_vector_order() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(37, &mut payload);
    payload.push(0);

    assert_eq!(
        parse_set_entity_motion_packet(&payload).expect("entity motion"),
        super::super::EntityMotionObservation {
            entity_id: photon_core::ids::EntityId(37),
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
            entity_id: photon_core::ids::EntityId(37),
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
            entity_id: photon_core::ids::EntityId(42),
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
            entity_id: photon_core::ids::EntityId(-7),
            x: 1.5,
            y: 2.25,
            z: -3.75,
            delta_x: 0.0,
            delta_y: 0.125,
            delta_z: -0.5,
            y_rot: 180.0,
            x_rot: -30.0,
            on_ground: false,
            steps: Vec::new(),
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
            entity_id: photon_core::ids::EntityId(42),
            delta_x: 256,
            delta_y: -512,
            delta_z: 1024,
            y_rot: None,
            x_rot: None,
            has_position: true,
            has_rotation: false,
            on_ground: true,
            steps: Vec::new(),
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
            entity_id: photon_core::ids::EntityId(42),
            delta_x: 256,
            delta_y: -512,
            delta_z: 1024,
            y_rot: Some(90.0),
            x_rot: Some(-90.0),
            has_position: true,
            has_rotation: true,
            on_ground: true,
            steps: Vec::new(),
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
            entity_id: photon_core::ids::EntityId(42),
            delta_x: 0,
            delta_y: 0,
            delta_z: 0,
            y_rot: Some(90.0),
            x_rot: Some(-90.0),
            has_position: false,
            has_rotation: true,
            on_ground: false,
            steps: Vec::new(),
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
    assert_eq!(entity_id, photon_core::ids::EntityId(9));
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].position, (1.5, 64.1, -2.0));
    assert_eq!(steps[0].movement, (0.25, 0.0, -0.5));
    assert_eq!((steps[0].y_rot, steps[0].x_rot), (90.0, -45.0));
    assert_eq!(steps[0].weight, 0.75);

    payload.push(0);
    assert!(parse_move_minecart_packet(&payload).is_err());
}

fn registries_263() -> std::sync::Arc<photon_data::version_registries::VersionRegistries> {
    photon_data::version_registries::VersionRegistries::of(photon_core::version::Version::V26_3)
}

#[test]
fn move_entity_26_3_reads_properties_steps_and_rotation_only_ground_byte() {
    let registries = registries_263();
    // Pos: id 5, properties = 2 steps << 1 | on ground, then two steps of (ticks, x, y, z).
    let mut payload = Vec::new();
    photon_core::wire::write_varint(5, &mut payload);
    photon_core::wire::write_varint(2 << 1 | 1, &mut payload);
    for (ticks, x, y, z) in [(2_i32, 100_i16, -50_i16, 7_i16), (3, 1, 2, 3)] {
        photon_core::wire::write_varint(ticks, &mut payload);
        for value in [x, y, z] {
            payload.extend_from_slice(&value.to_be_bytes());
        }
    }
    let packet = parse_move_entity_pos_packet_for(&payload, &registries).expect("stepped pos");
    assert!(packet.on_ground && packet.has_position && !packet.has_rotation);
    assert_eq!(packet.steps.len(), 2);
    assert_eq!(packet.steps[0].ticks, 2);
    assert_eq!((packet.delta_x, packet.delta_y, packet.delta_z), (1, 2, 3));

    // A plain offset has no step count.
    let mut plain = Vec::new();
    photon_core::wire::write_varint(5, &mut plain);
    photon_core::wire::write_varint(0, &mut plain);
    for value in [256_i16, -512, 1024] {
        plain.extend_from_slice(&value.to_be_bytes());
    }
    plain.extend_from_slice(&[64, 192]);
    let packet = parse_move_entity_pos_rot_packet_for(&plain, &registries).expect("plain pos rot");
    assert!(!packet.on_ground && packet.steps.is_empty());
    assert_eq!(packet.delta_y, -512);
    assert_eq!(packet.y_rot, Some(90.0));

    // A count bigger than the bytes left is refused.
    let mut bad = Vec::new();
    photon_core::wire::write_varint(5, &mut bad);
    photon_core::wire::write_varint(9 << 1, &mut bad);
    bad.extend_from_slice(&[0; 7]);
    assert!(parse_move_entity_pos_packet_for(&bad, &registries).is_err());

    // Rot: the on-ground byte comes before the turn.
    let mut rot = Vec::new();
    photon_core::wire::write_varint(5, &mut rot);
    rot.extend_from_slice(&[1, 64, 192]);
    let packet = parse_move_entity_rot_packet_for(&rot, &registries).expect("rot");
    assert!(packet.on_ground && packet.has_rotation && !packet.has_position);
    assert_eq!(packet.x_rot, Some(-90.0));
}

#[test]
fn position_sync_26_3_reads_linear_and_stepped_paths() {
    let registries = registries_263();
    let mut linear = Vec::new();
    photon_core::wire::write_varint(3, &mut linear);
    photon_core::wire::write_varint(0, &mut linear);
    for value in [1.0_f64, 2.0, 3.0] {
        linear.extend_from_slice(&value.to_be_bytes());
    }
    linear.extend_from_slice(&10.0_f32.to_be_bytes());
    linear.extend_from_slice(&20.0_f32.to_be_bytes());
    linear.push(1);
    let packet = parse_entity_position_sync_packet_for(&linear, &registries).expect("linear");
    assert_eq!((packet.x, packet.y, packet.z), (1.0, 2.0, 3.0));
    assert!(packet.steps.is_empty() && packet.on_ground);
    assert_eq!((packet.y_rot, packet.x_rot), (10.0, 20.0));

    let mut stepped = Vec::new();
    photon_core::wire::write_varint(3, &mut stepped);
    photon_core::wire::write_varint(1, &mut stepped);
    photon_core::wire::write_varint(2, &mut stepped);
    for (place, ticks) in [([1.0_f64, 2.0, 3.0], 4), ([5.0, 6.0, 7.0], 1)] {
        for value in place {
            stepped.extend_from_slice(&value.to_be_bytes());
        }
        photon_core::wire::write_varint(ticks, &mut stepped);
    }
    stepped.extend_from_slice(&0.0_f32.to_be_bytes());
    stepped.extend_from_slice(&0.0_f32.to_be_bytes());
    stepped.push(0);
    let packet = parse_entity_position_sync_packet_for(&stepped, &registries).expect("stepped");
    assert_eq!((packet.x, packet.y, packet.z), (5.0, 6.0, 7.0));
    assert_eq!(
        packet.steps,
        vec![((1.0, 2.0, 3.0), 4), ((5.0, 6.0, 7.0), 1)]
    );
}
