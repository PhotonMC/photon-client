//! The quads of items whose sprite has translucent texels (`BakedQuad.MaterialInfo.of`): the
//! reference draws them with the translucent item sheet (blended, depth written, texels under an
//! alpha of 0.1 dropped), after the cutout ones, and sorts them by distance to the camera.
//!
//! The item vertex lists are shared by many producers, so a quad of that kind carries a mark in
//! its light's first component until the backend splits the list ([`split`]).

use crate::scene::Vertex;

/// What a marked vertex adds to its block light, far above the lightmap's 240.
const MARK: f32 = 4096.0;

/// Mark the vertices of one quad as translucent.
pub fn mark(vertices: &mut [Vertex]) {
    for vertex in vertices {
        vertex.light[0] += MARK;
    }
}

/// Mark the vertices of a quad of `layer` when the item sheet draws it translucent
/// (`BakedQuad.MaterialInfo.of`: a sprite with translucent texels).
pub fn mark_layer(layer: crate::texture::sprite_alpha::Layer, vertices: &mut [Vertex]) {
    if layer == crate::texture::sprite_alpha::Layer::Translucent {
        mark(vertices);
    }
}

fn marked(vertex: &Vertex) -> bool {
    vertex.light[0] >= MARK / 2.0
}

/// The vertices of an item list split into the opaque triangles and the translucent quads (six
/// vertices each, unmarked), the quads farthest from `camera` first.
pub fn split(vertices: &[Vertex], camera: [f32; 3]) -> (Vec<Vertex>, Vec<Vertex>) {
    if !vertices.iter().any(marked) {
        return (vertices.to_vec(), Vec::new());
    }
    let mut opaque = Vec::with_capacity(vertices.len());
    let mut quads: Vec<[Vertex; 6]> = Vec::new();
    let mut index = 0;
    while index < vertices.len() {
        let rest = &vertices[index..];
        if marked(&rest[0]) && rest.len() >= 6 {
            let mut quad: [Vertex; 6] = std::array::from_fn(|corner| rest[corner]);
            for vertex in &mut quad {
                vertex.light[0] -= MARK;
            }
            quads.push(quad);
            index += 6;
        } else {
            let end = (index + 3).min(vertices.len());
            opaque.extend_from_slice(&vertices[index..end]);
            index = end;
        }
    }
    // `decodeQuadCentroids`: the middle of the first and third corners.
    let centroids: Vec<[f32; 3]> = quads
        .iter()
        .map(|quad| {
            std::array::from_fn(|axis| (quad[0].position[axis] + quad[2].position[axis]) / 2.0)
        })
        .collect();
    let translucent = crate::translucent::order(camera, &centroids)
        .into_iter()
        .flat_map(|quad| quads[quad])
        .collect();
    (opaque, translucent)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(x: f32, translucent: bool) -> Vec<Vertex> {
        let mut quad: Vec<Vertex> = (0..6)
            .map(|corner| Vertex {
                position: [x, corner as f32, 0.0],
                uv: [0.0; 2],
                shade: 1.0,
                tint: [1.0; 4],
                light: [240.0, 100.0],
            })
            .collect();
        if translucent {
            mark(&mut quad);
        }
        quad
    }

    #[test]
    fn marked_quads_split_off_farthest_first_and_lose_their_mark() {
        let mut list = quad(1.0, true);
        list.extend(quad(9.0, false));
        list.extend(quad(5.0, true));
        let (opaque, translucent) = split(&list, [0.0; 3]);
        assert_eq!(opaque.len(), 6);
        assert!(opaque.iter().all(|vertex| vertex.position[0] == 9.0));
        let order: Vec<f32> = translucent
            .iter()
            .map(|vertex| vertex.position[0])
            .collect();
        assert_eq!(order, [[5.0; 6], [1.0; 6]].concat());
        assert!(
            translucent
                .iter()
                .all(|vertex| vertex.light == [240.0, 100.0])
        );
    }

    #[test]
    fn an_unmarked_list_is_left_whole() {
        let list = quad(2.0, false);
        let (opaque, translucent) = split(&list, [0.0; 3]);
        assert_eq!(opaque.len(), 6);
        assert!(translucent.is_empty());
    }
}
