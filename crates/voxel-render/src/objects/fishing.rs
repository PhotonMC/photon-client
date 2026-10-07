//! A fishing bobber's line (`FishingHookRenderer`): sixteen segments hanging in a curve from the
//! bobber to the holder's hand, drawn as the reference's lines, and where that hand is.

use voxel_core::math::{mth_cos_f64, mth_sin_f64};

use crate::block::outline::LineVertex;
use crate::joml::Pose;

/// How many segments the line has.
const STEPS: i32 = 16;

/// The line's colour, `0xFF000000`.
const COLOUR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// One end of a segment as the reference submits it: placed, with the unit direction its
/// normal carries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StringVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

/// `Mth.sqrt(float)`.
fn sqrt(value: f32) -> f32 {
    f64::from(value).sqrt() as f32
}

/// `FishingHookRenderer.stringVertex`, from the pose the line is submitted at.
fn string_vertex(pose: &Pose, [xa, ya, za]: [f32; 3], aa: f32, next: f32) -> StringVertex {
    let x = xa * aa;
    let y = ya * (aa * aa + aa) * 0.5 + 0.25;
    let z = za * aa;
    let mut nx = xa * next - x;
    let mut ny = ya * (next * next + next) * 0.5 + 0.25 - y;
    let mut nz = za * next - z;
    let length = sqrt(nx * nx + ny * ny + nz * nz);
    nx /= length;
    ny /= length;
    nz /= length;
    StringVertex {
        position: pose.pose.transform_position(x, y, z),
        normal: pose.transform_normal([nx, ny, nz]),
    }
}

/// The vertices the line's custom geometry hands the vertex consumer, in order: each segment
/// as its two ends. `offset` runs from the bobber (a quarter above its position) to the hand.
pub fn string_vertices(pose: &Pose, offset: [f32; 3]) -> Vec<StringVertex> {
    let mut vertices = Vec::new();
    for i in 0..STEPS {
        let a0 = i as f32 / STEPS as f32;
        let a1 = (i + 1) as f32 / STEPS as f32;
        vertices.push(string_vertex(pose, offset, a0, a1));
        vertices.push(string_vertex(pose, offset, a1, a0));
    }
    vertices
}

/// The line as screen-aligned strips for the line shader (`RenderTypes.lines`): each segment
/// two triangles, its ends pulled a little towards `camera` as every line is.
pub fn string_strips(vertices: &[StringVertex], camera: [f32; 3], width: f32) -> Vec<LineVertex> {
    let mut strips = Vec::new();
    for segment in vertices.chunks_exact(2) {
        let vertex = |end: StringVertex, side: f32| LineVertex {
            position: crate::block::outline::pull_towards(end.position, camera),
            colour: COLOUR,
            normal: end.normal,
            width: width * side,
        };
        let [start, end] = [segment[0], segment[1]];
        // The reference's line index pattern: 0 1 2, 3 2 1.
        strips.extend([
            vertex(start, 1.0),
            vertex(start, -1.0),
            vertex(end, 1.0),
            vertex(end, -1.0),
            vertex(end, 1.0),
            vertex(start, -1.0),
        ]);
    }
    strips
}

/// Whoever holds the rod, as `FishingHookRenderer.getPlayerHandPos` reads them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Holder {
    /// `getEyePosition`.
    pub eye: [f64; 3],
    /// `Mth.lerp(partial, yBodyRotO, yBodyRot)`, a plain lerp.
    pub body_rot: f32,
    pub crouching: bool,
    /// `getScale`.
    pub scale: f32,
    /// Whether the rod's arm is the right (`getHoldingArm == RIGHT`).
    pub right_arm: bool,
}

/// `Vec3.yRot`.
pub fn y_rot([x, y, z]: [f64; 3], radians: f32) -> [f64; 3] {
    let (cos, sin) = (
        f64::from(mth_cos_f64(f64::from(radians))),
        f64::from(mth_sin_f64(f64::from(radians))),
    );
    [x * cos + z * sin, y, z * cos - x * sin]
}

/// `Vec3.xRot`.
pub fn x_rot([x, y, z]: [f64; 3], radians: f32) -> [f64; 3] {
    let (cos, sin) = (
        f64::from(mth_cos_f64(f64::from(radians))),
        f64::from(mth_sin_f64(f64::from(radians))),
    );
    [x, y * cos + z * sin, z * cos - y * sin]
}

