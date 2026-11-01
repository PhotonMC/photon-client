//! The game's clock.

use std::time::Instant;

/// The game's clock: the last frame, when it began and the whole milliseconds (`Util.getMillis`)
/// at the last frame.
pub(super) struct GameClock {
    pub(super) last_frame: Instant,
    pub(super) epoch: Instant,
    pub(super) last_ms: i64,
}

impl GameClock {
    pub(super) fn new() -> Self {
        let now = Instant::now();
        Self {
            last_frame: now,
            epoch: now,
            last_ms: 0,
        }
    }
}
