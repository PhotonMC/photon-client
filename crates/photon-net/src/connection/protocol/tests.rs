//! Wire-level protocol tests for the play/status packet codecs.

use super::{
    GAME_EVENT_CHANGE_GAME_MODE_ID, InventoryComponentCodec, PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID,
    PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID, PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID,
    PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID, PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
    PLAY_CLIENTBOUND_CHUNK_PACKET_ID, PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID,
    PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
    PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID, PLAY_CLIENTBOUND_COOLDOWN_PACKET_ID,
    PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID, PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID,
    PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID, PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
    PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID, PLAY_CLIENTBOUND_LOGIN_PACKET_ID,
    PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID, PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID,
    PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID, PLAY_CLIENTBOUND_MOVE_MINECART_PACKET_ID,
    PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID, PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID,
    PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID, PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
    PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID, PLAY_CLIENTBOUND_RESPAWN_PACKET_ID,
    PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID, PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
    PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID, PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
    PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID, PLAY_CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID,
    PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID, PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID,
    PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID, PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID,
    PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID,
    PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID, PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID,
    PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID,
    PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
    PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID, PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
    PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID, PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID, PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID,
    PLAY_SERVERBOUND_SELECT_BUNDLE_ITEM_PACKET_ID, PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
    PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID, PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID, STATUS_CLIENTBOUND_PONG_RESPONSE_PACKET_ID,
    STATUS_CLIENTBOUND_RESPONSE_PACKET_ID, STATUS_SERVERBOUND_PING_REQUEST_PACKET_ID,
    STATUS_SERVERBOUND_REQUEST_PACKET_ID, VANILLA_ITEM_COMPONENT_CONFORMANCE, decode_packet_frame,
    encode_abort_destroy_block, encode_container_slot_state, encode_creative_item_slot,
    encode_creative_slot, encode_creative_stone_slot, encode_inventory_click_with_input,
    encode_inventory_drag_empty_cancel_state_correction, encode_inventory_pickup_state_correction,
    encode_inventory_pickup_with_component_hash_prediction,
    encode_inventory_pickup_with_component_spec_prediction,
    encode_inventory_pickup_with_partial_changed_slot_prediction,
    encode_inventory_pickup_with_prediction, encode_inventory_pickup_with_wrong_both_prediction,
    encode_inventory_pickup_with_wrong_carried_prediction,
    encode_inventory_pickup_with_wrong_slot_prediction,
    encode_inventory_quick_move_state_correction,
    encode_inventory_secondary_pickup_state_correction, encode_inventory_swap_state_correction,
    encode_inventory_throw_all_state_correction, encode_inventory_throw_one_state_correction,
    encode_packet_frame, encode_perform_respawn, encode_release_use_item, encode_select_trade,
    encode_set_beacon, encode_set_carried_item, encode_stop_destroy_block, encode_use_item,
    encode_use_item_on, encode_use_item_on_hit, pack_block_position, parse_add_entity_packet,
    parse_block_destruction_packet, parse_block_entity_data_packet, parse_block_event_packet,
    parse_block_update_packet, parse_chunk_packet, parse_container_close_packet,
    parse_container_set_content_packet, parse_container_set_slot_packet, parse_cursor_item_packet,
    parse_damage_event_packet, parse_entity_equipment_packet, parse_entity_event_packet,
    parse_entity_metadata_packet, parse_entity_position_sync_packet, parse_entity_teleport_packet,
    parse_forget_level_chunk_packet, parse_game_event_packet, parse_hurt_animation_packet,
    parse_initialize_border_packet, parse_light_update_packet, parse_login_packet,
    parse_map_item_data_packet, parse_merchant_offers_packet, parse_move_entity_pos_packet,
    parse_move_entity_pos_rot_packet, parse_move_entity_rot_packet, parse_move_minecart_packet,
    parse_open_screen_packet, parse_optional_item_stack, parse_player_abilities_packet,
    parse_player_position_packet, parse_remove_entities_packet, parse_remove_mob_effect_packet,
    parse_respawn_packet, parse_rotate_head_packet, parse_section_data,
    parse_set_border_lerp_size_packet, parse_set_chunk_cache_center_packet,
    parse_set_chunk_cache_radius_packet, parse_set_entity_motion_packet, parse_set_health_packet,
    parse_set_passengers_packet, parse_set_player_inventory_packet, parse_system_chat_packet,
    parse_take_item_entity_packet, parse_update_attributes_packet, parse_update_mob_effect_packet,
    resolve_player_position, support_block_position,
};

fn write_component_fixture(codec: InventoryComponentCodec, output: &mut Vec<u8>) {
    match codec {
        InventoryComponentCodec::VarInt => photon_core::wire::write_varint(0, output),
        // A holder writes its registry id plus one; a zero is a direct value.
        InventoryComponentCodec::RegistryId => photon_core::wire::write_varint(1, output),
        InventoryComponentCodec::Unit => {}
        InventoryComponentCodec::Float => output.extend_from_slice(&0.0_f32.to_be_bytes()),
        InventoryComponentCodec::Byte => output.push(0),
        InventoryComponentCodec::Int => output.extend_from_slice(&0_i32.to_be_bytes()),
        InventoryComponentCodec::Identifier => {
            let identifier = photon_core::identifier::Identifier::parse("minecraft:test")
                .expect("identifier fixture");
            photon_core::identifier::write_identifier(&identifier, output)
                .expect("identifier fixture encoding");
        }
        InventoryComponentCodec::UseEffects => {
            output.push(1);
            output.push(0);
            output.extend_from_slice(&1.0_f32.to_be_bytes());
        }
        InventoryComponentCodec::CustomModelData => {
            photon_core::wire::write_varint(1, output);
            output.extend_from_slice(&1.0_f32.to_be_bytes());
            for _ in 0..3 {
                photon_core::wire::write_varint(0, output);
            }
        }
        InventoryComponentCodec::Food => {
            photon_core::wire::write_varint(1, output);
            output.extend_from_slice(&0.1_f32.to_be_bytes());
            output.push(1);
        }
        InventoryComponentCodec::Tool => {
            photon_core::wire::write_varint(0, output);
            output.extend_from_slice(&1.0_f32.to_be_bytes());
            photon_core::wire::write_varint(0, output);
            output.push(1);
        }
        InventoryComponentCodec::Weapon => {
            photon_core::wire::write_varint(1, output);
            output.extend_from_slice(&0.0_f32.to_be_bytes());
        }
    }
}

mod block;
mod chat;
mod chunk;
mod commands;
mod components;
mod container;
mod hud;
mod ids;
mod inventory;
mod map;
mod movement;
mod particles;
mod player;
mod recipe;
mod serverbound;
mod spawn;
mod state;
