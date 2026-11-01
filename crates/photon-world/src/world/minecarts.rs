use crate::*;

impl ClientWorld {
    /// `ClientboundUpdateEnabledFeaturesPacket`: whether `minecart_improvements` is on, which the
    /// minecarts made after this read to choose their behaviour.
    /// The numbering of the version the session speaks. A world starts with the newest version's.
    pub fn set_versions(
        &mut self,
        versions: std::sync::Arc<photon_data::version_registries::VersionRegistries>,
    ) {
        self.versions = versions;
    }

    pub fn set_minecart_improvements(&mut self, enabled: bool) {
        self.minecart_improvements = enabled;
    }

    /// `handleMinecartAlongTrack`: the steps a minecart of the new behaviour is to play.
    pub fn apply_minecart_steps(
        &mut self,
        entity_id: EntityId,
        steps: &[objects::minecart_track::MinecartStep],
    ) -> bool {
        let Some(track) = self
            .entities
            .at_mut(entity_id)
            .and_then(|entity| entity.into_kind().minecart_slot_mut())
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

    /// `!isOnRails() && getBehavior() instanceof NewMinecartBehavior`: a minecart of the new
    /// behaviour that is not on a rail, which rolls silently.
    pub fn minecart_off_rail(&self, entity: crate::entity::EntityView<'_>) -> bool {
        entity.minecart_track().is_some()
            && !objects::minecart::on_rails_new(
                &|x, y, z| self.block_state_at(x, y, z),
                entity.position(),
            )
    }

    /// `Entity.isUnderWater` of an entity the world holds: its eyes are in water (the fluid the
    /// last tick found there; a client entity is in water wherever its eyes are).
    pub fn entity_under_water(&self, entity: crate::entity::EntityView<'_>) -> bool {
        let (x, y, z) = entity.position();
        let eye_y = y + f64::from(entity.eye_height());
        let pos = photon_core::binary::BlockPos::new(
            photon_core::math::floor_f64(x),
            photon_core::math::floor_f64(eye_y),
            photon_core::math::floor_f64(z),
        );
        let level = crate::local_player::Level::new(self);
        let info = level.info(pos);
        info.is_water() && f64::from(pos.y) + f64::from(level.fluid_height(pos, &info)) > eye_y
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
