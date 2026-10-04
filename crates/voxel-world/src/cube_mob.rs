//! What a slime or a magma cube does of its own on the client
//! (`AbstractCubeMob.tick`): the squash it takes when it lands and the
//! stretch it takes when it leaves the ground.

use voxel_core::math::mth_lerp_f32;

/// `AbstractCubeMob`'s squish, with the ground it stood on a tick ago.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CubeMobState {
    target_squish: f32,
    squish: f32,
    old_squish: f32,
    was_on_ground: bool,
}

/// The squish is finite, so the comparison is a total one.
impl Eq for CubeMobState {}

impl CubeMobState {
    /// One tick: the squish eases half way to its target, a landing sets the
    /// target to a squash and a take-off to a stretch (the sound and the
    /// particles that go with a landing are not made), and the target decays
    /// by `decay` (`decreaseSquish`: 0.6, a magma cube's 0.9).
    pub(crate) fn tick(&mut self, on_ground: bool, decay: f32) {
        self.old_squish = self.squish;
        self.squish += (self.target_squish - self.squish) * 0.5;
        if on_ground && !self.was_on_ground {
            self.target_squish = -0.5;
        } else if !on_ground && self.was_on_ground {
            self.target_squish = 1.0;
        }
        self.was_on_ground = on_ground;
        self.target_squish *= decay;
    }

    /// `AbstractCubeMobRenderer.extractRenderState`'s squish between two ticks.
    pub fn squish(&self, partial_tick: f32) -> f32 {
        mth_lerp_f32(partial_tick, self.old_squish, self.squish)
    }

    pub fn current_squish(&self) -> f32 {
        self.squish
    }

    pub fn old_squish(&self) -> f32 {
        self.old_squish
    }

    pub fn target_squish(&self) -> f32 {
        self.target_squish
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_landing_squashes_and_a_take_off_stretches() {
        let mut state = CubeMobState::default();
        state.tick(true, 0.6);
        assert_eq!(state.target_squish(), -0.5 * 0.6);
        state.tick(true, 0.6);
        assert!(state.current_squish() < 0.0);
        state.tick(false, 0.6);
        assert_eq!(state.target_squish(), 0.6);
    }

    #[test]
    fn the_squish_between_two_ticks_lies_between_them() {
        let mut state = CubeMobState::default();
        state.tick(true, 0.9);
        state.tick(true, 0.9);
        let (old, now) = (state.old_squish(), state.current_squish());
        assert!(now < old);
        assert!((now..=old).contains(&state.squish(0.5)));
    }
}
