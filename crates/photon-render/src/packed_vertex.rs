//! A section's vertices as the terrain pipelines read them: 20 bytes instead of 28
//! ([ADR 0017](../../../docs/adr/0017-packed-terrain-vertices.md)).
//!
//! The position is three 16-bit steps of 1/2048 block from the mesh's origin corner, which the
//! draw hands the shader with the section's other per-draw values. The CPU mesh keeps its `f32`
//! positions: the face groups, the translucent order and the tests read those.

use crate::scene::BlockVertex;

/// Steps per block of a packed position.
pub(crate) const STEPS_PER_BLOCK: f32 = 2048.0;

/// One vertex of a section mesh as it is uploaded: 20 bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub(crate) struct PackedVertex {
    /// The position from the mesh's origin, in steps of 1/2048 block.
    pub(crate) position: [u16; 3],
    /// Block and sky light, each 0 to 240.
    pub(crate) light: [u8; 2],
    /// Texture coordinate in atlas units, already normalised.
    pub(crate) uv: [f32; 2],
    /// Red, green, blue and alpha.
    pub(crate) colour: [u8; 4],
}

impl PackedVertex {
    /// The position this vertex stands for, from the origin it was packed against.
    #[cfg(test)]
    pub(crate) fn position_from(&self, origin: [i32; 3]) -> [f32; 3] {
        std::array::from_fn(|axis| {
            origin[axis] as f32 + f32::from(self.position[axis]) / STEPS_PER_BLOCK
        })
    }
}

/// A mesh's packed vertices and the corner they count from, made where the mesh is built so the
/// upload does not have to.
#[derive(Debug, Clone, Default)]
pub struct PackedMesh {
    pub(crate) vertices: Vec<PackedVertex>,
    pub(crate) origin: [i32; 3],
}

impl PackedMesh {
    /// Pack `vertices`.
    pub(crate) fn new(vertices: &[BlockVertex]) -> Self {
        let (vertices, origin) = pack(vertices);
        Self { vertices, origin }
    }
}

/// The vertices of a mesh packed against the block corner at or below their lowest position, and
/// that corner (in blocks, from the mesh's origin).
///
/// The corner is a whole number of blocks, so the steps of two meshes that share a vertex fall on
/// the same grid and round it alike. A position more than 32 blocks from the corner is held at the
/// end of the range.
pub(crate) fn pack(vertices: &[BlockVertex]) -> (Vec<PackedVertex>, [i32; 3]) {
    let mut low = [f32::INFINITY; 3];
    for vertex in vertices {
        for (axis, low) in low.iter_mut().enumerate() {
            *low = low.min(vertex.position[axis]);
        }
    }
    let origin: [i32; 3] = low.map(|low| {
        if low.is_finite() {
            low.floor() as i32
        } else {
            0
        }
    });
    let packed = vertices
        .iter()
        .map(|vertex| PackedVertex {
            position: std::array::from_fn(|axis| {
                // The difference of a float and a whole number is exact where it matters, so the
                // rounding is the only error.
                let offset = vertex.position[axis] - origin[axis] as f32;
                // The cast holds a value out of the range at its end.
                (offset * STEPS_PER_BLOCK + 0.5) as u16
            }),
            light: vertex.light,
            uv: vertex.uv,
            colour: vertex.colour,
        })
        .collect();
    (packed, origin)
}

/// The origin as the two words the shader reads: x and y as 16-bit integers in the first, z in
/// the second.
pub(crate) fn origin_words(origin: [i32; 3]) -> [u32; 2] {
    let short = |value: i32| u32::from(value.clamp(-32768, 32767) as i16 as u16);
    [short(origin[0]) | short(origin[1]) << 16, short(origin[2])]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(position: [f32; 3]) -> BlockVertex {
        BlockVertex::new(position, [0.25, 0.75], [0.5, 0.25, 1.0, 1.0], [48.0, 240.0])
    }

    #[test]
    fn a_packed_position_is_within_a_quarter_of_a_step_of_the_mesh_position() {
        let vertices = [
            vertex([-47.3, 5.0, 120.123_46]),
            vertex([-31.0, 20.999, 130.0]),
            vertex([-40.0625, 12.5, 125.7]),
        ];
        let (packed, origin) = pack(&vertices);
        assert_eq!(origin, [-48, 5, 120]);
        for (packed, vertex) in packed.iter().zip(&vertices) {
            let back = packed.position_from(origin);
            for (back, exact) in back.iter().zip(vertex.position) {
                // 1/4096 block is half a step.
                assert!((back - exact).abs() <= 1.0 / 4096.0 + 1e-6);
            }
            assert_eq!(
                (packed.uv, packed.colour, packed.light),
                (vertex.uv, vertex.colour, vertex.light)
            );
        }
    }

    #[test]
    fn a_vertex_two_meshes_share_packs_alike_against_either_origin() {
        let shared = [16.123_4, 3.5, -7.77];
        let first = pack(&[vertex([0.0, 0.0, -16.0]), vertex(shared)]);
        let second = pack(&[vertex([16.0, 3.0, -8.0]), vertex(shared)]);
        let absolute =
            |(packed, origin): &(Vec<PackedVertex>, [i32; 3])| packed[1].position_from(*origin);
        assert_eq!(absolute(&first), absolute(&second));
    }

    #[test]
    fn the_origin_travels_as_two_words() {
        let [xy, z] = origin_words([-3, 70, -200]);
        assert_eq!(xy & 0xFFFF, (-3_i16) as u16 as u32);
        assert_eq!(xy >> 16, 70);
        assert_eq!(z, (-200_i16) as u16 as u32);
    }

    #[test]
    fn the_size_is_twenty_bytes() {
        assert_eq!(std::mem::size_of::<PackedVertex>(), 20);
    }
}
