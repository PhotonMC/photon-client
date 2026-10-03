//! What a polar bear does of its own on the client (`PolarBear.tick`): how
//! far it has stood up on its hind legs.

use voxel_core::math::mth_lerp_f32 as lerp_f32;

/// The ticks a bear takes to stand up fully (`getStandingAnimationScale`).
const STAND_TICKS: f32 = 6.0;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PolarBearState {
    standing: bool,
    stand: f32,
    stand_o: f32,
}

// The amounts are finite, so equality is total.
impl Eq for PolarBearState {}

impl PolarBearState {
    /// Takes `PolarBear.DATA_STANDING_ID`.
    pub(crate) fn set_standing(&mut self, standing: bool) {
        self.standing = standing;
    }

    /// One client tick: the stand animation steps by one towards the data.
    pub(crate) fn tick(&mut self) {
        self.stand_o = self.stand;
        let step = if self.standing { 1.0 } else { -1.0 };
        self.stand = (self.stand + step).clamp(0.0, STAND_TICKS);
    }

    /// `PolarBear.isStanding`.
    pub fn is_standing(&self) -> bool {
        self.standing
    }

    /// `PolarBear.getStandingAnimationScale`.
    pub fn standing_animation_scale(&self, partial: f32) -> f32 {
        lerp_f32(partial, self.stand_o, self.stand) / STAND_TICKS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bear_stands_up_in_six_ticks_and_sits_back_down_as_fast() {
        let mut bear = PolarBearState::default();
        bear.set_standing(true);
        for _ in 0..10 {
            bear.tick();
        }
        assert_eq!(bear.standing_animation_scale(1.0), 1.0);
        assert_eq!(bear.standing_animation_scale(0.0), 1.0);
        bear.set_standing(false);
        bear.tick();
        assert_eq!(bear.standing_animation_scale(0.0), 1.0);
        assert!((bear.standing_animation_scale(1.0) - 5.0 / 6.0).abs() < 1e-6);
    }
}
