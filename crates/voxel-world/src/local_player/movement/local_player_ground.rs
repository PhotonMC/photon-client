use super::*;

impl LocalPlayer {
    pub(in super::super) fn jump_from_ground(&mut self, level: &Level<'_>) {
        let jump_power = JUMP_STRENGTH * 1.0_f32 * self.block_jump_factor(level) + 0.0_f32;
        if jump_power <= 1.0e-5 {
            return;
        }
        self.delta = Vec3::new(
            self.delta.x,
            java_math::max(f64::from(jump_power), self.delta.y),
            self.delta.z,
        );
        if self.sprinting {
            let angle = self.y_rot * MTH_DEG_TO_RAD;
            self.delta = self.delta.add_vec3(Vec3::new(
                f64::from(-mth_sin_f64(f64::from(angle))) * 0.2,
                0.0,
                f64::from(mth_cos_f64(f64::from(angle))) * 0.2,
            ));
        }
    }
}
