use super::*;

#[test]
fn entity_metadata_parser_keeps_java_end_marker_and_raw_payload() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(42, &mut payload);
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
    photon_core::wire::write_varint(42, &mut missing_marker);
    missing_marker.push(0);
    assert!(parse_entity_metadata_packet(&missing_marker).is_err());
}

#[test]
fn mob_effect_parsers_match_java_registry_and_effect_fields() {
    let mut update_payload = Vec::new();
    photon_core::wire::write_varint(42, &mut update_payload);
    photon_core::wire::write_varint(19, &mut update_payload);
    photon_core::wire::write_varint(1, &mut update_payload);
    photon_core::wire::write_varint(1_200, &mut update_payload);
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
    photon_core::wire::write_varint(42, &mut remove_payload);
    photon_core::wire::write_varint(19, &mut remove_payload);

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
    photon_core::wire::write_varint(5, &mut payload);
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

#[test]
fn update_attributes_parser_matches_java_snapshot_and_modifier_order() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(42, &mut payload);
    photon_core::wire::write_varint(1, &mut payload);
    photon_core::wire::write_varint(17, &mut payload);
    payload.extend_from_slice(&2.5_f64.to_be_bytes());
    photon_core::wire::write_varint(1, &mut payload);
    photon_core::identifier::write_identifier(
        &photon_core::identifier::Identifier::parse("minecraft:test_modifier")
            .expect("modifier identifier"),
        &mut payload,
    )
    .expect("modifier identifier bytes");
    payload.extend_from_slice(&0.25_f64.to_be_bytes());
    payload.push(2);

    let observation = parse_update_attributes_packet(&payload).expect("update attributes packet");
    assert_eq!(observation.entity_id, 42);
    assert!(!observation.applied_to_world);
    assert_eq!(observation.attributes.len(), 1);
    let attribute = &observation.attributes[0];
    assert_eq!(attribute.attribute_id(), 17);
    assert_eq!(attribute.base_value(), 2.5);
    assert_eq!(attribute.modifiers().len(), 1);
    assert_eq!(attribute.modifiers()[0].id(), "minecraft:test_modifier");
    assert_eq!(attribute.modifiers()[0].amount(), 0.25);
    assert_eq!(attribute.modifiers()[0].operation(), 2);

    payload.push(0);
    assert!(parse_update_attributes_packet(&payload).is_err());
}

#[test]
fn an_entity_event_packet_names_the_entity_and_the_event() {
    let mut payload = 7_i32.to_be_bytes().to_vec();
    payload.push(26);
    assert_eq!(parse_entity_event_packet(&payload).unwrap(), (7, 26));
    assert!(parse_entity_event_packet(&payload[..4]).is_err());
}

#[test]
fn a_hurt_animation_names_the_entity_and_the_yaw_of_the_blow() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(7, &mut payload);
    payload.extend_from_slice(&(-135.5_f32).to_be_bytes());
    assert_eq!(parse_hurt_animation_packet(&payload).unwrap(), (7, -135.5));
    assert!(parse_hurt_animation_packet(&payload[..3]).is_err());
}

#[test]
fn a_damage_event_names_the_entity_and_the_type_by_registry_id_plus_one() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(300, &mut payload);
    photon_core::wire::write_varint(5, &mut payload);
    assert_eq!(parse_damage_event_packet(&payload).unwrap(), (300, Some(4)));
    // A type written out in the packet is not one the registry names.
    let mut direct = Vec::new();
    photon_core::wire::write_varint(7, &mut direct);
    photon_core::wire::write_varint(0, &mut direct);
    assert_eq!(parse_damage_event_packet(&direct).unwrap(), (7, None));
    assert!(parse_damage_event_packet(&payload[..2]).is_err());
}

#[test]
fn animate_reads_the_entity_and_the_action() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(300, &mut payload);
    payload.push(3);
    assert_eq!(
        super::super::parse_animate_packet(&payload).unwrap(),
        (300, 3)
    );
    assert!(super::super::parse_animate_packet(&payload[..2]).is_err());
}
