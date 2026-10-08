//! A particle as the quad the renderer writes: `SingleQuadParticle.extract` and the vertices
//! `QuadParticleRenderState` makes of it.

use photon_core::math::{Vec3, floor_f32};
use photon_world::particles::Particle;

use crate::joml::Quaternion;

/// One vertex of a particle quad, as the renderer writes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadVertex {
    /// Where, from the camera.
    pub position: [f32; 3],
    pub uv: [f32; 2],
    /// `ARGB.colorFromFloat`.
    pub color: i32,
    /// The packed light.
    pub light: i32,
}

/// The camera a particle is extracted for: how it is turned, and where it is.
#[derive(Debug, Clone, Copy)]
pub struct ParticleCamera {
    pub orientation: Quaternion,
    pub position: Vec3,
}

/// `ARGB.as8BitChannel`.
fn channel(value: f32) -> i32 {
    floor_f32(value * 255.0)
}

/// `ARGB.colorFromFloat`.
pub fn color_from_float([red, green, blue, alpha]: [f32; 4]) -> i32 {
    (channel(alpha) << 24) | (channel(red) << 16) | (channel(green) << 8) | channel(blue)
}

/// `QuadParticleRenderState.renderRotatedQuad`: the four corners of a quad turned by `rotation`,
/// `scale` blocks to the corner, about `at`, over the sprite's `uvs` (`u0`, `u1`, `v0`, `v1`).
pub fn quad(
    rotation: &Quaternion,
    (at, scale): ([f32; 3], f32),
    uvs: [f32; 4],
    (color, light): (i32, i32),
) -> [QuadVertex; 4] {
    let [u0, u1, v0, v1] = uvs;
    let corner = |nx: f32, ny: f32, u: f32, v: f32| {
        let turned = rotation.transform([nx, ny, 0.0]);
        QuadVertex {
            position: [
                turned[0] * scale + at[0],
                turned[1] * scale + at[1],
                turned[2] * scale + at[2],
            ],
            uv: [u, v],
            color,
            light,
        }
    };
    [
        corner(1.0, -1.0, u1, v1),
        corner(1.0, 1.0, u1, v0),
        corner(-1.0, 1.0, u0, v0),
        corner(-1.0, -1.0, u0, v1),
    ]
}

/// `SingleQuadParticle.extract` with `FacingCameraMode.LOOKAT_XYZ`: the quad of a particle at a
/// partial tick, over the sprite's `uvs`, lit by `light`.
pub fn extract(
    particle: &Particle,
    camera: &ParticleCamera,
    partial: f32,
    (uvs, light): ([f32; 4], i32),
) -> [QuadVertex; 4] {
    let mut rotation = camera.orientation;
    let (previous_roll, roll) = particle.roll();
    if roll != 0.0 {
        rotation = rotation.rotate_z(previous_roll + partial * (roll - previous_roll));
    }
    let (position, previous) = (particle.position(), particle.previous_position());
    let blend = f64::from(partial);
    let relative = |before: f64, now: f64, eye: f64| (before + blend * (now - before) - eye) as f32;
    let at = [
        relative(previous.x, position.x, camera.position.x),
        relative(previous.y, position.y, camera.position.y),
        relative(previous.z, position.z, camera.position.z),
    ];
    quad(
        &rotation,
        (at, particle.quad_size_at(partial)),
        uvs,
        (color_from_float(particle.colour_at(partial)), light),
    )
}