/// `getPlayerHandPos` for anyone but the player whose camera it is, or from outside their head.
pub fn hand_position(holder: &Holder) -> [f64; 3] {
    let invert = if holder.right_arm { 1 } else { -1 };
    let radians = holder.body_rot * (std::f64::consts::PI / 180.0) as f32;
    let sin = f64::from(mth_sin_f64(f64::from(radians)));
    let cos = f64::from(mth_cos_f64(f64::from(radians)));
    let scale = f64::from(holder.scale);
    let right = f64::from(invert) * 0.35 * scale;
    let forward = 0.8 * scale;
    let lower = if holder.crouching { -0.1875_f32 } else { 0.0 };
    [
        holder.eye[0] + (-cos * right - sin * forward),
        holder.eye[1] + (f64::from(lower) - 0.45 * scale),
        holder.eye[2] + (-sin * right + cos * forward),
    ]
}

/// The camera's near plane (`Camera.getNearPlane`): the forward, left and up vectors scaled to it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearPlane {
    pub forward: [f64; 3],
    pub left: [f64; 3],
    pub up: [f64; 3],
}

/// `Camera.getNearPlane(fov)`, from the camera's own three vectors, the projection's near
/// distance (`0.05`) and its aspect ratio.
pub fn near_plane(
    (forwards, left, up): ([f32; 3], [f32; 3], [f32; 3]),
    fov: f32,
    aspect: f64,
) -> NearPlane {
    let z_near = f64::from(0.05_f32);
    let height = (f64::from(fov * (std::f64::consts::PI / 180.0) as f32) / 2.0).tan() * z_near;
    let width = height * aspect;
    let scaled = |vector: [f32; 3], by: f64| vector.map(|value| f64::from(value) * by);
    NearPlane {
        forward: scaled(forwards, z_near),
        left: scaled(left, width),
        up: scaled(up, height),
    }
}

/// `getPlayerHandPos` for the player whose camera is in their head: a point on the near plane
/// by the rod's side, thrown out along the view and swung with the arm (`swing` is
/// `sin(sqrt(attackAnim) * PI)`).
pub fn first_person_hand_position(
    eye: [f64; 3],
    plane: &NearPlane,
    (fov, right_arm, swing): (f32, bool, f32),
) -> [f64; 3] {
    let invert = if right_arm { 1.0_f32 } else { -1.0 };
    let (x, y) = (invert * 0.525_f32, -0.1_f32);
    let point: [f64; 3] = std::array::from_fn(|axis| {
        plane.forward[axis] + plane.up[axis] * f64::from(y) - plane.left[axis] * f64::from(x)
    });
    let scale = 960.0 / f64::from(fov);
    let scaled = point.map(|value| value * scale);
    let view = x_rot(y_rot(scaled, swing * 0.5), -swing * 0.7);
    [eye[0] + view[0], eye[1] + view[1], eye[2] + view[2]]
}

/// The line's origin relative to the bobber's string end, as `extractRenderState` computes it:
/// the hand less the bobber's position and a quarter.
pub fn line_offset(hand: [f64; 3], hook: (f64, f64, f64)) -> [f32; 3] {
    [
        (hand[0] - hook.0) as f32,
        (hand[1] - (hook.1 + 0.25)) as f32,
        (hand[2] - hook.2) as f32,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_runs_from_the_bobber_to_the_hand_in_sixteen_segments() {
        let pose = Pose::default();
        let vertices = string_vertices(&pose, [0.0, 2.0, 4.0]);
        assert_eq!(vertices.len(), 32);
        // The first segment starts a quarter above the bobber, the last ends at the offset's far end.
        assert_eq!(vertices[0].position, [0.0, 0.25, 0.0]);
        assert_eq!(vertices[31].position, [0.0, 2.25, 4.0]);
    }

    #[test]
    fn a_hand_is_at_the_side_the_rod_is_held_on() {
        let holder = Holder {
            eye: [0.0, 1.62, 0.0],
            body_rot: 0.0,
            crouching: false,
            scale: 1.0,
            right_arm: true,
        };
        let hand = hand_position(&holder);
        // Facing south (yaw 0), the right arm is to the west, in front of the body.
        assert!(hand[0] < 0.0 && hand[2] > 0.0);
        let left = hand_position(&Holder {
            right_arm: false,
            ..holder
        });
        assert!(left[0] > 0.0);
    }
}
