//! The rays that leave an ender dragon as it dies (`EnderDragonRenderer.submitRays`):
//! triangles from the dragon's middle, in a bundle that grows and turns with its death
//! time, each pose compounding the last.

use voxel_core::random::RandomSource;

use crate::joml::{Pose, Quaternion};

/// One vertex of the rays, placed, with its colour (`0xAARRGGBB`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayVertex {
    pub position: [f32; 3],
    pub colour: u32,
}

/// The colour at the end of a ray (no alpha: it fades out).
const OUTER_COLOUR: u32 = 16_711_935;

/// `ARGB.as8BitChannel`.
pub(crate) fn channel(value: f32) -> u32 {
    (value * 255.0).floor() as u32
}

/// `EnderDragonRenderer.submitRays` from the pose the dragon's death leaves, in the
/// order the reference's vertex consumer receives them.
pub fn dragon_rays(mut pose: Pose, death_time: f32) -> Vec<RayVertex> {
    let two_pi = (std::f64::consts::PI * 2.0) as f32;
    let half_pi = (std::f64::consts::PI / 2.0) as f32;
    let half_sqrt_3 = (3.0_f64.sqrt() / 2.0) as f32;
    let over_drive = (if death_time > 0.8 {
        (death_time - 0.8) / 0.2
    } else {
        0.0
    })
    .min(1.0);
    // `ARGB.colorFromFloat(1.0F - overDrive, 1.0F, 1.0F, 1.0F)`.
    let inner = (channel(1.0 - over_drive) << 24) | 0x00FF_FFFF;
    let mut random = RandomSource::create(432);
    let ray_count = ((death_time + death_time * death_time) / 2.0 * 60.0).floor() as i32;
    let mut vertices = Vec::new();
    for _ in 0..ray_count {
        let angles = [
            random.next_float() * two_pi,
            random.next_float() * two_pi,
            random.next_float() * two_pi,
        ];
        let mut rotation = Quaternion::rotation_xyz(angles[0], angles[1], angles[2]);
        let angles = [
            random.next_float() * two_pi,
            random.next_float() * two_pi,
            random.next_float() * two_pi + death_time * half_pi,
        ];
        rotation.rotate_xyz(angles[0], angles[1], angles[2]);
        pose.rotate(&rotation);
        let length = random.next_float() * 20.0 + 5.0 + over_drive * 10.0;
        let width = random.next_float() * 2.0 + 1.0 + over_drive * 2.0;
        let origin = [0.0, 0.0, 0.0];
        let outer_left = [-half_sqrt_3 * width, length, -0.5 * width];
        let outer_right = [half_sqrt_3 * width, length, -0.5 * width];
        let outer_bottom = [0.0, length, width];
        for (point, colour) in [
            (origin, inner),
            (outer_left, OUTER_COLOUR),
            (outer_right, OUTER_COLOUR),
            (origin, inner),
            (outer_right, OUTER_COLOUR),
            (outer_bottom, OUTER_COLOUR),
            (origin, inner),
            (outer_bottom, OUTER_COLOUR),
            (outer_left, OUTER_COLOUR),
        ] {
            vertices.push(RayVertex {
                position: pose.pose.transform_position(point[0], point[1], point[2]),
                colour,
            });
        }
    }
    vertices
}
