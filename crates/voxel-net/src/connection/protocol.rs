//! Packet codecs, wire observations, and frame serialization for the Java protocol.

use std::collections::BTreeSet;
use std::fmt;
use std::io::{self, Read, Write};
use std::time::Duration;

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::frame::{self, FrameError, MAX_FRAME_LENGTH};
use crate::protocol::ConnectionProtocol;
use voxel_core::binary::{BitSet, Uuid};
use voxel_core::wire;
use voxel_world::minecart_track::MinecartStep;
use voxel_world::{
    ClientAttributeModifierState, ClientAttributeState, ClientItemStackState, ClientPlayerGameMode,
    ClientWorld, DimensionBounds, DimensionEnvironment,
};

pub(crate) const PROTOCOL_VERSION: i32 = 776;
pub(crate) const LOGIN_INTENTION_PACKET_ID: i32 = 0;
pub(crate) const LOGIN_HELLO_PACKET_ID: i32 = 0;
pub(crate) const LOGIN_ACKNOWLEDGED_PACKET_ID: i32 = 3;
pub(crate) const LOGIN_FINISHED_PACKET_ID: i32 = 2;
pub(crate) const LOGIN_COMPRESSION_PACKET_ID: i32 = 3;

pub(crate) const STATUS_SERVERBOUND_REQUEST_PACKET_ID: i32 = 0;
pub(crate) const STATUS_SERVERBOUND_PING_REQUEST_PACKET_ID: i32 = 1;
pub(crate) const STATUS_CLIENTBOUND_RESPONSE_PACKET_ID: i32 = 0;
pub(crate) const STATUS_CLIENTBOUND_PONG_RESPONSE_PACKET_ID: i32 = 1;
pub(crate) const STATUS_RESPONSE_MAX_UTF16_LENGTH: usize = 32_767;
pub(crate) const STATUS_PROBE_TIME: i64 = 0x0102_0304_0506_0708;

pub(crate) const CONFIG_CLIENT_INFORMATION_PACKET_ID: i32 = 0;
pub(crate) const CONFIG_CUSTOM_PAYLOAD_PACKET_ID: i32 = 2;
pub(crate) const CONFIG_FINISH_PACKET_ID: i32 = 3;
pub(crate) const CONFIG_KEEP_ALIVE_PACKET_ID: i32 = 4;
pub(crate) const CONFIG_PONG_PACKET_ID: i32 = 5;
pub(crate) const CONFIG_REGISTRY_DATA_PACKET_ID: i32 = 7;
pub(crate) const CONFIG_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 14;
pub(crate) const CONFIG_SERVERBOUND_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 7;
pub(crate) const CONFIG_ACCEPT_CODE_OF_CONDUCT_PACKET_ID: i32 = 9;
pub(crate) const CONFIG_ENABLED_FEATURES_PACKET_ID: i32 = 12;
pub(crate) const CONFIG_UPDATE_TAGS_PACKET_ID: i32 = 13;
pub(crate) const CONFIG_CODE_OF_CONDUCT_PACKET_ID: i32 = 19;

// The virtual bundle packet is not registered in the Java dispatch table; the delimiter
// occupies id 0 and shifts the listed play clientbound packet ids by one.
pub(crate) const PLAY_CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID: i32 = 0;
pub(crate) const PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID: i32 = 1;
pub(crate) const PLAY_CLIENTBOUND_ANIMATE_PACKET_ID: i32 = 2;
pub(crate) const PLAY_CLIENTBOUND_DAMAGE_EVENT_PACKET_ID: i32 = 25;
pub(crate) const PLAY_CLIENTBOUND_ENTITY_EVENT_PACKET_ID: i32 = 34;
pub(crate) const PLAY_CLIENTBOUND_HURT_ANIMATION_PACKET_ID: i32 = 42;
pub(crate) const PLAY_CLIENTBOUND_SET_SIMULATION_DISTANCE_PACKET_ID: i32 = 111;
pub(crate) const PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID: i32 = 35;
pub(crate) const PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID: i32 = 53;
pub(crate) const PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID: i32 = 54;
pub(crate) const PLAY_CLIENTBOUND_MOVE_MINECART_PACKET_ID: i32 = 55;
pub(crate) const PLAY_CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID: i32 = 51;
pub(crate) const PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID: i32 = 56;
pub(crate) const PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID: i32 = 83;
pub(crate) const PLAY_CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID: i32 = 100;
pub(crate) const PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID: i32 = 101;
pub(crate) const PLAY_CLIENTBOUND_SET_EQUIPMENT_PACKET_ID: i32 = 102;
pub(crate) const PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID: i32 = 107;
pub(crate) const PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID: i32 = 4;
pub(crate) const PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID: i32 = 5;
pub(crate) const PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID: i32 = 6;
pub(crate) const PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID: i32 = 8;
pub(crate) const PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID: i32 = 37;
pub(crate) const PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID: i32 = 38;
pub(crate) const PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID: i32 = 48;
pub(crate) const PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID: i32 = 94;
pub(crate) const PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID: i32 = 95;
pub(crate) const PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID: i32 = 11;
pub(crate) const PLAY_CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID: i32 = 12;
pub(crate) const PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID: i32 = 17;
pub(crate) const PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID: i32 = 18;
pub(crate) const PLAY_CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID: i32 = 19;
pub(crate) const PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID: i32 = 20;
pub(crate) const PLAY_CLIENTBOUND_UPDATE_RECIPES_PACKET_ID: i32 = 133;
pub(crate) const PLAY_CLIENTBOUND_PLACE_GHOST_RECIPE_PACKET_ID: i32 = 63;
pub(crate) const PLAY_CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID: i32 = 74;
pub(crate) const PLAY_CLIENTBOUND_RECIPE_BOOK_REMOVE_PACKET_ID: i32 = 75;
pub(crate) const PLAY_CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID: i32 = 76;
pub(crate) const PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID: i32 = 59;
pub(crate) const PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID: i32 = 64;
pub(crate) const PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID: i32 = 96;
pub(crate) const PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID: i32 = 108;
pub(crate) const PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID: i32 = 77;
pub(crate) const PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID: i32 = 78;
pub(crate) const PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID: i32 = 99;
pub(crate) const PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID: i32 = 125;
pub(crate) const PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID: i32 = 121;
pub(crate) const PLAY_CLIENTBOUND_CLEAR_TITLES_PACKET_ID: i32 = 14;
pub(crate) const PLAY_CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID: i32 = 15;
pub(crate) const PLAY_SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID: i32 = 15;
pub(crate) const PLAY_CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID: i32 = 69;
pub(crate) const PLAY_CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID: i32 = 70;
pub(crate) const PLAY_CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID: i32 = 87;
pub(crate) const PLAY_CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID: i32 = 112;
pub(crate) const PLAY_CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID: i32 = 114;
pub(crate) const PLAY_CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID: i32 = 115;
pub(crate) const PLAY_CLIENTBOUND_TAB_LIST_PACKET_ID: i32 = 122;
pub(crate) const PLAY_CLIENTBOUND_BOSS_EVENT_PACKET_ID: i32 = 9;
pub(crate) const PLAY_CLIENTBOUND_WAYPOINT_PACKET_ID: i32 = 138;
pub(crate) const PLAY_CLIENTBOUND_COMMANDS_PACKET_ID: i32 = 16;
pub(crate) const PLAY_CLIENTBOUND_RESET_SCORE_PACKET_ID: i32 = 79;
pub(crate) const PLAY_CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID: i32 = 98;
pub(crate) const PLAY_CLIENTBOUND_SET_OBJECTIVE_PACKET_ID: i32 = 106;
pub(crate) const PLAY_CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID: i32 = 109;
pub(crate) const PLAY_CLIENTBOUND_SET_SCORE_PACKET_ID: i32 = 110;
pub(crate) const PLAY_CLIENTBOUND_DELETE_CHAT_PACKET_ID: i32 = 31;
pub(crate) const PLAY_CLIENTBOUND_DISGUISED_CHAT_PACKET_ID: i32 = 33;
pub(crate) const PLAY_CLIENTBOUND_PLAYER_CHAT_PACKET_ID: i32 = 65;
pub(crate) const PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID: i32 = 44;
pub(crate) const PLAY_CLIENTBOUND_CHUNK_PACKET_ID: i32 = 45;
pub(crate) const PLAY_CLIENTBOUND_LOGIN_PACKET_ID: i32 = 49;
pub(crate) const PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID: i32 = 72;
pub(crate) const PLAY_CLIENTBOUND_RESPAWN_PACKET_ID: i32 = 82;
pub(crate) const PLAY_CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID: i32 = 68;
pub(crate) const PLAY_CLIENTBOUND_LEVEL_EVENT_PACKET_ID: i32 = 46;
pub(crate) const PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID: i32 = 116;
pub(crate) const PLAY_CLIENTBOUND_SOUND_PACKET_ID: i32 = 117;
pub(crate) const PLAY_CLIENTBOUND_STOP_SOUND_PACKET_ID: i32 = 119;
/// `ClientboundGameEventPacket.IMMEDIATE_RESPAWN`.
pub(crate) const GAME_EVENT_IMMEDIATE_RESPAWN_ID: u8 = 11;
pub(crate) const PLAY_CLIENTBOUND_SET_EXPERIENCE_PACKET_ID: i32 = 103;
pub(crate) const PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID: i32 = 104;
pub(crate) const PLAY_CLIENTBOUND_SET_HELD_SLOT_PACKET_ID: i32 = 105;
pub(crate) const PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID: i32 = 131;
pub(crate) const PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID: i32 = 132;
pub(crate) const PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID: i32 = 0;
pub(crate) const PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID: i32 = 41;
pub(crate) const PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID: i32 = 11;
pub(crate) const PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID: i32 = 12;
pub(crate) const PLAY_SERVERBOUND_CHAT_COMMAND_PACKET_ID: i32 = 7;
pub(crate) const PLAY_SERVERBOUND_CHAT_PACKET_ID: i32 = 9;
pub(crate) const PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID: i32 = 13;
pub(crate) const PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID: i32 = 28;
pub(crate) const PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID: i32 = 31;
pub(crate) const PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID: i32 = 44;
pub(crate) const PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID: i32 = 53;
pub(crate) const PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID: i32 = 56;
pub(crate) const PLAY_SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID: i32 = 17;
pub(crate) const PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID: i32 = 18;
pub(crate) const PLAY_SERVERBOUND_RENAME_ITEM_PACKET_ID: i32 = 48;
pub(crate) const PLAY_SERVERBOUND_PLACE_RECIPE_PACKET_ID: i32 = 39;
pub(crate) const PLAY_SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID: i32 = 46;
pub(crate) const PLAY_SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID: i32 = 47;
pub(crate) const PLAY_SERVERBOUND_ATTACK_PACKET_ID: i32 = 1;
pub(crate) const PLAY_SERVERBOUND_INTERACT_PACKET_ID: i32 = 26;
pub(crate) const PLAY_SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID: i32 = 36;
pub(crate) const PLAY_SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID: i32 = 37;
pub(crate) const PLAY_SERVERBOUND_SPECTATOR_ACTION_PACKET_ID: i32 = 62;
pub(crate) const PLAY_SERVERBOUND_SWING_PACKET_ID: i32 = 63;
pub(crate) const PLAY_SERVERBOUND_CONTAINER_CLOSE_PACKET_ID: i32 = 19;
pub(crate) const PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID: i32 = 66;
pub(crate) const PLAY_SERVERBOUND_USE_ITEM_PACKET_ID: i32 = 67;
pub(crate) const GAME_EVENT_CHANGE_GAME_MODE_ID: u8 = 3;

