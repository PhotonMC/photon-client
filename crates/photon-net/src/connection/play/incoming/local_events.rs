//! What the server's entity events do to the local player's own state.

use super::*;
use photon_core::ids::EntityId;

impl PlaySession {
    /// `LivingEntity.handleEntityEvent`: 2 is the kick of a spear's hit, which the server sends to
    /// the attacker too.
    pub(super) fn local_kinetic_hit(&mut self, id: EntityId, event: i8) {
        if event == 2
            && self.local.player_id == Some(id)
            && let Some(player) = self.local.player.as_mut()
        {
            player.on_kinetic_hit();
        }
    }

    /// `Player.handleEntityEvent`: 9 completes the use of the item the player holds, here as on
    /// the server.
    pub(super) fn local_use_complete(&mut self, id: EntityId, event: i8) {
        if event == 9
            && self.local.player_id == Some(id)
            && let Some(player) = self.local.player.as_mut()
        {
            photon_game::game_mode::finish_use(
                &mut self.world,
                player,
                &mut self.local.item_cooldowns,
                self.local.status.selected_slot,
            );
        }
    }
}
