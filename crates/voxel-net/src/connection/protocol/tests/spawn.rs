use super::*;

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
fn an_entity_link_names_the_leashed_entity_and_its_holder() {
    let mut packet = Vec::new();
    packet.extend_from_slice(&7_i32.to_be_bytes());
    packet.extend_from_slice(&12_i32.to_be_bytes());
    assert_eq!(
        super::super::parse_set_entity_link_packet(&packet).unwrap(),
        (7, 12)
    );
    // Nobody holds it any more.
    packet[4..].copy_from_slice(&0_i32.to_be_bytes());
    assert_eq!(
        super::super::parse_set_entity_link_packet(&packet).unwrap(),
        (7, 0)
    );
    assert!(super::super::parse_set_entity_link_packet(&packet[..6]).is_err());
}
