//! What a fox does of its own on the client (`Fox.tick`): its head rolls
//! while it is interested, and it crouches slowly. Its flags and variant are
//! synched data.

use photon_core::math::mth_lerp_f32 as lerp_f32;

/// `Fox.DATA_FLAGS_ID` bits.
const SITTING: u8 = 1;
const CROUCHING: u8 = 4;
const INTERESTED: u8 = 8;
const POUNCING: u8 = 16;
const SLEEPING: u8 = 32;
const FACEPLANTED: u8 = 64;

/// The most a fox crouches (`crouchAmount` stops at five).
const MAX_CROUCH: f32 = 5.0;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FoxState {
    flags: u8,
    variant: i32,
    interested_angle: f32,
    interested_angle_o: f32,
    crouch_amount: f32,
    crouch_amount_o: f32,
}

// The angles are finite, so equality is total.
impl Eq for FoxState {}

impl FoxState {
    /// Takes `Fox.DATA_FLAGS_ID` and `DATA_TYPE_ID`, whichever the data holds.
    pub(crate) fn set_data(&mut self, flags: Option<u8>, variant: Option<i32>) {
        if let Some(flags) = flags {
            self.flags = flags;
        }
        if let Some(variant) = variant {
            self.variant = variant;
        }
    }

    /// One client tick (`Fox.tick` after `super.tick`).
    pub(crate) fn tick(&mut self) {
        self.interested_angle_o = self.interested_angle;
        let target = if self.is_interested() { 1.0 } else { 0.0 };
        self.interested_angle += (target - self.interested_angle) * 0.4;
        self.crouch_amount_o = self.crouch_amount;
        if self.is_crouching() {
            self.crouch_amount = (self.crouch_amount + 0.2).min(MAX_CROUCH);
        } else {
            self.crouch_amount = 0.0;
        }
    }

    /// `Fox.getVariant`'s id: red (zero) for an id the variants lack.
    pub fn variant(&self) -> i32 {
        i32::from(self.variant == 1)
    }

    pub fn is_sitting(&self) -> bool {
        self.flags & SITTING != 0
    }

    /// `Fox.isCrouching`: the flag, not the pose.
    pub fn is_crouching(&self) -> bool {
        self.flags & CROUCHING != 0
    }

    pub fn is_interested(&self) -> bool {
        self.flags & INTERESTED != 0
    }

    pub fn is_pouncing(&self) -> bool {
        self.flags & POUNCING != 0
    }

    /// `Fox.isSleeping`: the flag, not a bed.
    pub fn is_sleeping(&self) -> bool {
        self.flags & SLEEPING != 0
    }

    pub fn is_faceplanted(&self) -> bool {
        self.flags & FACEPLANTED != 0
    }

    /// `Fox.getHeadRollAngle`.
    pub fn head_roll_angle(&self, partial: f32) -> f32 {
        lerp_f32(partial, self.interested_angle_o, self.interested_angle)
            * 0.11
            * std::f64::consts::PI as f32
    }

    /// `Fox.getCrouchAmount`.
    pub fn crouch_amount(&self, partial: f32) -> f32 {
        lerp_f32(partial, self.crouch_amount_o, self.crouch_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fox_crouches_in_a_slow_climb_and_straightens_at_once() {
        let mut fox = FoxState::default();
        fox.set_data(Some(CROUCHING), None);
        for _ in 0..30 {
            fox.tick();
        }
        assert_eq!(fox.crouch_amount(1.0), MAX_CROUCH);
        fox.set_data(Some(0), None);
        fox.tick();
        assert_eq!(fox.crouch_amount(1.0), 0.0);
        assert_eq!(fox.crouch_amount(0.0), MAX_CROUCH);
    }

    #[test]
    fn interest_rolls_the_head_and_an_unknown_variant_is_red() {
        let mut fox = FoxState::default();
        fox.set_data(Some(INTERESTED), Some(7));
        fox.tick();
        assert!(fox.head_roll_angle(1.0) > 0.0);
        assert_eq!(fox.variant(), 0);
        fox.set_data(None, Some(1));
        assert_eq!(fox.variant(), 1);
    }
}
