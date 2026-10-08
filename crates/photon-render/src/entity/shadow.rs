//! An entity's shadow (`ShadowFeatureRenderer`): one quad of the shadow
//! texture over each block square the world found for it, stretched so the
//! texture spans the shadow's diameter.

use photon_world::ShadowPiece;

use crate::scene::Vertex;

/// One corner as the renderer writes it: where (the pose's translation
/// applied), the colour (white with the alpha as 0 to 255) and the texture
/// coordinate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corner {
    pub position: [f32; 3],
    pub alpha: i32,
    pub uv: [f32; 2],
}

/// `ARGB.as8BitChannel`.
fn channel(value: f32) -> i32 {
    (value * 255.0).floor() as i32
}

/// `ShadowFeatureRenderer.prepare` for one piece: its four corners.
pub fn corners(piece: &ShadowPiece, radius: f32, translation: [f32; 3]) -> [Corner; 4] {
    let x01 = piece.relative[0] + piece.min[0];
    let x11 = piece.relative[0] + piece.max[0];
    let y01 = piece.relative[1] + piece.min[1];
    let z01 = piece.relative[2] + piece.min[2];
    let z11 = piece.relative[2] + piece.max[2];
    let u0 = -x01 / 2.0 / radius + 0.5;
    let u1 = -x11 / 2.0 / radius + 0.5;
    let v0 = -z01 / 2.0 / radius + 0.5;
    let v1 = -z11 / 2.0 / radius + 0.5;
    let alpha = channel(piece.alpha);
    let corner = |x: f32, z: f32, u: f32, v: f32| Corner {
        position: [x + translation[0], y01 + translation[1], z + translation[2]],
        alpha,
        uv: [u, v],
    };
    [
        corner(x01, z01, u0, v0),
        corner(x01, z11, u0, v1),
        corner(x11, z11, u1, v1),
        corner(x11, z01, u1, v0),
    ]
}

/// The pieces as triangles over `rect` of the packed entity textures. The
/// reference samples the shadow texture clamped to its edge, whose texels
/// are clear, so what lies outside the texture is left out and the rest
/// keeps its texture coordinates.
pub fn vertices(
    pieces: &[ShadowPiece],
    radius: f32,
    translation: [f32; 3],
    rect: crate::entity::textures::TextureRect,
    shift: [f32; 3],
) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for piece in pieces {
        let x01 = (piece.relative[0] + piece.min[0]).max(-radius);
        let x11 = (piece.relative[0] + piece.max[0]).min(radius);
        let z01 = (piece.relative[2] + piece.min[2]).max(-radius);
        let z11 = (piece.relative[2] + piece.max[2]).min(radius);
        if x01 >= x11 || z01 >= z11 {
            continue;
        }
        let y = piece.relative[1] + piece.min[1];
        let tint = [1.0, 1.0, 1.0, channel(piece.alpha) as f32 / 255.0];
        let corner = |x: f32, z: f32| Vertex {
            position: [
                x + translation[0] + shift[0],
                y + translation[1] + shift[1],
                z + translation[2] + shift[2],
            ],
            uv: crate::entity::textures::uv(
                rect,
                [-x / 2.0 / radius + 0.5, -z / 2.0 / radius + 0.5],
            ),
            shade: 0.0,
            tint,
            light: [0.0, 0.0],
        };
        let quad = [
            corner(x01, z01),
            corner(x01, z11),
            corner(x11, z11),
            corner(x11, z01),
        ];
        for index in [0, 1, 2, 0, 2, 3] {
            vertices.push(quad[index]);
        }
    }
    vertices
}
