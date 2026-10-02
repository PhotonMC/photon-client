//! What a sheep, a chicken and a creeper keep for their drawing: the sheep's
//! wool byte and its eating timer, the chicken's flap, the creeper's swell.

use voxel_core::math::mth_sin_f64;

/// `Sheep.eatAnimationTick` when the server tells a sheep to eat
/// (`handleEntityEvent` 10).
pub(crate) const SHEEP_EAT_TICKS: i32 = 40;

/// The entity event that starts a sheep eating.
pub(crate) const SHEEP_EAT_EVENT: i8 = 10;

/// A sheep's and a chicken's own state, once the server has told it or its
/// tick has moved it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AnimalState {
    /// `Sheep.DATA_WOOL_ID`: the dye in the low four bits, sheared in bit four.
    pub(crate) wool: u8,
    pub(crate) eat_tick: i32,
    flap_bits: u32,
    old_flap_bits: u32,
    flap_speed_bits: u32,
    old_flap_speed_bits: u32,
    flapping_bits: u32,
    /// `Creeper`: how far its fuse has burned and the last tick's, the
    /// direction it burns (`DATA_SWELL_DIR`) and whether it was ignited.
    swell: i32,
    old_swell: i32,
    swell_dir: i32,
    ignited: bool,
}

/// How many ticks a creeper's fuse is (`Creeper.maxSwell`).
const CREEPER_MAX_SWELL: i32 = 30;

/// `Sheep.getHeadEatPositionScale`: how far the head is down, with the eating
/// timer at `tick`.
pub fn head_eat_position_scale(tick: i32, partial: f32) -> f32 {
    if tick <= 0 {
        0.0
    } else if (4..=36).contains(&tick) {
        1.0
    } else if tick < 4 {
        (tick as f32 - partial) / 4.0
    } else {
        -((tick - 40) as f32 - partial) / 4.0
    }
}

/// `Sheep.getHeadEatAngleScale`: how the head is tipped, which is the pitch
/// (in degrees, as drawn) while the sheep is not eating.
pub fn head_eat_angle_scale(tick: i32, partial: f32, pitch: f32) -> f32 {
    if tick > 4 && tick <= 36 {
        let scale = ((tick - 4) as f32 - partial) / 32.0;
        (std::f64::consts::PI / 5.0) as f32 + 0.219_911_49 * mth_sin_f64(f64::from(scale * 28.7))
    } else if tick > 0 {
        (std::f64::consts::PI / 5.0) as f32
    } else {
        pitch * (std::f64::consts::PI / 180.0) as f32
    }
}

impl Default for AnimalState {
    /// `Chicken.flapping` starts at one.
    fn default() -> Self {
        Self {
            wool: 0,
            eat_tick: 0,
            flap_bits: 0,
            old_flap_bits: 0,
            flap_speed_bits: 0,
            old_flap_speed_bits: 0,
            flapping_bits: 1.0_f32.to_bits(),
            swell: 0,
            old_swell: 0,
            swell_dir: 0,
            ignited: false,
        }
    }
}

impl AnimalState {
    /// `Sheep.aiStep` on the client: the eating timer runs down.
    pub(crate) fn tick_sheep(&mut self) {
        self.eat_tick = (self.eat_tick - 1).max(0);
    }

    /// `Chicken.aiStep`'s flap: the wings beat faster while it is off the
    /// ground and settle back on it.
    pub(crate) fn tick_chicken(&mut self, on_ground: bool) {
        let (flap, flap_speed) = (
            f32::from_bits(self.flap_bits),
            f32::from_bits(self.flap_speed_bits),
        );
        let mut flapping = f32::from_bits(self.flapping_bits);
        self.old_flap_bits = self.flap_bits;
        self.old_flap_speed_bits = self.flap_speed_bits;
        let flap_speed =
            (flap_speed + (if on_ground { -1.0_f32 } else { 4.0 }) * 0.3).clamp(0.0, 1.0);
        if !on_ground && flapping < 1.0 {
            flapping = 1.0;
        }
        flapping *= 0.9;
        self.flap_bits = (flap + flapping * 2.0).to_bits();
        self.flap_speed_bits = flap_speed.to_bits();
        self.flapping_bits = flapping.to_bits();
    }

    pub(crate) fn set_swell_dir(&mut self, direction: i32) {
        self.swell_dir = direction;
    }

    pub(crate) fn set_ignited(&mut self, ignited: bool) {
        self.ignited = ignited;
    }

    /// `Creeper.tick` while it lives: the fuse burns in its direction (an
    /// ignited creeper sets it burning), no further than it goes.
    pub(crate) fn tick_creeper(&mut self, alive: bool) {
        if !alive {
            return;
        }
        self.old_swell = self.swell;
        if self.ignited {
            self.swell_dir = 1;
        }
        self.swell = (self.swell + self.swell_dir).clamp(0, CREEPER_MAX_SWELL);
    }

    /// `Creeper.getSwelling`: how far the fuse has burned, between two ticks.
    pub(crate) fn creeper_swelling(&self, partial: f32) -> f32 {
        let old = self.old_swell as f32;
        (old + partial * (self.swell as f32 - old)) / (CREEPER_MAX_SWELL - 2) as f32
    }

    /// `Mth.lerp` of the flap between two ticks.
    pub(crate) fn flap_at(&self, partial: f32) -> f32 {
        let (old, new) = (
            f32::from_bits(self.old_flap_bits),
            f32::from_bits(self.flap_bits),
        );
        old + partial * (new - old)
    }

    pub(crate) fn flap_speed_at(&self, partial: f32) -> f32 {
        let old = f32::from_bits(self.old_flap_speed_bits);
        let new = f32::from_bits(self.flap_speed_bits);
        old + partial * (new - old)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sheep_eats_for_forty_ticks_and_its_head_comes_up_at_the_end() {
        let mut sheep = AnimalState {
            eat_tick: SHEEP_EAT_TICKS,
            ..Default::default()
        };
        assert_eq!(head_eat_position_scale(sheep.eat_tick, 0.0), 0.0);
        let mut down = 0;
        for _ in 0..SHEEP_EAT_TICKS {
            sheep.tick_sheep();
            if head_eat_position_scale(sheep.eat_tick, 0.0) == 1.0 {
                down += 1;
            }
        }
        assert_eq!(sheep.eat_tick, 0);
        assert_eq!(down, 33);
        assert_eq!(
            head_eat_angle_scale(sheep.eat_tick, 0.5, 30.0),
            30.0 * (std::f64::consts::PI / 180.0) as f32
        );
    }
}
