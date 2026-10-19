use super::*;
use photon_core::smooth::SmoothDouble;

/// What `MouseHandler` keeps of the mouse between frames: the movement waiting to turn the
/// player, the two smoothers of the cinematic camera and when the last frame took its share.
#[derive(Default)]
pub struct Turning {
    pending: (f64, f64),
    smooth_x: SmoothDouble,
    smooth_y: SmoothDouble,
    last: Option<Instant>,
    /// `Options.smoothCamera`, toggled by `key.smoothCamera` and never saved.
    pub smooth_camera: bool,
}

impl Turning {
    /// `MouseHandler.accumulatedDX` and `accumulatedDY`: the cursor moved.
    pub fn moved(&mut self, (dx, dy): (f64, f64)) {
        self.pending.0 += dx;
        self.pending.1 += dy;
    }

    /// `MouseHandler.handleAccumulatedMovement` for the frame at `now`: the turn to apply.
    /// The cinematic camera smooths it over the frame's length in seconds; a spyglass turns
    /// slowly (the sensitivity cubed, without the factor of eight); else it is the plain turn.
    pub fn take(&mut self, options: &Options, scoping: bool, now: Instant) -> (f64, f64) {
        let seconds = self
            .last
            .replace(now)
            .map_or(0.0, |last| (now - last).as_secs_f64());
        let (dx, dy) = std::mem::take(&mut self.pending);
        let sensitivity = options.sensitivity * f64::from(0.6_f32) + f64::from(0.2_f32);
        let cubed = sensitivity * sensitivity * sensitivity;
        let (turn_x, turn_y) = if self.smooth_camera {
            let factor = cubed * 8.0;
            (
                self.smooth_x
                    .get_new_delta_value(dx * factor, seconds * factor),
                self.smooth_y
                    .get_new_delta_value(dy * factor, seconds * factor),
            )
        } else {
            self.smooth_x.reset();
            self.smooth_y.reset();
            let factor = if scoping { cubed } else { cubed * 8.0 };
            (dx * factor, dy * factor)
        };
        (
            if options.invert_mouse_x {
                -turn_x
            } else {
                turn_x
            },
            if options.invert_mouse_y {
                -turn_y
            } else {
                turn_y
            },
        )
    }
}

impl Controls {
    /// The frame's share of the mouse's movement turns the player.
    pub fn turn_frame(&mut self, session: &mut PlaySession, now: Instant) {
        let scoping = self.camera_type == CameraType::FirstPerson
            && session.player().is_some_and(|player| player.is_scoping());
        let (turn_x, turn_y) = self.turning.take(&self.options, scoping, now);
        if self.captured && (turn_x != 0.0 || turn_y != 0.0) {
            session.turn_player(turn_x, turn_y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn the_camera_turns_by_the_sensitivity_cubed_and_the_inversions() {
        let mut options = Options::default();
        let now = Instant::now();
        let mut turning = Turning::default();
        // At the least, (0.2)^3 * 8; at the most, (0.8)^3 * 8, in single precision where the
        // reference widens a float.
        options.sensitivity = 0.0;
        let low = f64::from(0.2_f32).powi(3) * 8.0;
        turning.moved((10.0, -5.0));
        assert_eq!(turning.take(&options, false, now), (10.0 * low, -5.0 * low));
        options.sensitivity = 1.0;
        let high = (f64::from(0.6_f32) + f64::from(0.2_f32)).powi(3) * 8.0;
        turning.moved((1.0, 1.0));
        assert_eq!(turning.take(&options, false, now), (high, high));
        options.invert_mouse_x = true;
        turning.moved((1.0, 1.0));
        assert_eq!(turning.take(&options, false, now), (-high, high));
        options.invert_mouse_y = true;
        turning.moved((1.0, 1.0));
        assert_eq!(turning.take(&options, false, now), (-high, -high));
    }

    #[test]
    fn a_spyglass_turns_without_the_factor_of_eight() {
        let options = Options::default();
        let now = Instant::now();
        let mut turning = Turning::default();
        turning.moved((8.0, 0.0));
        let (scoped, _) = turning.take(&options, true, now);
        turning.moved((1.0, 0.0));
        let (plain, _) = turning.take(&options, false, now);
        assert!((scoped - plain).abs() < 1e-12);
    }

    #[test]
    fn the_cinematic_camera_eases_a_movement_over_the_frames() {
        let options = Options::default();
        let start = Instant::now();
        let mut turning = Turning {
            smooth_camera: true,
            ..Turning::default()
        };
        turning.take(&options, false, start);
        turning.moved((100.0, 0.0));
        let (first, _) = turning.take(&options, false, start + Duration::from_millis(16));
        let (second, _) = turning.take(&options, false, start + Duration::from_millis(32));
        let plain = 100.0 * 8.0 * f64::from(0.6_f32 * 0.5 + 0.2).powi(3);
        assert!(first > 0.0 && first < plain);
        assert!(second > 0.0, "the turn goes on after the mouse stopped");
        turning.smooth_camera = false;
        assert_eq!(
            turning.take(&options, false, start + Duration::from_millis(48)),
            (0.0, 0.0)
        );
    }
}
