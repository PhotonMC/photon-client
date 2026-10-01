//! Wire-level protocol tests for the play/status packet codecs.

#![expect(
    clippy::cognitive_complexity,
    reason = "a scenario test reads as one sequence of steps"
)]

use super::{
    GAME_EVENT_CHANGE_GAME_MODE_ID, InventoryComponentCodec, PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID,
    PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID, PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID,
    PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID, PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
    PLAY_CLIENTBOUND_CHUNK_PACKET_ID, PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID,
    PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
    PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID, PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID,
    PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID, PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID,
    PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID, PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID,
    PLAY_CLIENTBOUND_LOGIN_PACKET_ID, PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID,
    PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID, PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID,
    PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID, PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID,
    PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID, PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
    PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID, PLAY_CLIENTBOUND_RESPAWN_PACKET_ID,
    PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID, PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
    PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID, PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
    PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID, PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID,
    PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID, PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID,
    PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID, PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID,
    PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID, PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID,
    PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID, PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID,
    PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
    PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID, PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
    PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID, PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID, PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID,
    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID, PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
    STATUS_CLIENTBOUND_PONG_RESPONSE_PACKET_ID, STATUS_CLIENTBOUND_RESPONSE_PACKET_ID,
    STATUS_SERVERBOUND_PING_REQUEST_PACKET_ID, STATUS_SERVERBOUND_REQUEST_PACKET_ID,
    VANILLA_ITEM_COMPONENT_CONFORMANCE, decode_packet_frame, encode_abort_destroy_block,
    encode_creative_item_slot, encode_creative_slot, encode_creative_stone_slot,
    encode_inventory_click_with_input, encode_inventory_drag_empty_cancel_state_correction,
    encode_inventory_pickup_state_correction,
    encode_inventory_pickup_with_component_hash_prediction,
    encode_inventory_pickup_with_component_spec_prediction,
    encode_inventory_pickup_with_partial_changed_slot_prediction,
    encode_inventory_pickup_with_prediction, encode_inventory_pickup_with_wrong_both_prediction,
    encode_inventory_pickup_with_wrong_carried_prediction,
    encode_inventory_pickup_with_wrong_slot_prediction,
    encode_inventory_quick_move_state_correction,
    encode_inventory_secondary_pickup_state_correction, encode_inventory_swap_state_correction,
    encode_inventory_throw_all_state_correction, encode_inventory_throw_one_state_correction,
    encode_packet_frame, encode_perform_respawn, encode_release_use_item, encode_set_carried_item,
    encode_stop_destroy_block, encode_use_item, encode_use_item_on, encode_use_item_on_hit,
    pack_block_position, parse_add_entity_packet, parse_block_destruction_packet,
    parse_block_entity_data_packet, parse_block_update_packet, parse_chunk_packet,
    parse_container_close_packet, parse_container_set_content_packet,
    parse_container_set_slot_packet, parse_cursor_item_packet, parse_entity_metadata_packet,
    parse_entity_position_sync_packet, parse_entity_teleport_packet,
    parse_forget_level_chunk_packet, parse_game_event_packet, parse_light_update_packet,
    parse_login_packet, parse_move_entity_pos_packet, parse_move_entity_pos_rot_packet,
    parse_move_entity_rot_packet, parse_open_screen_packet, parse_optional_item_stack,
    parse_player_abilities_packet, parse_player_position_packet, parse_remove_entities_packet,
    parse_remove_mob_effect_packet, parse_respawn_packet, parse_rotate_head_packet,
    parse_section_data, parse_set_chunk_cache_center_packet, parse_set_chunk_cache_radius_packet,
    parse_set_entity_motion_packet, parse_set_health_packet, parse_set_passengers_packet,
    parse_set_player_inventory_packet, parse_system_chat_packet, parse_update_attributes_packet,
    parse_update_mob_effect_packet, resolve_player_position, support_block_position,
};

#[test]
fn compression_uses_java_boundary_and_round_trips() {
    let below = vec![0x5a; 255];
    let at = vec![0xa5; 256];

    let below_frame = encode_packet_frame(&below, Some(256)).expect("frame below threshold");
    let mut below_header = below_frame.as_slice();
    voxel_core::wire::read_varint(&mut below_header).expect("below frame header");
    assert_eq!(below_header[0], 0x00);
    assert_eq!(decode_packet_frame(below_header, Some(256)).unwrap(), below);

    let at_frame = encode_packet_frame(&at, Some(256)).expect("frame at threshold");
    let mut at_header = at_frame.as_slice();
    voxel_core::wire::read_varint(&mut at_header).expect("at frame header");
    assert_ne!(at_header[0], 0x00);
    assert_eq!(decode_packet_frame(at_header, Some(256)).unwrap(), at);
}

#[test]
fn play_packet_ids_match_java_26_2_dispatch_order() {
    assert_eq!(PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID, 1);
    assert_eq!(PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID, 35);
    assert_eq!(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID, 53);
    assert_eq!(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID, 54);
    assert_eq!(PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID, 56);
    assert_eq!(PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID, 83);
    assert_eq!(PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID, 101);
    assert_eq!(PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID, 107);
    assert_eq!(PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID, 44);
    assert_eq!(PLAY_CLIENTBOUND_CHUNK_PACKET_ID, 45);
    assert_eq!(PLAY_CLIENTBOUND_LOGIN_PACKET_ID, 49);
    assert_eq!(PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID, 72);
    assert_eq!(PLAY_CLIENTBOUND_RESPAWN_PACKET_ID, 82);
    assert_eq!(PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID, 104);
    assert_eq!(PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID, 131);
    assert_eq!(PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID, 8);
    assert_eq!(PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID, 6);
    assert_eq!(PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID, 37);
    assert_eq!(PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID, 38);
    assert_eq!(PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID, 48);
    assert_eq!(PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID, 94);
    assert_eq!(PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID, 95);
    assert_eq!(PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID, 4);
    assert_eq!(PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID, 5);
    assert_eq!(PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID, 18);
    assert_eq!(PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID, 20);
    assert_eq!(PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID, 17);
    assert_eq!(PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID, 59);
    assert_eq!(PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID, 64);
    assert_eq!(PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID, 96);
    assert_eq!(PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID, 108);
    assert_eq!(PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID, 77);
    assert_eq!(PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID, 78);
    assert_eq!(PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID, 99);
    assert_eq!(PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID, 125);
    assert_eq!(PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, 121);
    assert_eq!(PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID, 132);
    assert_eq!(PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, 0);
    assert_eq!(PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID, 41);
    assert_eq!(PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, 11);
    assert_eq!(PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID, 12);
    assert_eq!(PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID, 13);
    assert_eq!(PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID, 28);
    assert_eq!(PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, 31);
    assert_eq!(PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID, 44);
    assert_eq!(PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID, 18);
    assert_eq!(PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, 53);
    assert_eq!(PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID, 56);
    assert_eq!(PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID, 66);
    assert_eq!(PLAY_SERVERBOUND_USE_ITEM_PACKET_ID, 67);
    assert_eq!(GAME_EVENT_CHANGE_GAME_MODE_ID, 3);
}

