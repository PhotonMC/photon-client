//! What a wither does of its own on the client: how long it has been
//! invulnerable (the data the server syncs), and where its two side heads turn
//! to (`WitherBoss.aiStep`): after the entity each looks at, or after the body.

use photon_core::math::{mth_atan2, mth_cos_f64, mth_sin_f64, wrap_degrees_f32};

use crate::mobs::guardian::GuardianTarget;

/// `WitherBoss`'s data ids: `Mob` ends at 15, then the three head targets
/// (the first the body's own, which the server uses) and the invulnerable ticks.
pub const TARGET_A_DATA_ID: u8 = 16;
pub const TARGET_B_DATA_ID: u8 = 17;
pub const TARGET_C_DATA_ID: u8 = 18;
pub const INVULNERABLE_DATA_ID: u8 = 19;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WitherState {
    /// `WitherBoss.getHeadXRots` and `getHeadYRots`: the side heads' turns, in degrees.
    x_rot_heads: [f32; 2],
    y_rot_heads: [f32; 2],
    /// `getAlternativeTarget(1)` and `(2)`: the ids of the entities the side heads look at (zero for none).
    pub(crate) targets: [i32; 2],
    pub(crate) invulnerable_ticks: i32,
}

/// Where the wither is: what the head turns are worked from.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WitherPlace {
    pub position: (f64, f64, f64),
    pub body_rot: f32,
    pub scale: f32,
}

impl WitherPlace {
    /// `WitherBoss.getHeadX/Y/Z`: head 0 is the middle one above the body, the others
    /// stand to either side of it, a body turn apart.
    fn head(self, index: i32) -> (f64, f64, f64) {
        let height = if index <= 0 { 3.0_f32 } else { 2.2_f32 };
        let y = self.position.1 + f64::from(height * self.scale);
        if index <= 0 {
            return (self.position.0, y, self.position.2);
        }
        let angle =
            (self.body_rot + (180 * (index - 1)) as f32) * (std::f64::consts::PI / 180.0) as f32;
        let cos = f64::from(mth_cos_f64(f64::from(angle)));
        let sin = f64::from(mth_sin_f64(f64::from(angle)));
        let scale = f64::from(self.scale);
        (
            self.position.0 + cos * 1.3 * scale,
            y,
            self.position.2 + sin * 1.3 * scale,
        )
    }
}

/// `WitherBoss.rotlerp`: a turn of at most `max` degrees from `a` toward `b`.
fn rotlerp(a: f32, b: f32, max: f32) -> f32 {
    let mut diff = wrap_degrees_f32(b - a);
    if diff > max {
        diff = max;
    }
    if diff < -max {
        diff = -max;
    }
    a + diff
}

// The turns are sums of finite steps: never NaN.
impl Eq for WitherState {}

impl WitherState {
    /// `WitherBoss.getInvulnerableTicks`.
    pub fn invulnerable_ticks(&self) -> i32 {
        self.invulnerable_ticks
    }

    /// `WitherBoss.getHeadXRots`.
    pub fn x_rot_heads(&self) -> [f32; 2] {
        self.x_rot_heads
    }

    /// `WitherBoss.getHeadYRots`.
    pub fn y_rot_heads(&self) -> [f32; 2] {
        self.y_rot_heads
    }

    /// The head turns of `WitherBoss.aiStep`, after the rest of it: a head with a
    /// target in the world turns toward its eyes (at most 40° up or down and 10°
    /// sideways a tick), one without turns after the body.
    pub(crate) fn tick_heads(&mut self, place: WitherPlace, targets: [Option<GuardianTarget>; 2]) {
        for (i, target) in targets.into_iter().enumerate() {
            if let Some(target) = target {
                let (hx, hy, hz) = place.head(i as i32 + 1);
                let xd = target.position.0 - hx;
                let yd = (target.position.1 + f64::from(target.eye_height)) - hy;
                let zd = target.position.2 - hz;
                let sd = (xd * xd + zd * zd).sqrt();
                // javac folds `180.0F / (float) Math.PI` into one float constant.
                let degrees = f64::from(180.0_f32 / std::f64::consts::PI as f32);
                let y_rot_d = (mth_atan2(zd, xd) * degrees) as f32 - 90.0;
                let x_rot_d = (-(mth_atan2(yd, sd) * degrees)) as f32;
                self.x_rot_heads[i] = rotlerp(self.x_rot_heads[i], x_rot_d, 40.0);
                self.y_rot_heads[i] = rotlerp(self.y_rot_heads[i], y_rot_d, 10.0);
            } else {
                self.y_rot_heads[i] = rotlerp(self.y_rot_heads[i], place.body_rot, 10.0);
            }
        }
    }
}
