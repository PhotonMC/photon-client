//! The tracking emitters entity events and animations ask for.

use super::*;
use photon_core::ids::EntityId;

impl PlaySession {
    /// `createTrackingEmitter` for an entity the level holds, read as the packet is handled: where
    /// it is, and whether it is a player. An unknown entity asks for nothing.
    pub(super) fn queue_emitter(
        &mut self,
        entity: EntityId,
        particle: &'static str,
        life_time: i32,
        sound: Option<&'static str>,
    ) {
        let local = self
            .local
            .player
            .as_ref()
            .filter(|_| self.local.player_id == Some(entity));
        let (body, is_player) = if let Some(player) = local {
            let position = player.position();
            let body = photon_world::particles::EntityBody {
                position: (position.x, position.y, position.z),
                width: player.bounding_box().x_size() as f32,
                height: player.bb_height(),
            };
            (body, true)
        } else if let Some(found) = self.world.entities().at(entity) {
            (found.tracking_body(), found.is_player())
        } else {
            return;
        };
        self.effects
            .tracking_emitters
            .push(photon_world::particles::TrackingRequest {
                entity,
                particle,
                life_time,
                sound,
                body,
                is_player,
            });
    }
}
