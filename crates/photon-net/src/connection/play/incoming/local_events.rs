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

    /// What the local player takes from its own entity data: the shared flags' gliding bit (flag
    /// 7) is the server's word over the client's start, and the sleeping place puts it in a bed.
    pub(super) fn local_metadata(&mut self, packet: &EntityMetadataObservation) {
        if self.local.player_id != Some(packet.entity_id) {
            return;
        }
        let Some(player) = self.local.player.as_mut() else {
            return;
        };
        if let Some((_, [flags, ..])) = photon_world::entity_metadata_field(&packet.metadata, 0) {
            player.set_server_fall_flying(flags & 0x80 != 0);
        }
        if let Some(bed) = photon_world::entity_sleeping_pos(&packet.metadata) {
            player.set_sleeping_pos(bed);
        }
    }

    /// `handleAnimate`'s wake-up (action 2) for the local player.
    pub(super) fn wake_local_player(&mut self) {
        if let Some(player) = self.local.player.as_mut() {
            player.set_sleeping_pos(None);
        }
        self.world.wake_local_player();
    }
}
