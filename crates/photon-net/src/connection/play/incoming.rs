use std::collections::HashMap;
use std::sync::OnceLock;

use super::*;
use crate::connection::probe::{store_cookie, take_custom_payload, take_report_details};

mod chat;
mod connection;
mod containers;
mod entity;
mod player;
mod recipes;
mod world;

/// A clientbound play packet, parsed (the reference's `Packet`): what it does to the client is
/// its `handle`, the listener's `handleX` of the reference.
pub(super) trait ClientboundPacket: Send {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError>;

    /// Whether loading may be over once the packet is handled.
    fn checks_loading(&self) -> bool {
        true
    }
}

/// A packet as the reader thread hands it over; the order the packets came in is kept, so an
/// error surfaces where it always did.
pub(super) type Incoming = Box<dyn ClientboundPacket>;

/// Parses the packet with this id from its payload.
type Decoder = fn(i32, &[u8]) -> Result<Incoming, ConnectionError>;

/// The packets of the families that are typed, by packet id.
fn decoders() -> &'static HashMap<i32, Decoder> {
    static DECODERS: OnceLock<HashMap<i32, Decoder>> = OnceLock::new();
    DECODERS.get_or_init(|| {
        [
            world::DECODERS,
            connection::DECODERS,
            chat::DECODERS,
            recipes::DECODERS,
            player::DECODERS,
            containers::DECODERS,
        ]
        .concat()
        .into_iter()
        .collect()
    })
}

/// Parse the packet in `body` (its id, then its payload). A family that is not typed yet stays
/// bytes, parsed where it is handled.
pub(super) fn decode_packet(body: Vec<u8>) -> Result<Incoming, ConnectionError> {
    let (packet_id, payload) = split_packet(&body)?;
    match decoders().get(&packet_id) {
        Some(decode) => decode(packet_id, payload),
        None => Ok(Box::new(RawPacket(body))),
    }
}

/// The decoder of a packet type, as its table row holds it.
fn typed<P>(_packet_id: i32, payload: &[u8]) -> Result<Incoming, ConnectionError>
where
    P: Decode + ClientboundPacket + 'static,
{
    match P::decode(payload) {
        Ok(packet) => Ok(Box::new(packet)),
        Err(error) => match P::ON_ERROR {
            OnError::Fail => Err(error),
            OnError::Ignore(name) => {
                log::debug!("ignored {name} packet: {error}");
                Ok(Box::new(Ignored))
            }
            OnError::Warn(label) => Ok(Box::new(Malformed(format!("{label}: {error:?}")))),
        },
    }
}

/// A packet of a family that is not typed yet: its whole body.
struct RawPacket(Vec<u8>);

impl ClientboundPacket for RawPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.handle(&self.0)
    }

    /// `handle` ends with the check itself.
    fn checks_loading(&self) -> bool {
        false
    }
}

/// A packet that did not parse and is dropped.
struct Ignored;

impl ClientboundPacket for Ignored {
    fn handle(self: Box<Self>, _session: &mut PlaySession) -> Result<(), ConnectionError> {
        Ok(())
    }
}

/// A packet that did not parse: the session warns, once for each message.
struct Malformed(String);

impl ClientboundPacket for Malformed {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.warn(self.0);
        Ok(())
    }
}

impl PlaySession {
    /// A packet from the reader thread, applied to the client's state.
    pub(super) fn handle_incoming(&mut self, packet: Incoming) -> Result<(), ConnectionError> {
        let checks_loading = packet.checks_loading();
        packet.handle(self)?;
        if checks_loading {
            self.maybe_finish_loading()?;
        }
        Ok(())
    }

    /// A packet's bytes, parsed and applied: the way a test hands one over.
    #[cfg(test)]
    pub(super) fn handle_bytes(&mut self, body: &[u8]) -> Result<(), ConnectionError> {
        self.handle_incoming(decode_packet(body.to_vec())?)
    }

