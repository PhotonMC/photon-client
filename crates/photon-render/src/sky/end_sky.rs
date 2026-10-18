//! The End's sky (`SkyRenderer.buildEndSky`): a cube of six dark grey quads around the camera,
//! the End sky texture repeated sixteen times over each.

use super::{Attributes, SkyBlend, SkyFrame, SkySprites, SkyVertex, SkyView, Skybox};
use crate::joml::{Pose, Quaternion};
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

/// The pose of the End flash's quad: turned about the vertical by `180 - y_angle` degrees and about
/// the side by `-90 - x_angle`, lifted a hundred blocks and stretched sixty wide.
pub fn end_flash_pose(x_angle: f32, y_angle: f32) -> Pose {
    let radians = Quaternion::degrees_to_radians;
    let mut pose = Pose::default();
    pose.rotate(&Quaternion::rotation_y(radians(180.0 - y_angle)));
    pose.rotate(&Quaternion::rotation_x(radians(-90.0 - x_angle)));
    pose.translate(0.0, 100.0, 0.0);
    pose.scale(60.0, 1.0, 60.0);
    pose
}

/// The End flash's quad (`SkyRenderer.buildCelestialQuad` of its sprite, `[u0, v0, u1, v1]`) as two
/// triangles, white at the flash's intensity in every channel and the alpha.
pub fn end_flash_vertices(
    (intensity, x_angle, y_angle): (f32, f32, f32),
    [u0, v0, u1, v1]: [f32; 4],
) -> Vec<SkyVertex> {
    let pose = end_flash_pose(x_angle, y_angle);
    let corners = [
        ([-1.0, 0.0, -1.0], [u0, v0]),
        ([1.0, 0.0, -1.0], [u1, v0]),
        ([1.0, 0.0, 1.0], [u1, v1]),
        ([-1.0, 0.0, 1.0], [u0, v1]),
    ];
    let vertex = |(corner, uv): ([f32; 3], [f32; 2])| SkyVertex {
        position: std::array::from_fn(|row| {
            let m = &pose.pose.m;
            m[0][row] * corner[0] + (m[1][row] * corner[1] + (m[2][row] * corner[2] + m[3][row]))
        }),
        uv,
        colour: [intensity; 4],
        fog_distance: [-1.0, -1.0],
    };
    [0, 1, 2, 0, 2, 3]
        .map(|index| vertex(corners[index]))
        .to_vec()
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
        let cube = frame.vertices.len() as u32;
        frame.batches = vec![(SkyBlend::EndSky, 0, cube)];
        // `SkyRenderer.renderEndFlash`: after the cube, while the flash is lit.
        if let Some((intensity, x_angle, y_angle)) = view.end_flash.filter(|flash| flash.0 > 1.0e-5)
        {
            let flash = end_flash_vertices((intensity, x_angle, y_angle), sprites.end_flash);
            frame
                .batches
                .push((SkyBlend::Overlay, cube, flash.len() as u32));
            frame.vertices.extend(flash);
        }
    }
    frame
}
