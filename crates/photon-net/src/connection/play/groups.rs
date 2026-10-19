//! What a play session keeps, by what it is about. Each part is what a system of its own will take
//! as a resource of the ECS world.

use super::*;
use photon_core::ids::EntityId;

/// The connection itself: the socket, how it is framed and who is on it.
pub(super) struct Link {
    pub(super) writer: PacketWriter,
    pub(super) compression_threshold: Option<usize>,
    pub(super) incoming: Receiver<Result<Incoming, ConnectionError>>,
    pub(super) username: String,
    pub(super) auto_jump: bool,
    /// What the server was last told of the client (`remoteClientInformation`).
    pub(super) client_information: photon_protocol::configuration::ClientInformation,
    /// Why the connection is over, once it is: the server's word or what
    /// the connection said (`onDisconnect`).
    pub(super) disconnect: Option<photon_game::text::Component>,
    pub(super) warned: Option<String>,
    /// `Connection`'s packet counts and their averages.
    pub(super) traffic: super::traffic::Traffic,
}

/// The local player and what the server last said of it.
pub(super) struct LocalState {
    pub(super) player_id: Option<EntityId>,
    pub(super) game_mode: ClientPlayerGameMode,
    /// `MultiPlayerGameMode.previousLocalPlayerMode`: the mode played in before this one.
    pub(super) previous_game_mode: Option<ClientPlayerGameMode>,
    pub(super) abilities: Option<ClientPlayerAbilities>,
    pub(super) player: Option<LocalPlayer>,
    pub(super) status: PlayerStatus,
    /// The player's operator level, 0 to 4 (`LocalPlayer.permissions`), which
    /// the server tells by entity events 24 to 28.
    pub(super) permission_level: i32,
    /// Whether the level is hardcore, as the login said.
    pub(super) hardcore: bool,
    /// `Player.isReducedDebugInfo`: the login's word, then entity events 22 and 23.
    pub(super) reduced_debug_info: bool,
    /// `ClientAdvancements`: the tree the server told of and how far the player is.
    pub(super) advancements: photon_game::advancements::ClientAdvancements,
    /// The advancements done by updates whose toasts the screen is yet to show.
    pub(super) advancement_toasts: Vec<String>,
    /// `LocalPlayer.getStats` (`StatsCounter`): what `AwardStats` last said, kept across respawns.
    pub(super) stats: std::collections::HashMap<StatKey, i32>,
    /// An `AwardStats` was handled since the statistics screen last looked.
    pub(super) stats_updated: bool,
    /// `LocalPlayer.shouldShowDeathScreen`: off when the level respawns at once.
    pub(super) show_death_screen: bool,
    /// The message of a death the death screen is yet to show.
    pub(super) death: Option<photon_game::text::Component>,
    /// A `WIN_GAME` event the credits screen is yet to show.
    pub(super) win_game: bool,
    /// Whether a health update has arrived since the player last respawned.
    pub(super) health_received: bool,
    /// The portal effect's intensities and the cooldown a respawn gives the player to come.
    pub(super) respawn_portal: Option<((f32, f32), i32)>,
    /// The entity the crosshair last landed on (`Minecraft.hitResult`).
    pub(super) aimed_entity: Option<EntityId>,
    /// The hotbar slot the server was last told about.
    pub(super) sent_slot: i32,
    /// Breaking state and the miss wait.
    pub(super) game_mode_state: photon_game::game_mode::GameMode,
    /// `LocalPlayer.dropSpamThrottler`: what creative drops have added up.
    pub(super) drop_spam: i32,
    /// The vehicle the server removed with the local player aboard (`removedPlayerVehicleId`).
    pub(super) removed_player_vehicle: Option<EntityId>,
    pub(super) teleports: u32,
    /// Swing packets sent, for scripted runs to report.
    pub(super) swings: u32,
    pub(super) loaded: bool,
    /// `Player.getCooldowns`.
    pub(super) item_cooldowns: photon_game::item::cooldowns::ItemCooldowns,
}

/// The menus the server opened and what they read.
#[derive(Default)]
pub(super) struct Menus {
    /// The player's inventory menu, whose drag state lasts between clicks.
    pub(super) container_menu: photon_game::menus::container::ContainerMenu,
    /// The menu of the container the server opened, likewise.
    pub(super) open_menu: Option<photon_game::menus::container::ContainerMenu>,
    /// What the server last said of the merchant whose menu is open, by container.
    pub(super) merchant: Option<(i32, photon_game::menus::merchant::MerchantState)>,
    /// The mount whose screen the server opened, by container: what its menu reads of it.
    pub(super) mount: Option<(
        i32,
        photon_game::menus::container::Mount,
        photon_game::menus::container::MountEntity,
    )>,
    /// The sign the server last asked the player to write on, and whether its front, until the
    /// client opens its screen.
    pub(super) sign_editor: Option<((i32, i32, i32), bool)>,
    /// The hand whose book the server last opened, until the client shows it.
    pub(super) opened_book: Option<u8>,
    /// What menus read of the enchantments, from the data pack.
    pub(super) enchantment_traits:
        std::sync::Arc<Vec<photon_game::menus::container::EnchantmentTraits>>,
}