// The pinned vanilla built-in item registry is not sent in configuration registry data.
// Items.java registers AIR before STONE, so STONE has id 1 in BuiltInRegistries.ITEM.
pub(crate) const VANILLA_STONE_ITEM_REGISTRY_ID: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InventoryComponentCodec {
    VarInt,
    RegistryId,
    Unit,
    Float,
    Byte,
    Int,
    Identifier,
    UseEffects,
    CustomModelData,
    Food,
    Tool,
    Weapon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InventoryComponentProbeSpec {
    pub(crate) component_path: &'static str,
    pub(crate) component_type_id: i32,
    pub(crate) codec: InventoryComponentCodec,
    pub(crate) component_hash: i32,
    pub(crate) item_registry_id: i32,
    pub(crate) expected_positive_component_type_ids: &'static [i32],
    pub(crate) carried_components: &'static [(i32, i32)],
}

pub(crate) const VANILLA_ITEM_COMPONENT_CONFORMANCE: &[InventoryComponentProbeSpec] = &[
    InventoryComponentProbeSpec {
        component_path: "custom_model_data",
        component_type_id: 17,
        codec: InventoryComponentCodec::CustomModelData,
        component_hash: 0x77a4_a24f,
        item_registry_id: 1_046,
        expected_positive_component_type_ids: &[17],
        carried_components: &[(17, 0x77a4_a24f)],
    },
    InventoryComponentProbeSpec {
        component_path: "food",
        component_type_id: 23,
        codec: InventoryComponentCodec::Food,
        component_hash: 0xfac6_9b4d_u32 as i32,
        item_registry_id: 1_089,
        expected_positive_component_type_ids: &[23],
        carried_components: &[(23, 0xfac6_9b4d_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "damage",
        component_type_id: 3,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[3],
        carried_components: &[(3, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "tool",
        component_type_id: 28,
        codec: InventoryComponentCodec::Tool,
        component_hash: 0x37de_0062,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[28],
        carried_components: &[(28, 0x37de_0062)],
    },
    InventoryComponentProbeSpec {
        component_path: "weapon",
        component_type_id: 29,
        codec: InventoryComponentCodec::Weapon,
        component_hash: 0xc574_b4c8_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[29],
        carried_components: &[(29, 0xc574_b4c8_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "max_stack_size",
        component_type_id: 1,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[1],
        carried_components: &[(1, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "max_damage",
        component_type_id: 2,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[1, 2],
        carried_components: &[(1, 0x5d50_d71c), (2, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "unbreakable",
        component_type_id: 4,
        codec: InventoryComponentCodec::Unit,
        component_hash: 0xc574_b4c8_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[4],
        carried_components: &[(4, 0xc574_b4c8_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "minimum_attack_charge",
        component_type_id: 7,
        codec: InventoryComponentCodec::Float,
        component_hash: 0x9ec7_d769_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[7],
        carried_components: &[(7, 0x9ec7_d769_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "repair_cost",
        component_type_id: 19,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[19],
        carried_components: &[(19, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "enchantment_glint_override",
        component_type_id: 21,
        codec: InventoryComponentCodec::Byte,
        component_hash: 0xc336_cec2_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[21],
        carried_components: &[(21, 0xc336_cec2_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "rarity",
        component_type_id: 12,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0xab53_df3a_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[12],
        carried_components: &[(12, 0xab53_df3a_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "item_model",
        component_type_id: 10,
        codec: InventoryComponentCodec::Identifier,
        component_hash: 0x75c9_3192_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[10],
        carried_components: &[(10, 0x75c9_3192_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "use_effects",
        component_type_id: 5,
        codec: InventoryComponentCodec::UseEffects,
        component_hash: 0x9619_e224_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[5],
        carried_components: &[(5, 0x9619_e224_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "damage_type",
        component_type_id: 8,
        codec: InventoryComponentCodec::RegistryId,
        component_hash: 0x55e3_3a50_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[8],
        carried_components: &[(8, 0x55e3_3a50_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "glider",
        component_type_id: 34,
        codec: InventoryComponentCodec::Unit,
        component_hash: 0xc574_b4c8_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[34],
        carried_components: &[(34, 0xc574_b4c8_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "tooltip_style",
        component_type_id: 35,
        codec: InventoryComponentCodec::Identifier,
        component_hash: 0xabab_f3db_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[35],
        carried_components: &[(35, 0xabab_f3db_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "enchantable",
        component_type_id: 31,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0xdeed_7aaa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[31],
        carried_components: &[(31, 0xdeed_7aaa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "dye",
        component_type_id: 43,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[43],
        carried_components: &[(43, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "dyed_color",
        component_type_id: 44,
        codec: InventoryComponentCodec::Int,
        component_hash: 0x23f6_5927_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[44],
        carried_components: &[(44, 0x23f6_5927_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "potion_duration_scale",
        component_type_id: 52,
        codec: InventoryComponentCodec::Float,
        component_hash: 0x9a13_eb94_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[52],
        carried_components: &[(52, 0x9a13_eb94_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "map_color",
        component_type_id: 45,
        codec: InventoryComponentCodec::Int,
        component_hash: 0x23f6_5927_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[45],
        carried_components: &[(45, 0x23f6_5927_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "map_id",
        component_type_id: 46,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[46],
        carried_components: &[(46, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "base_color",
        component_type_id: 73,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[73],
        carried_components: &[(73, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "note_block_sound",
        component_type_id: 71,
        codec: InventoryComponentCodec::Identifier,
        component_hash: 0xda54_5935_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[71],
        carried_components: &[(71, 0xda54_5935_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "ominous_bottle_amplifier",
        component_type_id: 63,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x3f72_5e25_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[63],
        carried_components: &[(63, 0x3f72_5e25_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "break_sound",
        component_type_id: 81,
        codec: InventoryComponentCodec::RegistryId,
        component_hash: 0x9662_f221_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[81],
        carried_components: &[(81, 0x9662_f221_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "wolf/collar",
        component_type_id: 85,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[85],
        carried_components: &[(85, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "tropical_fish/base_color",
        component_type_id: 90,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[90],
        carried_components: &[(90, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "tropical_fish/pattern",
        component_type_id: 89,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0xe196_874b_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[89],
        carried_components: &[(89, 0xe196_874b_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "tropical_fish/pattern_color",
        component_type_id: 91,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[91],
        carried_components: &[(91, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "cat/collar",
        component_type_id: 108,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[108],
        carried_components: &[(108, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "sheep/color",
        component_type_id: 109,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[109],
        carried_components: &[(109, 0x8c64_52aa_u32 as i32)],
    },
];

pub(crate) fn inventory_component_spec(
    component_path: &str,
) -> Option<&'static InventoryComponentProbeSpec> {
    let component_path = component_path
        .strip_prefix("minecraft:")
        .unwrap_or(component_path);
    VANILLA_ITEM_COMPONENT_CONFORMANCE
        .iter()
        .find(|spec| spec.component_path == component_path)
}

pub(crate) const VANILLA_MILK_BUCKET_ITEM_REGISTRY_ID: i32 = 1_046;
pub(crate) const VANILLA_SNOWBALL_ITEM_REGISTRY_ID: i32 = 1_044;
// DataComponents registers CUSTOM_MODEL_DATA after the first 17 built-in component types.

pub(crate) const PLAYER_RELATIVE_X_FLAG: i32 = 1 << 0;
pub(crate) const PLAYER_RELATIVE_Y_FLAG: i32 = 1 << 1;
pub(crate) const PLAYER_RELATIVE_Z_FLAG: i32 = 1 << 2;
pub(crate) const PLAYER_RELATIVE_Y_ROT_FLAG: i32 = 1 << 3;
pub(crate) const PLAYER_RELATIVE_X_ROT_FLAG: i32 = 1 << 4;

pub(crate) const MAX_PACKET_UNCOMPRESSED_LENGTH: usize = 8 * 1024 * 1024;
pub(crate) const MAX_CHUNK_DATA_LENGTH: usize = 2 * 1024 * 1024;
pub(crate) const MAX_KNOWN_PACKS: usize = 64;
pub(crate) const MAX_READABLE_FRAME_LENGTH: usize = MAX_FRAME_LENGTH;
pub(crate) const WORLD_READY_DRAIN_TIMEOUT: Duration = Duration::from_millis(250);
pub(crate) const DELAYED_DESTROY_DRAIN_TIMEOUT: Duration = Duration::from_millis(1_200);
pub(crate) const WATER_BUCKET_DRAIN_TIMEOUT: Duration = Duration::from_millis(1_000);
pub(crate) const CONSUMABLE_DRAIN_TIMEOUT: Duration = Duration::from_millis(2_500);
pub(crate) const COMPONENT_PREPARATION_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const WORLD_LIFECYCLE_DRAIN_TIMEOUT: Duration = Duration::from_secs(15);
pub(crate) const DIMENSION_RESPAWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const LIGHT_UPDATE_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const BLOCK_ENTITY_UPDATE_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const ENTITY_ADD_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const ENTITY_REMOVE_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const ENTITY_METADATA_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const ENTITY_POSITION_SYNC_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

/// Errors raised while driving the initial real-server client session.
#[derive(Debug)]
pub enum ConnectionError {
    Io(io::Error),
    Frame(FrameError),
    Wire(wire::ReadError),
    Compression(String),
    Codec(&'static str),
    UnsupportedPacket { phase: ConnectionProtocol, id: i32 },
    ServerDisconnected,
    InvalidArgument(&'static str),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Frame(error) => write!(formatter, "frame error: {error:?}"),
            Self::Wire(error) => write!(formatter, "wire error: {error:?}"),
            Self::Compression(error) => write!(formatter, "compression error: {error}"),
            Self::Codec(name) => write!(formatter, "packet codec failed: {name}"),
            Self::UnsupportedPacket { phase, id } => {
                write!(formatter, "unsupported packet id {id} in {phase:?}")
            }
            Self::ServerDisconnected => formatter.write_str("server disconnected"),
            Self::InvalidArgument(name) => write!(formatter, "invalid argument: {name}"),
        }
    }
}

impl std::error::Error for ConnectionError {}

impl From<io::Error> for ConnectionError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<FrameError> for ConnectionError {
    fn from(error: FrameError) -> Self {
        Self::Frame(error)
    }
}

impl From<wire::ReadError> for ConnectionError {
    fn from(error: wire::ReadError) -> Self {
        Self::Wire(error)
    }
}

/// One received packet after the outer frame and optional compression were removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketObservation {
    pub phase: ConnectionProtocol,
    pub id: i32,
    pub body_length: usize,
}

/// The observable result of the initial real-server session.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayConnectionTrace {
    pub packets: Vec<PacketObservation>,
    pub compression_threshold: Option<usize>,
    pub login_finished: bool,
    /// The profile id the server logged the player in with.
    pub profile_id: Option<voxel_core::binary::Uuid>,
    pub configuration_finished: bool,
    pub enabled_feature_count: usize,
    pub known_pack_count: usize,
    pub selected_known_pack_count: usize,
    pub registry_packet_count: usize,
    pub registry_entry_count: usize,
    /// Every synchronised registry's entry names, in the order the server
    /// numbered them.
    pub registry_entry_names: std::collections::BTreeMap<String, Vec<String>>,
    pub dimension_type_registry_bounds: Vec<Option<DimensionBounds>>,
    pub dimension_type_registry_environments: Vec<Option<DimensionEnvironment>>,
    pub update_tags_packet_count: usize,
    pub play_login: bool,
    pub login_observation: Option<LoginObservation>,
    pub respawn_observations: Vec<RespawnObservation>,
    pub attribute_update_observations: Vec<AttributeUpdateObservation>,
    pub health_observations: Vec<HealthObservation>,
    pub player_abilities_observations: Vec<PlayerAbilitiesObservation>,
    pub game_event_observations: Vec<GameEventObservation>,
    pub respawn_command_packets_sent: usize,
    pub player_position_observations: Vec<PlayerPositionObservation>,
    pub player_loaded_sent: bool,
    pub movement_packets_sent: usize,
    pub player_action_packets_sent: usize,
    pub use_item_on_packets_sent: usize,
    pub use_item_packets_sent: usize,
    pub creative_mode_slot_packets_sent: usize,
    pub carried_item_packets_sent: usize,
    pub container_click_packets_sent: usize,
    pub container_click_state_id_sent: Option<i32>,
    pub stone_item_registry_id: Option<i32>,
    pub snowball_item_registry_id: Option<i32>,
    pub add_entity_observations: Vec<AddEntityObservation>,
    pub world_entity_tick_observations: Vec<WorldEntityTickObservation>,
    pub experience_orb_tick_observations: Vec<ExperienceOrbTickObservation>,
    pub entity_interpolation_observations: Vec<EntityInterpolationObservation>,
    pub entity_position_snap_observations: Vec<EntityPositionSnapObservation>,
    pub entity_motion_observations: Vec<EntityMotionObservation>,
    pub entity_motion_interpolation_observations: Vec<EntityMotionInterpolationObservation>,
    pub entity_head_rotation_observations: Vec<EntityHeadRotationObservation>,
    pub remove_entities_observations: Vec<RemoveEntitiesObservation>,
    pub entity_passengers_observations: Vec<EntityPassengersObservation>,
    pub entity_passenger_tick_observations: Vec<EntityPassengerTickObservation>,
    pub entity_metadata_observations: Vec<EntityMetadataObservation>,
    pub entity_position_sync_observations: Vec<EntityPositionSyncObservation>,
    pub entity_teleport_observations: Vec<EntityTeleportObservation>,
    pub entity_move_observations: Vec<EntityMoveObservation>,
    pub open_screen_observations: Vec<OpenScreenObservation>,
    pub container_close_observations: Vec<ContainerCloseObservation>,
    pub container_content_observations: Vec<ContainerSetContentObservation>,
    pub container_slot_observations: Vec<ContainerSetSlotObservation>,
    pub cursor_item_observations: Vec<CursorItemObservation>,
    pub player_inventory_observations: Vec<SetPlayerInventoryObservation>,
    pub mob_effect_remove_observations: Vec<MobEffectRemoveObservation>,
    pub mob_effect_update_observations: Vec<MobEffectUpdateObservation>,
    pub system_chat_observations: Vec<SystemChatObservation>,
    pub creative_place_target_update: Option<BlockUpdateObservation>,
    pub creative_place_neighbor_update: Option<BlockUpdateObservation>,
    pub survival_water_bucket_target_update: Option<BlockUpdateObservation>,
    pub survival_water_bucket_neighbor_update: Option<BlockUpdateObservation>,
    pub block_destruction_observations: Vec<BlockDestructionObservation>,
    pub block_changed_ack_sequences: Vec<i32>,
    pub block_update_observations: Vec<BlockUpdateObservation>,
    pub chunk_unload_observations: Vec<ChunkUnloadObservation>,
    pub light_update_observations: Vec<LightUpdateObservation>,
    pub block_entity_observations: Vec<BlockEntityObservation>,
    pub chunk_packet_count: usize,
    pub chunk_batch_finished_count: usize,
    pub chunk_observations: Vec<ChunkPacketObservation>,
    pub world: ClientWorld,
}

/// Fields shared by the login and respawn spawn-info payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnInfoObservation {
    pub dimension_type_id: i32,
    pub dimension: String,
    pub seed: i64,
    pub game_type: u8,
    pub previous_game_type: Option<u8>,
    pub is_debug: bool,
    pub is_flat: bool,
    pub last_death_location: Option<GlobalPositionObservation>,
    pub portal_cooldown: i32,
    pub sea_level: i32,
}

/// The dimension and player-state fields carried by the initial login packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginObservation {
    pub player_id: i32,
    pub hardcore: bool,
    pub level_count: usize,
    pub max_players: i32,
    pub chunk_radius: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub do_limited_crafting: bool,
    pub spawn_info: SpawnInfoObservation,
    pub online_mode: bool,
    pub enforces_secure_chat: bool,
}

/// One clientbound respawn packet plus its client-world transition result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespawnObservation {
    pub spawn_info: SpawnInfoObservation,
    pub data_to_keep: u8,
    pub local_player_id: Option<i32>,
    pub local_player_recreated: bool,
    pub local_player_generation: Option<u64>,
    pub keep_entity_data: bool,
    pub keep_attribute_modifiers: bool,
    pub local_player_attribute_count: usize,
    pub local_player_attribute_modifier_count: usize,
    pub dimension_changed: bool,
    pub cleared_chunk_count: usize,
    pub cleared_entity_count: usize,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound health update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthObservation {
    pub health: f32,
    pub food: i32,
    pub saturation: f32,
    pub applied_to_world: bool,
}

/// Wire-level values decoded from one clientbound player-abilities packet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAbilitiesPacket {
    pub invulnerable: bool,
    pub flying: bool,
    pub can_fly: bool,
    pub can_instabuild: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

/// One clientbound player-abilities packet and its local-player application result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAbilitiesObservation {
    pub packet: PlayerAbilitiesPacket,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound game-event packet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameEventPacket {
    pub event_id: u8,
    pub param: f32,
}

/// One clientbound game-event packet and its local game-mode application result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameEventObservation {
    pub packet: GameEventPacket,
    pub applied_to_world: bool,
    pub local_game_mode: Option<ClientPlayerGameMode>,
}

/// Wire-level fields decoded from one clientbound attribute update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeUpdateObservation {
    pub entity_id: i32,
    pub attributes: Vec<ClientAttributeState>,
    pub applied_to_world: bool,
}

/// A serialized global position embedded in spawn info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPositionObservation {
    pub dimension: String,
    pub position: i64,
}

/// Wire-level fields observed from a clientbound player-position correction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerPositionObservation {
    pub teleport_id: i32,
    pub relative_flags: i32,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound open-screen packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenScreenObservation {
    pub container_id: i32,
    pub menu_type_id: i32,
    pub title_nbt: Vec<u8>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound container-close packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerCloseObservation {
    pub container_id: i32,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound block update packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockUpdateObservation {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub state_id: i32,
}

/// Wire-level fields observed from one clientbound chunk-unload packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkUnloadObservation {
    pub x: i32,
    pub z: i32,
    pub removed_from_world: bool,
    pub tracked_entity_count: usize,
    pub ticking_entity_count: usize,
    pub stopped_entity_count: usize,
}

/// Wire-level fields decoded from one clientbound light-update packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightUpdateObservation {
    pub x: i32,
    pub z: i32,
    pub sky_mask_word_count: usize,
    pub block_mask_word_count: usize,
    pub empty_sky_mask_word_count: usize,
    pub empty_block_mask_word_count: usize,
    pub sky_update_lengths: Vec<usize>,
    pub block_update_lengths: Vec<usize>,
    pub applied_sky_sections: usize,
    pub applied_block_sections: usize,
    pub queued_for_chunk: bool,
    pub(crate) light_data: LightUpdateData,
}

/// Wire-level fields decoded from one clientbound block-entity data record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityObservation {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub type_id: i32,
    pub tag_length: usize,
    pub applied_to_world: bool,
    pub(crate) tag: Option<Vec<u8>>,
}

/// Wire-level fields decoded from one clientbound block-destruction progress packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDestructionObservation {
    pub entity_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub progress: u8,
}

/// Wire-level fields decoded from one clientbound add-entity packet.
#[derive(Debug, Clone, PartialEq)]
pub struct AddEntityObservation {
    pub entity_id: i32,
    pub uuid: Uuid,
    pub entity_type_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub movement: (f64, f64, f64),
    pub x_rot: f32,
    pub y_rot: f32,
    pub y_head_rot: f32,
    pub data: i32,
    pub applied_to_world: bool,
    pub applied_rotation: Option<(f32, f32, f32)>,
}

/// One client-world tick applied to a supported world entity after its add packet.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldEntityTickObservation {
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_position: (f64, f64, f64),
    pub after_position: (f64, f64, f64),
    pub before_motion: (f64, f64, f64),
    pub after_motion: (f64, f64, f64),
    pub before_rotation: (f32, f32, f32),
    pub after_rotation: (f32, f32, f32),
    pub tick_count: i32,
}

/// Two client-world ticks applied to a newly added Experience Orb.
#[derive(Debug, Clone, PartialEq)]
pub struct ExperienceOrbTickObservation {
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_position: (f64, f64, f64),
    pub after_first_tick_position: (f64, f64, f64),
    pub after_second_tick_position: (f64, f64, f64),
    pub before_motion: (f64, f64, f64),
    pub after_first_tick_motion: (f64, f64, f64),
    pub after_second_tick_motion: (f64, f64, f64),
    pub tick_count: i32,
}

/// One Java LivingEntity interpolation target and its three world-tick steps.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityInterpolationObservation {
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_position: (f64, f64, f64),
    pub target_position: (f64, f64, f64),
    pub before_rotation: (f32, f32),
    pub target_rotation: (f32, f32),
    pub positions_after_ticks: Vec<(f64, f64, f64)>,
    pub rotations_after_ticks: Vec<(f32, f32)>,
    pub tick_count: i32,
}

/// One large remote Player position update and its immediate snap state.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPositionSnapObservation {
    pub packet_kind: &'static str,
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_position: (f64, f64, f64),
    pub target_position: (f64, f64, f64),
    pub before_rotation: (f32, f32),
    pub target_rotation: (f32, f32),
    pub applied_position: (f64, f64, f64),
    pub applied_rotation: (f32, f32, f32),
    pub previous_position: (f64, f64, f64),
    pub previous_rotation: (f32, f32, f32),
    pub interpolation_steps: u8,
}

/// Wire-level fields decoded from one clientbound entity-motion packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMotionObservation {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub applied_to_world: bool,
    pub applied_motion: Option<(f64, f64, f64)>,
}

/// One remote Player motion target and its three world-tick interpolation steps.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMotionInterpolationObservation {
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_motion: (f64, f64, f64),
    pub target_motion: (f64, f64, f64),
    pub applied_motion: (f64, f64, f64),
    pub motions_after_ticks: Vec<(f64, f64, f64)>,
    pub tick_count: i32,
}

/// Wire-level fields decoded from one clientbound head-rotation packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityHeadRotationObservation {
    pub entity_id: i32,
    pub y_head_rot: f32,
    pub applied_to_world: bool,
    pub applied_head_rotation: Option<f32>,
}

/// Wire-level fields decoded from one clientbound remove-entities packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveEntitiesObservation {
    pub entity_ids: Vec<i32>,
    pub applied_entity_count: usize,
}

/// Wire-level fields decoded from one clientbound passenger-list packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPassengersObservation {
    pub vehicle_id: i32,
    pub passenger_ids: Vec<i32>,
    pub applied_to_world: bool,
    pub applied_vehicle_passengers: Option<Vec<i32>>,
    pub applied_passenger_vehicles: Vec<(i32, Option<i32>)>,
}

/// One client-world tick observed across a vehicle and a known passenger.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPassengerTickObservation {
    pub vehicle_id: i32,
    pub passenger_id: i32,
    pub passenger_type_id: i32,
    pub before_vehicle_tick_count: i32,
    pub after_vehicle_tick_count: i32,
    pub before_passenger_tick_count: i32,
    pub after_passenger_tick_count: i32,
    pub before_passenger_position: (f64, f64, f64),
    pub after_passenger_position: (f64, f64, f64),
    pub before_passenger_motion: (f64, f64, f64),
    pub after_passenger_motion: (f64, f64, f64),
    pub before_passenger_rotation: (f32, f32, f32),
    pub after_passenger_rotation: (f32, f32, f32),
    pub before_passenger_body_y_rotation: f32,
    pub after_passenger_body_y_rotation: f32,
}

/// Wire-level fields decoded from one clientbound entity-metadata packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityMetadataObservation {
    pub entity_id: i32,
    pub metadata_length: usize,
    pub applied_to_world: bool,
    pub(crate) metadata: Vec<u8>,
}

/// What one clientbound set-equipment packet puts in the entity's slots, by
/// `EquipmentSlot` ordinal; an empty stack clears its slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityEquipmentObservation {
    pub entity_id: i32,
    pub slots: Vec<(usize, Option<ClientItemStackState>)>,
}

/// Wire-level fields decoded from one clientbound entity-position-sync packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPositionSyncObservation {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
    pub applied_to_world: bool,
    pub applied_on_ground: Option<bool>,
    pub applied_position: Option<(f64, f64, f64)>,
    pub applied_rotation: Option<(f32, f32, f32)>,
}

/// Wire-level fields decoded from one clientbound entity-teleport packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityTeleportObservation {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub relative_flags: i32,
    pub on_ground: bool,
    pub motion_before: Option<(f64, f64, f64)>,
    pub applied_to_world: bool,
    pub applied_on_ground: Option<bool>,
    pub applied_position: Option<(f64, f64, f64)>,
    pub applied_rotation: Option<(f32, f32, f32)>,
    pub applied_motion: Option<(f64, f64, f64)>,
}

/// Wire-level fields decoded from one clientbound relative entity-move packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMoveObservation {
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub y_rot: Option<f32>,
    pub x_rot: Option<f32>,
    pub has_position: bool,
    pub has_rotation: bool,
    pub on_ground: bool,
    pub applied_to_world: bool,
    pub applied_on_ground: Option<bool>,
    pub applied_position: Option<(f64, f64, f64)>,
    pub applied_rotation: Option<(f32, f32, f32)>,
}

/// Wire-level fields decoded from one clientbound container slot update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSetSlotObservation {
    pub container_id: i32,
    pub state_id: i32,
    pub slot: i16,
    pub item_count: Option<i32>,
    pub item_registry_id: Option<i32>,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    pub item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound cursor item update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorItemObservation {
    pub item_count: Option<i32>,
    pub item_registry_id: Option<i32>,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    pub item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one direct local-player inventory update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPlayerInventoryObservation {
    pub slot: i32,
    pub item_count: Option<i32>,
    pub item_registry_id: Option<i32>,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    pub item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound container content update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSetContentObservation {
    pub container_id: i32,
    pub state_id: i32,
    pub item_count: usize,
    pub items: Vec<Option<ClientItemStackState>>,
    pub hotbar_slot_zero_item_count: Option<i32>,
    pub hotbar_slot_zero_item_registry_id: Option<i32>,
    pub hotbar_slot_zero_positive_component_type_ids: Vec<i32>,
    pub hotbar_slot_zero_negative_component_type_ids: Vec<i32>,
    pub main_inventory_slot_zero_item_count: Option<i32>,
    pub main_inventory_slot_zero_item_registry_id: Option<i32>,
    pub main_inventory_slot_zero_positive_component_type_ids: Vec<i32>,
    pub main_inventory_slot_zero_negative_component_type_ids: Vec<i32>,
    pub carried_item_count: Option<i32>,
    pub carried_item_registry_id: Option<i32>,
    pub carried_positive_component_type_ids: Vec<i32>,
    pub carried_negative_component_type_ids: Vec<i32>,
    pub carried_item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound mob-effect removal packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectRemoveObservation {
    pub entity_id: i32,
    pub effect_registry_id: i32,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound mob-effect update packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectUpdateObservation {
    pub entity_id: i32,
    pub effect_registry_id: i32,
    pub amplifier: i32,
    pub duration_ticks: i32,
    pub flags: u8,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound system-chat packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemChatObservation {
    pub overlay: bool,
    pub translation_key: Option<String>,
}

/// Structural fields decoded from one clientbound level-chunk-with-light packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkPacketObservation {
    pub x: i32,
    pub z: i32,
    pub heightmap_count: usize,
    pub heightmap_observations: Vec<HeightmapObservation>,
    pub section_data_length: usize,
    pub section_observations: Vec<ChunkSectionObservation>,
    pub block_entity_count: usize,
    pub block_entities: Vec<BlockEntityObservation>,
    pub sky_mask_word_count: usize,
    pub block_mask_word_count: usize,
    pub empty_sky_mask_word_count: usize,
    pub empty_block_mask_word_count: usize,
    pub sky_update_count: usize,
    pub sky_update_lengths: Vec<usize>,
    pub block_update_count: usize,
    pub block_update_lengths: Vec<usize>,
    pub(crate) light_data: LightUpdateData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LightUpdateData {
    pub(crate) sky_mask: BitSet,
    pub(crate) block_mask: BitSet,
    pub(crate) empty_sky_mask: BitSet,
    pub(crate) empty_block_mask: BitSet,
    pub(crate) sky_updates: Vec<Vec<u8>>,
    pub(crate) block_updates: Vec<Vec<u8>>,
}

/// Wire-level fields observed from one chunk heightmap entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightmapObservation {
    pub type_id: i32,
    pub word_count: usize,
}

/// Structural fields decoded from one serialized 16-by-16 chunk section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSectionObservation {
    pub non_empty_block_count: i16,
    pub fluid_count: i16,
    pub block_state_bits_per_entry: u8,
    pub block_state_palette_entry_count: usize,
    pub block_state_palette_ids: Vec<i32>,
    pub block_state_storage_word_count: usize,
    pub block_state_first_values: Vec<i32>,
    pub block_state_values: Vec<i32>,
    pub block_state_distinct_value_count: usize,
    pub biome_bits_per_entry: u8,
    pub biome_palette_entry_count: usize,
    pub biome_palette_ids: Vec<i32>,
    pub biome_storage_word_count: usize,
    pub biome_first_values: Vec<i32>,
    pub biome_values: Vec<i32>,
    pub biome_distinct_value_count: usize,
}

/// The observable result of a status and ping session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusConnectionTrace {
    pub packets: Vec<PacketObservation>,
    pub status_response: String,
    pub pong_time: i64,
}

pub(crate) fn encode_move_player_pos_rot(
    output: &mut Vec<u8>,
    position: (f64, f64, f64, f32, f32),
    flags: u8,
) -> Result<(), ConnectionError> {
    output.extend_from_slice(&position.0.to_be_bytes());
    output.extend_from_slice(&position.1.to_be_bytes());
    output.extend_from_slice(&position.2.to_be_bytes());
    output.extend_from_slice(&position.3.to_be_bytes());
    output.extend_from_slice(&position.4.to_be_bytes());
    output.push(flags);
    Ok(())
}

pub(crate) fn encode_perform_respawn(output: &mut Vec<u8>) -> Result<(), ConnectionError> {
    output.push(0);
    Ok(())
}

pub(crate) fn encode_abort_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 1, position, 0, sequence)
}

pub(crate) fn encode_start_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 0, position, 1, sequence)
}

pub(crate) fn encode_stop_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 2, position, 1, sequence)
}

pub(crate) fn encode_release_use_item(output: &mut Vec<u8>) -> Result<(), ConnectionError> {
    encode_player_action(output, 5, (0, 0, 0), 0, 0)
}

pub(crate) fn encode_use_item_on(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_use_item_on_hit(output, position, 1, (0.5, 1.0, 0.5), sequence)
}

pub(crate) fn encode_use_item(
    output: &mut Vec<u8>,
    sequence: i32,
    y_rot: f32,
    x_rot: f32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(sequence, output);
    output.extend_from_slice(&y_rot.to_be_bytes());
    output.extend_from_slice(&x_rot.to_be_bytes());
    Ok(())
}

pub(crate) fn encode_use_item_on_hit(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    direction: u8,
    click: (f32, f32, f32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    output.extend_from_slice(&pack_block_position(position).to_be_bytes());
    wire::write_varint(i32::from(direction), output);
    output.extend_from_slice(&click.0.to_be_bytes());
    output.extend_from_slice(&click.1.to_be_bytes());
    output.extend_from_slice(&click.2.to_be_bytes());
    output.push(0);
    output.push(0);
    wire::write_varint(sequence, output);
    Ok(())
}

pub(crate) fn encode_creative_stone_slot(
    output: &mut Vec<u8>,
    slot: i16,
    stone_item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_creative_item_slot(output, slot, 1, stone_item_registry_id)
}

pub(crate) fn encode_creative_item_slot(
    output: &mut Vec<u8>,
    slot: i16,
    count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    output.extend_from_slice(&slot.to_be_bytes());
    wire::write_varint(count, output);
    wire::write_varint(item_registry_id, output);
    wire::write_varint(0, output);
    wire::write_varint(0, output);
    Ok(())
}

/// `ServerboundSetCreativeModeSlotPacket`: a slot and its stack as the
/// network encodes an optional item stack for the server, component patch
/// included, each value behind its length.
pub(crate) fn encode_creative_slot(
    output: &mut Vec<u8>,
    slot: i16,
    stack: Option<&ClientItemStackState>,
) {
    output.extend_from_slice(&slot.to_be_bytes());
    match stack {
        Some(stack) if stack.item_count() > 0 => {
            wire::write_varint(stack.item_count(), output);
            wire::write_varint(stack.item_registry_id(), output);
            // The server reads a client's components behind their lengths. A
            // patch this cannot read is left off rather than sent wrong.
            match voxel_game::item_components::delimit(stack.patch_bytes()) {
                Some(patch) if !stack.patch_bytes().is_empty() => output.extend_from_slice(&patch),
                _ => {
                    wire::write_varint(0, output);
                    wire::write_varint(0, output);
                }
            }
        }
        _ => wire::write_varint(0, output),
    }
}

pub(crate) fn encode_inventory_pickup_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(0, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_secondary_pickup_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(1);
    wire::write_varint(0, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_throw_one_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(4, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_throw_all_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(1);
    wire::write_varint(4, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_drag_empty_cancel_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&(-999_i16).to_be_bytes());
    output.push(0);
    wire::write_varint(5, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_click_with_input(
    output: &mut Vec<u8>,
    state_id: i32,
    slot: i16,
    button: u8,
    input_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&slot.to_be_bytes());
    output.push(button);
    wire::write_varint(input_id, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_pickup_with_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction(output, state_id, None, Some((item_count, item_registry_id)))
}

pub(crate) fn encode_inventory_pickup_with_wrong_slot_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction(
        output,
        state_id,
        Some((item_count, item_registry_id)),
        Some((item_count, item_registry_id)),
    )
}

pub(crate) fn encode_inventory_pickup_with_wrong_carried_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction(output, state_id, None, None)
}

pub(crate) fn encode_inventory_pickup_with_wrong_both_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction(output, state_id, Some((item_count, item_registry_id)), None)
}

pub(crate) fn encode_inventory_pickup_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    changed_slot_item: Option<(i32, i32)>,
    carried_item: Option<(i32, i32)>,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction_with_carried_patch(
        output,
        state_id,
        changed_slot_item,
        carried_item,
        &[],
        &[],
    )
}

pub(crate) fn encode_inventory_pickup_with_component_hash_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    let spec = inventory_component_spec("custom_model_data")
        .expect("custom model data component is in the conformance table");
    let added_components = [(spec.component_type_id, 0x0102_0304)];
    let removed_components = [spec.component_type_id];
    encode_inventory_pickup_prediction_with_carried_patch(
        output,
        state_id,
        None,
        Some((item_count, item_registry_id)),
        &added_components,
        &removed_components,
    )
}

pub(crate) fn encode_inventory_pickup_with_component_spec_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    spec: &'static InventoryComponentProbeSpec,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction_with_carried_patch(
        output,
        state_id,
        None,
        Some((1, spec.item_registry_id)),
        spec.carried_components,
        &[],
    )
}

pub(crate) fn encode_inventory_pickup_with_partial_changed_slot_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction_packet(
        output,
        state_id,
        None,
        Some((item_count, item_registry_id)),
        &[],
        &[],
    )
}

pub(crate) fn encode_inventory_pickup_prediction_with_carried_patch(
    output: &mut Vec<u8>,
    state_id: i32,
    changed_slot_item: Option<(i32, i32)>,
    carried_item: Option<(i32, i32)>,
    carried_added_components: &[(i32, i32)],
    carried_removed_components: &[i32],
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction_packet(
        output,
        state_id,
        Some(changed_slot_item),
        carried_item,
        carried_added_components,
        carried_removed_components,
    )
}

pub(crate) fn encode_inventory_pickup_prediction_packet(
    output: &mut Vec<u8>,
    state_id: i32,
    changed_slot_item: Option<Option<(i32, i32)>>,
    carried_item: Option<(i32, i32)>,
    carried_added_components: &[(i32, i32)],
    carried_removed_components: &[i32],
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(0, output);
    if let Some(changed_slot_item) = changed_slot_item {
        wire::write_varint(1, output);
        output.extend_from_slice(&36_i16.to_be_bytes());
        encode_hashed_stack(output, changed_slot_item);
    } else {
        wire::write_varint(0, output);
    }
    encode_hashed_stack_with_components(
        output,
        carried_item,
        carried_added_components,
        carried_removed_components,
    );
    Ok(())
}

pub(crate) fn encode_hashed_stack(output: &mut Vec<u8>, item: Option<(i32, i32)>) {
    encode_hashed_stack_with_components(output, item, &[], &[]);
}

pub(crate) fn encode_hashed_stack_with_components(
    output: &mut Vec<u8>,
    item: Option<(i32, i32)>,
    added_components: &[(i32, i32)],
    removed_components: &[i32],
) {
    let Some((item_count, item_registry_id)) = item else {
        output.push(0);
        return;
    };

    output.push(1);
    wire::write_varint(item_registry_id, output);
    wire::write_varint(item_count, output);
    wire::write_varint(added_components.len() as i32, output);
    for (component_type_id, component_hash) in added_components {
        wire::write_varint(*component_type_id, output);
        output.extend_from_slice(&component_hash.to_be_bytes());
    }
    wire::write_varint(removed_components.len() as i32, output);
    for component_type_id in removed_components {
        wire::write_varint(*component_type_id, output);
    }
}

pub(crate) fn encode_inventory_quick_move_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(1, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_swap_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&9_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(2, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_set_carried_item(
    output: &mut Vec<u8>,
    slot: i16,
) -> Result<(), ConnectionError> {
    output.extend_from_slice(&slot.to_be_bytes());
    Ok(())
}

pub(crate) fn encode_player_action(
    output: &mut Vec<u8>,
    action: i32,
    position: (i32, i32, i32),
    direction: u8,
    sequence: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(action, output);
    output.extend_from_slice(&pack_block_position(position).to_be_bytes());
    output.push(direction);
    wire::write_varint(sequence, output);
    Ok(())
}

pub(crate) fn pack_block_position(position: (i32, i32, i32)) -> i64 {
    const HORIZONTAL_MASK: i64 = (1_i64 << 26) - 1;
    const Y_MASK: i64 = (1_i64 << 12) - 1;

    ((i64::from(position.0) & HORIZONTAL_MASK) << 38)
        | ((i64::from(position.2) & HORIZONTAL_MASK) << 12)
        | (i64::from(position.1) & Y_MASK)
}

pub(crate) fn support_block_position(position: (f64, f64, f64, f32, f32)) -> (i32, i32, i32) {
    let block = voxel_core::binary::BlockPos::containing(position.0, position.1 - 1.0, position.2);
    (block.x, block.y, block.z)
}

pub(crate) fn encode_packet_frame(
    body: &[u8],
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    if body.len() > MAX_PACKET_UNCOMPRESSED_LENGTH {
        return Err(ConnectionError::Frame(FrameError::PacketTooLarge));
    }

    let mut payload = Vec::new();
    match compression_threshold {
        Some(threshold) if body.len() < threshold => {
            wire::write_varint(0, &mut payload);
            payload.extend_from_slice(body);
        }
        Some(_) => {
            wire::write_varint(
                i32::try_from(body.len())
                    .map_err(|_| ConnectionError::Frame(FrameError::PacketTooLarge))?,
                &mut payload,
            );
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body)?;
            payload.extend_from_slice(
                &encoder
                    .finish()
                    .map_err(|error| ConnectionError::Compression(error.to_string()))?,
            );
        }
        None => payload.extend_from_slice(body),
    }
    frame::prepend(&payload).map_err(ConnectionError::from)
}

pub(crate) fn decode_packet_frame(
    frame: &[u8],
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    let Some(_) = compression_threshold else {
        return Ok(frame.to_vec());
    };

    let mut input = frame;
    let uncompressed_length = wire::read_varint(&mut input)?;
    if uncompressed_length < 0 {
        return Err(ConnectionError::Compression(
            "negative uncompressed length".to_owned(),
        ));
    }
    if uncompressed_length == 0 {
        return Ok(input.to_vec());
    }

    let uncompressed_length = usize::try_from(uncompressed_length)
        .map_err(|_| ConnectionError::Compression("invalid uncompressed length".to_owned()))?;
    if uncompressed_length > MAX_PACKET_UNCOMPRESSED_LENGTH {
        return Err(ConnectionError::Compression(
            "uncompressed packet exceeds 8 MiB".to_owned(),
        ));
    }

    let decoder = ZlibDecoder::new(input);
    let mut output = Vec::with_capacity(uncompressed_length);
    decoder
        .take((uncompressed_length as u64) + 1)
        .read_to_end(&mut output)?;
    if output.len() != uncompressed_length {
        return Err(ConnectionError::Compression(format!(
            "declared {uncompressed_length} bytes, decoded {}",
            output.len()
        )));
    }
    Ok(output)
}

pub(crate) fn split_packet(body: &[u8]) -> Result<(i32, &[u8]), ConnectionError> {
    let mut input = body;
    let packet_id = wire::read_varint(&mut input)?;
    Ok((packet_id, input))
}

pub(crate) fn parse_login_packet(input: &[u8]) -> Result<LoginObservation, ConnectionError> {
    let mut input = input;
    let packet = LoginObservation {
        player_id: read_i32(&mut input, "login player id")?,
        hardcore: read_bool(&mut input, "login hardcore")?,
        level_count: {
            let count = read_count(&mut input, "login level count")?;
            for _ in 0..count {
                read_identifier_string(&mut input, "login level identifier")?;
            }
            count
        },
        max_players: read_varint(&mut input, "login max players")?,
        chunk_radius: read_varint(&mut input, "login chunk radius")?,
        simulation_distance: read_varint(&mut input, "login simulation distance")?,
        reduced_debug_info: read_bool(&mut input, "login reduced debug info")?,
        show_death_screen: read_bool(&mut input, "login show death screen")?,
        do_limited_crafting: read_bool(&mut input, "login limited crafting")?,
        spawn_info: parse_spawn_info(&mut input)?,
        online_mode: read_bool(&mut input, "login online mode")?,
        enforces_secure_chat: read_bool(&mut input, "login secure chat")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("login trailing payload"));
    }
    Ok(packet)
}

pub(crate) fn parse_respawn_packet(
    input: &[u8],
) -> Result<(SpawnInfoObservation, u8), ConnectionError> {
    let mut input = input;
    let spawn_info = parse_spawn_info(&mut input)?;
    let data_to_keep = read_byte(&mut input, "respawn data to keep")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("respawn trailing payload"));
    }
    Ok((spawn_info, data_to_keep))
}

pub(crate) fn parse_open_screen_packet(
    input: &[u8],
) -> Result<OpenScreenObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "open screen container id")?;
    let menu_type_id = read_varint(&mut input, "open screen menu type")?;
    let title_input = input;
    let title = voxel_core::nbt::read_any_nbt(&mut input)
        .map_err(|_| ConnectionError::Codec("open screen title"))?;
    if title.is_none() {
        return Err(ConnectionError::Codec("open screen title"));
    }
    let title_length = title_input.len() - input.len();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("open screen trailing payload"));
    }
    Ok(OpenScreenObservation {
        container_id,
        menu_type_id,
        title_nbt: title_input[..title_length].to_vec(),
        applied_to_world: false,
    })
}

pub(crate) fn parse_container_close_packet(
    input: &[u8],
) -> Result<ContainerCloseObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container close id")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container close trailing payload"));
    }
    Ok(ContainerCloseObservation {
        container_id,
        applied_to_world: false,
    })
}

/// One clientbound container-data packet: a menu's `ContainerData` value
/// (a furnace's burn time, an enchanting table's cost).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerDataObservation {
    pub container_id: i32,
    pub id: i16,
    pub value: i16,
}

pub(crate) fn parse_container_set_data_packet(
    input: &[u8],
) -> Result<ContainerDataObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container data id")?;
    let id = read_i16(&mut input, "container data slot")?;
    let value = read_i16(&mut input, "container data value")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container data trailing payload"));
    }
    Ok(ContainerDataObservation {
        container_id,
        id,
        value,
    })
}

/// What a clientbound update-recipes packet carries: the item sets by name
/// (the items a furnace, a blast furnace or a smoker accepts, and so on, as
/// registry ids) and the stonecutter's entries.
pub(crate) struct RecipeUpdate {
    pub sets: Vec<(String, Vec<i32>)>,
    pub stonecutter: Vec<voxel_world::StonecutterEntry>,
}

/// `SlotDisplay.STREAM_CODEC`.
fn read_slot_display(
    input: &mut &[u8],
    depth: usize,
) -> Result<voxel_world::SlotDisplay, ConnectionError> {
    use voxel_world::SlotDisplay as D;
    if depth > 16 {
        return Err(ConnectionError::Codec("slot display too deep"));
    }
    let next = |input: &mut &[u8]| read_slot_display(input, depth + 1);
    Ok(match read_varint(input, "slot display type")? {
        0 => D::Empty,
        1 => D::AnyFuel,
        2 => D::WithAnyPotion(Box::new(next(input)?)),
        3 => {
            let source = next(input)?;
            D::OnlyWithComponent(Box::new(source), read_varint(input, "display component")?)
        }
        4 => D::Item(read_varint(input, "display item")?),
        5 => {
            let item = read_varint(input, "display stack item")?;
            let count = read_varint(input, "display stack count")?;
            voxel_game::item_components::read_patch(input)
                .map_err(|_| ConnectionError::Codec("display stack components"))?;
            D::ItemStack(item, count)
        }
        6 => D::Tag(read_identifier_string(input, "display tag")?),
        7 => {
            let dye = next(input)?;
            D::Dyed(Box::new(dye), Box::new(next(input)?))
        }
        8 => {
            let base = next(input)?;
            let material = next(input)?;
            // `Holder<TrimPattern>`: a registry id plus one, or zero and the
            // pattern written out (its asset, its name and whether it is a decal).
            let pattern = match read_varint(input, "display trim pattern")? {
                0 => {
                    read_identifier_string(input, "trim pattern asset")?;
                    voxel_core::nbt::read_any_nbt(input)
                        .map_err(|_| ConnectionError::Codec("trim pattern name"))?;
                    read_bool(input, "trim pattern decal")?;
                    -1
                }
                id => id - 1,
            };
            D::SmithingTrim(Box::new(base), Box::new(material), pattern)
        }
        9 => {
            let source = next(input)?;
            D::WithRemainder(Box::new(source), Box::new(next(input)?))
        }
        10 => {
            let count = read_varint(input, "display count")?;
            let mut all = Vec::new();
            for _ in 0..count {
                all.push(next(input)?);
            }
            D::Composite(all)
        }
        _ => return Err(ConnectionError::Codec("unknown slot display")),
    })
}

/// `Ingredient.CONTENTS_STREAM_CODEC`: a tag, or the items it lists.
fn read_ingredient(input: &mut &[u8]) -> Result<voxel_world::Ingredient, ConnectionError> {
    let marker = read_varint(input, "ingredient marker")?;
    if marker == 0 {
        return Ok(voxel_world::Ingredient::Tag(read_identifier_string(
            input,
            "ingredient tag",
        )?));
    }
    let mut items = Vec::new();
    for _ in 0..marker - 1 {
        items.push(read_varint(input, "ingredient item")?);
    }
    Ok(voxel_world::Ingredient::Items(items))
}

pub(crate) fn parse_update_recipes_packet(input: &[u8]) -> Result<RecipeUpdate, ConnectionError> {
    let mut input = input;
    let sets = read_varint(&mut input, "recipe property set count")?;
    let mut out = Vec::new();
    for _ in 0..sets {
        let name = read_identifier_string(&mut input, "recipe property set name")?;
        let count = read_varint(&mut input, "recipe property set size")?;
        let mut items = Vec::new();
        for _ in 0..count {
            items.push(read_varint(&mut input, "recipe property set item")?);
        }
        out.push((name, items));
    }
    let entries = read_varint(&mut input, "stonecutter recipe count")?;
    let mut stonecutter = Vec::new();
    for _ in 0..entries {
        let ingredient = read_ingredient(&mut input)?;
        let display = read_slot_display(&mut input, 0)?;
        stonecutter.push(voxel_world::StonecutterEntry {
            input: ingredient,
            display,
        });
    }
    Ok(RecipeUpdate {
        sets: out,
        stonecutter,
    })
}

fn read_slot_displays(input: &mut &[u8]) -> Result<Vec<voxel_world::SlotDisplay>, ConnectionError> {
    let count = read_varint(input, "recipe display count")?;
    let mut all = Vec::new();
    for _ in 0..count {
        all.push(read_slot_display(input, 0)?);
    }
    Ok(all)
}

/// `RecipeDisplay.STREAM_CODEC`: the crafting station each one ends with is
/// read and dropped.
pub(crate) fn read_recipe_display(
    input: &mut &[u8],
) -> Result<voxel_world::RecipeDisplay, ConnectionError> {
    use voxel_world::RecipeDisplay as R;
    let slot = |input: &mut &[u8]| read_slot_display(input, 0);
    Ok(match read_varint(input, "recipe display type")? {
        0 => {
            let ingredients = read_slot_displays(input)?;
            let result = slot(input)?;
            slot(input)?;
            R::Shapeless {
                ingredients,
                result,
            }
        }
        1 => {
            let width = read_varint(input, "shaped width")?;
            let height = read_varint(input, "shaped height")?;
            let ingredients = read_slot_displays(input)?;
            if i64::from(width) * i64::from(height) != ingredients.len() as i64 {
                return Err(ConnectionError::Codec("shaped recipe size"));
            }
            let result = slot(input)?;
            slot(input)?;
            R::Shaped {
                width,
                height,
                ingredients,
                result,
            }
        }
        2 => {
            let ingredient = slot(input)?;
            let fuel = slot(input)?;
            let result = slot(input)?;
            slot(input)?;
            read_varint(input, "furnace duration")?;
            read_f32(input, "furnace experience")?;
            R::Furnace {
                ingredient,
                fuel,
                result,
            }
        }
        3 => {
            slot(input)?;
            let result = slot(input)?;
            slot(input)?;
            R::Stonecutter { result }
        }
        4 => {
            slot(input)?;
            slot(input)?;
            slot(input)?;
            let result = slot(input)?;
            slot(input)?;
            R::Smithing { result }
        }
        _ => return Err(ConnectionError::Codec("unknown recipe display")),
    })
}

/// `ClientboundRecipeBookAddPacket`: the entries with their flag bytes, and
/// whether they replace the book.
pub(crate) fn parse_recipe_book_add_packet(
    input: &[u8],
) -> Result<(Vec<(voxel_world::RecipeDisplayEntry, u8)>, bool), ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "recipe book add count")?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let id = read_varint(&mut input, "recipe id")?;
        let display = read_recipe_display(&mut input)?;
        let group = match read_varint(&mut input, "recipe group")? {
            0 => None,
            value => Some(value - 1),
        };
        let category = read_varint(&mut input, "recipe category")?;
        let requirements = if read_byte(&mut input, "recipe requirements flag")? != 0 {
            let count = read_varint(&mut input, "recipe requirement count")?;
            let mut all = Vec::new();
            for _ in 0..count {
                all.push(read_ingredient(&mut input)?);
            }
            Some(all)
        } else {
            None
        };
        let flags = read_byte(&mut input, "recipe flags")?;
        entries.push((
            voxel_world::RecipeDisplayEntry {
                id,
                display,
                group,
                category,
                requirements,
            },
            flags,
        ));
    }
    let replace = read_byte(&mut input, "recipe book replace")? != 0;
    Ok((entries, replace))
}

pub(crate) fn parse_recipe_book_remove_packet(input: &[u8]) -> Result<Vec<i32>, ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "recipe book remove count")?;
    let mut ids = Vec::new();
    for _ in 0..count {
        ids.push(read_varint(&mut input, "recipe book remove id")?);
    }
    Ok(ids)
}

/// `RecipeBookSettings.read`: open and filtering flags for crafting,
/// furnace, blast furnace and smoker, in that order.
pub(crate) fn parse_recipe_book_settings_packet(
    input: &[u8],
) -> Result<[voxel_world::BookTypeSettings; 4], ConnectionError> {
    let mut input = input;
    let mut settings = [voxel_world::BookTypeSettings::default(); 4];
    for entry in &mut settings {
        entry.open = read_byte(&mut input, "recipe book open")? != 0;
        entry.filtering = read_byte(&mut input, "recipe book filtering")? != 0;
    }
    Ok(settings)
}

/// `ClientboundPlaceGhostRecipePacket`.
pub(crate) fn parse_place_ghost_recipe_packet(
    input: &[u8],
) -> Result<(i32, voxel_world::RecipeDisplay), ConnectionError> {
    let mut input = input;
    let container = read_varint(&mut input, "ghost container")?;
    Ok((container, read_recipe_display(&mut input)?))
}

pub(crate) fn parse_set_health_packet(input: &[u8]) -> Result<HealthObservation, ConnectionError> {
    let mut input = input;
    let packet = HealthObservation {
        health: read_f32(&mut input, "set health health")?,
        food: read_varint(&mut input, "set health food")?,
        saturation: read_f32(&mut input, "set health saturation")?,
        applied_to_world: false,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("set health trailing payload"));
    }
    Ok(packet)
}

pub fn parse_player_abilities_packet(
    input: &[u8],
) -> Result<PlayerAbilitiesPacket, ConnectionError> {
    let mut input = input;
    let flags = read_byte(&mut input, "player abilities flags")?;
    let packet = PlayerAbilitiesPacket {
        invulnerable: flags & 0x01 != 0,
        flying: flags & 0x02 != 0,
        can_fly: flags & 0x04 != 0,
        can_instabuild: flags & 0x08 != 0,
        flying_speed: read_f32(&mut input, "player abilities flying speed")?,
        walking_speed: read_f32(&mut input, "player abilities walking speed")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("player abilities trailing payload"));
    }
    Ok(packet)
}

pub fn parse_game_event_packet(input: &[u8]) -> Result<GameEventPacket, ConnectionError> {
    let mut input = input;
    let packet = GameEventPacket {
        event_id: read_byte(&mut input, "game event id")?,
        param: read_f32(&mut input, "game event parameter")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("game event trailing payload"));
    }
    Ok(packet)
}

pub(crate) fn parse_update_attributes_packet(
    input: &[u8],
) -> Result<AttributeUpdateObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "update attributes entity id")?;
    let attribute_count = read_count(&mut input, "update attributes count")?;
    if attribute_count > 128 {
        return Err(ConnectionError::Codec("update attributes count"));
    }
    let mut attributes = Vec::with_capacity(attribute_count);
    for _ in 0..attribute_count {
        let attribute_id = read_varint(&mut input, "update attribute id")?;
        let base_value = read_f64(&mut input, "update attribute base value")?;
        let modifier_count = read_count(&mut input, "update attribute modifier count")?;
        let mut modifiers = Vec::with_capacity(modifier_count);
        for _ in 0..modifier_count {
            let id = read_identifier_string(&mut input, "update attribute modifier id")?;
            let amount = read_f64(&mut input, "update attribute modifier amount")?;
            let operation = read_byte(&mut input, "update attribute modifier operation")?;
            modifiers.push(ClientAttributeModifierState::from_wire(
                id, amount, operation,
            ));
        }
        attributes.push(ClientAttributeState::from_wire(
            attribute_id,
            base_value,
            modifiers,
        ));
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("update attributes trailing payload"));
    }
    Ok(AttributeUpdateObservation {
        entity_id,
        attributes,
        applied_to_world: false,
    })
}

fn parse_spawn_info(input: &mut &[u8]) -> Result<SpawnInfoObservation, ConnectionError> {
    let dimension_type_id = read_varint(input, "spawn dimension type")?;
    let dimension = read_identifier_string(input, "spawn dimension")?;
    let seed = read_i64(input, "spawn seed")?;
    let game_type = read_byte(input, "spawn game type")?;
    let previous_game_type = match read_byte(input, "spawn previous game type")? {
        u8::MAX => None,
        value => Some(value),
    };
    let is_debug = read_bool(input, "spawn debug flag")?;
    let is_flat = read_bool(input, "spawn flat flag")?;
    let last_death_location = if read_bool(input, "spawn death-location presence")? {
        Some(GlobalPositionObservation {
            dimension: read_identifier_string(input, "spawn death-location dimension")?,
            position: read_i64(input, "spawn death-location position")?,
        })
    } else {
        None
    };
    let portal_cooldown = read_varint(input, "spawn portal cooldown")?;
    let sea_level = read_varint(input, "spawn sea level")?;
    Ok(SpawnInfoObservation {
        dimension_type_id,
        dimension,
        seed,
        game_type,
        previous_game_type,
        is_debug,
        is_flat,
        last_death_location,
        portal_cooldown,
        sea_level,
    })
}

fn read_identifier_string(
    input: &mut &[u8],
    name: &'static str,
) -> Result<String, ConnectionError> {
    voxel_core::identifier::read_identifier(input)
        .map(|identifier| identifier.as_string())
        .map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn parse_block_update_packet(
    input: &[u8],
) -> Result<BlockUpdateObservation, ConnectionError> {
    let mut input = input;
    let position =
        voxel_core::binary::BlockPos::unpack(read_i64(&mut input, "block update position")? as u64);
    let state_id = read_varint(&mut input, "block update state")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("block update trailing payload"));
    }
    Ok(BlockUpdateObservation {
        x: position.x,
        y: position.y,
        z: position.z,
        state_id,
    })
}

pub(crate) fn parse_block_entity_data_packet(
    input: &[u8],
) -> Result<BlockEntityObservation, ConnectionError> {
    let mut input = input;
    let position =
        voxel_core::binary::BlockPos::unpack(read_i64(&mut input, "block entity position")? as u64);
    let type_id = read_varint(&mut input, "block entity type")?;
    let tag = read_nbt_bytes(&mut input, "block entity nbt")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("block entity trailing payload"));
    }
    let tag_length = tag.as_ref().map_or(0, Vec::len);
    Ok(BlockEntityObservation {
        x: position.x,
        y: position.y,
        z: position.z,
        type_id,
        tag_length,
        applied_to_world: false,
        tag,
    })
}

pub(crate) fn parse_block_destruction_packet(
    input: &[u8],
) -> Result<BlockDestructionObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "block destruction entity id")?;
    let position = voxel_core::binary::BlockPos::unpack(read_i64(
        &mut input,
        "block destruction position",
    )? as u64);
    let progress = read_byte(&mut input, "block destruction progress")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("block destruction trailing payload"));
    }
    Ok(BlockDestructionObservation {
        entity_id,
        x: position.x,
        y: position.y,
        z: position.z,
        progress,
    })
}

