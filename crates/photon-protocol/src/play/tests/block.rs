use super::*;

#[test]
fn a_block_event_names_the_block_the_event_and_its_data() {
    let mut payload = photon_core::binary::BlockPos::new(-3, 64, 12)
        .pack()
        .to_be_bytes()
        .to_vec();
    payload.extend_from_slice(&[1, 2, 0x05]);
    assert_eq!(
        parse_block_event_packet(&payload).unwrap(),
        ((-3, 64, 12), 1, 2)
    );
    assert!(parse_block_event_packet(&payload[..9]).is_err());
}

#[test]
fn block_entity_data_packet_parser_decodes_packed_position_and_nbt() {
    let mut payload = pack_block_position((-12, -63, 9)).to_be_bytes().to_vec();
    photon_core::wire::write_varint(42, &mut payload);
    payload.extend_from_slice(&[10, 0]);

    let observation = parse_block_entity_data_packet(&payload).expect("block entity data");
    assert_eq!(observation.x, -12);
    assert_eq!(observation.y, -63);
    assert_eq!(observation.z, 9);
    assert_eq!(observation.type_id, 42);
    assert_eq!(observation.tag_length, 2);
    assert!(!observation.applied_to_world);

    payload.push(0);
    assert!(parse_block_entity_data_packet(&payload).is_err());
}

#[test]
fn block_update_parser_matches_java_packed_position_and_state() {
    let mut payload = pack_block_position((-12, -63, 9)).to_be_bytes().to_vec();
    photon_core::wire::write_varint(123, &mut payload);

    assert_eq!(
        parse_block_update_packet(&payload).expect("block update"),
        super::super::BlockUpdateObservation {
            x: -12,
            y: -63,
            z: 9,
            state_id: photon_core::ids::BlockStateId(123),
        }
    );
}

#[test]
fn block_destruction_parser_matches_java_progress_fields() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(37, &mut payload);
    payload.extend_from_slice(&pack_block_position((-12, 67, 9)).to_be_bytes());
    payload.push(255);

    assert_eq!(
        parse_block_destruction_packet(&payload).expect("block destruction"),
        super::super::BlockDestructionObservation {
            entity_id: photon_core::ids::EntityId(37),
            x: -12,
            y: 67,
            z: 9,
            progress: 255,
        }
    );
}

#[test]
fn block_break_target_uses_java_floor_coordinates() {
    assert_eq!(
        support_block_position((20.5, 64.0, -37.5, 0.0, 0.0)),
        (20, 63, -38)
    );
}
