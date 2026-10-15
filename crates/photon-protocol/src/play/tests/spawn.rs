use super::*;

#[test]
fn add_entity_parser_matches_java_spawn_field_order() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(37, &mut payload);
    payload.extend_from_slice(&[0; 16]);
    photon_core::wire::write_varint(120, &mut payload);
    for value in [1.25_f64, 80.0, -35.5] {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    payload.push(0);
    payload.extend_from_slice(&[64, 192, 32]);
    photon_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_add_entity_packet(&payload).expect("add entity"),
        super::super::AddEntityObservation {
            entity_id: 37,
            uuid: photon_core::binary::Uuid::new(0, 0),
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
    photon_core::wire::write_varint(3, &mut payload);
    photon_core::wire::write_varint(37, &mut payload);
    photon_core::wire::write_varint(-12, &mut payload);
    photon_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_remove_entities_packet(&payload).expect("remove entities"),
        super::super::RemoveEntitiesObservation {
            entity_ids: vec![37, -12, 0],
            applied_entity_count: 0,
        }
    );

    let mut negative_count = Vec::new();
    photon_core::wire::write_varint(-1, &mut negative_count);
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
    photon_core::wire::write_varint(37, &mut payload);
    photon_core::wire::write_varint(3, &mut payload);
    photon_core::wire::write_varint(4, &mut payload);
    photon_core::wire::write_varint(128, &mut payload);
    photon_core::wire::write_varint(-1, &mut payload);

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
    photon_core::wire::write_varint(37, &mut empty);
    photon_core::wire::write_varint(0, &mut empty);
    assert_eq!(
        parse_set_passengers_packet(&empty)
            .expect("empty set passengers")
            .passenger_ids,
        Vec::<i32>::new()
    );

    let mut negative_count = Vec::new();
    photon_core::wire::write_varint(37, &mut negative_count);
    photon_core::wire::write_varint(-1, &mut negative_count);
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

#[test]
fn take_item_entity_parser_reads_three_varints() {
    let mut payload = Vec::new();
    for value in [41, 7, 3] {
        photon_core::wire::write_varint(value, &mut payload);
    }
    assert_eq!(
        parse_take_item_entity_packet(&payload).expect("take item"),
        (41, 7, 3)
    );
    payload.push(0);
    assert!(parse_take_item_entity_packet(&payload).is_err());
}

#[test]
fn explode_parser_reads_the_push_the_particles_and_the_sound() {
    let mut payload = Vec::new();
    for value in [1.5_f64, 64.0, -2.0] {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    payload.extend_from_slice(&4.0_f32.to_be_bytes());
    payload.extend_from_slice(&7_i32.to_be_bytes());
    payload.push(1);
    for value in [0.25_f64, 0.5, 0.0] {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    let explosion = photon_data::particle_type::id("explosion").unwrap();
    photon_core::wire::write_varint(explosion, &mut payload);
    let sound = photon_data::sound_event::id("minecraft:entity.generic.explode").unwrap();
    photon_core::wire::write_varint(sound + 1, &mut payload);
    photon_core::wire::write_varint(1, &mut payload);
    let poof = photon_data::particle_type::id("poof").unwrap();
    photon_core::wire::write_varint(poof, &mut payload);
    payload.extend_from_slice(&0.5_f32.to_be_bytes());
    payload.extend_from_slice(&1.0_f32.to_be_bytes());
    photon_core::wire::write_varint(3, &mut payload);
    let (read, knockback) = super::super::parse_explode_packet(&payload)
        .expect("explode")
        .expect("known options");
    assert_eq!(knockback, Some((0.25, 0.5, 0.0)));
    assert_eq!(
        (read.center, read.radius, read.block_count),
        ((1.5, 64.0, -2.0), 4.0, 7)
    );
    assert_eq!(read.sound, Some("minecraft:entity.generic.explode"));
    assert_eq!(read.block_particles.len(), 1);
    assert_eq!(read.block_particles[0].weight, 3);
}
