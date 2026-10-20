//! What the first-person hand does with the stack in it (`ItemInHandRenderer.submitArmWithItem`):
//! the swing, and the poses of a drawn bow and a charging or loaded crossbow.

use crate::joml::{Pose, Quaternion};

/// What the held stack's hand is doing, which places its item after the arm transform.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandMotion {
    /// The default whack of the swing.
    Swing,
    /// A bow being drawn: how long it has been, in ticks and the frame's part of one.
    Bow { held: f32 },
    /// A crossbow being charged, and how long its charge takes.
    CrossbowCharge { held: f32, charge_duration: f32 },
    /// A loaded crossbow in the main hand: the swing, and the hold's offset when none is on.
    LoadedCrossbow,
}

fn sin(value: f32) -> f32 {
    photon_core::math::mth_sin_f64(f64::from(value))
}

fn rotate(pose: &mut Pose, axis: char, degrees: f32) {
    let rad = Quaternion::degrees_to_radians(degrees);
    pose.rotate(&match axis {
        'x' => Quaternion::rotation_x(rad),
        'y' => Quaternion::rotation_y(rad),
        _ => Quaternion::rotation_z(rad),
    });
}

/// `ItemInHandRenderer.swingArm`: the stack's travel in a swing and `applyItemArmAttackTransform`.
fn swing_arm(pose: &mut Pose, invert: f32, attack: f32) {
    let pi = std::f64::consts::PI as f32;
    let sqrt_attack = f64::from(attack).sqrt() as f32;
    pose.translate(
        invert * (-0.4 * sin(sqrt_attack * pi)),
        0.2 * sin(sqrt_attack * (std::f64::consts::PI * 2.0) as f32),
        -0.2 * sin(attack * pi),
    );
    let y_swing_rotation = sin(attack * attack * pi);
    rotate(pose, 'y', invert * (45.0 + y_swing_rotation * -20.0));
    let xz_swing_rotation = sin(sqrt_attack * pi);
    rotate(pose, 'z', invert * xz_swing_rotation * -20.0);
    rotate(pose, 'x', xz_swing_rotation * -80.0);
    rotate(pose, 'y', invert * -45.0);
}

/// The shake, push and stretch both draws end with, for a draw `power` of the way through.
fn draw_finish(pose: &mut Pose, (held, power): (f32, f32), invert: f32) {
    if power > 0.1 {
        let shake = sin((held - 0.1) * 1.3) * (power - 0.1);
        pose.translate(shake * 0.0, shake * 0.004, shake * 0.0);
    }
    pose.translate(power * 0.0, power * 0.0, power * 0.04);
    pose.scale(1.0, 1.0, 1.0 + power * 0.2);
    rotate(pose, 'y', -invert * 45.0);
}

/// The pose of the hand's stack after the arm transform: `invert` is 1 for the right arm and -1
/// for the left, `attack` how far the swing is.
pub fn apply(pose: &mut Pose, motion: HandMotion, (invert, attack): (f32, f32)) {
    match motion {
        HandMotion::Swing => swing_arm(pose, invert, attack),
        HandMotion::Bow { held } => {
            pose.translate(invert * -0.278_568_2, 0.183_443_87, 0.157_315_31);
            rotate(pose, 'x', -13.935);
            rotate(pose, 'y', invert * 35.3);
            rotate(pose, 'z', invert * -9.785);
            let power = held / 20.0;
            let power = ((power * power + power * 2.0) / 3.0).min(1.0);
            draw_finish(pose, (held, power), invert);
        }
        HandMotion::CrossbowCharge {
            held,
            charge_duration,
        } => {
            pose.translate(invert * -0.478_568_2, -0.094_387, 0.057_315_31);
            rotate(pose, 'x', -11.935);
            rotate(pose, 'y', invert * 65.3);
            rotate(pose, 'z', invert * -9.785);
            let power = (held / charge_duration).min(1.0);
            draw_finish(pose, (held, power), invert);
        }
        HandMotion::LoadedCrossbow => {
            swing_arm(pose, invert, attack);
            if attack < 0.001 {
                pose.translate(invert * -0.641_864, 0.0, 0.0);
                rotate(pose, 'y', invert * 10.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position(motion: HandMotion, attack: f32) -> [f32; 3] {
        let mut pose = Pose::default();
        apply(&mut pose, motion, (1.0, attack));
        pose.pose.transform_position(0.0, 0.0, 0.0)
    }

    #[test]
    fn a_drawn_bow_sits_further_back_than_one_just_raised() {
        let raised = position(HandMotion::Bow { held: 0.0 }, 0.0);
        let drawn = position(HandMotion::Bow { held: 25.0 }, 0.0);
        assert!(drawn[2] > raised[2], "{raised:?} {drawn:?}");
    }

    #[test]
    fn a_loaded_crossbow_is_held_off_to_the_side_until_a_swing() {
        let idle = position(HandMotion::LoadedCrossbow, 0.0);
        let plain = position(HandMotion::Swing, 0.0);
        assert!((idle[0] - plain[0]).abs() > 0.5, "{idle:?} {plain:?}");
        let swinging = position(HandMotion::LoadedCrossbow, 0.5);
        assert_eq!(swinging, position(HandMotion::Swing, 0.5));
    }
}
