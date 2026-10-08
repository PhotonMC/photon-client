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
use photon_core::math::Vec3;
use photon_core::wire;
use photon_world::input::Input;
use photon_world::local_player::{LocalPlayer, ServerboundMovement};
use photon_world::{
    BlockEntityInput, ClientPlayerAbilities, ClientPlayerGameMode, ClientWorld, DimensionInput,
    LightLayerInput, LightUpdateInput, SectionInput,
};

use super::entities;
use super::probe::{
    dimension_bounds_for_spawn_info, dimension_environment_for_spawn_info, open_play_connection,
};
use super::protocol::*;
use super::session::{read_packet, send_packet};

mod books;
mod chat;
mod chunks;
mod containers;
mod incoming;
mod lifecycle;
mod movement;
mod recipes;
mod signs;
mod state;

pub(super) const PLAY_CLIENTBOUND_DISCONNECT_PACKET_ID: i32 = 32;

pub(super) const PLAY_CLIENTBOUND_PING_PACKET_ID: i32 = 61;

pub(super) const PLAY_CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID: i32 = 84;

pub(super) const PLAY_CLIENTBOUND_SET_TIME_PACKET_ID: i32 = 113;

pub(super) const PLAY_CLIENTBOUND_UPDATE_TAGS_PACKET_ID: i32 = 134;

pub(super) const PLAY_SERVERBOUND_PONG_PACKET_ID: i32 = 45;

pub(super) const PLAY_SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID: i32 = 30;

pub(super) const PLAY_SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID: i32 = 32;

pub(super) const PLAY_SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID: i32 = 33;

pub(super) const PLAY_SERVERBOUND_MOVE_VEHICLE_PACKET_ID: i32 = 34;

pub(super) const PLAY_SERVERBOUND_PADDLE_BOAT_PACKET_ID: i32 = 35;

pub(super) const PLAY_SERVERBOUND_PLAYER_ABILITIES_PACKET_ID: i32 = 40;

pub(super) const PLAY_SERVERBOUND_PLAYER_COMMAND_PACKET_ID: i32 = 42;

pub(super) const PLAY_SERVERBOUND_PLAYER_INPUT_PACKET_ID: i32 = 43;

/// A packet as the reader thread hands it over. Chunk and light packets,
/// which are the heavy ones to parse, arrive already parsed; the order the
/// packets came in is kept, so an error surfaces where it always did.
pub(super) enum Incoming {
    Raw(Vec<u8>),
    Chunk(Box<ChunkPacketObservation>),
    Light(Box<LightUpdateObservation>),
}

