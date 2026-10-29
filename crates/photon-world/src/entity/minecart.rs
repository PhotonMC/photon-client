use super::*;
use photon_core::ids::BlockStateId;

impl ClientEntity {
    /// A minecart of the new behaviour: the steps the server sent along the track.
    pub fn minecart_track(&self) -> Option<&crate::objects::minecart_track::TrackLerp> {
        self.kind
            .minecart_slot()
            .and_then(|slot| slot.track.as_ref())
    }

    /// Whether this is one of the minecarts (`AbstractMinecart`).
    pub fn is_minecart(&self) -> bool {
        photon_data::entity_type::name(self.type_id)
            .and_then(crate::objects::minecart::default_display_offset)
            .is_some()
    }

    /// A minecart's hit, carried block and fuel, if it has any.
    pub fn minecart(&self) -> Option<&crate::objects::minecart::MinecartState> {
        self.kind
            .minecart_slot()
            .and_then(|slot| slot.state.as_ref())
    }

    pub(crate) fn minecart_mut(&mut self) -> &mut crate::objects::minecart::MinecartState {
        self.kind
            .minecart_slot_or_insert_with(Default::default)
            .state
            .get_or_insert_with(Default::default)
    }

    /// `AbstractMinecart.getDisplayBlockState`: the block state the server named, else the
    /// type's own (the plain minecart shows none).
    pub fn minecart_display_block(&self) -> Option<BlockStateId> {
        let state = self.minecart().copied().unwrap_or_default();
        state.custom_display.or_else(|| {
            photon_data::entity_type::name(self.type_id)
                .and_then(|name| crate::objects::minecart::default_display_block(name, state.fuel))
        })
    }

    /// `AbstractMinecart.getDisplayOffset`, in sixteenths of a block.
    pub fn minecart_display_offset(&self) -> i32 {
        let default = photon_data::entity_type::name(self.type_id)
            .and_then(crate::objects::minecart::default_display_offset)
            .unwrap_or(6);
        self.minecart()
            .copied()
            .unwrap_or_default()
            .display_offset(default)
    }
}

pub(crate) fn is_vanilla_minecart_entity_type(entity_type_id: EntityTypeId) -> bool {
    VANILLA_MINECART_ENTITY_TYPE_IDS.contains(&entity_type_id.0)
}

impl EntityViewMut<'_> {
    /// `AbstractMinecart.tick` on the client, with `OldMinecartBehavior.tick`'s client half: the
    /// hit runs down, then the position and turn ease to the last packet's while that lasts,
    /// else the turns are kept under a whole turn.
    pub(crate) fn tick_minecart(&mut self) {
        self.minecart_mut().tick();
        if photon_data::entity_type::name(self.type_id) == Some("minecraft:tnt_minecart") {
            self.minecart_mut().burn_fuse();
        }
        if self.minecart_track().is_some() {
            self.tick_minecart_track();
        } else if self.as_view().is_interpolating() {
            self.tick_position_interpolation();
        } else {
            let (y_rot, x_rot) = (self.rotation.y_rot.get(), self.rotation.x_rot.get());
            self.rotation.x_rot = JavaF32::new(x_rot % 360.0);
            self.rotation.y_rot = JavaF32::new(y_rot % 360.0);
        }
    }

    /// `NewMinecartBehavior.lerpClientPositionAndRotation`: the steps the server sent are played
    /// back, putting the minecart where they lead.
    pub(super) fn tick_minecart_track(&mut self) {
        let (y_rot, x_rot) = self.rotation.body_rotation();
        let now = crate::objects::minecart_track::MinecartStep {
            position: self.position(),
            movement: self.motion(),
            y_rot,
            x_rot,
            weight: 0.0,
        };
        let Some(place) = self
            .kind
            .minecart_slot_mut()
            .and_then(|slot| slot.track.as_mut())
            .and_then(|track| track.tick(now))
        else {
            return;
        };
        self.set_position(place.position);
        self.vel.motion.x = JavaF64::new(place.movement.0);
        self.vel.motion.y = JavaF64::new(place.movement.1);
        self.vel.motion.z = JavaF64::new(place.movement.2);
        // `Entity.setXRot` and `setYRot` discard what is not finite.
        if place.x_rot.is_finite() {
            self.rotation.x_rot = JavaF32::new((place.x_rot % 360.0).clamp(-90.0, 90.0));
        }
        if place.y_rot.is_finite() {
            self.rotation.y_rot = JavaF32::new(place.y_rot);
        }
    }
}
