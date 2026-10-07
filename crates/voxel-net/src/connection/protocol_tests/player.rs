use super::*;

#[test]
fn player_position_parser_matches_java_field_order_and_relative_resolution() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(17, &mut payload);
    for value in [1.25_f64, -63.0, 4.5, 0.125, 0.0, -0.25] {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    payload.extend_from_slice(&30.0_f32.to_be_bytes());
    payload.extend_from_slice(&100.0_f32.to_be_bytes());
    payload.extend_from_slice(&25_i32.to_be_bytes());

    let packet = parse_player_position_packet(&payload).expect("player position packet");
    assert_eq!(packet.teleport_id, 17);
    assert_eq!(packet.x, 1.25);
    assert_eq!(packet.y, -63.0);
    assert_eq!(packet.z, 4.5);
    assert_eq!(packet.delta_x, 0.125);
    assert_eq!(packet.delta_y, 0.0);
    assert_eq!(packet.delta_z, -0.25);
    assert_eq!(packet.y_rot, 30.0);
    assert_eq!(packet.x_rot, 100.0);
    assert_eq!(packet.relative_flags, 25);

    assert_eq!(
        resolve_player_position(packet, Some((10.0, 20.0, 30.0, 350.0, -80.0))),
        (11.25, -63.0, 4.5, 380.0, 20.0)
    );
}

#[test]
fn player_abilities_parser_matches_java_flag_and_float_order() {
    let payload = [0xff, 0x3e, 0x4c, 0xcc, 0xcd, 0x3e, 0x99, 0x99, 0x9a];
    let packet = parse_player_abilities_packet(&payload).expect("player abilities packet");
    assert!(packet.invulnerable);
    assert!(packet.flying);
    assert!(packet.can_fly);
    assert!(packet.can_instabuild);
    assert_eq!(packet.flying_speed.to_bits(), 0x3e4c_cccd);
    assert_eq!(packet.walking_speed.to_bits(), 0x3e99_999a);

    let mut trailing = payload.to_vec();
    trailing.push(0);
    assert!(parse_player_abilities_packet(&trailing).is_err());
}

#[test]
fn player_action_encoder_matches_java_abort_field_order() {
    let mut payload = Vec::new();
    encode_abort_destroy_block(&mut payload, (-12, 67, 9), 1).expect("action packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("action"),
        1
    );
    let (packed, rest) = input.split_at(8);
    input = rest;
    assert_eq!(
        i64::from_be_bytes(packed.try_into().expect("packed block position")),
        pack_block_position((-12, 67, 9))
    );
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        1
    );
    assert!(input.is_empty());
}

#[test]
fn player_action_encoder_matches_java_stop_action() {
    let mut payload = Vec::new();
    encode_stop_destroy_block(&mut payload, (-12, 67, 9), 9).expect("stop action packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("action"),
        2
    );
    input = &input[8..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        9
    );
    assert!(input.is_empty());
}

#[test]
fn player_action_encoder_matches_java_release_use_item() {
    let mut payload = Vec::new();
    encode_release_use_item(&mut payload).expect("release use item packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("action"),
        5
    );
    assert_eq!(input[..8], [0; 8]);
    input = &input[8..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn player_chat_reads_the_text_the_type_and_the_sender() {
    let mut payload = vec![0]; // global index
    payload.extend([0; 16]); // sender
    payload.push(0); // index
    payload.push(0); // no signature
    payload.extend([2, b'h', b'i']); // content
    payload.extend([0; 16]); // timestamp and salt
    payload.push(0); // last seen
    payload.push(0); // no unsigned content
    payload.push(0); // filter mask: pass through
    payload.push(1); // chat type: registry id 0
    payload.extend([8, 0, 3, b'B', b'o', b'b']); // the sender's name
    payload.push(0); // no target
    let packet = super::super::parse_player_chat_packet(&payload).unwrap();
    assert_eq!(packet.global_index, 0);
    assert_eq!(packet.signed_text, "hi");
    assert_eq!(packet.signature, None);
    assert_eq!(packet.filter, super::super::FilterMask::PassThrough);
    assert_eq!(packet.bound.holder, super::super::ChatHolder::Registry(0));
    assert_eq!(
        packet.bound.name,
        voxel_game::text::Component::literal("Bob")
    );
}

#[test]
fn player_info_update_reads_its_actions_in_order() {
    // Add, update listed, update latency for one player.
    let mut payload = vec![1 | 8 | 16, 1];
    payload.extend([0; 15]);
    payload.push(9); // uuid
    payload.extend([2, b'A', b'l', 0]); // name, no properties
    payload.extend([1, 50]); // listed, latency
    let entries = super::super::parse_player_info_update(&payload).unwrap();
    assert_eq!(entries[0].0, (0, 9));
    assert_eq!(
        entries[0].1,
        vec![
            super::super::InfoAction::Add { name: "Al".into() },
            super::super::InfoAction::Listed(true),
            super::super::InfoAction::Latency(50),
        ]
    );
}
