use super::*;

impl MountBody {
    /// `LivingEntity.travelRidden`.
    pub(super) fn travel_ridden(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let ridden_input = self.ridden_input();
        self.tick_ridden(level, ridden_input);
        if self.kind == MountKind::Camel {
            // `Camel.getRiddenSpeed`, after the dash may have started: faster for a rider that
            // sprints while it can dash.
            let cooldown = self.camel.dash_cooldown.unwrap_or(self.camel.cooldown);
            let bonus = if self.controller.sprinting && cooldown == 0 {
                0.1_f32
            } else {
                0.0
            };
            self.attributes.speed = self.attributes.base_speed as f32 + bonus;
        }
        self.travel(level, ridden_input)
    }

    /// `AbstractHorse.getRiddenInput`: still while it rears, otherwise the rider's keys, sideways
    /// at half and backwards at a quarter.
    pub(super) fn ridden_input(&self) -> Vec3 {
        // `Pig.getRiddenInput`: always straight ahead; the stick's rider only steers.
        if matches!(self.kind, MountKind::Pig | MountKind::Strider) {
            return Vec3::new(0.0, 0.0, 1.0);
        }
        if self.kind == MountKind::Camel && self.camel.refuses_to_move {
            return Vec3::ZERO;
        }
        if self.on_ground
            && self.state.jump_pending_scale == 0.0
            && self.standing
            && !self.allow_stand_sliding
        {
            return Vec3::ZERO;
        }
        let sideways = self.controller.xxa * 0.5;
        let mut forward = self.controller.zza;
        if forward <= 0.0 {
            forward *= 0.25;
        }
        Vec3::new(f64::from(sideways), 0.0, f64::from(forward))
    }

    /// `AbstractHorse.tickRidden`: it faces where the rider does (looking up only half as far),
    /// and, on the ground, carries out the jump the rider asked for.
    pub(super) fn tick_ridden(&mut self, level: &Level<'_>, ridden_input: Vec3) {
        // `Camel.getRiddenRotation`: a camel that sits, or sits down or rises, keeps its look.
        if !(self.kind == MountKind::Camel && self.camel.refuses_to_move) {
            self.x_rot = (self.controller.x_rot * 0.5) % 360.0;
            self.y_rot = self.controller.y_rot % 360.0;
        }
        // A pig has no jump to carry out (`Pig.tickRidden`).
        if matches!(self.kind, MountKind::Horse | MountKind::Camel) && self.on_ground {
            if self.state.jump_pending_scale > 0.0 {
                self.execute_riders_jump(level, self.state.jump_pending_scale, ridden_input);
            }
            self.state.jump_pending_scale = 0.0;
        }
        // `Camel.tickRidden`: pushing forward on a camel that sits makes it rise.
        if self.kind == MountKind::Camel
            && self.controller.zza > 0.0
            && self.camel.sitting
            && !self.camel.in_pose_transition
        {
            self.camel.stand_up = true;
        }
    }

    /// `AbstractHorse.executeRidersJump`.
    pub(super) fn execute_riders_jump(&mut self, level: &Level<'_>, amount: f32, input: Vec3) {
        if self.kind == MountKind::Camel {
            return self.execute_camels_dash(level, amount);
        }
        let impulse = f64::from(self.jump_power(level, amount));
        self.delta = Vec3::new(self.delta.x, impulse, self.delta.z);
        if input.z > 0.0 {
            let sin = mth_sin_f64(f64::from(self.y_rot * DEG_TO_RAD));
            let cos = mth_cos_f64(f64::from(self.y_rot * DEG_TO_RAD));
            self.delta = self.delta.add_vec3(Vec3::new(
                f64::from(-0.4_f32 * sin * amount),
                0.0,
                f64::from(0.4_f32 * cos * amount),
            ));
        }
    }

    /// `LivingEntity.getJumpPower(multiplier)`: the jump strength, the block's own jump factor
    /// (no jump boost on a mount here).
    pub(super) fn jump_power(&self, level: &Level<'_>, multiplier: f32) -> f32 {
        self.attributes.jump_strength * multiplier * self.block_jump_factor(level) + 0.0
    }
}
