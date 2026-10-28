use super::*;

impl ClientEntity {
    /// `ClientAvatarState.tick` of a remote player: the cloak follows the position it has at the
    /// start of the tick.
    pub(crate) fn tick_cloak(&mut self) {
        let position = self.position();
        self.living.cloak.tick(position);
    }

    /// `AbstractClientPlayer.updateBob`: the bob eases to the walking speed (at most a tenth of a
    /// block a tick) while the player is on the ground and neither dying nor swimming.
    pub(crate) fn tick_bob(&mut self) {
        let (x, _, z) = self.motion();
        let walking = self.on_ground() && !self.is_dead_or_dying() && !self.is_swimming();
        let target = if walking {
            0.1_f32.min(photon_core::math::mth_length_f64_2(x, z) as f32)
        } else {
            0.0
        };
        self.living.cloak.update_bob(target);
    }

    /// `ClientAvatarState.getInterpolatedBob` as `Camera.extractRenderState` reads it for
    /// `bobView`: a player's, no other entity has one.
    pub fn camera_bob(&self, partial: f32) -> f32 {
        if self.is_player() {
            self.living.cloak.interpolated_bob(partial)
        } else {
            0.0
        }
    }

    /// The cloak and the bob of the avatar's cape.
    pub fn cloak(&self) -> &crate::cloak::Cloak {
        &self.living.cloak
    }
}