#[test]
fn status_packet_ids_match_java_26_2_dispatch_order() {
    assert_eq!(STATUS_SERVERBOUND_REQUEST_PACKET_ID, 0);
    assert_eq!(STATUS_SERVERBOUND_PING_REQUEST_PACKET_ID, 1);
    assert_eq!(STATUS_CLIENTBOUND_RESPONSE_PACKET_ID, 0);
    assert_eq!(STATUS_CLIENTBOUND_PONG_RESPONSE_PACKET_ID, 1);
}

#[test]
fn chunk_packet_parser_consumes_empty_java_payload() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&(-17_i32).to_be_bytes());
    payload.extend_from_slice(&23_i32.to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    for _ in 0..6 {
        voxel_core::wire::write_varint(0, &mut payload);
    }

    let observation = parse_chunk_packet(&payload).expect("empty chunk payload");
    assert_eq!(observation.x, -17);
    assert_eq!(observation.z, 23);
    assert_eq!(observation.heightmap_count, 0);
    assert!(observation.heightmap_observations.is_empty());
    assert_eq!(observation.section_data_length, 0);
    assert_eq!(observation.block_entity_count, 0);
    assert_eq!(observation.sky_mask_word_count, 0);
    assert_eq!(observation.block_mask_word_count, 0);
    assert_eq!(observation.empty_sky_mask_word_count, 0);
    assert_eq!(observation.empty_block_mask_word_count, 0);
    assert_eq!(observation.sky_update_count, 0);
    assert!(observation.sky_update_lengths.is_empty());
    assert_eq!(observation.block_update_count, 0);
    assert!(observation.block_update_lengths.is_empty());
}

#[test]
fn chunk_packet_parser_consumes_nullable_block_entity_nbt() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&0_i32.to_be_bytes());
    payload.extend_from_slice(&0_i32.to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    payload.push(0x21);
    payload.extend_from_slice(&(-64_i16).to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);
    payload.push(0);
    for _ in 0..6 {
        voxel_core::wire::write_varint(0, &mut payload);
    }

    let observation = parse_chunk_packet(&payload).expect("nullable block entity tag");
    assert_eq!(observation.block_entity_count, 1);
    assert_eq!(
        observation.block_entities,
        [super::BlockEntityObservation {
            x: 2,
            y: -64,
            z: 1,
            type_id: 0,
            tag_length: 0,
            applied_to_world: false,
            tag: None,
        }]
    );
}

#[test]
fn block_entity_data_packet_parser_decodes_packed_position_and_nbt() {
    let mut payload = pack_block_position((-12, -63, 9)).to_be_bytes().to_vec();
    voxel_core::wire::write_varint(42, &mut payload);
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
fn chunk_packet_parser_records_heightmap_and_light_lengths() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&4_i32.to_be_bytes());
    payload.extend_from_slice(&(-9_i32).to_be_bytes());

    voxel_core::wire::write_varint(2, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(2, &mut payload);
    payload.extend_from_slice(&[0; 16]);
    voxel_core::wire::write_varint(4, &mut payload);
    voxel_core::wire::write_varint(3, &mut payload);
    payload.extend_from_slice(&[0; 24]);

    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    for _ in 0..4 {
        voxel_core::wire::write_varint(0, &mut payload);
    }

    voxel_core::wire::write_varint(2, &mut payload);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0xa5; 2_048]);
    voxel_core::wire::write_varint(3, &mut payload);
    payload.extend_from_slice(&[0x5a; 3]);

    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0x3c; 2_048]);

    let observation = parse_chunk_packet(&payload).expect("heightmap and light lengths");
    assert_eq!(observation.heightmap_count, 2);
    assert_eq!(
        observation.heightmap_observations,
        [
            super::HeightmapObservation {
                type_id: 1,
                word_count: 2,
            },
            super::HeightmapObservation {
                type_id: 4,
                word_count: 3,
            },
        ]
    );
    assert_eq!(observation.sky_update_count, 2);
    assert_eq!(observation.sky_update_lengths, [2_048, 3]);
    assert_eq!(observation.block_update_count, 1);
    assert_eq!(observation.block_update_lengths, [2_048]);
}

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
fn login_and_respawn_parsers_match_java_common_spawn_info_order() {
    fn append_identifier(output: &mut Vec<u8>, value: &str) {
        let identifier = voxel_core::identifier::Identifier::parse(value).expect("identifier");
        voxel_core::identifier::write_identifier(&identifier, output).expect("identifier bytes");
    }

    fn append_spawn_info(output: &mut Vec<u8>) {
        voxel_core::wire::write_varint(1, output);
        append_identifier(output, "minecraft:the_nether");
        output.extend_from_slice(&0x0102_0304_0506_0708_i64.to_be_bytes());
        output.push(1);
        output.push(0);
        output.push(1);
        output.push(0);
        output.push(1);
        append_identifier(output, "minecraft:overworld");
        output.extend_from_slice(&pack_block_position((-3, 64, 7)).to_be_bytes());
        voxel_core::wire::write_varint(42, output);
        voxel_core::wire::write_varint(32, output);
    }

    let mut login_payload = Vec::new();
    login_payload.extend_from_slice(&17_i32.to_be_bytes());
    login_payload.push(1);
    voxel_core::wire::write_varint(2, &mut login_payload);
    append_identifier(&mut login_payload, "minecraft:overworld");
    append_identifier(&mut login_payload, "minecraft:the_nether");
    voxel_core::wire::write_varint(20, &mut login_payload);
    voxel_core::wire::write_varint(5, &mut login_payload);
    voxel_core::wire::write_varint(6, &mut login_payload);
    login_payload.push(0);
    login_payload.push(1);
    login_payload.push(0);
    append_spawn_info(&mut login_payload);
    login_payload.push(0);
    login_payload.push(1);

    let login = parse_login_packet(&login_payload).expect("login packet");
    assert_eq!(login.player_id, 17);
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
        Some(super::GlobalPositionObservation {
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
fn set_health_parser_and_perform_respawn_encoder_match_java_order() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&0.0_f32.to_be_bytes());
    voxel_core::wire::write_varint(20, &mut payload);
    payload.extend_from_slice(&5.5_f32.to_be_bytes());

    assert_eq!(
        parse_set_health_packet(&payload).expect("set health packet"),
        super::HealthObservation {
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
fn game_event_parser_matches_java_id_and_float_order() {
    let payload = [GAME_EVENT_CHANGE_GAME_MODE_ID, 0x3f, 0x80, 0x00, 0x00];
    assert_eq!(
        parse_game_event_packet(&payload).expect("game event packet"),
        super::GameEventPacket {
            event_id: GAME_EVENT_CHANGE_GAME_MODE_ID,
            param: 1.0,
        }
    );

    let mut trailing = payload.to_vec();
    trailing.push(0);
    assert!(parse_game_event_packet(&trailing).is_err());
}

#[test]
fn update_attributes_parser_matches_java_snapshot_and_modifier_order() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(42, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(17, &mut payload);
    payload.extend_from_slice(&2.5_f64.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::identifier::write_identifier(
        &voxel_core::identifier::Identifier::parse("minecraft:test_modifier")
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
fn block_update_parser_matches_java_packed_position_and_state() {
    let mut payload = pack_block_position((-12, -63, 9)).to_be_bytes().to_vec();
    voxel_core::wire::write_varint(123, &mut payload);

    assert_eq!(
        parse_block_update_packet(&payload).expect("block update"),
        super::BlockUpdateObservation {
            x: -12,
            y: -63,
            z: 9,
            state_id: 123,
        }
    );
}

#[test]
fn forget_level_chunk_parser_matches_java_chunk_pos_packing() {
    let packed = ((2_u64 << 32) | u64::from(u32::MAX)) as i64;

    assert_eq!(
        parse_forget_level_chunk_packet(&packed.to_be_bytes()).expect("chunk position"),
        (-1, 2)
    );
    assert!(parse_forget_level_chunk_packet(&packed.to_be_bytes()[..7]).is_err());

    let mut trailing = packed.to_be_bytes().to_vec();
    trailing.push(0);
    assert!(parse_forget_level_chunk_packet(&trailing).is_err());
}

#[test]
fn chunk_cache_center_and_radius_parsers_match_varint_payloads() {
    let mut center = Vec::new();
    voxel_core::wire::write_varint(-12, &mut center);
    voxel_core::wire::write_varint(37, &mut center);
    assert_eq!(
        parse_set_chunk_cache_center_packet(&center).expect("chunk cache center"),
        (-12, 37)
    );

    let mut radius = Vec::new();
    voxel_core::wire::write_varint(6, &mut radius);
    assert_eq!(
        parse_set_chunk_cache_radius_packet(&radius).expect("chunk cache radius"),
        6
    );

    radius.push(0);
    assert!(parse_set_chunk_cache_radius_packet(&radius).is_err());
}

#[test]
fn light_update_parser_matches_java_masks_and_data_layer_lengths() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(-3, &mut payload);
    voxel_core::wire::write_varint(7, &mut payload);
    voxel_core::binary::write_bitset(
        &voxel_core::binary::BitSet::from_bits(&[0, 2]),
        &mut payload,
    );
    voxel_core::binary::write_bitset(&voxel_core::binary::BitSet::from_bits(&[1]), &mut payload);
    voxel_core::binary::write_bitset(&voxel_core::binary::BitSet::from_bits(&[3]), &mut payload);
    voxel_core::binary::write_bitset(&voxel_core::binary::BitSet::from_bits(&[4]), &mut payload);
    voxel_core::wire::write_varint(2, &mut payload);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0x12; 2_048]);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0x34; 2_048]);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0x56; 2_048]);

    let observation = parse_light_update_packet(&payload).expect("light update");
    assert_eq!((observation.x, observation.z), (-3, 7));
    assert_eq!(observation.sky_mask_word_count, 1);
    assert_eq!(observation.block_mask_word_count, 1);
    assert_eq!(observation.empty_sky_mask_word_count, 1);
    assert_eq!(observation.empty_block_mask_word_count, 1);
    assert_eq!(observation.sky_update_lengths, [2_048, 2_048]);
    assert_eq!(observation.block_update_lengths, [2_048]);
    assert_eq!(observation.applied_sky_sections, 0);
    assert_eq!(observation.applied_block_sections, 0);

    let mut trailing = payload;
    trailing.push(0);
    assert!(parse_light_update_packet(&trailing).is_err());
}

