//! A live play session: the connection kept open, the world kept current,
//! and the local player ticked against it.
//!
//! The probes in this crate each drive one scripted scene and hang up. This
//! session stays: a reader thread takes packets off the socket as they come,
//! the caller drains them between frames, and each client tick runs the
//! local player and sends what the tick asked for, then the tick-end marker,
//! as `Minecraft.tick` does.

use std::io;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

use crate::protocol::ConnectionProtocol;
use voxel_core::math::Vec3;
use voxel_core::wire;
use voxel_world::input::Input;
use voxel_world::local_player::{LocalPlayer, ServerboundMovement};
use voxel_world::{
    ClientPlayerAbilities, ClientPlayerGameMode, ClientWorld, DimensionInput, LightLayerInput,
    LightUpdateInput, SectionInput,
};

use super::entities;
use super::probe::{
    dimension_bounds_for_spawn_info, dimension_environment_for_spawn_info, open_play_connection,
};
use super::protocol::*;
use super::session::{read_packet, send_packet};

const PLAY_CLIENTBOUND_DISCONNECT_PACKET_ID: i32 = 32;
const PLAY_CLIENTBOUND_PING_PACKET_ID: i32 = 61;
const PLAY_CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID: i32 = 84;
const PLAY_CLIENTBOUND_SET_TIME_PACKET_ID: i32 = 113;
const PLAY_CLIENTBOUND_UPDATE_TAGS_PACKET_ID: i32 = 134;
const PLAY_SERVERBOUND_PONG_PACKET_ID: i32 = 45;
const PLAY_SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID: i32 = 30;
const PLAY_SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID: i32 = 32;
const PLAY_SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID: i32 = 33;
const PLAY_SERVERBOUND_MOVE_VEHICLE_PACKET_ID: i32 = 34;
const PLAY_SERVERBOUND_PADDLE_BOAT_PACKET_ID: i32 = 35;
const PLAY_SERVERBOUND_PLAYER_ABILITIES_PACKET_ID: i32 = 40;
const PLAY_SERVERBOUND_PLAYER_COMMAND_PACKET_ID: i32 = 42;
const PLAY_SERVERBOUND_PLAYER_INPUT_PACKET_ID: i32 = 43;

/// A packet as the reader thread hands it over. Chunk and light packets,
/// which are the heavy ones to parse, arrive already parsed; the order the
/// packets came in is kept, so an error surfaces where it always did.
enum Incoming {
    Raw(Vec<u8>),
    Chunk(Box<ChunkPacketObservation>),
    Light(Box<LightUpdateObservation>),
}

/// Parse the packets worth parsing off the game's thread.
fn classify(body: Vec<u8>) -> Result<Incoming, ConnectionError> {
    let (packet_id, payload) = split_packet(&body)?;
    match packet_id {
        PLAY_CLIENTBOUND_CHUNK_PACKET_ID => {
            Ok(Incoming::Chunk(Box::new(parse_chunk_packet(payload)?)))
        }
        PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID => Ok(Incoming::Light(Box::new(
            parse_light_update_packet(payload)?,
        ))),
        _ => Ok(Incoming::Raw(body)),
    }
}

/// A connection in the play phase with its world and local player.
pub struct PlaySession {
    writer: TcpStream,
    compression_threshold: Option<usize>,
    incoming: Receiver<Result<Incoming, ConnectionError>>,
    trace: PlayConnectionTrace,
    player_id: Option<i32>,
    game_mode: ClientPlayerGameMode,
    abilities: Option<ClientPlayerAbilities>,
    player: Option<LocalPlayer>,
    loaded: bool,
    auto_jump: bool,
    warned: Option<String>,
    teleports: u32,
    view_distance: i32,
    dimension_type_id: i32,
    hashed_seed: i64,
    game_time: i64,
    rain_level: f32,
    thunder_level: f32,
    status: PlayerStatus,
    /// `LocalPlayer.shouldShowDeathScreen`: off when the level respawns at once.
    show_death_screen: bool,
    /// Why the connection is over, once it is: the server's word or what
    /// the connection said (`onDisconnect`).
    disconnect: Option<voxel_game::text::Component>,
    /// The message of a death the death screen is yet to show.
    death: Option<voxel_game::text::Component>,
    /// Swing packets sent, for scripted runs to report.
    swings: u32,
    /// The sound packets since the last [`Self::take_sounds`].
    sounds: Vec<voxel_game::sounds::SoundCommand>,
    /// The entity the crosshair last landed on (`Minecraft.hitResult`).
    aimed_entity: Option<i32>,
    /// Whether the level is hardcore, as the login said.
    hardcore: bool,
    /// Whether a health update has arrived since the player last respawned.
    health_received: bool,
    /// Set when the server moved the player to a new level.
    level_reset: bool,
    /// The hotbar slot the server was last told about.
    sent_slot: i32,
    /// Breaking state and the miss wait.
    game_mode_state: voxel_game::game_mode::GameMode,
    /// The player's inventory menu, whose drag state lasts between clicks.
    container_menu: voxel_game::container_menu::ContainerMenu,
    /// The menu of the container the server opened, likewise.
    open_menu: Option<voxel_game::container_menu::ContainerMenu>,
    /// Chat the server sent that the client has not shown yet, and the
    /// `nextChatIndex` its player chat must come in.
    chat_events: Vec<ChatEvent>,
    signature_cache: SignatureCache,
    /// Title, action bar and tab list packets not yet shown, and the players
    /// the tab list names.
    hud_events: Vec<HudEvent>,
    /// The vehicle the server removed with the local player aboard (`removedPlayerVehicleId`).
    removed_player_vehicle: Option<i32>,
    scoreboard: voxel_game::scoreboard::Scoreboard,
    waypoints: voxel_game::waypoint::WaypointManager,
    /// The player's operator level, 0 to 4 (`LocalPlayer.permissions`), which
    /// the server tells by entity events 24 to 28.
    permission_level: i32,
    commands: Option<std::sync::Arc<voxel_game::commands::CommandTree>>,
    username: String,
    suggestions: Option<CommandSuggestions>,
    player_infos: std::collections::BTreeMap<(u64, u64), PlayerInfo>,
    next_chat_index: i32,
    /// `LocalPlayer.dropSpamThrottler`: what creative drops have added up.
    drop_spam: i32,
    dirty_sections: std::collections::BTreeSet<(i32, i32, i32)>,
    forgotten_chunks: Vec<(i32, i32)>,
    clocks: std::collections::BTreeMap<i32, ClockState>,
}

/// `ClientClockManager`'s state for one world clock.
#[derive(Debug, Clone, Copy)]
struct ClockState {
    total_ticks: i64,
    partial_tick: f32,
    rate: f32,
}

impl PlaySession {
    /// Log in, configure, and start reading play packets in the background.
    pub fn connect(
        address: SocketAddr,
        username: &str,
        timeout: Duration,
        auto_jump: bool,
    ) -> Result<Self, ConnectionError> {
        let mut trace = PlayConnectionTrace::default();
        let (stream, compression_threshold) =
            open_play_connection(address, username, timeout, true, &mut trace)?;
        stream.set_read_timeout(None)?;
        let mut reader = stream.try_clone()?;
        let (sender, incoming) = mpsc::channel();
        std::thread::spawn(move || {
            loop {
                let packet = read_packet(&mut reader, compression_threshold).and_then(classify);
                let failed = packet.is_err();
                if sender.send(packet).is_err() || failed {
                    return;
                }
            }
        });
        Ok(Self::assemble(
            stream,
            compression_threshold,
            incoming,
            trace,
            (username, auto_jump),
        ))
    }

    /// A session over a connection already made, with nothing yet known.
    fn assemble(
        writer: TcpStream,
        compression_threshold: Option<usize>,
        incoming: Receiver<Result<Incoming, ConnectionError>>,
        trace: PlayConnectionTrace,
        (username, auto_jump): (&str, bool),
    ) -> Self {
        Self {
            writer,
            compression_threshold,
            incoming,
            trace,
            player_id: None,
            game_mode: ClientPlayerGameMode::Survival,
            abilities: None,
            player: None,
            loaded: false,
            auto_jump,
            warned: None,
            teleports: 0,
            view_distance: 0,
            dimension_type_id: 0,
            hashed_seed: 0,
            game_time: 0,
            rain_level: 0.0,
            thunder_level: 0.0,
            status: PlayerStatus::default(),
            show_death_screen: true,
            disconnect: None,
            death: None,
            swings: 0,
            sounds: Vec::new(),
            aimed_entity: None,
            hardcore: false,
            health_received: false,
            level_reset: false,
            sent_slot: 0,
            game_mode_state: voxel_game::game_mode::GameMode::default(),
            container_menu: voxel_game::container_menu::ContainerMenu::default(),
            open_menu: None,
            chat_events: Vec::new(),
            signature_cache: SignatureCache::default(),
            hud_events: Vec::new(),
            removed_player_vehicle: None,
            scoreboard: voxel_game::scoreboard::Scoreboard::default(),
            waypoints: voxel_game::waypoint::WaypointManager::default(),
            permission_level: 0,
            commands: None,
            username: username.to_owned(),
            suggestions: None,
            player_infos: std::collections::BTreeMap::new(),
            next_chat_index: 0,
            drop_spam: 0,
            dirty_sections: std::collections::BTreeSet::new(),
            forgotten_chunks: Vec::new(),
            clocks: std::collections::BTreeMap::new(),
        }
    }

    pub fn world(&self) -> &ClientWorld {
        &self.trace.world
    }

    /// The sounds the world's events and the player's own blocks made since the last call.
    pub fn take_world_sounds(&mut self) -> Vec<voxel_world::local_player::LocalSound> {
        self.trace.world.take_sounds()
    }

    /// Where an entity is drawn between two ticks.
    pub fn entity_snapshot(
        &self,
        entity_id: i32,
        partial_tick: f32,
    ) -> Option<voxel_world::EntitySnapshot> {
        self.trace.world.entity_snapshot(entity_id, partial_tick)
    }

    /// How many of the entities tick (those in a loaded chunk).
    pub fn ticking_entity_count(&self) -> usize {
        self.trace
            .world
            .entities()
            .filter(|entity| entity.is_ticking())
            .count()
    }

    /// How many entities the world holds besides the player.
    pub fn entity_count(&self) -> usize {
        self.trace.world.entities().count()
    }

    pub fn player(&self) -> Option<&LocalPlayer> {
        self.player.as_ref()
    }

    pub fn player_mut(&mut self) -> Option<&mut LocalPlayer> {
        self.player.as_mut()
    }

    /// The registry id of what the off hand holds (air when empty).
    fn off_hand_item(&self) -> i32 {
        self.trace
            .world
            .local_player()
            .and_then(|state| state.inventory_slot(45))
            .map_or(0, |stack| stack.item_registry_id())
    }

