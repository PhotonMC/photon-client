//! A guardian's beam (`GuardianRenderer.renderBeam`): a spinning prism from
//! its eye to what it aims at, drawn with the beam's texture, fully lit and
//! coloured from violet toward white as the attack charges.

use voxel_core::math::{mth_cos_f64, mth_sin_f64};

use crate::entity::textures::{TextureRect, uv};
use crate::joml::{Pose, Quaternion};
use crate::scene::Vertex;

/// One vertex of the beam's geometry, placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeamVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

/// The beam's quads in the order the reference submits them (four vertices
/// each: three sides, then the cap) with the colour and normal they share.
#[derive(Debug, Clone, PartialEq)]
pub struct Beam {
    pub vertices: Vec<BeamVertex>,
    /// The red, green and blue every vertex carries.
    pub colour: [i32; 3],
    /// `Pose.transformNormal(0, 1, 0)`: the one normal of every vertex.
    pub normal: [f32; 3],
}

/// The packed light every vertex of the beam carries (full bright).
pub const BEAM_LIGHT: i32 = 15_728_880;

/// `GuardianRenderer.renderBeam` from the pose the entity's feet and eye
/// leave (`vector` runs from the eye to the target): the vertices as the
/// reference's vertex consumer receives them.
pub fn guardian_beam(
    mut pose: Pose,
    vector: [f64; 3],
    time_in_ticks: f32,
    scale: f32,
    tex_v_off: f32,
) -> Beam {
    let length = ((vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt()
        + 1.0) as f32;
    // `Vec3.normalize`.
    let raw = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    let direction = if raw < 1.0E-5 {
        [0.0; 3]
    } else {
        [vector[0] / raw, vector[1] / raw, vector[2] / raw]
    };
    let x_rot = direction[1].acos() as f32;
    let y_rot = (std::f64::consts::PI / 2.0) as f32 - direction[2].atan2(direction[0]) as f32;
    let degrees = 180.0_f32 / std::f64::consts::PI as f32;
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        y_rot * degrees,
    )));
    pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
        x_rot * degrees,
    )));
    let rot = time_in_ticks * 0.05 * -1.5;
    let colour_scale = scale * scale;
    let colour = [
        64 + (colour_scale * 191.0) as i32,
        32 + (colour_scale * 191.0) as i32,
        128 - (colour_scale * 64.0) as i32,
    ];
    let pi = std::f64::consts::PI as f32;
    let cos = |angle: f32| mth_cos_f64(f64::from(rot + angle));
    let sin = |angle: f32| mth_sin_f64(f64::from(rot + angle));
    let three_quarters = (std::f64::consts::PI * 3.0 / 4.0) as f32;
    let quarter = (std::f64::consts::PI / 4.0) as f32;
    let five_quarters = pi * 5.0 / 4.0;
    let seven_quarters = pi * 7.0 / 4.0;
    let half = (std::f64::consts::PI / 2.0) as f32;
    let three_halves = (std::f64::consts::PI * 3.0 / 2.0) as f32;
    let (wnx, wnz) = (cos(three_quarters) * 0.282, sin(three_quarters) * 0.282);
    let (enx, enz) = (cos(quarter) * 0.282, sin(quarter) * 0.282);
    let (wsx, wsz) = (cos(five_quarters) * 0.282, sin(five_quarters) * 0.282);
    let (esx, esz) = (cos(seven_quarters) * 0.282, sin(seven_quarters) * 0.282);
    let (wx, wz) = (cos(pi) * 0.2, sin(pi) * 0.2);
    let (ex, ez) = (cos(0.0) * 0.2, sin(0.0) * 0.2);
    let (nx, nz) = (cos(half) * 0.2, sin(half) * 0.2);
    let (sx, sz) = (cos(three_halves) * 0.2, sin(three_halves) * 0.2);
    let top = length;
    let min_v = -1.0 + tex_v_off;
    let max_v = min_v + length * 2.5;
    let v_base = if (time_in_ticks.floor() as i32) % 2 == 0 {
        0.5
    } else {
        0.0
    };
    let vertex = |x: f32, y: f32, z: f32, u: f32, v: f32| BeamVertex {
        position: pose.pose.transform_position(x, y, z),
        uv: [u, v],
    };
    let vertices = vec![
        vertex(wx, top, wz, 0.4999, max_v),
        vertex(wx, 0.0, wz, 0.4999, min_v),
        vertex(ex, 0.0, ez, 0.0, min_v),
        vertex(ex, top, ez, 0.0, max_v),
        vertex(nx, top, nz, 0.4999, max_v),
        vertex(nx, 0.0, nz, 0.4999, min_v),
        vertex(sx, 0.0, sz, 0.0, min_v),
        vertex(sx, top, sz, 0.0, max_v),
        vertex(wnx, top, wnz, 0.5, v_base + 0.5),
        vertex(enx, top, enz, 1.0, v_base + 0.5),
        vertex(esx, top, esz, 1.0, v_base),
        vertex(wsx, top, wsz, 0.5, v_base),
    ];
    Beam {
        vertices,
        colour,
        normal: pose.transform_normal([0.0, 1.0, 0.0]),
    }
}

