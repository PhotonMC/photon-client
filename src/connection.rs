//! A small synchronous client connection used by the first real-server integration probe.
//!
//! This module owns the byte pipeline up to the first play-world packets. It does not
//! implement a renderer or a client world; its purpose is to make the phase transition
//! and the first world delivery observable without turning the test into a manual launch.

use std::collections::BTreeSet;
use std::fmt;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::binary::Uuid;
use crate::configuration::{
    ChatVisibility, ClientInformation, MainHand, ParticleStatus, ServerboundClientInformationPacket,
};
use crate::frame::{self, FrameError, MAX_FRAME_LENGTH, MAX_VARINT21_BYTES};
use crate::handshake::{ClientIntent, ClientIntentionPacket};
use crate::login::{ClientboundLoginFinishedPacket, ServerboundHelloPacket};
use crate::protocol::ConnectionProtocol;
use crate::resources::KnownPack;
use crate::wire;

const PROTOCOL_VERSION: i32 = 776;
const LOGIN_INTENTION_PACKET_ID: i32 = 0;
const LOGIN_HELLO_PACKET_ID: i32 = 0;
const LOGIN_ACKNOWLEDGED_PACKET_ID: i32 = 3;
const LOGIN_FINISHED_PACKET_ID: i32 = 2;
const LOGIN_COMPRESSION_PACKET_ID: i32 = 3;

const STATUS_SERVERBOUND_REQUEST_PACKET_ID: i32 = 0;
const STATUS_SERVERBOUND_PING_REQUEST_PACKET_ID: i32 = 1;
const STATUS_CLIENTBOUND_RESPONSE_PACKET_ID: i32 = 0;
const STATUS_CLIENTBOUND_PONG_RESPONSE_PACKET_ID: i32 = 1;
const STATUS_RESPONSE_MAX_UTF16_LENGTH: usize = 32_767;
const STATUS_PROBE_TIME: i64 = 0x0102_0304_0506_0708;

const CONFIG_CLIENT_INFORMATION_PACKET_ID: i32 = 0;
const CONFIG_CUSTOM_PAYLOAD_PACKET_ID: i32 = 2;
const CONFIG_FINISH_PACKET_ID: i32 = 3;
const CONFIG_KEEP_ALIVE_PACKET_ID: i32 = 4;
const CONFIG_PONG_PACKET_ID: i32 = 5;
const CONFIG_REGISTRY_DATA_PACKET_ID: i32 = 7;
const CONFIG_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 14;
const CONFIG_SERVERBOUND_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 7;
const CONFIG_ACCEPT_CODE_OF_CONDUCT_PACKET_ID: i32 = 9;
const CONFIG_ENABLED_FEATURES_PACKET_ID: i32 = 12;
const CONFIG_UPDATE_TAGS_PACKET_ID: i32 = 13;
const CONFIG_CODE_OF_CONDUCT_PACKET_ID: i32 = 19;

// The virtual bundle packet is not registered in the Java dispatch table; the delimiter
// occupies id 0 and shifts the listed play clientbound packet ids by one.
const PLAY_CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID: i32 = 0;
const PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID: i32 = 1;
const PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID: i32 = 4;
const PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID: i32 = 5;
const PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID: i32 = 8;
const PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID: i32 = 11;
const PLAY_CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID: i32 = 12;
const PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID: i32 = 18;
const PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID: i32 = 20;
const PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID: i32 = 78;
const PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID: i32 = 121;
const PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID: i32 = 44;
const PLAY_CLIENTBOUND_CHUNK_PACKET_ID: i32 = 45;
const PLAY_CLIENTBOUND_LOGIN_PACKET_ID: i32 = 49;
const PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID: i32 = 72;
const PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID: i32 = 132;
const PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID: i32 = 0;
const PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID: i32 = 41;
const PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID: i32 = 11;
const PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID: i32 = 28;
const PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID: i32 = 31;
const PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID: i32 = 44;
const PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID: i32 = 53;
const PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID: i32 = 56;
const PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID: i32 = 66;
const PLAY_SERVERBOUND_USE_ITEM_PACKET_ID: i32 = 67;

// The pinned vanilla built-in item registry is not sent in configuration registry data.
// Items.java registers AIR before STONE, so STONE has id 1 in BuiltInRegistries.ITEM.
const VANILLA_STONE_ITEM_REGISTRY_ID: i32 = 1;
const VANILLA_SNOWBALL_ITEM_REGISTRY_ID: i32 = 1_044;
const VANILLA_DAMAGE_COMPONENT_ID: i32 = 3;
// DataComponents registers CUSTOM_MODEL_DATA after the first 17 built-in component types.
const VANILLA_CUSTOM_MODEL_DATA_COMPONENT_ID: i32 = 17;
const VANILLA_TOOL_COMPONENT_ID: i32 = 28;
const VANILLA_WEAPON_COMPONENT_ID: i32 = 29;

const PLAYER_RELATIVE_X_FLAG: i32 = 1 << 0;
const PLAYER_RELATIVE_Y_FLAG: i32 = 1 << 1;
const PLAYER_RELATIVE_Z_FLAG: i32 = 1 << 2;
const PLAYER_RELATIVE_Y_ROT_FLAG: i32 = 1 << 3;
const PLAYER_RELATIVE_X_ROT_FLAG: i32 = 1 << 4;

const MAX_PACKET_UNCOMPRESSED_LENGTH: usize = 8 * 1024 * 1024;
const MAX_CHUNK_DATA_LENGTH: usize = 2 * 1024 * 1024;
const MAX_KNOWN_PACKS: usize = 64;
const MAX_READABLE_FRAME_LENGTH: usize = MAX_FRAME_LENGTH;
const WORLD_READY_DRAIN_TIMEOUT: Duration = Duration::from_millis(250);
const DELAYED_DESTROY_DRAIN_TIMEOUT: Duration = Duration::from_millis(1_200);
const WATER_BUCKET_DRAIN_TIMEOUT: Duration = Duration::from_millis(1_000);
const CONSUMABLE_DRAIN_TIMEOUT: Duration = Duration::from_millis(2_500);

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
    pub configuration_finished: bool,
    pub enabled_feature_count: usize,
    pub known_pack_count: usize,
    pub registry_packet_count: usize,
    pub registry_entry_count: usize,
    pub update_tags_packet_count: usize,
    pub play_login: bool,
    pub player_position_observations: Vec<PlayerPositionObservation>,
    pub player_loaded_sent: bool,
    pub movement_packets_sent: usize,
    pub player_action_packets_sent: usize,
    pub use_item_on_packets_sent: usize,
    pub use_item_packets_sent: usize,
    pub creative_mode_slot_packets_sent: usize,
    pub carried_item_packets_sent: usize,
    pub stone_item_registry_id: Option<i32>,
    pub snowball_item_registry_id: Option<i32>,
    pub add_entity_observations: Vec<AddEntityObservation>,
    pub container_content_observations: Vec<ContainerSetContentObservation>,
    pub container_slot_observations: Vec<ContainerSetSlotObservation>,
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
    pub chunk_packet_count: usize,
    pub chunk_batch_finished_count: usize,
    pub chunk_observations: Vec<ChunkPacketObservation>,
}

/// Wire-level fields observed from a clientbound player-position correction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerPositionObservation {
    pub teleport_id: i32,
    pub relative_flags: i32,
}

/// Wire-level fields decoded from one clientbound block update packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockUpdateObservation {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub state_id: i32,
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
    pub entity_type_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub data: i32,
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
}

/// Wire-level fields decoded from one clientbound container content update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSetContentObservation {
    pub container_id: i32,
    pub state_id: i32,
    pub item_count: usize,
    pub hotbar_slot_zero_item_count: Option<i32>,
    pub hotbar_slot_zero_item_registry_id: Option<i32>,
    pub hotbar_slot_zero_positive_component_type_ids: Vec<i32>,
    pub hotbar_slot_zero_negative_component_type_ids: Vec<i32>,
    pub carried_item_count: Option<i32>,
    pub carried_item_registry_id: Option<i32>,
    pub carried_positive_component_type_ids: Vec<i32>,
    pub carried_negative_component_type_ids: Vec<i32>,
}

/// Wire-level fields decoded from one clientbound mob-effect removal packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectRemoveObservation {
    pub entity_id: i32,
    pub effect_registry_id: i32,
}

/// Wire-level fields decoded from one clientbound mob-effect update packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectUpdateObservation {
    pub entity_id: i32,
    pub effect_registry_id: i32,
    pub amplifier: i32,
    pub duration_ticks: i32,
    pub flags: u8,
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
    pub sky_mask_word_count: usize,
    pub block_mask_word_count: usize,
    pub empty_sky_mask_word_count: usize,
    pub empty_block_mask_word_count: usize,
    pub sky_update_count: usize,
    pub sky_update_lengths: Vec<usize>,
    pub block_update_count: usize,
    pub block_update_lengths: Vec<usize>,
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
    pub block_state_distinct_value_count: usize,
    pub biome_bits_per_entry: u8,
    pub biome_palette_entry_count: usize,
    pub biome_palette_ids: Vec<i32>,
    pub biome_storage_word_count: usize,
    pub biome_first_values: Vec<i32>,
    pub biome_distinct_value_count: usize,
}

