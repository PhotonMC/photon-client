use super::*;

impl MobModel {
    /// `CodModel.setupAnim` and `TropicalFish*Model.setupAnim`: the tail (named
    /// `tail_fin` on a cod, `tail` on a tropical fish) swings, wider out of water.
    pub fn setup_tail_fish(&mut self, tail: &str, age_in_ticks: f32, in_water: bool) {
        self.root.reset();
        let amplitude = if in_water { 1.0_f32 } else { 1.5_f32 };
        if let Some(tail) = self.root.child_mut(tail) {
            tail.y_rot = -amplitude * 0.45 * mth_sin_f64(f64::from(0.6 * age_in_ticks));
        }
    }

    /// `SalmonModel.setupAnim`: the back half of the body swings, harder and
    /// faster out of water.
    pub fn setup_salmon(&mut self, age_in_ticks: f32, in_water: bool) {
        self.root.reset();
        let (amplitude, angle) = if in_water {
            (1.0_f32, 1.0_f32)
        } else {
            (1.3, 1.7)
        };
        if let Some(back) = self.root.child_mut("body_back") {
            back.y_rot = -amplitude * 0.25 * mth_sin_f64(f64::from(angle * 0.6 * age_in_ticks));
        }
    }

    /// `Pufferfish*Model.setupAnim`: the two fins (named `right_fin` and
    /// `left_fin` on a small one, `right_blue_fin` and `left_blue_fin` on the
    /// others) beat against each other.
    pub fn setup_pufferfish(&mut self, fins: [&str; 2], age_in_ticks: f32) {
        self.root.reset();
        let beat = mth_sin_f64(f64::from(age_in_ticks * 0.2));
        if let Some(right) = self.root.child_mut(fins[0]) {
            right.z_rot = -0.2 + 0.4 * beat;
        }
        if let Some(left) = self.root.child_mut(fins[1]) {
            left.z_rot = 0.2 - 0.4 * beat;
        }
    }
}

/// What a fish renderer's `setupRotations` does besides the common turns: the
/// pufferfish's bob (before them), the swing about y after the fall (none for
/// the pufferfish), and the
/// shift a fish out of water makes before it lies on its side.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FishTurns {
    pub bob: f32,
    pub swing: Option<f32>,
    pub on_land: Option<[f32; 3]>,
}

/// `CodRenderer.setupRotations`: the swing about y, and out of water the shift
/// that lays it on its side.
pub fn cod_turns(age_in_ticks: f32, in_water: bool) -> FishTurns {
    FishTurns {
        bob: 0.0,
        swing: Some(4.3 * mth_sin_f64(f64::from(0.6 * age_in_ticks))),
        on_land: (!in_water).then_some([0.1, 0.1, -0.1]),
    }
}

/// `SalmonRenderer.setupRotations`: the swing is larger and quicker out of
/// water, where the fish lies on its side.
pub fn salmon_turns(age_in_ticks: f32, in_water: bool) -> FishTurns {
    let (amplitude, angle) = if in_water {
        (1.0_f32, 1.0_f32)
    } else {
        (1.3, 1.7)
    };
    FishTurns {
        bob: 0.0,
        swing: Some(amplitude * 4.3 * mth_sin_f64(f64::from(angle * 0.6 * age_in_ticks))),
        on_land: (!in_water).then_some([0.2, 0.1, 0.0]),
    }
}

/// `TropicalFishRenderer.setupRotations`.
pub fn tropical_fish_turns(age_in_ticks: f32, in_water: bool) -> FishTurns {
    FishTurns {
        bob: 0.0,
        swing: Some(4.3 * mth_sin_f64(f64::from(0.6 * age_in_ticks))),
        on_land: (!in_water).then_some([0.2, 0.1, 0.0]),
    }
}

/// `PufferfishRenderer.setupRotations`: it only bobs.
pub fn pufferfish_turns(age_in_ticks: f32) -> FishTurns {
    FishTurns {
        bob: mth_cos_f64(f64::from(age_in_ticks * 0.05)) * 0.08,
        swing: None,
        on_land: None,
    }
}
