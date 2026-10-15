//! What a connection learns while it logs in and configures, and keeps for the session.

use std::collections::BTreeMap;

use photon_core::binary::Uuid;
use photon_core::tags::ClientboundUpdateTagsPacket;
use photon_protocol::cookie::CookieStore;
use photon_protocol::play::SpawnInfoObservation;
use photon_world::{ClientWorld, DimensionBounds, DimensionEnvironment};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConnectionState {
    /// The profile id the server logged the player in with.
    pub profile_id: Option<Uuid>,
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
    /// Every synchronised registry's entry names, in the order the server numbered them.
    pub registry_entry_names: BTreeMap<String, Vec<String>>,
    /// The dialog registry's entries, in network order; one without data (a known pack's own) is
    /// `None`.
    pub dialog_registry: Vec<Option<photon_game::dialog::Dialog>>,
    pub dimension_type_registry_bounds: Vec<Option<DimensionBounds>>,
    pub dimension_type_registry_environments: Vec<Option<DimensionEnvironment>>,
    /// The tags the configuration phase sent, for the world they belong to.
    pub tags: Vec<ClientboundUpdateTagsPacket>,
    /// Whether the server enables the minecart improvements, if it said.
    pub minecart_improvements: Option<bool>,
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
