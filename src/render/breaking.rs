//! The crack drawn over a block being broken, as the reference's breaking
//! overlay draws it: every quad of the block's model again, textured with
//! the stage's crack. The texture coordinates come from where each corner
//! sits in the block, turned to face the way its quad faces, so the crack
//! lies flat on every side whatever the model's own texture layout.
//!
//! The ten stages are stacked in one texture; each vertex carries its stage
//! in the shade slot, and the shader wraps the coordinates into that tile.

use crate::direction::Direction;
use crate::render::scene::Vertex;

/// `SheetedDecalTextureGenerator` at scale one: the corner, half turned
/// round and tipped over, then turned by the facing's rotation; the texture
/// coordinates are the negated first two components.
pub fn decal_uv(corner: [f32; 3], facing: Direction) -> [f32; 2] {
    let [x, y, z] = corner;
    // `rotateY(π)` then `rotateX(-π/2)` send (x, y, z) to (-x, -z, -y);
    // `Direction.getRotation` then turns that as below.
    let turned = match facing {
        Direction::Up => [-x, -z],
        Direction::Down => [-x, z],
        Direction::South => [-x, y],
        Direction::North => [x, y],
        Direction::West => [z, y],
        Direction::East => [-z, y],
    };
    [-turned[0], -turned[1]]
}

/// The overlay for one block: its quads (in the model's unit cube, with the
/// block's offset) placed at `block` in mesh units, at a crack stage from 0
/// to 9.
pub fn breaking_vertices(
    quads: &[([[f32; 3]; 4], Direction)],
    offset: [f32; 3],
    block: [f32; 3],
    stage: i32,
) -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(quads.len() * 6);
    for (corners, facing) in quads {
        let vertex = |corner: usize| {
            let local = corners[corner];
            Vertex {
                position: std::array::from_fn(|axis| block[axis] + offset[axis] + local[axis]),
                uv: decal_uv(local, *facing),
                shade: stage as f32,
                tint: [1.0; 4],
                light: [240.0, 240.0],
            }
        };
        // The reference draws quads as the triangles 0 1 2 and 2 3 0.
        vertices.extend([0, 1, 2, 2, 3, 0].map(vertex));
    }
    vertices
}
