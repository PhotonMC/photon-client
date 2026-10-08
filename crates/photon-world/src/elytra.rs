//! `ElytraAnimationState`: how far an entity's wings are spread, which follows what it is doing
//! (gliding, sneaking, standing) a third of the way each tick.

use photon_core::math::mth_lerp_f32;

const DEFAULT_X_ROT: f32 = (std::f64::consts::PI / 12.0) as f32;
const DEFAULT_Z_ROT: f32 = (-std::f64::consts::PI / 12.0) as f32;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ElytraState {
    rot: [f32; 3],
    old: [f32; 3],
}

// The rotations are never NaN.
impl Eq for ElytraState {}

/// What the tick looks at of the entity.
#[derive(Debug, Clone, Copy)]
pub struct ElytraInput {
    pub fall_flying: bool,
    /// `getDeltaMovement`.
    pub motion: (f64, f64, f64),
    pub crouching: bool,
}

impl ElytraState {
    /// `ElytraAnimationState.tick`.
    pub fn tick(&mut self, input: ElytraInput) {
        self.old = self.rot;
        let pi = std::f64::consts::PI;
        let (target_x, target_z, target_y);
        if input.fall_flying {
            let mut ratio = 1.0_f32;
            let (x, y, z) = input.motion;
            if y < 0.0 {
                // `Vec3.normalize`.
                let length = (x * x + y * y + z * z).sqrt();
                let vertical = if length < 1.0e-5 { 0.0 } else { y / length };
                ratio = 1.0 - (-vertical).powf(1.5) as f32;
            }
            target_x = mth_lerp_f32(ratio, DEFAULT_X_ROT, (pi / 9.0) as f32);
            target_z = mth_lerp_f32(ratio, DEFAULT_Z_ROT, (-pi / 2.0) as f32);
            target_y = 0.0;
        } else if input.crouching {
            target_x = (pi * 2.0 / 9.0) as f32;
            target_z = (-pi / 4.0) as f32;
            target_y = 0.087_266_46;
        } else {
            target_x = DEFAULT_X_ROT;
            target_z = DEFAULT_Z_ROT;
            target_y = 0.0;
        }
        for (rot, target) in self.rot.iter_mut().zip([target_x, target_y, target_z]) {
            *rot += (target - *rot) * 0.3;
        }
    }

    /// `getRotX`, `getRotY` and `getRotZ` between two ticks.
    pub fn rotations(&self, partial_tick: f32) -> [f32; 3] {
        std::array::from_fn(|axis| mth_lerp_f32(partial_tick, self.old[axis], self.rot[axis]))
    }
}