pub(crate) fn parse_add_entity_packet(
    input: &[u8],
) -> Result<AddEntityObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "add entity id")?;
    let uuid = voxel_core::binary::read_uuid(&mut input)
        .map_err(|_| ConnectionError::Codec("add entity uuid"))?;
    let entity_type_id = read_varint(&mut input, "add entity type")?;
    let x = read_f64(&mut input, "add entity x")?;
    let y = read_f64(&mut input, "add entity y")?;
    let z = read_f64(&mut input, "add entity z")?;
    let movement = read_lp_vec3(&mut input, "add entity movement")?;
    let x_rot = decode_packed_degrees(read_byte(&mut input, "add entity x rotation")?);
    let y_rot = decode_packed_degrees(read_byte(&mut input, "add entity y rotation")?);
    let y_head_rot = decode_packed_degrees(read_byte(&mut input, "add entity head rotation")?);
    let data = read_varint(&mut input, "add entity data")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("add entity trailing payload"));
    }
    Ok(AddEntityObservation {
        entity_id,
        uuid,
        entity_type_id,
        x,
        y,
        z,
        movement,
        x_rot,
        y_rot,
        y_head_rot,
        data,
        applied_to_world: false,
        applied_rotation: None,
    })
}

/// `ClientboundSetEntityLinkPacket`: the entity on a lead and the entity that holds it (zero
/// when it is let go), each as an `int`.
pub(crate) fn parse_set_entity_link_packet(input: &[u8]) -> Result<(i32, i32), ConnectionError> {
    let mut input = input;
    let source = read_i32(&mut input, "entity link source id")?;
    let holder = read_i32(&mut input, "entity link holder id")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("entity link trailing payload"));
    }
    Ok((source, holder))
}