    /// A packet of a family that is not typed yet, parsed where it is handled.
    pub(super) fn handle(&mut self, body: &[u8]) -> Result<(), ConnectionError> {
        let (packet_id, payload) = split_packet(body)?;
        match packet_id {
            PLAY_CLIENTBOUND_START_CONFIGURATION_PACKET_ID => self.reconfigure()?,
            PLAY_CLIENTBOUND_TRANSFER_PACKET_ID => self.transfer(payload)?,
            PLAY_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID => self.on_resource_pack_push(payload)?,
            // Nothing was loaded, so there is nothing to take off.
            PLAY_CLIENTBOUND_RESOURCE_PACK_POP_PACKET_ID => {
                parse_resource_pack_pop(payload)?;
            }
            PLAY_CLIENTBOUND_SHOW_DIALOG_PACKET_ID => self.on_show_dialog(payload)?,
            PLAY_CLIENTBOUND_CLEAR_DIALOG_PACKET_ID => self.dialog_events.push(DialogEvent::Clear),
            PLAY_CLIENTBOUND_COOKIE_REQUEST_PACKET_ID => self.on_cookie_request(payload)?,
            PLAY_CLIENTBOUND_STORE_COOKIE_PACKET_ID => store_cookie(&mut self.trace, payload)?,
            PLAY_CLIENTBOUND_CUSTOM_PAYLOAD_PACKET_ID => {
                take_custom_payload(&mut self.trace, payload)?;
            }
            PLAY_CLIENTBOUND_CUSTOM_REPORT_DETAILS_PACKET_ID => {
                take_report_details(&mut self.trace, payload)?;
            }
            PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID => self.on_add_entity(payload)?,
            PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID => self.on_set_entity_motion(payload)?,
            PLAY_CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID => self.on_map_item_data(payload)?,
            PLAY_CLIENTBOUND_MOVE_MINECART_PACKET_ID => self.on_move_minecart(payload)?,
            PLAY_CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID => self.on_set_entity_link(payload)?,
            PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID => self.on_rotate_head(payload)?,
            PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID => self.on_remove_entities(payload)?,
            PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID => self.on_set_passengers(payload)?,
            PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID => {
                self.on_entity_position_sync(payload)?
            }
            PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID => self.on_teleport_entity(payload)?,
            PLAY_CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID => self.on_take_item_entity(payload)?,
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID => self.on_move_entity_pos(payload)?,
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID => {
                self.on_move_entity_pos_rot(payload)?
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID => self.on_move_entity_rot(payload)?,
            PLAY_CLIENTBOUND_ANIMATE_PACKET_ID => self.on_animate(payload)?,
            PLAY_CLIENTBOUND_ENTITY_EVENT_PACKET_ID => self.on_entity_event(payload)?,
            PLAY_CLIENTBOUND_DAMAGE_EVENT_PACKET_ID => self.on_damage_event(payload)?,
            PLAY_CLIENTBOUND_HURT_ANIMATION_PACKET_ID => self.on_hurt_animation(payload)?,
            PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID => self.on_forget_level_chunk(payload)?,
            PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID => self.on_block_update(payload)?,
            PLAY_CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID => {
                self.on_section_blocks_update(payload)?
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID => {
                self.on_set_chunk_cache_radius(payload)?
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID => {
                self.on_set_chunk_cache_center(payload)?
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID => {
                self.on_chunk_batch_finished(payload)?
            }
            PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID => self.on_set_entity_data(payload)?,
            PLAY_CLIENTBOUND_SET_EQUIPMENT_PACKET_ID => self.on_set_equipment(payload)?,
            PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID => self.on_update_attributes(payload)?,
            PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID => self.on_update_mob_effect(payload)?,
            PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID => self.on_remove_mob_effect(payload)?,
            PLAY_CLIENTBOUND_LEVEL_EVENT_PACKET_ID => self.on_level_event(payload)?,
            PLAY_CLIENTBOUND_EXPLODE_PACKET_ID => self.on_explode(payload)?,
            PLAY_CLIENTBOUND_LEVEL_PARTICLES_PACKET_ID => self.on_level_particles(payload)?,
            PLAY_CLIENTBOUND_SOUND_PACKET_ID
            | PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID
            | PLAY_CLIENTBOUND_STOP_SOUND_PACKET_ID => self.on_sound(payload, packet_id)?,
            PLAY_CLIENTBOUND_SET_TIME_PACKET_ID => self.on_set_time(payload)?,
            PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID => self.on_block_changed_ack(payload)?,
            PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID => self.on_block_destruction(payload)?,
            PLAY_CLIENTBOUND_OPEN_SIGN_EDITOR_PACKET_ID => self.on_open_sign_editor(payload)?,
            PLAY_CLIENTBOUND_OPEN_BOOK_PACKET_ID => self.on_open_book(payload)?,
            PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID => self.on_block_entity_data(payload)?,
            PLAY_CLIENTBOUND_BLOCK_EVENT_PACKET_ID => self.on_block_event(payload)?,
            PLAY_CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID
            | PLAY_CLIENTBOUND_CLEAR_TITLES_PACKET_ID
            | PLAY_CLIENTBOUND_BOSS_EVENT_PACKET_ID
            | PLAY_CLIENTBOUND_TAB_LIST_PACKET_ID => self.on_hud(payload, packet_id)?,
            PLAY_CLIENTBOUND_SET_OBJECTIVE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_SCORE_PACKET_ID
            | PLAY_CLIENTBOUND_RESET_SCORE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID => {
                self.on_scoreboard(payload, packet_id)?
            }
            _ => {}
        }
        self.maybe_finish_loading()
    }
}