    /// The mouse turns the player (`Entity.turn`), who in a boat is then held within a quarter
    /// turn of where the boat faces (`onPassengerTurned`).
    pub fn turn_player(&mut self, y_rot_delta: f64, x_rot_delta: f64) {
        let boat_y_rot = self
            .trace
            .world
            .ridden_vehicle_of_local_player()
            .and_then(|id| self.trace.world.vehicle_view(id))
            .filter(|view| view.kind == voxel_world::VehicleKind::Boat)
            .map(|view| view.y_rot);
        if let Some(player) = self.player.as_mut() {
            player.turn(y_rot_delta, x_rot_delta);
            if let Some(y_rot) = boat_y_rot {
                player.clamp_in_boat(y_rot);
            }
        }
    }

    /// Whether the level is ready and the player is being ticked.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// How many times the server has placed the player: once on joining,
    /// and once more for every movement it refused.
    pub fn teleports(&self) -> u32 {
        self.teleports
    }

    /// The server's view distance in chunks, from the login packet.
    pub fn view_distance(&self) -> i32 {
        self.view_distance
    }

    /// Sections whose drawing may have changed since the last call: their
    /// blocks, their light, or a neighbour they cull or shade against.
    pub fn take_dirty_sections(&mut self) -> Vec<(i32, i32, i32)> {
        std::mem::take(&mut self.dirty_sections)
            .into_iter()
            .collect()
    }

    /// Chunks the server unloaded since the last call.
    pub fn take_forgotten_chunks(&mut self) -> Vec<(i32, i32)> {
        std::mem::take(&mut self.forgotten_chunks)
    }

    fn section_range(&self) -> std::ops::Range<i32> {
        let dimension = self.trace.world.dimension();
        let low = dimension.min_y.div_euclid(16);
        low..low + dimension.height / 16
    }

