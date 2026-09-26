//! A single textured cube, drawn from the pinned geometry and texture paths.
//!
//! This is scaffolding, not a parity claim. What it does carry over from the
//! verified slices is the data: the corner order and default UVs come from
//! [`crate::render::geometry`], the sprite rectangle from
//! [`crate::render::atlas`], and the texels from [`crate::render::png`]. The
//! camera, the shaders and the lighting are this module's own invention and
//! match nothing in the reference yet.

use crate::direction::Direction;
use crate::render::geometry::{Quadrant, default_face_uv, face_vertex_uv, face_vertices};

/// One vertex as the shaders expect it.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    /// Position in block units.
    pub position: [f32; 3],
    /// Texture coordinate in atlas units, already normalised.
    pub uv: [f32; 2],
    /// Flat face shade, the only lighting this scaffolding applies.
    pub shade: f32,
}

/// Vertices and indices for one cube.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// Vertex data, four per face.
    pub vertices: Vec<Vertex>,
    /// Triangle indices, six per face.
    pub indices: Vec<u32>,
}

/// Where one sprite sits on the atlas, in normalised coordinates.
#[derive(Debug, Clone, Copy)]
pub struct SpriteRect {
    /// Left edge.
    pub min_u: f32,
    /// Top edge.
    pub min_v: f32,
    /// Right edge.
    pub max_u: f32,
    /// Bottom edge.
    pub max_v: f32,
}

/// Build a full-block cube from the pinned corner and UV tables.
///
/// The element box is the whole block, so every face takes the default UV
/// rectangle; the sprite rectangle then maps those sixteenths onto the atlas.
pub fn full_cube(sprite: SpriteRect) -> Mesh {
    const FACES: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];
    // Flat per-face shading, the same relative ordering the reference uses for
    // its face light, but the values here are this module's own choice.
    const SHADE: [f32; 6] = [0.5, 1.0, 0.8, 0.8, 0.6, 0.6];

    let from = [0.0_f32; 3];
    let to = [16.0_f32; 3];
    let mut mesh = Mesh::default();
    for (face_index, facing) in FACES.into_iter().enumerate() {
        let positions = face_vertices(from, to, facing);
        let uvs = default_face_uv(from, to, facing);
        let base = mesh.vertices.len() as u32;
        for (slot, position) in positions.into_iter().enumerate() {
            // The per-vertex corner and the unrotated case both come from the
            // pinned face UV table rather than being written out here.
            let (u, v) = face_vertex_uv(uvs, Quadrant::R0, slot as u32);
            mesh.vertices.push(Vertex {
                position,
                uv: [
                    sprite.min_u + (sprite.max_u - sprite.min_u) * u,
                    sprite.min_v + (sprite.max_v - sprite.min_v) * v,
                ],
                shade: SHADE[face_index],
            });
        }
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    mesh
}

/// A right-handed perspective projection with the depth range Vulkan expects.
pub fn perspective(vertical_fov_radians: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let focal = 1.0 / (vertical_fov_radians / 2.0).tan();
    [
        [focal / aspect, 0.0, 0.0, 0.0],
        // Vulkan's clip space points down, unlike the maths convention.
        [0.0, -focal, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), -1.0],
        [0.0, 0.0, near * far / (near - far), 0.0],
    ]
}

/// A view matrix looking from `eye` at `target`.
pub fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let forward = normalise(subtract(target, eye));
    let right = normalise(cross(forward, up));
    let true_up = cross(right, forward);
    [
        [right[0], true_up[0], -forward[0], 0.0],
        [right[1], true_up[1], -forward[1], 0.0],
        [right[2], true_up[2], -forward[2], 0.0],
        [-dot(right, eye), -dot(true_up, eye), dot(forward, eye), 1.0],
    ]
}

/// Column-major matrix product, `left` applied after `right`.
pub fn multiply(left: [[f32; 4]; 4], right: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0_f32; 4]; 4];
    for (column, source) in out.iter_mut().zip(right) {
        for (row, slot) in column.iter_mut().enumerate() {
            *slot = (0..4).map(|k| left[k][row] * source[k]).sum();
        }
    }
    out
}

fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn normalise(value: [f32; 3]) -> [f32; 3] {
    let length = dot(value, value).sqrt();
    if length == 0.0 {
        return value;
    }
    [value[0] / length, value[1] / length, value[2] / length]
}
