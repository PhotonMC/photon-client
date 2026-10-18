//! `EndFlashState`: the flashes of light in the End's sky, one try every six hundred ticks of the
//! dimension's clock, each from a seed of that interval.

use photon_core::math::{mth_lerp_f32, mth_sin_f64};
use photon_core::random::RandomSource;

/// How many ticks of the clock make one interval, with at most one flash in it.
const FLASH_INTERVAL: i64 = 600;
const MAX_FLASH_OFFSET: i32 = 200;
const MIN_FLASH_DURATION: i32 = 100;
const MAX_FLASH_DURATION: i32 = 380;

/// The flash of the clock's current interval and how bright the sky is with it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EndFlashState {
    flash_seed: i64,
    offset: i32,
    duration: i32,
    intensity: f32,
    old_intensity: f32,
    x_angle: f32,
    y_angle: f32,
}

impl EndFlashState {
    /// `tick`: the clock's time, once a client tick.
    pub fn tick(&mut self, clock_time: i64) {
        self.calculate_flash_parameters(clock_time);
        self.old_intensity = self.intensity;
        self.intensity = self.calculate_intensity(clock_time);
    }

    fn calculate_flash_parameters(&mut self, clock_time: i64) {
        let new_seed = clock_time / FLASH_INTERVAL;
        if new_seed == self.flash_seed {
            return;
        }
        let mut random = RandomSource::create(new_seed);
        random.next_float();
        // `Mth.randomBetweenInclusive` and `Mth.randomBetween`.
        let between_inclusive = |random: &mut RandomSource, min: i32, max: i32| {
            random.next_int_bound(max - min + 1).unwrap_or(0) + min
        };
        let between =
            |random: &mut RandomSource, min: f32, max: f32| random.next_float() * (max - min) + min;
        self.offset = between_inclusive(&mut random, 0, MAX_FLASH_OFFSET);
        self.duration = between_inclusive(
            &mut random,
            MIN_FLASH_DURATION,
            MAX_FLASH_DURATION.min(FLASH_INTERVAL as i32 - self.offset),
        );
        self.x_angle = between(&mut random, -60.0, 10.0);
        self.y_angle = between(&mut random, -180.0, 180.0);
        self.flash_seed = new_seed;
    }

    fn calculate_intensity(&self, clock_time: i64) -> f32 {
        let within = clock_time % FLASH_INTERVAL;
        if within >= i64::from(self.offset) && within <= i64::from(self.offset + self.duration) {
            let angle = (within - i64::from(self.offset)) as f32 * std::f32::consts::PI
                / self.duration as f32;
            mth_sin_f64(f64::from(angle))
        } else {
            0.0
        }
    }

    pub fn x_angle(&self) -> f32 {
        self.x_angle
    }

    pub fn y_angle(&self) -> f32 {
        self.y_angle
    }

    /// `getIntensity`: between the last tick's and this one's.
    pub fn intensity(&self, partial_tick: f32) -> f32 {
        mth_lerp_f32(partial_tick, self.old_intensity, self.intensity)
    }

    /// `flashStartedThisTick`.
    pub fn flash_started_this_tick(&self) -> bool {
        self.intensity > 0.0 && self.old_intensity <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fifth interval's flash lights the sky from tick 77 to 273, rising to a peak and
    /// falling, starts once and is gone for the rest of the interval.
    #[test]
    fn a_flash_rises_peaks_and_ends_within_its_interval() {
        let mut state = EndFlashState::default();
        let (mut lit, mut starts, mut peak) = (Vec::new(), 0, 0.0_f32);
        for t in 3000..3600 {
            state.tick(t);
            if state.intensity(1.0) > 0.0 {
                lit.push(t - 3000);
            }
            starts += i32::from(state.flash_started_this_tick());
            peak = peak.max(state.intensity(1.0));
        }
        assert_eq!((lit.first(), lit.last()), (Some(&77), Some(&273)));
        assert_eq!(starts, 1);
        assert!(peak > 0.99);
        state.tick(3601);
        assert_eq!(state.intensity(1.0), 0.0);
    }
}
