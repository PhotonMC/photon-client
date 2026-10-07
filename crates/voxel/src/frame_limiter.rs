//! `FramerateLimiter` and `FramerateLimitTracker`: how long a frame lasts at the least, and what
//! lowers that rate: a minimised window, and a player who has not touched the keyboard or mouse.

use std::time::{Duration, Instant};

use voxel_game::options::{InactivityFpsLimit, UNLIMITED_FRAMERATE_CUTOFF};

/// The rate of a minimised window, and of one nobody has used for ten minutes.
const LOW_LIMIT: i32 = 10;
/// The most a window nobody has used for a minute draws.
const SHORT_AFK_LIMIT: i32 = 30;
const AFK_THRESHOLD: Duration = Duration::from_secs(60);
const LONG_AFK_THRESHOLD: Duration = Duration::from_secs(600);
/// `OVERSHOOT_SMOOTHING`, `MAX_CURRENT_OVERSHOOT_NS`, `MAX_AVERAGE_OVERSHOOT_NS` and
/// `SPIN_SAFETY_BUFFER_NS`.
const OVERSHOOT_SMOOTHING: f64 = 0.1;
const MAX_CURRENT_OVERSHOOT_NS: i64 = 25_000_000;
const MAX_AVERAGE_OVERSHOOT_NS: i64 = 2_000_000;
const SPIN_SAFETY_BUFFER_NS: i64 = 500_000;

/// `FramerateThrottleReason`. The reference's fifth reason, a menu out of a level, never comes up
/// here: the client only runs in a level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Throttle {
    None,
    Iconified,
    LongAfk,
    ShortAfk,
}

/// What the frame rate limit is now, and why.
pub struct FrameLimiter {
    last_frame: Instant,
    average_overshoot_ns: i64,
    last_limit: i32,
    latest_input: Instant,
}

impl FrameLimiter {
    pub fn new(now: Instant) -> Self {
        Self {
            last_frame: now,
            average_overshoot_ns: 0,
            last_limit: 0,
            latest_input: now,
        }
    }

    /// `onInputReceived`: a key or the mouse was used.
    pub fn input_received(&mut self, now: Instant) {
        self.latest_input = now;
    }

    /// `getThrottleReason`.
    pub fn throttle(
        &self,
        inactivity: InactivityFpsLimit,
        iconified: bool,
        now: Instant,
    ) -> Throttle {
        if iconified {
            return Throttle::Iconified;
        }
        if inactivity == InactivityFpsLimit::Afk {
            let idle = now.saturating_duration_since(self.latest_input);
            if idle > LONG_AFK_THRESHOLD {
                return Throttle::LongAfk;
            }
            if idle > AFK_THRESHOLD {
                return Throttle::ShortAfk;
            }
        }
        Throttle::None
    }

    /// `getFramerateLimit`: the player's limit, or the lower one a throttle gives.
    pub fn limit(
        &self,
        player_limit: i32,
        inactivity: InactivityFpsLimit,
        iconified: bool,
        now: Instant,
    ) -> i32 {
        match self.throttle(inactivity, iconified, now) {
            Throttle::None => player_limit,
            Throttle::Iconified | Throttle::LongAfk => LOW_LIMIT,
            Throttle::ShortAfk => player_limit.min(SHORT_AFK_LIMIT),
        }
    }

