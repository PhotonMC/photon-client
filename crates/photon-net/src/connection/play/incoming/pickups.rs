//! `handleTakeItemEntity`: an item or orb taken, and where it flies.

use super::*;
use photon_core::ids::EntityId;
use photon_world::particles::pickup_target;

impl PlaySession {
    pub(super) fn on_take_item_entity(
        &mut self,
        (item_id, player_id, amount): (EntityId, EntityId, i32),
    ) -> Result<(), ConnectionError> {
        let world = &self.world;
        let is_local = Some(player_id) == self.local.player_id;
        let taker = world
            .entities()
            .at(player_id)
            .filter(|entity| !is_local && entity.is_living_entity())
            .map(|entity| (player_id, entity.position(), entity.eye_height()));
        // A taker the level does not know is the local player in 26.2 and nobody in 26.3.
        let falls_back = is_local
            || photon_core::versions::rules()
                .world
                .unknown_taker_is_the_local_player;
        let target_at = match taker {
            Some((_, position, eye_height)) => Some(pickup_target(position, eye_height)),
            None if falls_back => self.local.player.as_ref().map(|player| {
                let at = player.position();
                pickup_target((at.x, at.y, at.z), player.eye_height())
            }),
            None => None,
        };
        // The sound and the shrunk stack happen whoever takes the item; only the flight needs a taker.
        let taken = self
            .world
            .take_item_entity(item_id, amount, &mut self.effects.listener_random);
        if let (Some(target_at), Some(taken)) = (target_at, taken) {
            self.effects
                .item_pickups
                .push(photon_world::particles::ItemPickup::new(
                    taken,
                    taker.map(|(id, ..)| id),
                    target_at,
                ));
        }
        Ok(())
    }
}