    /// Mark a chunk column and every column around it.
    fn mark_columns_around(&mut self, chunk_x: i32, chunk_z: i32) {
        for section_y in self.section_range() {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    self.dirty_sections
                        .insert((chunk_x + dx, section_y, chunk_z + dz));
                }
            }
        }
    }

    /// `ClientLevel.update`'s light part, once a frame: queued light tasks
    /// and the engine run, and the sections whose light changed redraw.
    pub fn update_light(&mut self) {
        for section in self.trace.world.update_light() {
            self.dirty_sections.insert(section);
        }
    }

    /// Mark the section holding a block and the sections touching it.
    fn mark_block(&mut self, x: i32, y: i32, z: i32) {
        for dx in [-1, 0, 1] {
            for dy in [-1, 0, 1] {
                for dz in [-1, 0, 1] {
                    self.dirty_sections.insert((
                        (x + dx).div_euclid(16),
                        (y + dy).div_euclid(16),
                        (z + dz).div_euclid(16),
                    ));
                }
            }
        }
    }

    /// The hashed world seed the biome zoom is jittered with.
    pub fn hashed_seed(&self) -> i64 {
        self.hashed_seed
    }

    /// The current dimension type's registry name.
    pub fn dimension_type(&self) -> Option<&str> {
        self.registry_names("minecraft:dimension_type")
            .get(usize::try_from(self.dimension_type_id).ok()?)
            .map(String::as_str)
    }

    /// The entry names of one synchronised registry, in network order.
    pub fn registry_names(&self, registry: &str) -> &[String] {
        self.trace
            .registry_entry_names
            .get(registry)
            .map_or(&[], Vec::as_slice)
    }

    /// The level's game time, as the server last set it and the ticks since.
    pub fn game_time(&self) -> i64 {
        self.game_time
    }

    /// `LocalPlayer.swing` with the main hand: the arm swings, and the
    /// server hears of it.
    pub fn swing(&mut self) -> Result<(), ConnectionError> {
        let Some(player) = self.player.as_mut() else {
            return Ok(());
        };
        player.swing();
        self.swings += 1;
        self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
            wire::write_varint(0, output);
            Ok(())
        })
    }

    /// Whether the level is hardcore.
    pub fn hardcore(&self) -> bool {
        self.hardcore
    }

    /// The player's game mode, as the server last set it.
    pub fn game_mode(&self) -> ClientPlayerGameMode {
        self.game_mode
    }

    /// The sound packets that arrived since the last call, in order (`handleSoundEvent`,
    /// `handleSoundEntityEvent`, `handleStopSoundEvent`): the sound engine's to carry out.
    pub fn take_sounds(&mut self) -> Vec<voxel_game::sounds::SoundCommand> {
        std::mem::take(&mut self.sounds)
    }

    /// How many swings the session has sent.
    pub fn swings(&self) -> u32 {
        self.swings
    }

    /// The player's profile id, as the server logged it in.
    pub fn profile_id(&self) -> Option<voxel_core::binary::Uuid> {
        self.trace.profile_id
    }

    /// Whether the server replaced the level since the last call, so
    /// everything drawn from the old one should go.
    pub fn take_level_reset(&mut self) -> bool {
        std::mem::take(&mut self.level_reset)
    }

    /// What the HUD shows of the player.
    pub fn status(&self) -> &PlayerStatus {
        &self.status
    }

    /// Select a hotbar slot, 0 to 8; the server hears of it next tick.
    pub fn select_slot(&mut self, slot: i32) {
        if (0..9).contains(&slot) {
            self.status.selected_slot = slot;
        }
    }

    /// `Level.getRainLevel`: how hard it rains, 0 to 1.
    pub fn rain_level(&self) -> f32 {
        self.rain_level
    }

    /// `Level.getThunderLevel`: the thunder level scaled by the rain.
    pub fn thunder_level(&self) -> f32 {
        self.thunder_level * self.rain_level
    }

    /// A world clock's total ticks, by its registry name.
    pub fn clock_ticks(&self, clock: &str) -> Option<i64> {
        let id = self
            .registry_names("minecraft:world_clock")
            .iter()
            .position(|name| name == clock)?;
        self.clocks.get(&(id as i32)).map(|state| state.total_ticks)
    }

    /// `ClientClockManager.tick`: every clock runs at its own rate.
    fn tick_clocks(&mut self, game_time: i64) {
        let delta = game_time - self.game_time;
        self.game_time = game_time;
        for state in self.clocks.values_mut() {
            let ticks = f64::from(state.partial_tick) + delta as f64 * f64::from(state.rate);
            let whole = ticks.floor();
            state.partial_tick = (ticks - whole) as f32;
            state.total_ticks += whole as i64;
        }
    }

    /// The first unported movement branch the player reached, if any.
    pub fn warning(&self) -> Option<&str> {
        self.warned.as_deref()
    }

    /// Handle every packet that has arrived, without waiting for more.
    pub fn drain(&mut self) -> Result<(), ConnectionError> {
        loop {
            if self.disconnect.is_some() {
                return Ok(());
            }
            match self.incoming.try_recv() {
                Ok(Ok(packet)) => self.handle_incoming(packet)?,
                // `ClientCommonPacketListenerImpl.onDisconnect`: a failure to
                // read ends the connection with what went wrong.
                Ok(Err(ConnectionError::Io(error))) => {
                    self.disconnect = Some(voxel_game::text::Component::translatable_with(
                        "disconnect.genericReason",
                        vec![voxel_game::text::Component::literal(format!(
                            "Internal Exception: {error}"
                        ))],
                    ));
                }
                Ok(Err(error)) => return Err(error),
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    self.disconnect = Some(voxel_game::text::Component::translatable(
                        "disconnect.endOfStream",
                    ));
                }
            }
        }
    }

    /// Why the connection is over, once it is.
    pub fn disconnected(&self) -> Option<&voxel_game::text::Component> {
        self.disconnect.as_ref()
    }

    /// Wait up to `limit` for the next packet and handle it.
    pub fn wait(&mut self, limit: Duration) -> Result<(), ConnectionError> {
        match self.incoming.recv_timeout(limit) {
            Ok(packet) => self.handle_incoming(packet?),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(()),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(ConnectionError::ServerDisconnected),
        }
    }

    fn warn(&mut self, message: String) {
        if self.warned.as_deref() != Some(message.as_str()) {
            eprintln!("{message}");
            self.warned = Some(message);
        }
    }

    /// `MultiPlayerGameMode.ensureHasSentCarriedItem`: the server hears of
    /// a new hotbar slot before anything the slot's item does.
    fn ensure_has_sent_carried_item(&mut self) -> Result<(), ConnectionError> {
        if self.status.selected_slot != self.sent_slot {
            self.sent_slot = self.status.selected_slot;
            let slot = self.sent_slot as i16;
            self.send(PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, |output| {
                output.extend_from_slice(&slot.to_be_bytes());
                Ok(())
            })?;
        }
        Ok(())
    }

    /// The breaking state, for drawing the crack and reporting.
    pub fn game_mode_state(&self) -> &voxel_game::game_mode::GameMode {
        &self.game_mode_state
    }

    /// The player's inventory menu as the client holds it now.
    pub fn container_menu(&mut self) -> &voxel_game::container_menu::ContainerMenu {
        if let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() {
            self.container_menu.load(slots, carried, state_id);
        }
        &self.container_menu
    }

    /// The container the server opened, as a screen needs to know it: its id,
    /// the menu it is and its title as NBT. `None` with none open, or one of
    /// a kind not modelled yet.
    pub fn open_container_info(
        &self,
    ) -> Option<(i32, voxel_game::container_menu::MenuKind, Vec<u8>)> {
        let container = self.trace.world.local_player()?.open_container()?;
        let kind = voxel_game::container_menu::MenuKind::from_menu_type(container.menu_type_id())?;
        // A screen opens once the server has sent what is in the container.
        self.trace.world.local_player_container_menu()?;
        Some((
            container.container_id(),
            kind,
            container.title_nbt().to_vec(),
        ))
    }

    /// Whether the server has opened a container this client has no menu for.
    pub fn open_container_unsupported(&self) -> bool {
        self.trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .is_some_and(|container| {
                voxel_game::container_menu::MenuKind::from_menu_type(container.menu_type_id())
                    .is_none()
            })
    }

    /// The menu an open screen is over: the container the server opened, or
    /// the player's own.
    pub fn active_menu(&mut self) -> &voxel_game::container_menu::ContainerMenu {
        if self.open_container_menu().is_some() {
            return self.open_menu.as_ref().expect("synced above");
        }
        self.container_menu()
    }

    /// The menu of the container the server opened, as the client holds it
    /// now; `None` until its contents have come.
    pub fn open_container_menu(&mut self) -> Option<&voxel_game::container_menu::ContainerMenu> {
        let (container_id, menu_type, slots, carried, state_id) =
            self.trace.world.local_player_container_menu()?;
        let kind = voxel_game::container_menu::MenuKind::from_menu_type(menu_type)?;
        match self.open_menu.as_mut() {
            Some(menu) if menu.container_id == container_id && menu.kind == kind => {
                menu.load(slots, carried, state_id);
            }
            _ => {
                self.open_menu = Some(voxel_game::container_menu::ContainerMenu::for_menu(
                    kind,
                    container_id,
                    slots,
                    carried,
                    state_id,
                ));
            }
        }
        // What a furnace menu reads besides its slots.
        if let Some(menu) = self.open_menu.as_mut() {
            menu.accepted = menu
                .kind
                .input_sets()
                .iter()
                .map(|set| self.trace.world.recipe_property_set(set))
                .collect();
            if menu.kind == voxel_game::container_menu::MenuKind::Stonecutter {
                menu.accepted = vec![std::sync::Arc::new(self.trace.world.stonecutter_inputs())];
                menu.options = menu
                    .slots
                    .first()
                    .and_then(Option::as_ref)
                    .map(|input| {
                        self.trace
                            .world
                            .stonecutter_options(input.item_registry_id())
                    })
                    .unwrap_or_default();
            }
            if menu.kind == voxel_game::container_menu::MenuKind::Loom {
                let (patterns, layers) = Self::loom_patterns(&self.trace.world, menu);
                menu.patterns = patterns;
                menu.banner_layers = layers;
            }
            menu.item_tags = self.trace.world.block_state_registry().item_tags().clone();
            menu.xp_level = self.status.experience_level;
            if let Some(container) = self
                .trace
                .world
                .local_player()
                .and_then(|player| player.open_container())
            {
                menu.data = (0..4).map(|index| container.data(index)).collect();
            }
        }
        self.open_menu.as_ref()
    }

    /// `MultiPlayerGameMode.handleInventoryButtonClick`: a button of the open
    /// menu (a stonecutter recipe, a loom pattern, an enchantment) pressed.
    pub fn container_button(&mut self, button: i32) -> Result<(), ConnectionError> {
        let Some(container_id) = self
            .trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id())
        else {
            return Ok(());
        };
        // `StonecutterMenu.clickMenuButton`: the entry is chosen, and the
        // result, which only the server can make, is emptied.
        let stonecutter = self
            .open_menu
            .as_ref()
            .is_some_and(|menu| menu.kind == voxel_game::container_menu::MenuKind::Stonecutter);
        let loom = self
            .open_menu
            .as_ref()
            .is_some_and(|menu| menu.kind == voxel_game::container_menu::MenuKind::Loom);
        if loom {
            // `LoomMenu.clickMenuButton`: the pattern is chosen.
            self.trace
                .world
                .apply_local_player_container_data(container_id, 0, button as i16);
        }
        if stonecutter {
            self.trace
                .world
                .apply_local_player_container_data(container_id, 0, button as i16);
            if let Some((_, _, mut slots, carried, _)) =
                self.trace.world.local_player_container_menu()
            {
                slots[1] = None;
                self.trace
                    .world
                    .set_local_player_container_menu(&slots, carried);
            }
        }
        self.send(
            PLAY_SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID,
            |output| {
                voxel_core::wire::write_varint(container_id, output);
                voxel_core::wire::write_varint(button, output);
                Ok(())
            },
        )
    }

    /// `ClientPacketListener.applyPlayerInfoUpdate`: an `Add` makes the
    /// player; the rest change one that is known and are dropped for one
    /// that is not.
    fn apply_player_info(&mut self, id: (u64, u64), actions: Vec<InfoAction>) {
        for action in actions {
            if let InfoAction::Add { name } = action {
                self.player_infos.entry(id).or_insert_with(|| PlayerInfo {
                    name,
                    ..PlayerInfo::default()
                });
                continue;
            }
            let Some(info) = self.player_infos.get_mut(&id) else {
                continue;
            };
            match action {
                InfoAction::GameMode(mode) => info.game_mode = mode,
                InfoAction::Listed(listed) => info.listed = listed,
                InfoAction::Latency(latency) => info.latency = latency,
                InfoAction::DisplayName(name) => info.display_name = name,
                InfoAction::ListOrder(order) => info.list_order = order,
                InfoAction::Add { .. } => {}
            }
        }
    }

    /// `ServerboundCommandSuggestionPacket`: ask for what could follow `text`.
    pub fn request_suggestions(&mut self, id: i32, text: &str) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID, |output| {
            voxel_core::wire::write_varint(id, output);
            let units: Vec<u16> = text.encode_utf16().collect();
            voxel_core::utf8::write_utf8(output, &units, 32500)
                .map_err(|_| ConnectionError::Codec("suggestion request"))
        })
    }

    /// The answer to the last request, if one came.
    pub fn take_suggestions(&mut self) -> Option<CommandSuggestions> {
        self.suggestions.take()
    }

    /// The title, action bar and tab list packets since the last call.
    pub fn drain_hud(&mut self) -> Vec<HudEvent> {
        std::mem::take(&mut self.hud_events)
    }

    /// A death the death screen is yet to show: the server's message.
    pub fn take_death(&mut self) -> Option<voxel_game::text::Component> {
        self.death.take()
    }

    /// `LocalPlayer.respawn`: ask the server to bring the player back.
    pub fn respawn(&mut self) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            encode_perform_respawn,
        )
    }

    /// Whether the player may use entity selectors
    /// (`Permissions.COMMANDS_ENTITY_SELECTORS`: game master and above).
    pub fn can_use_selectors(&self) -> bool {
        self.permission_level >= 2
    }

    /// The waypoints the server has the client track.
    pub fn waypoints(&self) -> &voxel_game::waypoint::WaypointManager {
        &self.waypoints
    }

    /// What the server's scoreboard packets built.
    pub fn scoreboard(&self) -> &voxel_game::scoreboard::Scoreboard {
        &self.scoreboard
    }

    /// The command tree the server sent, once it has.
    pub fn commands(&self) -> Option<&std::sync::Arc<voxel_game::commands::CommandTree>> {
        self.commands.as_ref()
    }

    /// The local player's name, which scoreboard teams and scores go by.
    pub fn local_name(&self) -> &str {
        &self.username
    }

    /// `ClientSuggestionProvider.getSelectedEntities`: the string UUID of the
    /// entity the crosshair is on.
    pub fn selected_entities(&self) -> Vec<String> {
        self.aimed_entity
            .and_then(|id| self.trace.world.entity_at(id))
            .map(|entity| entity.uuid.to_string_java())
            .into_iter()
            .collect()
    }

    /// `getOnlinePlayers`: the names of every player the server told of.
    pub fn online_player_names(&self) -> Vec<String> {
        self.player_infos
            .values()
            .map(|info| info.name.clone())
            .collect()
    }

    /// `ClientPacketListener.getPlayerInfo(uuid)`'s profile name: what a
    /// remote player is called, if the server told of them.
    pub fn player_name(&self, uuid: voxel_core::binary::Uuid) -> Option<&str> {
        self.player_infos
            .get(&(uuid.most, uuid.least))
            .map(|info| info.name.as_str())
    }

    /// The players the tab list names (`getOnlinePlayers`, listed ones).
    pub fn listed_players(&self) -> Vec<&PlayerInfo> {
        self.player_infos
            .values()
            .filter(|info| info.listed)
            .collect()
    }

    /// `ClientPacketListener.handlePlayerChat`: the count, the signatures
    /// looked up and pushed into the cache, and the sender looked up.
    fn receive_player_chat(&mut self, packet: PlayerChatPacket) {
        // The reference drops the connection when the count skips; this
        // client counts on from what it saw.
        if packet.global_index != self.next_chat_index {
            self.warn(format!(
                "chat index {}, expected {}",
                packet.global_index, self.next_chat_index
            ));
        }
        self.next_chat_index = packet.global_index + 1;
        let mut queue = std::collections::VecDeque::new();
        for seen in &packet.last_seen {
            match self.signature_cache.unpack(seen) {
                Some(signature) => queue.push_back(signature),
                None => {
                    self.warn("player chat: unknown signature".to_owned());
                    return;
                }
            }
        }
        if let Some(signature) = &packet.signature {
            queue.push_back(signature.clone());
        }
        self.signature_cache.push(queue);
        let sender_known = self.player_infos.contains_key(&packet.sender);
        let sender_is_local = self
            .trace
            .profile_id
            .is_some_and(|id| (id.most, id.least) == packet.sender);
        self.chat_events.push(ChatEvent::Player {
            packet,
            sender_known,
            sender_is_local,
        });
    }

    /// The chat the server sent since the last call.
    pub fn drain_chat(&mut self) -> Vec<ChatEvent> {
        std::mem::take(&mut self.chat_events)
    }

    /// `ClientPacketListener.sendChat` and `sendCommand`: a line typed into
    /// the chat, sent as a command when `command` (without its slash).
    /// Without a chat key the message goes unsigned.
    pub fn send_chat(&mut self, text: &str, command: bool) -> Result<(), ConnectionError> {
        if command {
            return self.send(PLAY_SERVERBOUND_CHAT_COMMAND_PACKET_ID, |output| {
                let units: Vec<u16> = text.encode_utf16().collect();
                voxel_core::utf8::write_utf8(output, &units, 32767)
                    .map_err(|_| ConnectionError::Codec("chat command"))
            });
        }
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |time| time.as_millis() as i64);
        let salt = i64::from(rand_salt());
        self.send(PLAY_SERVERBOUND_CHAT_PACKET_ID, |output| {
            encode_chat(text, millis, salt, output);
            Ok(())
        })
    }

    /// `ServerboundPlaceRecipePacket`: the recipe book asks the server to
    /// move a recipe's ingredients into the open menu's grid.
    pub fn place_recipe(&mut self, recipe: i32, shift: bool) -> Result<(), ConnectionError> {
        let container_id = self.open_container_id();
        self.send(PLAY_SERVERBOUND_PLACE_RECIPE_PACKET_ID, |output| {
            voxel_core::wire::write_varint(container_id, output);
            voxel_core::wire::write_varint(recipe, output);
            output.push(u8::from(shift));
            Ok(())
        })
    }

    /// `ServerboundRecipeBookChangeSettingsPacket`: `book_type` is the
    /// `RecipeBookType` ordinal.
    pub fn recipe_book_settings(
        &mut self,
        book_type: i32,
        open: bool,
        filtering: bool,
    ) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID,
            |output| {
                voxel_core::wire::write_varint(book_type, output);
                output.push(u8::from(open));
                output.push(u8::from(filtering));
                Ok(())
            },
        )
    }

    /// `RecipeBookComponent.setVisible` and the filter button: the book's
    /// settings change here at once and the server is told.
    pub fn set_recipe_book_settings(
        &mut self,
        book_type: i32,
        open: bool,
        filtering: bool,
    ) -> Result<(), ConnectionError> {
        let book = self.trace.world.recipe_book_mut();
        if let Some(settings) = book.settings.get_mut(book_type as usize) {
            settings.open = open;
            settings.filtering = filtering;
            book.revision += 1;
        }
        self.recipe_book_settings(book_type, open, filtering)
    }

    /// `GhostSlots.clear`.
    pub fn clear_recipe_ghost(&mut self) {
        self.trace.world.recipe_book_mut().ghost = None;
    }

    /// `LocalPlayer.removeRecipeHighlight`: a new recipe that showed on a page
    /// stops being new, and the server is told.
    pub fn mark_recipe_seen(&mut self, recipe: i32) -> Result<(), ConnectionError> {
        if self.trace.world.recipe_book_mut().highlight.remove(&recipe) {
            self.recipe_seen(recipe)?;
        }
        Ok(())
    }

    /// `ServerboundRecipeBookSeenRecipePacket`: a highlighted recipe was seen.
    pub fn recipe_seen(&mut self, recipe: i32) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID,
            |output| {
                voxel_core::wire::write_varint(recipe, output);
                Ok(())
            },
        )
    }

    /// The open menu's container id; 0 is the player's own.
    fn open_container_id(&self) -> i32 {
        self.trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map_or(0, |container| container.container_id())
    }

    /// `ServerboundRenameItemPacket`: the anvil's name box changed.
    pub fn rename_item(&mut self, name: &str) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_RENAME_ITEM_PACKET_ID, |output| {
            voxel_core::wire::write_varint(name.len() as i32, output);
            output.extend_from_slice(name.as_bytes());
            Ok(())
        })
    }

    /// `LoomMenu.slotsChanged`'s list of patterns: with a banner and a dye in,
    /// those that need no item, or those the pattern item in the loom gives.
    /// Nothing once the banner has six layers.
    fn loom_patterns(
        world: &voxel_world::ClientWorld,
        menu: &voxel_game::container_menu::ContainerMenu,
    ) -> (Vec<i32>, usize) {
        let stack = |slot: usize| menu.slots.get(slot).and_then(Option::as_ref);
        let (Some(banner), Some(_dye)) = (stack(0), stack(1)) else {
            return (Vec::new(), 0);
        };
        let layers =
            voxel_game::item_values::Values::new(banner.item_registry_id(), banner.patch_bytes())
                .banner_layers()
                .len();
        let tags = world.block_state_registry().banner_pattern_tags();
        let tag = match stack(2) {
            None => "minecraft:no_item_required".to_owned(),
            Some(pattern) => {
                // `provides_banner_patterns` names `pattern_item/<name>`.
                let name = usize::try_from(pattern.item_registry_id())
                    .ok()
                    .and_then(|id| voxel_data::item::registry::ITEMS.get(id))
                    .map_or("", |item| item.name);
                let short = name
                    .trim_start_matches("minecraft:")
                    .trim_end_matches("_banner_pattern");
                format!("minecraft:pattern_item/{short}")
            }
        };
        (tags.members(&tag).collect(), layers)
    }

    /// `AbstractContainerScreen.slotClicked` into the open menu: the player's
    /// own, or the container the server opened. The click as the client
    /// predicts it, the arm swings its drops cause, then the click packet.
    pub fn container_click(
        &mut self,
        slot: i32,
        button: i32,
        kind: voxel_game::container_menu::ClickType,
    ) -> Result<(), ConnectionError> {
        let player = voxel_game::container_menu::Player {
            selected: self.status.selected_slot.clamp(0, 8) as usize,
            infinite_materials: self.game_mode == voxel_world::ClientPlayerGameMode::Creative,
        };
        let opened = self
            .trace
            .world
            .local_player()
            .is_some_and(|player| player.open_container().is_some());
        let (swings, body) = if opened {
            // Synced before the click; `None` when the container is of a kind
            // with no menu here, which no screen sends clicks for.
            if self.open_container_menu().is_none() {
                return Ok(());
            }
            let menu = self.open_menu.as_mut().expect("synced above");
            let clicked = menu.handle_click(slot, button, kind, player);
            let (slots, carried) = (menu.slots.clone(), menu.carried.clone());
            self.trace
                .world
                .set_local_player_container_menu(&slots, carried);
            clicked
        } else {
            let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() else {
                return Ok(());
            };
            self.container_menu.load(slots, carried, state_id);
            let clicked = self.container_menu.handle_click(slot, button, kind, player);
            let menu = &self.container_menu;
            self.trace
                .world
                .set_local_player_menu(&menu.slots, menu.carried.clone());
            clicked
        };
        for _ in 0..swings {
            // `LocalPlayer.swing`: the arm, then the packet.
            if let Some(player) = self.player.as_mut() {
                player.swing();
            }
            self.swings += 1;
            self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
                output.push(0);
                Ok(())
            })?;
        }
        self.send(PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID, |output| {
            output.extend_from_slice(&body);
            Ok(())
        })
    }

    /// `CreativeModeInventoryScreen.slotClicked` into the player's own menu:
    /// the click as the client predicts it, then what it sends for it.
    pub fn creative_click(
        &mut self,
        creative: &mut voxel_game::creative_menu::CreativeMenu,
        click: voxel_game::creative_menu::Click,
    ) -> Result<(), ConnectionError> {
        use voxel_game::creative_menu::Effect;
        let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() else {
            return Ok(());
        };
        self.container_menu.load(slots, carried, state_id);
        let player = voxel_game::container_menu::Player {
            selected: self.status.selected_slot.clamp(0, 8) as usize,
            infinite_materials: self.game_mode == voxel_world::ClientPlayerGameMode::Creative,
        };
        // `Player.canDropItems`: under the throttler's threshold.
        let can_drop = self.drop_spam < 1280;
        let effects = creative.slot_clicked(click, &mut self.container_menu, player, can_drop);
        let menu = &self.container_menu;
        self.trace
            .world
            .set_local_player_menu(&menu.slots, menu.carried.clone());
        if !player.infinite_materials {
            return Ok(());
        }
        for effect in effects {
            match effect {
                Effect::Swing => {
                    if let Some(player) = self.player.as_mut() {
                        player.swing();
                    }
                    self.swings += 1;
                    self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
                        output.push(0);
                        Ok(())
                    })?;
                }
                Effect::SetSlot(slot, stack) => {
                    self.send(
                        PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                        |output| {
                            encode_creative_slot(output, slot, stack.as_ref());
                            Ok(())
                        },
                    )?;
                }
                Effect::Drop(stack) => {
                    self.send(
                        PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                        |output| {
                            encode_creative_slot(output, -1, Some(&stack));
                            Ok(())
                        },
                    )?;
                    self.drop_spam += 20;
                }
            }
        }
        Ok(())
    }

    /// `LocalPlayer.closeContainer` for the inventory: the close packet,
    /// then `ContainerMenu.removed`, which empties the result slot.
    pub fn close_inventory(&mut self) -> Result<(), ConnectionError> {
        let opened = self
            .trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id());
        let id = opened.unwrap_or(0);
        self.send(PLAY_SERVERBOUND_CONTAINER_CLOSE_PACKET_ID, |output| {
            voxel_core::wire::write_varint(id, output);
            Ok(())
        })?;
        if opened.is_some() {
            // `LocalPlayer.clientSideCloseContainer`: back to the player's menu.
            self.trace.world.apply_local_player_container_close(id);
            self.open_menu = None;
            return Ok(());
        }
        if let Some((mut slots, carried, _)) = self.trace.world.local_player_menu() {
            slots[0] = None;
            self.trace.world.set_local_player_menu(&slots, carried);
        }
        Ok(())
    }

    /// The hands whose stack a use spent since the last call.
    pub fn take_items_used(&mut self) -> Vec<u8> {
        self.game_mode_state.take_items_used()
    }

    /// The attack and use keys' part of `handleKeybinds`, with what the
    /// player aims at. Runs before the tick, as the reference handles keys
    /// before the level ticks the player.
    pub fn keys(
        &mut self,
        keys: voxel_game::game_mode::Keys,
        aim: voxel_game::game_mode::Aim,
    ) -> Result<(), ConnectionError> {
        use voxel_game::game_mode::{Context, Miner, Outgoing};
        self.aimed_entity = match aim {
            voxel_game::game_mode::Aim::Entity { id, .. } => Some(id),
            _ => None,
        };
        self.ensure_has_sent_carried_item()?;
        let Some(player) = self.player.as_mut() else {
            return Ok(());
        };
        let miner = Miner::of(&self.trace.world, self.status.selected_slot);
        let mut ctx = Context {
            world: &mut self.trace.world,
            player,
            miner: &miner,
            mode: self.game_mode,
            selected_slot: self.status.selected_slot,
        };
        let sent = self.game_mode_state.keys_tick(&mut ctx, keys, aim);
        for packet in sent {
            let id = match packet {
                Outgoing::Swing { .. } => {
                    self.swings += 1;
                    PLAY_SERVERBOUND_SWING_PACKET_ID
                }
                Outgoing::PlayerAction { x, y, z, .. } => {
                    // A predicted break changes the block it names.
                    self.mark_block(x, y, z);
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID
                }
                Outgoing::UseItemOn { x, y, z, face, .. } => {
                    // A predicted placement changes the block or the one
                    // in front of the face, and its neighbours.
                    let (dx, dy, dz) =
                        voxel_core::direction::Direction::from_3d_data_value(i32::from(face))
                            .step();
                    self.mark_block(x, y, z);
                    self.mark_block(x + dx, y + dy, z + dz);
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID
                }
                Outgoing::UseItem { .. } => PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                Outgoing::Attack { .. } => PLAY_SERVERBOUND_ATTACK_PACKET_ID,
                Outgoing::PickItemFromBlock { .. } => {
                    PLAY_SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID
                }
                Outgoing::PickItemFromEntity { .. } => {
                    PLAY_SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID
                }
                Outgoing::SpectatorAction { .. } => PLAY_SERVERBOUND_SPECTATOR_ACTION_PACKET_ID,
                Outgoing::Interact { .. } => PLAY_SERVERBOUND_INTERACT_PACKET_ID,
            };
            let body = packet.encode_body();
            self.send(id, |output| {
                output.extend_from_slice(&body);
                Ok(())
            })?;
        }
        Ok(())
    }

    /// One client tick: the local player, its packets, then the tick end.
    pub fn tick(&mut self, keys: Input) -> Result<(), ConnectionError> {
        // The game mode's tick runs before the player moves.
        self.ensure_has_sent_carried_item()?;
        if self.loaded {
            self.drop_spam = (self.drop_spam - 1).max(0);
        }
        let main_hand = self
            .trace
            .world
            .local_player()
            .and_then(|state| state.inventory_slot((36 + self.status.selected_slot) as i16))
            .map_or(0, |stack| stack.item_registry_id());
        // The level ticks a vehicle before its passengers: a boat the player rides moves first
        // (and tells the server of its paddles), then the player sits in it.
        // What the rider steers with: its input, its look, and the stick in its hands.
        if let Some(player) = self.player.as_ref() {
            let mut controls = player.controller();
            let holds = |name: &str| {
                [main_hand, self.off_hand_item()].into_iter().any(|item| {
                    voxel_world::block_rules::item(item).is_some_and(|item| item.name == name)
                })
            };
            controls.holds_carrot_on_a_stick = holds("minecraft:carrot_on_a_stick");
            controls.holds_warped_fungus_on_a_stick = holds("minecraft:warped_fungus_on_a_stick");
            self.trace.world.set_rider_controls(controls);
        }
        let ridden = self.trace.world.ridden_vehicle_of_local_player();
        if ridden.is_some() {
            if let Some(player) = self.player.as_ref() {
                let position = player.position();
                self.trace
                    .world
                    .set_local_player_chunk(position.x, position.z);
            }
            self.trace.world.tick_entities();
            for (left, right) in self.trace.world.take_paddle_packets() {
                self.send_movement(ServerboundMovement::PaddleBoat { left, right })?;
            }
        }
        if self.loaded
            && let Some(player) = self.player.as_mut()
        {
            player.set_main_hand_item(main_hand);
            let ticked = match ridden.and_then(|id| self.trace.world.vehicle_view(id)) {
                Some(view) => player
                    .tick_riding(&self.trace.world, keys, &view)
                    .map(|rode| {
                        if view.kind == voxel_world::VehicleKind::Boat && view.controlled {
                            self.trace
                                .world
                                .set_boat_input(view.vehicle_id, rode.boat_input);
                        }
                        if let Some(amount) = rode.jump {
                            self.trace.world.on_player_jump(view.vehicle_id, amount);
                        }
                        rode.sent
                    }),
                None => player.tick(&self.trace.world, keys),
            };
            match ticked {
                Ok(sent) => {
                    for packet in sent {
                        self.send_movement(packet)?;
                    }
                }
                Err(unsupported) => {
                    let message = unsupported.to_string();
                    if self.warned.as_deref() != Some(message.as_str()) {
                        eprintln!("movement: {message}");
                        self.warned = Some(message);
                    }
                }
            }
        }
        // `ClientLevel.tickEntities`: the entities the server added move on.
        if let Some(player) = self.player.as_ref() {
            let position = player.position();
            self.trace
                .world
                .set_local_player_chunk(position.x, position.z);
        }
        if ridden.is_none() {
            self.trace.world.tick_entities();
        }
        self.trace.world.tick_sky_flash();
        // ClientLevel.tickTime: the game time and the clocks move on.
        self.tick_clocks(self.game_time + 1);
        self.send(PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID, |_| Ok(()))
    }

    fn send<F>(&mut self, id: i32, body: F) -> Result<(), ConnectionError>
    where
        F: FnOnce(&mut Vec<u8>) -> Result<(), ConnectionError>,
    {
        // Nothing is sent over a connection that is over.
        if self.disconnect.is_some() {
            return Ok(());
        }
        send_packet(
            &mut self.writer,
            ConnectionProtocol::Play,
            id,
            body,
            self.compression_threshold,
        )
    }

    fn send_movement(&mut self, packet: ServerboundMovement) -> Result<(), ConnectionError> {
        let id = match packet {
            ServerboundMovement::PlayerInput(_) => PLAY_SERVERBOUND_PLAYER_INPUT_PACKET_ID,
            ServerboundMovement::PlayerCommand { .. } => PLAY_SERVERBOUND_PLAYER_COMMAND_PACKET_ID,
            ServerboundMovement::PlayerAbilities { .. } => {
                PLAY_SERVERBOUND_PLAYER_ABILITIES_PACKET_ID
            }
            ServerboundMovement::MovePlayerPos { .. } => PLAY_SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
            ServerboundMovement::MovePlayerPosRot { .. } => {
                PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID
            }
            ServerboundMovement::MovePlayerRot { .. } => PLAY_SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID,
            ServerboundMovement::MovePlayerStatusOnly { .. } => {
                PLAY_SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID
            }
            ServerboundMovement::MoveVehicle { .. } => PLAY_SERVERBOUND_MOVE_VEHICLE_PACKET_ID,
            ServerboundMovement::PaddleBoat { .. } => PLAY_SERVERBOUND_PADDLE_BOAT_PACKET_ID,
        };
        let body = packet.encode_body();
        self.send(id, |output| {
            output.extend_from_slice(&body);
            Ok(())
        })
    }

    /// A packet from the reader thread: the heavy ones arrive parsed.
    fn handle_incoming(&mut self, packet: Incoming) -> Result<(), ConnectionError> {
        match packet {
            Incoming::Raw(body) => self.handle(&body),
            Incoming::Chunk(chunk) => {
                self.apply_chunk(&chunk);
                Ok(())
            }
            Incoming::Light(update) => {
                self.apply_light_update(&update);
                Ok(())
            }
        }
    }

    /// `handleLevelChunkWithLight`: the chunk, then its light.
    fn apply_chunk(&mut self, chunk: &ChunkPacketObservation) {
        self.mark_columns_around(chunk.x, chunk.z);
        self.trace.world.replace_chunk(
            chunk.x,
            chunk.z,
            chunk
                .section_observations
                .iter()
                .map(|section| SectionInput {
                    non_empty_block_count: section.non_empty_block_count,
                    fluid_count: section.fluid_count,
                    block_states: &section.block_state_values,
                    biomes: &section.biome_values,
                }),
        );
        self.trace.world.apply_chunk_light(
            chunk.x,
            chunk.z,
            LightUpdateInput {
                sky: LightLayerInput {
                    mask: &chunk.light_data.sky_mask,
                    empty_mask: &chunk.light_data.empty_sky_mask,
                    updates: &chunk.light_data.sky_updates,
                },
                block: LightLayerInput {
                    mask: &chunk.light_data.block_mask,
                    empty_mask: &chunk.light_data.empty_block_mask,
                    updates: &chunk.light_data.block_updates,
                },
            },
        );
    }

    /// `handleLightUpdatePacket`: `readSectionList` redraws the sections
    /// whose light arrived when the queued task runs.
    fn apply_light_update(&mut self, update: &LightUpdateObservation) {
        self.trace.world.apply_light_updates(
            update.x,
            update.z,
            LightUpdateInput {
                sky: LightLayerInput {
                    mask: &update.light_data.sky_mask,
                    empty_mask: &update.light_data.empty_sky_mask,
                    updates: &update.light_data.sky_updates,
                },
                block: LightLayerInput {
                    mask: &update.light_data.block_mask,
                    empty_mask: &update.light_data.empty_block_mask,
                    updates: &update.light_data.block_updates,
                },
            },
        );
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one arm per packet the play state reads"
    )]
    #[expect(
        clippy::cognitive_complexity,
        reason = "one arm per packet id, read like the reference's handler list"
    )]
    fn handle(&mut self, body: &[u8]) -> Result<(), ConnectionError> {
        let (packet_id, payload) = split_packet(body)?;
        match packet_id {
            PLAY_CLIENTBOUND_LOGIN_PACKET_ID => {
                let login = parse_login_packet(payload)?;
                self.next_chat_index = 0;
                self.trace
                    .world
                    .set_server_simulation_distance(login.simulation_distance);
                let bounds = dimension_bounds_for_spawn_info(&self.trace, &login.spawn_info);
                let environment =
                    dimension_environment_for_spawn_info(&self.trace, &login.spawn_info);
                self.trace.world.replace_dimension_with_environment(
                    DimensionInput {
                        key: &login.spawn_info.dimension,
                        dimension_type_id: login.spawn_info.dimension_type_id,
                        seed: login.spawn_info.seed,
                        is_debug: login.spawn_info.is_debug,
                        is_flat: login.spawn_info.is_flat,
                        sea_level: login.spawn_info.sea_level,
                    },
                    bounds,
                    environment,
                );
                self.trace.world.configure_chunk_cache(login.chunk_radius);
                self.view_distance = login.chunk_radius;
                self.dimension_type_id = login.spawn_info.dimension_type_id;
                self.hashed_seed = login.spawn_info.seed;
                self.trace.world.set_local_player_id(login.player_id);
                self.hardcore = login.hardcore;
                self.show_death_screen = login.show_death_screen;
                self.player_id = Some(login.player_id);
                self.permission_level = 0;
                self.game_mode =
                    ClientPlayerGameMode::from_id(i32::from(login.spawn_info.game_type));
            }
            // The entity packets change the world's entities; one that does
            // not parse is left alone, as the reference ignores what it
            // cannot place.
            PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID => {
                if let Ok(packet) = parse_add_entity_packet(payload) {
                    if self.removed_player_vehicle == Some(packet.entity_id) {
                        self.removed_player_vehicle = None;
                    }
                    entities::add(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID => {
                if let Ok(packet) = parse_set_entity_motion_packet(payload) {
                    entities::motion(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID => {
                if let Ok((source, holder)) = parse_set_entity_link_packet(payload) {
                    self.trace.world.set_leash_holder(source, holder);
                }
            }
            PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID => {
                if let Ok(packet) = parse_rotate_head_packet(payload) {
                    entities::rotate_head(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID => {
                if let Ok(packet) = parse_remove_entities_packet(payload) {
                    // `handleRemoveEntities`: a vehicle that goes with its rider aboard is
                    // remembered, for a teleport the server may still address to it.
                    for &id in &packet.entity_ids {
                        if self.trace.world.has_entity(id)
                            && self.trace.world.entity_carries_local_player(id)
                        {
                            self.removed_player_vehicle = Some(id);
                        }
                    }
                    entities::remove(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID => {
                if let Ok(packet) = parse_set_passengers_packet(payload) {
                    let was_aboard = self.trace.world.ridden_vehicle_of_local_player();
                    entities::passengers(&mut self.trace.world, &packet);
                    if self
                        .player_id
                        .is_some_and(|id| packet.passenger_ids.contains(&id))
                    {
                        self.removed_player_vehicle = None;
                    }
                    // `handleSetEntityPassengersPacket`: boarding tells the rider how to get off.
                    if was_aboard != Some(packet.vehicle_id)
                        && self.trace.world.ridden_vehicle_of_local_player()
                            == Some(packet.vehicle_id)
                    {
                        self.hud_events.push(HudEvent::ActionBar(
                            voxel_game::text::Component::translatable_with(
                                "mount.onboard",
                                vec![voxel_game::text::Component::translatable(
                                    "key.keyboard.left.shift",
                                )],
                            ),
                        ));
                    }
                    // A player who has just boarded looks the way the boat or the horse faces.
                    if was_aboard.is_none()
                        && let Some(vehicle) = self.trace.world.ridden_vehicle_of_local_player()
                        && let Some(view) = self.trace.world.vehicle_view(vehicle)
                        && let Some(player) = self.player.as_mut()
                    {
                        match view.kind {
                            voxel_world::VehicleKind::Boat => player.face_boat(view.y_rot),
                            voxel_world::VehicleKind::Horse => {
                                player.set_rotation(view.y_rot, view.x_rot);
                            }
                            // Any other vehicle leaves its rider looking where it did.
                            voxel_world::VehicleKind::Pig | voxel_world::VehicleKind::Other => {}
                        }
                    }
                }
            }
            PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID => {
                if let Ok(packet) = parse_entity_position_sync_packet(payload) {
                    entities::position_sync(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID => {
                if let Ok(packet) = parse_entity_teleport_packet(payload) {
                    if !self.trace.world.has_entity(packet.entity_id)
                        && self.removed_player_vehicle == Some(packet.entity_id)
                    {
                        // The vehicle is gone: the teleport meant for it moves the rider.
                        self.move_former_rider(&packet)?;
                    }
                    entities::teleport(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID => {
                if let Ok(packet) = parse_move_entity_pos_packet(payload) {
                    entities::step(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID => {
                if let Ok(packet) = parse_move_entity_pos_rot_packet(payload) {
                    entities::step(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID => {
                if let Ok(packet) = parse_move_entity_rot_packet(payload) {
                    entities::step(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_ANIMATE_PACKET_ID => {
                if let Ok((id, action)) = parse_animate_packet(payload) {
                    self.trace.world.apply_entity_animate(id, action);
                }
            }
            PLAY_CLIENTBOUND_ENTITY_EVENT_PACKET_ID => {
                let (id, event) = parse_entity_event_packet(payload)?;
                // `LocalPlayer.handleEntityEvent`: 24 to 28 set the level.
                if self.player_id == Some(id) && (24..=28).contains(&event) {
                    self.permission_level = i32::from(event) - 24;
                }
                // `LivingEntity.handleEntityEvent`: 3 is a death.
                if event == 3 {
                    self.trace.world.apply_entity_death_event(id);
                } else {
                    self.trace.world.apply_entity_event(id, event);
                }
            }
            PLAY_CLIENTBOUND_SET_SIMULATION_DISTANCE_PACKET_ID => {
                if let Ok(chunks) = parse_entity_id_prefix(payload) {
                    self.trace.world.set_server_simulation_distance(chunks);
                }
            }
            PLAY_CLIENTBOUND_DAMAGE_EVENT_PACKET_ID => {
                if let Ok(id) = parse_entity_id_prefix(payload) {
                    self.trace.world.apply_entity_damage_event(id);
                }
            }
            PLAY_CLIENTBOUND_HURT_ANIMATION_PACKET_ID => {
                if let Ok(id) = parse_entity_id_prefix(payload) {
                    self.trace.world.apply_entity_hurt_animation(id);
                }
            }
            PLAY_CLIENTBOUND_RESPAWN_PACKET_ID => {
                let (spawn_info, data_to_keep) = parse_respawn_packet(payload)?;
                // A new level starts dry; the server repeats its weather.
                self.rain_level = 0.0;
                self.thunder_level = 0.0;
                self.trace.world.set_weather(0.0, 0.0);
                let bounds = dimension_bounds_for_spawn_info(&self.trace, &spawn_info);
                let environment = dimension_environment_for_spawn_info(&self.trace, &spawn_info);
                self.trace.world.replace_dimension_with_environment(
                    DimensionInput {
                        key: &spawn_info.dimension,
                        dimension_type_id: spawn_info.dimension_type_id,
                        seed: spawn_info.seed,
                        is_debug: spawn_info.is_debug,
                        is_flat: spawn_info.is_flat,
                        sea_level: spawn_info.sea_level,
                    },
                    bounds,
                    environment,
                );
                self.game_mode = ClientPlayerGameMode::from_id(i32::from(spawn_info.game_type));
                // The new player starts with an empty inventory; the server
                // sends it again.
                self.trace.world.recreate_local_player(data_to_keep);
                self.dimension_type_id = spawn_info.dimension_type_id;
                self.hashed_seed = spawn_info.seed;
                // A respawn makes a new player that waits for its position.
                self.player = None;
                self.permission_level = 0;
                self.loaded = false;
                self.level_reset = true;
                self.health_received = false;
            }
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => {
                self.apply_chunk(&parse_chunk_packet(payload)?);
            }
            PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID => {
                self.apply_light_update(&parse_light_update_packet(payload)?);
            }
            PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID => {
                let (x, z) = parse_forget_level_chunk_packet(payload)?;
                self.trace.world.unload_chunk(x, z);
                self.forgotten_chunks.push((x, z));
                self.mark_columns_around(x, z);
            }
            PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID => {
                let update = parse_block_update_packet(payload)?;
                self.mark_block(update.x, update.y, update.z);
                self.trace.world.apply_server_block_state(
                    update.x,
                    update.y,
                    update.z,
                    update.state_id,
                );
            }
            PLAY_CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID => {
                let mut input = payload;
                let section = read_i64(&mut input)?;
                let section_x = (section >> 42) as i32;
                let section_y = ((section << 44) >> 44) as i32;
                let section_z = ((section << 22) >> 42) as i32;
                let count = wire::read_varint(&mut input)
                    .map_err(|_| ConnectionError::Codec("section blocks update"))?;
                for _ in 0..count {
                    let packed = wire::read_varlong(&mut input)
                        .map_err(|_| ConnectionError::Codec("section blocks update"))?;
                    let state = (packed as u64 >> 12) as i32;
                    let local = (packed & 0xfff) as i32;
                    let (bx, by, bz) = (
                        section_x * 16 + ((local >> 8) & 15),
                        section_y * 16 + (local & 15),
                        section_z * 16 + ((local >> 4) & 15),
                    );
                    self.mark_block(bx, by, bz);
                    self.trace.world.apply_server_block_state(bx, by, bz, state);
                }
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID => {
                let radius = parse_set_chunk_cache_radius_packet(payload)?;
                self.trace.world.update_chunk_cache_radius(radius);
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID => {
                let (x, z) = parse_set_chunk_cache_center_packet(payload)?;
                self.trace.world.update_chunk_cache_center(x, z);
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID => {
                self.send(PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, |output| {
                    // The chunk batch calculator's starting estimate.
                    output.extend_from_slice(&3.5_f32.to_be_bytes());
                    Ok(())
                })?;
            }
            PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("play keep alive"))?;
                self.send(PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID, |output| {
                    crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                    Ok(())
                })?;
            }
            PLAY_CLIENTBOUND_PING_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundPingPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("play ping"))?;
                self.send(PLAY_SERVERBOUND_PONG_PACKET_ID, |output| {
                    crate::common::ServerboundPongPacket::new(packet.id).encode(output);
                    Ok(())
                })?;
            }
            PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID => {
                // Data of a kind this client cannot read leaves it as it was.
                if let Ok(packet) = parse_entity_metadata_packet(payload) {
                    entities::metadata(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_SET_EQUIPMENT_PACKET_ID => {
                if let Ok(packet) = parse_entity_equipment_packet(payload) {
                    entities::equipment(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID => {
                let packet = parse_update_attributes_packet(payload)?;
                let world = &mut self.trace.world;
                if !world.apply_local_player_attributes(packet.entity_id, &packet.attributes) {
                    world.apply_entity_attributes(packet.entity_id, &packet.attributes);
                }
            }
            PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID => {
                let packet = parse_update_mob_effect_packet(payload)?;
                self.trace.world.apply_local_player_mob_effect(
                    packet.entity_id,
                    packet.effect_registry_id,
                    packet.amplifier,
                    packet.duration_ticks,
                    packet.flags,
                );
            }
            PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID => {
                let packet = parse_remove_mob_effect_packet(payload)?;
                self.trace
                    .world
                    .remove_local_player_mob_effect(packet.entity_id, packet.effect_registry_id);
            }
            PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID => {
                let packet = parse_set_health_packet(payload)?;
                self.health_received = true;
                if let Some(player) = self.player.as_mut() {
                    player.hurt_to(packet.health);
                }
                self.status.health = packet.health;
                self.status.food = packet.food;
                self.status.saturation = packet.saturation;
            }
            PLAY_CLIENTBOUND_LEVEL_EVENT_PACKET_ID => {
                let mut input = payload;
                let event = read_level_event_packet(&mut input)?;
                if !input.is_empty() {
                    return Err(ConnectionError::Codec("level event trailing payload"));
                }
                self.trace
                    .world
                    .apply_level_event(event.event, event.pos, event.data);
            }
            PLAY_CLIENTBOUND_SOUND_PACKET_ID
            | PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID
            | PLAY_CLIENTBOUND_STOP_SOUND_PACKET_ID => {
                let mut input = payload;
                let command = match packet_id {
                    PLAY_CLIENTBOUND_SOUND_PACKET_ID => {
                        voxel_game::sounds::SoundCommand::At(read_sound_packet(&mut input)?)
                    }
                    PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID => {
                        voxel_game::sounds::SoundCommand::FollowingEntity(read_sound_entity_packet(
                            &mut input,
                        )?)
                    }
                    _ => {
                        voxel_game::sounds::SoundCommand::Stop(read_stop_sound_packet(&mut input)?)
                    }
                };
                if !input.is_empty() {
                    return Err(ConnectionError::Codec("sound packet trailing payload"));
                }
                self.sounds.push(command);
            }
            PLAY_CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID => {
                let (id, message) = parse_player_combat_kill_packet(payload)?;
                if self.player_id == Some(id) {
                    if self.show_death_screen {
                        self.death = Some(message);
                    } else {
                        self.respawn()?;
                    }
                }
            }
            PLAY_CLIENTBOUND_SET_EXPERIENCE_PACKET_ID => {
                let mut input = payload;
                let progress = read_f32(&mut input)?;
                if progress != self.status.experience_progress {
                    // `setExperienceDisplayStartTickToTickCount`: the first
                    // change, the one a login brings, never shows.
                    let tick_count = self.player.as_ref().map_or(0, LocalPlayer::tick_count);
                    self.status.experience_display_start_tick =
                        if self.status.experience_display_start_tick == i32::MIN {
                            -i32::MAX
                        } else {
                            tick_count
                        };
                }
                self.status.experience_progress = progress;
                self.status.experience_level = read_varint(&mut input, "experience level")?;
                self.status.total_experience = read_varint(&mut input, "total experience")?;
            }
            PLAY_CLIENTBOUND_SET_HELD_SLOT_PACKET_ID => {
                let mut input = payload;
                let slot = read_varint(&mut input, "held slot")?;
                // `Inventory.isHotbarSlot`: anything else is ignored.
                if (0..9).contains(&slot) {
                    self.status.selected_slot = slot;
                }
            }
            PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID => {
                let event = parse_game_event_packet(payload)?;
                // The weather events set the level's rain and thunder, both
                // the current and the previous tick's value, clamped.
                match event.event_id {
                    1 => self.rain_level = 0.0,
                    2 => self.rain_level = 1.0,
                    7 => self.rain_level = event.param.clamp(0.0, 1.0),
                    8 => self.thunder_level = event.param.clamp(0.0, 1.0),
                    _ => {}
                }
                self.trace
                    .world
                    .set_weather(self.rain_level, self.thunder_level);
                if event.event_id == GAME_EVENT_IMMEDIATE_RESPAWN_ID {
                    self.show_death_screen = event.param == 0.0;
                }
                if event.event_id == GAME_EVENT_CHANGE_GAME_MODE_ID {
                    self.game_mode = ClientPlayerGameMode::from_game_event_param(event.param);
                    if let Some(player) = self.player.as_mut() {
                        player.set_game_mode(self.game_mode);
                    }
                }
            }
            PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID => {
                let packet = parse_player_abilities_packet(payload)?;
                let abilities = ClientPlayerAbilities::new(
                    packet.invulnerable,
                    packet.flying,
                    packet.can_fly,
                    packet.can_instabuild,
                    packet.flying_speed,
                    packet.walking_speed,
                );
                self.abilities = Some(abilities);
                if let Some(player) = self.player.as_mut() {
                    player.set_abilities(abilities);
                }
            }
            PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID => {
                let packet = parse_player_position_packet(payload)?;
                self.teleports += 1;
                self.handle_player_position(packet)?;
                self.trace.world.prediction_teleport();
            }
            PLAY_CLIENTBOUND_SET_TIME_PACKET_ID => {
                let mut input = payload;
                let game_time = read_i64(&mut input)?;
                let count = wire::read_varint(&mut input)
                    .map_err(|_| ConnectionError::Codec("set time"))?;
                self.trace.world.set_game_time(game_time as u64);
                // handleUpdates: the clocks catch up first, then take the
                // server's values.
                self.tick_clocks(game_time);
                for _ in 0..count {
                    let clock = wire::read_varint(&mut input)
                        .map_err(|_| ConnectionError::Codec("set time clock"))?;
                    let total_ticks = wire::read_varlong(&mut input)
                        .map_err(|_| ConnectionError::Codec("set time ticks"))?;
                    let partial_tick = read_f32(&mut input)?;
                    let rate = read_f32(&mut input)?;
                    self.clocks.insert(
                        clock,
                        ClockState {
                            total_ticks,
                            partial_tick,
                            rate,
                        },
                    );
                }
            }
            PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID => {
                let sequence = read_varint(&mut &payload[..], "block changed ack sequence")?;
                for synced in self.trace.world.handle_block_changed_ack(sequence) {
                    self.mark_block(synced.x, synced.y, synced.z);
                    // `syncBlockState`: a block put back into the player
                    // moves it back to where it stood when it predicted.
                    let pos = voxel_core::binary::BlockPos::new(synced.x, synced.y, synced.z);
                    if let Some(position) = synced.player_position
                        && let Some(player) = self.player.as_mut()
                        && player.is_colliding(&self.trace.world, pos, synced.state)
                    {
                        player.abs_snap_to(position);
                    }
                }
            }
            PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID => {
                let packet = parse_block_destruction_packet(payload)?;
                self.trace.world.destroy_block_progress(
                    packet.entity_id,
                    packet.x,
                    packet.y,
                    packet.z,
                    i32::from(packet.progress),
                );
            }
            PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID => {
                let packet = parse_open_screen_packet(payload)?;
                self.trace.world.apply_local_player_open_menu(
                    packet.container_id,
                    packet.menu_type_id,
                    &packet.title_nbt,
                );
            }
            PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID => {
                let packet = parse_container_close_packet(payload)?;
                self.trace
                    .world
                    .apply_local_player_container_close(packet.container_id);
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID => {
                let packet = match parse_container_set_content_packet(payload) {
                    Ok(packet) => packet,
                    // An item with a component the parser does not know
                    // leaves that stack unknown rather than the session.
                    Err(error) => {
                        self.warn(format!("inventory: {error:?}"));
                        return self.maybe_finish_loading();
                    }
                };
                self.trace.world.apply_local_player_container_content(
                    packet.container_id,
                    packet.state_id,
                    &packet.items,
                    packet.carried_item.as_ref(),
                );
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID => {
                let packet = parse_container_set_data_packet(payload)?;
                self.trace.world.apply_local_player_container_data(
                    packet.container_id,
                    packet.id,
                    packet.value,
                );
            }
            PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID => match parse_system_chat_event(payload) {
                Ok(event) => self.chat_events.push(event),
                Err(error) => self.warn(format!("system chat: {error:?}")),
            },
            PLAY_CLIENTBOUND_DISGUISED_CHAT_PACKET_ID => {
                match parse_disguised_chat_event(payload) {
                    Ok(event) => self.chat_events.push(event),
                    Err(error) => self.warn(format!("disguised chat: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_PLAYER_CHAT_PACKET_ID => match parse_player_chat_packet(payload) {
                Ok(packet) => self.receive_player_chat(packet),
                Err(error) => self.warn(format!("player chat: {error:?}")),
            },
            PLAY_CLIENTBOUND_DELETE_CHAT_PACKET_ID => match parse_delete_chat_packet(payload) {
                Ok(packed) => match self.signature_cache.unpack(&packed) {
                    Some(signature) => self.chat_events.push(ChatEvent::Delete(signature)),
                    // The reference drops the connection over a number it
                    // does not know.
                    None => self.warn("delete chat: unknown signature".to_owned()),
                },
                Err(error) => self.warn(format!("delete chat: {error:?}")),
            },
            PLAY_CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID => {
                match parse_command_suggestions(payload) {
                    Ok(suggestions) => self.suggestions = Some(suggestions),
                    Err(error) => self.warn(format!("suggestions: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID
            | PLAY_CLIENTBOUND_CLEAR_TITLES_PACKET_ID
            | PLAY_CLIENTBOUND_BOSS_EVENT_PACKET_ID
            | PLAY_CLIENTBOUND_TAB_LIST_PACKET_ID => match parse_hud_packet(packet_id, payload) {
                Ok(Some(event)) => self.hud_events.push(event),
                Ok(None) => {}
                Err(error) => self.warn(format!("hud packet {packet_id}: {error:?}")),
            },
            PLAY_CLIENTBOUND_WAYPOINT_PACKET_ID => match parse_waypoint_packet(payload) {
                Ok((operation, waypoint)) => self.waypoints.apply(operation, waypoint),
                Err(error) => self.warn(format!("waypoint: {error:?}")),
            },
            PLAY_CLIENTBOUND_COMMANDS_PACKET_ID => match parse_commands_packet(payload) {
                Ok(tree) => self.commands = Some(std::sync::Arc::new(tree)),
                Err(error) => self.warn(format!("commands: {error:?}")),
            },
            PLAY_CLIENTBOUND_SET_OBJECTIVE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_SCORE_PACKET_ID
            | PLAY_CLIENTBOUND_RESET_SCORE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID => {
                match parse_scoreboard_packet(packet_id, payload) {
                    Ok(Some(event)) => self.scoreboard.apply(event),
                    Ok(None) => {}
                    Err(error) => self.warn(format!("scoreboard packet {packet_id}: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID => {
                match parse_player_info_update(payload) {
                    Ok(entries) => {
                        for (id, actions) in entries {
                            self.apply_player_info(id, actions);
                        }
                    }
                    Err(error) => self.warn(format!("player info: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID => {
                match parse_player_info_remove(payload) {
                    Ok(ids) => {
                        for id in ids {
                            self.player_infos.remove(&id);
                        }
                    }
                    Err(error) => self.warn(format!("player info remove: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID => {
                match parse_recipe_book_add_packet(payload) {
                    Ok((entries, replace)) => {
                        self.trace.world.recipe_book_mut().add(entries, replace);
                    }
                    Err(error) => self.warn(format!("recipe book add: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_RECIPE_BOOK_REMOVE_PACKET_ID => {
                match parse_recipe_book_remove_packet(payload) {
                    Ok(ids) => self.trace.world.recipe_book_mut().remove(&ids),
                    Err(error) => self.warn(format!("recipe book remove: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID => {
                match parse_recipe_book_settings_packet(payload) {
                    Ok(settings) => self.trace.world.recipe_book_mut().set_settings(settings),
                    Err(error) => self.warn(format!("recipe book settings: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_PLACE_GHOST_RECIPE_PACKET_ID => {
                match parse_place_ghost_recipe_packet(payload) {
                    Ok(ghost) => self.trace.world.recipe_book_mut().ghost = Some(ghost),
                    Err(error) => self.warn(format!("place ghost recipe: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_UPDATE_RECIPES_PACKET_ID => {
                match parse_update_recipes_packet(payload) {
                    Ok(update) => {
                        self.trace.world.apply_recipe_property_sets(update.sets);
                        self.trace
                            .world
                            .apply_stonecutter_recipes(update.stonecutter);
                    }
                    Err(error) => self.warn(format!("recipes: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID => {
                let packet = match parse_container_set_slot_packet(payload) {
                    Ok(packet) => packet,
                    // An item with a component the parser does not know
                    // leaves that stack unknown rather than the session.
                    Err(error) => {
                        self.warn(format!("inventory: {error:?}"));
                        return self.maybe_finish_loading();
                    }
                };
                self.trace.world.apply_local_player_container_slot(
                    packet.container_id,
                    packet.state_id,
                    packet.slot,
                    packet.item.as_ref(),
                );
            }
            PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID => {
                let packet = match parse_cursor_item_packet(payload) {
                    Ok(packet) => packet,
                    // An item with a component the parser does not know
                    // leaves that stack unknown rather than the session.
                    Err(error) => {
                        self.warn(format!("inventory: {error:?}"));
                        return self.maybe_finish_loading();
                    }
                };
                self.trace
                    .world
                    .apply_local_player_carried_item(packet.item.as_ref());
            }
            PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID => {
                let packet = match parse_set_player_inventory_packet(payload) {
                    Ok(packet) => packet,
                    // An item with a component the parser does not know
                    // leaves that stack unknown rather than the session.
                    Err(error) => {
                        self.warn(format!("inventory: {error:?}"));
                        return self.maybe_finish_loading();
                    }
                };
                self.trace
                    .world
                    .apply_local_player_inventory_slot(packet.slot, packet.item.as_ref());
            }
            PLAY_CLIENTBOUND_UPDATE_TAGS_PACKET_ID => {
                let mut input = payload;
                let packet = voxel_core::tags::ClientboundUpdateTagsPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("update tags"))?;
                self.trace.world.apply_update_tags_packet(&packet);
            }
            PLAY_CLIENTBOUND_DISCONNECT_PACKET_ID => {
                let mut input = payload;
                self.disconnect = Some(read_disconnect_reason(&mut input)?);
            }
            _ => {}
        }
        self.maybe_finish_loading()
    }

    /// `ClientPacketListener.handleMovePlayer`.
    /// `setValuesFromPositionPacket` for the local player: a rider stays where its vehicle puts it.
    fn place_player(&mut self, packet: &PlayerPositionPacket) {
        let passenger = self.trace.world.local_player_is_passenger();
        let current = self.player.as_ref().map(|player| {
            let position = player.position();
            let (y_rot, x_rot) = player.rotation();
            (position.x, position.y, position.z, y_rot, x_rot)
        });
        let absolute = resolve_player_position(*packet, current);
        let position = Vec3::new(absolute.0, absolute.1, absolute.2);
        let known_health = self.health_received.then_some(self.status.health);
        let player = self.player.get_or_insert_with(|| {
            let mut player = LocalPlayer::new(
                self.player_id.unwrap_or(0),
                position,
                absolute.3,
                absolute.4,
                self.game_mode,
                self.auto_jump,
            );
            // A health update that came before the player did is its first.
            if let Some(health) = known_health {
                player.hurt_to(health);
            }
            player
        });
        if let Some(abilities) = self.abilities {
            player.set_abilities(abilities);
        }
        let current_delta = player.delta_movement();
        let delta = Vec3::new(
            packet.delta_x
                + if packet.relative_flags & (1 << 5) != 0 {
                    current_delta.x
                } else {
                    0.0
                },
            packet.delta_y
                + if packet.relative_flags & (1 << 6) != 0 {
                    current_delta.y
                } else {
                    0.0
                },
            packet.delta_z
                + if packet.relative_flags & (1 << 7) != 0 {
                    current_delta.z
                } else {
                    0.0
                },
        );
        if !passenger {
            player.apply_teleport(position, delta, absolute.3, absolute.4);
        }
    }

    /// `handleMovePlayer`: where the server puts the player, which it accepts and answers with
    /// where the player now is.
    fn handle_player_position(
        &mut self,
        packet: PlayerPositionPacket,
    ) -> Result<(), ConnectionError> {
        self.place_player(&packet);
        self.send(PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, |output| {
            wire::write_varint(packet.teleport_id, output);
            Ok(())
        })?;
        self.send_player_pos_rot()
    }

    /// `handleTeleportEntity` for a vehicle that was removed under its rider: the rider is moved
    /// in its place and tells the server where it is.
    fn move_former_rider(
        &mut self,
        packet: &EntityTeleportObservation,
    ) -> Result<(), ConnectionError> {
        self.place_player(&PlayerPositionPacket {
            teleport_id: 0,
            x: packet.x,
            y: packet.y,
            z: packet.z,
            delta_x: packet.delta_x,
            delta_y: packet.delta_y,
            delta_z: packet.delta_z,
            y_rot: packet.y_rot,
            x_rot: packet.x_rot,
            relative_flags: packet.relative_flags,
        });
        self.send_player_pos_rot()
    }

    /// The `PosRot` the player sends after a teleport: where it is now, in the air, free of walls.
    fn send_player_pos_rot(&mut self) -> Result<(), ConnectionError> {
        let Some(player) = self.player.as_ref() else {
            return Ok(());
        };
        let position = player.position();
        let (y_rot, x_rot) = player.rotation();
        let absolute = (position.x, position.y, position.z, y_rot, x_rot);
        self.send(PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, |output| {
            encode_move_player_pos_rot(output, absolute, 0)
        })
    }

    /// The level counts as loaded once the player has a position and the
    /// chunk under it has arrived; then the client says so and starts ticking.
    fn maybe_finish_loading(&mut self) -> Result<(), ConnectionError> {
        if self.loaded {
            return Ok(());
        }
        let Some(player) = self.player.as_ref() else {
            return Ok(());
        };
        let position = player.position();
        let chunk_x = voxel_core::math::floor_f64(position.x).div_euclid(16);
        let chunk_z = voxel_core::math::floor_f64(position.z).div_euclid(16);
        if !self.trace.world.contains_chunk(chunk_x, chunk_z) {
            return Ok(());
        }
        self.send(PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID, |_| Ok(()))?;
        self.loaded = true;
        Ok(())
    }
}

fn read_f32(input: &mut &[u8]) -> Result<f32, ConnectionError> {
    super::protocol::read_f32(input, "float")
}

fn read_i64(input: &mut &[u8]) -> Result<i64, ConnectionError> {
    voxel_core::wire::take(input)
        .map(i64::from_be_bytes)
        .ok_or_else(|| {
            ConnectionError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "section position",
            ))
        })
}

/// The player's health, food, experience and selected hotbar slot, as the
/// server last set them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerStatus {
    pub health: f32,
    pub food: i32,
    pub saturation: f32,
    pub experience_progress: f32,
    pub experience_level: i32,
    pub total_experience: i32,
    pub selected_slot: i32,
    /// `experienceDisplayStartTick`: the player tick the experience last
    /// changed at, which keeps the experience bar over the locator bar for
    /// five seconds.
    pub experience_display_start_tick: i32,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            health: 20.0,
            food: 20,
            saturation: 5.0,
            experience_progress: 0.0,
            experience_level: 0,
            total_experience: 0,
            selected_slot: 0,
            experience_display_start_tick: i32::MIN,
        }
    }
}

/// A salt for a chat message: the clock's nanoseconds, which only has to
/// differ from one message to the next.
fn rand_salt() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |time| time.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    /// A session over a loopback socket, and the far end of it, which a
    /// test keeps open so what the session sends has somewhere to go.
    fn detached() -> (PlaySession, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (far_end, _) = listener.accept().unwrap();
        let (_sender, incoming) = mpsc::channel();
        let session = PlaySession::assemble(
            stream,
            None,
            incoming,
            PlayConnectionTrace::default(),
            ("test", false),
        );
        (session, far_end)
    }

    fn varint(mut value: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        loop {
            let low = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                bytes.push(low);
                return bytes;
            }
            bytes.push(low | 0x80);
        }
    }

    fn add_entity(id: u32, x: f64) -> Vec<u8> {
        add_entity_of_type(id, 94, x)
    }

    fn add_entity_of_type(id: u32, type_id: u32, x: f64) -> Vec<u8> {
        let mut body = varint(PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID as u32);
        body.extend(varint(id));
        body.extend([0_u8; 16]);
        body.extend(varint(type_id));
        for value in [x, 64.0, 8.0] {
            body.extend(value.to_be_bytes());
        }
        body.push(0);
        body.extend([0_u8; 3]);
        body.extend(varint(0));
        body
    }

    #[test]
    fn the_session_keeps_the_entities_the_server_adds_moves_and_removes() {
        let (mut session, _far_end) = detached();
        assert_eq!(session.entity_count(), 0);
        session.handle(&add_entity(7, 8.0)).unwrap();
        session.handle(&add_entity(8, 9.0)).unwrap();
        assert_eq!(session.entity_count(), 2);
        // A step of an entity that is there moves it; of one that is not, nothing.
        let mut step = varint(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID as u32);
        step.extend(varint(7));
        for delta in [4096_i16, 0, 0] {
            step.extend(delta.to_be_bytes());
        }
        step.push(1);
        session.handle(&step).unwrap();
        let moved = session.world().entity_at(7).unwrap().position();
        assert!((moved.0 - 9.0).abs() < 1e-9, "{moved:?}");
        // An entity packet that does not parse leaves the session alone.
        let mut broken = varint(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID as u32);
        broken.push(7);
        session.handle(&broken).unwrap();
        let mut remove = varint(PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID as u32);
        remove.extend(varint(1));
        remove.extend(varint(7));
        session.handle(&remove).unwrap();
        assert_eq!(session.entity_count(), 1);
        assert!(session.world().entity_at(7).is_none());
    }

    #[test]
    fn a_client_tick_moves_the_entities_on() {
        let (mut session, _far_end) = detached();
        // A player entity ticks wherever it is; others wait for their chunk.
        session.handle(&add_entity_of_type(7, 156, 8.0)).unwrap();
        let mut step = varint(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID as u32);
        step.extend(varint(7));
        for delta in [4096_i16, 0, 0] {
            step.extend(delta.to_be_bytes());
        }
        step.push(1);
        session.handle(&step).unwrap();
        let before = session.world().entity_at(7).unwrap().tick_count();
        session.tick(Input::EMPTY).unwrap();
        session.tick(Input::EMPTY).unwrap();
        let after = session.world().entity_at(7).unwrap().tick_count();
        assert_eq!(after, before + 2);
        // Between two ticks the entity is drawn between where it was and is.
        let (old, new) = {
            let entity = session.world().entity_at(7).unwrap();
            (entity.previous_position(), entity.position())
        };
        let half = session.entity_snapshot(7, 0.5).unwrap().position;
        assert!((half.0 - (old.0 + new.0) / 2.0).abs() < 1e-9);
        assert!(session.entity_snapshot(99, 0.5).is_none());
    }

    #[test]
    fn a_rider_is_told_how_to_dismount_and_follows_a_vehicle_removed_under_it() {
        let (mut session, _far_end) = detached();
        session.player_id = Some(1);
        session.trace.world.set_local_player_id(1);
        session.player = Some(LocalPlayer::new(
            1,
            Vec3::new(8.0, 64.0, 8.0),
            0.0,
            0.0,
            session.game_mode,
            session.auto_jump,
        ));
        let minecart = voxel_data::entity_type::id("minecraft:minecart").unwrap() as u32;
        session.handle(&add_entity_of_type(1, 156, 8.0)).unwrap();
        session
            .handle(&add_entity_of_type(7, minecart, 8.0))
            .unwrap();
        let mut passengers = varint(PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID as u32);
        passengers.extend(varint(7));
        passengers.extend(varint(1));
        passengers.extend(varint(1));
        session.handle(&passengers).unwrap();
        assert_eq!(session.world().ridden_vehicle_of_local_player(), Some(7));
        assert!(matches!(
            session.drain_hud().as_slice(),
            [HudEvent::ActionBar(message)] if matches!(
                &message.content,
                voxel_game::text::Content::Translatable { key, .. } if key == "mount.onboard"
            )
        ));
        // The server removes the minecart with the player in it, then moves "it".
        let mut remove = varint(PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID as u32);
        remove.extend(varint(1));
        remove.extend(varint(7));
        session.handle(&remove).unwrap();
        assert_eq!(session.removed_player_vehicle, Some(7));
        let mut teleport = varint(PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID as u32);
        teleport.extend(varint(7));
        for value in [20.0_f64, 70.0, -4.0, 0.0, 0.0, 0.0] {
            teleport.extend(value.to_be_bytes());
        }
        for value in [90.0_f32, 10.0] {
            teleport.extend(value.to_be_bytes());
        }
        teleport.extend(0_i32.to_be_bytes());
        teleport.push(0);
        session.handle(&teleport).unwrap();
        let player = session.player.as_ref().unwrap();
        let at = player.position();
        assert_eq!((at.x, at.y, at.z), (20.0, 70.0, -4.0));
        // Boarding again forgets the minecart that went.
        session
            .handle(&add_entity_of_type(7, minecart, 8.0))
            .unwrap();
        assert_eq!(session.removed_player_vehicle, None);
    }
}