/// The observable result of a status and ping session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusConnectionTrace {
    pub packets: Vec<PacketObservation>,
    pub status_response: String,
    pub pong_time: i64,
}

/// Query a Java-protocol server's status and verify its ping echo.
pub fn query_status_and_ping(
    address: SocketAddr,
    timeout: Duration,
) -> Result<StatusConnectionTrace, ConnectionError> {
    if timeout.is_zero() {
        return Err(ConnectionError::InvalidArgument("timeout"));
    }

    let mut stream = connect_stream(address, timeout)?;
    send_handshake_intention(&mut stream, address, ClientIntent::Status)?;
    send_packet(
        &mut stream,
        ConnectionProtocol::Status,
        STATUS_SERVERBOUND_REQUEST_PACKET_ID,
        |payload| {
            crate::status::ServerboundStatusRequestPacket::INSTANCE.encode(payload);
            Ok(())
        },
        None,
    )?;

    let mut packets = Vec::with_capacity(2);
    let body = read_packet(&mut stream, None)?;
    let (packet_id, payload) = split_packet(&body)?;
    packets.push(PacketObservation {
        phase: ConnectionProtocol::Status,
        id: packet_id,
        body_length: body.len(),
    });
    if packet_id != STATUS_CLIENTBOUND_RESPONSE_PACKET_ID {
        return Err(ConnectionError::UnsupportedPacket {
            phase: ConnectionProtocol::Status,
            id: packet_id,
        });
    }

    let mut input = payload;
    let status_units = crate::utf8::read_utf8(&mut input, STATUS_RESPONSE_MAX_UTF16_LENGTH)
        .map_err(|_| ConnectionError::Codec("status response"))?;
    let status_response = String::from_utf16(&status_units)
        .map_err(|_| ConnectionError::Codec("status response UTF-16"))?;

    let ping = crate::status::ServerboundPingRequestPacket::new(STATUS_PROBE_TIME);
    send_packet(
        &mut stream,
        ConnectionProtocol::Status,
        STATUS_SERVERBOUND_PING_REQUEST_PACKET_ID,
        |payload| {
            ping.encode(payload);
            Ok(())
        },
        None,
    )?;

    let body = read_packet(&mut stream, None)?;
    let (packet_id, payload) = split_packet(&body)?;
    packets.push(PacketObservation {
        phase: ConnectionProtocol::Status,
        id: packet_id,
        body_length: body.len(),
    });
    if packet_id != STATUS_CLIENTBOUND_PONG_RESPONSE_PACKET_ID {
        return Err(ConnectionError::UnsupportedPacket {
            phase: ConnectionProtocol::Status,
            id: packet_id,
        });
    }

    let mut input = payload;
    let pong = crate::status::ClientboundPongResponsePacket::decode(&mut input)
        .map_err(|_| ConnectionError::Codec("status pong"))?;
    if pong.time != STATUS_PROBE_TIME {
        return Err(ConnectionError::Codec("status pong echo"));
    }

    Ok(StatusConnectionTrace {
        packets,
        status_response,
        pong_time: pong.time,
    })
}

/// Connect to a Java-protocol server and drive the session until the first play chunk.
///
/// The server must use offline authentication, as the project deliberately does not
/// implement Mojang's encryption/authentication exchange in this first slice.
pub fn connect_and_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(address, username, timeout, PlayProbeMode::Basic)
}

/// Connect to a Java-protocol server and send a short controlled position trace after loading.
pub fn connect_and_move_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(address, username, timeout, PlayProbeMode::Movement)
}

/// Connect to a Java-protocol server and send a harmless block-action packet after loading.
pub fn connect_and_action_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(address, username, timeout, PlayProbeMode::BlockAction)
}

/// Connect to a Java-protocol server and break the block supporting the spawned player.
pub fn connect_and_block_break_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(address, username, timeout, PlayProbeMode::BlockBreak)
}

/// Connect to a Java-protocol server and send a block-break action outside interaction range.
pub fn connect_and_block_break_rejection_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::BlockBreakOutOfRange,
    )
}

/// Connect to a Java-protocol server and use the empty main hand on the supporting block.
pub fn connect_and_use_item_on_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(address, username, timeout, PlayProbeMode::UseItemOn)
}

/// Connect to a Java-protocol server and use the empty main hand in open space.
pub fn connect_and_use_item_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(address, username, timeout, PlayProbeMode::UseItem)
}

/// Connect to a Java-protocol server, place a snowball stack in the creative hotbar,
/// select it and use the main hand to spawn a projectile.
pub fn connect_and_creative_snowball_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::CreativeSnowballUse,
    )
}

/// Connect to a Java-protocol server, select a supplied survival snowball stack and use it.
pub fn connect_and_survival_snowball_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalSnowballUse,
    )
}

/// Connect to a Java-protocol server, select a supplied survival water bucket and use it
/// on the upper face of the supporting block.
pub fn connect_and_survival_water_bucket_use_on_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalWaterBucketUseOn,
    )
}

/// Connect to a Java-protocol server, select a supplied survival empty bucket and use it
/// to pick up a water source beside the supporting block.
pub fn connect_and_survival_empty_bucket_pickup_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalEmptyBucketPickup,
    )
}

/// Connect to a Java-protocol server, use a component-bearing survival milk bucket,
/// and observe its completed bucket remainder.
pub fn connect_and_survival_milk_bucket_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalMilkBucketUse,
    )
}

/// Connect to a Java-protocol server, use a component-bearing survival honey bottle,
/// and observe its completed glass-bottle remainder.
pub fn connect_and_survival_honey_bottle_use_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalHoneyBottleUse,
    )
}

/// Connect to a Java-protocol server, use a component-bearing survival honey bottle
/// while poison is active, and observe the completed glass-bottle remainder and the
/// authoritative mob-effect removal packet.
pub fn connect_and_survival_honey_bottle_poison_cleanup_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalHoneyBottlePoisonCleanup,
    )
}

/// Connect to a Java-protocol server, place a stone stack in the creative hotbar,
/// select it and request a side placement against the supporting block.
pub fn connect_and_creative_place_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(address, username, timeout, PlayProbeMode::CreativePlace)
}

/// Connect to a Java-protocol server, start breaking the supporting block in survival
/// mode and immediately abort the same action.
pub fn connect_and_survival_break_abort_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalBreakAbort,
    )
}

/// Connect to a Java-protocol server, break the supporting dirt block in survival mode
/// with a timed stop action and observe the authoritative air update.
pub fn connect_and_survival_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(address, username, timeout, PlayProbeMode::SurvivalBreakStop)
}

/// Connect to a Java-protocol server, start breaking the supporting stone block in survival
/// mode and stop before the bare-hand hardness threshold is reached.
pub fn connect_and_survival_stone_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalStoneBreakStop,
    )
}

/// Connect to a Java-protocol server, start breaking the supporting oak-planks block in
/// survival mode and stop before the bare-hand hardness threshold is reached.
pub fn connect_and_survival_oak_planks_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalOakPlanksBreakStop,
    )
}

/// Connect to a Java-protocol server, select a wooden pickaxe supplied in hotbar slot 0,
/// and break the supporting stone block in survival mode.
pub fn connect_and_survival_stone_wooden_pickaxe_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalStoneWoodenPickaxeBreakStop,
    )
}

/// Connect to a Java-protocol server, select a wooden shovel supplied in hotbar slot 0,
/// and break the supporting dirt block in survival mode.
pub fn connect_and_survival_dirt_wooden_shovel_break_stop_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalDirtWoodenShovelBreakStop,
    )
}

/// Connect to a Java-protocol server, stop a short survival dirt break early and
/// observe the server's delayed destroy completion on later ticks.
pub fn connect_and_survival_delayed_destroy_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalDelayedDestroy,
    )
}

