//! An experience orb (`ExperienceOrbRenderer`): a quad that always faces
//! the camera, whose texel is the icon its value picks and whose colour
//! sways with its age.

use voxel_core::math::mth_sin_f64;

use crate::joml::{Pose, Quaternion};
use crate::scene::Vertex;

/// `ExperienceOrb.getIcon`: the sprite a value is drawn with.
pub fn icon(value: i32) -> i32 {
    match value {
        2477.. => 10,
        1237.. => 9,
        617.. => 8,
        307.. => 7,
        149.. => 6,
        73.. => 5,
        37.. => 4,
        17.. => 3,
        7.. => 2,
        3.. => 1,
        _ => 0,
    }
}

/// One corner as the renderer writes it: where, the colour (red, green,
/// blue, alpha as 0 to 255), the texture coordinate and the packed light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corner {
    pub position: [f32; 3],
    pub colour: [i32; 4],
    pub uv: [f32; 2],
    /// `setNormal(pose, 0, 1, 0)`: the quad's up, turned by the camera's
    /// orientation, which the entity shader lights it by.
    pub normal: [f32; 3],
}

/// `ExperienceOrbRenderer.submit`: the four corners of the quad. `placement`
/// is where the orb stands from the origin the vertices are measured from,
/// and `orientation` the camera's.
pub fn corners(
    icon: i32,
    age_in_ticks: f32,
    placement: [f32; 3],
    orientation: &Quaternion,
) -> [Corner; 4] {
    let u0 = (icon % 4 * 16) as f32 / 64.0;
    let u1 = (icon % 4 * 16 + 16) as f32 / 64.0;
    let v0 = (icon / 4 * 16) as f32 / 64.0;
    let v1 = (icon / 4 * 16 + 16) as f32 / 64.0;
    let rr = age_in_ticks / 2.0;
    let sin = |angle: f32| mth_sin_f64(f64::from(angle));
    let red = ((sin(rr + 0.0) + 1.0) * 0.5 * 255.0) as i32;
    let blue = ((sin(rr + (std::f64::consts::PI * 4.0 / 3.0) as f32) + 1.0) * 0.1 * 255.0) as i32;
    let mut pose = Pose::default();
    pose.translate(placement[0], placement[1], placement[2]);
    pose.translate(0.0, 0.1, 0.0);
    pose.rotate(orientation);
    pose.scale(0.3, 0.3, 0.3);
    let normal = pose.transform_normal([0.0, 1.0, 0.0]);
    let corner = |x: f32, y: f32, u: f32, v: f32| Corner {
        position: pose.pose.transform_position(x, y, 0.0),
        colour: [red, 255, blue, 128],
        uv: [u, v],
        normal,
    };
    [
        corner(-0.5, -0.25, u0, v1),
        corner(0.5, -0.25, u1, v1),
        corner(0.5, 0.75, u1, v0),
        corner(-0.5, 0.75, u0, v0),
    ]
}

/// `minecraft_mix_light` for the level's two diffuse lights: how much of the
/// colour a face turned this way keeps.
fn diffuse_light(normal: [f32; 3]) -> f32 {
    let direction = |x: f32, y: f32, z: f32| {
        let length = (x * x + y * y + z * z).sqrt();
        [x / length, y / length, z / length]
    };
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let first = dot(direction(0.2, 1.0, -0.7), normal).max(0.0);
    let second = dot(direction(-0.2, 1.0, 0.7), normal).max(0.0);
    ((first + second) * 0.6 + 0.4).min(1.0)
}

/// The quad as two triangles, from `rect` (where the orb's texture sits in
/// the packed entity textures), shifted by `shift` and lit by `light`. The
/// colour carries the diffuse light, which the reference's vertex shader
/// folds into it.
pub fn vertices(
    corners: &[Corner; 4],
    rect: crate::entity::textures::TextureRect,
    shift: [f32; 3],
    light: i32,
) -> Vec<Vertex> {
    let light = crate::block::light::light_uv(light);
    [0, 1, 2, 0, 2, 3]
        .into_iter()
        .map(|index| {
            let corner = corners[index];
            let diffuse = diffuse_light(corner.normal);
            let [red, green, blue, alpha] = corner.colour.map(|channel| channel as f32 / 255.0);
            Vertex {
                position: [
                    corner.position[0] + shift[0],
                    corner.position[1] + shift[1],
                    corner.position[2] + shift[2],
                ],
                uv: crate::entity::textures::uv(rect, corner.uv),
                shade: 1.0,
                tint: [red * diffuse, green * diffuse, blue * diffuse, alpha],
                light,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ExperienceOrb.getIcon`'s thresholds: each value that starts an icon
    /// and the one before it.
    #[test]
    fn a_value_picks_the_icon_whose_threshold_it_reaches() {
        let starts = [3, 7, 17, 37, 73, 149, 307, 617, 1237, 2477];
        assert_eq!(icon(0), 0);
        assert_eq!(icon(2), 0);
        for (index, start) in starts.into_iter().enumerate() {
            assert_eq!(icon(start), index as i32 + 1, "{start}");
            assert_eq!(icon(start - 1), index as i32, "{}", start - 1);
        }
        assert_eq!(icon(i32::MAX), 10);
    }
}
