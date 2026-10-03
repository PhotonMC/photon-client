//! What a wolf does of its own on the client (`Wolf.tick`): its head tilts
//! while it is interested, and it shakes itself dry when the server says so.
//!
//! Whether the wolf is in water or rain (`isInWaterOrRain`), which makes it
//! wet and darkens it until it has shaken, is not tracked: a wolf is never
//! wet here.

use voxel_core::math::mth_lerp_f32 as lerp_f32;

/// `Wolf.handleEntityEvent`: the wolf starts shaking itself, and stops.
pub const SHAKE_EVENT: i8 = 8;
pub const CANCEL_SHAKE_EVENT: i8 = 56;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WolfState {
    interested_angle: f32,
    interested_angle_o: f32,
    shake_anim: f32,
    shake_anim_o: f32,
    is_shaking: bool,
}

// The angles are finite, so equality is total.
impl Eq for WolfState {}

impl WolfState {
    /// One client tick (`Wolf.tick` after `super.tick`).
    pub(crate) fn tick(&mut self, alive: bool, interested: bool) {
        if !alive {
            return;
        }
        self.interested_angle_o = self.interested_angle;
        let target = if interested { 1.0 } else { 0.0 };
        self.interested_angle += (target - self.interested_angle) * 0.4;
        if !self.is_shaking {
            return;
        }
        self.shake_anim_o = self.shake_anim;
        self.shake_anim += 0.05;
        if self.shake_anim_o >= 2.0 {
            self.cancel_shake();
        }
    }

    /// `Wolf.handleEntityEvent(8)`.
    pub(crate) fn start_shake(&mut self) {
        self.is_shaking = true;
        self.shake_anim = 0.0;
        self.shake_anim_o = 0.0;
    }

    /// `Wolf.cancelShake`, and what `die` resets.
    pub(crate) fn cancel_shake(&mut self) {
        self.is_shaking = false;
        self.shake_anim = 0.0;
        self.shake_anim_o = 0.0;
    }

    /// `Wolf.getHeadRollAngle`.
    pub fn head_roll_angle(&self, partial: f32) -> f32 {
        lerp_f32(partial, self.interested_angle_o, self.interested_angle)
            * 0.15
            * std::f64::consts::PI as f32
    }

    /// `Wolf.getShakeAnim`.
    pub fn shake_anim(&self, partial: f32) -> f32 {
        lerp_f32(partial, self.shake_anim_o, self.shake_anim)
    }

    pub fn is_shaking(&self) -> bool {
        self.is_shaking
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shake_runs_about_forty_ticks_and_then_stops() {
        let mut wolf = WolfState::default();
        wolf.start_shake();
        let mut ticks = 0;
        while wolf.is_shaking() {
            wolf.tick(true, false);
            ticks += 1;
            assert!(ticks < 100, "the shake never ended");
        }
        // 0.05 a tick until the previous value reaches two (the sum is not exact).
        assert!((41..=42).contains(&ticks), "{ticks}");
        assert_eq!(wolf.shake_anim(0.5), 0.0);
    }

    #[test]
    fn interest_tilts_the_head_and_a_dead_wolf_keeps_its_pose() {
        let mut wolf = WolfState::default();
        wolf.tick(true, true);
        assert!(wolf.head_roll_angle(1.0) > 0.0);
        let tilted = wolf.head_roll_angle(1.0);
        wolf.tick(false, false);
        assert_eq!(wolf.head_roll_angle(1.0), tilted);
    }
}