/// Connect to a Java-protocol server, request a protected spawn-area block break
/// in survival mode and observe the denial message and sequence acknowledgement.
pub fn connect_and_survival_spawn_protection_probe(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
) -> Result<PlayConnectionTrace, ConnectionError> {
    connect_and_probe_with_mode(
        address,
        username,
        timeout,
        PlayProbeMode::SurvivalSpawnProtection,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayProbeMode {
    Basic,
    Movement,
    BlockAction,
    BlockBreak,
    BlockBreakOutOfRange,
    UseItemOn,
    UseItem,
    CreativeSnowballUse,
    SurvivalSnowballUse,
    SurvivalWaterBucketUseOn,
    SurvivalEmptyBucketPickup,
    SurvivalMilkBucketUse,
    SurvivalHoneyBottleUse,
    SurvivalHoneyBottlePoisonCleanup,
    CreativePlace,
    SurvivalBreakAbort,
    SurvivalBreakStop,
    SurvivalStoneBreakStop,
    SurvivalOakPlanksBreakStop,
    SurvivalStoneWoodenPickaxeBreakStop,
    SurvivalDirtWoodenShovelBreakStop,
    SurvivalDelayedDestroy,
    SurvivalSpawnProtection,
}

fn connect_and_probe_with_mode(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
    probe_mode: PlayProbeMode,
) -> Result<PlayConnectionTrace, ConnectionError> {
    if username.is_empty() || username.encode_utf16().count() > crate::login::MAX_NAME_LENGTH {
        return Err(ConnectionError::InvalidArgument("username"));
    }
    if timeout.is_zero() {
        return Err(ConnectionError::InvalidArgument("timeout"));
    }

    let mut stream = connect_stream(address, timeout)?;
    send_handshake_intention(&mut stream, address, ClientIntent::Login)?;

    let hello = ServerboundHelloPacket::new(username.encode_utf16().collect(), Uuid::new(0, 1));
    send_packet(
        &mut stream,
        ConnectionProtocol::Login,
        LOGIN_HELLO_PACKET_ID,
        |payload| {
            hello
                .encode(payload)
                .map_err(|_| ConnectionError::Codec("login hello"))
        },
        None,
    )?;

    let started = Instant::now();
    let mut compression_threshold = None;
    let mut trace = PlayConnectionTrace::default();
    if matches!(probe_mode, PlayProbeMode::CreativePlace) {
        trace.stone_item_registry_id = Some(VANILLA_STONE_ITEM_REGISTRY_ID);
    }
    if matches!(probe_mode, PlayProbeMode::CreativeSnowballUse) {
        trace.snowball_item_registry_id = Some(VANILLA_SNOWBALL_ITEM_REGISTRY_ID);
    }

    loop {
        let body = read_packet(&mut stream, compression_threshold)?;
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Login,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            LOGIN_COMPRESSION_PACKET_ID => {
                let mut input = payload;
                let packet = crate::login::ClientboundLoginCompressionPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("login compression"))?;
                compression_threshold = if packet.threshold < 0 {
                    None
                } else {
                    Some(packet.threshold as usize)
                };
                trace.compression_threshold = compression_threshold;
            }
            LOGIN_FINISHED_PACKET_ID => {
                let mut input = payload;
                ClientboundLoginFinishedPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("login finished"))?;
                trace.login_finished = true;

                send_packet(
                    &mut stream,
                    ConnectionProtocol::Login,
                    LOGIN_ACKNOWLEDGED_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
                send_configuration_start(&mut stream, compression_threshold)?;
                break;
            }
            0 => return Err(ConnectionError::ServerDisconnected),
            1 => {
                return Err(ConnectionError::UnsupportedPacket {
                    phase: ConnectionProtocol::Login,
                    id: packet_id,
                });
            }
            _ => {}
        }

        if started.elapsed() > timeout {
            return Err(ConnectionError::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "login timeout",
            )));
        }
    }

    let configuration_started = Instant::now();
    loop {
        let body = read_packet(&mut stream, compression_threshold)?;
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Configuration,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            CONFIG_REGISTRY_DATA_PACKET_ID => {
                let mut input = payload;
                let packet =
                    crate::registry_data::ClientboundRegistryDataPacket::decode(&mut input)
                        .map_err(|_| ConnectionError::Codec("registry data"))?;
                trace.registry_packet_count += 1;
                trace.registry_entry_count += packet.entries.len();
            }
            CONFIG_UPDATE_TAGS_PACKET_ID => {
                let mut input = payload;
                crate::tags::ClientboundUpdateTagsPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("update tags"))?;
                trace.update_tags_packet_count += 1;
            }
            CONFIG_ENABLED_FEATURES_PACKET_ID => {
                let mut input = payload;
                let packet = crate::configuration::ClientboundUpdateEnabledFeaturesPacket::decode(
                    &mut input,
                )
                .map_err(|_| ConnectionError::Codec("enabled features"))?;
                trace.enabled_feature_count = packet.features.len();
            }
            CONFIG_SELECT_KNOWN_PACKS_PACKET_ID => {
                let mut input = payload;
                let packs = KnownPack::decode_list(&mut input, usize::MAX)
                    .map_err(|_| ConnectionError::Codec("clientbound known packs"))?;
                trace.known_pack_count = packs.len();

                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_SERVERBOUND_SELECT_KNOWN_PACKS_PACKET_ID,
                    |output| {
                        KnownPack::encode_list(&packs, output, MAX_KNOWN_PACKS)
                            .map_err(|_| ConnectionError::Codec("serverbound known packs"))
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("configuration keep alive"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_KEEP_ALIVE_PACKET_ID,
                    |output| {
                        crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_PONG_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundPingPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("configuration ping"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_PONG_PACKET_ID,
                    |output| {
                        crate::common::ServerboundPongPacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            CONFIG_CODE_OF_CONDUCT_PACKET_ID => {
                let mut input = payload;
                crate::configuration::ClientboundCodeOfConductPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("code of conduct"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_ACCEPT_CODE_OF_CONDUCT_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
            }
            CONFIG_FINISH_PACKET_ID => {
                crate::configuration::ClientboundFinishConfigurationPacket::decode(
                    &mut &payload[..],
                )
                .map_err(|_| ConnectionError::Codec("finish configuration"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Configuration,
                    CONFIG_FINISH_PACKET_ID,
                    |_payload| Ok(()),
                    compression_threshold,
                )?;
                trace.configuration_finished = true;
                break;
            }
            2 => return Err(ConnectionError::ServerDisconnected),
            9 | 11 => {
                return Err(ConnectionError::UnsupportedPacket {
                    phase: ConnectionProtocol::Configuration,
                    id: packet_id,
                });
            }
            _ => {}
        }

        if configuration_started.elapsed() > timeout {
            return Err(ConnectionError::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "configuration timeout",
            )));
        }
    }

    let play_started = Instant::now();
    let mut chunk_batch_started = false;
    let mut player_loaded_sent = false;
    let mut player_position = None;
    let mut creative_place_target_position = None;
    let mut creative_place_neighbor_position = None;
    let mut survival_water_bucket_target_position = None;
    let mut survival_water_bucket_neighbor_position = None;
    let mut world_ready_drain_started = None;
    while play_started.elapsed() <= timeout {
        let body = match read_packet(&mut stream, compression_threshold) {
            Ok(body) => body,
            Err(ConnectionError::Io(error))
                if world_ready_drain_started.is_some()
                    && matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
            {
                return Ok(trace);
            }
            Err(error) => return Err(error),
        };
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Play,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID => {
                trace
                    .add_entity_observations
                    .push(parse_add_entity_packet(payload)?);
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID => {
                trace
                    .container_content_observations
                    .push(parse_container_set_content_packet(payload)?);
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID => {
                trace
                    .container_slot_observations
                    .push(parse_container_set_slot_packet(payload)?);
            }
            PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID => {
                trace
                    .mob_effect_remove_observations
                    .push(parse_remove_mob_effect_packet(payload)?);
            }
            PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID => {
                trace
                    .system_chat_observations
                    .push(parse_system_chat_packet(payload)?);
            }
            PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID => {
                let mut input = payload;
                let sequence = read_varint(&mut input, "block changed ack sequence")?;
                if !input.is_empty() {
                    return Err(ConnectionError::Codec("block changed ack trailing payload"));
                }
                trace.block_changed_ack_sequences.push(sequence);
            }
            PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID => {
                trace
                    .block_destruction_observations
                    .push(parse_block_destruction_packet(payload)?);
            }
            PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID => {
                let observation = parse_block_update_packet(payload)?;
                if creative_place_target_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.creative_place_target_update = Some(observation.clone());
                }
                if creative_place_neighbor_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.creative_place_neighbor_update = Some(observation.clone());
                }
                if survival_water_bucket_target_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.survival_water_bucket_target_update = Some(observation.clone());
                }
                if survival_water_bucket_neighbor_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.survival_water_bucket_neighbor_update = Some(observation.clone());
                }
                trace.block_update_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID => {
                chunk_batch_started = true;
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID => {
                let mut input = payload;
                wire::read_varint(&mut input)
                    .map_err(|_| ConnectionError::Codec("chunk batch finished"))?;
                trace.chunk_batch_finished_count += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID,
                    |output| {
                        // This is the initial value used by Java's chunk batch calculator:
                        // 7,000,000 ns / 2,000,000 ns per chunk.
                        output.extend_from_slice(&3.5_f32.to_be_bytes());
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            PLAY_CLIENTBOUND_LOGIN_PACKET_ID => trace.play_login = true,
            PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID => {
                let packet = parse_player_position_packet(payload)?;
                trace
                    .player_position_observations
                    .push(PlayerPositionObservation {
                        teleport_id: packet.teleport_id,
                        relative_flags: packet.relative_flags,
                    });
                let absolute = resolve_player_position(packet, player_position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID,
                    |output| {
                        wire::write_varint(packet.teleport_id, output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
                    |output| encode_move_player_pos_rot(output, absolute, 0),
                    compression_threshold,
                )?;
                player_position = Some(absolute);
            }
            PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID => {
                trace
                    .mob_effect_update_observations
                    .push(parse_update_mob_effect_packet(payload)?);
            }
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => {
                trace.chunk_observations.push(parse_chunk_packet(payload)?);
                trace.chunk_packet_count += 1;
            }
            PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("play keep alive"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID,
                    |output| {
                        crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            PLAY_CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID => {}
            _ => {}
        }

        if trace.play_login
            && trace.chunk_packet_count > 0
            && (!chunk_batch_started || trace.chunk_batch_finished_count > 0)
            && world_ready_drain_started.is_none()
        {
            if !player_loaded_sent {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID,
                    |_output| Ok(()),
                    compression_threshold,
                )?;
                player_loaded_sent = true;
                trace.player_loaded_sent = true;
            }
            if matches!(probe_mode, PlayProbeMode::Movement) && trace.movement_packets_sent == 0 {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec("movement probe player position"));
                };
                for offset in [0.25_f64, 0.5, 0.75] {
                    let moved = (
                        position.0 + offset,
                        position.1,
                        position.2,
                        position.3,
                        position.4,
                    );
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
                        |output| encode_move_player_pos_rot(output, moved, 1),
                        compression_threshold,
                    )?;
                    trace.movement_packets_sent += 1;
                    player_position = Some(moved);
                }
            }
            if matches!(probe_mode, PlayProbeMode::BlockAction)
                && trace.player_action_packets_sent == 0
            {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_abort_destroy_block(output, (0, 0, 0), 1),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::BlockBreak)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec("block break probe player position"));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 2),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::BlockBreakOutOfRange)
                && trace.player_action_packets_sent == 0
            {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, (1_000_000, 64, 1_000_000), 3),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalBreakAbort)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival break probe player position",
                    ));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 6),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_abort_destroy_block(output, target, 7),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(
                probe_mode,
                PlayProbeMode::SurvivalBreakStop
                    | PlayProbeMode::SurvivalStoneBreakStop
                    | PlayProbeMode::SurvivalOakPlanksBreakStop
            ) && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival break stop probe player position",
                    ));
                };
                let target = support_block_position(position);
                let (start_sequence, stop_sequence) =
                    if matches!(probe_mode, PlayProbeMode::SurvivalStoneBreakStop) {
                        (12, 13)
                    } else if matches!(probe_mode, PlayProbeMode::SurvivalOakPlanksBreakStop) {
                        (18, 19)
                    } else {
                        (8, 9)
                    };
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, start_sequence),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(650));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, stop_sequence),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(
                probe_mode,
                PlayProbeMode::SurvivalStoneWoodenPickaxeBreakStop
            ) && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival wooden pickaxe probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 20),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(2_200));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 21),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalDirtWoodenShovelBreakStop)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival wooden shovel probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 30),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(650));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 31),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalDelayedDestroy)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival delayed destroy probe player position",
                    ));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 14),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(100));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 15),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalSpawnProtection)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival spawn protection probe player position",
                    ));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 16),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::UseItemOn) && trace.use_item_on_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec("use item on probe player position"));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on(output, target, 4),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::UseItem) && trace.use_item_packets_sent == 0 {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 10, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalSnowballUse)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 22, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalWaterBucketUseOn)
                && trace.use_item_on_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival water bucket probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                survival_water_bucket_target_position = Some(target);
                survival_water_bucket_neighbor_position = Some((target.0, target.1 + 1, target.2));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on_hit(output, target, 1, (0.5, 1.0, 0.5), 23),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 24, 0.0, 90.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalEmptyBucketPickup)
                && trace.use_item_on_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival empty bucket pickup probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let support = support_block_position(position);
                let target = (support.0, support.1 + 1, support.2);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on_hit(output, target, 1, (0.5, 1.0, 0.5), 25),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 26, 0.0, 90.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalMilkBucketUse)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 27, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalHoneyBottleUse)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 28, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::SurvivalHoneyBottlePoisonCleanup)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 29, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::CreativeSnowballUse)
                && trace.creative_mode_slot_packets_sent == 0
            {
                let Some(snowball_item_registry_id) = trace.snowball_item_registry_id else {
                    return Err(ConnectionError::Codec(
                        "creative snowball use item registry id",
                    ));
                };
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                    |output| encode_creative_item_slot(output, 36, 1, snowball_item_registry_id),
                    compression_threshold,
                )?;
                trace.creative_mode_slot_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 11, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe_mode, PlayProbeMode::CreativePlace)
                && trace.creative_mode_slot_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "creative place probe player position",
                    ));
                };
                let Some(stone_item_registry_id) = trace.stone_item_registry_id else {
                    return Err(ConnectionError::Codec(
                        "creative place stone item registry id",
                    ));
                };
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                    |output| encode_creative_stone_slot(output, 36, stone_item_registry_id),
                    compression_threshold,
                )?;
                trace.creative_mode_slot_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;

                let target = support_block_position(position);
                creative_place_target_position = Some(target);
                creative_place_neighbor_position = Some((target.0, target.1, target.2 - 1));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on_hit(output, target, 2, (0.5, 0.5, 0.0), 5),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
            }
            world_ready_drain_started = Some(Instant::now());
            let drain_timeout = match probe_mode {
                PlayProbeMode::SurvivalDelayedDestroy => DELAYED_DESTROY_DRAIN_TIMEOUT,
                PlayProbeMode::SurvivalWaterBucketUseOn
                | PlayProbeMode::SurvivalEmptyBucketPickup => WATER_BUCKET_DRAIN_TIMEOUT,
                PlayProbeMode::SurvivalMilkBucketUse => CONSUMABLE_DRAIN_TIMEOUT,
                PlayProbeMode::SurvivalHoneyBottleUse => CONSUMABLE_DRAIN_TIMEOUT,
                PlayProbeMode::SurvivalHoneyBottlePoisonCleanup => CONSUMABLE_DRAIN_TIMEOUT,
                _ => WORLD_READY_DRAIN_TIMEOUT,
            };
            stream.set_read_timeout(Some(drain_timeout))?;
        }

        let drain_timeout = match probe_mode {
            PlayProbeMode::SurvivalDelayedDestroy => DELAYED_DESTROY_DRAIN_TIMEOUT,
            PlayProbeMode::SurvivalWaterBucketUseOn | PlayProbeMode::SurvivalEmptyBucketPickup => {
                WATER_BUCKET_DRAIN_TIMEOUT
            }
            PlayProbeMode::SurvivalMilkBucketUse => CONSUMABLE_DRAIN_TIMEOUT,
            PlayProbeMode::SurvivalHoneyBottleUse => CONSUMABLE_DRAIN_TIMEOUT,
            PlayProbeMode::SurvivalHoneyBottlePoisonCleanup => CONSUMABLE_DRAIN_TIMEOUT,
            _ => WORLD_READY_DRAIN_TIMEOUT,
        };
        if world_ready_drain_started.is_some_and(|started| started.elapsed() >= drain_timeout) {
            return Ok(trace);
        }
    }

    Err(ConnectionError::Io(io::Error::new(
        io::ErrorKind::TimedOut,
        "play world did not arrive before timeout",
    )))
}