/// Chat, the HUD, the tab list, commands and what the server asked the player.
#[derive(Default)]
pub(super) struct Social {
    /// Chat the server sent that the client has not shown yet, and the
    /// `nextChatIndex` its player chat must come in.
    pub(super) chat_events: Vec<ChatEvent>,
    pub(super) signature_cache: SignatureCache,
    pub(super) next_chat_index: i32,
    /// Title, action bar and tab list packets not yet shown, and the players
    /// the tab list names.
    pub(super) hud_events: Vec<HudEvent>,
    pub(super) scoreboard: photon_game::scoreboard::Scoreboard,
    pub(super) waypoints: photon_game::waypoint::WaypointManager,
    pub(super) commands: Option<std::sync::Arc<photon_game::commands::CommandTree>>,
    pub(super) suggestions: Option<CommandSuggestions>,
    pub(super) player_infos: std::collections::BTreeMap<(u64, u64), PlayerInfo>,
    /// `ClientPacketListener.seenPlayers`: the players whose entity was added, by id and name,
    /// kept when they leave the tab list.
    pub(super) seen_players: std::collections::BTreeMap<(u64, u64), String>,
    /// Resource packs the server asked for that the player is yet to be asked about.
    pub(super) pack_requests: Vec<PackRequest>,
    /// Dialogs the server showed or cleared that the client is yet to act on.
    pub(super) dialog_events: Vec<DialogEvent>,
    /// The language item names are compared in.
    pub(super) lang: std::sync::Arc<photon_game::text::Lang>,
}

/// What the packets made happen that the client has yet to show or hear.
pub(super) struct Effects {
    /// The sound packets since the last [`Self::take_sounds`].
    pub(super) sounds: Vec<photon_game::sounds::SoundCommand>,
    /// The particle packets since the last [`Self::take_level_particles`].
    pub(super) level_particles: Vec<photon_world::particles::LevelParticles>,
    /// The emitters asked for since the last `take_tracking_emitters`.
    pub(super) tracking_emitters: Vec<photon_world::particles::TrackingRequest>,
    /// The stack a totem's animation was asked to show (the latest), until it is taken.
    pub(super) item_activation: Option<photon_world::ClientItemStackState>,
    /// The unsecure-server toast is due (`seenInsecureChatWarning` says whether it was shown).
    pub(super) unsecure_warning: bool,
    pub(super) seen_unsecure_warning: bool,
    /// The entities taken since the last [`Self::take_item_pickups`], flying to their takers.
    pub(super) item_pickups: Vec<photon_world::particles::ItemPickup>,
    pub(super) dirty_sections: std::collections::BTreeSet<(i32, i32, i32)>,
    pub(super) forgotten_chunks: Vec<(i32, i32)>,
    /// `ClientPacketListener.random`: the pitch of a pickup's sound.
    pub(super) listener_random: photon_core::random::RandomSource,
    /// `ClientLevel.random`, unseeded as the reference's is.
    pub(super) level_random: photon_core::random::RandomSource,
}

/// What the server said of the level and its clocks.
#[derive(Default)]
pub(super) struct LevelInfo {
    pub(super) view_distance: i32,
    pub(super) dimension_type_id: i32,
    pub(super) hashed_seed: i64,
    pub(super) game_time: i64,
    pub(super) rain_level: f32,
    pub(super) thunder_level: f32,
    /// `ClientLevel.tickRateManager`.
    pub(super) tick_rate: photon_world::tick_rate::TickRateManager,
    pub(super) clocks: std::collections::BTreeMap<i32, ClockState>,
    /// Set when the server moved the player to a new level.
    pub(super) level_reset: bool,
    /// Set when the client got a new level, not the same one again (`Minecraft.setLevel`).
    pub(super) level_swapped: bool,
    /// The server took the connection through the configuration phase again since the last
    /// [`Self::take_reconfigured`].
    pub(super) reconfigured: bool,
}

impl Link {
    /// A connection over `writer`, with nothing said yet.
    pub(super) fn new(
        writer: PacketWriter,
        compression_threshold: Option<usize>,
        incoming: Receiver<Result<Incoming, ConnectionError>>,
        (username, auto_jump): (&str, bool),
    ) -> Self {
        Self {
            writer,
            compression_threshold,
            incoming,
            username: username.to_owned(),
            auto_jump,
            client_information: photon_protocol::configuration::ClientInformation::default(),
            disconnect: None,
            warned: None,
            traffic: super::traffic::Traffic::default(),
        }
    }
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            player_id: None,
            game_mode: ClientPlayerGameMode::Survival,
            previous_game_mode: None,
            abilities: None,
            player: None,
            status: PlayerStatus::default(),
            permission_level: 0,
            hardcore: false,
            reduced_debug_info: false,
            advancements: photon_game::advancements::ClientAdvancements::default(),
            advancement_toasts: Vec::new(),
            stats: std::collections::HashMap::new(),
            stats_updated: false,
            show_death_screen: true,
            death: None,
            win_game: false,
            health_received: false,
            respawn_portal: None,
            aimed_entity: None,
            sent_slot: 0,
            game_mode_state: photon_game::game_mode::GameMode::default(),
            drop_spam: 0,
            removed_player_vehicle: None,
            teleports: 0,
            swings: 0,
            loaded: false,
            item_cooldowns: photon_game::item::cooldowns::ItemCooldowns::default(),
        }
    }
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            sounds: Vec::new(),
            level_particles: Vec::new(),
            tracking_emitters: Vec::new(),
            item_activation: None,
            unsecure_warning: false,
            seen_unsecure_warning: false,
            item_pickups: Vec::new(),
            dirty_sections: std::collections::BTreeSet::new(),
            forgotten_chunks: Vec::new(),
            listener_random: clock_seeded_random(0),
            level_random: clock_seeded_random(0x5DEE_CE66),
        }
    }
}

/// A random source seeded from the clock, mixed with `salt`.
fn clock_seeded_random(salt: i64) -> photon_core::random::RandomSource {
    photon_core::random::RandomSource::create(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64 ^ salt),
    )
}
