//! The beam between an end crystal and the dragon it heals or the tower it
//! stands on (`EnderDragonRenderer.submitCrystalBeams`): an eight-sided tube
//! from the one to the other, black at its start and white at its end, with
//! the beam's texture running along it.

use photon_core::math::{mth_cos_f64, mth_sin_f64};

use crate::entity::textures::{TextureRect, uv};
use crate::joml::{Pose, Quaternion};
use crate::scene::Vertex;

/// One vertex of the beam, placed: where, which texel, and how far from black
/// to white (the colour `0xFF000000` or `0xFFFFFFFF`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrystalBeamVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub white: bool,
}

/// The beam's quads in the order the reference submits them (four vertices
/// each), with the one normal they share.
#[derive(Debug, Clone, PartialEq)]
pub struct CrystalBeam {
    pub vertices: Vec<CrystalBeamVertex>,
    /// `Pose.transformNormal(0, -1, 0)`.
    pub normal: [f32; 3],
}

/// `Mth.sqrt(float)`.
fn sqrt(value: f32) -> f32 {
    f64::from(value).sqrt() as f32
}

/// `EnderDragonRenderer.submitCrystalBeams`, from the pose it is submitted at: the
/// vertices as the reference's vertex consumer receives them. `delta` runs from the
/// start of the beam to its end.
pub fn crystal_beam(mut pose: Pose, delta: [f32; 3], time_in_ticks: f32) -> CrystalBeam {
    let [dx, dy, dz] = delta;
    let horizontal_length = sqrt(dx * dx + dz * dz);
    let length = sqrt(dx * dx + dy * dy + dz * dz);
    let half_pi = (std::f64::consts::PI / 2.0) as f32;
    pose.translate(0.0, 2.0, 0.0);
    pose.rotate(&Quaternion::rotation_y(
        (-f64::from(dz).atan2(f64::from(dx))) as f32 - half_pi,
    ));
    pose.rotate(&Quaternion::rotation_x(
        (-f64::from(horizontal_length).atan2(f64::from(dy))) as f32 - half_pi,
    ));
    let v0 = 0.0 - time_in_ticks * 0.01;
    let v1 = length / 32.0 - time_in_ticks * 0.01;
    let two_pi = (std::f64::consts::PI * 2.0) as f32;
    let (mut last_sin, mut last_cos, mut last_u) = (0.0_f32, 0.75_f32, 0.0_f32);
    let mut vertices = Vec::new();
    for i in 1..=8 {
        let angle = i as f32 * two_pi / 8.0;
        let sin = mth_sin_f64(f64::from(angle)) * 0.75;
        let cos = mth_cos_f64(f64::from(angle)) * 0.75;
        let u = i as f32 / 8.0;
        let at = |x: f32, y: f32, z: f32, u: f32, v: f32, white: bool| CrystalBeamVertex {
            position: pose.pose.transform_position(x, y, z),
            uv: [u, v],
            white,
        };
        vertices.extend([
            at(last_sin * 0.2, last_cos * 0.2, 0.0, last_u, v0, false),
            at(last_sin, last_cos, length, last_u, v1, true),
            at(sin, cos, length, u, v1, true),
            at(sin * 0.2, cos * 0.2, 0.0, u, v0, false),
        ]);
        (last_sin, last_cos, last_u) = (sin, cos, u);
    }
    CrystalBeam {
        vertices,
        normal: pose.transform_normal([0.0, -1.0, 0.0]),
    }
}

/// A beam vertex's colour for the entity shader's tint: white is none (zero), black
/// is the shader's own negative marker, and anything between is `0xRRGGBB`.
fn tint_colour(grey: f32) -> f32 {
    if grey >= 1.0 {
        0.0
    } else if grey <= 0.0 {
        -1.0
    } else {
        let channel = ((grey * 255.0).round() as u32).min(255);
        ((channel << 16) | (channel << 8) | channel) as f32
    }
}

/// The beam as triangles for the entity shader, in the beam's own texture
/// (`RenderTypes.endCrystalBeam`: cut out, no overlay, lit by the world and by the
/// beam's one normal, never by which face shows: the shade is `-1` for that). The
/// reference's texture repeats along the beam, which the packed image does not,
/// so a quad longer than the texture is cut at each repeat, which draws the same
/// picture, the colour running from black to white along the cuts.
pub fn crystal_beam_triangles(
    beam: &CrystalBeam,
    texture: TextureRect,
    light: [f32; 2],
) -> Vec<Vertex> {
    let mut triangles = Vec::new();
    for quad in beam.vertices.as_chunks::<4>().0 {
        for corners in repeats(quad) {
            for index in [0, 1, 2, 0, 2, 3] {
                let (position, coordinate, grey) = corners[index];
                triangles.push(Vertex {
                    position,
                    uv: uv(texture, coordinate),
                    shade: -1.0,
                    tint: [
                        beam.normal[0],
                        beam.normal[1],
                        beam.normal[2],
                        tint_colour(grey),
                    ],
                    light,
                });
            }
        }
    }
    triangles
}

type Corner = ([f32; 3], [f32; 2], f32);

/// A quad cut at every whole texture coordinate its `v` crosses, each piece's `v`
/// made to start from its own repeat (a side runs along `v` from its first vertex
/// to its second, and from its fourth to its third).
fn repeats(quad: &[CrystalBeamVertex]) -> Vec<[Corner; 4]> {
    let grey = |vertex: CrystalBeamVertex| if vertex.white { 1.0 } else { 0.0 };
    let corner = |vertex: CrystalBeamVertex| (vertex.position, vertex.uv, grey(vertex));
    let [a, b, c, d] = [quad[0], quad[1], quad[2], quad[3]];
    let (v0, v1) = (a.uv[1], b.uv[1]);
    if v0 == v1 {
        return vec![[corner(a), corner(b), corner(c), corner(d)]];
    }
    let (low, high) = (v0.min(v1), v0.max(v1));
    // Where a long edge is at a `v`: its position and colour, by how far along it that is.
    let along = |from: CrystalBeamVertex, to: CrystalBeamVertex, v: f32| {
        let t = (v - v0) / (v1 - v0);
        let position = std::array::from_fn(|axis| {
            from.position[axis] + (to.position[axis] - from.position[axis]) * t
        });
        (position, grey(from) + (grey(to) - grey(from)) * t)
    };
    let mut pieces = Vec::new();
    let mut start = low;
    while start < high {
        let cell = start.floor();
        let end = (cell + 1.0).min(high);
        let (near, far) = if v0 >= v1 { (end, start) } else { (start, end) };
        let at = |(position, grey): ([f32; 3], f32), u: f32, v: f32| -> Corner {
            (position, [u, v - cell], grey)
        };
        pieces.push([
            at(along(a, b, near), a.uv[0], near),
            at(along(a, b, far), a.uv[0], far),
            at(along(d, c, far), d.uv[0], far),
            at(along(d, c, near), d.uv[0], near),
        ]);
        start = end;
    }
    pieces
}
