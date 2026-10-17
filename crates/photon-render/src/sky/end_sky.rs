//! The End's sky (`SkyRenderer.buildEndSky`): a cube of six dark grey quads around the camera,
//! the End sky texture repeated sixteen times over each.

use super::{Attributes, SkyBlend, SkyFrame, SkySprites, SkyVertex, SkyView, Skybox};
use crate::joml::{cos_from_sin, sin};

/// Where the tint and the repeat are marked: the texture repeats, and there is no fog.
const REPEATING_NO_FOG: [f32; 2] = [-1.0, -2.0];

/// `Matrix4f.rotationX` and `rotationZ` as the quad `i` of the cube is turned (the first is not).
fn rotation(i: usize) -> [[f32; 3]; 3] {
    let half = std::f32::consts::FRAC_PI_2;
    let turn = |angle: f32, about_x: bool| {
        let sin = sin(angle);
        let cos = cos_from_sin(sin, angle);
        if about_x {
            [[1.0, 0.0, 0.0], [0.0, cos, sin], [0.0, -sin, cos]]
        } else {
            [[cos, sin, 0.0], [-sin, cos, 0.0], [0.0, 0.0, 1.0]]
        }
    };
    match i {
        1 => turn(half, true),
        2 => turn(-half, true),
        3 => turn(std::f32::consts::PI, true),
        4 => turn(half, false),
        5 => turn(-half, false),
        _ => [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    }
}

/// `Matrix4f.transformPosition`, column-major (`m[column][row]`), without a translation.
fn transform(m: &[[f32; 3]; 3], [x, y, z]: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|row| m[0][row] * x + (m[1][row] * y + (m[2][row] * z + 0.0)))
}

/// The cube's quads as triangles, six vertices a quad.
pub fn end_sky_vertices() -> Vec<SkyVertex> {
    // `-14145496`: opaque, 40 of 255 in every channel.
    let grey = 40.0 / 255.0;
    let corners = [
        ([-100.0, -100.0, -100.0], [0.0, 0.0]),
        ([-100.0, -100.0, 100.0], [0.0, 16.0]),
        ([100.0, -100.0, 100.0], [16.0, 16.0]),
        ([100.0, -100.0, -100.0], [16.0, 0.0]),
    ];
    let mut vertices = Vec::with_capacity(36);
    for i in 0..6 {
        let pose = rotation(i);
        let vertex = |(position, uv): ([f32; 3], [f32; 2])| SkyVertex {
            position: transform(&pose, position),
            uv,
            colour: [grey, grey, grey, 1.0],
            fog_distance: REPEATING_NO_FOG,
        };
        for index in [0, 1, 2, 0, 2, 3] {
            vertices.push(vertex(corners[index]));
        }
    }
    vertices
}

/// One frame of the dimension's sky: the overworld's shapes, or the End's cube, none where the
/// camera is out of the sky's sight (`LevelRenderer.addSkyPass`).
pub fn build_sky_frame(
    attributes: &Attributes,
    view: &SkyView,
    stars: &[[[f32; 3]; 4]],
    sprites: &SkySprites,
    skybox: Skybox,
) -> SkyFrame {
    let mut frame = super::build_frame(attributes, view, stars, sprites, skybox);
    if skybox == Skybox::End && !view.fog.hides_sky() {
        frame.vertices = end_sky_vertices();
        frame.batches = vec![(SkyBlend::EndSky, 0, frame.vertices.len() as u32)];
    }
    frame
}
