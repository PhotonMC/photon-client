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

use crate::input::Input;
use crate::math::Vec3;
use crate::protocol::ConnectionProtocol;
use crate::wire;
use crate::world::local_player::{LocalPlayer, ServerboundMovement};
use crate::world::{
    ClientPlayerAbilities, ClientPlayerGameMode, ClientWorld, DimensionInput, SectionInput,
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
    game_time: i64,
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
            game_time: 0,
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

    /// One client tick: the local player, its packets, then the tick end.
    pub fn tick(&mut self, keys: Input) -> Result<(), ConnectionError> {
        if self.loaded
            && let Some(player) = self.player.as_mut()
        {
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
                self.trace.world.set_local_player_id(login.player_id);
                self.player_id = Some(login.player_id);
                self.game_mode =
                    ClientPlayerGameMode::from_id(i32::from(login.spawn_info.game_type));
            }
            PLAY_CLIENTBOUND_RESPAWN_PACKET_ID => {
                let (spawn_info, _) = parse_respawn_packet(payload)?;
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
                self.dimension_type_id = spawn_info.dimension_type_id;
                // A respawn makes a new player that waits for its position.
                self.player = None;
                self.loaded = false;
            }
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => {
                let chunk = parse_chunk_packet(payload)?;
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
            }
            PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID => {
                let (x, z) = parse_forget_level_chunk_packet(payload)?;
                self.trace.world.unload_chunk(x, z);
            }
            PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID => {
                let update = parse_block_update_packet(payload)?;
                self.trace
                    .world
                    .update_block_state(update.x, update.y, update.z, update.state_id);
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
                    self.trace.world.update_block_state(
                        section_x * 16 + ((local >> 8) & 15),
                        section_y * 16 + (local & 15),
                        section_z * 16 + ((local >> 4) & 15),
                        state,
                    );
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
            PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID => {
                let event = parse_game_event_packet(payload)?;
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
        let player = self.player.get_or_insert_with(|| {
            LocalPlayer::new(
                self.player_id.unwrap_or(0),
                position,
                absolute.3,
                absolute.4,
                self.game_mode,
                self.auto_jump,
            )
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
        let chunk_x = crate::math::floor_f64(position.x).div_euclid(16);
        let chunk_z = crate::math::floor_f64(position.z).div_euclid(16);
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
