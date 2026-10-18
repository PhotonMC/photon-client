use super::*;

/// An entity model layer's quads as an item layer's mesh: corners in the
/// unit block, texture coordinates on the layer's sprite in the atlas, and
/// the layer's tint.
pub(super) fn model_mesh(
    layer: &crate::entity::special_models::ModelLayer,
    rect: &[f32; 4],
) -> Mesh {
    let tint = [
        ((layer.tint >> 16) & 0xFF) as f32 / 255.0,
        ((layer.tint >> 8) & 0xFF) as f32 / 255.0,
        (layer.tint & 0xFF) as f32 / 255.0,
        ((layer.tint >> 24) & 0xFF) as f32 / 255.0,
    ];
    let mut mesh = Mesh::default();
    for quad in &layer.quads {
        let base = mesh.vertices.len() as u32;
        let normal = quad[0].normal;
        for vertex in quad {
            mesh.vertices.push(crate::scene::BlockVertex::new(
                vertex.position,
                [
                    rect[0] + (rect[2] - rect[0]) * vertex.uv[0],
                    rect[1] + (rect[3] - rect[1]) * vertex.uv[1],
                ],
                tint,
                crate::scene::FULL_BRIGHT,
            ));
        }
        mesh.quads.push(crate::scene::QuadInfo {
            direction: crate::block::geometry::closest_direction(normal).unwrap_or(Direction::Up),
            cull: None,
            shade: true,
            tint_index: -1,
            // An entity model draws through its own render type.
            layer: crate::texture::sprite_alpha::Layer::Solid,
        });
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    mesh
}

/// A layer's `transformation` applied to its quads, in the unit block. The
/// quads keep facing outwards: a mirroring matrix turns each quad's corners
/// round, and the facing each quad is shaded by follows the matrix.
pub(super) fn apply_local(mesh: &mut Mesh, matrix: &crate::item_model::Matrix4) {
    if *matrix == crate::item_model::IDENTITY {
        return;
    }
    let determinant = {
        let m = matrix;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    for vertex in &mut mesh.vertices {
        let [x, y, z] = vertex.position;
        vertex.position = std::array::from_fn(|row| {
            matrix[row][0] * x + matrix[row][1] * y + matrix[row][2] * z + matrix[row][3]
        });
    }
    for (quad, info) in mesh.quads.iter_mut().enumerate() {
        let first = quad * 4;
        if determinant < 0.0 {
            mesh.vertices[first + 1..first + 4].reverse();
        }
        let normal = crate::block::geometry::triangle_normal(
            mesh.vertices[first].position,
            mesh.vertices[first + 1].position,
            mesh.vertices[first + 2].position,
        );
        if let Some(direction) = crate::block::geometry::closest_direction(normal) {
            info.direction = direction;
        }
    }
}