/// `ClientboundMoveMinecartPacket`: the minecart and the steps it took along the track, each a
/// position and a movement (doubles), two turns (a byte each), and a weight (a float).
pub(crate) fn parse_move_minecart_packet(
    input: &[u8],
) -> Result<(i32, Vec<MinecartStep>), ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "minecart entity id")?;
    let count = read_varint(&mut input, "minecart step count")?;
    let mut steps = Vec::new();
    for _ in 0..count {
        let mut vec3 = |name| -> Result<(f64, f64, f64), ConnectionError> {
            Ok((
                read_f64(&mut input, name)?,
                read_f64(&mut input, name)?,
                read_f64(&mut input, name)?,
            ))
        };
        let position = vec3("minecart step position")?;
        let movement = vec3("minecart step movement")?;
        let y_rot = decode_packed_degrees(read_byte(&mut input, "minecart step y rot")?);
        let x_rot = decode_packed_degrees(read_byte(&mut input, "minecart step x rot")?);
        let weight = read_f32(&mut input, "minecart step weight")?;
        steps.push(MinecartStep {
            position,
            movement,
            y_rot,
            x_rot,
            weight,
        });
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("minecart steps trailing payload"));
    }
    Ok((entity_id, steps))
}

/// `ClientboundMapItemDataPacket`: the map's id, scale and lock, the decorations (if sent) and the
/// patch of colours (a width of zero is none).
pub(crate) fn parse_map_item_data_packet(
    input: &[u8],
) -> Result<voxel_world::map::MapItemDataInput, ConnectionError> {
    use voxel_world::map::{MapDecoration, MapItemDataInput, MapPatch};
    let mut input = input;
    let map_id = read_varint(&mut input, "map id")?;
    let scale = read_byte(&mut input, "map scale")? as i8;
    let locked = read_bool(&mut input, "map locked")?;
    let decorations = if read_bool(&mut input, "map decorations present")? {
        let count = read_varint(&mut input, "map decoration count")?;
        let mut decorations = Vec::new();
        for _ in 0..count {
            let type_id = read_varint(&mut input, "map decoration type")?;
            let x = read_byte(&mut input, "map decoration x")? as i8;
            let y = read_byte(&mut input, "map decoration y")? as i8;
            let rot = read_byte(&mut input, "map decoration rot")? as i8;
            let name = if read_bool(&mut input, "map decoration name present")? {
                let before = input;
                voxel_core::nbt::read_any_nbt(&mut input)
                    .map_err(|_| ConnectionError::Codec("map decoration name"))?;
                Some(before[..before.len() - input.len()].to_vec())
            } else {
                None
            };
            decorations.push(MapDecoration {
                type_id,
                x,
                y,
                rot: rot & 15,
                name,
            });
        }
        Some(decorations)
    } else {
        None
    };
    let width = read_byte(&mut input, "map patch width")?;
    let patch = if width > 0 {
        let height = read_byte(&mut input, "map patch height")?;
        let start_x = read_byte(&mut input, "map patch x")?;
        let start_y = read_byte(&mut input, "map patch y")?;
        let length = read_count(&mut input, "map patch colour count")?;
        let colors = input
            .get(..length)
            .ok_or(ConnectionError::Codec("map patch colours"))?
            .to_vec();
        input = &input[length..];
        Some(MapPatch {
            start_x: usize::from(start_x),
            start_y: usize::from(start_y),
            width: usize::from(width),
            height: usize::from(height),
            colors,
        })
    } else {
        None
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("map item data trailing payload"));
    }
    Ok(MapItemDataInput {
        map_id,
        scale,
        locked,
        decorations,
        patch,
    })
}

