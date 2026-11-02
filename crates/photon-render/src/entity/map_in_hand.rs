//! A map in the first-person view (`ItemInHandRenderer.renderOneHandedMap` and
//! `renderTwoHandedMap`): the arms that hold it and the pose its quad is drawn from.

use super::player_model::{Arm, PlayerModel};
use super::player_renderer::{HandView, arm_vertices, player_arm_pose, view_pose};
use crate::joml::{Pose, Quaternion};
use crate::scene::Vertex;

/// The arms of a held map and where its quad is.
pub struct MapInHand {
    pub arms: Vec<Vertex>,
    /// The pose `renderMap` leaves: the map's own units, 128 to a side, from its corner.
    pub pose: Pose,
}

fn degrees(value: f32) -> f32 {
    Quaternion::degrees_to_radians(value)
}

fn sin(value: f32) -> f32 {
    photon_core::math::mth_sin_f64(f64::from(value))
}

fn pi() -> f32 {
    std::f64::consts::PI as f32
}

/// `ItemInHandRenderer.renderMap`'s turn and scale, from the pose of the hand.
fn map_pose(mut pose: Pose) -> Pose {
    pose.rotate(&Quaternion::rotation_y(degrees(180.0)));
    pose.rotate(&Quaternion::rotation_z(degrees(180.0)));
    pose.scale(0.38, 0.38, 0.38);
    pose.translate(-0.5, -0.5, 0.0);
    pose.scale(0.007_812_5, 0.007_812_5, 0.007_812_5);
    pose
}

/// A map in one hand (the off hand's, or the main hand's with the off hand full), the arm that
/// holds it drawn tipped out to the side unless the player is invisible.
pub fn one_handed(
    model: &mut PlayerModel,
    view: &HandView,
    (right_arm, attack, inverse_arm_height): (bool, f32, f32),
    invisible: bool,
) -> MapInHand {
    let invert = if right_arm { 1.0_f32 } else { -1.0 };
    let mut pose = view_pose(view);
    pose.translate(invert * 0.125, -0.125, 0.0);
    let mut arms = Vec::new();
    if !invisible {
        let mut arm_pose = pose;
        arm_pose.rotate(&Quaternion::rotation_z(degrees(invert * 10.0)));
        player_arm_pose(&mut arm_pose, right_arm, (attack, inverse_arm_height));
        let (arm, sleeve) = if right_arm {
            (Arm::Right, view.sleeves[0])
        } else {
            (Arm::Left, view.sleeves[1])
        };
        arms = arm_vertices(model, &arm_pose, (arm, sleeve), view);
    }
    pose.translate(invert * 0.51, -0.08 + inverse_arm_height * -1.2, -0.75);
    let sqrt_attack = f64::from(attack).sqrt() as f32;
    let x_swing = sin(sqrt_attack * pi());
    let x_swing_position = -0.5 * x_swing;
    let y_swing_position = 0.4 * sin(sqrt_attack * (std::f64::consts::PI * 2.0) as f32);
    let z_swing_position = -0.3 * sin(attack * pi());
    pose.translate(
        invert * x_swing_position,
        y_swing_position - 0.3 * x_swing,
        z_swing_position,
    );
    pose.rotate(&Quaternion::rotation_x(degrees(x_swing * -45.0)));
    pose.rotate(&Quaternion::rotation_y(degrees(invert * x_swing * -30.0)));
    MapInHand {
        arms,
        pose: map_pose(pose),
    }
}

/// `calculateMapTilt`: how far the map is tipped toward the face as the player looks down.
fn map_tilt(x_rot: f32) -> f32 {
    let tilt = (1.0 - x_rot / 45.0 + 0.1).clamp(0.0, 1.0);
    -photon_core::math::mth_cos_f64(f64::from(tilt * pi())) * 0.5 + 0.5
}

/// A map in both hands: the main hand's with the off hand empty.
pub fn two_handed(
    model: &mut PlayerModel,
    view: &HandView,
    (x_rot, attack, inverse_arm_height): (f32, f32, f32),
    invisible: bool,
) -> MapInHand {
    let mut pose = view_pose(view);
    let sqrt_attack = f64::from(attack).sqrt() as f32;
    let y_swing_position = -0.2 * sin(attack * pi());
    let z_swing_position = -0.4 * sin(sqrt_attack * pi());
    pose.translate(0.0, -y_swing_position / 2.0, z_swing_position);
    let tilt = map_tilt(x_rot);
    pose.translate(0.0, 0.04 + inverse_arm_height * -1.2 + tilt * -0.5, -0.72);
    pose.rotate(&Quaternion::rotation_x(degrees(tilt * -85.0)));
    let mut arms = Vec::new();
    if !invisible {
        let mut both = pose;
        both.rotate(&Quaternion::rotation_y(degrees(90.0)));
        for (arm, invert, sleeve) in [
            (Arm::Right, 1.0_f32, view.sleeves[0]),
            (Arm::Left, -1.0, view.sleeves[1]),
        ] {
            let mut arm_pose = both;
            arm_pose.rotate(&Quaternion::rotation_y(degrees(92.0)));
            arm_pose.rotate(&Quaternion::rotation_x(degrees(45.0)));
            arm_pose.rotate(&Quaternion::rotation_z(degrees(invert * -41.0)));
            arm_pose.translate(invert * 0.3, -1.1, 0.45);
            arms.extend(arm_vertices(model, &arm_pose, (arm, sleeve), view));
        }
    }
    let z_swing_rotation = sin(sqrt_attack * pi());
    pose.rotate(&Quaternion::rotation_x(degrees(z_swing_rotation * 20.0)));
    pose.scale(2.0, 2.0, 2.0);
    MapInHand {
        arms,
        pose: map_pose(pose),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> HandView {
        HandView {
            right_arm: true,
            sleeve: true,
            sleeves: [true; 2],
            bob_walked: 0.0,
            bob: 0.0,
            view_x_rot: 40.0,
            view_y_rot: 0.0,
            x_bob: 40.0,
            y_bob: 0.0,
            inverse_arm_height: 0.0,
            attack: 0.0,
            light: 0,
            skin: [0.0, 0.0, 1.0, 1.0],
            hurt: crate::camera::HurtView::none(),
        }
    }

    #[test]
    fn a_two_handed_map_is_in_front_of_the_face() {
        let mut model = PlayerModel::new(false);
        model.setup_anim(&Default::default());
        let held = two_handed(&mut model, &view(), (40.0, 0.0, 0.0), false);
        let corner = |x: f32, y: f32| held.pose.pose.transform_position(x, y, 0.0);
        println!("{:?} {:?}", corner(0.0, 0.0), corner(128.0, 128.0));
        let (low, high) = (corner(0.0, 0.0), corner(128.0, 128.0));
        assert!((low[0] + high[0]).abs() < 0.2, "{low:?} {high:?}");
        assert!(low[2] < -0.4 && high[2] < -0.4, "{low:?} {high:?}");
    }
}