/// The beam as triangles for the entity shader, in the beam's own texture
/// (`RenderTypes.entityCutout`): each vertex carries the beam's normal, its
/// colour (`0xRRGGBB`) and full light. The reference's texture repeats, which
/// the packed image does not, so a quad longer than the texture is cut at each
/// repeat, which draws the same picture.
pub fn beam_triangles(beam: &Beam, texture: TextureRect) -> Vec<Vertex> {
    let [red, green, blue] = beam.colour;
    let colour = crate::scene::entity_colour(
        ((red as u32 & 0xFF) << 16) | ((green as u32 & 0xFF) << 8) | (blue as u32 & 0xFF),
    );
    let light = crate::block::light::light_uv(BEAM_LIGHT);
    let mut triangles = Vec::new();
    for quad in beam.vertices.chunks_exact(4) {
        for corners in repeats(quad) {
            for index in [0, 1, 2, 0, 2, 3] {
                let (position, coordinate) = corners[index];
                triangles.push(Vertex {
                    position,
                    uv: uv(texture, coordinate),
                    shade: 0.0,
                    tint: [beam.normal[0], beam.normal[1], beam.normal[2], colour],
                    light,
                });
            }
        }
    }
    triangles
}

type Corner = ([f32; 3], [f32; 2]);

/// A quad cut at every whole texture coordinate its `v` crosses, each piece's
/// `v` made to start from its own repeat. A side of the beam runs along `v`
/// from its first vertex to its second (and from its fourth to its third); the
/// cap does not and is kept as it is.
fn repeats(quad: &[BeamVertex]) -> Vec<[Corner; 4]> {
    let corner = |vertex: BeamVertex| (vertex.position, vertex.uv);
    let [a, b, c, d] = [quad[0], quad[1], quad[2], quad[3]];
    let along_v = a.uv[1] != b.uv[1] && a.uv[1] == d.uv[1] && b.uv[1] == c.uv[1];
    if !along_v {
        return vec![[corner(a), corner(b), corner(c), corner(d)]];
    }
    let (v0, v1) = (a.uv[1], b.uv[1]);
    let (low, high) = (v0.min(v1), v0.max(v1));
    // Where a long edge is at a `v`: its position, by how far along it that is.
    let along = |from: BeamVertex, to: BeamVertex, v: f32| {
        let t = (v - v0) / (v1 - v0);
        std::array::from_fn(|axis| {
            from.position[axis] + (to.position[axis] - from.position[axis]) * t
        })
    };
    let mut pieces = Vec::new();
    let mut start = low;
    while start < high {
        let cell = start.floor();
        let end = (cell + 1.0).min(high);
        let (near, far) = if v0 >= v1 { (end, start) } else { (start, end) };
        let at = |position: [f32; 3], u: f32, v: f32| -> Corner { (position, [u, v - cell]) };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_side_longer_than_the_texture_is_cut_at_each_repeat_and_covers_the_same_length() {
        let beam = guardian_beam(Pose::default(), [0.0, 0.0, 6.0], 4.0, 0.5, 0.25);
        let side = &beam.vertices[0..4];
        let pieces = repeats(side);
        // From -0.75 to about 17.25: eighteen or nineteen whole and part repeats.
        assert!(pieces.len() >= 17, "{}", pieces.len());
        let mut covered = 0.0;
        for piece in &pieces {
            for (_, coordinate) in piece {
                assert!((0.0..=1.0).contains(&coordinate[1]), "{coordinate:?}");
            }
            covered += piece[0].1[1] - piece[1].1[1];
        }
        assert!(
            (covered - (side[0].uv[1] - side[1].uv[1])).abs() < 1e-3,
            "{covered}"
        );
    }
}
