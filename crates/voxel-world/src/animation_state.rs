//! `AnimationState`: when an animation started, in the entity's ticks, and
//! from that how long it has run.

const STOPPED: i32 = i32::MIN;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationState {
    start_tick: i32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            start_tick: STOPPED,
        }
    }
}

impl AnimationState {
    pub fn start(&mut self, tick_count: i32) {
        self.start_tick = tick_count;
    }

    pub fn start_if_stopped(&mut self, tick_count: i32) {
        if !self.is_started() {
            self.start(tick_count);
        }
    }

    pub fn animate_when(&mut self, condition: bool, tick_count: i32) {
        if condition {
            self.start_if_stopped(tick_count);
        } else {
            self.stop();
        }
    }

    pub fn stop(&mut self) {
        self.start_tick = STOPPED;
    }

    pub fn fast_forward(&mut self, ticks: i32, time_scale: f32) {
        if self.is_started() {
            self.start_tick = self
                .start_tick
                .wrapping_sub((ticks as f32 * time_scale) as i32);
        }
    }

    /// `getTimeInMillis`: how long it has run at this age, in milliseconds.
    pub fn time_in_millis(&self, age_in_ticks: f32) -> i64 {
        ((age_in_ticks - self.start_tick as f32) * 50.0) as i64
    }

    pub fn is_started(&self) -> bool {
        self.start_tick != STOPPED
    }
}
