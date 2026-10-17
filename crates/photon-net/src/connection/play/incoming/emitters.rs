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

    /// `ClientPacketListener.findTotem`: the stack in a hand that protects from death, the main
    /// hand first, or a plain totem when neither holds one. Read as the packet is handled.
    pub(super) fn queue_item_activation(&mut self) {
        let held = |slot: i16| {
            self.world
                .local_player()
                .and_then(|state| state.inventory_slot(slot))
                .filter(|stack| stack.item_count() > 0)
        };
        let main = 36 + self.local.status.selected_slot as i16;
        let protects = |stack: &&photon_world::ClientItemStackState| {
            photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
                .has(photon_data::item::component_types::DEATH_PROTECTION)
        };
        let found = held(main)
            .filter(protects)
            .or_else(|| held(45).filter(protects))
            .cloned();
        self.effects.item_activation = found.or_else(|| {
            photon_game::item::id_of("minecraft:totem_of_undying")
                .map(|totem| photon_world::ClientItemStackState::new(1, totem))
        });
    }
}
