//! A live play session: the connection kept open, the world kept current,
//! and the local player ticked against it.
//!
//! The probes in this crate each drive one scripted scene and hang up. This
//! session stays: a reader thread takes packets off the socket as they come,
//! the caller drains them between frames, and each client tick runs the
//! local player and sends what the tick asked for, then the tick-end marker,
//! as `Minecraft.tick` does.

use photon_data::version_registries::VersionRegistries;
use std::net::SocketAddr;

use super::transport::Transport;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use photon_core::math::Vec3;
use photon_core::wire;
use photon_world::input::Input;
use photon_world::local_player::{LocalPlayer, ServerboundMovement};
use photon_world::{
    BlockEntityInput, ClientPlayerAbilities, ClientPlayerGameMode, ClientWorld, DimensionInput,
    LightLayerInput, LightUpdateInput, SectionInput,
};

use super::entities;
use super::open::open_play_connection;
use super::record::NoRecorder;
use super::session::{frame_packet, read_packet};
use super::state::ConnectionState;
use super::writer::PacketWriter;
use groups::*;
use incoming::Incoming;
use photon_protocol::play::*;

mod books;
mod chat;
mod chat_session;
mod chat_validation;
mod chunks;
mod command_blocks;
mod containers;
mod groups;
mod incoming;
mod last_seen;
mod level_load;
pub use level_load::LoadingReason;
use level_load::{LevelLoadTracker, PlayerNeed};
mod lifecycle;
mod menu_sync;
mod movement;
#[cfg(test)]
mod pack_tests;
#[cfg(test)]
mod profile_tests;
mod recipes;
mod reconfigure;
pub use command_blocks::{CommandBlockSettings, CommandBlockState};
pub use groups::GlobalBlock;
pub use reconfigure::SwitchKind;
mod jigsaw_blocks;
mod signs;
pub use jigsaw_blocks::JigsawBlockSettings;
mod structure_blocks;
mod test_blocks;
mod test_instances;
pub use structure_blocks::{StructureBlockSettings, StructureFlags};
mod state;
mod traffic;

pub(super) const PLAY_CLIENTBOUND_DISCONNECT_PACKET_ID: i32 = 32;

pub(super) const PLAY_CLIENTBOUND_PING_PACKET_ID: i32 = 61;

pub(super) const PLAY_CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID: i32 = 84;

pub(super) const PLAY_CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID: i32 = 97;

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

/// A connection in the play phase with its world and local player.
pub struct PlaySession {
    pub(super) state: ConnectionState,
    pub(super) world: ClientWorld,
    link: Link,
    local: LocalState,
    menus: Menus,
    social: Social,
    effects: Effects,
    level_info: LevelInfo,
    /// The numbering of the connection's version, for ids that arrive after the packet is parsed.
    registries: std::sync::Arc<VersionRegistries>,
}

/// `ClientClockManager`'s state for one world clock.
#[derive(Debug, Clone, Copy)]
pub(super) struct ClockState {
    pub(super) total_ticks: i64,
    pub(super) partial_tick: f32,
    pub(super) rate: f32,
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
    use photon_core::ids::EntityId;
    use std::net::{TcpListener, TcpStream};