pub(crate) fn parse_set_entity_motion_packet(
    input: &[u8],
) -> Result<EntityMotionObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity motion entity id")?;
    let movement = read_lp_vec3(&mut input, "entity motion movement")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("entity motion trailing payload"));
    }
    Ok(EntityMotionObservation {
        entity_id,
        x: movement.0,
        y: movement.1,
        z: movement.2,
        applied_to_world: false,
        applied_motion: None,
    })
}

pub(crate) fn parse_rotate_head_packet(
    input: &[u8],
) -> Result<EntityHeadRotationObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity head rotation entity id")?;
    let y_head_rot = decode_packed_degrees(read_byte(&mut input, "entity head rotation angle")?);
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "entity head rotation trailing payload",
        ));
    }
    Ok(EntityHeadRotationObservation {
        entity_id,
        y_head_rot,
        applied_to_world: false,
        applied_head_rotation: None,
    })
}

pub(crate) fn parse_remove_entities_packet(
    input: &[u8],
) -> Result<RemoveEntitiesObservation, ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "remove entities count")?;
    let mut entity_ids = Vec::new();
    if count > 0 {
        let count =
            usize::try_from(count).map_err(|_| ConnectionError::Codec("remove entities count"))?;
        entity_ids.reserve(count.min(input.len()));
        for _ in 0..count {
            entity_ids.push(read_varint(&mut input, "remove entity id")?);
        }
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("remove entities trailing payload"));
    }
    Ok(RemoveEntitiesObservation {
        entity_ids,
        applied_entity_count: 0,
    })
}

pub(crate) fn parse_set_passengers_packet(
    input: &[u8],
) -> Result<EntityPassengersObservation, ConnectionError> {
    let mut input = input;
    let vehicle_id = read_varint(&mut input, "set passengers vehicle id")?;
    let count = read_varint(&mut input, "set passengers count")?;
    if count < 0 {
        return Err(ConnectionError::Codec("set passengers count"));
    }
    let count =
        usize::try_from(count).map_err(|_| ConnectionError::Codec("set passengers count"))?;
    if count > input.len() {
        return Err(ConnectionError::Codec("set passengers count"));
    }
    let mut passenger_ids = Vec::with_capacity(count);
    for _ in 0..count {
        passenger_ids.push(read_varint(&mut input, "set passenger id")?);
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("set passengers trailing payload"));
    }
    Ok(EntityPassengersObservation {
        vehicle_id,
        passenger_ids,
        applied_to_world: false,
        applied_vehicle_passengers: None,
        applied_passenger_vehicles: Vec::new(),
    })
}

pub(crate) fn parse_entity_metadata_packet(
    input: &[u8],
) -> Result<EntityMetadataObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity metadata entity id")?;
    if input.last() != Some(&255) {
        return Err(ConnectionError::Codec("entity metadata end marker"));
    }
    let metadata = input.to_vec();
    Ok(EntityMetadataObservation {
        entity_id,
        metadata_length: metadata.len(),
        applied_to_world: false,
        metadata,
    })
}

/// `ClientboundSetEquipmentPacket`: the entity, then each slot's byte (the
/// top bit says another follows) and stack.
pub(crate) fn parse_entity_equipment_packet(
    input: &[u8],
) -> Result<EntityEquipmentObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "set equipment entity id")?;
    let mut slots = Vec::new();
    loop {
        let slot_byte = read_byte(&mut input, "set equipment slot")?;
        let stack = parse_optional_item_stack(&mut input, "set equipment item stack")?;
        slots.push((usize::from(slot_byte & 127), stack.client_state()));
        if slot_byte & 128 == 0 {
            break;
        }
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("set equipment trailing payload"));
    }
    Ok(EntityEquipmentObservation { entity_id, slots })
}

pub(crate) fn parse_entity_position_sync_packet(
    input: &[u8],
) -> Result<EntityPositionSyncObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity position sync entity id")?;
    let x = read_f64(&mut input, "entity position sync x")?;
    let y = read_f64(&mut input, "entity position sync y")?;
    let z = read_f64(&mut input, "entity position sync z")?;
    let delta_x = read_f64(&mut input, "entity position sync delta x")?;
    let delta_y = read_f64(&mut input, "entity position sync delta y")?;
    let delta_z = read_f64(&mut input, "entity position sync delta z")?;
    let y_rot = read_f32(&mut input, "entity position sync y rotation")?;
    let x_rot = read_f32(&mut input, "entity position sync x rotation")?;
    let on_ground = read_byte(&mut input, "entity position sync on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "entity position sync trailing payload",
        ));
    }
    Ok(EntityPositionSyncObservation {
        entity_id,
        x,
        y,
        z,
        delta_x,
        delta_y,
        delta_z,
        y_rot,
        x_rot,
        on_ground,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
    })
}

pub(crate) fn parse_entity_teleport_packet(
    input: &[u8],
) -> Result<EntityTeleportObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity teleport entity id")?;
    let x = read_f64(&mut input, "entity teleport x")?;
    let y = read_f64(&mut input, "entity teleport y")?;
    let z = read_f64(&mut input, "entity teleport z")?;
    let delta_x = read_f64(&mut input, "entity teleport delta x")?;
    let delta_y = read_f64(&mut input, "entity teleport delta y")?;
    let delta_z = read_f64(&mut input, "entity teleport delta z")?;
    let y_rot = read_f32(&mut input, "entity teleport y rotation")?;
    let x_rot = read_f32(&mut input, "entity teleport x rotation")?;
    let relative_flags = read_i32(&mut input, "entity teleport relative flags")?;
    let on_ground = read_byte(&mut input, "entity teleport on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("entity teleport trailing payload"));
    }
    Ok(EntityTeleportObservation {
        entity_id,
        x,
        y,
        z,
        delta_x,
        delta_y,
        delta_z,
        y_rot,
        x_rot,
        relative_flags,
        on_ground,
        motion_before: None,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
        applied_motion: None,
    })
}

pub(crate) fn parse_move_entity_pos_packet(
    input: &[u8],
) -> Result<EntityMoveObservation, ConnectionError> {
    parse_move_entity_packet(input, true, false)
}

pub(crate) fn parse_move_entity_pos_rot_packet(
    input: &[u8],
) -> Result<EntityMoveObservation, ConnectionError> {
    parse_move_entity_packet(input, true, true)
}

pub(crate) fn parse_move_entity_rot_packet(
    input: &[u8],
) -> Result<EntityMoveObservation, ConnectionError> {
    parse_move_entity_packet(input, false, true)
}

fn parse_move_entity_packet(
    input: &[u8],
    has_position: bool,
    has_rotation: bool,
) -> Result<EntityMoveObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "move entity id")?;
    let (delta_x, delta_y, delta_z) = if has_position {
        (
            read_i16(&mut input, "move entity delta x")?,
            read_i16(&mut input, "move entity delta y")?,
            read_i16(&mut input, "move entity delta z")?,
        )
    } else {
        (0, 0, 0)
    };
    let (y_rot, x_rot) = if has_rotation {
        (
            Some(decode_packed_degrees(read_byte(
                &mut input,
                "move entity y rotation",
            )?)),
            Some(decode_packed_degrees(read_byte(
                &mut input,
                "move entity x rotation",
            )?)),
        )
    } else {
        (None, None)
    };
    let on_ground = read_byte(&mut input, "move entity on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("move entity trailing payload"));
    }
    Ok(EntityMoveObservation {
        entity_id,
        delta_x,
        delta_y,
        delta_z,
        y_rot,
        x_rot,
        has_position,
        has_rotation,
        on_ground,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
    })
}

pub(crate) fn parse_remove_mob_effect_packet(
    input: &[u8],
) -> Result<MobEffectRemoveObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "remove mob effect entity id")?;
    let effect_registry_id = read_varint(&mut input, "remove mob effect registry id")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("remove mob effect trailing payload"));
    }
    Ok(MobEffectRemoveObservation {
        entity_id,
        effect_registry_id,
        applied_to_world: false,
    })
}

pub(crate) fn parse_update_mob_effect_packet(
    input: &[u8],
) -> Result<MobEffectUpdateObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "update mob effect entity id")?;
    let effect_registry_id = read_varint(&mut input, "update mob effect registry id")?;
    let amplifier = read_varint(&mut input, "update mob effect amplifier")?;
    let duration_ticks = read_varint(&mut input, "update mob effect duration")?;
    let flags = read_byte(&mut input, "update mob effect flags")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("update mob effect trailing payload"));
    }
    Ok(MobEffectUpdateObservation {
        entity_id,
        effect_registry_id,
        amplifier,
        duration_ticks,
        flags,
        applied_to_world: false,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedItemStackObservation {
    item_count: Option<i32>,
    item_registry_id: Option<i32>,
    positive_component_type_ids: Vec<i32>,
    negative_component_type_ids: Vec<i32>,
    patch_bytes: Vec<u8>,
}

impl ParsedItemStackObservation {
    fn client_state(&self) -> Option<ClientItemStackState> {
        Some(
            ClientItemStackState::from_wire(
                self.item_count?,
                self.item_registry_id?,
                self.positive_component_type_ids.clone(),
                self.negative_component_type_ids.clone(),
            )
            .with_patch_bytes(self.patch_bytes.clone()),
        )
    }
}

fn parse_optional_item_stack(
    input: &mut &[u8],
    name: &'static str,
) -> Result<ParsedItemStackObservation, ConnectionError> {
    let item_count = read_varint(input, name)?;
    if item_count <= 0 {
        return Ok(ParsedItemStackObservation {
            item_count: None,
            item_registry_id: None,
            positive_component_type_ids: Vec::new(),
            negative_component_type_ids: Vec::new(),
            patch_bytes: Vec::new(),
        });
    }
    let item_registry_id = read_varint(input, name)?;
    let patch_start = *input;
    let patch =
        voxel_game::item_components::read_patch(input).map_err(|_| ConnectionError::Codec(name))?;
    let positive_component_type_ids = patch.added.iter().map(|(id, _)| *id).collect();
    let negative_component_type_ids = patch.removed;
    let patch_bytes = patch_start[..patch_start.len() - input.len()].to_vec();
    Ok(ParsedItemStackObservation {
        item_count: Some(item_count),
        item_registry_id: Some(item_registry_id),
        positive_component_type_ids,
        negative_component_type_ids,
        patch_bytes,
    })
}

/// An item stack as an entity's data holds it (`ItemStack.OPTIONAL_STREAM_CODEC`
/// at the start of `value`): none when the field is empty or cannot be read.
pub(crate) fn parse_entity_item_stack(value: &[u8]) -> Option<ClientItemStackState> {
    let mut input = value;
    parse_optional_item_stack(&mut input, "entity item stack")
        .ok()?
        .client_state()
}

pub(crate) fn parse_container_set_content_packet(
    input: &[u8],
) -> Result<ContainerSetContentObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container content container id")?;
    let state_id = read_varint(&mut input, "container content state id")?;
    let item_count = read_count(&mut input, "container content item count")?;
    let mut hotbar_slot_zero_item_count = None;
    let mut hotbar_slot_zero_item_registry_id = None;
    let mut hotbar_slot_zero_positive_component_type_ids = Vec::new();
    let mut hotbar_slot_zero_negative_component_type_ids = Vec::new();
    let mut main_inventory_slot_zero_item_count = None;
    let mut main_inventory_slot_zero_item_registry_id = None;
    let mut main_inventory_slot_zero_positive_component_type_ids = Vec::new();
    let mut main_inventory_slot_zero_negative_component_type_ids = Vec::new();
    let mut items = Vec::with_capacity(item_count);
    for slot in 0..item_count {
        let item = parse_optional_item_stack(&mut input, "container content item stack")?;
        let client_state = item.client_state();
        if slot == 36 {
            hotbar_slot_zero_item_count = item.item_count;
            hotbar_slot_zero_item_registry_id = item.item_registry_id;
            hotbar_slot_zero_positive_component_type_ids = item.positive_component_type_ids.clone();
            hotbar_slot_zero_negative_component_type_ids = item.negative_component_type_ids.clone();
        } else if slot == 9 {
            main_inventory_slot_zero_item_count = item.item_count;
            main_inventory_slot_zero_item_registry_id = item.item_registry_id;
            main_inventory_slot_zero_positive_component_type_ids =
                item.positive_component_type_ids.clone();
            main_inventory_slot_zero_negative_component_type_ids =
                item.negative_component_type_ids.clone();
        }
        items.push(client_state);
    }
    let carried_item = parse_optional_item_stack(&mut input, "container content carried item")?;
    let carried_client_state = carried_item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container content trailing payload"));
    }
    Ok(ContainerSetContentObservation {
        container_id,
        state_id,
        item_count,
        items,
        hotbar_slot_zero_item_count,
        hotbar_slot_zero_item_registry_id,
        hotbar_slot_zero_positive_component_type_ids,
        hotbar_slot_zero_negative_component_type_ids,
        main_inventory_slot_zero_item_count,
        main_inventory_slot_zero_item_registry_id,
        main_inventory_slot_zero_positive_component_type_ids,
        main_inventory_slot_zero_negative_component_type_ids,
        carried_item_count: carried_item.item_count,
        carried_item_registry_id: carried_item.item_registry_id,
        carried_positive_component_type_ids: carried_item.positive_component_type_ids,
        carried_negative_component_type_ids: carried_item.negative_component_type_ids,
        carried_item: carried_client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_container_set_slot_packet(
    input: &[u8],
) -> Result<ContainerSetSlotObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container slot container id")?;
    let state_id = read_varint(&mut input, "container slot state id")?;
    let slot = read_i16(&mut input, "container slot index")?;
    let item = parse_optional_item_stack(&mut input, "container slot item stack")?;
    let client_state = item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container slot trailing payload"));
    }
    Ok(ContainerSetSlotObservation {
        container_id,
        state_id,
        slot,
        item_count: item.item_count,
        item_registry_id: item.item_registry_id,
        positive_component_type_ids: item.positive_component_type_ids,
        negative_component_type_ids: item.negative_component_type_ids,
        item: client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_cursor_item_packet(
    input: &[u8],
) -> Result<CursorItemObservation, ConnectionError> {
    let mut input = input;
    let item = parse_optional_item_stack(&mut input, "cursor item stack")?;
    let client_state = item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("cursor item trailing payload"));
    }
    Ok(CursorItemObservation {
        item_count: item.item_count,
        item_registry_id: item.item_registry_id,
        positive_component_type_ids: item.positive_component_type_ids,
        negative_component_type_ids: item.negative_component_type_ids,
        item: client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_set_player_inventory_packet(
    input: &[u8],
) -> Result<SetPlayerInventoryObservation, ConnectionError> {
    let mut input = input;
    let slot = read_varint(&mut input, "player inventory slot")?;
    let item = parse_optional_item_stack(&mut input, "player inventory item stack")?;
    let client_state = item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("player inventory trailing payload"));
    }
    Ok(SetPlayerInventoryObservation {
        slot,
        item_count: item.item_count,
        item_registry_id: item.item_registry_id,
        positive_component_type_ids: item.positive_component_type_ids,
        negative_component_type_ids: item.negative_component_type_ids,
        item: client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_system_chat_packet(
    input: &[u8],
) -> Result<SystemChatObservation, ConnectionError> {
    let mut input = input;
    let component = voxel_core::nbt::read_nbt(&mut input)
        .map_err(|_| ConnectionError::Codec("system chat component"))?;
    let translation_key = component.and_then(|fields| {
        fields.into_iter().find_map(|field| {
            if field.key == "translate" {
                match field.value {
                    voxel_core::nbt::FieldValue::Str(value) => Some(value),
                    _ => None,
                }
            } else {
                None
            }
        })
    });
    let overlay = read_byte(&mut input, "system chat overlay")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("system chat trailing payload"));
    }
    Ok(SystemChatObservation {
        overlay,
        translation_key,
    })
}

/// What the title packets, the action bar and the tab list ask the HUD to show.
#[derive(Debug, Clone, PartialEq)]
pub enum HudEvent {
    Title(voxel_game::text::Component),
    Subtitle(voxel_game::text::Component),
    /// `ClientboundSetTitlesAnimationPacket`, in ticks.
    Times {
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    },
    Clear {
        reset_times: bool,
    },
    ActionBar(voxel_game::text::Component),
    TabList {
        header: voxel_game::text::Component,
        footer: voxel_game::text::Component,
    },
    /// `ClientboundBossEventPacket`.
    Boss {
        id: u128,
        operation: BossOperation,
    },
}

/// What a boss event packet does to the bar `id`.
#[derive(Debug, Clone, PartialEq)]
pub enum BossOperation {
    Add {
        name: voxel_game::text::Component,
        progress: f32,
        /// `BossBarColor` ordinal: pink, blue, red, green, yellow, purple, white.
        color: u8,
        /// `BossBarOverlay` ordinal: progress, then notched 6, 10, 12, 20.
        overlay: u8,
        flags: u8,
    },
    Remove,
    Progress(f32),
    Name(voxel_game::text::Component),
    Style {
        color: u8,
        overlay: u8,
    },
    Properties(u8),
}

/// A `PlayerInfo` of the tab list: what the client keeps of a player.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerInfo {
    pub name: String,
    /// 0 survival, 1 creative, 2 adventure, 3 spectator.
    pub game_mode: i32,
    pub latency: i32,
    pub listed: bool,
    pub display_name: Option<voxel_game::text::Component>,
    pub list_order: i32,
}

/// One `ClientboundPlayerInfoUpdatePacket` action of a player.
#[derive(Debug, Clone, PartialEq)]
pub enum InfoAction {
    Add { name: String },
    GameMode(i32),
    Listed(bool),
    Latency(i32),
    DisplayName(Option<voxel_game::text::Component>),
    ListOrder(i32),
}

/// A player's id and what a player info packet does to it.
pub type PlayerActions = ((u64, u64), Vec<InfoAction>);

/// `ClientboundPlayerInfoUpdatePacket`: for each player (by its id), the
/// actions the packet's set names, in order. Chat sessions and hat flags are
/// read and dropped.
pub(crate) fn parse_player_info_update(
    input: &[u8],
) -> Result<Vec<PlayerActions>, ConnectionError> {
    let mut input = input;
    let actions = read_byte(&mut input, "player info actions")?;
    let count = read_varint(&mut input, "player info count")?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let uuid = voxel_core::binary::read_uuid(&mut input)
            .map_err(|_| ConnectionError::Codec("player info uuid"))?;
        let mut list = Vec::new();
        for bit in 0..8 {
            if actions & (1 << bit) == 0 {
                continue;
            }
            match bit {
                0 => {
                    let name = read_chat_string(&mut input, 16, "player info name")?;
                    let properties = read_varint(&mut input, "player info properties")?;
                    for _ in 0..properties {
                        read_chat_string(&mut input, 64, "property name")?;
                        read_chat_string(&mut input, 32767, "property value")?;
                        if read_bool(&mut input, "property signature flag")? {
                            read_chat_string(&mut input, 1024, "property signature")?;
                        }
                    }
                    list.push(InfoAction::Add { name });
                }
                1 => {
                    if read_bool(&mut input, "player info chat flag")? {
                        skip_bytes(&mut input, 16 + 8, "chat session")?;
                        for _ in 0..2 {
                            let length = read_varint(&mut input, "chat key length")?;
                            skip_bytes(&mut input, length.max(0) as usize, "chat key")?;
                        }
                    }
                }
                2 => list.push(InfoAction::GameMode(read_varint(&mut input, "game mode")?)),
                3 => list.push(InfoAction::Listed(read_bool(&mut input, "listed")?)),
                4 => list.push(InfoAction::Latency(read_varint(&mut input, "latency")?)),
                5 => {
                    let name = if read_bool(&mut input, "display name flag")? {
                        Some(read_chat_component(&mut input, "display name")?)
                    } else {
                        None
                    };
                    list.push(InfoAction::DisplayName(name));
                }
                6 => list.push(InfoAction::ListOrder(read_varint(
                    &mut input,
                    "list order",
                )?)),
                _ => {
                    read_bool(&mut input, "hat")?;
                }
            }
        }
        entries.push(((uuid.most, uuid.least), list));
    }
    Ok(entries)
}

/// `ClientboundPlayerInfoRemovePacket`.
pub(crate) fn parse_player_info_remove(input: &[u8]) -> Result<Vec<(u64, u64)>, ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "player info remove count")?;
    let mut ids = Vec::new();
    for _ in 0..count {
        let uuid = voxel_core::binary::read_uuid(&mut input)
            .map_err(|_| ConnectionError::Codec("player info remove uuid"))?;
        ids.push((uuid.most, uuid.least));
    }
    Ok(ids)
}

