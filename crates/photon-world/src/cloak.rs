//! `ClientAvatarState`'s cloak and bob: the point a cape hangs from, which follows the avatar a
//! quarter of the way each tick, and the walking bob that lifts it.

use photon_core::math::{mth_lerp_f32, mth_lerp_f64};

/// How far from the avatar the cloak may lag before it is moved at once.
const TELEPORT_THRESHOLD: f64 = 10.0;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Cloak {
    position: [f64; 3],
    old_position: [f64; 3],
    bob: f32,
    old_bob: f32,
}

// An entity compares as a whole, so its cloak must too; the positions it holds are finite.
impl Eq for Cloak {}

impl Cloak {
    /// `ClientAvatarState.moveCloak`, at the start of the avatar's tick.
    pub fn tick(&mut self, position: (f64, f64, f64)) {
        self.old_position = self.position;
        let target = [position.0, position.1, position.2];
        for (axis, &wanted) in target.iter().enumerate() {
            let delta = wanted - self.position[axis];
            // NaN is within the threshold, as the reference's negated comparisons have it.
            if delta.abs() > TELEPORT_THRESHOLD {
                self.position[axis] = wanted;
                self.old_position[axis] = wanted;
            } else {
                self.position[axis] += delta * 0.25;
            }
        }
    }

    /// `ClientAvatarState.updateBob`.
    pub fn update_bob(&mut self, target: f32) {
        self.old_bob = self.bob;
        self.bob += (target - self.bob) * 0.4;
    }

    /// `ClientAvatarState.resetBob`.
    pub fn reset_bob(&mut self) {
        self.old_bob = self.bob;
        self.bob = 0.0;
    }

    /// `getInterpolatedCloakX`, `Y` and `Z`.
    pub fn interpolated(&self, partial: f32) -> [f64; 3] {
        let alpha = f64::from(partial);
        std::array::from_fn(|axis| {
            mth_lerp_f64(alpha, self.old_position[axis], self.position[axis])
        })
    }

    /// `getInterpolatedBob`.
    pub fn interpolated_bob(&self, partial: f32) -> f32 {
        mth_lerp_f32(partial, self.old_bob, self.bob)
    }
}

#[cfg(test)]
mod tests;