fn send_configuration_start(
    stream: &mut TcpStream,
    compression_threshold: Option<usize>,
) -> Result<(), ConnectionError> {
    send_packet(
        stream,
        ConnectionProtocol::Configuration,
        CONFIG_CUSTOM_PAYLOAD_PACKET_ID,
        |output| {
            let id = crate::identifier::Identifier::parse("minecraft:brand")
                .map_err(|_| ConnectionError::Codec("brand identifier"))?;
            crate::identifier::write_identifier(&id, output)
                .map_err(|_| ConnectionError::Codec("brand identifier"))?;
            crate::utf8::write_utf8(
                output,
                &"vanilla".encode_utf16().collect::<Vec<_>>(),
                32_767,
            )
            .map_err(|_| ConnectionError::Codec("brand payload"))?;
            Ok(())
        },
        compression_threshold,
    )?;

    let information = ClientInformation::new(
        "en_us".encode_utf16().collect(),
        12,
        ChatVisibility::Full,
        true,
        127,
        MainHand::Right,
        false,
        true,
        ParticleStatus::All,
    );
    send_packet(
        stream,
        ConnectionProtocol::Configuration,
        CONFIG_CLIENT_INFORMATION_PACKET_ID,
        |output| {
            ServerboundClientInformationPacket::new(information)
                .encode(output)
                .map_err(|_| ConnectionError::Codec("client information"))
        },
        compression_threshold,
    )
}

fn encode_move_player_pos_rot(
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

fn encode_abort_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 1, position, 0, sequence)
}

fn encode_start_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 0, position, 1, sequence)
}

fn encode_stop_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 2, position, 1, sequence)
}

fn encode_use_item_on(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_use_item_on_hit(output, position, 1, (0.5, 1.0, 0.5), sequence)
}

