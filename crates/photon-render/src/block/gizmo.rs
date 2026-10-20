//! The F3 3D crosshair: the three world axes drawn in front of the camera, as
//! `DebugCrosshairRenderer` does. Each axis is a black line four pixels wide
//! with a coloured one two pixels wide over it; the vertices are in view space
//! and drawn with the hand's projection.

use super::outline::LineVertex;

/// `CROSSHAIR_SCALE`.
const SCALE: f32 = 0.01;
/// Each axis's colour: red, green and a light blue, then the black under them.
const COLOURS: [[f32; 4]; 3] = [
    [1.0, 0.0, 0.0, 1.0],
    [0.0, 1.0, 0.0, 1.0],
    [127.0 / 255.0, 127.0 / 255.0, 1.0, 1.0],
];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
/// How far the coloured lines are pulled towards the eye, standing in for the depth bias the
/// reference gives them so the black does not cover them.
const COLOUR_PULL: f32 = 0.999;

/// A point of the gizmo in view space: one unit along `axis` of the model, which is mirrored in x
/// and z, turned by the camera's pitch then yaw (degrees) and moved one block ahead.
fn place(axis: [f32; 3], (yaw, pitch): (f32, f32), gui_scale: f32) -> [f32; 3] {
    let scale = SCALE * gui_scale;
    let [x, y, z] = [-axis[0] * scale, axis[1] * scale, -axis[2] * scale];
    let (sy, cy) = yaw.to_radians().sin_cos();
    let (x, z) = (x * cy + z * sy, -x * sy + z * cy);
    let (sp, cp) = pitch.to_radians().sin_cos();
    [x, y * cp - z * sp, y * sp + z * cp - 1.0]
}

/// The strips of the six lines, the black ones first: two triangles each, as the outline's are.
pub fn gizmo_vertices(rotation: (f32, f32), gui_scale: f32) -> Vec<LineVertex> {
    let mut vertices = Vec::with_capacity(36);
    for (colour_pass, width) in [(false, 4.0), (true, 2.0)] {
        for (index, colour) in COLOURS.iter().enumerate() {
            let mut unit = [0.0; 3];
            unit[index] = 1.0;
            let pull = if colour_pass { COLOUR_PULL } else { 1.0 };
            let start = place([0.0; 3], rotation, gui_scale).map(|v| v * pull);
            let end = place(unit, rotation, gui_scale).map(|v| v * pull);
            let delta: [f32; 3] = std::array::from_fn(|i| end[i] - start[i]);
            let length = delta.iter().map(|v| v * v).sum::<f32>().sqrt();
            let normal = delta.map(|v| v / length);
            let vertex = |position, side: f32| LineVertex {
                position,
                colour: if colour_pass { *colour } else { BLACK },
                normal,
                width: width * side,
            };
            vertices.extend([
                vertex(start, 1.0),
                vertex(start, -1.0),
                vertex(end, 1.0),
                vertex(end, -1.0),
                vertex(end, 1.0),
                vertex(start, -1.0),
            ]);
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_north_the_axes_point_right_up_and_towards_the_viewer() {
        // Yaw 180 is north: the view looks along -z, so world +x is to the right.
        let north = gizmo_vertices((180.0, 0.0), 1.0);
        let tip = |i: usize| north[6 * i + 2].position;
        assert!(tip(0)[0] > 0.0, "x to the right");
        assert!(tip(1)[1] > 0.0, "y up");
        assert!(tip(2)[2] > -1.0, "z towards the viewer");
        assert_eq!(north.len(), 36);
        assert_eq!(north[0].width, 4.0);
        assert_eq!(north[18].width, 2.0);
    }
}
