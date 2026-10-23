//! `LayeringTransform` of the render types that pull their quads off a coplanar face: under a
//! perspective projection (`ProjectionType.PERSPECTIVE`) the model-view is scaled about the eye
//! by `1 - bias / 4096`, which changes only a vertex's depth, never the pixel it lands on.

use crate::scene::Vertex;

/// The layering a render type declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layering {
    /// `VIEW_OFFSET_Z_LAYERING` (bias 1): towards the camera.
    ViewOffset,
    /// `VIEW_OFFSET_Z_LAYERING_FORWARD` (bias -1): away from the camera.
    ViewOffsetForward,
}

impl Layering {
    /// The factor the model-view is scaled by.
    pub const fn scale(self) -> f32 {
        let bias = match self {
            Self::ViewOffset => 1.0,
            Self::ViewOffsetForward => -1.0,
        };
        1.0 - bias / 4096.0
    }

    /// `vertices` (positions relative to a mesh origin) scaled about the eye, which is `eye`
    /// from that origin.
    pub fn apply(self, vertices: &mut [Vertex], eye: [f32; 3]) {
        let scale = self.scale();
        for vertex in vertices {
            for (axis, value) in vertex.position.iter_mut().enumerate() {
                *value = (*value - eye[axis]) * scale + eye[axis];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(position: [f32; 3]) -> Vertex {
        Vertex {
            position,
            uv: [0.0; 2],
            shade: 0.0,
            tint: [1.0; 4],
            light: [0.0; 2],
        }
    }

    #[test]
    fn the_scale_follows_the_bias_of_each_transform() {
        assert_eq!(Layering::ViewOffset.scale(), 1.0 - 1.0 / 4096.0);
        assert_eq!(Layering::ViewOffsetForward.scale(), 1.0 + 1.0 / 4096.0);
    }

    #[test]
    fn a_vertex_moves_along_its_line_to_the_eye() {
        let mut vertices = [vertex([14.0, 6.0, 4.0])];
        Layering::ViewOffset.apply(&mut vertices, [10.0, 6.0, 4.0]);
        let [x, y, z] = vertices[0].position;
        assert_eq!(x, 10.0 + 4.0 * (1.0 - 1.0 / 4096.0));
        assert_eq!((y, z), (6.0, 4.0));
        Layering::ViewOffsetForward.apply(&mut vertices, [10.0, 6.0, 4.0]);
        assert!(vertices[0].position[0] > 13.99);
    }
}
