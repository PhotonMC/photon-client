//! What the server's entity events do to the local player's own state.

use super::*;
use photon_core::ids::EntityId;

impl PlaySession {
    /// `Entity.lerpMotion`: only `RemotePlayer` lerps the motion; the local player takes it as
    /// it is, so a knockback or a launch pad moves it on the next tick.
    pub(super) fn local_motion(&mut self, packet: &EntityMotionObservation) -> bool {
        if self.local.player_id != Some(packet.entity_id) {
            return false;
        }
        let Some(player) = self.local.player.as_mut() else {
            return false;
        };
        player.set_delta_movement(photon_core::math::Vec3::new(packet.x, packet.y, packet.z));
        true
    }

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

    /// `LivingEntity.handleEntityEvent` 47 to 52, 65 and 68: the item the local player wears or
    /// holds breaks, which sounds from the inventory it is still in.
    pub(super) fn local_break_sound(&mut self, id: EntityId, event: i8) {
        if self.local.player_id != Some(id) {
            return;
        }
        let Some(slot) = photon_world::ClientWorld::broken_slot(event) else {
            return;
        };
        let Some(menu_slot) = player_menu_slot(slot, self.local.status.selected_slot) else {
            return;
        };
        let Some(stack) = self
            .world
            .local_player()
            .and_then(|state| state.inventory_slot(menu_slot))
            .cloned()
        else {
            return;
        };
        let Some(position) = self.local.player.as_ref().map(|player| player.position()) else {
            return;
        };
        self.world.play_item_break(
            &stack,
            photon_world::local_player::LocalSoundSource::Players,
            (position.x, position.y, position.z),
        );
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

    /// `LivingEntity.handleEntityEvent(46)`: the portal particles of a teleport, around the
    /// local player or the entity the world holds.
    pub(super) fn show_teleport(&mut self, id: EntityId, event: i8) {
        if event != 46 {
            return;
        }
        match self.local.player.as_mut() {
            Some(player) if self.local.player_id == Some(id) => player.throw_teleport_particles(),
            _ => self.world.throw_teleport_particles(id),
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
        // `onSyncedDataUpdated` acts on each packet's use flags, so a use the server ended and
        // began again between two ticks still starts the client's own.
        if let Some(flags) = photon_world::entity_living_flags(&packet.metadata) {
            player.synced_use_flags(flags & 1 != 0, flags & 2 != 0);
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

/// The slot of the player's inventory menu that holds what the equipment slot does; the local
/// player has none for a body or a saddle.
fn player_menu_slot(slot: photon_world::EquipmentSlot, selected: i32) -> Option<i16> {
    use photon_world::EquipmentSlot;
    Some(match slot {
        EquipmentSlot::MainHand => 36 + selected as i16,
        EquipmentSlot::OffHand => 45,
        EquipmentSlot::Head => 5,
        EquipmentSlot::Chest => 6,
        EquipmentSlot::Legs => 7,
        EquipmentSlot::Feet => 8,
        EquipmentSlot::Body | EquipmentSlot::Saddle => return None,
    })
}
