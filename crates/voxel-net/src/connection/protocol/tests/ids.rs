use super::*;

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
    assert_eq!(PLAY_CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID, 100);
    assert_eq!(PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID, 101);
    assert_eq!(PLAY_CLIENTBOUND_MOVE_MINECART_PACKET_ID, 55);
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
