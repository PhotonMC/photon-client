//! What a shulker does of its own on the client (`Shulker.tick`): its lid
//! opens and closes by a twentieth a tick towards the peek the server says.

/// `Shulker.currentPeekAmount` and the one a tick before.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShulkerState {
    peek: f32,
    old_peek: f32,
}

/// The peek is finite, so the comparison is a total one.
impl Eq for ShulkerState {}

impl ShulkerState {
    /// `Shulker.updatePeekAmount`: the peek moves a twentieth towards the raw
    /// amount the data holds (hundredths, a byte).
    pub(crate) fn tick(&mut self, raw_peek: i8) {
        self.old_peek = self.peek;
        let target = f32::from(raw_peek) * 0.01;
        if self.peek == target {
            return;
        }
        // `Mth.clamp(float, min, max)`: below the minimum, else above the maximum.
        let clamp = |value: f32, min: f32, max: f32| {
            if value < min {
                min
            } else if value > max {
                max
            } else {
                value
            }
        };
        self.peek = if self.peek > target {
            clamp(self.peek - 0.05, target, 1.0)
        } else {
            clamp(self.peek + 0.05, 0.0, target)
        };
    }

    /// `Shulker.getClientPeekAmount`.
    pub fn peek(&self, partial_tick: f32) -> f32 {
        voxel_core::math::mth_lerp_f32(partial_tick, self.old_peek, self.peek)
    }

    pub fn current_peek(&self) -> f32 {
        self.peek
    }

    pub fn old_peek(&self) -> f32 {
        self.old_peek
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lid_opens_a_twentieth_a_tick_and_stops_at_the_peek() {
        let mut state = ShulkerState::default();
        for _ in 0..4 {
            state.tick(30);
        }
        assert!(state.current_peek() > 0.19 && state.current_peek() < 0.21);
        for _ in 0..20 {
            state.tick(30);
        }
        let target = f32::from(30_i8) * 0.01;
        assert_eq!(state.current_peek(), target);
        assert_eq!(state.old_peek(), target);
    }

    #[test]
    fn the_lid_closes_to_the_peek_without_passing_it() {
        let mut state = ShulkerState::default();
        for _ in 0..30 {
            state.tick(100);
        }
        for _ in 0..30 {
            state.tick(0);
        }
        assert_eq!(state.current_peek(), 0.0);
    }
}