/// `ClientboundCommandSuggestionsPacket`: the request it answers, the range
/// of the typed text the matches replace, and the matches.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandSuggestions {
    pub id: i32,
    pub start: usize,
    pub length: usize,
    pub matches: Vec<String>,
}

pub(crate) fn parse_command_suggestions(
    input: &[u8],
) -> Result<CommandSuggestions, ConnectionError> {
    let mut input = input;
    let id = read_varint(&mut input, "suggestion id")?;
    let start = read_varint(&mut input, "suggestion start")?.max(0) as usize;
    let length = read_varint(&mut input, "suggestion length")?.max(0) as usize;
    let count = read_varint(&mut input, "suggestion count")?;
    let mut matches = Vec::new();
    for _ in 0..count {
        matches.push(read_chat_string(&mut input, 32767, "suggestion text")?);
        // The tooltip is read and dropped.
        if read_bool(&mut input, "suggestion tooltip flag")? {
            read_chat_component(&mut input, "suggestion tooltip")?;
        }
    }
    Ok(CommandSuggestions {
        id,
        start,
        length,
        matches,
    })
}

/// The title, action bar and tab list packets, by id.
pub(crate) fn parse_hud_packet(id: i32, input: &[u8]) -> Result<Option<HudEvent>, ConnectionError> {
    let mut input = input;
    let input = &mut input;
    Ok(Some(match id {
        PLAY_CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID => {
            HudEvent::Title(read_chat_component(input, "title")?)
        }
        PLAY_CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID => {
            HudEvent::Subtitle(read_chat_component(input, "subtitle")?)
        }
        PLAY_CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID => {
            HudEvent::ActionBar(read_chat_component(input, "action bar")?)
        }
        PLAY_CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID => HudEvent::Times {
            fade_in: read_i32(input, "title fade in")?,
            stay: read_i32(input, "title stay")?,
            fade_out: read_i32(input, "title fade out")?,
        },
        PLAY_CLIENTBOUND_CLEAR_TITLES_PACKET_ID => HudEvent::Clear {
            reset_times: read_bool(input, "clear titles reset")?,
        },
        PLAY_CLIENTBOUND_TAB_LIST_PACKET_ID => HudEvent::TabList {
            header: read_chat_component(input, "tab list header")?,
            footer: read_chat_component(input, "tab list footer")?,
        },
        PLAY_CLIENTBOUND_BOSS_EVENT_PACKET_ID => parse_boss_event(input)?,
        _ => return Ok(None),
    }))
}

/// `ClientboundCommandsPacket`: the entries, linked into a tree.
pub fn parse_commands_packet(
    input: &[u8],
) -> Result<voxel_game::commands::CommandTree, ConnectionError> {
    use voxel_game::commands::{CommandTree, NodeKind, NodeStub};
    let mut input = input;
    let input = &mut input;
    let count = read_count(input, "command node count")?;
    let mut stubs = Vec::with_capacity(count.min(4096));
    for _ in 0..count {
        let flags = read_byte(input, "command node flags")?;
        let children = (0..read_count(input, "command children count")?)
            .map(|_| read_count(input, "command child"))
            .collect::<Result<Vec<_>, _>>()?;
        let redirect = if flags & 8 != 0 {
            Some(read_count(input, "command redirect")?)
        } else {
            None
        };
        let mut custom = false;
        let mut type_id = 0;
        let kind = match flags & 3 {
            1 => NodeKind::Literal(read_chat_string(input, 32767, "command literal")?),
            2 => {
                let name = read_chat_string(input, 32767, "command argument name")?;
                let id = read_varint(input, "command argument type")?;
                let kind = read_argument_kind(input, id)?;
                type_id = u8::try_from(id).unwrap_or(u8::MAX);
                if flags & 16 != 0 {
                    read_chat_string(input, 32767, "command suggestion id")?;
                    custom = true;
                }
                NodeKind::Argument { name, kind }
            }
            _ => NodeKind::Root,
        };
        stubs.push(NodeStub {
            kind,
            children,
            redirect,
            executable: flags & 4 != 0,
            custom_suggestions: custom,
            type_id,
        });
    }
    let root = read_count(input, "command root index")?;
    CommandTree::build(stubs, root).ok_or(ConnectionError::Codec("command tree"))
}

#[expect(clippy::too_many_lines, reason = "one arm for each argument type id")]
/// An argument type by registry id with its parameters
/// (`ArgumentTypeInfo.deserializeFromNetwork`).
fn read_argument_kind(
    input: &mut &[u8],
    id: i32,
) -> Result<voxel_game::commands::ArgumentKind, ConnectionError> {
    use voxel_game::commands::{ArgumentKind as K, StringType};
    let name = "command argument parameters";
    Ok(match id {
        0 => K::Bool,
        1 => {
            let flags = read_byte(input, name)?;
            let min = if flags & 1 != 0 {
                read_f32(input, name)?
            } else {
                -f32::MAX
            };
            let max = if flags & 2 != 0 {
                read_f32(input, name)?
            } else {
                f32::MAX
            };
            K::Float { min, max }
        }
        2 => {
            let flags = read_byte(input, name)?;
            let min = if flags & 1 != 0 {
                read_f64(input, name)?
            } else {
                -f64::MAX
            };
            let max = if flags & 2 != 0 {
                read_f64(input, name)?
            } else {
                f64::MAX
            };
            K::Double { min, max }
        }
        3 => {
            let flags = read_byte(input, name)?;
            let min = if flags & 1 != 0 {
                read_i32(input, name)?
            } else {
                i32::MIN
            };
            let max = if flags & 2 != 0 {
                read_i32(input, name)?
            } else {
                i32::MAX
            };
            K::Integer { min, max }
        }
        4 => {
            let flags = read_byte(input, name)?;
            let min = if flags & 1 != 0 {
                read_i64(input, name)?
            } else {
                i64::MIN
            };
            let max = if flags & 2 != 0 {
                read_i64(input, name)?
            } else {
                i64::MAX
            };
            K::Long { min, max }
        }
        5 => K::String(match read_varint(input, name)? {
            0 => StringType::Word,
            1 => StringType::Phrase,
            _ => StringType::Greedy,
        }),
        6 => {
            let flags = read_byte(input, name)?;
            K::Entity {
                single: flags & 1 != 0,
                players_only: flags & 2 != 0,
            }
        }
        // Game profiles are players; score holders also take a bare name.
        7 => K::Entity {
            single: false,
            players_only: true,
        },
        31 => {
            read_byte(input, name)?;
            K::ScoreHolder
        }
        12..=15 | 18 | 19 | 21..=23 | 37 | 52..=55 => K::Balanced,
        8 => K::BlockPos,
        9 => K::ColumnPos,
        10 => K::Vec3,
        11 => K::Vec2,
        16 => K::TeamColor,
        17 => K::HexColor,
        20 => K::Greedy,
        24 | 33 => K::Unquoted,
        25 | 27 | 34 | 35 | 39 | 40 | 49..=51 => K::Word,
        26 => K::Operation,
        28 => K::Angle,
        29 => K::Rotation,
        30 => K::ScoreboardSlot,
        32 => K::Swizzle,
        36 | 41 => K::Identifier,
        38 => K::EntityAnchor,
        42 => K::GameMode,
        43 => K::Time {
            min: read_i32(input, name)?,
        },
        // Resources name the registry they come from.
        44..=48 => {
            read_chat_string(input, 32767, name)?;
            K::Word
        }
        56 => K::Uuid,
        _ => return Err(ConnectionError::Codec("command argument type")),
    })
}

/// `ClientboundDisconnectPacket`: the reason the server gives.
pub(crate) fn read_disconnect_reason(
    input: &mut &[u8],
) -> Result<voxel_game::text::Component, ConnectionError> {
    read_chat_component(input, "disconnect reason")
}

/// `ClientboundPlayerCombatKillPacket`: who died and the message for them.
pub(crate) fn parse_player_combat_kill_packet(
    input: &[u8],
) -> Result<(i32, voxel_game::text::Component), ConnectionError> {
    let mut input = input;
    let id = read_varint(&mut input, "combat kill player id")?;
    Ok((id, read_chat_component(&mut input, "combat kill message")?))
}

/// `ClientboundLevelEventPacket`: an event of the level at a place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelEvent {
    pub event: i32,
    pub pos: (i32, i32, i32),
    pub data: i32,
    pub global: bool,
}

/// `ClientboundLevelEventPacket`, read from the front of `input`.
pub fn read_level_event_packet(input: &mut &[u8]) -> Result<LevelEvent, ConnectionError> {
    let event = read_i32(input, "level event")?;
    let pos = voxel_core::binary::BlockPos::unpack(read_i64(input, "level event position")? as u64);
    let data = read_i32(input, "level event data")?;
    let global = read_bool(input, "level event global")?;
    Ok(LevelEvent {
        event,
        pos: (pos.x, pos.y, pos.z),
        data,
        global,
    })
}

