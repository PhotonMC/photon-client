//! The flames a burning entity wears (`FlameFeatureRenderer`): upright
//! quads of the fire sprites, stacked up the entity's height and turned to
//! face the camera's heading.

use crate::block::light::light_uv;
use crate::joml::{Pose, Quaternion};
use crate::objects::item::entity_light;
use crate::scene::Vertex;

/// What the flames of one entity are drawn from.
pub struct Flame {
    /// Where the entity is, from the mesh origin, with its render offset.
    pub origin: [f32; 3],
    /// `getBbWidth` and `getBbHeight`.
    pub size: (f32, f32),
    /// The light at the entity, packed (`block << 4 | sky << 20`).
    pub light: i32,
    /// The camera's turn about the vertical axis ([`camera_twist`]).
    pub twist: Quaternion,
}

/// `Mth.rotationAroundAxis` about the vertical axis: the part of the camera's orientation that
/// turns about it.
pub fn camera_twist(orientation: &Quaternion) -> Quaternion {
    let length_squared = orientation.y * orientation.y + orientation.w * orientation.w;
    let inverse = 1.0 / length_squared.sqrt();
    Quaternion {
        x: 0.0,
        y: orientation.y * inverse,
        z: 0.0,
        w: orientation.w * inverse,
    }
}

/// The triangles of the flames; `fire` is the two sprites (`block/fire_0` and `block/fire_1`) as
/// `[u0, v0, u1, v1]` in the block atlas. The light is the entity's with the block light at
/// its brightest, the normal is straight up and no colour or overlay touches the sprite.
pub fn flame_vertices(flame: &Flame, fire: [[f32; 4]; 2]) -> Vec<Vertex> {
    let (width, height) = flame.size;
    let scale = width * 1.4;
    let mut remaining = height / scale;
    let mut pose = Pose::default();
    pose.translate(flame.origin[0], flame.origin[1], flame.origin[2]);
    pose.scale(scale, scale, scale);
    pose.rotate(&flame.twist);
    pose.translate(0.0, 0.0, 0.3 - (remaining as i32) as f32 * 0.02);
    let light = light_uv((flame.light & !0xFFFF) | (15 << 4));
    let shade = entity_light([0.0, 1.0, 0.0]);
    let (mut radius, mut down, mut depth) = (0.5_f32, 0.0_f32, 0.0_f32);
    let mut vertices = Vec::new();
    let mut stage = 0;
    while remaining > 0.0 {
        let [mut u0, v0, mut u1, v1] = fire[stage % 2];
        if stage / 2 % 2 == 0 {
            std::mem::swap(&mut u0, &mut u1);
        }
        let corners = [
            (-radius, 0.0 - down, u1, v1),
            (radius, 0.0 - down, u0, v1),
            (radius, 1.4 - down, u0, v0),
            (-radius, 1.4 - down, u1, v0),
        ];
        for index in [0, 1, 2, 0, 2, 3] {
            let (x, y, u, v) = corners[index];
            vertices.push(Vertex {
                position: pose.pose.transform_position(x, y, depth),
                uv: [u, v],
                shade,
                tint: [1.0; 4],
                light,
            });
        }
        remaining -= 0.45;
        down -= 0.45;
        radius *= 0.9;
        depth -= 0.03;
        stage += 1;
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zombie() -> Flame {
        Flame {
            origin: [0.0; 3],
            size: (0.6, 1.95),
            light: 14 << 20,
            twist: Quaternion::identity(),
        }
    }

    const FIRE: [[f32; 4]; 2] = [[0.0, 0.0, 0.25, 0.25], [0.5, 0.5, 0.75, 0.75]];

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn a_zombie_wears_six_stacked_quads() {
        // Scale 0.84, so 1.95 / 0.84 = 2.32 blocks of flame, 0.45 less for each quad.
        assert_eq!(flame_vertices(&zombie(), FIRE).len(), 6 * 6);
    }

    #[test]
    fn the_first_quad_sits_in_front_of_the_origin_at_the_scale() {
        let vertices = flame_vertices(&zombie(), FIRE);
        // (int) 2.32 = 2, so the quads are moved 0.3 - 0.04 = 0.26 and scaled by 0.84.
        let [x, y, z] = vertices[0].position;
        assert!(near(x, -0.42) && near(y, 0.0) && near(z, 0.26 * 0.84));
        let [_, top, _] = vertices[2].position;
        assert!(near(top, 1.4 * 0.84));
    }

    #[test]
    fn each_quad_is_lower_narrower_and_further_back() {
        let vertices = flame_vertices(&zombie(), FIRE);
        // The sixth: radius 0.5 * 0.9^5, down by 5 * 0.45, back by 5 * 0.03.
        let [x, y, z] = vertices[5 * 6].position;
        assert!(near(x, -0.5 * 0.9_f32.powi(5) * 0.84));
        assert!(near(y, 2.25 * 0.84));
        assert!(near(z, (0.26 - 0.15) * 0.84));
    }

    #[test]
    fn the_sprites_alternate_and_every_other_pair_mirrors() {
        let vertices = flame_vertices(&zombie(), FIRE);
        // The first two pairs of quads swap `u0` and `u1`, the next two do not; the corner at the
        // bottom left takes `u1`.
        assert_eq!(vertices[0].uv, [0.0, 0.25]);
        assert_eq!(vertices[6].uv, [0.5, 0.75]);
        assert_eq!(vertices[12].uv, [0.25, 0.25]);
        assert_eq!(vertices[18].uv, [0.75, 0.75]);
    }

    #[test]
    fn the_block_light_is_at_its_brightest_and_the_sky_light_is_kept() {
        let vertices = flame_vertices(&zombie(), FIRE);
        assert_eq!(vertices[0].light, [240.0, 224.0]);
    }

    #[test]
    fn the_camera_twist_keeps_only_the_turn_about_the_vertical_axis() {
        let turned = Quaternion::rotation_yxz(1.0, 0.5, 0.0);
        let twist = camera_twist(&turned);
        assert!(twist.x == 0.0 && twist.z == 0.0);
        assert!(near(twist.y * twist.y + twist.w * twist.w, 1.0));
    }
}
