use photon_net::connection::{ConnectionState, Recorder};
use photon_protocol::phase::ConnectionProtocol;
use photon_protocol::play::*;
use photon_world::{ClientWorld, DimensionBounds, DimensionEnvironment};

/// One received packet after the outer frame and optional compression were removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketObservation {
    pub phase: ConnectionProtocol,
    pub id: i32,
    pub body_length: usize,
}

/// The observable result of the initial real-server session.
#[derive(Debug, Default)]
pub struct PlayConnectionTrace {
    pub packets: Vec<PacketObservation>,
    pub compression_threshold: Option<usize>,
    pub login_finished: bool,
    /// The profile id the server logged the player in with.
    pub profile_id: Option<photon_core::binary::Uuid>,
    pub configuration_finished: bool,
    /// What the server asked the client to keep, which stays through a reconfiguration.
    pub cookies: photon_protocol::cookie::CookieStore,
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
    /// The dialog registry's entries, in network order; one without data (a known pack's own) is
    /// `None`.
    pub dialog_registry: Vec<Option<photon_game::dialog::Dialog>>,
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
    pub stone_item_registry_id: Option<photon_core::ids::ItemId>,
    pub snowball_item_registry_id: Option<photon_core::ids::ItemId>,
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

/// What the login and configuration exchange showed, as the probes watch it.
#[derive(Debug, Default)]
pub(crate) struct LoginTrace {
    packets: Vec<PacketObservation>,
    login_finished: bool,
    registry_packet_count: usize,
    registry_entry_count: usize,
    update_tags_packet_count: usize,
    enabled_feature_count: usize,
    known_pack_count: usize,
    selected_known_pack_count: usize,
    configuration_finished: bool,
}

impl Recorder for LoginTrace {
    fn packet(&mut self, phase: ConnectionProtocol, id: i32, body_length: usize) {
        self.packets.push(PacketObservation {
            phase,
            id,
            body_length,
        });
    }

    fn login_finished(&mut self) {
        self.login_finished = true;
    }

    fn registry_data(&mut self, entries: usize) {
        self.registry_packet_count += 1;
        self.registry_entry_count += entries;
    }

    fn update_tags(&mut self) {
        self.update_tags_packet_count += 1;
    }

    fn enabled_features(&mut self, count: usize) {
        self.enabled_feature_count = count;
    }

    fn known_packs(&mut self, offered: usize, selected: usize) {
        self.known_pack_count = offered;
        self.selected_known_pack_count = selected;
    }

    fn configuration_finished(&mut self) {
        self.configuration_finished = true;
    }
}

impl PlayConnectionTrace {
    /// Take what logging in and configuring left behind: the packets seen, and what the server
    /// told the client to keep, which also decides what the world starts with.
    pub(crate) fn absorb_login(
        &mut self,
        (state, login): (ConnectionState, LoginTrace),
        compression_threshold: Option<usize>,
    ) {
        state.configure_world(&mut self.world);
        self.compression_threshold = compression_threshold;
        self.packets = login.packets;
        self.login_finished = login.login_finished;
        self.registry_packet_count = login.registry_packet_count;
        self.registry_entry_count = login.registry_entry_count;
        self.update_tags_packet_count = login.update_tags_packet_count;
        self.enabled_feature_count = login.enabled_feature_count;
        self.known_pack_count = login.known_pack_count;
        self.selected_known_pack_count = login.selected_known_pack_count;
        self.configuration_finished = login.configuration_finished;
        self.profile_id = state.profile_id;
        self.cookies = state.cookies;
        self.server_brand = state.server_brand;
        self.custom_report_details = state.custom_report_details;
        self.registry_entry_names = state.registry_entry_names;
        self.dialog_registry = state.dialog_registry;
        self.dimension_type_registry_bounds = state.dimension_type_registry_bounds;
        self.dimension_type_registry_environments = state.dimension_type_registry_environments;
    }
}

/// Log in and configure for a probe, which watches the exchange: the stream and the compression
/// threshold, with what was seen kept in `trace`.
pub(crate) fn open_for_probe(
    (address, username): (std::net::SocketAddr, &str),
    (timeout, select_known_packs): (std::time::Duration, bool),
    trace: &mut PlayConnectionTrace,
) -> Result<(photon_net::connection::Transport, Option<usize>), ConnectionError> {
    let mut state = ConnectionState::default();
    let mut login = LoginTrace::default();
    let (stream, compression_threshold) = photon_net::connection::open_play_connection(
        (address, username),
        timeout,
        select_known_packs,
        &photon_protocol::configuration::ClientInformation::default(),
        (&mut state, &mut login),
    )?;
    trace.absorb_login((state, login), compression_threshold);
    Ok((stream, compression_threshold))
}
