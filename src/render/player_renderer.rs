//! Drawing a player in the world: which default skin it wears, and how its
//! posed model is placed where it stands.
//!
//! The placement is the reference's living-entity renderer for a player:
//! turned to face away from its body's facing, tipped towards the swimming
//! pose, flipped upright, scaled to fifteen sixteenths and lowered so its
//! feet meet the ground; a crouching player also sinks two sixteenths.

use crate::binary::Uuid;
use crate::render::joml::{Pose, Quaternion};
use crate::render::player_model::PlayerModel;
use crate::render::scene::Vertex;

/// A default skin: its texture, as a path under `textures/`, and whether it
/// has slim arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultSkin {
    pub texture: &'static str,
    pub slim: bool,
}

/// `DefaultPlayerSkin`'s list, in its order.
const DEFAULT_SKINS: [DefaultSkin; 18] = {
    const fn skin(texture: &'static str, slim: bool) -> DefaultSkin {
        DefaultSkin { texture, slim }
    }
    [
        skin("entity/player/slim/alex", true),
        skin("entity/player/slim/ari", true),
        skin("entity/player/slim/efe", true),
        skin("entity/player/slim/kai", true),
        skin("entity/player/slim/makena", true),
        skin("entity/player/slim/noor", true),
        skin("entity/player/slim/steve", true),
        skin("entity/player/slim/sunny", true),
        skin("entity/player/slim/zuri", true),
        skin("entity/player/wide/alex", false),
        skin("entity/player/wide/ari", false),
        skin("entity/player/wide/efe", false),
        skin("entity/player/wide/kai", false),
        skin("entity/player/wide/makena", false),
        skin("entity/player/wide/noor", false),
        skin("entity/player/wide/steve", false),
        skin("entity/player/wide/sunny", false),
        skin("entity/player/wide/zuri", false),
    ]
};

/// `DefaultPlayerSkin.get(UUID)`: picked by the id's hash.
pub fn default_skin(id: Uuid) -> DefaultSkin {
    // `UUID.hashCode`: the two halves folded together, then the two words.
    let folded = id.most ^ id.least;
    let hash = ((folded >> 32) as i32) ^ (folded as i32);
    DEFAULT_SKINS[hash.rem_euclid(DEFAULT_SKINS.len() as i32) as usize]
}

/// Where and how a posed player model is placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The feet, from the mesh origin.
    pub feet: [f32; 3],
    /// The body's facing, in degrees.
    pub body_rot: f32,
    /// The pitch, in degrees, which tips a swimmer.
    pub x_rot: f32,
    pub swim_amount: f32,
    pub in_water: bool,
    /// In the swimming pose (in water or crawling).
    pub visually_swimming: bool,
    pub crouching: bool,
    /// Packed block and sky light at the player.
    pub light: i32,
}

/// The pose the model is drawn from: `LivingEntityRenderer.submit` for a
/// player standing on its feet.
pub fn placement_pose(placement: &Placement) -> Pose {
    let mut pose = Pose::default();
    // `AvatarRenderer.getRenderOffset`: a crouching player sits lower.
    let lower = if placement.crouching {
        -2.0 / 16.0
    } else {
        0.0
    };
    pose.translate(
        placement.feet[0],
        placement.feet[1] + lower,
        placement.feet[2],
    );
    pose.scale(1.0, 1.0, 1.0);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        180.0 - placement.body_rot,
    )));
    // `AvatarRenderer.setupRotations`: the swimmer tips forward.
    if placement.swim_amount > 0.0 {
        let target = if placement.in_water {
            -90.0 - placement.x_rot
        } else {
            -90.0
        };
        let angle = 0.0 + placement.swim_amount * (target - 0.0);
        pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
            angle,
        )));
        if placement.visually_swimming {
            pose.translate(0.0, -1.0, 0.3);
        }
    }
    pose.scale(-1.0, -1.0, 1.0);
    pose.scale(0.9375, 0.9375, 0.9375);
    pose.translate(0.0, -1.501, 0.0);
    pose
}

/// The posed model's triangles, placed, each vertex carrying its normal in
/// its tint for the entity shader.
pub fn player_vertices(model: &PlayerModel, placement: &Placement) -> Vec<Vertex> {
    let pose = placement_pose(placement);
    let light = crate::render::block_light::light_uv(placement.light);
    let mut vertices = Vec::new();
    for (_, corners) in model.render(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: corner.uv,
                shade: 1.0,
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], 1.0],
                light,
            });
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_skin_follows_the_id_hash() {
        // The zero id hashes to zero: the first slim skin.
        assert_eq!(
            default_skin(Uuid { most: 0, least: 0 }).texture,
            "entity/player/slim/alex"
        );
        // A hash of seventeen picks the last wide skin.
        assert_eq!(
            default_skin(Uuid { most: 0, least: 17 }).texture,
            "entity/player/wide/zuri"
        );
    }

    #[test]
    fn a_standing_player_reaches_from_the_ground_to_its_head() {
        let mut model = PlayerModel::new(false);
        model.setup_anim(&Default::default());
        let vertices = player_vertices(
            &model,
            &Placement {
                feet: [0.0; 3],
                body_rot: 0.0,
                x_rot: 0.0,
                swim_amount: 0.0,
                in_water: false,
                visually_swimming: false,
                crouching: false,
                light: 0,
            },
        );
        let lowest = vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::MAX, f32::min);
        let highest = vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::MIN, f32::max);
        // The trousers' overlay, on legs splayed by a few thousandths of a
        // radian, reaches a hair below the ground; the hat reaches
        // (8.5 / 16 + 1.501) * 15 / 16 above it.
        assert!(lowest > -0.02 && lowest < 0.0, "{lowest}");
        assert!(
            (highest - (0.531_25 + 1.501) * 0.9375).abs() < 1e-5,
            "{highest}"
        );
    }
}
