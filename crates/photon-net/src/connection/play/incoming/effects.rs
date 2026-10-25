//! The attributes and mob effects of entities and the local player.

use super::*;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (
        PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID,
        typed::<UpdateAttributes>,
    ),
    (
        PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID,
        typed::<UpdateMobEffect>,
    ),
    (
        PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID,
        typed::<RemoveMobEffect>,
    ),
];

impl ClientboundPacket for UpdateAttributes {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_update_attributes(self.0)
    }
}

impl ClientboundPacket for UpdateMobEffect {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_update_mob_effect(self.0)
    }
}

impl ClientboundPacket for RemoveMobEffect {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_remove_mob_effect(self.0)
    }
}

impl PlaySession {
    fn on_update_attributes(
        &mut self,
        packet: AttributeUpdateObservation,
    ) -> Result<(), ConnectionError> {
        let world = &mut self.world;
        if world.apply_local_player_attributes(packet.entity_id, &packet.attributes) {
            // The player's own copy moves it; the world's outlives it for the next player made.
            if let Some(player) = self.local.player.as_mut() {
                player.set_attributes(&packet.attributes);
            }
        } else {
            world
                .entities_mut()
                .apply_attributes(packet.entity_id, &packet.attributes);
        }
        Ok(())
    }

    fn on_update_mob_effect(
        &mut self,
        packet: MobEffectUpdateObservation,
    ) -> Result<(), ConnectionError> {
        self.world.apply_local_player_mob_effect(
            packet.entity_id,
            packet.effect_registry_id,
            packet.amplifier,
            packet.duration_ticks,
            packet.flags,
        );
        Ok(())
    }

    fn on_remove_mob_effect(
        &mut self,
        packet: MobEffectRemoveObservation,
    ) -> Result<(), ConnectionError> {
        self.world
            .remove_local_player_mob_effect(packet.entity_id, packet.effect_registry_id);
        Ok(())
    }
}