fn encode_use_item(
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

fn encode_use_item_on_hit(
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

fn encode_creative_stone_slot(
    output: &mut Vec<u8>,
    slot: i16,
    stone_item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_creative_item_slot(output, slot, 1, stone_item_registry_id)
}

fn encode_creative_item_slot(
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

fn encode_set_carried_item(output: &mut Vec<u8>, slot: i16) -> Result<(), ConnectionError> {
    output.extend_from_slice(&slot.to_be_bytes());
    Ok(())
}

fn encode_player_action(
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

fn pack_block_position(position: (i32, i32, i32)) -> i64 {
    const HORIZONTAL_MASK: i64 = (1_i64 << 26) - 1;
    const Y_MASK: i64 = (1_i64 << 12) - 1;

    ((i64::from(position.0) & HORIZONTAL_MASK) << 38)
        | ((i64::from(position.2) & HORIZONTAL_MASK) << 12)
        | (i64::from(position.1) & Y_MASK)
}

fn support_block_position(position: (f64, f64, f64, f32, f32)) -> (i32, i32, i32) {
    let block = crate::binary::BlockPos::containing(position.0, position.1 - 1.0, position.2);
    (block.x, block.y, block.z)
}

fn connect_stream(address: SocketAddr, timeout: Duration) -> Result<TcpStream, ConnectionError> {
    let stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.set_nodelay(true)?;
    Ok(stream)
}

fn send_handshake_intention(
    stream: &mut TcpStream,
    address: SocketAddr,
    intent: ClientIntent,
) -> Result<(), ConnectionError> {
    let host = address.ip().to_string();
    let intention = ClientIntentionPacket::new(
        PROTOCOL_VERSION,
        host.encode_utf16().collect(),
        i32::from(address.port()),
        intent,
    );
    send_packet(
        stream,
        ConnectionProtocol::Handshaking,
        LOGIN_INTENTION_PACKET_ID,
        |payload| {
            intention
                .encode(payload)
                .map_err(|_| ConnectionError::Codec("handshake"))
        },
        None,
    )
}

fn send_packet<F>(
    stream: &mut TcpStream,
    _phase: ConnectionProtocol,
    packet_id: i32,
    encode_payload: F,
    compression_threshold: Option<usize>,
) -> Result<(), ConnectionError>
where
    F: FnOnce(&mut Vec<u8>) -> Result<(), ConnectionError>,
{
    let mut body = Vec::new();
    wire::write_varint(packet_id, &mut body);
    encode_payload(&mut body)?;
    let framed = encode_packet_frame(&body, compression_threshold)?;
    stream.write_all(&framed)?;
    stream.flush()?;
    Ok(())
}

fn read_packet(
    stream: &mut TcpStream,
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    let frame = read_frame(stream)?;
    decode_packet_frame(&frame, compression_threshold)
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, ConnectionError> {
    let mut header = Vec::with_capacity(MAX_VARINT21_BYTES);
    for _ in 0..MAX_VARINT21_BYTES {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte)?;
        header.push(byte[0]);
        if byte[0] & 0x80 == 0 {
            let mut input = header.as_slice();
            let length = wire::read_varint(&mut input)?;
            if length <= 0 {
                return Err(ConnectionError::Frame(if length == 0 {
                    FrameError::ZeroLength
                } else {
                    FrameError::PacketTooLarge
                }));
            }
            let length = usize::try_from(length)
                .map_err(|_| ConnectionError::Frame(FrameError::PacketTooLarge))?;
            if length > MAX_READABLE_FRAME_LENGTH {
                return Err(ConnectionError::Frame(FrameError::PacketTooLarge));
            }
            let mut body = vec![0_u8; length];
            stream.read_exact(&mut body)?;
            return Ok(body);
        }
    }
    Err(ConnectionError::Frame(FrameError::LengthTooWide))
}

fn encode_packet_frame(
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

fn decode_packet_frame(
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

fn split_packet(body: &[u8]) -> Result<(i32, &[u8]), ConnectionError> {
    let mut input = body;
    let packet_id = wire::read_varint(&mut input)?;
    Ok((packet_id, input))
}

fn parse_block_update_packet(input: &[u8]) -> Result<BlockUpdateObservation, ConnectionError> {
    if input.len() < 8 {
        return Err(ConnectionError::Codec("block update position"));
    }
    let (packed, input) = input.split_at(8);
    let position = crate::binary::BlockPos::unpack(u64::from_be_bytes(
        packed
            .try_into()
            .expect("checked block update position length"),
    ));
    let mut input = input;
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

fn parse_block_destruction_packet(
    input: &[u8],
) -> Result<BlockDestructionObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "block destruction entity id")?;
    if input.len() < 8 {
        return Err(ConnectionError::Codec("block destruction position"));
    }
    let (packed, rest) = input.split_at(8);
    input = rest;
    let position = crate::binary::BlockPos::unpack(u64::from_be_bytes(
        packed
            .try_into()
            .expect("checked block destruction position length"),
    ));
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

fn parse_add_entity_packet(input: &[u8]) -> Result<AddEntityObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "add entity id")?;
    if input.len() < 16 {
        return Err(ConnectionError::Codec("add entity uuid"));
    }
    input = &input[16..];
    let entity_type_id = read_varint(&mut input, "add entity type")?;
    let x = read_f64(&mut input, "add entity x")?;
    let y = read_f64(&mut input, "add entity y")?;
    let z = read_f64(&mut input, "add entity z")?;
    skip_lp_vec3(&mut input)?;
    for name in [
        "add entity x rotation",
        "add entity y rotation",
        "add entity head rotation",
    ] {
        read_byte(&mut input, name)?;
    }
    let data = read_varint(&mut input, "add entity data")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("add entity trailing payload"));
    }
    Ok(AddEntityObservation {
        entity_id,
        entity_type_id,
        x,
        y,
        z,
        data,
    })
}

fn parse_remove_mob_effect_packet(
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
    })
}

fn parse_update_mob_effect_packet(
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
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedItemStackObservation {
    item_count: Option<i32>,
    item_registry_id: Option<i32>,
    positive_component_type_ids: Vec<i32>,
    negative_component_type_ids: Vec<i32>,
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
        });
    }
    let item_registry_id = read_varint(input, name)?;
    let positive_component_count = read_count(input, name)?;
    let negative_component_count = read_count(input, name)?;
    let mut positive_component_type_ids = Vec::with_capacity(positive_component_count);
    for _ in 0..positive_component_count {
        let component_type_id = read_varint(input, name)?;
        match component_type_id {
            VANILLA_DAMAGE_COMPONENT_ID => {
                read_varint(input, name)?;
            }
            VANILLA_CUSTOM_MODEL_DATA_COMPONENT_ID => skip_custom_model_data(input, name)?,
            VANILLA_TOOL_COMPONENT_ID => skip_tool(input, name)?,
            VANILLA_WEAPON_COMPONENT_ID => skip_weapon(input, name)?,
            _ => return Err(ConnectionError::Codec(name)),
        }
        positive_component_type_ids.push(component_type_id);
    }
    let mut negative_component_type_ids = Vec::with_capacity(negative_component_count);
    for _ in 0..negative_component_count {
        negative_component_type_ids.push(read_varint(input, name)?);
    }
    Ok(ParsedItemStackObservation {
        item_count: Some(item_count),
        item_registry_id: Some(item_registry_id),
        positive_component_type_ids,
        negative_component_type_ids,
    })
}

fn skip_custom_model_data(input: &mut &[u8], name: &'static str) -> Result<(), ConnectionError> {
    let float_count = read_count(input, name)?;
    for _ in 0..float_count {
        read_f32(input, name)?;
    }
    let flag_count = read_count(input, name)?;
    for _ in 0..flag_count {
        read_byte(input, name)?;
    }
    let string_count = read_count(input, name)?;
    for _ in 0..string_count {
        read_length_prefixed_bytes(input, 131_068, name)?;
    }
    let color_count = read_count(input, name)?;
    for _ in 0..color_count {
        read_i32(input, name)?;
    }
    Ok(())
}

fn skip_tool(input: &mut &[u8], name: &'static str) -> Result<(), ConnectionError> {
    let rule_count = read_count(input, name)?;
    for _ in 0..rule_count {
        skip_holder_set(input, name)?;
        if read_byte(input, name)? != 0 {
            read_f32(input, name)?;
        }
        if read_byte(input, name)? != 0 {
            read_byte(input, name)?;
        }
    }
    read_f32(input, name)?;
    read_varint(input, name)?;
    read_byte(input, name)?;
    Ok(())
}

fn skip_holder_set(input: &mut &[u8], name: &'static str) -> Result<(), ConnectionError> {
    let encoded_count = read_varint(input, name)?;
    if encoded_count == 0 {
        read_length_prefixed_bytes(input, 131_068, name)?;
    } else if encoded_count > 0 {
        let holder_count =
            usize::try_from(encoded_count - 1).map_err(|_| ConnectionError::Codec(name))?;
        for _ in 0..holder_count {
            read_varint(input, name)?;
        }
    } else {
        return Err(ConnectionError::Codec(name));
    }
    Ok(())
}

fn skip_weapon(input: &mut &[u8], name: &'static str) -> Result<(), ConnectionError> {
    read_varint(input, name)?;
    read_f32(input, name)?;
    Ok(())
}

