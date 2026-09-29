//! The aimed block's outline, as the reference draws it: every edge of the
//! block's outline shape as a line a few pixels wide, in 40% black, drawn a
//! little in front of the block so it is not hidden by its faces.
//!
//! A line becomes a screen-aligned strip in the vertex shader. Each edge is
//! two triangles; a vertex carries the edge's direction and its width, signed
//! for the side of the edge it sits on.

use crate::shape::VoxelShape;

/// One vertex of a line strip.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineVertex {
    /// Position in mesh units, pulled towards the camera.
    pub position: [f32; 3],
    pub colour: [f32; 4],
    /// The edge's direction, unit length.
    pub normal: [f32; 3],
    /// The line width in pixels, negative on one side of the edge.
    pub width: f32,
}

/// `Window.getAppropriateLineWidth`: at least 2.5 pixels, wider on windows
/// wider than 1920.
pub fn appropriate_line_width(window_width: u32) -> f32 {
    (window_width as f32 / 1920.0 * 2.5).max(2.5)
}

/// `ARGB.black(102)`.
pub const OUTLINE_COLOUR: [f32; 4] = [0.0, 0.0, 0.0, 102.0 / 255.0];

/// The view-space shrink the lines get: the render type's layering
/// transform, then the line shader's own.
const VIEW_SHRINK: f32 = (1.0 - 1.0 / 4096.0) * (1.0 - 1.0 / 256.0);

/// The strips for every edge of `shape` placed at `block` (mesh units), seen
/// from `camera` (mesh units).
pub fn outline_vertices(
    shape: &VoxelShape,
    block: [f32; 3],
    camera: [f32; 3],
    colour: [f32; 4],
    width: f32,
) -> Vec<LineVertex> {
    let mut vertices = Vec::new();
    let place = |x: f64, y: f64, z: f64| {
        let world = [
            block[0] + x as f32,
            block[1] + y as f32,
            block[2] + z as f32,
        ];
        std::array::from_fn(|axis| camera[axis] + (world[axis] - camera[axis]) * VIEW_SHRINK)
    };
    shape.for_all_edges(|x1, y1, z1, x2, y2, z2| {
        let delta = [(x2 - x1) as f32, (y2 - y1) as f32, (z2 - z1) as f32];
        let length = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
        let normal = delta.map(|value| value / length);
        let start = place(x1, y1, z1);
        let end = place(x2, y2, z2);
        let vertex = |position, side: f32| LineVertex {
            position,
            colour,
            normal,
            width: width * side,
        };
        // The reference's line index pattern: 0 1 2, 3 2 1.
        vertices.extend([
            vertex(start, 1.0),
            vertex(start, -1.0),
            vertex(end, 1.0),
            vertex(end, -1.0),
            vertex(end, 1.0),
            vertex(start, -1.0),
        ]);
    });
    vertices
}
