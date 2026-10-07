use super::*;

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