fn parse_container_set_content_packet(
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
    for slot in 0..item_count {
        let item = parse_optional_item_stack(&mut input, "container content item stack")?;
        if slot == 36 {
            hotbar_slot_zero_item_count = item.item_count;
            hotbar_slot_zero_item_registry_id = item.item_registry_id;
            hotbar_slot_zero_positive_component_type_ids = item.positive_component_type_ids;
            hotbar_slot_zero_negative_component_type_ids = item.negative_component_type_ids;
        }
    }
    let carried_item = parse_optional_item_stack(&mut input, "container content carried item")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container content trailing payload"));
    }
    Ok(ContainerSetContentObservation {
        container_id,
        state_id,
        item_count,
        hotbar_slot_zero_item_count,
        hotbar_slot_zero_item_registry_id,
        hotbar_slot_zero_positive_component_type_ids,
        hotbar_slot_zero_negative_component_type_ids,
        carried_item_count: carried_item.item_count,
        carried_item_registry_id: carried_item.item_registry_id,
        carried_positive_component_type_ids: carried_item.positive_component_type_ids,
        carried_negative_component_type_ids: carried_item.negative_component_type_ids,
    })
}

fn parse_container_set_slot_packet(
    input: &[u8],
) -> Result<ContainerSetSlotObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container slot container id")?;
    let state_id = read_varint(&mut input, "container slot state id")?;
    let slot = read_i16(&mut input, "container slot index")?;
    let item = parse_optional_item_stack(&mut input, "container slot item stack")?;
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
    })
}