/// `SoundEvent.STREAM_CODEC`: a registry place plus one, or zero and the event whole.
fn read_sound_holder(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundHolder, ConnectionError> {
    use voxel_game::sounds::SoundHolder;
    let holder = read_varint(input, "sound holder")?;
    if holder == 0 {
        let location = read_identifier_string(input, "sound location")?;
        let fixed_range = if read_bool(input, "sound range present")? {
            Some(read_f32(input, "sound range")?)
        } else {
            None
        };
        return Ok(SoundHolder::Direct {
            location,
            fixed_range,
        });
    }
    let id = holder - 1;
    if voxel_data::sound_event::name(id).is_none() {
        return Err(ConnectionError::Codec("unknown sound event id"));
    }
    Ok(SoundHolder::Registered(id))
}

/// `FriendlyByteBuf.readEnum(SoundSource.class)`.
fn read_sound_source(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundSource, ConnectionError> {
    let ordinal = read_varint(input, "sound source")?;
    voxel_game::sounds::SoundSource::from_ordinal(ordinal)
        .ok_or(ConnectionError::Codec("unknown sound source"))
}

/// `ClientboundSoundPacket`, read from the front of `input` (what follows is the caller's).
pub fn read_sound_packet(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundPacket, ConnectionError> {
    let sound = read_sound_holder(input)?;
    let source = read_sound_source(input)?;
    // The packet carries eighths of a block as ints; the reference divides as a float.
    let place = |input: &mut &[u8], name| -> Result<f64, ConnectionError> {
        Ok(f64::from(read_i32(input, name)? as f32 / 8.0))
    };
    let position = (
        place(input, "sound x")?,
        place(input, "sound y")?,
        place(input, "sound z")?,
    );
    Ok(voxel_game::sounds::SoundPacket {
        sound,
        source,
        position,
        volume: read_f32(input, "sound volume")?,
        pitch: read_f32(input, "sound pitch")?,
        seed: read_i64(input, "sound seed")?,
    })
}

/// `ClientboundSoundEntityPacket`, read from the front of `input`.
pub fn read_sound_entity_packet(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundEntityPacket, ConnectionError> {
    Ok(voxel_game::sounds::SoundEntityPacket {
        sound: read_sound_holder(input)?,
        source: read_sound_source(input)?,
        entity_id: read_varint(input, "sound entity id")?,
        volume: read_f32(input, "sound volume")?,
        pitch: read_f32(input, "sound pitch")?,
        seed: read_i64(input, "sound seed")?,
    })
}

/// `ClientboundStopSoundPacket`, read from the front of `input`.
pub fn read_stop_sound_packet(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::StopSoundPacket, ConnectionError> {
    let flags = read_byte(input, "stop sound flags")?;
    let source = if flags & 1 > 0 {
        Some(read_sound_source(input)?)
    } else {
        None
    };
    let name = if flags & 2 > 0 {
        Some(read_identifier_string(input, "stop sound name")?)
    } else {
        None
    };
    Ok(voxel_game::sounds::StopSoundPacket { name, source })
}

/// The scoreboard packets, by id.
pub(crate) fn parse_scoreboard_packet(
    id: i32,
    input: &[u8],
) -> Result<Option<voxel_game::scoreboard::ScoreboardEvent>, ConnectionError> {
    use voxel_game::scoreboard::{ScoreboardEvent, TeamParameters};
    let mut input = input;
    let input = &mut input;
    let string = |input: &mut &[u8], name| read_chat_string(input, 32767, name);
    Ok(Some(match id {
        PLAY_CLIENTBOUND_SET_OBJECTIVE_PACKET_ID => {
            let name = string(input, "objective name")?;
            let method = read_byte(input, "objective method")?;
            if method == 0 || method == 2 {
                ScoreboardEvent::Objective {
                    name,
                    method,
                    display_name: read_chat_component(input, "objective display name")?,
                    hearts: read_varint(input, "objective render type")? == 1,
                    number_format: read_number_format(input)?,
                }
            } else {
                ScoreboardEvent::Objective {
                    name,
                    method,
                    display_name: voxel_game::text::Component::empty(),
                    hearts: false,
                    number_format: None,
                }
            }
        }
        PLAY_CLIENTBOUND_SET_SCORE_PACKET_ID => ScoreboardEvent::Score {
            owner: string(input, "score owner")?,
            objective: string(input, "score objective")?,
            value: read_varint(input, "score value")?,
            display: if read_bool(input, "score display present")? {
                Some(read_chat_component(input, "score display")?)
            } else {
                None
            },
            number_format: read_number_format(input)?,
        },
        PLAY_CLIENTBOUND_RESET_SCORE_PACKET_ID => ScoreboardEvent::ResetScore {
            owner: string(input, "reset score owner")?,
            objective: if read_bool(input, "reset score objective present")? {
                Some(string(input, "reset score objective")?)
            } else {
                None
            },
        },
        PLAY_CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID => {
            let slot = read_varint(input, "display slot")?;
            let name = string(input, "display objective")?;
            ScoreboardEvent::Display {
                slot: u8::try_from(slot).unwrap_or(u8::MAX),
                objective: (!name.is_empty()).then_some(name),
            }
        }
        PLAY_CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID => {
            let name = string(input, "team name")?;
            let method = read_byte(input, "team method")?;
            let parameters = if method == 0 || method == 2 {
                let display_name = read_chat_component(input, "team display name")?;
                let prefix = read_chat_component(input, "team prefix")?;
                let suffix = read_chat_component(input, "team suffix")?;
                read_varint(input, "team name tag visibility")?;
                read_varint(input, "team collision rule")?;
                let colour = if read_bool(input, "team colour present")? {
                    Some(read_varint(input, "team colour")?.clamp(0, 15) as u8)
                } else {
                    None
                };
                read_byte(input, "team options")?;
                Some(TeamParameters {
                    display_name,
                    prefix,
                    suffix,
                    colour,
                })
            } else {
                None
            };
            let mut players = Vec::new();
            if matches!(method, 0 | 3 | 4) {
                for _ in 0..read_count(input, "team player count")? {
                    players.push(string(input, "team player")?);
                }
            }
            ScoreboardEvent::Team {
                name,
                method,
                parameters,
                players,
            }
        }
        _ => return Ok(None),
    }))
}

/// `NumberFormatTypes.OPTIONAL_STREAM_CODEC`: absent, or a type (blank,
/// styled, fixed) and what it holds.
fn read_number_format(
    input: &mut &[u8],
) -> Result<Option<voxel_game::scoreboard::NumberFormat>, ConnectionError> {
    use voxel_game::scoreboard::NumberFormat;
    if !read_bool(input, "number format present")? {
        return Ok(None);
    }
    Ok(Some(match read_varint(input, "number format type")? {
        0 => NumberFormat::Blank,
        1 => NumberFormat::Styled(
            voxel_core::nbt::read_any_nbt(input)
                .ok()
                .flatten()
                .and_then(|tag| voxel_game::text::Style::from_nbt(&tag))
                .ok_or(ConnectionError::Codec("number format style"))?,
        ),
        2 => NumberFormat::Fixed(read_chat_component(input, "number format value")?),
        _ => return Err(ConnectionError::Codec("number format type")),
    }))
}

/// `ClientboundBossEventPacket(RegistryFriendlyByteBuf)`.
fn parse_boss_event(input: &mut &[u8]) -> Result<HudEvent, ConnectionError> {
    let id = wire::take::<16>(input)
        .map(u128::from_be_bytes)
        .ok_or(ConnectionError::Codec("boss event id"))?;
    let operation = match read_varint(input, "boss event operation")? {
        0 => BossOperation::Add {
            name: read_chat_component(input, "boss bar name")?,
            progress: read_f32(input, "boss bar progress")?,
            color: boss_ordinal(input, 7, "boss bar color")?,
            overlay: boss_ordinal(input, 5, "boss bar overlay")?,
            flags: read_byte(input, "boss bar flags")?,
        },
        1 => BossOperation::Remove,
        2 => BossOperation::Progress(read_f32(input, "boss bar progress")?),
        3 => BossOperation::Name(read_chat_component(input, "boss bar name")?),
        4 => BossOperation::Style {
            color: boss_ordinal(input, 7, "boss bar color")?,
            overlay: boss_ordinal(input, 5, "boss bar overlay")?,
        },
        5 => BossOperation::Properties(read_byte(input, "boss bar flags")?),
        _ => return Err(ConnectionError::Codec("boss event operation")),
    };
    Ok(HudEvent::Boss { id, operation })
}

/// `ClientboundEntityEventPacket.STREAM_CODEC`: the entity's id and the
/// event (a byte).
/// `ClientboundAnimatePacket`: the entity and the action.
pub(crate) fn parse_animate_packet(input: &[u8]) -> Result<(i32, u8), ConnectionError> {
    let mut input = input;
    let id = read_varint(&mut input, "animate entity id")?;
    let action = read_byte(&mut input, "animate action")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("animate trailing payload"));
    }
    Ok((id, action))
}

pub(crate) fn parse_entity_event_packet(input: &[u8]) -> Result<(i32, i8), ConnectionError> {
    let mut input = input;
    let id = read_i32(&mut input, "entity event id")?;
    let event = *input
        .first()
        .ok_or(ConnectionError::Codec("entity event"))? as i8;
    Ok((id, event))
}

/// The entity a `ClientboundDamageEventPacket` or a
/// `ClientboundHurtAnimationPacket` names: both begin with its id as a
/// var int. The rest (the source, the yaw) is not read.
pub(crate) fn parse_entity_id_prefix(input: &[u8]) -> Result<i32, ConnectionError> {
    let mut input = input;
    voxel_core::wire::read_varint(&mut input).map_err(|_| ConnectionError::Codec("entity id"))
}

/// `ClientboundDamageEventPacket`: the entity hurt, and the registry id of the damage type that
/// hurt it (`None` when the packet carries the type itself instead of naming one, which a server
/// that syncs its registries does not do). The rest (the causes, the place) is not read.
pub(crate) fn parse_damage_event_packet(
    input: &[u8],
) -> Result<(i32, Option<usize>), ConnectionError> {
    let mut input = input;
    let id = read_varint(&mut input, "damage event entity id")?;
    // `ByteBufCodecs.holder`: the registry id plus one, or zero for a type written out.
    let holder = read_varint(&mut input, "damage event type")?;
    Ok((
        id,
        usize::try_from(holder)
            .ok()
            .and_then(|value| value.checked_sub(1)),
    ))
}

/// `ClientboundTrackedWaypointPacket.STREAM_CODEC`: the operation, then the
/// waypoint (`TrackedWaypoint.read`).
pub fn parse_waypoint_packet(
    input: &[u8],
) -> Result<
    (
        voxel_game::waypoint::WaypointOperation,
        voxel_game::waypoint::TrackedWaypoint,
    ),
    ConnectionError,
> {
    use voxel_game::waypoint::{
        TrackedWaypoint, WaypointIcon, WaypointId, WaypointKind, WaypointOperation,
    };
    let mut input = input;
    let input = &mut input;
    // `ByIdMap.continuous` with `WRAP`: an id past the end comes round.
    let operation = match read_varint(input, "waypoint operation")?.rem_euclid(3) {
        0 => WaypointOperation::Track,
        1 => WaypointOperation::Untrack,
        _ => WaypointOperation::Update,
    };
    let id = if read_bool(input, "waypoint id side")? {
        WaypointId::Uuid(
            wire::take::<16>(input)
                .map(u128::from_be_bytes)
                .ok_or(ConnectionError::Codec("waypoint uuid"))?,
        )
    } else {
        WaypointId::Name(read_chat_string(input, 32767, "waypoint name")?)
    };
    let style = read_identifier_string(input, "waypoint style")?;
    let colour = if read_bool(input, "waypoint colour present")? {
        let red = i32::from(read_byte(input, "waypoint red")?);
        let green = i32::from(read_byte(input, "waypoint green")?);
        let blue = i32::from(read_byte(input, "waypoint blue")?);
        Some(red << 16 | green << 8 | blue)
    } else {
        None
    };
    // `readEnum`: the ordinal, which must exist.
    let kind = match read_varint(input, "waypoint type")? {
        0 => WaypointKind::Empty,
        1 => WaypointKind::Position([
            read_varint(input, "waypoint x")?,
            read_varint(input, "waypoint y")?,
            read_varint(input, "waypoint z")?,
        ]),
        2 => WaypointKind::Chunk([
            read_varint(input, "waypoint chunk x")?,
            read_varint(input, "waypoint chunk z")?,
        ]),
        3 => WaypointKind::Azimuth(read_f32(input, "waypoint angle")?),
        _ => return Err(ConnectionError::Codec("waypoint type")),
    };
    let icon = WaypointIcon { style, colour };
    Ok((operation, TrackedWaypoint { id, icon, kind }))
}

/// An enum on the wire: its ordinal as a varint, below `count`.
fn boss_ordinal(input: &mut &[u8], count: i32, name: &'static str) -> Result<u8, ConnectionError> {
    let ordinal = read_varint(input, name)?;
    if (0..count).contains(&ordinal) {
        Ok(ordinal as u8)
    } else {
        Err(ConnectionError::Codec(name))
    }
}

/// How a chat type is named on the wire (`Holder<ChatType>`): by its
/// registry id, or written out.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatHolder {
    Registry(i32),
    Direct(voxel_game::chat::ChatDecoration),
}

/// `ChatType.Bound`: the type, the sender's name and the target's.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatBound {
    pub holder: ChatHolder,
    pub name: voxel_game::text::Component,
    pub target: Option<voxel_game::text::Component>,
}

/// A `MessageSignature`: 256 bytes.
pub type ChatSignature = Vec<u8>;

/// `MessageSignature.Packed`: an id in the signature cache, or the signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackedSignature {
    Cached(i32),
    Full(ChatSignature),
}

/// `FilterMask`: what the server hid of a message.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FilterMask {
    #[default]
    PassThrough,
    FullyFiltered,
    /// The bit set of hidden characters, as the words the network writes.
    Partial(Vec<u64>),
}

impl FilterMask {
    /// Whether character `index` is hidden.
    pub fn hides(&self, index: usize) -> bool {
        match self {
            Self::Partial(words) => words
                .get(index / 64)
                .is_some_and(|word| word >> (index % 64) & 1 == 1),
            _ => false,
        }
    }
}

/// `ClientboundPlayerChatPacket` as it arrives, its signatures not yet
/// looked up in the cache.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerChatPacket {
    pub global_index: i32,
    pub sender: (u64, u64),
    pub signature: Option<ChatSignature>,
    pub signed_text: String,
    pub timestamp_millis: i64,
    pub last_seen: Vec<PackedSignature>,
    pub unsigned: Option<voxel_game::text::Component>,
    pub filter: FilterMask,
    pub bound: ChatBound,
}

/// A chat packet as the client shows it.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatEvent {
    /// `ClientboundSystemChatPacket`: the action bar when `overlay`.
    System {
        content: voxel_game::text::Component,
        overlay: bool,
    },
    /// `ClientboundDisguisedChatPacket`.
    Disguised {
        content: voxel_game::text::Component,
        bound: ChatBound,
    },
    /// `ClientboundPlayerChatPacket`, its signatures looked up; whether the
    /// sender is a player the client knows and is the local player.
    Player {
        packet: PlayerChatPacket,
        sender_known: bool,
        sender_is_local: bool,
    },
    /// `ClientboundDeleteChatPacket`: the signature of the message to delete.
    Delete(ChatSignature),
}

fn read_chat_component(
    input: &mut &[u8],
    name: &'static str,
) -> Result<voxel_game::text::Component, ConnectionError> {
    voxel_core::nbt::read_any_nbt(input)
        .ok()
        .flatten()
        .and_then(|tag| voxel_game::text::Component::from_nbt(&tag))
        .ok_or(ConnectionError::Codec(name))
}

fn read_chat_string(
    input: &mut &[u8],
    max: usize,
    name: &'static str,
) -> Result<String, ConnectionError> {
    voxel_core::utf8::read_utf8(input, max)
        .map(|units| String::from_utf16_lossy(&units))
        .map_err(|_| ConnectionError::Codec(name))
}

/// `ChatTypeDecoration.STREAM_CODEC`.
fn read_chat_decoration(
    input: &mut &[u8],
) -> Result<voxel_game::chat::ChatDecoration, ConnectionError> {
    let translation_key = read_chat_string(input, 32767, "chat decoration key")?;
    let count = read_varint(input, "chat decoration parameters")?;
    let mut parameters = Vec::new();
    for _ in 0..count {
        let id = read_varint(input, "chat decoration parameter")?;
        parameters.push(
            voxel_game::chat::ChatParameter::from_id(id)
                .ok_or(ConnectionError::Codec("chat decoration parameter"))?,
        );
    }
    let style = voxel_core::nbt::read_any_nbt(input)
        .ok()
        .flatten()
        .and_then(|tag| voxel_game::text::Component::from_nbt(&tag))
        .map(|component| component.style)
        .ok_or(ConnectionError::Codec("chat decoration style"))?;
    Ok(voxel_game::chat::ChatDecoration {
        translation_key,
        parameters,
        style,
    })
}

/// `ChatType.Bound.STREAM_CODEC`: a holder whose id 0 is a chat type written
/// out (its chat decoration, then its narration), then the names.
fn read_chat_bound(input: &mut &[u8]) -> Result<ChatBound, ConnectionError> {
    let holder = match read_varint(input, "chat type")? {
        0 => {
            let chat = read_chat_decoration(input)?;
            read_chat_decoration(input)?;
            ChatHolder::Direct(chat)
        }
        id => ChatHolder::Registry(id - 1),
    };
    let name = read_chat_component(input, "chat sender name")?;
    let target = if read_bool(input, "chat target flag")? {
        Some(read_chat_component(input, "chat target name")?)
    } else {
        None
    };
    Ok(ChatBound {
        holder,
        name,
        target,
    })
}

fn skip_bytes(input: &mut &[u8], count: usize, name: &'static str) -> Result<(), ConnectionError> {
    if input.len() < count {
        return Err(ConnectionError::Codec(name));
    }
    *input = &input[count..];
    Ok(())
}

/// `ClientboundSystemChatPacket`.
pub(crate) fn parse_system_chat_event(input: &[u8]) -> Result<ChatEvent, ConnectionError> {
    let mut input = input;
    let content = read_chat_component(&mut input, "system chat component")?;
    let overlay = read_bool(&mut input, "system chat overlay")?;
    Ok(ChatEvent::System { content, overlay })
}

/// `ClientboundDisguisedChatPacket`.
pub(crate) fn parse_disguised_chat_event(input: &[u8]) -> Result<ChatEvent, ConnectionError> {
    let mut input = input;
    let content = read_chat_component(&mut input, "disguised chat message")?;
    let bound = read_chat_bound(&mut input)?;
    Ok(ChatEvent::Disguised { content, bound })
}

fn read_packed_signature(input: &mut &[u8]) -> Result<PackedSignature, ConnectionError> {
    match read_varint(input, "packed signature")? {
        0 => Ok(PackedSignature::Full(read_signature(input)?)),
        id => Ok(PackedSignature::Cached(id - 1)),
    }
}

fn read_signature(input: &mut &[u8]) -> Result<ChatSignature, ConnectionError> {
    if input.len() < 256 {
        return Err(ConnectionError::Codec("signature"));
    }
    let (signature, rest) = input.split_at(256);
    *input = rest;
    Ok(signature.to_vec())
}

/// `ClientboundPlayerChatPacket`.
pub(crate) fn parse_player_chat_packet(input: &[u8]) -> Result<PlayerChatPacket, ConnectionError> {
    let mut input = input;
    let global_index = read_varint(&mut input, "chat global index")?;
    let sender = voxel_core::binary::read_uuid(&mut input)
        .map_err(|_| ConnectionError::Codec("chat sender"))?;
    read_varint(&mut input, "chat index")?;
    let signature = if read_bool(&mut input, "chat signature flag")? {
        Some(read_signature(&mut input)?)
    } else {
        None
    };
    let signed_text = read_chat_string(&mut input, 256, "chat content")?;
    let timestamp_millis = read_i64(&mut input, "chat timestamp")?;
    skip_bytes(&mut input, 8, "chat salt")?;
    let count = read_varint(&mut input, "chat last seen")?;
    let mut last_seen = Vec::new();
    for _ in 0..count {
        last_seen.push(read_packed_signature(&mut input)?);
    }
    let unsigned = if read_bool(&mut input, "chat unsigned flag")? {
        Some(read_chat_component(&mut input, "chat unsigned content")?)
    } else {
        None
    };
    // `FilterMask`: pass through, fully filtered, or partly (with a bit set).
    let filter = match read_varint(&mut input, "chat filter mask")? {
        1 => FilterMask::FullyFiltered,
        2 => {
            let words = read_varint(&mut input, "chat filter bits")?.max(0) as usize;
            let mut bits = Vec::with_capacity(words);
            for _ in 0..words {
                bits.push(read_i64(&mut input, "chat filter word")? as u64);
            }
            FilterMask::Partial(bits)
        }
        _ => FilterMask::PassThrough,
    };
    let bound = read_chat_bound(&mut input)?;
    Ok(PlayerChatPacket {
        global_index,
        sender: (sender.most, sender.least),
        signature,
        signed_text,
        timestamp_millis,
        last_seen,
        unsigned,
        filter,
        bound,
    })
}

/// `ClientboundDeleteChatPacket`.
pub(crate) fn parse_delete_chat_packet(input: &[u8]) -> Result<PackedSignature, ConnectionError> {
    let mut input = input;
    read_packed_signature(&mut input)
}

/// `MessageSignatureCache`: the last 128 signatures, by place, which the
/// packets refer to by number.
#[derive(Debug, Clone)]
pub struct SignatureCache {
    entries: Vec<Option<ChatSignature>>,
}

impl Default for SignatureCache {
    fn default() -> Self {
        Self {
            entries: vec![None; 128],
        }
    }
}

impl SignatureCache {
    /// `MessageSignature.Packed.unpack`: `None` for a number nothing holds.
    pub fn unpack(&self, packed: &PackedSignature) -> Option<ChatSignature> {
        match packed {
            PackedSignature::Full(signature) => Some(signature.clone()),
            PackedSignature::Cached(id) => usize::try_from(*id)
                .ok()
                .and_then(|id| self.entries.get(id))
                .and_then(Clone::clone),
        }
    }

    /// `MessageSignatureCache.push`: the new signatures go in at the front,
    /// the old ones they do not repeat shifted back.
    pub fn push(&mut self, mut queue: std::collections::VecDeque<ChatSignature>) {
        let new: std::collections::HashSet<ChatSignature> = queue.iter().cloned().collect();
        for index in 0..self.entries.len() {
            if queue.is_empty() {
                break;
            }
            let old = self.entries[index].take();
            self.entries[index] = queue.pop_back();
            if let Some(old) = old
                && !new.contains(&old)
            {
                queue.push_front(old);
            }
        }
    }
}

