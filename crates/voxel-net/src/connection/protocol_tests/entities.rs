use super::*;

#[test]
fn set_health_parser_and_perform_respawn_encoder_match_java_order() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&0.0_f32.to_be_bytes());
    voxel_core::wire::write_varint(20, &mut payload);
    payload.extend_from_slice(&5.5_f32.to_be_bytes());

    assert_eq!(
        parse_set_health_packet(&payload).expect("set health packet"),
        super::super::HealthObservation {
            health: 0.0,
            food: 20,
            saturation: 5.5,
            applied_to_world: false,
        }
    );

    payload.push(0);
    assert!(parse_set_health_packet(&payload).is_err());

    let mut command_payload = Vec::new();
    encode_perform_respawn(&mut command_payload).expect("perform respawn command");
    assert_eq!(command_payload, [0]);
}

#[test]
fn add_entity_parser_matches_java_spawn_field_order() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(37, &mut payload);
    payload.extend_from_slice(&[0; 16]);
    voxel_core::wire::write_varint(120, &mut payload);
    for value in [1.25_f64, 80.0, -35.5] {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    payload.push(0);
    payload.extend_from_slice(&[64, 192, 32]);
    voxel_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_add_entity_packet(&payload).expect("add entity"),
        super::super::AddEntityObservation {
            entity_id: 37,
            uuid: voxel_core::binary::Uuid::new(0, 0),
            entity_type_id: 120,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            x_rot: 90.0,
            y_rot: -90.0,
            y_head_rot: 45.0,
            data: 0,
            applied_to_world: false,
            applied_rotation: None,
        }
    );
}

#[test]
fn entity_motion_parser_matches_java_id_and_packed_vector_order() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(37, &mut payload);
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
    voxel_core::wire::write_varint(37, &mut payload);
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
fn remove_entities_parser_matches_java_int_id_list() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(3, &mut payload);
    voxel_core::wire::write_varint(37, &mut payload);
    voxel_core::wire::write_varint(-12, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_remove_entities_packet(&payload).expect("remove entities"),
        super::super::RemoveEntitiesObservation {
            entity_ids: vec![37, -12, 0],
            applied_entity_count: 0,
        }
    );

    let mut negative_count = Vec::new();
    voxel_core::wire::write_varint(-1, &mut negative_count);
    assert_eq!(
        parse_remove_entities_packet(&negative_count).expect("negative count"),
        super::super::RemoveEntitiesObservation {
            entity_ids: Vec::new(),
            applied_entity_count: 0,
        }
    );

    payload.push(0);
    assert!(parse_remove_entities_packet(&payload).is_err());
}

#[test]
fn set_passengers_parser_matches_java_vehicle_and_varint_array_order() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(37, &mut payload);
    voxel_core::wire::write_varint(3, &mut payload);
    voxel_core::wire::write_varint(4, &mut payload);
    voxel_core::wire::write_varint(128, &mut payload);
    voxel_core::wire::write_varint(-1, &mut payload);

    assert_eq!(
        parse_set_passengers_packet(&payload).expect("set passengers"),
        super::super::EntityPassengersObservation {
            vehicle_id: 37,
            passenger_ids: vec![4, 128, -1],
            applied_to_world: false,
            applied_vehicle_passengers: None,
            applied_passenger_vehicles: Vec::new(),
        }
    );

    let mut empty = Vec::new();
    voxel_core::wire::write_varint(37, &mut empty);
    voxel_core::wire::write_varint(0, &mut empty);
    assert_eq!(
        parse_set_passengers_packet(&empty)
            .expect("empty set passengers")
            .passenger_ids,
        Vec::<i32>::new()
    );

    let mut negative_count = Vec::new();
    voxel_core::wire::write_varint(37, &mut negative_count);
    voxel_core::wire::write_varint(-1, &mut negative_count);
    assert!(parse_set_passengers_packet(&negative_count).is_err());

    payload.push(0);
    assert!(parse_set_passengers_packet(&payload).is_err());
}

#[test]
fn entity_metadata_parser_keeps_java_end_marker_and_raw_payload() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(42, &mut payload);
    payload.extend_from_slice(&[0, 0, 5, 255]);

    assert_eq!(
        parse_entity_metadata_packet(&payload).expect("entity metadata"),
        super::super::EntityMetadataObservation {
            entity_id: 42,
            metadata_length: 4,
            applied_to_world: false,
            metadata: vec![0, 0, 5, 255],
        }
    );

    let mut missing_marker = Vec::new();
    voxel_core::wire::write_varint(42, &mut missing_marker);
    missing_marker.push(0);
    assert!(parse_entity_metadata_packet(&missing_marker).is_err());
}

#[test]
fn entity_teleport_parser_matches_java_position_move_rotation_order() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(42, &mut payload);
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
    voxel_core::wire::write_varint(-7, &mut payload);
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
    voxel_core::wire::write_varint(42, &mut pos_payload);
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
    voxel_core::wire::write_varint(42, &mut rot_payload);
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
fn mob_effect_parsers_match_java_registry_and_effect_fields() {
    let mut update_payload = Vec::new();
    voxel_core::wire::write_varint(42, &mut update_payload);
    voxel_core::wire::write_varint(19, &mut update_payload);
    voxel_core::wire::write_varint(1, &mut update_payload);
    voxel_core::wire::write_varint(1_200, &mut update_payload);
    update_payload.push(0b1111);

    assert_eq!(
        parse_update_mob_effect_packet(&update_payload).expect("mob effect update"),
        super::super::MobEffectUpdateObservation {
            entity_id: 42,
            effect_registry_id: 19,
            amplifier: 1,
            duration_ticks: 1_200,
            flags: 0b1111,
            applied_to_world: false,
        }
    );

    let mut remove_payload = Vec::new();
    voxel_core::wire::write_varint(42, &mut remove_payload);
    voxel_core::wire::write_varint(19, &mut remove_payload);

    assert_eq!(
        parse_remove_mob_effect_packet(&remove_payload).expect("mob effect removal"),
        super::super::MobEffectRemoveObservation {
            entity_id: 42,
            effect_registry_id: 19,
            applied_to_world: false,
        }
    );
}

#[test]
fn set_equipment_reads_each_slot_until_the_one_without_the_continue_bit() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(5, &mut payload);
    // The main hand holds two of item 10 (no component changes), the head
    // is emptied.
    payload.extend_from_slice(&[128, 2, 10, 0, 0, 5, 0]);
    let packet = parse_entity_equipment_packet(&payload).expect("set equipment");
    assert_eq!(packet.entity_id, 5);
    assert_eq!(packet.slots.len(), 2);
    assert_eq!(packet.slots[0].0, 0);
    let stack = packet.slots[0].1.as_ref().expect("a stack");
    assert_eq!((stack.item_count(), stack.item_registry_id()), (2, 10));
    assert_eq!(packet.slots[1], (5, None));
    assert!(parse_entity_equipment_packet(&payload[..4]).is_err());
}
