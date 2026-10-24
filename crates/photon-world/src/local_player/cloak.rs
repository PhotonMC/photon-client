use super::*;

impl LocalPlayer {
    /// What the cape of the player hangs from between the last two ticks: where the cloak is,
    /// how far the player has walked and how much it bobs (`ClientAvatarState`).
    pub fn cloak_state(&self, partial_tick: f32) -> CloakState {
        let walk = mth_lerp_f32(
            partial_tick,
            self.avatar.walk_dist_old,
            self.avatar.walk_dist,
        );
        CloakState {
            position: self.avatar.cloak.interpolated(partial_tick),
            walk_distance: walk,
            bob: mth_lerp_f32(partial_tick, self.bob_old, self.bob),
        }
    }

    /// `Entity.rideTick`: the bob is put out after the tick of a passenger.
    pub(super) fn reset_bob(&mut self) {
        self.bob_old = self.bob;
        self.bob = 0.0;
    }
}

/// `ClientAvatarState`: the distance walked and the cloak, which the camera and the cape read.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct AvatarState {
    pub(super) walk_dist: f32,
    pub(super) walk_dist_old: f32,
    pub(super) cloak: crate::cloak::Cloak,
}

/// The values the cape of an avatar is hung from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CloakState {
    /// `getInterpolatedCloakX`, `Y` and `Z`.
    pub position: [f64; 3],
    /// `getInterpolatedWalkDistance`; the remote players never add to it.
    pub walk_distance: f32,
    /// `getInterpolatedBob`.
    pub bob: f32,
}