    /// A session over a loopback socket, and the far end of it, which a
    /// test keeps open so what the session sends has somewhere to go.
    fn detached() -> (PlaySession, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = Transport::from(TcpStream::connect(listener.local_addr().unwrap()).unwrap());
        let (far_end, _) = listener.accept().unwrap();
        let (_sender, incoming) = mpsc::channel();
        let session = PlaySession::assemble(
            stream,
            None,
            incoming,
            ConnectionState::default(),
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

    /// An entity that has no interpolation handler, so a step puts it at the place at once.
    fn add_entity(id: u32, x: f64) -> Vec<u8> {
        add_entity_of_type(id, wire_entity_type("minecraft:arrow"), x)
    }

    /// The id the connect version's wire gives the type.
    fn wire_entity_type(name: &str) -> u32 {
        let canonical = photon_data::entity_type::named(name).0;
        photon_data::version_registries::VersionRegistries::of(crate::connection::connect_version())
            .entity_types
            .wire_or_unknown(canonical) as u32
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
    fn award_stats_set_the_numbers_the_screen_reads() {
        let (mut session, _far_end) = detached();
        let jump = StatKey {
            stat_type: 8,
            value: 0,
        };
        assert_eq!(session.stat(jump), 0);
        assert!(!session.take_stats_updated());
        let mut body = varint(PLAY_CLIENTBOUND_AWARD_STATS_PACKET_ID as u32);
        body.extend(varint(2));
        for (stat_type, value, amount) in [(8, 0, 5), (0, 1, 7)] {
            body.extend(varint(stat_type));
            body.extend(varint(value));
            body.extend(varint(amount));
        }
        session.handle_bytes(&body).unwrap();
        assert_eq!(session.stat(jump), 5);
        assert_eq!(
            session.stat(StatKey {
                stat_type: 0,
                value: 1
            }),
            7
        );
        assert!(session.take_stats_updated());
        assert!(!session.take_stats_updated());
        // The server's number replaces the one held; it is not added to it.
        let mut again = varint(PLAY_CLIENTBOUND_AWARD_STATS_PACKET_ID as u32);
        again.extend(varint(1));
        again.extend([8, 0, 2]);
        session.handle_bytes(&again).unwrap();
        assert_eq!(session.stat(jump), 2);
    }

    #[test]
    fn the_session_keeps_the_entities_the_server_adds_moves_and_removes() {
        let (mut session, _far_end) = detached();
        assert_eq!(session.entity_count(), 0);
        session.handle_bytes(&add_entity(7, 8.0)).unwrap();
        session.handle_bytes(&add_entity(8, 9.0)).unwrap();
        assert_eq!(session.entity_count(), 2);
        // A step of an entity that is there moves it; of one that is not, nothing.
        let mut step = varint(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID as u32);
        step.extend(varint(7));
        for delta in [4096_i16, 0, 0] {
            step.extend(delta.to_be_bytes());
        }
        step.push(1);
        session.handle_bytes(&step).unwrap();
        let moved = session
            .world()
            .entities()
            .at(photon_core::ids::EntityId(7))
            .unwrap()
            .position();
        assert!((moved.0 - 9.0).abs() < 1e-9, "{moved:?}");
        // An entity packet that does not parse leaves the session alone.
        let mut broken = varint(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID as u32);
        broken.push(7);
        session.handle_bytes(&broken).unwrap();
        let mut remove = varint(PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID as u32);
        remove.extend(varint(1));
        remove.extend(varint(7));
        session.handle_bytes(&remove).unwrap();
        assert_eq!(session.entity_count(), 1);
        assert!(
            session
                .world()
                .entities()
                .at(photon_core::ids::EntityId(7))
                .is_none()
        );
    }

    #[test]
    fn a_client_tick_moves_the_entities_on() {
        let (mut session, _far_end) = detached();
        // A player entity ticks wherever it is; others wait for their chunk.
        session
            .handle_bytes(&add_entity_of_type(
                7,
                wire_entity_type("minecraft:player"),
                8.0,
            ))
            .unwrap();
        let mut step = varint(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID as u32);
        step.extend(varint(7));
        for delta in [4096_i16, 0, 0] {
            step.extend(delta.to_be_bytes());
        }
        step.push(1);
        session.handle_bytes(&step).unwrap();
        let before = session
            .world()
            .entities()
            .at(photon_core::ids::EntityId(7))
            .unwrap()
            .tick_count();
        session.tick(Input::EMPTY).unwrap();
        session.tick(Input::EMPTY).unwrap();
        let after = session
            .world()
            .entities()
            .at(photon_core::ids::EntityId(7))
            .unwrap()
            .tick_count();
        assert_eq!(after, before + 2);
        // Between two ticks the entity is drawn between where it was and is.
        let (old, new) = {
            let entity = session
                .world()
                .entities()
                .at(photon_core::ids::EntityId(7))
                .unwrap();
            (entity.previous_position(), entity.position())
        };
        let half = session
            .entity_snapshot(photon_core::ids::EntityId(7), 0.5)
            .unwrap()
            .position;
        assert!((half.0 - (old.0 + new.0) / 2.0).abs() < 1e-9);
        assert!(
            session
                .entity_snapshot(photon_core::ids::EntityId(99), 0.5)
                .is_none()
        );
    }

    #[test]
    fn a_rider_is_told_how_to_dismount_and_follows_a_vehicle_removed_under_it() {
        let (mut session, _far_end) = detached();
        session.local.player_id = Some(photon_core::ids::EntityId(1));
        session
            .world
            .set_local_player_id(photon_core::ids::EntityId(1));
        session.local.player = Some(LocalPlayer::new(
            photon_core::ids::EntityId(1),
            Vec3::new(8.0, 64.0, 8.0),
            0.0,
            0.0,
            session.local.game_mode,
            session.link.auto_jump,
        ));
        let minecart = wire_entity_type("minecraft:minecart");
        session
            .handle_bytes(&add_entity_of_type(
                1,
                wire_entity_type("minecraft:player"),
                8.0,
            ))
            .unwrap();
        session
            .handle_bytes(&add_entity_of_type(7, minecart, 8.0))
            .unwrap();
        let mut passengers = varint(PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID as u32);
        passengers.extend(varint(7));
        passengers.extend(varint(1));
        passengers.extend(varint(1));
        session.handle_bytes(&passengers).unwrap();
        assert_eq!(
            session.world().ridden_vehicle_of_local_player(),
            Some(EntityId(7))
        );
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
        session.handle_bytes(&remove).unwrap();
        assert_eq!(session.local.removed_player_vehicle, Some(EntityId(7)));
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
        session.handle_bytes(&teleport).unwrap();
        let player = session.local.player.as_ref().unwrap();
        let at = player.position();
        assert_eq!((at.x, at.y, at.z), (20.0, 70.0, -4.0));
        // Boarding again forgets the minecart that went.
        session
            .handle_bytes(&add_entity_of_type(7, minecart, 8.0))
            .unwrap();
        assert_eq!(session.local.removed_player_vehicle, None);
    }

    #[test]
    fn set_camera_hands_the_view_to_a_known_entity_and_back() {
        let (mut session, _far_end) = detached();
        let own = EntityId(1);
        session.local.player_id = Some(own);
        session.world.set_local_player_id(own);
        session.local.player = Some(LocalPlayer::new(
            own,
            Vec3::new(8.0, 64.0, 8.0),
            0.0,
            0.0,
            session.local.game_mode,
            session.link.auto_jump,
        ));
        session
            .handle_bytes(&add_entity_of_type(
                1,
                wire_entity_type("minecraft:player"),
                8.0,
            ))
            .unwrap();
        session
            .handle_bytes(&add_entity_of_type(
                7,
                wire_entity_type("minecraft:pale_oak_boat"),
                8.0,
            ))
            .unwrap();
        let camera = |id: u32| {
            let mut body = varint(PLAY_CLIENTBOUND_SET_CAMERA_PACKET_ID as u32);
            body.extend(varint(id));
            body
        };
        let controlled = |session: &PlaySession| {
            session
                .local
                .player
                .as_ref()
                .unwrap()
                .is_controlled_camera()
        };
        assert!(controlled(&session));
        // An entity the client never heard of changes nothing.
        session.handle_bytes(&camera(99)).unwrap();
        assert_eq!(session.camera_entity(), None);
        assert!(controlled(&session));
        session.handle_bytes(&camera(7)).unwrap();
        assert_eq!(session.camera_entity(), Some(EntityId(7)));
        assert!(!controlled(&session));
        // Naming the player itself gives the view back.
        session.handle_bytes(&camera(1)).unwrap();
        assert_eq!(session.camera_entity(), None);
        assert!(controlled(&session));
    }

    /// `handleRespawn`: the new player's attributes come from the old player's own, the sprinting
    /// modifier the client put on among them, kept whole or only as their bases.
    #[test]
    fn a_respawn_carries_the_old_players_own_attributes() {
        use photon_world::{ClientAttributeModifierState, ClientAttributeState};
        let speed = photon_data::entity_type::attribute("minecraft:movement_speed")
            .unwrap()
            .0 as i32;
        for (keep, expected) in [
            (
                photon_world::PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS,
                0.1 * (1.0 + 0.3),
            ),
            (0, 0.1),
        ] {
            let (mut session, _far_end) = detached();
            let own = EntityId(1);
            session.local.player_id = Some(own);
            session.world.set_local_player_id(own);
            let mut player = LocalPlayer::new(
                own,
                Vec3::new(8.0, 64.0, 8.0),
                0.0,
                0.0,
                session.local.game_mode,
                session.link.auto_jump,
            );
            // The client's own copy has the modifier; the server never sent it.
            let sprinting = ClientAttributeModifierState::new("minecraft:sprinting".into(), 0.3, 2);
            player.set_attributes(&[ClientAttributeState::new(speed, 0.1, vec![sprinting])]);
            session.local.player = Some(player);
            session.recreate_player_state(keep);
            let state = session.world.local_player().unwrap();
            assert_eq!(
                state.attribute_value("minecraft:movement_speed"),
                Some(expected)
            );
        }
    }

    /// A server never adds the client's own player as an entity, yet naming it ends a spectating.
    #[test]
    fn set_camera_to_the_player_returns_the_view_without_an_added_player() {
        let (mut session, _far_end) = detached();
        let own = EntityId(1);
        session.local.player_id = Some(own);
        session.local.player = Some(LocalPlayer::new(
            own,
            Vec3::new(8.0, 64.0, 8.0),
            0.0,
            0.0,
            session.local.game_mode,
            session.link.auto_jump,
        ));
        session
            .handle_bytes(&add_entity_of_type(
                7,
                wire_entity_type("minecraft:pale_oak_boat"),
                8.0,
            ))
            .unwrap();
        let camera = |id: u32| {
            let mut body = varint(PLAY_CLIENTBOUND_SET_CAMERA_PACKET_ID as u32);
            body.extend(varint(id));
            body
        };
        session.handle_bytes(&camera(7)).unwrap();
        assert!(
            !session
                .local
                .player
                .as_ref()
                .unwrap()
                .is_controlled_camera()
        );
        session.handle_bytes(&camera(1)).unwrap();
        assert_eq!(session.camera_entity(), None);
        assert!(
            session
                .local
                .player
                .as_ref()
                .unwrap()
                .is_controlled_camera()
        );
    }
}
