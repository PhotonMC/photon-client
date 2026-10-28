use super::*;

#[test]
fn set_health_parser_and_perform_respawn_encoder_match_java_order() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&0.0_f32.to_be_bytes());
    photon_core::wire::write_varint(20, &mut payload);
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
fn login_and_respawn_parsers_match_java_common_spawn_info_order() {
    fn append_identifier(output: &mut Vec<u8>, value: &str) {
        let identifier = photon_core::identifier::Identifier::parse(value).expect("identifier");
        photon_core::identifier::write_identifier(&identifier, output).expect("identifier bytes");
    }

    fn append_spawn_info(output: &mut Vec<u8>) {
        photon_core::wire::write_varint(1, output);
        append_identifier(output, "minecraft:the_nether");
        output.extend_from_slice(&0x0102_0304_0506_0708_i64.to_be_bytes());
        output.push(1);
        output.push(0);
        output.push(1);
        output.push(0);
        output.push(1);
        append_identifier(output, "minecraft:overworld");
        output.extend_from_slice(&pack_block_position((-3, 64, 7)).to_be_bytes());
        photon_core::wire::write_varint(42, output);
        photon_core::wire::write_varint(32, output);
    }

    let mut login_payload = Vec::new();
    login_payload.extend_from_slice(&17_i32.to_be_bytes());
    login_payload.push(1);
    photon_core::wire::write_varint(2, &mut login_payload);
    append_identifier(&mut login_payload, "minecraft:overworld");
    append_identifier(&mut login_payload, "minecraft:the_nether");
    photon_core::wire::write_varint(20, &mut login_payload);
    photon_core::wire::write_varint(5, &mut login_payload);
    photon_core::wire::write_varint(6, &mut login_payload);
    login_payload.push(0);
    login_payload.push(1);
    login_payload.push(0);
    append_spawn_info(&mut login_payload);
    login_payload.push(0);
    login_payload.push(1);

    let login = parse_login_packet(&login_payload).expect("login packet");
    assert_eq!(login.player_id, photon_core::ids::EntityId(17));
    assert!(login.hardcore);
    assert_eq!(login.level_count, 2);
    assert_eq!(login.max_players, 20);
    assert_eq!(login.chunk_radius, 5);
    assert_eq!(login.simulation_distance, 6);
    assert!(login.show_death_screen);
    assert_eq!(login.spawn_info.dimension_type_id, 1);
    assert_eq!(login.spawn_info.dimension, "minecraft:the_nether");
    assert_eq!(login.spawn_info.seed, 0x0102_0304_0506_0708);
    assert_eq!(login.spawn_info.game_type, 1);
    assert_eq!(login.spawn_info.previous_game_type, Some(0));
    assert!(login.spawn_info.is_debug);
    assert!(!login.spawn_info.is_flat);
    assert_eq!(login.spawn_info.portal_cooldown, 42);
    assert_eq!(login.spawn_info.sea_level, 32);
    assert!(!login.online_mode);
    assert!(login.enforces_secure_chat);
    assert_eq!(
        login.spawn_info.last_death_location,
        Some(super::super::GlobalPositionObservation {
            dimension: "minecraft:overworld".to_owned(),
            position: pack_block_position((-3, 64, 7)),
        })
    );

    let mut respawn_payload = Vec::new();
    append_spawn_info(&mut respawn_payload);
    respawn_payload.push(3);
    let (spawn_info, data_to_keep) = parse_respawn_packet(&respawn_payload).expect("respawn");
    assert_eq!(spawn_info, login.spawn_info);
    assert_eq!(data_to_keep, 3);

    respawn_payload.push(0);
    assert!(parse_respawn_packet(&respawn_payload).is_err());
}

#[test]
fn game_event_parser_matches_java_id_and_float_order() {
    let payload = [GAME_EVENT_CHANGE_GAME_MODE_ID, 0x3f, 0x80, 0x00, 0x00];
    assert_eq!(
        parse_game_event_packet(&payload).expect("game event packet"),
        super::super::GameEventPacket {
            event_id: GAME_EVENT_CHANGE_GAME_MODE_ID,
            param: 1.0,
        }
    );

    let mut trailing = payload.to_vec();
    trailing.push(0);
    assert!(parse_game_event_packet(&trailing).is_err());
}

#[test]
fn combat_kill_names_who_died_and_the_message() {
    let packet = [0xE8, 0x07, 8, 0, 2, b'h', b'i'];
    let (id, message) = super::super::parse_player_combat_kill_packet(&packet).unwrap();
    assert_eq!(id, photon_core::ids::EntityId(1000));
    assert_eq!(message, photon_core::text::Component::literal("hi"));
}

#[test]
fn a_disconnect_carries_its_reason() {
    let mut input: &[u8] = &[8, 0, 2, b'h', b'i'];
    assert_eq!(
        super::super::read_disconnect_reason(&mut input).unwrap(),
        photon_core::text::Component::literal("hi")
    );
}

#[test]
fn player_position_parser_matches_java_field_order_and_relative_resolution() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(17, &mut payload);
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
fn test_instance_status_carries_the_words_and_an_optional_size() {
    let with_size = [8, 0, 2, b'h', b'i', 1, 3, 4, 5];
    let packet = <crate::play::TestInstanceBlockStatus as crate::play::Decode>::decode(
        &with_size,
        &registries(),
    )
    .unwrap();
    assert_eq!(packet.status, photon_core::text::Component::literal("hi"));
    assert_eq!(packet.size, Some((3, 4, 5)));
    let without = [8, 0, 2, b'h', b'i', 0];
    assert_eq!(
        <crate::play::TestInstanceBlockStatus as crate::play::Decode>::decode(
            &without,
            &registries()
        )
        .unwrap()
        .size,
        None
    );
    assert!(
        <crate::play::TestInstanceBlockStatus as crate::play::Decode>::decode(
            &[8, 0, 2, b'h', b'i', 0, 0],
            &registries(),
        )
        .is_err()
    );
}
