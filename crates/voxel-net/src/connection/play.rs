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
const PLAY_SERVERBOUND_PLAYER_ABILITIES_PACKET_ID: i32 = 40;
const PLAY_SERVERBOUND_PLAYER_COMMAND_PACKET_ID: i32 = 42;
const PLAY_SERVERBOUND_PLAYER_INPUT_PACKET_ID: i32 = 43;

/// A connection in the play phase with its world and local player.
pub struct PlaySession {
    writer: TcpStream,
    compression_threshold: Option<usize>,
    incoming: Receiver<Result<Vec<u8>, ConnectionError>>,
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
    /// Whether a respawn was asked for since the player last had health.
    respawn_requested: bool,
    /// Swing packets sent, for scripted runs to report.
    swings: u32,
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
    inventory_menu: voxel_game::inventory_menu::InventoryMenu,
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
                let packet = read_packet(&mut reader, compression_threshold);
                let failed = packet.is_err();
                if sender.send(packet).is_err() || failed {
                    return;
                }
            }
        });
        Ok(Self {
            writer: stream,
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
            respawn_requested: false,
            swings: 0,
            hardcore: false,
            health_received: false,
            level_reset: false,
            sent_slot: 0,
            game_mode_state: voxel_game::game_mode::GameMode::default(),
            inventory_menu: voxel_game::inventory_menu::InventoryMenu::default(),
            drop_spam: 0,
            dirty_sections: std::collections::BTreeSet::new(),
            forgotten_chunks: Vec::new(),
            clocks: std::collections::BTreeMap::new(),
        })
    }

    pub fn world(&self) -> &ClientWorld {
        &self.trace.world
    }

    pub fn player(&self) -> Option<&LocalPlayer> {
        self.player.as_ref()
    }

    pub fn player_mut(&mut self) -> Option<&mut LocalPlayer> {
        self.player.as_mut()
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
            match self.incoming.try_recv() {
                Ok(packet) => self.handle(&packet?)?,
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => return Err(ConnectionError::ServerDisconnected),
            }
        }
    }

    /// Wait up to `limit` for the next packet and handle it.
    pub fn wait(&mut self, limit: Duration) -> Result<(), ConnectionError> {
        match self.incoming.recv_timeout(limit) {
            Ok(packet) => self.handle(&packet?),
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
    pub fn inventory_menu(&mut self) -> &voxel_game::inventory_menu::InventoryMenu {
        if let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() {
            self.inventory_menu.load(slots, carried, state_id);
        }
        &self.inventory_menu
    }

    /// `AbstractContainerScreen.slotClicked` into the player's own menu:
    /// the click as the client predicts it, the arm swings its drops cause,
    /// then the click packet.
    pub fn container_click(
        &mut self,
        slot: i32,
        button: i32,
        kind: voxel_game::inventory_menu::ClickType,
    ) -> Result<(), ConnectionError> {
        let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() else {
            return Ok(());
        };
        self.inventory_menu.load(slots, carried, state_id);
        let player = voxel_game::inventory_menu::Player {
            selected: self.status.selected_slot.clamp(0, 8) as usize,
            infinite_materials: self.game_mode == voxel_world::ClientPlayerGameMode::Creative,
        };
        let (swings, body) = self.inventory_menu.handle_click(slot, button, kind, player);
        let menu = &self.inventory_menu;
        self.trace
            .world
            .set_local_player_menu(&menu.slots, menu.carried.clone());
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
        self.inventory_menu.load(slots, carried, state_id);
        let player = voxel_game::inventory_menu::Player {
            selected: self.status.selected_slot.clamp(0, 8) as usize,
            infinite_materials: self.game_mode == voxel_world::ClientPlayerGameMode::Creative,
        };
        // `Player.canDropItems`: under the throttler's threshold.
        let can_drop = self.drop_spam < 1280;
        let effects = creative.slot_clicked(click, &mut self.inventory_menu, player, can_drop);
        let menu = &self.inventory_menu;
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
    /// then `InventoryMenu.removed`, which empties the result slot.
    pub fn close_inventory(&mut self) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_CONTAINER_CLOSE_PACKET_ID, |output| {
            output.push(0);
            Ok(())
        })?;
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
        if self.loaded
            && let Some(player) = self.player.as_mut()
        {
            player.set_main_hand_item(main_hand);
            match player.tick(&self.trace.world, keys) {
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
        // ClientLevel.tickTime: the game time and the clocks move on.
        self.tick_clocks(self.game_time + 1);
        self.send(PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID, |_| Ok(()))
    }

    fn send<F>(&mut self, id: i32, body: F) -> Result<(), ConnectionError>
    where
        F: FnOnce(&mut Vec<u8>) -> Result<(), ConnectionError>,
    {
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
        };
        let body = packet.encode_body();
        self.send(id, |output| {
            output.extend_from_slice(&body);
            Ok(())
        })
    }

    fn handle(&mut self, body: &[u8]) -> Result<(), ConnectionError> {
        let (packet_id, payload) = split_packet(body)?;
        match packet_id {
            PLAY_CLIENTBOUND_LOGIN_PACKET_ID => {
                let login = parse_login_packet(payload)?;
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
                self.player_id = Some(login.player_id);
                self.game_mode =
                    ClientPlayerGameMode::from_id(i32::from(login.spawn_info.game_type));
            }
            PLAY_CLIENTBOUND_RESPAWN_PACKET_ID => {
                let (spawn_info, data_to_keep) = parse_respawn_packet(payload)?;
                // A new level starts dry; the server repeats its weather.
                self.rain_level = 0.0;
                self.thunder_level = 0.0;
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
                self.loaded = false;
                self.level_reset = true;
                self.health_received = false;
            }
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => {
                let chunk = parse_chunk_packet(payload)?;
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
            PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID => {
                let update = parse_light_update_packet(payload)?;
                // `readSectionList` redraws the sections whose light arrived
                // when the queued task runs.
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
                let packet = parse_entity_metadata_packet(payload)?;
                self.trace
                    .world
                    .apply_entity_metadata(packet.entity_id, &packet.metadata);
            }
            PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID => {
                let packet = parse_update_attributes_packet(payload)?;
                self.trace
                    .world
                    .apply_local_player_attributes(packet.entity_id, &packet.attributes);
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
                // The death screen is not ported, so a dead player asks to
                // respawn at once, as the reference does when the server
                // turns the death screen off.
                if packet.health <= 0.0 {
                    if !self.respawn_requested {
                        self.respawn_requested = true;
                        self.send(
                            PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
                            encode_perform_respawn,
                        )?;
                    }
                } else {
                    self.respawn_requested = false;
                }
            }
            PLAY_CLIENTBOUND_SET_EXPERIENCE_PACKET_ID => {
                let mut input = payload;
                self.status.experience_progress = read_f32(&mut input)?;
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
                return Err(ConnectionError::ServerDisconnected);
            }
            _ => {}
        }
        self.maybe_finish_loading()
    }

    /// `ClientPacketListener.handleMovePlayer`.
    fn handle_player_position(
        &mut self,
        packet: PlayerPositionPacket,
    ) -> Result<(), ConnectionError> {
        let current = self.player.as_ref().map(|player| {
            let position = player.position();
            let (y_rot, x_rot) = player.rotation();
            (position.x, position.y, position.z, y_rot, x_rot)
        });
        let absolute = resolve_player_position(packet, current);
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
        player.apply_teleport(position, delta, absolute.3, absolute.4);
        self.send(PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, |output| {
            wire::write_varint(packet.teleport_id, output);
            Ok(())
        })?;
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
    if input.len() < 4 {
        return Err(ConnectionError::Codec("float"));
    }
    let (head, rest) = input.split_at(4);
    *input = rest;
    Ok(f32::from_be_bytes(head.try_into().expect("four bytes")))
}

fn read_i64(input: &mut &[u8]) -> Result<i64, ConnectionError> {
    if input.len() < 8 {
        return Err(ConnectionError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "section position",
        )));
    }
    let (head, rest) = input.split_at(8);
    *input = rest;
    Ok(i64::from_be_bytes(head.try_into().expect("eight bytes")))
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
        }
    }
}
