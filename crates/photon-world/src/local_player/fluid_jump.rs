use super::*;

/// What the held jump key does in a fluid.
enum FluidJump {
    /// Swim up a little.
    Rise,
    /// Jump off the ground or the surface.
    Ground,
    Nothing,
}

impl LocalPlayer {
    /// The jump key held: a jump from the ground, or the rise in a fluid (`LivingEntity.aiStep`).
    pub(super) fn jump_or_rise_in_fluid(&mut self, level: &Level<'_>) {
        if !(self.jumping && self.is_affected_by_fluids()) {
            self.no_jump_delay = 0;
            return;
        }
        match self.fluid_jump() {
            FluidJump::Rise => {
                self.delta = self.delta.add_components(0.0, f64::from(0.04_f32), 0.0);
            }
            FluidJump::Ground if self.no_jump_delay == 0 => {
                self.jump_from_ground(level);
                self.no_jump_delay = 10;
            }
            FluidJump::Ground | FluidJump::Nothing => {}
        }
    }

    /// What the held jump key does against the fluid the player stands in.
    fn fluid_jump(&self) -> FluidJump {
        let fluid_height = self.jump_fluid_height();
        let threshold = self.fluid_jump_threshold();
        let wading = self.is_in_water() && fluid_height > 0.0;
        if !(self.may_leave_water(wading, fluid_height, threshold)
            && self.may_leave_lava(threshold))
        {
            FluidJump::Rise
        } else if self.on_ground || wading && fluid_height <= threshold {
            FluidJump::Ground
        } else {
            FluidJump::Nothing
        }
    }

    fn jump_fluid_height(&self) -> f64 {
        if self.is_in_lava() {
            self.lava.height
        } else {
            self.water.height
        }
    }

    fn may_leave_water(&self, wading: bool, fluid_height: f64, threshold: f64) -> bool {
        !wading || self.on_ground && !(fluid_height > threshold)
    }

    fn may_leave_lava(&self, threshold: f64) -> bool {
        !self.is_in_lava() || self.on_ground && self.lava.height <= threshold
    }
}
