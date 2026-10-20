//! The F3+B entity hitboxes, as `EntityHitboxDebugRenderer` emits them: `Gizmos.cuboid` strokes
//! and `Gizmos.arrow` lines, each a few screen-aligned strips of the line pipeline.

use super::outline::{LineVertex, pull_towards};

/// `GizmoStyle.DEFAULT_WIDTH` and `ArrowGizmo.DEFAULT_WIDTH`.
pub const WIDTH: f32 = 2.5;

/// `Gizmos.line`: two triangles from `start` to `end`, both pulled towards the camera.
pub fn line_strip(
    (start, end): ([f32; 3], [f32; 3]),
    colour: [f32; 4],
    camera: [f32; 3],
) -> [LineVertex; 6] {
    let (start, end) = (pull_towards(start, camera), pull_towards(end, camera));
    let delta: [f32; 3] = std::array::from_fn(|axis| end[axis] - start[axis]);
    let length = delta.iter().map(|value| value * value).sum::<f32>().sqrt();
    let normal = delta.map(|value| value / length);
    let vertex = |position, side: f32| LineVertex {
        position,
        colour,
        normal,
        width: WIDTH * side,
    };
    [
        vertex(start, 1.0),
        vertex(start, -1.0),
        vertex(end, 1.0),
        vertex(end, -1.0),
        vertex(end, 1.0),
        vertex(start, -1.0),
    ]
}

/// `CuboidGizmo.emit` for a stroke: the twelve edges of the box, in the reference's order.
pub fn cuboid_strips(
    (min, max): ([f32; 3], [f32; 3]),
    colour: [f32; 4],
    camera: [f32; 3],
) -> Vec<LineVertex> {
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;
    let edges = [
        ([x0, y0, z0], [x1, y0, z0]),
        ([x0, y0, z0], [x0, y1, z0]),
        ([x0, y0, z0], [x0, y0, z1]),
        ([x1, y0, z0], [x1, y1, z0]),
        ([x1, y1, z0], [x0, y1, z0]),
        ([x0, y1, z0], [x0, y1, z1]),
        ([x0, y1, z1], [x0, y0, z1]),
        ([x0, y0, z1], [x1, y0, z1]),
        ([x1, y0, z1], [x1, y0, z0]),
        ([x0, y1, z1], [x1, y1, z1]),
        ([x1, y0, z1], [x1, y1, z1]),
        ([x1, y1, z0], [x1, y1, z1]),
    ];
    edges
        .into_iter()
        .flat_map(|edge| line_strip(edge, colour, camera))
        .collect()
}

/// The quaternion that turns the x axis onto the unit vector `to` (`Quaternionf.rotationTo`),
/// as `(x, y, z, w)`.
fn x_axis_to([x, y, z]: [f32; 3]) -> [f32; 4] {
    // The cross product of (1, 0, 0) with `to` is (0, -z, y).
    if x < -0.999_999 {
        // Opposite: a half turn about any axis across x.
        return [0.0, 0.0, 1.0, 0.0];
    }
    let (qy, qz, qw) = (-z, y, 1.0 + x);
    let length = (qy * qy + qz * qz + qw * qw).sqrt();
    [0.0, qy / length, qz / length, qw / length]
}

/// A vector turned by a unit quaternion.
fn rotate([qx, qy, qz, qw]: [f32; 4], [x, y, z]: [f32; 3]) -> [f32; 3] {
    // v + 2w (q x v) + 2 q x (q x v)
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let q = [qx, qy, qz];
    let t = cross(q, [x, y, z]).map(|value| 2.0 * value);
    let u = cross(q, t);
    [
        x + qw * t[0] + u[0],
        y + qw * t[1] + u[1],
        z + qw * t[2] + u[2],
    ]
}

/// `ArrowGizmo.emit`: the shaft and four tips folded back from its end.
pub fn arrow_strips(
    (start, end): ([f32; 3], [f32; 3]),
    colour: [f32; 4],
    camera: [f32; 3],
) -> Vec<LineVertex> {
    let delta: [f32; 3] = std::array::from_fn(|axis| end[axis] - start[axis]);
    let length = delta.iter().map(|value| value * value).sum::<f32>().sqrt();
    let rotation = x_axis_to(delta.map(|value| value / length));
    let tip_length = (length * 0.1).clamp(0.1, 1.0);
    let tips = [
        [-tip_length, tip_length, 0.0],
        [-tip_length, 0.0, tip_length],
        [-tip_length, -tip_length, 0.0],
        [-tip_length, 0.0, -tip_length],
    ];
    let mut strips = line_strip((start, end), colour, camera).to_vec();
    for tip in tips {
        let turned = rotate(rotation, tip);
        let from = std::array::from_fn(|axis| end[axis] + turned[axis]);
        strips.extend(line_strip((from, end), colour, camera));
    }
    strips
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_is_twelve_edges_and_an_arrow_a_shaft_with_four_tips() {
        let camera = [0.0; 3];
        let white = [1.0; 4];
        assert_eq!(
            cuboid_strips(([0.0; 3], [1.0; 3]), white, camera).len(),
            12 * 6
        );
        assert_eq!(
            arrow_strips(([0.0; 3], [0.0, 0.0, 2.0]), white, camera).len(),
            5 * 6
        );
    }

    #[test]
    fn the_tips_fold_back_from_the_end_of_the_arrow() {
        // An arrow along +x: the tips lie behind the end by a tenth of its length, at least 0.1.
        let strips = arrow_strips(([0.0; 3], [2.0, 0.0, 0.0]), [1.0; 4], [0.0; 3]);
        let tip = strips[6].position;
        assert!((tip[0] - 1.8).abs() < 0.01 && (tip[1] - 0.2).abs() < 0.01 && tip[2].abs() < 0.01);
        // Along -x the half turn mirrors them across the end.
        let back = arrow_strips(([0.0; 3], [-2.0, 0.0, 0.0]), [1.0; 4], [0.0; 3]);
        assert!((back[6].position[0] + 1.8).abs() < 0.01);
    }
}