fn parse_system_chat_packet(input: &[u8]) -> Result<SystemChatObservation, ConnectionError> {
    let mut input = input;
    let component = crate::nbt::read_nbt(&mut input)
        .map_err(|_| ConnectionError::Codec("system chat component"))?;
    let translation_key = component.and_then(|fields| {
        fields.into_iter().find_map(|field| {
            if field.key == "translate" {
                match field.value {
                    crate::nbt::FieldValue::Str(value) => Some(value),
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

fn skip_lp_vec3(input: &mut &[u8]) -> Result<(), ConnectionError> {
    let lowest = read_byte(input, "add entity movement marker")?;
    if lowest == 0 {
        return Ok(());
    }
    read_byte(input, "add entity movement middle")?;
    for _ in 0..4 {
        read_byte(input, "add entity movement high")?;
    }
    if lowest & 4 != 0 {
        read_varint(input, "add entity movement scale")?;
    }
    Ok(())
}

fn parse_chunk_packet(mut input: &[u8]) -> Result<ChunkPacketObservation, ConnectionError> {
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
    for _ in 0..block_entity_count {
        read_byte(&mut input, "block entity packed xz")?;
        read_i16(&mut input, "block entity y")?;
        read_varint(&mut input, "block entity type")?;
        crate::nbt::read_nbt(&mut input).map_err(|_| ConnectionError::Codec("block entity nbt"))?;
    }

    let sky_mask_word_count = read_long_array(&mut input, "sky light mask")?.len();
    let block_mask_word_count = read_long_array(&mut input, "block light mask")?.len();
    let empty_sky_mask_word_count = read_long_array(&mut input, "empty sky light mask")?.len();
    let empty_block_mask_word_count = read_long_array(&mut input, "empty block light mask")?.len();
    let sky_update_lengths = read_light_updates(&mut input, "sky light updates")?;
    let block_update_lengths = read_light_updates(&mut input, "block light updates")?;

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
        sky_mask_word_count,
        block_mask_word_count,
        empty_sky_mask_word_count,
        empty_block_mask_word_count,
        sky_update_count: sky_update_lengths.len(),
        sky_update_lengths,
        block_update_count: block_update_lengths.len(),
        block_update_lengths,
    })
}

fn read_varint(input: &mut &[u8], name: &'static str) -> Result<i32, ConnectionError> {
    wire::read_varint(input).map_err(|_| ConnectionError::Codec(name))
}

fn read_count(input: &mut &[u8], name: &'static str) -> Result<usize, ConnectionError> {
    let count = read_varint(input, name)?;
    usize::try_from(count).map_err(|_| ConnectionError::Codec(name))
}

fn read_byte(input: &mut &[u8], name: &'static str) -> Result<u8, ConnectionError> {
    let Some((&byte, rest)) = input.split_first() else {
        return Err(ConnectionError::Codec(name));
    };
    *input = rest;
    Ok(byte)
}

fn read_i16(input: &mut &[u8], name: &'static str) -> Result<i16, ConnectionError> {
    if input.len() < 2 {
        return Err(ConnectionError::Codec(name));
    }
    let (head, rest) = input.split_at(2);
    *input = rest;
    Ok(i16::from_be_bytes(
        head.try_into().expect("checked short length"),
    ))
}

fn read_i32(input: &mut &[u8], name: &'static str) -> Result<i32, ConnectionError> {
    if input.len() < 4 {
        return Err(ConnectionError::Codec(name));
    }
    let (head, rest) = input.split_at(4);
    *input = rest;
    Ok(i32::from_be_bytes(
        head.try_into().expect("checked int length"),
    ))
}

fn read_f32(input: &mut &[u8], name: &'static str) -> Result<f32, ConnectionError> {
    if input.len() < 4 {
        return Err(ConnectionError::Codec(name));
    }
    let (head, rest) = input.split_at(4);
    *input = rest;
    Ok(f32::from_be_bytes(
        head.try_into().expect("checked float length"),
    ))
}

fn read_f64(input: &mut &[u8], name: &'static str) -> Result<f64, ConnectionError> {
    if input.len() < 8 {
        return Err(ConnectionError::Codec(name));
    }
    let (head, rest) = input.split_at(8);
    *input = rest;
    Ok(f64::from_be_bytes(
        head.try_into().expect("checked double length"),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlayerPositionPacket {
    teleport_id: i32,
    x: f64,
    y: f64,
    z: f64,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    y_rot: f32,
    x_rot: f32,
    relative_flags: i32,
}

fn parse_player_position_packet(input: &[u8]) -> Result<PlayerPositionPacket, ConnectionError> {
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

fn resolve_player_position(
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

fn read_long_array(input: &mut &[u8], name: &'static str) -> Result<Vec<i64>, ConnectionError> {
    crate::binary::read_long_array(input).map_err(|_| ConnectionError::Codec(name))
}

fn read_length_prefixed_bytes<'a>(
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

fn read_light_updates(
    input: &mut &[u8],
    name: &'static str,
) -> Result<Vec<usize>, ConnectionError> {
    let count = read_count(input, name)?;
    let mut lengths = Vec::with_capacity(count);
    for _ in 0..count {
        let bytes = read_length_prefixed_bytes(input, 2_048, name)?;
        lengths.push(bytes.len());
    }
    Ok(lengths)
}

fn parse_section_data(mut input: &[u8]) -> Result<Vec<ChunkSectionObservation>, ConnectionError> {
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
            block_state_distinct_value_count: block_states.distinct_value_count,
            biome_bits_per_entry: biomes.bits_per_entry,
            biome_palette_entry_count: biomes.palette_entry_count,
            biome_palette_ids: biomes.palette_ids,
            biome_storage_word_count: biomes.storage_word_count,
            biome_first_values: biomes.first_values,
            biome_distinct_value_count: biomes.distinct_value_count,
        });
    }
    Ok(sections)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PaletteObservation {
    bits_per_entry: u8,
    palette_entry_count: usize,
    palette_ids: Vec<i32>,
    storage_word_count: usize,
    first_values: Vec<i32>,
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

    let (first_values, distinct_value_count) = decode_palette_values(
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

    Ok(PaletteObservation {
        bits_per_entry,
        palette_entry_count: palette_ids.len(),
        palette_ids,
        storage_word_count,
        first_values,
        distinct_value_count,
    })
}

fn read_fixed_long_array(
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

fn decode_palette_values(
    storage: &[u64],
    entry_count: usize,
    storage_bits: u8,
    palette_ids: &[i32],
    name: &'static str,
) -> Result<(Vec<i32>, usize), ConnectionError> {
    if storage_bits == 0 {
        let Some(&value) = palette_ids.first() else {
            return Err(ConnectionError::Codec(name));
        };
        return Ok((vec![value; entry_count.min(4)], 1));
    }

    let bits = usize::from(storage_bits);
    let values_per_long = 64 / bits;
    let mask = if bits == 32 {
        u64::from(u32::MAX)
    } else {
        (1_u64 << bits) - 1
    };
    let mut first_values = Vec::with_capacity(entry_count.min(4));
    let mut distinct_values = BTreeSet::new();
    for index in 0..entry_count {
        let word = storage[index / values_per_long];
        let palette_index = ((word >> ((index % values_per_long) * bits)) & mask) as usize;
        let value = if palette_ids.is_empty() {
            i32::try_from(palette_index).map_err(|_| ConnectionError::Codec(name))?
        } else {
            *palette_ids
                .get(palette_index)
                .ok_or(ConnectionError::Codec(name))?
        };
        if first_values.len() < 4 {
            first_values.push(value);
        }
        distinct_values.insert(value);
    }
    Ok((first_values, distinct_values.len()))
}

#[cfg(test)]
mod tests {
    use super::{
        PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID, PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
        PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID, PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
        PLAY_CLIENTBOUND_CHUNK_PACKET_ID, PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID, PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
        PLAY_CLIENTBOUND_LOGIN_PACKET_ID, PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID,
        PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID, PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID,
        PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID,
        PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID,
        PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID,
        PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
        PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID, PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
        PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID, PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
        PLAY_SERVERBOUND_USE_ITEM_PACKET_ID, STATUS_CLIENTBOUND_PONG_RESPONSE_PACKET_ID,
        STATUS_CLIENTBOUND_RESPONSE_PACKET_ID, STATUS_SERVERBOUND_PING_REQUEST_PACKET_ID,
        STATUS_SERVERBOUND_REQUEST_PACKET_ID, decode_packet_frame, encode_abort_destroy_block,
        encode_creative_item_slot, encode_creative_stone_slot, encode_packet_frame,
        encode_set_carried_item, encode_stop_destroy_block, encode_use_item, encode_use_item_on,
        encode_use_item_on_hit, pack_block_position, parse_add_entity_packet,
        parse_block_destruction_packet, parse_block_update_packet, parse_chunk_packet,
        parse_container_set_content_packet, parse_container_set_slot_packet,
        parse_player_position_packet, parse_remove_mob_effect_packet, parse_section_data,
        parse_system_chat_packet, parse_update_mob_effect_packet, resolve_player_position,
        support_block_position,
    };

    #[test]
    fn compression_uses_java_boundary_and_round_trips() {
        let below = vec![0x5a; 255];
        let at = vec![0xa5; 256];

        let below_frame = encode_packet_frame(&below, Some(256)).expect("frame below threshold");
        let mut below_header = below_frame.as_slice();
        crate::wire::read_varint(&mut below_header).expect("below frame header");
        assert_eq!(below_header[0], 0x00);
        assert_eq!(decode_packet_frame(below_header, Some(256)).unwrap(), below);

        let at_frame = encode_packet_frame(&at, Some(256)).expect("frame at threshold");
        let mut at_header = at_frame.as_slice();
        crate::wire::read_varint(&mut at_header).expect("at frame header");
        assert_ne!(at_header[0], 0x00);
        assert_eq!(decode_packet_frame(at_header, Some(256)).unwrap(), at);
    }

    #[test]
    fn play_packet_ids_match_java_26_2_dispatch_order() {
        assert_eq!(PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID, 1);
        assert_eq!(PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID, 44);
        assert_eq!(PLAY_CLIENTBOUND_CHUNK_PACKET_ID, 45);
        assert_eq!(PLAY_CLIENTBOUND_LOGIN_PACKET_ID, 49);
        assert_eq!(PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID, 72);
        assert_eq!(PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID, 8);
        assert_eq!(PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID, 4);
        assert_eq!(PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID, 5);
        assert_eq!(PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID, 18);
        assert_eq!(PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID, 20);
        assert_eq!(PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID, 78);
        assert_eq!(PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, 121);
        assert_eq!(PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID, 132);
        assert_eq!(PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, 0);
        assert_eq!(PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID, 41);
        assert_eq!(PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, 11);
        assert_eq!(PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID, 28);
        assert_eq!(PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, 31);
        assert_eq!(PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID, 44);
        assert_eq!(PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, 53);
        assert_eq!(PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID, 56);
        assert_eq!(PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID, 66);
        assert_eq!(PLAY_SERVERBOUND_USE_ITEM_PACKET_ID, 67);
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
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(0, &mut payload);
        for _ in 0..6 {
            crate::wire::write_varint(0, &mut payload);
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
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(1, &mut payload);
        payload.push(0x21);
        payload.extend_from_slice(&(-64_i16).to_be_bytes());
        crate::wire::write_varint(0, &mut payload);
        payload.push(0);
        for _ in 0..6 {
            crate::wire::write_varint(0, &mut payload);
        }

        let observation = parse_chunk_packet(&payload).expect("nullable block entity tag");
        assert_eq!(observation.block_entity_count, 1);
    }

    #[test]
    fn chunk_packet_parser_records_heightmap_and_light_lengths() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&4_i32.to_be_bytes());
        payload.extend_from_slice(&(-9_i32).to_be_bytes());

        crate::wire::write_varint(2, &mut payload);
        crate::wire::write_varint(1, &mut payload);
        crate::wire::write_varint(2, &mut payload);
        payload.extend_from_slice(&[0; 16]);
        crate::wire::write_varint(4, &mut payload);
        crate::wire::write_varint(3, &mut payload);
        payload.extend_from_slice(&[0; 24]);

        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(0, &mut payload);
        for _ in 0..4 {
            crate::wire::write_varint(0, &mut payload);
        }

        crate::wire::write_varint(2, &mut payload);
        crate::wire::write_varint(2_048, &mut payload);
        payload.extend_from_slice(&[0xa5; 2_048]);
        crate::wire::write_varint(3, &mut payload);
        payload.extend_from_slice(&[0x5a; 3]);

        crate::wire::write_varint(1, &mut payload);
        crate::wire::write_varint(2_048, &mut payload);
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
        crate::wire::write_varint(17, &mut payload);
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
    fn player_action_encoder_matches_java_abort_field_order() {
        let mut payload = Vec::new();
        encode_abort_destroy_block(&mut payload, (-12, 67, 9), 1).expect("action packet");

        let mut input = payload.as_slice();
        assert_eq!(crate::wire::read_varint(&mut input).expect("action"), 1);
        let (packed, rest) = input.split_at(8);
        input = rest;
        assert_eq!(
            i64::from_be_bytes(packed.try_into().expect("packed block position")),
            pack_block_position((-12, 67, 9))
        );
        assert_eq!(input[0], 0);
        input = &input[1..];
        assert_eq!(crate::wire::read_varint(&mut input).expect("sequence"), 1);
        assert!(input.is_empty());
    }

    #[test]
    fn player_action_encoder_matches_java_stop_action() {
        let mut payload = Vec::new();
        encode_stop_destroy_block(&mut payload, (-12, 67, 9), 9).expect("stop action packet");

        let mut input = payload.as_slice();
        assert_eq!(crate::wire::read_varint(&mut input).expect("action"), 2);
        input = &input[8..];
        assert_eq!(input[0], 1);
        input = &input[1..];
        assert_eq!(crate::wire::read_varint(&mut input).expect("sequence"), 9);
        assert!(input.is_empty());
    }

    #[test]
    fn block_update_parser_matches_java_packed_position_and_state() {
        let mut payload = pack_block_position((-12, -63, 9)).to_be_bytes().to_vec();
        crate::wire::write_varint(123, &mut payload);

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
    fn block_destruction_parser_matches_java_progress_fields() {
        let mut payload = Vec::new();
        crate::wire::write_varint(37, &mut payload);
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
        crate::wire::write_varint(37, &mut payload);
        payload.extend_from_slice(&[0; 16]);
        crate::wire::write_varint(120, &mut payload);
        for value in [1.25_f64, 80.0, -35.5] {
            payload.extend_from_slice(&value.to_be_bytes());
        }
        payload.push(0);
        payload.extend_from_slice(&[0, 0, 0]);
        crate::wire::write_varint(0, &mut payload);

        assert_eq!(
            parse_add_entity_packet(&payload).expect("add entity"),
            super::AddEntityObservation {
                entity_id: 37,
                entity_type_id: 120,
                x: 1.25,
                y: 80.0,
                z: -35.5,
                data: 0,
            }
        );
    }

    #[test]
    fn mob_effect_parsers_match_java_registry_and_effect_fields() {
        let mut update_payload = Vec::new();
        crate::wire::write_varint(42, &mut update_payload);
        crate::wire::write_varint(19, &mut update_payload);
        crate::wire::write_varint(1, &mut update_payload);
        crate::wire::write_varint(1_200, &mut update_payload);
        update_payload.push(0b1111);

        assert_eq!(
            parse_update_mob_effect_packet(&update_payload).expect("mob effect update"),
            super::MobEffectUpdateObservation {
                entity_id: 42,
                effect_registry_id: 19,
                amplifier: 1,
                duration_ticks: 1_200,
                flags: 0b1111,
            }
        );

        let mut remove_payload = Vec::new();
        crate::wire::write_varint(42, &mut remove_payload);
        crate::wire::write_varint(19, &mut remove_payload);

        assert_eq!(
            parse_remove_mob_effect_packet(&remove_payload).expect("mob effect removal"),
            super::MobEffectRemoveObservation {
                entity_id: 42,
                effect_registry_id: 19,
            }
        );
    }

    #[test]
    fn system_chat_parser_matches_translatable_overlay_component() {
        let mut payload = Vec::new();
        crate::nbt::write_nbt(
            Some(&[crate::nbt::Field {
                key: "translate".to_owned(),
                value: crate::nbt::FieldValue::Str("build.spawn_protection".to_owned()),
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
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(7, &mut payload);
        payload.extend_from_slice(&36_i16.to_be_bytes());
        crate::wire::write_varint(0, &mut payload);

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
            }
        );
    }

    #[test]
    fn container_slot_parser_skips_custom_model_data_component() {
        let mut payload = Vec::new();
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(8, &mut payload);
        payload.extend_from_slice(&36_i16.to_be_bytes());
        crate::wire::write_varint(1, &mut payload);
        crate::wire::write_varint(1_046, &mut payload);
        crate::wire::write_varint(1, &mut payload);
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(17, &mut payload);
        crate::wire::write_varint(1, &mut payload);
        payload.extend_from_slice(&1.0_f32.to_be_bytes());
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(0, &mut payload);

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
            }
        );
    }

    #[test]
    fn container_slot_parser_skips_damage_tool_and_weapon_components() {
        let mut payload = Vec::new();
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(9, &mut payload);
        payload.extend_from_slice(&36_i16.to_be_bytes());
        crate::wire::write_varint(1, &mut payload);
        crate::wire::write_varint(1_000, &mut payload);
        crate::wire::write_varint(3, &mut payload);
        crate::wire::write_varint(0, &mut payload);

        crate::wire::write_varint(3, &mut payload);
        crate::wire::write_varint(1, &mut payload);

        crate::wire::write_varint(28, &mut payload);
        crate::wire::write_varint(2, &mut payload);
        let incorrect_tag = b"minecraft:incorrect_for_wooden_tool";
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(incorrect_tag.len() as i32, &mut payload);
        payload.extend_from_slice(incorrect_tag);
        payload.push(0);
        payload.push(1);
        payload.push(0);
        crate::wire::write_varint(0, &mut payload);
        let mineable_tag = b"minecraft:mineable/shovel";
        crate::wire::write_varint(mineable_tag.len() as i32, &mut payload);
        payload.extend_from_slice(mineable_tag);
        payload.push(1);
        payload.extend_from_slice(&2.0_f32.to_be_bytes());
        payload.push(1);
        payload.push(1);
        payload.extend_from_slice(&1.0_f32.to_be_bytes());
        crate::wire::write_varint(1, &mut payload);
        payload.push(1);

        crate::wire::write_varint(29, &mut payload);
        crate::wire::write_varint(2, &mut payload);
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
            }
        );
    }

    #[test]
    fn container_content_parser_matches_empty_hotbar_stack() {
        let mut payload = Vec::new();
        crate::wire::write_varint(0, &mut payload);
        crate::wire::write_varint(9, &mut payload);
        crate::wire::write_varint(37, &mut payload);
        for _ in 0..37 {
            crate::wire::write_varint(0, &mut payload);
        }
        crate::wire::write_varint(0, &mut payload);

        assert_eq!(
            parse_container_set_content_packet(&payload).expect("container content packet"),
            super::ContainerSetContentObservation {
                container_id: 0,
                state_id: 9,
                item_count: 37,
                hotbar_slot_zero_item_count: None,
                hotbar_slot_zero_item_registry_id: None,
                hotbar_slot_zero_positive_component_type_ids: Vec::new(),
                hotbar_slot_zero_negative_component_type_ids: Vec::new(),
                carried_item_count: None,
                carried_item_registry_id: None,
                carried_positive_component_type_ids: Vec::new(),
                carried_negative_component_type_ids: Vec::new(),
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
        assert_eq!(crate::wire::read_varint(&mut input).expect("hand"), 0);
        let (packed, rest) = input.split_at(8);
        input = rest;
        assert_eq!(
            i64::from_be_bytes(packed.try_into().expect("packed block position")),
            pack_block_position((-12, 67, 9))
        );
        assert_eq!(crate::wire::read_varint(&mut input).expect("direction"), 1);
        assert_eq!(input.len(), 4 * 3 + 2 + 1);
        assert_eq!(&input[0..4], &0.5_f32.to_be_bytes());
        assert_eq!(&input[4..8], &1.0_f32.to_be_bytes());
        assert_eq!(&input[8..12], &0.5_f32.to_be_bytes());
        assert_eq!(input[12], 0);
        assert_eq!(input[13], 0);
        input = &input[14..];
        assert_eq!(crate::wire::read_varint(&mut input).expect("sequence"), 4);
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
        assert_eq!(crate::wire::read_varint(&mut input).expect("item count"), 1);
        assert_eq!(
            crate::wire::read_varint(&mut input).expect("item registry id"),
            1
        );
        assert_eq!(
            crate::wire::read_varint(&mut input).expect("positive components"),
            0
        );
        assert_eq!(
            crate::wire::read_varint(&mut input).expect("negative components"),
            0
        );
        assert!(input.is_empty());
    }

    #[test]
    fn creative_item_slot_encoder_keeps_registry_id_and_empty_components() {
        let mut payload = Vec::new();
        encode_creative_item_slot(&mut payload, 36, 1, 1_044).expect("creative item slot");

        let mut input = payload.as_slice();
        assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
        input = &input[2..];
        assert_eq!(crate::wire::read_varint(&mut input).expect("item count"), 1);
        assert_eq!(
            crate::wire::read_varint(&mut input).expect("item registry id"),
            1_044
        );
        assert_eq!(
            crate::wire::read_varint(&mut input).expect("positive components"),
            0
        );
        assert_eq!(
            crate::wire::read_varint(&mut input).expect("negative components"),
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
    fn side_use_item_on_encoder_keeps_click_position_and_face() {
        let mut payload = Vec::new();
        encode_use_item_on_hit(&mut payload, (2, 70, -4), 2, (0.5, 0.5, 0.0), 5)
            .expect("side use item on packet");

        let mut input = payload.as_slice();
        assert_eq!(crate::wire::read_varint(&mut input).expect("hand"), 0);
        input = &input[8..];
        assert_eq!(
            crate::wire::read_varint(&mut input).expect("north direction"),
            2
        );
        assert_eq!(&input[0..4], &0.5_f32.to_be_bytes());
        assert_eq!(&input[4..8], &0.5_f32.to_be_bytes());
        assert_eq!(&input[8..12], &0.0_f32.to_be_bytes());
        assert_eq!(input[12], 0);
        assert_eq!(input[13], 0);
        input = &input[14..];
        assert_eq!(crate::wire::read_varint(&mut input).expect("sequence"), 5);
        assert!(input.is_empty());
    }

    #[test]
    fn use_item_encoder_matches_java_field_order() {
        let mut payload = Vec::new();
        encode_use_item(&mut payload, 10, 30.0, -15.0).expect("use item packet");

        let mut input = payload.as_slice();
        assert_eq!(crate::wire::read_varint(&mut input).expect("hand"), 0);
        assert_eq!(crate::wire::read_varint(&mut input).expect("sequence"), 10);
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
            crate::wire::write_varint(0, &mut payload);
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
        crate::wire::write_varint(2, &mut section);
        crate::wire::write_varint(0, &mut section);
        crate::wire::write_varint(3, &mut section);
        section.extend_from_slice(&0x101_u64.to_be_bytes());
        section.resize(section.len() + 255 * 8, 0);

        section.push(3);
        crate::wire::write_varint(2, &mut section);
        crate::wire::write_varint(0, &mut section);
        crate::wire::write_varint(1, &mut section);
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
        assert_eq!(observation.block_state_distinct_value_count, 2);
        assert_eq!(observation.biome_bits_per_entry, 3);
        assert_eq!(observation.biome_palette_entry_count, 2);
        assert_eq!(observation.biome_palette_ids, [0, 1]);
        assert_eq!(observation.biome_storage_word_count, 4);
        assert_eq!(observation.biome_first_values, [1, 1, 1, 1]);
        assert_eq!(observation.biome_distinct_value_count, 2);
    }

    #[test]
    fn section_parser_reads_zero_bit_palettes_without_storage() {
        let mut section = Vec::new();
        section.extend_from_slice(&4096_i16.to_be_bytes());
        section.extend_from_slice(&0_i16.to_be_bytes());
        section.push(0);
        crate::wire::write_varint(0, &mut section);
        section.push(0);
        crate::wire::write_varint(0, &mut section);

        let observations = parse_section_data(&section).expect("single-valued section");
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].block_state_bits_per_entry, 0);
        assert_eq!(observations[0].block_state_palette_entry_count, 1);
        assert_eq!(observations[0].block_state_palette_ids, [0]);
        assert_eq!(observations[0].block_state_storage_word_count, 0);
        assert_eq!(observations[0].block_state_first_values, [0, 0, 0, 0]);
        assert_eq!(observations[0].block_state_distinct_value_count, 1);
        assert_eq!(observations[0].biome_bits_per_entry, 0);
        assert_eq!(observations[0].biome_palette_entry_count, 1);
        assert_eq!(observations[0].biome_palette_ids, [0]);
        assert_eq!(observations[0].biome_storage_word_count, 0);
        assert_eq!(observations[0].biome_first_values, [0, 0, 0, 0]);
        assert_eq!(observations[0].biome_distinct_value_count, 1);
    }
}
