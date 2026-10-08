//! An item frame (`ItemFrameRenderer`): where its frame model and the item in it are drawn, from
//! the way it faces, whether it is invisible and how the item is turned.

use photon_core::direction::{Axis, Direction};

use crate::joml::{Pose, Quaternion};

/// How far the frame hangs from the middle of its block towards the wall (`ItemFrame`'s box and
/// `ItemFrameRenderer.submit`'s shift back out).
const WALL_SHIFT: f64 = 0.46875;

/// Where `ItemFrame.createBoundingBox` puts the entity: `Vec3.atCenterOf(block)` moved
/// `0.46875` towards the wall.
pub fn item_frame_position(block: (i32, i32, i32), direction: Direction) -> (f64, f64, f64) {
    let (step_x, step_y, step_z) = direction.step();
    (
        f64::from(block.0) + 0.5 + f64::from(step_x) * -WALL_SHIFT,
        f64::from(block.1) + 0.5 + f64::from(step_y) * -WALL_SHIFT,
        f64::from(block.2) + 0.5 + f64::from(step_z) * -WALL_SHIFT,
    )
}

/// `ItemFrameRenderer.getRenderOffset`: the shift the dispatcher adds, which `submit` takes back.
pub fn render_offset(direction: Direction) -> (f64, f64, f64) {
    (
        f64::from(direction.get_step_x() as f32 * 0.3),
        -0.25,
        f64::from(direction.get_step_z() as f32 * 0.3),
    )
}

/// The poses `ItemFrameRenderer.submit` draws with: the frame's model (when it has one: not an
/// invisible frame's), the item, and the corner of the map that takes the item's place when the
/// frame holds one whose data has come.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemFramePoses {
    pub frame: Option<Pose>,
    pub item: Pose,
    pub map: Pose,
}

/// `base` is the pose the dispatcher leaves: the entity's place moved by [`render_offset`].
pub fn item_frame_poses(
    base: &Pose,
    direction: Direction,
    invisible: bool,
    rotation: i32,
) -> ItemFramePoses {
    let degrees = Quaternion::degrees_to_radians;
    let mut pose = *base;
    let offset = render_offset(direction);
    pose.translate(-offset.0 as f32, -offset.1 as f32, -offset.2 as f32);
    pose.translate(
        (f64::from(direction.get_step_x()) * WALL_SHIFT) as f32,
        (f64::from(direction.get_step_y()) * WALL_SHIFT) as f32,
        (f64::from(direction.get_step_z()) * WALL_SHIFT) as f32,
    );
    let (x_rot, y_rot) = if direction.axis() != Axis::Y {
        (0.0, 180.0 - direction.to_y_rot())
    } else {
        (-90.0 * direction.get_step_y() as f32, 180.0)
    };
    pose.rotate(&Quaternion::rotation_x(degrees(x_rot)));
    pose.rotate(&Quaternion::rotation_y(degrees(y_rot)));
    let frame = (!invisible).then(|| {
        let mut frame = pose;
        frame.translate(-0.5, -0.5, -0.5);
        frame
    });
    pose.translate(0.0, 0.0, if invisible { 0.5 } else { 0.4375 });
    // A map: turned by quarter turns (`rotation % 4 * 2` eighths), flipped, scaled to its 128
    // colours a block wide, and its corner put at the middle.
    let mut map = pose;
    let eighths = rotation % 4 * 2;
    map.rotate(&Quaternion::rotation_z(degrees(
        eighths as f32 * 360.0 / 8.0,
    )));
    map.rotate(&Quaternion::rotation_z(degrees(180.0)));
    map.scale(0.007_812_5, 0.007_812_5, 0.007_812_5);
    map.translate(-64.0, -64.0, 0.0);
    map.translate(0.0, 0.0, -1.0);
    pose.rotate(&Quaternion::rotation_z(degrees(
        rotation as f32 * 360.0 / 8.0,
    )));
    pose.scale(0.5, 0.5, 0.5);
    ItemFramePoses {
        frame,
        item: pose,
        map,
    }
}