/// `ServerboundChatPacket` for a session without a chat key: the message, the
/// time, a salt, no signature and an empty acknowledgement (offset 0, no
/// messages, checksum 1).
pub(crate) fn encode_chat(message: &str, millis: i64, salt: i64, output: &mut Vec<u8>) {
    let units: Vec<u16> = message.encode_utf16().collect();
    let _ = voxel_core::utf8::write_utf8(output, &units, 256);
    output.extend_from_slice(&millis.to_be_bytes());
    output.extend_from_slice(&salt.to_be_bytes());
    output.push(0);
    voxel_core::wire::write_varint(0, output);
    output.extend_from_slice(&[0, 0, 0]);
    output.push(1);
}

fn read_lp_vec3(input: &mut &[u8], name: &'static str) -> Result<(f64, f64, f64), ConnectionError> {
    let movement =
        voxel_core::binary::read_lp_vec3(input).map_err(|_| ConnectionError::Codec(name))?;
    Ok((movement.x, movement.y, movement.z))
}

fn decode_packed_degrees(value: u8) -> f32 {
    i8::from_ne_bytes([value]) as f32 * 360.0 / 256.0
}

fn read_nbt_bytes(
    input: &mut &[u8],
    name: &'static str,
) -> Result<Option<Vec<u8>>, ConnectionError> {
    let before = *input;
    let value = voxel_core::nbt::read_nbt(input).map_err(|_| ConnectionError::Codec(name))?;
    let consumed = before.len() - input.len();
    Ok(value.map(|_| before[..consumed].to_vec()))
}

pub(crate) fn parse_chunk_packet(
    mut input: &[u8],
) -> Result<ChunkPacketObservation, ConnectionError> {
    let x = read_i32(&mut input, "chunk x")?;
    let z = read_i32(&mut input, "chunk z")?;

    let heightmap_count = read_count(&mut input, "heightmap count")?;
    let mut heightmap_observations = Vec::with_capacity(heightmap_count);
    for _ in 0..heightmap_count {
        let type_id = read_varint(&mut input, "heightmap type")?;
        let word_count = read_long_array(&mut input, "heightmap data")?.len();
        heightmap_observations.push(HeightmapObservation {
            type_id,
            word_count,
        });
    }

    let section_data =
        read_length_prefixed_bytes(&mut input, MAX_CHUNK_DATA_LENGTH, "chunk section data")?;
    let section_data_length = section_data.len();
    let section_observations = parse_section_data(section_data)?;

    let block_entity_count = read_count(&mut input, "block entity count")?;
    let mut block_entities = Vec::with_capacity(block_entity_count);
    for _ in 0..block_entity_count {
        let packed_xz = read_byte(&mut input, "block entity packed xz")?;
        let y = i32::from(read_i16(&mut input, "block entity y")?);
        let type_id = read_varint(&mut input, "block entity type")?;
        let tag = read_nbt_bytes(&mut input, "block entity nbt")?;
        let local_x = i32::from(packed_xz >> 4);
        let local_z = i32::from(packed_xz & 0x0f);
        let x = x.wrapping_mul(16).wrapping_add(local_x);
        let z = z.wrapping_mul(16).wrapping_add(local_z);
        let tag_length = tag.as_ref().map_or(0, Vec::len);
        block_entities.push(BlockEntityObservation {
            x,
            y,
            z,
            type_id,
            tag_length,
            applied_to_world: false,
            tag,
        });
    }

    let light_data = parse_light_data(&mut input)?;
    let sky_mask_word_count = light_data.sky_mask.to_longs().len();
    let block_mask_word_count = light_data.block_mask.to_longs().len();
    let empty_sky_mask_word_count = light_data.empty_sky_mask.to_longs().len();
    let empty_block_mask_word_count = light_data.empty_block_mask.to_longs().len();
    let sky_update_lengths = light_data
        .sky_updates
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();
    let block_update_lengths = light_data
        .block_updates
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();

    if !input.is_empty() {
        return Err(ConnectionError::Codec("chunk trailing payload"));
    }

    Ok(ChunkPacketObservation {
        x,
        z,
        heightmap_count,
        heightmap_observations,
        section_data_length,
        section_observations,
        block_entity_count,
        block_entities,
        sky_mask_word_count,
        block_mask_word_count,
        empty_sky_mask_word_count,
        empty_block_mask_word_count,
        sky_update_count: sky_update_lengths.len(),
        sky_update_lengths,
        block_update_count: block_update_lengths.len(),
        block_update_lengths,
        light_data,
    })
}

pub(crate) fn parse_light_update_packet(
    mut input: &[u8],
) -> Result<LightUpdateObservation, ConnectionError> {
    let x = read_varint(&mut input, "light update chunk x")?;
    let z = read_varint(&mut input, "light update chunk z")?;
    let light_data = parse_light_data(&mut input)?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("light update trailing payload"));
    }
    let sky_mask_word_count = light_data.sky_mask.to_longs().len();
    let block_mask_word_count = light_data.block_mask.to_longs().len();
    let empty_sky_mask_word_count = light_data.empty_sky_mask.to_longs().len();
    let empty_block_mask_word_count = light_data.empty_block_mask.to_longs().len();
    let sky_update_lengths = light_data
        .sky_updates
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();
    let block_update_lengths = light_data
        .block_updates
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();
    Ok(LightUpdateObservation {
        x,
        z,
        sky_mask_word_count,
        block_mask_word_count,
        empty_sky_mask_word_count,
        empty_block_mask_word_count,
        sky_update_lengths,
        block_update_lengths,
        applied_sky_sections: 0,
        applied_block_sections: 0,
        queued_for_chunk: false,
        light_data,
    })
}

fn parse_light_data(input: &mut &[u8]) -> Result<LightUpdateData, ConnectionError> {
    let sky_mask = read_bitset(input, "sky light mask")?;
    let block_mask = read_bitset(input, "block light mask")?;
    let empty_sky_mask = read_bitset(input, "empty sky light mask")?;
    let empty_block_mask = read_bitset(input, "empty block light mask")?;
    let sky_updates = read_light_update_bytes(input, "sky light updates")?;
    let block_updates = read_light_update_bytes(input, "block light updates")?;
    Ok(LightUpdateData {
        sky_mask,
        block_mask,
        empty_sky_mask,
        empty_block_mask,
        sky_updates,
        block_updates,
    })
}

pub(crate) fn parse_forget_level_chunk_packet(
    mut input: &[u8],
) -> Result<(i32, i32), ConnectionError> {
    let packed = read_i64(&mut input, "forget level chunk position")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "forget level chunk trailing payload",
        ));
    }
    Ok((packed as i32, (packed >> 32) as i32))
}

pub(crate) fn parse_set_chunk_cache_radius_packet(
    mut input: &[u8],
) -> Result<i32, ConnectionError> {
    let radius = read_varint(&mut input, "chunk cache radius")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "chunk cache radius trailing payload",
        ));
    }
    Ok(radius)
}

pub(crate) fn parse_set_chunk_cache_center_packet(
    mut input: &[u8],
) -> Result<(i32, i32), ConnectionError> {
    let x = read_varint(&mut input, "chunk cache center x")?;
    let z = read_varint(&mut input, "chunk cache center z")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "chunk cache center trailing payload",
        ));
    }
    Ok((x, z))
}

pub(crate) fn read_varint(input: &mut &[u8], name: &'static str) -> Result<i32, ConnectionError> {
    wire::read_varint(input).map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn read_count(input: &mut &[u8], name: &'static str) -> Result<usize, ConnectionError> {
    let count = read_varint(input, name)?;
    usize::try_from(count).map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn read_byte(input: &mut &[u8], name: &'static str) -> Result<u8, ConnectionError> {
    let Some((&byte, rest)) = input.split_first() else {
        return Err(ConnectionError::Codec(name));
    };
    *input = rest;
    Ok(byte)
}

pub(crate) fn read_bool(input: &mut &[u8], name: &'static str) -> Result<bool, ConnectionError> {
    Ok(read_byte(input, name)? != 0)
}

pub(crate) fn read_i64(input: &mut &[u8], name: &'static str) -> Result<i64, ConnectionError> {
    wire::take(input)
        .map(i64::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub(crate) fn read_i16(input: &mut &[u8], name: &'static str) -> Result<i16, ConnectionError> {
    wire::take(input)
        .map(i16::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub(crate) fn read_i32(input: &mut &[u8], name: &'static str) -> Result<i32, ConnectionError> {
    wire::take(input)
        .map(i32::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub(crate) fn read_f32(input: &mut &[u8], name: &'static str) -> Result<f32, ConnectionError> {
    wire::take(input)
        .map(f32::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub(crate) fn read_f64(input: &mut &[u8], name: &'static str) -> Result<f64, ConnectionError> {
    wire::take(input)
        .map(f64::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlayerPositionPacket {
    pub(crate) teleport_id: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) delta_x: f64,
    pub(crate) delta_y: f64,
    pub(crate) delta_z: f64,
    pub(crate) y_rot: f32,
    pub(crate) x_rot: f32,
    pub(crate) relative_flags: i32,
}

pub(crate) fn parse_player_position_packet(
    input: &[u8],
) -> Result<PlayerPositionPacket, ConnectionError> {
    let mut input = input;
    let packet = PlayerPositionPacket {
        teleport_id: read_varint(&mut input, "player position teleport id")?,
        x: read_f64(&mut input, "player position x")?,
        y: read_f64(&mut input, "player position y")?,
        z: read_f64(&mut input, "player position z")?,
        delta_x: read_f64(&mut input, "player position delta x")?,
        delta_y: read_f64(&mut input, "player position delta y")?,
        delta_z: read_f64(&mut input, "player position delta z")?,
        y_rot: read_f32(&mut input, "player position y rotation")?,
        x_rot: read_f32(&mut input, "player position x rotation")?,
        relative_flags: read_i32(&mut input, "player position relative flags")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("player position trailing payload"));
    }
    Ok(packet)
}

pub(crate) fn resolve_player_position(
    packet: PlayerPositionPacket,
    current: Option<(f64, f64, f64, f32, f32)>,
) -> (f64, f64, f64, f32, f32) {
    let current = current.unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0));
    let x = if packet.relative_flags & PLAYER_RELATIVE_X_FLAG != 0 {
        current.0 + packet.x
    } else {
        packet.x
    };
    let y = if packet.relative_flags & PLAYER_RELATIVE_Y_FLAG != 0 {
        current.1 + packet.y
    } else {
        packet.y
    };
    let z = if packet.relative_flags & PLAYER_RELATIVE_Z_FLAG != 0 {
        current.2 + packet.z
    } else {
        packet.z
    };
    let y_rot = if packet.relative_flags & PLAYER_RELATIVE_Y_ROT_FLAG != 0 {
        current.3 + packet.y_rot
    } else {
        packet.y_rot
    };
    let x_rot = if packet.relative_flags & PLAYER_RELATIVE_X_ROT_FLAG != 0 {
        (current.4 + packet.x_rot).clamp(-90.0, 90.0)
    } else {
        packet.x_rot.clamp(-90.0, 90.0)
    };
    (x, y, z, y_rot, x_rot)
}

pub(crate) fn read_long_array(
    input: &mut &[u8],
    name: &'static str,
) -> Result<Vec<i64>, ConnectionError> {
    voxel_core::binary::read_long_array(input).map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn read_bitset(
    input: &mut &[u8],
    name: &'static str,
) -> Result<BitSet, ConnectionError> {
    voxel_core::binary::read_bitset(input).map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn read_length_prefixed_bytes<'a>(
    input: &mut &'a [u8],
    max_length: usize,
    name: &'static str,
) -> Result<&'a [u8], ConnectionError> {
    let length = read_count(input, name)?;
    if length > max_length || input.len() < length {
        return Err(ConnectionError::Codec(name));
    }
    let (head, rest) = input.split_at(length);
    *input = rest;
    Ok(head)
}

pub(crate) fn read_light_update_bytes(
    input: &mut &[u8],
    name: &'static str,
) -> Result<Vec<Vec<u8>>, ConnectionError> {
    let count = read_count(input, name)?;
    let mut updates = Vec::with_capacity(count);
    for _ in 0..count {
        let bytes = read_length_prefixed_bytes(input, 2_048, name)?;
        updates.push(bytes.to_vec());
    }
    Ok(updates)
}

pub(crate) fn parse_section_data(
    mut input: &[u8],
) -> Result<Vec<ChunkSectionObservation>, ConnectionError> {
    let mut sections = Vec::new();
    while !input.is_empty() {
        let non_empty_block_count = read_i16(&mut input, "section non-empty block count")?;
        let fluid_count = read_i16(&mut input, "section fluid count")?;
        let block_states = parse_palette_container(&mut input, 4_096, true)?;
        let biomes = parse_palette_container(&mut input, 64, false)?;
        sections.push(ChunkSectionObservation {
            non_empty_block_count,
            fluid_count,
            block_state_bits_per_entry: block_states.bits_per_entry,
            block_state_palette_entry_count: block_states.palette_entry_count,
            block_state_palette_ids: block_states.palette_ids,
            block_state_storage_word_count: block_states.storage_word_count,
            block_state_first_values: block_states.first_values,
            block_state_values: block_states.values,
            block_state_distinct_value_count: block_states.distinct_value_count,
            biome_bits_per_entry: biomes.bits_per_entry,
            biome_palette_entry_count: biomes.palette_entry_count,
            biome_palette_ids: biomes.palette_ids,
            biome_storage_word_count: biomes.storage_word_count,
            biome_first_values: biomes.first_values,
            biome_values: biomes.values,
            biome_distinct_value_count: biomes.distinct_value_count,
        });
    }
    Ok(sections)
}

/// Decode the section payload with the same parser used by chunk packets.
pub fn decode_chunk_section_data(input: &[u8]) -> Result<Vec<ChunkSectionObservation>, String> {
    parse_section_data(input).map_err(|error| error.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PaletteObservation {
    bits_per_entry: u8,
    palette_entry_count: usize,
    palette_ids: Vec<i32>,
    storage_word_count: usize,
    first_values: Vec<i32>,
    values: Vec<i32>,
    distinct_value_count: usize,
}

fn parse_palette_container(
    input: &mut &[u8],
    entry_count: usize,
    block_states: bool,
) -> Result<PaletteObservation, ConnectionError> {
    let bits_per_entry = read_byte(
        input,
        if block_states {
            "block state bits"
        } else {
            "biome bits"
        },
    )?;
    if bits_per_entry > 32 {
        return Err(ConnectionError::Codec(if block_states {
            "block state bits"
        } else {
            "biome bits"
        }));
    }

    let (palette_capacity, storage_bits) = if bits_per_entry == 0 {
        (Some(1), 0)
    } else if block_states && bits_per_entry <= 4 {
        (Some(1_usize << 4), 4)
    } else if (block_states && bits_per_entry <= 8) || (!block_states && bits_per_entry <= 3) {
        (Some(1_usize << bits_per_entry), bits_per_entry)
    } else {
        (None, bits_per_entry)
    };

    let palette_ids = if bits_per_entry == 0 {
        let value = read_varint(
            input,
            if block_states {
                "block state palette value"
            } else {
                "biome palette value"
            },
        )?;
        vec![value]
    } else if let Some(capacity) = palette_capacity {
        let name = if block_states {
            "block state palette"
        } else {
            "biome palette"
        };
        let count = read_count(input, name)?;
        if count > capacity {
            return Err(ConnectionError::Codec(name));
        }
        let mut palette_ids = Vec::with_capacity(count);
        for _ in 0..count {
            palette_ids.push(read_varint(
                input,
                if block_states {
                    "block state palette value"
                } else {
                    "biome palette value"
                },
            )?);
        }
        palette_ids
    } else {
        Vec::new()
    };

    let storage_word_count = if storage_bits == 0 {
        0
    } else {
        let values_per_long = 64 / usize::from(storage_bits);
        entry_count.div_ceil(values_per_long)
    };
    let storage = read_fixed_long_array(
        input,
        storage_word_count,
        if block_states {
            "block state storage"
        } else {
            "biome storage"
        },
    )?;

    let values = decode_palette_values(
        &storage,
        entry_count,
        storage_bits,
        &palette_ids,
        if block_states {
            "block state value"
        } else {
            "biome value"
        },
    )?;
    let first_values = values.iter().copied().take(4).collect();
    let distinct_value_count = values.iter().copied().collect::<BTreeSet<_>>().len();

    Ok(PaletteObservation {
        bits_per_entry,
        palette_entry_count: palette_ids.len(),
        palette_ids,
        storage_word_count,
        first_values,
        values,
        distinct_value_count,
    })
}

pub(crate) fn read_fixed_long_array(
    input: &mut &[u8],
    word_count: usize,
    name: &'static str,
) -> Result<Vec<u64>, ConnectionError> {
    let byte_count = word_count
        .checked_mul(8)
        .ok_or(ConnectionError::Codec(name))?;
    if input.len() < byte_count {
        return Err(ConnectionError::Codec(name));
    }
    let (head, rest) = input.split_at(byte_count);
    *input = rest;
    let mut words = Vec::with_capacity(word_count);
    let (chunks, remainder) = head.as_chunks::<8>();
    debug_assert!(remainder.is_empty());
    for word in chunks {
        words.push(u64::from_be_bytes(*word));
    }
    Ok(words)
}

pub(crate) fn decode_palette_values(
    storage: &[u64],
    entry_count: usize,
    storage_bits: u8,
    palette_ids: &[i32],
    name: &'static str,
) -> Result<Vec<i32>, ConnectionError> {
    if storage_bits == 0 {
        let Some(&value) = palette_ids.first() else {
            return Err(ConnectionError::Codec(name));
        };
        return Ok(vec![value; entry_count]);
    }

    let bits = usize::from(storage_bits);
    let values_per_long = 64 / bits;
    let mask = if bits == 32 {
        u64::from(u32::MAX)
    } else {
        (1_u64 << bits) - 1
    };
    let mut values = Vec::with_capacity(entry_count);
    for index in 0..entry_count {
        let Some(&word) = storage.get(index / values_per_long) else {
            return Err(ConnectionError::Codec(name));
        };
        let palette_index = ((word >> ((index % values_per_long) * bits)) & mask) as usize;
        let value = if palette_ids.is_empty() {
            i32::try_from(palette_index).map_err(|_| ConnectionError::Codec(name))?
        } else {
            *palette_ids
                .get(palette_index)
                .ok_or(ConnectionError::Codec(name))?
        };
        values.push(value);
    }
    Ok(values)
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
