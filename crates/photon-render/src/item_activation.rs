//! `ScreenEffectRenderer`'s item activation: the totem that spins up to the screen, hangs there
//! and falls away over forty ticks.

use crate::joml::{Pose, Quaternion};
use photon_core::math::{mth_cos_f64, mth_sin_f64};
use photon_world::ClientItemStackState;

/// `ScreenEffectRenderer.itemActivationTicks` at the start.
const DURATION: i32 = 40;

/// The stack on show, how long it has left and where it drifts to.
#[derive(Debug, Clone, Default)]
pub struct ItemActivation {
    stack: Option<ClientItemStackState>,
    ticks: i32,
    off_x: f32,
    off_y: f32,
}

impl ItemActivation {
    /// `displayItemActivation`: the offsets are the renderer's own two random floats.
    pub fn display(&mut self, stack: ClientItemStackState, (off_x, off_y): (f32, f32)) {
        self.stack = Some(stack);
        self.ticks = DURATION;
        self.off_x = off_x * 2.0 - 1.0;
        self.off_y = off_y * 2.0 - 1.0;
    }

    /// `ScreenEffectRenderer.tick`.
    pub fn tick(&mut self) {
        if self.ticks > 0 {
            self.ticks -= 1;
            if self.ticks == 0 {
                self.stack = None;
            }
        }
    }

    /// `resetItemActivation`.
    pub fn reset(&mut self) {
        self.stack = None;
    }

    /// The stack and the pose it is drawn in, while it is on show
    /// (`renderItemActivationAnimation`).
    pub fn shown(&self, partial_tick: f32, aspect: f32) -> Option<(&ClientItemStackState, Pose)> {
        let stack = self.stack.as_ref().filter(|_| self.ticks > 0)?;
        Some((
            stack,
            activation_pose(self.ticks, partial_tick, aspect, (self.off_x, self.off_y)),
        ))
    }
}

/// The pose of the animation `ticks_left` ticks from its end.
pub fn activation_pose(
    ticks_left: i32,
    partial_tick: f32,
    aspect: f32,
    (off_x, off_y): (f32, f32),
) -> Pose {
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let rad = Quaternion::degrees_to_radians;
    let tick = DURATION - ticks_left;
    let scale = (tick as f32 + partial_tick) / 40.0;
    let ts = scale * scale;
    let tc = scale * ts;
    let smooth_scale = 10.25 * tc * ts - 24.95 * ts * ts + 25.5 * tc - 13.8 * ts + 4.0 * scale;
    let pi_scale = smooth_scale * std::f64::consts::PI as f32;
    let off_x = off_x * 0.3 * aspect;
    let off_y = off_y * 0.3;
    let mut pose = Pose::default();
    pose.translate(
        off_x * sin(pi_scale * 2.0).abs(),
        off_y * sin(pi_scale * 2.0).abs(),
        -10.0 + 9.0 * sin(pi_scale),
    );
    pose.scale(0.8, 0.8, 0.8);
    pose.rotate(&Quaternion::rotation_y(rad(900.0 * sin(pi_scale).abs())));
    pose.rotate(&Quaternion::rotation_x(rad(6.0 * cos(scale * 8.0))));
    pose.rotate(&Quaternion::rotation_z(rad(6.0 * cos(scale * 8.0))));
    pose
}
