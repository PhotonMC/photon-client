//! What a shulker does of its own on the client (`Shulker.tick`): its lid
//! opens and closes by a twentieth a tick towards the peek the server says,
//! and where it was before it moved to another block slides it across.

use voxel_core::aabb::Aabb;

/// `Shulker.currentPeekAmount` and the one a tick before, with the block it
/// slides from (`clientOldAttachPosition`) and the ticks the slide has left
/// (`clientSideTeleportInterpolation`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShulkerState {
    peek: f32,
    old_peek: f32,
    old_attach: Option<[i32; 3]>,
    teleport_steps: i32,
}

/// The ticks a shulker's slide to a new block takes.
const TELEPORT_STEPS: i32 = 6;

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

    /// The tail of `Shulker.tick` on the client: the slide runs down, and
    /// when it is over the block it came from is forgotten.
    pub(crate) fn tick_teleport(&mut self) {
        if self.teleport_steps > 0 {
            self.teleport_steps -= 1;
        } else {
            self.old_attach = None;
        }
    }

    /// `Shulker.setPos` (client): the shulker has moved to another block while
    /// not riding, unless it came from where it already slides from.
    pub(crate) fn start_teleport(&mut self, from: [i32; 3], to: [i32; 3]) -> bool {
        if self.old_attach == Some(to) {
            return false;
        }
        self.old_attach = Some(from);
        self.teleport_steps = TELEPORT_STEPS;
        true
    }

    /// `Shulker.startRiding` (client): no slide.
    pub(crate) fn cancel_teleport(&mut self) {
        self.old_attach = None;
        self.teleport_steps = 0;
    }

    /// `Shulker.stopRiding` (client): it will slide from where it stands.
    pub(crate) fn set_old_attach(&mut self, block: [i32; 3]) {
        self.old_attach = Some(block);
    }

    /// `Shulker.getRenderPosition`: how far the shulker is drawn from its
    /// block, back towards the one it came from, none when it is not sliding.
    pub fn render_offset(
        &self,
        block: [i32; 3],
        scale: f32,
        partial_tick: f32,
    ) -> Option<[f64; 3]> {
        let from = self.old_attach.filter(|_| self.teleport_steps > 0)?;
        let mut factor = (f64::from(self.teleport_steps) - f64::from(partial_tick)) / 6.0;
        factor *= factor;
        factor *= f64::from(scale);
        let offset = |axis: usize| f64::from(block[axis] - from[axis]) * factor;
        Some([-offset(0), -offset(1), -offset(2)])
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

/// `Shulker.getPhysicalPeek`.
pub fn physical_peek(amount: f32) -> f32 {
    0.5 - voxel_core::math::mth_sin_f64(f64::from((0.5 + amount) * std::f32::consts::PI)) * 0.5
}

/// `Shulker.getProgressDeltaAabb`: the box of the shulker's body and the lid
/// opened from one progress to another, towards `direction` (a data value, the
/// side the lid faces).
pub fn progress_delta_aabb(
    size: f32,
    direction: i32,
    (progress_from, progress_to): (f32, f32),
    position: (f64, f64, f64),
) -> Aabb {
    let size = f64::from(size);
    let step = match direction {
        0 => (0.0, -1.0, 0.0),
        1 => (0.0, 1.0, 0.0),
        2 => (0.0, 0.0, -1.0),
        3 => (0.0, 0.0, 1.0),
        4 => (-1.0, 0.0, 0.0),
        _ => (1.0, 0.0, 0.0),
    };
    let bounds = Aabb::new(-size * 0.5, 0.0, -size * 0.5, size * 0.5, size, size * 0.5);
    let max_movement = f64::from(progress_from.max(progress_to));
    let min_movement = f64::from(progress_from.min(progress_to));
    bounds
        .expand_towards(
            step.0 * max_movement * size,
            step.1 * max_movement * size,
            step.2 * max_movement * size,
        )
        .contract(
            -step.0 * (1.0 + min_movement) * size,
            -step.1 * (1.0 + min_movement) * size,
            -step.2 * (1.0 + min_movement) * size,
        )
        .move_by(position.0, position.1, position.2)
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