    /// `limitDisplayFPS`: wait until a frame at `limit` is over, sleeping for most of the time and
    /// spinning for the last of it, then start the next one. Nothing waits from the cutoff on.
    pub fn wait(&mut self, limit: i32) {
        if limit >= UNLIMITED_FRAMERATE_CUTOFF {
            self.last_frame = Instant::now();
            return;
        }
        let per_frame = Duration::from_nanos(1_000_000_000 / u64::from(limit.max(1) as u32));
        let target = self.last_frame + per_frame;
        if limit != self.last_limit {
            self.average_overshoot_ns = 0;
            self.last_limit = limit;
        }
        loop {
            let remaining = target.saturating_duration_since(Instant::now());
            let remaining_ns = remaining.as_nanos() as i64;
            if remaining_ns == 0 {
                break;
            }
            if remaining_ns > self.average_overshoot_ns + SPIN_SAFETY_BUFFER_NS {
                let started = Instant::now();
                let expected_ns = remaining_ns - self.average_overshoot_ns - SPIN_SAFETY_BUFFER_NS;
                std::thread::sleep(Duration::from_nanos(expected_ns as u64));
                let overshoot_ns = started.elapsed().as_nanos() as i64 - expected_ns;
                if overshoot_ns > 0 && overshoot_ns < MAX_CURRENT_OVERSHOOT_NS {
                    self.average_overshoot_ns = ((OVERSHOOT_SMOOTHING * overshoot_ns as f64
                        + (1.0 - OVERSHOOT_SMOOTHING) * self.average_overshoot_ns as f64)
                        as i64)
                        .min(MAX_AVERAGE_OVERSHOOT_NS);
                }
            } else {
                std::hint::spin_loop();
            }
        }
        self.last_frame = Instant::now();
    }
}

/// Whether a window event is the keyboard or the mouse being used (`MouseHandler` and
/// `KeyboardHandler` call `onInputReceived`).
pub fn is_input(event: &glfw::WindowEvent) -> bool {
    matches!(
        event,
        glfw::WindowEvent::Key(..)
            | glfw::WindowEvent::Char(_)
            | glfw::WindowEvent::MouseButton(..)
            | glfw::WindowEvent::CursorPos(..)
            | glfw::WindowEvent::Scroll(..)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn after(start: Instant, seconds: u64) -> Instant {
        start + Duration::from_secs(seconds)
    }

    #[test]
    fn a_used_window_draws_at_the_players_limit() {
        let start = Instant::now();
        let limiter = FrameLimiter::new(start);
        assert_eq!(
            limiter.limit(120, InactivityFpsLimit::Afk, false, after(start, 59)),
            120
        );
    }

    #[test]
    fn a_minute_without_input_lowers_the_limit_to_thirty_and_ten_minutes_to_ten() {
        let start = Instant::now();
        let limiter = FrameLimiter::new(start);
        assert_eq!(
            limiter.limit(120, InactivityFpsLimit::Afk, false, after(start, 61)),
            30
        );
        assert_eq!(
            limiter.limit(20, InactivityFpsLimit::Afk, false, after(start, 61)),
            20
        );
        assert_eq!(
            limiter.limit(120, InactivityFpsLimit::Afk, false, after(start, 601)),
            10
        );
    }

    #[test]
    fn input_starts_the_idle_time_again() {
        let start = Instant::now();
        let mut limiter = FrameLimiter::new(start);
        limiter.input_received(after(start, 100));
        assert_eq!(
            limiter.limit(120, InactivityFpsLimit::Afk, false, after(start, 130)),
            120
        );
    }

    #[test]
    fn idleness_matters_only_when_the_option_asks_for_it() {
        let start = Instant::now();
        let limiter = FrameLimiter::new(start);
        assert_eq!(
            limiter.limit(
                120,
                InactivityFpsLimit::Minimized,
                false,
                after(start, 5000)
            ),
            120
        );
    }

    #[test]
    fn a_minimised_window_draws_at_ten() {
        let start = Instant::now();
        let limiter = FrameLimiter::new(start);
        assert_eq!(
            limiter.limit(120, InactivityFpsLimit::Minimized, true, start),
            10
        );
    }

    #[test]
    fn waiting_lasts_at_least_a_frame_and_not_at_the_cutoff() {
        let mut limiter = FrameLimiter::new(Instant::now());
        let started = Instant::now();
        limiter.wait(50);
        limiter.wait(50);
        assert!(started.elapsed() >= Duration::from_millis(39));
        let started = Instant::now();
        limiter.wait(UNLIMITED_FRAMERATE_CUTOFF);
        assert!(started.elapsed() < Duration::from_millis(5));
    }
}
