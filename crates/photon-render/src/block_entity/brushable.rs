//! The item a suspicious block holds (`BrushableBlockRenderer`): it sits against the face that was
//! brushed and rises out of the dust as the block is brushed away.

use crate::joml::{Pose, Quaternion};
use photon_core::direction::Direction;

/// `BrushableBlockRenderer.translations`: where the item sits, `completion` being how many of the
/// ten stages of dust are left.
fn translations(direction: Direction, completion: i32) -> [f32; 3] {
    let mut offset = [0.5, 0.0, 0.5];
    let step = completion as f32 / 10.0 * 0.75;
    match direction {
        Direction::East => offset[0] = 0.73 + step,
        Direction::West => offset[0] = 0.25 - step,
        Direction::Up => offset[1] = 0.25 + step,
        Direction::Down => offset[1] = -0.23 - step,
        Direction::North => offset[2] = 0.25 - step,
        Direction::South => offset[2] = 0.73 + step,
    }
    offset
}

/// `BrushableBlockRenderer.submit`: the pose the item is drawn from, with `base` the pose of the
/// block's corner.
pub fn item_pose(base: &Pose, direction: Direction, dust: i32) -> Pose {
    let mut pose = *base;
    pose.translate(0.0, 0.5, 0.0);
    let [x, y, z] = translations(direction, dust);
    pose.translate(x, y, z);
    let turn = |degrees: f32| Quaternion::rotation_y(Quaternion::degrees_to_radians(degrees));
    pose.rotate(&turn(75.0));
    let east_west = matches!(direction, Direction::East | Direction::West);
    pose.rotate(&turn((if east_west { 90 } else { 0 } + 11) as f32));
    pose.scale(0.5, 0.5, 0.5);
    pose
}