#[test]
fn block_destruction_parser_matches_java_progress_fields() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(37, &mut payload);
    payload.extend_from_slice(&pack_block_position((-12, 67, 9)).to_be_bytes());
    payload.push(255);

    assert_eq!(
        parse_block_destruction_packet(&payload).expect("block destruction"),
        super::BlockDestructionObservation {
            entity_id: 37,
            x: -12,
            y: 67,
            z: 9,
            progress: 255,
        }
    );
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
        super::AddEntityObservation {
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
        super::EntityMotionObservation {
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
        super::EntityHeadRotationObservation {
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
        super::RemoveEntitiesObservation {
            entity_ids: vec![37, -12, 0],
            applied_entity_count: 0,
        }
    );

    let mut negative_count = Vec::new();
    voxel_core::wire::write_varint(-1, &mut negative_count);
    assert_eq!(
        parse_remove_entities_packet(&negative_count).expect("negative count"),
        super::RemoveEntitiesObservation {
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
        super::EntityPassengersObservation {
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
        super::EntityMetadataObservation {
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
        super::EntityTeleportObservation {
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
        super::EntityPositionSyncObservation {
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
        super::EntityMoveObservation {
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
        super::EntityMoveObservation {
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
        super::EntityMoveObservation {
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
        super::MobEffectUpdateObservation {
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
        super::MobEffectRemoveObservation {
            entity_id: 42,
            effect_registry_id: 19,
            applied_to_world: false,
        }
    );
}

#[test]
fn system_chat_parser_matches_translatable_overlay_component() {
    let mut payload = Vec::new();
    voxel_core::nbt::write_nbt(
        Some(&[voxel_core::nbt::Field {
            key: "translate".to_owned(),
            value: voxel_core::nbt::FieldValue::Str("build.spawn_protection".to_owned()),
        }]),
        &mut payload,
    );
    payload.push(1);

    assert_eq!(
        parse_system_chat_packet(&payload).expect("system chat packet"),
        super::SystemChatObservation {
            overlay: true,
            translation_key: Some("build.spawn_protection".to_owned()),
        }
    );
}

#[test]
fn container_slot_parser_matches_empty_optional_item_stack() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(7, &mut payload);
    payload.extend_from_slice(&36_i16.to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_container_set_slot_packet(&payload).expect("container slot packet"),
        super::ContainerSetSlotObservation {
            container_id: 0,
            state_id: 7,
            slot: 36,
            item_count: None,
            item_registry_id: None,
            positive_component_type_ids: Vec::new(),
            negative_component_type_ids: Vec::new(),
            item: None,
            applied_to_world: false,
        }
    );
}

#[test]
fn container_slot_parser_skips_custom_model_data_component() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(8, &mut payload);
    payload.extend_from_slice(&36_i16.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(1_046, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(17, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    payload.extend_from_slice(&1.0_f32.to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_container_set_slot_packet(&payload).expect("component-bearing container slot"),
        super::ContainerSetSlotObservation {
            container_id: 0,
            state_id: 8,
            slot: 36,
            item_count: Some(1),
            item_registry_id: Some(1_046),
            positive_component_type_ids: vec![17],
            negative_component_type_ids: Vec::new(),
            item: Some(super::ClientItemStackState::from_wire(
                1,
                1_046,
                vec![17],
                Vec::new(),
            )),
            applied_to_world: false,
        }
    );
}

#[test]
fn container_slot_parser_skips_food_component() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(10, &mut payload);
    payload.extend_from_slice(&36_i16.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(1_089, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(23, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    payload.extend_from_slice(&0.1_f32.to_be_bytes());
    payload.push(1);

    assert_eq!(
        parse_container_set_slot_packet(&payload).expect("food component-bearing container slot"),
        super::ContainerSetSlotObservation {
            container_id: 0,
            state_id: 10,
            slot: 36,
            item_count: Some(1),
            item_registry_id: Some(1_089),
            positive_component_type_ids: vec![23],
            negative_component_type_ids: Vec::new(),
            item: Some(super::ClientItemStackState::from_wire(
                1,
                1_089,
                vec![23],
                Vec::new(),
            )),
            applied_to_world: false,
        }
    );
}

#[test]
fn container_slot_parser_skips_damage_tool_and_weapon_components() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(9, &mut payload);
    payload.extend_from_slice(&36_i16.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(1_000, &mut payload);
    voxel_core::wire::write_varint(3, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);

    voxel_core::wire::write_varint(3, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);

    voxel_core::wire::write_varint(28, &mut payload);
    voxel_core::wire::write_varint(2, &mut payload);
    let incorrect_tag = b"minecraft:incorrect_for_wooden_tool";
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(incorrect_tag.len() as i32, &mut payload);
    payload.extend_from_slice(incorrect_tag);
    payload.push(0);
    payload.push(1);
    payload.push(0);
    voxel_core::wire::write_varint(0, &mut payload);
    let mineable_tag = b"minecraft:mineable/shovel";
    voxel_core::wire::write_varint(mineable_tag.len() as i32, &mut payload);
    payload.extend_from_slice(mineable_tag);
    payload.push(1);
    payload.extend_from_slice(&2.0_f32.to_be_bytes());
    payload.push(1);
    payload.push(1);
    payload.extend_from_slice(&1.0_f32.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    payload.push(1);

    voxel_core::wire::write_varint(29, &mut payload);
    voxel_core::wire::write_varint(2, &mut payload);
    payload.extend_from_slice(&0.0_f32.to_be_bytes());

    assert_eq!(
        parse_container_set_slot_packet(&payload).expect("tool component-bearing slot"),
        super::ContainerSetSlotObservation {
            container_id: 0,
            state_id: 9,
            slot: 36,
            item_count: Some(1),
            item_registry_id: Some(1_000),
            positive_component_type_ids: vec![3, 28, 29],
            negative_component_type_ids: Vec::new(),
            item: Some(super::ClientItemStackState::from_wire(
                1,
                1_000,
                vec![3, 28, 29],
                Vec::new(),
            )),
            applied_to_world: false,
        }
    );
}

#[test]
fn container_content_parser_matches_empty_hotbar_stack() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(9, &mut payload);
    voxel_core::wire::write_varint(37, &mut payload);
    for _ in 0..37 {
        voxel_core::wire::write_varint(0, &mut payload);
    }
    voxel_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_container_set_content_packet(&payload).expect("container content packet"),
        super::ContainerSetContentObservation {
            container_id: 0,
            state_id: 9,
            item_count: 37,
            items: vec![None; 37],
            hotbar_slot_zero_item_count: None,
            hotbar_slot_zero_item_registry_id: None,
            hotbar_slot_zero_positive_component_type_ids: Vec::new(),
            hotbar_slot_zero_negative_component_type_ids: Vec::new(),
            main_inventory_slot_zero_item_count: None,
            main_inventory_slot_zero_item_registry_id: None,
            main_inventory_slot_zero_positive_component_type_ids: Vec::new(),
            main_inventory_slot_zero_negative_component_type_ids: Vec::new(),
            carried_item_count: None,
            carried_item_registry_id: None,
            carried_positive_component_type_ids: Vec::new(),
            carried_negative_component_type_ids: Vec::new(),
            carried_item: None,
            applied_to_world: false,
        }
    );
}

#[test]
fn container_content_parser_observes_main_inventory_slot_zero() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(10, &mut payload);
    voxel_core::wire::write_varint(46, &mut payload);
    for slot in 0..46 {
        if slot == 9 {
            voxel_core::wire::write_varint(1, &mut payload);
            voxel_core::wire::write_varint(1, &mut payload);
            voxel_core::wire::write_varint(0, &mut payload);
            voxel_core::wire::write_varint(0, &mut payload);
        } else {
            voxel_core::wire::write_varint(0, &mut payload);
        }
    }
    voxel_core::wire::write_varint(0, &mut payload);

    let observation =
        parse_container_set_content_packet(&payload).expect("main inventory slot packet");
    assert_eq!(observation.main_inventory_slot_zero_item_count, Some(1));
    assert_eq!(
        observation.main_inventory_slot_zero_item_registry_id,
        Some(1)
    );
    assert!(
        observation
            .main_inventory_slot_zero_positive_component_type_ids
            .is_empty()
    );
    assert!(
        observation
            .main_inventory_slot_zero_negative_component_type_ids
            .is_empty()
    );
    assert_eq!(
        observation.items[9],
        Some(super::ClientItemStackState::from_wire(1, 1, vec![], vec![]))
    );
    assert_eq!(observation.items.len(), 46);
    assert!(observation.carried_item.is_none());
    assert!(!observation.applied_to_world);
}

#[test]
fn cursor_and_direct_player_inventory_parsers_match_java_optional_stack_codecs() {
    let cursor_payload = [0];
    assert_eq!(
        parse_cursor_item_packet(&cursor_payload).expect("empty cursor item packet"),
        super::CursorItemObservation {
            item_count: None,
            item_registry_id: None,
            positive_component_type_ids: Vec::new(),
            negative_component_type_ids: Vec::new(),
            item: None,
            applied_to_world: false,
        }
    );

    let mut inventory_payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut inventory_payload);
    voxel_core::wire::write_varint(1, &mut inventory_payload);
    voxel_core::wire::write_varint(1, &mut inventory_payload);
    voxel_core::wire::write_varint(0, &mut inventory_payload);
    voxel_core::wire::write_varint(0, &mut inventory_payload);
    assert_eq!(
        parse_set_player_inventory_packet(&inventory_payload)
            .expect("direct player inventory packet"),
        super::SetPlayerInventoryObservation {
            slot: 0,
            item_count: Some(1),
            item_registry_id: Some(1),
            positive_component_type_ids: Vec::new(),
            negative_component_type_ids: Vec::new(),
            item: Some(super::ClientItemStackState::from_wire(
                1,
                1,
                Vec::new(),
                Vec::new(),
            )),
            applied_to_world: false,
        }
    );
}

#[test]
fn open_screen_and_container_close_parsers_preserve_java_fields() {
    let title = voxel_core::nbt::FieldValue::Compound(vec![voxel_core::nbt::Field {
        key: "text".to_owned(),
        value: voxel_core::nbt::FieldValue::Str("Chest".to_owned()),
    }]);
    let mut title_payload = Vec::new();
    voxel_core::nbt::write_any_nbt(Some(&title), &mut title_payload);

    let mut open_payload = Vec::new();
    voxel_core::wire::write_varint(7, &mut open_payload);
    voxel_core::wire::write_varint(2, &mut open_payload);
    open_payload.extend_from_slice(&title_payload);
    assert_eq!(
        parse_open_screen_packet(&open_payload).expect("open screen packet"),
        super::OpenScreenObservation {
            container_id: 7,
            menu_type_id: 2,
            title_nbt: title_payload,
            applied_to_world: false,
        }
    );

    let mut close_payload = Vec::new();
    voxel_core::wire::write_varint(7, &mut close_payload);
    assert_eq!(
        parse_container_close_packet(&close_payload).expect("container close packet"),
        super::ContainerCloseObservation {
            container_id: 7,
            applied_to_world: false,
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

#[test]
fn use_item_on_encoder_matches_java_block_hit_result_field_order() {
    let mut payload = Vec::new();
    encode_use_item_on(&mut payload, (-12, 67, 9), 4).expect("use item on packet");

    let mut input = payload.as_slice();
    assert_eq!(voxel_core::wire::read_varint(&mut input).expect("hand"), 0);
    let (packed, rest) = input.split_at(8);
    input = rest;
    assert_eq!(
        i64::from_be_bytes(packed.try_into().expect("packed block position")),
        pack_block_position((-12, 67, 9))
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("direction"),
        1
    );
    assert_eq!(input.len(), 4 * 3 + 2 + 1);
    assert_eq!(&input[0..4], &0.5_f32.to_be_bytes());
    assert_eq!(&input[4..8], &1.0_f32.to_be_bytes());
    assert_eq!(&input[8..12], &0.5_f32.to_be_bytes());
    assert_eq!(input[12], 0);
    assert_eq!(input[13], 0);
    input = &input[14..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        4
    );
    assert!(input.is_empty());
}

#[test]
fn creative_stone_slot_encoder_matches_java_optional_item_stack() {
    let mut payload = Vec::new();
    encode_creative_stone_slot(&mut payload, 36, 1).expect("creative slot packet");

    let mut input = payload.as_slice();
    let (slot, rest) = input.split_at(2);
    input = rest;
    assert_eq!(i16::from_be_bytes(slot.try_into().expect("slot")), 36);
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("positive components"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("negative components"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn creative_slot_encoder_writes_the_patch_the_stack_carries() {
    use voxel_world::ClientItemStackState as Stack;
    // A stack made here has no patch: both counts are zero.
    let mut payload = Vec::new();
    encode_creative_slot(&mut payload, 38, Some(&Stack::new(3, 1_044)));
    let mut input = &payload[2..];
    assert_eq!(
        i16::from_be_bytes(payload[..2].try_into().expect("slot")),
        38
    );
    assert_eq!(voxel_core::wire::read_varint(&mut input).expect("count"), 3);
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item"),
        1_044
    );
    assert_eq!(input, &[0, 0]);
    // A stack with a patch writes each value behind its length.
    let stack = Stack::from_wire(1, 5, vec![103], Vec::new()).with_patch_bytes(vec![1, 0, 103, 1]);
    let mut payload = Vec::new();
    encode_creative_slot(&mut payload, -1, Some(&stack));
    assert_eq!(
        i16::from_be_bytes(payload[..2].try_into().expect("slot")),
        -1
    );
    assert_eq!(&payload[2..], &[1, 5, 1, 0, 103, 1, 1]);
    // No stack is a count of zero.
    let mut payload = Vec::new();
    encode_creative_slot(&mut payload, 36, None);
    assert_eq!(&payload[2..], &[0]);
}

#[test]
fn creative_item_slot_encoder_keeps_registry_id_and_empty_components() {
    let mut payload = Vec::new();
    encode_creative_item_slot(&mut payload, 36, 1, 1_044).expect("creative item slot");

    let mut input = payload.as_slice();
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
        1_044
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("positive components"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("negative components"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn carried_item_encoder_matches_java_short_slot() {
    let mut payload = Vec::new();
    encode_set_carried_item(&mut payload, 0).expect("carried item packet");
    assert_eq!(payload, [0, 0]);
}

#[test]
fn inventory_click_encoder_matches_java_empty_prediction_fields() {
    let mut payload = Vec::new();
    encode_inventory_pickup_state_correction(&mut payload, 0).expect("inventory click packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_secondary_pickup_encoder_keeps_secondary_button() {
    let mut payload = Vec::new();
    encode_inventory_secondary_pickup_state_correction(&mut payload, 0)
        .expect("inventory secondary pickup packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_throw_encoder_keeps_throw_input_id() {
    let mut payload = Vec::new();
    encode_inventory_throw_one_state_correction(&mut payload, 0).expect("inventory throw packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        4
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_throw_all_encoder_keeps_secondary_throw_button() {
    let mut payload = Vec::new();
    encode_inventory_throw_all_state_correction(&mut payload, 0)
        .expect("inventory throw-all packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        4
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_drag_encoder_keeps_empty_slot_and_quick_craft_input_id() {
    let mut payload = Vec::new();
    encode_inventory_drag_empty_cancel_state_correction(&mut payload, 0)
        .expect("inventory drag packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("slot")),
        -999
    );
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        5
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_drag_distribution_encoder_keeps_slot_button_and_input() {
    let mut payload = Vec::new();
    encode_inventory_click_with_input(&mut payload, 0, -999, 2, 5)
        .expect("inventory drag distribution packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("slot")),
        -999
    );
    input = &input[2..];
    assert_eq!(input[0], 2);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        5
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_double_click_encoder_keeps_empty_target_and_pickup_all_input() {
    let mut payload = Vec::new();
    encode_inventory_click_with_input(&mut payload, 0, 10, 0, 6)
        .expect("inventory double-click packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 10);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        6
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_click_encoder_keeps_ahead_state_id() {
    let mut payload = Vec::new();
    encode_inventory_click_with_input(&mut payload, 99, 36, 0, 0)
        .expect("ahead-state inventory click packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        99
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_pickup_prediction_encoder_keeps_item_hash_fields() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_prediction(&mut payload, 2, 8, 1)
        .expect("inventory prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        2
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        1
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("changed slot")),
        36
    );
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        8
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("component map count"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("removed component count"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn inventory_wrong_slot_prediction_encoder_keeps_pre_click_slot_and_carried_hashes() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_wrong_slot_prediction(&mut payload, 2, 8, 1)
        .expect("wrong-slot inventory prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        2
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        1
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("changed slot")),
        36
    );
    input = &input[2..];

    for hash in 0..2 {
        assert_eq!(input[0], 1, "hash {hash} should be non-empty");
        input = &input[1..];
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("item registry id"),
            1
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("item count"),
            8
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("component map count"),
            0
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("removed component count"),
            0
        );
    }
    assert!(input.is_empty());
}

#[test]
fn inventory_wrong_carried_prediction_encoder_keeps_empty_carried_hash() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_wrong_carried_prediction(&mut payload, 2)
        .expect("wrong-carried inventory prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        2
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        1
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("changed slot")),
        36
    );
    input = &input[2..];
    assert_eq!(input[0], 0, "changed slot should be empty");
    input = &input[1..];
    assert_eq!(input[0], 0, "carried prediction should be empty");
    input = &input[1..];
    assert!(input.is_empty());
}

#[test]
fn inventory_wrong_both_prediction_encoder_keeps_slot_item_and_empty_carried_hash() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_wrong_both_prediction(&mut payload, 2, 8, 1)
        .expect("wrong-both inventory prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        2
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        1
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("changed slot")),
        36
    );
    input = &input[2..];
    assert_eq!(input[0], 1, "changed slot should be non-empty");
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        8
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("component map count"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("removed component count"),
        0
    );
    assert_eq!(input[0], 0, "carried prediction should be empty");
    input = &input[1..];
    assert!(input.is_empty());
}

#[test]
fn inventory_component_hash_prediction_encoder_keeps_registry_hash_fields() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_component_hash_prediction(&mut payload, 2, 1, 1_046)
        .expect("component-hash inventory prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        2
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        1
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("changed slot")),
        36
    );
    input = &input[2..];
    assert_eq!(input[0], 0, "changed slot should be empty");
    input = &input[1..];
    assert_eq!(input[0], 1, "carried should be non-empty");
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
        1_046
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("component map count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("component type id"),
        17
    );
    assert_eq!(
        i32::from_be_bytes(input[..4].try_into().expect("component hash")),
        0x0102_0304
    );
    input = &input[4..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("removed component count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("removed component type id"),
        17
    );
    assert!(input.is_empty());
}

fn write_component_fixture(codec: InventoryComponentCodec, output: &mut Vec<u8>) {
    match codec {
        InventoryComponentCodec::VarInt => voxel_core::wire::write_varint(0, output),
        // A holder writes its registry id plus one; a zero is a direct value.
        InventoryComponentCodec::RegistryId => voxel_core::wire::write_varint(1, output),
        InventoryComponentCodec::Unit => {}
        InventoryComponentCodec::Float => output.extend_from_slice(&0.0_f32.to_be_bytes()),
        InventoryComponentCodec::Byte => output.push(0),
        InventoryComponentCodec::Int => output.extend_from_slice(&0_i32.to_be_bytes()),
        InventoryComponentCodec::Identifier => {
            let identifier = voxel_core::identifier::Identifier::parse("minecraft:test")
                .expect("identifier fixture");
            voxel_core::identifier::write_identifier(&identifier, output)
                .expect("identifier fixture encoding");
        }
        InventoryComponentCodec::UseEffects => {
            output.push(1);
            output.push(0);
            output.extend_from_slice(&1.0_f32.to_be_bytes());
        }
        InventoryComponentCodec::CustomModelData => {
            voxel_core::wire::write_varint(1, output);
            output.extend_from_slice(&1.0_f32.to_be_bytes());
            for _ in 0..3 {
                voxel_core::wire::write_varint(0, output);
            }
        }
        InventoryComponentCodec::Food => {
            voxel_core::wire::write_varint(1, output);
            output.extend_from_slice(&0.1_f32.to_be_bytes());
            output.push(1);
        }
        InventoryComponentCodec::Tool => {
            voxel_core::wire::write_varint(0, output);
            output.extend_from_slice(&1.0_f32.to_be_bytes());
            voxel_core::wire::write_varint(0, output);
            output.push(1);
        }
        InventoryComponentCodec::Weapon => {
            voxel_core::wire::write_varint(1, output);
            output.extend_from_slice(&0.0_f32.to_be_bytes());
        }
    }
}

#[test]
fn inventory_item_component_conformance_encoder_uses_table_rows() {
    for spec in VANILLA_ITEM_COMPONENT_CONFORMANCE {
        let carried_ids: Vec<_> = spec
            .carried_components
            .iter()
            .map(|(component_type_id, _)| *component_type_id)
            .collect();
        assert_eq!(
            carried_ids.as_slice(),
            spec.expected_positive_component_type_ids,
            "table component ids for {}",
            spec.component_path
        );
        assert_eq!(
            spec.carried_components
                .iter()
                .find(|(component_type_id, _)| *component_type_id == spec.component_type_id)
                .map(|(_, hash)| *hash),
            Some(spec.component_hash),
            "table component hash for {}",
            spec.component_path
        );

        let mut payload = Vec::new();
        encode_inventory_pickup_with_component_spec_prediction(&mut payload, 2, spec)
            .expect("table component prediction packet");
        let mut input = payload.as_slice();
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("container id"),
            0
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("state id"),
            2
        );
        assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
        input = &input[2..];
        assert_eq!(input[0], 0, "button");
        input = &input[1..];
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("container input"),
            0
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
            1
        );
        assert_eq!(
            i16::from_be_bytes(input[..2].try_into().expect("changed slot")),
            36
        );
        input = &input[2..];
        assert_eq!(input[0], 0, "changed slot should be empty");
        input = &input[1..];
        assert_eq!(input[0], 1, "carried should be non-empty");
        input = &input[1..];
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("item registry id"),
            spec.item_registry_id
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("item count"),
            1
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("component map count"),
            spec.carried_components.len() as i32
        );
        for (expected_type_id, expected_hash) in spec.carried_components {
            assert_eq!(
                voxel_core::wire::read_varint(&mut input).expect("component type id"),
                *expected_type_id,
                "component type for {}",
                spec.component_path
            );
            assert_eq!(
                i32::from_be_bytes(input[..4].try_into().expect("component hash")),
                *expected_hash,
                "component hash for {}",
                spec.component_path
            );
            input = &input[4..];
        }
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("removed component count"),
            0
        );
        assert!(
            input.is_empty(),
            "trailing bytes for {}",
            spec.component_path
        );
    }
}

#[test]
fn inventory_item_component_conformance_parser_consumes_table_codecs() {
    for spec in VANILLA_ITEM_COMPONENT_CONFORMANCE {
        let mut payload = Vec::new();
        voxel_core::wire::write_varint(1, &mut payload);
        voxel_core::wire::write_varint(spec.item_registry_id, &mut payload);
        voxel_core::wire::write_varint(1, &mut payload);
        voxel_core::wire::write_varint(0, &mut payload);
        voxel_core::wire::write_varint(spec.component_type_id, &mut payload);
        write_component_fixture(spec.codec, &mut payload);

        let mut input = payload.as_slice();
        let observation = parse_optional_item_stack(&mut input, "table component item stack")
            .unwrap_or_else(|error| {
                panic!(
                    "table component item stack {}: {error:?}",
                    spec.component_path
                )
            });
        assert_eq!(
            observation.item_count,
            Some(1),
            "item count for {}",
            spec.component_path
        );
        assert_eq!(
            observation.item_registry_id,
            Some(spec.item_registry_id),
            "item registry id for {}",
            spec.component_path
        );
        assert_eq!(
            observation.positive_component_type_ids,
            vec![spec.component_type_id],
            "component type for {}",
            spec.component_path
        );
        assert!(observation.negative_component_type_ids.is_empty());
        assert!(
            input.is_empty(),
            "trailing bytes for {}",
            spec.component_path
        );
    }
}

#[test]
fn inventory_partial_changed_slot_prediction_encoder_omits_slot_hash() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_partial_changed_slot_prediction(&mut payload, 2, 8, 1)
        .expect("partial changed-slot prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        2
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input[0], 1, "carried should be non-empty");
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item id"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        8
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("component map count"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("removed component count"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn inventory_quick_move_encoder_keeps_quick_move_input_id() {
    let mut payload = Vec::new();
    encode_inventory_quick_move_state_correction(&mut payload, 0)
        .expect("inventory quick move packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[3..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_swap_encoder_keeps_target_slot_and_swap_input_id() {
    let mut payload = Vec::new();
    encode_inventory_swap_state_correction(&mut payload, 0).expect("inventory swap packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 9);
    input = &input[3..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        2
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn side_use_item_on_encoder_keeps_click_position_and_face() {
    let mut payload = Vec::new();
    encode_use_item_on_hit(&mut payload, (2, 70, -4), 2, (0.5, 0.5, 0.0), 5)
        .expect("side use item on packet");

    let mut input = payload.as_slice();
    assert_eq!(voxel_core::wire::read_varint(&mut input).expect("hand"), 0);
    input = &input[8..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("north direction"),
        2
    );
    assert_eq!(&input[0..4], &0.5_f32.to_be_bytes());
    assert_eq!(&input[4..8], &0.5_f32.to_be_bytes());
    assert_eq!(&input[8..12], &0.0_f32.to_be_bytes());
    assert_eq!(input[12], 0);
    assert_eq!(input[13], 0);
    input = &input[14..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        5
    );
    assert!(input.is_empty());
}

#[test]
fn use_item_encoder_matches_java_field_order() {
    let mut payload = Vec::new();
    encode_use_item(&mut payload, 10, 30.0, -15.0).expect("use item packet");

    let mut input = payload.as_slice();
    assert_eq!(voxel_core::wire::read_varint(&mut input).expect("hand"), 0);
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        10
    );
    assert_eq!(&input[0..4], &30.0_f32.to_be_bytes());
    assert_eq!(&input[4..8], &(-15.0_f32).to_be_bytes());
    assert!(input[8..].is_empty());
}

#[test]
fn chunk_packet_parser_rejects_trailing_payload() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&0_i32.to_be_bytes());
    payload.extend_from_slice(&0_i32.to_be_bytes());
    for _ in 0..9 {
        voxel_core::wire::write_varint(0, &mut payload);
    }
    payload.push(0xff);

    match parse_chunk_packet(&payload) {
        Err(super::ConnectionError::Codec(name)) => {
            assert_eq!(name, "chunk trailing payload")
        }
        result => panic!("unexpected chunk parser result: {result:?}"),
    }
}

#[test]
fn section_parser_matches_java_palette_storage_layout() {
    let mut section = Vec::new();
    section.extend_from_slice(&7_i16.to_be_bytes());
    section.extend_from_slice(&2_i16.to_be_bytes());

    section.push(4);
    voxel_core::wire::write_varint(2, &mut section);
    voxel_core::wire::write_varint(0, &mut section);
    voxel_core::wire::write_varint(3, &mut section);
    section.extend_from_slice(&0x101_u64.to_be_bytes());
    section.resize(section.len() + 255 * 8, 0);

    section.push(3);
    voxel_core::wire::write_varint(2, &mut section);
    voxel_core::wire::write_varint(0, &mut section);
    voxel_core::wire::write_varint(1, &mut section);
    section.extend_from_slice(&0x249_u64.to_be_bytes());
    section.resize(section.len() + 3 * 8, 0);

    let observations = parse_section_data(&section).expect("serialized section");
    assert_eq!(observations.len(), 1);
    let observation = &observations[0];
    assert_eq!(observation.non_empty_block_count, 7);
    assert_eq!(observation.fluid_count, 2);
    assert_eq!(observation.block_state_bits_per_entry, 4);
    assert_eq!(observation.block_state_palette_entry_count, 2);
    assert_eq!(observation.block_state_palette_ids, [0, 3]);
    assert_eq!(observation.block_state_storage_word_count, 256);
    assert_eq!(observation.block_state_first_values, [3, 0, 3, 0]);
    assert_eq!(observation.block_state_values.len(), 4_096);
    assert_eq!(&observation.block_state_values[..4], [3, 0, 3, 0]);
    assert_eq!(observation.block_state_distinct_value_count, 2);
    assert_eq!(observation.biome_bits_per_entry, 3);
    assert_eq!(observation.biome_palette_entry_count, 2);
    assert_eq!(observation.biome_palette_ids, [0, 1]);
    assert_eq!(observation.biome_storage_word_count, 4);
    assert_eq!(observation.biome_first_values, [1, 1, 1, 1]);
    assert_eq!(observation.biome_values.len(), 64);
    assert_eq!(&observation.biome_values[..4], [1, 1, 1, 1]);
    assert_eq!(observation.biome_distinct_value_count, 2);
}

#[test]
fn section_parser_reads_zero_bit_palettes_without_storage() {
    let mut section = Vec::new();
    section.extend_from_slice(&4096_i16.to_be_bytes());
    section.extend_from_slice(&0_i16.to_be_bytes());
    section.push(0);
    voxel_core::wire::write_varint(0, &mut section);
    section.push(0);
    voxel_core::wire::write_varint(0, &mut section);

    let observations = parse_section_data(&section).expect("single-valued section");
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].block_state_bits_per_entry, 0);
    assert_eq!(observations[0].block_state_palette_entry_count, 1);
    assert_eq!(observations[0].block_state_palette_ids, [0]);
    assert_eq!(observations[0].block_state_storage_word_count, 0);
    assert_eq!(observations[0].block_state_first_values, [0, 0, 0, 0]);
    assert_eq!(observations[0].block_state_values, vec![0; 4_096]);
    assert_eq!(observations[0].block_state_distinct_value_count, 1);
    assert_eq!(observations[0].biome_bits_per_entry, 0);
    assert_eq!(observations[0].biome_palette_entry_count, 1);
    assert_eq!(observations[0].biome_palette_ids, [0]);
    assert_eq!(observations[0].biome_storage_word_count, 0);
    assert_eq!(observations[0].biome_first_values, [0, 0, 0, 0]);
    assert_eq!(observations[0].biome_values, vec![0; 64]);
    assert_eq!(observations[0].biome_distinct_value_count, 1);
}

#[test]
fn recipe_book_add_reads_a_shaped_entry_and_its_flags() {
    // One entry: id 7, a 2x1 shaped display (an item, an empty slot), the
    // result item 20, an empty station, no group, category 3, one
    // requirement of item 10, the highlight flag, replacing the book.
    let payload = [
        1, 7, 1, 2, 1, 2, 4, 10, 0, 4, 20, 0, 0, 3, 1, 1, 2, 10, 2, 1,
    ];
    let (entries, replace) = super::parse_recipe_book_add_packet(&payload).unwrap();
    assert!(replace);
    let (entry, flags) = &entries[0];
    assert_eq!(
        (entry.id, entry.group, entry.category, *flags),
        (7, None, 3, 2)
    );
    assert_eq!(
        entry.display,
        voxel_world::RecipeDisplay::Shaped {
            width: 2,
            height: 1,
            ingredients: vec![
                voxel_world::SlotDisplay::Item(10),
                voxel_world::SlotDisplay::Empty
            ],
            result: voxel_world::SlotDisplay::Item(20),
        }
    );
    assert_eq!(
        entry.requirements,
        Some(vec![voxel_world::Ingredient::Items(vec![10])])
    );
    let mut book = voxel_world::RecipeBook::default();
    book.add(entries, replace);
    assert!(book.highlight.contains(&7));
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
    let packet = super::parse_player_chat_packet(&payload).unwrap();
    assert_eq!(packet.global_index, 0);
    assert_eq!(packet.signed_text, "hi");
    assert_eq!(packet.signature, None);
    assert_eq!(packet.filter, super::FilterMask::PassThrough);
    assert_eq!(packet.bound.holder, super::ChatHolder::Registry(0));
    assert_eq!(
        packet.bound.name,
        voxel_game::text::Component::literal("Bob")
    );
}

#[test]
fn the_signature_cache_pushes_new_entries_to_the_front() {
    let sig = |byte: u8| vec![byte; 256];
    let mut cache = super::SignatureCache::default();
    cache.push([sig(1), sig(2)].into());
    // The newest is at place 0, the one before it at 1.
    assert_eq!(
        cache.unpack(&super::PackedSignature::Cached(0)),
        Some(sig(2))
    );
    assert_eq!(
        cache.unpack(&super::PackedSignature::Cached(1)),
        Some(sig(1))
    );
    // One more shifts the old ones back; one it repeats is not kept twice.
    cache.push([sig(2), sig(3)].into());
    assert_eq!(
        cache.unpack(&super::PackedSignature::Cached(0)),
        Some(sig(3))
    );
    assert_eq!(
        cache.unpack(&super::PackedSignature::Cached(1)),
        Some(sig(2))
    );
    assert_eq!(
        cache.unpack(&super::PackedSignature::Cached(2)),
        Some(sig(1))
    );
    assert_eq!(cache.unpack(&super::PackedSignature::Cached(3)), None);
}

#[test]
fn a_partial_filter_mask_hides_its_set_bits() {
    let mut payload = vec![0];
    payload.extend([0; 16]);
    payload.extend([0, 0, 2, b'h', b'i']);
    payload.extend([0; 16]);
    payload.extend([0, 0, 2, 1]);
    payload.extend([0, 0, 0, 0, 0, 0, 0, 0b10]); // the second character
    payload.push(1);
    payload.extend([8, 0, 3, b'B', b'o', b'b', 0]);
    let packet = super::parse_player_chat_packet(&payload).unwrap();
    assert!(!packet.filter.hides(0) && packet.filter.hides(1));
}

#[test]
fn an_unsigned_chat_packet_ends_in_an_empty_acknowledgement() {
    let mut output = Vec::new();
    super::encode_chat("hi", 5, 7, &mut output);
    assert_eq!(output[..3], [2, b'h', b'i']);
    // offset 0, three bytes of bits, checksum 1, after the signature flag.
    assert_eq!(output[3 + 16..], [0, 0, 0, 0, 0, 1]);
}

#[test]
fn player_info_update_reads_its_actions_in_order() {
    // Add, update listed, update latency for one player.
    let mut payload = vec![1 | 8 | 16, 1];
    payload.extend([0; 15]);
    payload.push(9); // uuid
    payload.extend([2, b'A', b'l', 0]); // name, no properties
    payload.extend([1, 50]); // listed, latency
    let entries = super::parse_player_info_update(&payload).unwrap();
    assert_eq!(entries[0].0, (0, 9));
    assert_eq!(
        entries[0].1,
        vec![
            super::InfoAction::Add { name: "Al".into() },
            super::InfoAction::Listed(true),
            super::InfoAction::Latency(50),
        ]
    );
}

#[test]
fn title_packets_read_their_fields() {
    let animation = [0, 0, 0, 10, 0, 0, 0, 70, 0, 0, 0, 20];
    assert_eq!(
        super::parse_hud_packet(115, &animation).unwrap(),
        Some(super::HudEvent::Times {
            fade_in: 10,
            stay: 70,
            fade_out: 20
        })
    );
    assert_eq!(
        super::parse_hud_packet(14, &[1]).unwrap(),
        Some(super::HudEvent::Clear { reset_times: true })
    );
    let title = [8, 0, 2, b'h', b'i'];
    assert_eq!(
        super::parse_hud_packet(114, &title).unwrap(),
        Some(super::HudEvent::Title(
            voxel_game::text::Component::literal("hi")
        ))
    );
}

#[test]
fn boss_event_packets_read_each_operation() {
    let mut add = 7u128.to_be_bytes().to_vec();
    add.extend([0, 8, 0, 2, b'h', b'i']);
    add.extend(0.5f32.to_be_bytes());
    add.extend([3, 2, 0b101]);
    assert_eq!(
        super::parse_hud_packet(9, &add).unwrap(),
        Some(super::HudEvent::Boss {
            id: 7,
            operation: super::BossOperation::Add {
                name: voxel_game::text::Component::literal("hi"),
                progress: 0.5,
                color: 3,
                overlay: 2,
                flags: 0b101,
            },
        })
    );
    let mut remove = 7u128.to_be_bytes().to_vec();
    remove.push(1);
    assert_eq!(
        super::parse_hud_packet(9, &remove).unwrap(),
        Some(super::HudEvent::Boss {
            id: 7,
            operation: super::BossOperation::Remove,
        })
    );
    let mut style = 7u128.to_be_bytes().to_vec();
    style.extend([4, 7, 0]);
    assert!(super::parse_hud_packet(9, &style).is_err());
}

#[test]
fn scoreboard_packets_read_their_fields() {
    use voxel_game::scoreboard::{NumberFormat, ScoreboardEvent};
    use voxel_game::text::Component;
    // Objective "o": add, display name "hi", hearts, a blank number format.
    let mut objective = vec![1, b'o', 0, 8, 0, 2, b'h', b'i', 1, 1, 0];
    assert_eq!(
        super::parse_scoreboard_packet(106, &objective).unwrap(),
        Some(ScoreboardEvent::Objective {
            name: "o".to_owned(),
            method: 0,
            display_name: Component::literal("hi"),
            hearts: true,
            number_format: Some(NumberFormat::Blank),
        })
    );
    // A removal carries nothing more.
    objective.truncate(3);
    objective[2] = 1;
    assert!(matches!(
        super::parse_scoreboard_packet(106, &objective).unwrap(),
        Some(ScoreboardEvent::Objective { method: 1, .. })
    ));
    // Score of "al" on "o": 5, no display, a fixed format.
    let score = [2, b'a', b'l', 1, b'o', 5, 0, 1, 2, 8, 0, 1, b'x'];
    assert_eq!(
        super::parse_scoreboard_packet(110, &score).unwrap(),
        Some(ScoreboardEvent::Score {
            owner: "al".to_owned(),
            objective: "o".to_owned(),
            value: 5,
            display: None,
            number_format: Some(NumberFormat::Fixed(Component::literal("x"))),
        })
    );
    // Reset of every score of "al".
    assert_eq!(
        super::parse_scoreboard_packet(79, &[2, b'a', b'l', 0]).unwrap(),
        Some(ScoreboardEvent::ResetScore {
            owner: "al".to_owned(),
            objective: None,
        })
    );
    // Slot 1 shows "o"; an empty name clears the slot.
    assert_eq!(
        super::parse_scoreboard_packet(98, &[1, 1, b'o']).unwrap(),
        Some(ScoreboardEvent::Display {
            slot: 1,
            objective: Some("o".to_owned()),
        })
    );
    assert_eq!(
        super::parse_scoreboard_packet(98, &[1, 0]).unwrap(),
        Some(ScoreboardEvent::Display {
            slot: 1,
            objective: None,
        })
    );
    // Team "t": add, three components, two rules, colour 12, options, one player.
    let mut team = vec![1, b't', 0];
    for text in ["t", "[", "]"] {
        team.extend([8, 0, 1, text.as_bytes()[0]]);
    }
    team.extend([0, 0, 1, 12, 3, 1, 2, b'a', b'l']);
    let Some(ScoreboardEvent::Team {
        parameters: Some(parameters),
        players,
        ..
    }) = super::parse_scoreboard_packet(109, &team).unwrap()
    else {
        panic!("a team with parameters");
    };
    assert_eq!(parameters.colour, Some(12));
    assert_eq!(players, ["al"]);
}

#[test]
fn a_commands_packet_links_into_a_tree() {
    use voxel_game::commands::NodeKind;
    // Node 0 is the root with children 1; node 1 literal "a" (executable)
    // with child 2; node 2 an integer argument "n" from 1 to 5.
    let mut packet = vec![3];
    packet.extend([0, 1, 1]);
    packet.extend([1 | 4, 1, 2, 1, b'a']);
    packet.extend([2 | 4, 0, 1, b'n', 3, 0b11]);
    packet.extend(1i32.to_be_bytes());
    packet.extend(5i32.to_be_bytes());
    packet.push(0);
    let tree = super::parse_commands_packet(&packet).unwrap();
    let root = tree.node(tree.root());
    assert_eq!(root.children.len(), 1);
    assert_eq!(
        tree.node(root.children[0]).kind,
        NodeKind::Literal("a".to_owned())
    );
    let parse = tree.parse("a 3", 0);
    assert!(!parse.can_read() && parse.exceptions.is_empty());
    let parse = tree.parse("a 9", 0);
    assert_eq!(parse.exceptions.len(), 1);
    // A child past the end is refused.
    assert!(super::parse_commands_packet(&[1, 0, 1, 7, 0]).is_err());
}

#[test]
fn combat_kill_names_who_died_and_the_message() {
    let packet = [0xE8, 0x07, 8, 0, 2, b'h', b'i'];
    let (id, message) = super::parse_player_combat_kill_packet(&packet).unwrap();
    assert_eq!(id, 1000);
    assert_eq!(message, voxel_game::text::Component::literal("hi"));
}

#[test]
fn a_disconnect_carries_its_reason() {
    let mut input: &[u8] = &[8, 0, 2, b'h', b'i'];
    assert_eq!(
        super::read_disconnect_reason(&mut input).unwrap(),
        voxel_game::text::Component::literal("hi")
    );
}
