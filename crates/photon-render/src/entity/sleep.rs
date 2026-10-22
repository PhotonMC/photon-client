//! How a living entity lies in a bed.

/// How a sleeper lies: the turn that puts its head on the pillow and the shift of its body away
/// from the head's end of the bed (`LivingEntityRenderer.submit` and `sleepDirectionToRotation`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sleep {
    /// The turn about the vertical, in degrees.
    pub angle: f32,
    /// The shift along x and z, in blocks.
    pub shift: [f32; 2],
}

impl Sleep {
    /// A sleeper in a bed whose head end faces `bed` (the step of its direction in x and z), or
    /// in none a block entity knows, which turns it as its body is turned.
    pub fn in_bed(bed: Option<(i32, i32)>, eye_height: f32, body_rot: f32) -> Self {
        let Some((x, z)) = bed else {
            return Self {
                angle: body_rot,
                shift: [0.0; 2],
            };
        };
        let angle = match (x, z) {
            (0, 1) => 90.0,
            (0, -1) => 270.0,
            (1, 0) => 180.0,
            _ => 0.0,
        };
        let head_offset = eye_height - 0.1;
        Self {
            angle,
            shift: [-(x as f32) * head_offset, -(z as f32) * head_offset],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_head_end_of_a_bed_gives_the_turn_and_the_shift() {
        // The turns of `sleepDirectionToRotation`; the shift is the standing eye height less a
        // tenth, away from the head end.
        for (step, angle, shift) in [
            ((0, 1), 90.0, [0.0, -1.52]),
            ((-1, 0), 0.0, [1.52, 0.0]),
            ((0, -1), 270.0, [0.0, 1.52]),
            ((1, 0), 180.0, [-1.52, 0.0]),
        ] {
            let sleep = Sleep::in_bed(Some(step), 1.62, 33.0);
            assert_eq!(sleep.angle, angle);
            assert!((sleep.shift[0] - shift[0]).abs() < 1e-6, "{step:?}");
            assert!((sleep.shift[1] - shift[1]).abs() < 1e-6, "{step:?}");
        }
    }

    #[test]
    fn a_sleeper_without_a_bed_turns_as_its_body_does() {
        let sleep = Sleep::in_bed(None, 1.62, 33.0);
        assert_eq!((sleep.angle, sleep.shift), (33.0, [0.0, 0.0]));
    }
}
