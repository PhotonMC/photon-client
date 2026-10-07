use crate::*;

impl ClientWorld {
    /// `ClientboundUpdateEnabledFeaturesPacket`: whether `minecart_improvements` is on, which the
    /// minecarts made after this read to choose their behaviour.
    pub fn set_minecart_improvements(&mut self, enabled: bool) {
        self.minecart_improvements = enabled;
    }

    /// `handleMinecartAlongTrack`: the steps a minecart of the new behaviour is to play.
    pub fn apply_minecart_steps(
        &mut self,
        entity_id: i32,
        steps: &[objects::minecart_track::MinecartStep],
    ) -> bool {
        let Some(track) = self
            .entities
            .get_mut(&entity_id)
            .and_then(|entity| entity.kind.minecart_slot_mut())
            .and_then(|slot| slot.track.as_mut())
        else {
            return false;
        };
        track.push_steps(steps);
        true
    }

    /// `OldMinecartBehavior.getPos`: where the rail puts a minecart that is at this place.
    pub fn minecart_rail_pos(&self, at: (f64, f64, f64)) -> Option<(f64, f64, f64)> {
        objects::minecart::rail_pos(&|x, y, z| self.block_state_at(x, y, z), at)
    }

    /// `OldMinecartBehavior.getPosOffs`: the same this far along the rail from here.
    pub fn minecart_rail_pos_offset(
        &self,
        at: (f64, f64, f64),
        offset: f64,
    ) -> Option<(f64, f64, f64)> {
        objects::minecart::rail_pos_offset(&|x, y, z| self.block_state_at(x, y, z), at, offset)
    }
}
