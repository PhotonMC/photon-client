use super::*;

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
    pub profile_id: Option<photon_core::binary::Uuid>,
    pub configuration_finished: bool,
    /// What the server asked the client to keep, which stays through a reconfiguration.
    pub cookies: crate::cookie::CookieStore,
    /// The host name the handshake gives, where it is not the address's own, and whether the
    /// connection is one a server sent the client on to (`ClientIntent::Transfer`).
    pub handshake_host: Option<String>,
    pub transferring: bool,
    /// The server's brand (`minecraft:brand`).
    pub server_brand: Option<String>,
    /// `ClientboundCustomReportDetailsPacket`: what the server wants a crash report to carry.
    pub custom_report_details: Vec<(String, String)>,
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

/// The observable result of a status and ping session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusConnectionTrace {
    pub packets: Vec<PacketObservation>,
    pub status_response: String,
    pub pong_time: i64,
    /// From the ping request sent to its pong read, in milliseconds (`ServerData.ping`).
    pub round_trip_ms: i64,
}