/// Parse the packets worth parsing off the game's thread.
pub(super) fn classify(body: Vec<u8>) -> Result<Incoming, ConnectionError> {
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
    pub(super) writer: TcpStream,
    pub(super) compression_threshold: Option<usize>,
    pub(super) incoming: Receiver<Result<Incoming, ConnectionError>>,
    pub(super) trace: PlayConnectionTrace,
    pub(super) player_id: Option<i32>,
    pub(super) game_mode: ClientPlayerGameMode,
    pub(super) abilities: Option<ClientPlayerAbilities>,
    pub(super) player: Option<LocalPlayer>,
    pub(super) loaded: bool,
    pub(super) auto_jump: bool,
    pub(super) warned: Option<String>,
    pub(super) teleports: u32,
    pub(super) view_distance: i32,
    /// What the server was last told of the client (`remoteClientInformation`).
    pub(super) client_information: crate::configuration::ClientInformation,
    pub(super) dimension_type_id: i32,
    pub(super) hashed_seed: i64,
    pub(super) game_time: i64,
    pub(super) rain_level: f32,
    pub(super) thunder_level: f32,
    pub(super) status: PlayerStatus,
    /// `LocalPlayer.shouldShowDeathScreen`: off when the level respawns at once.
    pub(super) show_death_screen: bool,
    /// Why the connection is over, once it is: the server's word or what
    /// the connection said (`onDisconnect`).
    pub(super) disconnect: Option<photon_game::text::Component>,
    /// The message of a death the death screen is yet to show.
    pub(super) death: Option<photon_game::text::Component>,
    /// Swing packets sent, for scripted runs to report.
    pub(super) swings: u32,
    /// The sound packets since the last [`Self::take_sounds`].
    pub(super) sounds: Vec<photon_game::sounds::SoundCommand>,
    /// The particle packets since the last [`Self::take_level_particles`].
    pub(super) level_particles: Vec<photon_world::particles::LevelParticles>,
    /// The entities taken since the last [`Self::take_item_pickups`], flying to their takers.
    pub(super) item_pickups: Vec<photon_world::particles::ItemPickup>,
    /// `ClientPacketListener.random`: the pitch of a pickup's sound.
    pub(super) listener_random: photon_core::random::RandomSource,
    /// The entity the crosshair last landed on (`Minecraft.hitResult`).
    pub(super) aimed_entity: Option<i32>,
    /// Whether the level is hardcore, as the login said.
    pub(super) hardcore: bool,
    /// Whether a health update has arrived since the player last respawned.
    pub(super) health_received: bool,
    /// Set when the server moved the player to a new level.
    pub(super) level_reset: bool,
    /// The hotbar slot the server was last told about.
    pub(super) sent_slot: i32,
    /// Breaking state and the miss wait.
    pub(super) game_mode_state: photon_game::game_mode::GameMode,
    /// The player's inventory menu, whose drag state lasts between clicks.
    pub(super) container_menu: photon_game::menus::container::ContainerMenu,
    /// The menu of the container the server opened, likewise.
    pub(super) open_menu: Option<photon_game::menus::container::ContainerMenu>,
    /// The sign the server last asked the player to write on, and whether its front, until the
    /// client opens its screen.
    pub(super) sign_editor: Option<((i32, i32, i32), bool)>,
    /// What the server last said of the merchant whose menu is open, by container.
    pub(super) merchant: Option<(i32, photon_game::menus::merchant::MerchantState)>,
    /// Chat the server sent that the client has not shown yet, and the
    /// `nextChatIndex` its player chat must come in.
    pub(super) chat_events: Vec<ChatEvent>,
    pub(super) signature_cache: SignatureCache,
    /// Title, action bar and tab list packets not yet shown, and the players
    /// the tab list names.
    pub(super) hud_events: Vec<HudEvent>,
    /// The vehicle the server removed with the local player aboard (`removedPlayerVehicleId`).
    pub(super) removed_player_vehicle: Option<i32>,
    pub(super) scoreboard: photon_game::scoreboard::Scoreboard,
    pub(super) waypoints: photon_game::waypoint::WaypointManager,
    /// The player's operator level, 0 to 4 (`LocalPlayer.permissions`), which
    /// the server tells by entity events 24 to 28.
    pub(super) permission_level: i32,
    pub(super) commands: Option<std::sync::Arc<photon_game::commands::CommandTree>>,
    pub(super) username: String,
    pub(super) suggestions: Option<CommandSuggestions>,
    pub(super) player_infos: std::collections::BTreeMap<(u64, u64), PlayerInfo>,
    pub(super) next_chat_index: i32,
    /// `LocalPlayer.dropSpamThrottler`: what creative drops have added up.
    pub(super) drop_spam: i32,
    pub(super) dirty_sections: std::collections::BTreeSet<(i32, i32, i32)>,
    pub(super) forgotten_chunks: Vec<(i32, i32)>,
    pub(super) clocks: std::collections::BTreeMap<i32, ClockState>,
}

/// `ClientClockManager`'s state for one world clock.
#[derive(Debug, Clone, Copy)]
pub(super) struct ClockState {
    pub(super) total_ticks: i64,
    pub(super) partial_tick: f32,
    pub(super) rate: f32,
}

/// The packet a parser made, or nothing after logging why it could not: the
/// reference ignores a packet it cannot place, and the log keeps the reason.
pub(super) fn parsed<T, E: std::fmt::Display>(result: Result<T, E>, packet: &str) -> Option<T> {
    result
        .map_err(|error| log::debug!("ignored {packet} packet: {error}"))
        .ok()
}

pub(super) fn read_f32(input: &mut &[u8]) -> Result<f32, ConnectionError> {
    super::protocol::read_f32(input, "float")
}

pub(super) fn read_i64(input: &mut &[u8]) -> Result<i64, ConnectionError> {
    photon_core::wire::take(input)
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
pub(super) fn rand_salt() -> u32 {
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
        let moved = session.world().entities().at(7).unwrap().position();
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
        assert!(session.world().entities().at(7).is_none());
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
        let before = session.world().entities().at(7).unwrap().tick_count();
        session.tick(Input::EMPTY).unwrap();
        session.tick(Input::EMPTY).unwrap();
        let after = session.world().entities().at(7).unwrap().tick_count();
        assert_eq!(after, before + 2);
        // Between two ticks the entity is drawn between where it was and is.
        let (old, new) = {
            let entity = session.world().entities().at(7).unwrap();
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
        let minecart = photon_data::entity_type::id("minecraft:minecart").unwrap() as u32;
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
                photon_game::text::Content::Translatable { key, .. } if key == "mount.onboard"
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
