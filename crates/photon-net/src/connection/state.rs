//! What a connection learns while it logs in and configures, and keeps for the session.

use std::collections::BTreeMap;

use photon_core::binary::Uuid;
use photon_core::tags::ClientboundUpdateTagsPacket;
use photon_protocol::cookie::CookieStore;
use photon_protocol::play::SpawnInfoObservation;
use photon_world::{ClientWorld, DimensionBounds, DimensionEnvironment};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConnectionState {
    /// The version the connection speaks, chosen before it opens ([`super::select_version`]).
    /// The newest until then: a path that builds a state and skips the choice speaks the newest.
    pub version: photon_core::version::Version,
    /// The protocol number the server list saw the server answer with, when it did: the
    /// connection takes its version from it instead of asking the server.
    pub protocol_hint: Option<i32>,
    /// The account the connection logs in with, where the server checks accounts.
    pub account: Option<crate::account::Account>,
    /// Whether this login went through the session server (the server checked the account). A
    /// version whose login packet has no online-mode flag (26.1) is online exactly then.
    pub authenticated: std::cell::Cell<bool>,
    /// The profile id the server logged the player in with.
    pub profile_id: Option<Uuid>,
    /// The name of that profile, which can differ in case from the one the client asked for.
    pub profile_name: Option<String>,
    /// What the server asked the client to keep, which stays through a reconfiguration.
    pub cookies: CookieStore,
    /// The host name the handshake gives, where it is not the address's own, and whether the
    /// connection is one a server sent the client on to (`ClientIntent::Transfer`).
    pub handshake_host: Option<String>,
    pub transferring: bool,
    /// The server's brand (`minecraft:brand`).
    pub server_brand: Option<String>,
    /// `ClientboundCustomReportDetailsPacket`: what the server wants a crash report to carry.
    pub custom_report_details: Vec<(String, String)>,
    /// `ClientboundServerLinksPacket`: the addresses the server offers, once checked.
    pub server_links: photon_game::server_links::ServerLinks,
    /// Every synchronised registry's entry names, in the order the server numbered them.
    pub registry_entry_names: BTreeMap<String, Vec<String>>,
    /// The dialog registry's entries, in network order; one without data (a known pack's own) is
    /// `None`.
    pub dialog_registry: Vec<Option<photon_game::dialog::Dialog>>,
    pub dimension_type_registry_bounds: Vec<Option<DimensionBounds>>,
    pub dimension_type_registry_environments: Vec<Option<DimensionEnvironment>>,
    /// The server packs the client keeps; the same source goes through every connection.
    pub packs: crate::server_packs::PackSource,
    /// The tags the configuration phase sent, for the world they belong to.
    pub tags: Vec<ClientboundUpdateTagsPacket>,
    /// `ClientPacketListener.seenPlayers` and `seenInsecureChatWarning`, which go on to the
    /// connection a reconfiguration or a transfer makes.
    pub seen_players: BTreeMap<(u64, u64), String>,
    pub seen_insecure_warning: bool,
    /// The server reset the chat in the configuration phase (`ClientboundResetChatPacket`), so
    /// the chat is not kept through a reconfiguration.
    pub chat_reset: bool,
    /// Whether the server enables the minecart improvements, if it said.
    pub minecart_improvements: Option<bool>,
    /// Where the login has come to, for a screen that shows it
    /// (`ClientHandshakePacketListenerImpl.updateStatus`).
    pub login_stage: Option<LoginStage>,
}

/// The cell a connection writes its login stage to.
#[derive(Debug, Clone, Default)]
pub struct LoginStage(pub std::sync::Arc<std::sync::atomic::AtomicU8>);

impl PartialEq for LoginStage {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}

/// The login stages a connection reports, by `ClientHandshakePacketListenerImpl.State`.
pub mod stage {
    pub const CONNECTING: u8 = 0;
    pub const AUTHORIZING: u8 = 1;
    pub const ENCRYPTING: u8 = 2;
    pub const JOINING: u8 = 3;
    /// A login custom query was answered (`connect.negotiating`).
    pub const NEGOTIATING: u8 = 4;
}

impl ConnectionState {
    /// Tell a screen that the login has come to `stage`.
    pub(super) fn reach(&self, stage: u8) {
        if let Some(shared) = &self.login_stage {
            shared.0.store(stage, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

impl ConnectionState {
    /// A world with what the configuration phase decided about it.
    pub fn new_world(&self) -> ClientWorld {
        let mut world = ClientWorld::default();
        self.configure_world(&mut world);
        world
    }

    /// What the configuration phase decided about the world it will be played in.
    pub fn configure_world(&self, world: &mut ClientWorld) {
        for tags in &self.tags {
            world.apply_update_tags_packet(tags);
        }
        if let Some(enabled) = self.minecart_improvements {
            world.set_minecart_improvements(enabled);
        }
    }
}

impl ConnectionState {
    /// The vertical extent of the dimension a spawn info names: the registry's entry for its
    /// dimension type, or the built-in dimension's.
    pub fn dimension_bounds(&self, spawn_info: &SpawnInfoObservation) -> DimensionBounds {
        usize::try_from(spawn_info.dimension_type_id)
            .ok()
            .and_then(|id| self.dimension_type_registry_bounds.get(id))
            .and_then(|bounds| *bounds)
            .unwrap_or_else(|| DimensionBounds::for_builtin_dimension(&spawn_info.dimension))
    }

    /// What the dimension a spawn info names is like (its light, sky and fog).
    pub fn dimension_environment(&self, spawn_info: &SpawnInfoObservation) -> DimensionEnvironment {
        usize::try_from(spawn_info.dimension_type_id)
            .ok()
            .and_then(|id| self.dimension_type_registry_environments.get(id))
            .and_then(|environment| environment.clone())
            .unwrap_or_else(|| DimensionEnvironment::for_builtin_dimension(&spawn_info.dimension))
    }
}
