//! Drawing a player in the world: which default skin it wears, and how its
//! posed model is placed where it stands.
//!
//! The placement is the reference's living-entity renderer for a player:
//! turned to face away from its body's facing, tipped towards the swimming
//! pose, flipped upright, scaled to fifteen sixteenths and lowered so its
//! feet meet the ground; a crouching player also sinks two sixteenths.

pub use super::gui_view::*;
use crate::entity::player_model::PlayerModel;
use crate::joml::{Pose, Quaternion};
use crate::scene::Vertex;
use photon_core::binary::Uuid;

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

/// Which of the default skins a profile id wears (`DefaultPlayerSkin.get`
/// picks by the id's hash).
pub fn skin_index(id: Uuid) -> usize {
    // `UUID.hashCode`: the two halves folded together, then the two words.
    let folded = id.most ^ id.least;
    let hash = ((folded >> 32) as i32) ^ (folded as i32);
    hash.rem_euclid(DEFAULT_SKINS.len() as i32) as usize
}

/// `DefaultPlayerSkin.get(UUID)`: picked by the id's hash.
pub fn default_skin(id: Uuid) -> DefaultSkin {
    DEFAULT_SKINS[skin_index(id)]
}

/// The default skin at a place in the list.
pub fn skin_at(index: usize) -> DefaultSkin {
    DEFAULT_SKINS[index]
}

/// The paths of the default skins' textures (under `textures/`).
pub fn skin_textures() -> impl Iterator<Item = &'static str> {
    DEFAULT_SKINS.iter().map(|skin| skin.texture)
}

/// A gliding player (`AvatarRenderState.isFallFlying`): how far it has tipped forward, and which
/// way the flight turns it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FallFlying {
    /// `fallFlyingScale`: the tip grows with the time spent gliding.
    pub scale: f32,
    /// `flyingYRot`, in radians, when the movement and the look both have a heading.
    pub y_rot: Option<f32>,
}

pub use super::sleep::Sleep;

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
    /// Where the skin the model is drawn in is in the entity textures.
    pub skin: crate::entity::textures::TextureRect,
    /// `LivingEntityRenderState.deathTime`: zero while it lives.
    pub death_time: f32,
    /// Drawn with the hurt overlay (`hasRedOverlay`).
    pub red_overlay: bool,
    /// Gliding, which tips the player forward in place of swimming.
    pub fall_flying: Option<FallFlying>,
    /// `isAutoSpinAttack` (a riptide): the age in ticks the player spins by.
    pub auto_spin: Option<f32>,
    /// `isUpsideDown`: the height of the bounding box of a player named Dinnerbone or Grumm.
    pub upside_down: Option<f32>,
    /// The pose of a player in a bed (`Pose.SLEEPING`).
    pub sleeping: Option<Sleep>,
}

/// What `LivingEntityRenderer.setupRotations` does after the turn to face away: the fall of a
/// dying player, else a riptide spin, else the lying of a sleeper, else the flip of one named
/// Dinnerbone or Grumm.
fn fall_or_turn(pose: &mut Pose, placement: &Placement) {
    let degrees = Quaternion::degrees_to_radians;
    if placement.death_time > 0.0 {
        if let Some(roll) = crate::entity::mob_model::death_roll(placement.death_time, 90.0) {
            pose.rotate(&Quaternion::rotation_z(degrees(roll)));
        }
    } else if let Some(age) = placement.auto_spin {
        pose.rotate(&Quaternion::rotation_x(degrees(-90.0 - placement.x_rot)));
        pose.rotate(&Quaternion::rotation_y(degrees(age * -75.0)));
    } else if let Some(sleep) = placement.sleeping {
        pose.rotate(&Quaternion::rotation_y(degrees(sleep.angle)));
        pose.rotate(&Quaternion::rotation_z(degrees(90.0)));
        pose.rotate(&Quaternion::rotation_y(degrees(270.0)));
    } else if let Some(height) = placement.upside_down {
        pose.translate(0.0, height + 0.1, 0.0);
        pose.rotate(&Quaternion::rotation_z(degrees(180.0)));
    }
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
    if let Some(sleep) = placement.sleeping {
        pose.translate(sleep.shift[0], 0.0, sleep.shift[1]);
    }
    pose.scale(1.0, 1.0, 1.0);
    if placement.sleeping.is_none() {
        pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
            180.0 - placement.body_rot,
        )));
    }
    fall_or_turn(&mut pose, placement);
    // `AvatarRenderer.setupRotations`: a glider tips forward, else a swimmer does.
    if let Some(flying) = placement.fall_flying {
        if placement.auto_spin.is_none() {
            pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
                flying.scale * (-90.0 - placement.x_rot),
            )));
        }
        if let Some(y_rot) = flying.y_rot {
            pose.rotate(&Quaternion::rotation_y(y_rot));
        }
    } else if placement.swim_amount > 0.0 {
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
    let light = crate::block::light::light_uv(placement.light);
    let mut vertices = Vec::new();
    for corners in model.quads(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: crate::entity::textures::uv(placement.skin, corner.uv),
                shade: f32::from(placement.red_overlay),
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], 0.0],
                light,
            });
        }
    }
    vertices
}

/// [`player_vertices`] for the ghost of an invisible player a spectator sees
/// (`forceTransparent`, the colour `0x26FFFFFF`): blended, and sorted far to near from the camera.
pub fn player_ghost_vertices(
    model: &PlayerModel,
    placement: &Placement,
    camera: [f32; 3],
) -> Vec<Vertex> {
    let mut vertices = player_vertices(model, placement);
    for vertex in &mut vertices {
        vertex.shade = crate::entity::mob_model::BLENDED_SHADE + f32::from(GHOST_ALPHA);
    }
    crate::entity::mob_model::sort_translucent_quads(&mut vertices, camera);
    vertices
}

/// The alpha byte of the ghost's colour `0x26FFFFFF`.
const GHOST_ALPHA: u8 = 0x26;

/// Another model of the wearer's (an armor model), placed as the player is,
/// in its own texture and multiplied by a colour (`0xRRGGBB`).
pub fn layer_vertices(
    model: &crate::entity::mob_model::MobModel,
    placement: &Placement,
    texture: crate::entity::textures::TextureRect,
    colour: Option<u32>,
) -> Vec<Vertex> {
    let pose = placement_pose(placement);
    let light = crate::block::light::light_uv(placement.light);
    let colour = colour.map_or(0.0, crate::scene::entity_colour);
    let mut vertices = Vec::new();
    for corners in model.quads(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: crate::entity::textures::uv(texture, corner.uv),
                shade: crate::entity::mob_model::overlay_code(placement.red_overlay, 0.0),
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], colour],
                light,
            });
        }
    }
    vertices
}

/// What places the first-person hand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandView {
    /// The view bob's walked distance and bob.
    pub bob_walked: f32,
    pub bob: f32,
    /// The view's pitch and yaw, and the hand's lagging copies of them.
    pub view_x_rot: f32,
    pub view_y_rot: f32,
    pub x_bob: f32,
    pub y_bob: f32,
    /// How far the hand has dropped out of view, 0 when fully up.
    pub inverse_arm_height: f32,
    /// How far through a swing the hand is.
    pub attack: f32,
    pub light: i32,
    /// Where the skin the arm is drawn in is in the entity textures.
    pub skin: crate::entity::textures::TextureRect,
    /// The blow the view tilts by, as `GameRenderer.bobHurt` does the hand's pose first.
    pub hurt: crate::camera::HurtView,
}

/// The pose both hands start from: `GameRenderer.bobView`, then the hands'
/// lag behind the view.
fn view_pose(view: &HandView) -> Pose {
    use photon_core::math::{mth_cos_f64, mth_sin_f64};
    let rad = Quaternion::degrees_to_radians;
    let sin = |value: f32| mth_sin_f64(f64::from(value));
    let cos = |value: f32| mth_cos_f64(f64::from(value));
    let pi = std::f64::consts::PI as f32;
    let mut pose = Pose::default();
    crate::camera::apply_hurt(&mut pose, &view.hurt);
    // `GameRenderer.bobView` on the hand's pose.
    let angle = view.bob_walked * pi;
    pose.translate(
        sin(angle) * view.bob * 0.5,
        -(cos(angle) * view.bob).abs(),
        0.0,
    );
    pose.rotate(&Quaternion::rotation_z(rad(sin(angle) * view.bob * 3.0)));
    pose.rotate(&Quaternion::rotation_x(rad((cos(angle - 0.2) * view.bob)
        .abs()
        * 5.0)));
    // The hand trails the view.
    pose.rotate(&Quaternion::rotation_x(rad(
        (view.view_x_rot - view.x_bob) * 0.1
    )));
    pose.rotate(&Quaternion::rotation_y(rad(
        (view.view_y_rot - view.y_bob) * 0.1
    )));
    pose
}

/// A held item's triangles, in view space: `submitArmWithItem` for a stack
/// that is not a map, with the motion the hand makes with it, then
/// `ItemTransform.apply` in the first-person context and the item's quads.
/// Each vertex's shade is the entity lights on its turned normal, as the
/// arm's shader lights it; its tint is the layer's.
pub fn held_item_vertices(
    view: &HandView,
    layers: &[crate::block::world_mesh::ItemLayer],
    (right_arm, attack, motion): (bool, f32, crate::entity::hand_motion::HandMotion),
    inverse_arm_height: f32,
) -> Vec<Vertex> {
    let invert = if right_arm { 1.0_f32 } else { -1.0 };
    let mut pose = view_pose(view);
    crate::entity::hand_motion::apply(&mut pose, motion, (invert, attack, inverse_arm_height));
    let light = crate::block::light::light_uv(view.light);
    let mut vertices = Vec::new();
    for layer in layers {
        let mut pose = pose;
        // `ItemTransform.apply`, mirrored for the left hand.
        if let Some(transform) = layer.transform {
            let [rx, mut ry, mut rz] = transform.rotation;
            let mut tx = transform.translation[0];
            if !right_arm {
                tx = -tx;
                ry = -ry;
                rz = -rz;
            }
            pose.translate(tx, transform.translation[1], transform.translation[2]);
            let degrees = (std::f64::consts::PI / 180.0) as f32;
            pose.rotate(&Quaternion::rotation_xyz(
                rx * degrees,
                ry * degrees,
                rz * degrees,
            ));
            let [sx, sy, sz] = transform.scale;
            pose.scale(sx, sy, sz);
        }
        pose.translate(-0.5, -0.5, -0.5);
        for (quad, info) in layer.mesh.quads.iter().enumerate() {
            let (dx, dy, dz) = info.direction.step();
            let normal = pose.transform_normal([dx as f32, dy as f32, dz as f32]);
            let shade = entity_light(normal);
            let first = vertices.len();
            for index in [0, 1, 2, 0, 2, 3] {
                let source = &layer.mesh.vertices[quad * 4 + index];
                let [x, y, z] = source.position;
                vertices.push(Vertex {
                    position: pose.pose.transform_position(x, y, z),
                    uv: source.uv,
                    shade,
                    tint: source.colour(),
                    light,
                });
            }
            crate::objects::translucent_items::mark_layer(info.layer, &mut vertices[first..]);
        }
    }
    vertices
}

/// `minecraft_mix_light` with the level's two lights, which the arm's
/// shader applies to its normals in the same space.
fn entity_light(normal: [f32; 3]) -> f32 {
    let light = |direction: [f32; 3]| {
        let direction = crate::joml::normalize(direction);
        (direction[0] * normal[0] + direction[1] * normal[1] + direction[2] * normal[2]).max(0.0)
    };
    let [first, second] = crate::diffuse::level_directions();
    ((light(first) + light(second)) * 0.6 + 0.4).min(1.0)
}

/// The first-person hand's triangles, in view space: the view bob, the
/// hand's lag behind the view, then `ItemInHandRenderer.renderPlayerArm`
/// for an empty right hand.
pub fn hand_vertices(model: &mut PlayerModel, view: &HandView) -> Vec<Vertex> {
    let rad = Quaternion::degrees_to_radians;
    let sin = |value: f32| photon_core::math::mth_sin_f64(f64::from(value));
    let pi = std::f64::consts::PI as f32;
    let mut pose = view_pose(view);
    // `renderPlayerArm`, the right arm.
    let invert = 1.0_f32;
    let attack = view.attack;
    let sqrt_attack = f64::from(attack).sqrt() as f32;
    let x_swing = -0.3 * sin(sqrt_attack * pi);
    let y_swing = 0.4 * sin(sqrt_attack * (std::f64::consts::PI * 2.0) as f32);
    let z_swing = -0.4 * sin(attack * pi);
    pose.translate(
        invert * (x_swing + 0.640_000_05),
        y_swing + -0.6 + view.inverse_arm_height * -0.6,
        z_swing + -0.719_999_97,
    );
    pose.rotate(&Quaternion::rotation_y(rad(invert * 45.0)));
    let z_swing_rotation = sin(attack * attack * pi);
    let y_swing_rotation = sin(sqrt_attack * pi);
    pose.rotate(&Quaternion::rotation_y(rad(invert
        * y_swing_rotation
        * 70.0)));
    pose.rotate(&Quaternion::rotation_z(rad(invert
        * z_swing_rotation
        * -20.0)));
    pose.translate(-invert, 3.6, 3.5);
    pose.rotate(&Quaternion::rotation_z(rad(invert * 120.0)));
    pose.rotate(&Quaternion::rotation_x(rad(200.0)));
    pose.rotate(&Quaternion::rotation_y(rad(invert * -135.0)));
    pose.translate(invert * 5.6, 0.0, 0.0);
    let light = crate::block::light::light_uv(view.light);
    let mut vertices = Vec::new();
    for corners in model.render_arm(crate::entity::player_model::Arm::Right, &pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: crate::entity::textures::uv(view.skin, corner.uv),
                shade: 0.0,
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], 0.0],
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
    fn a_sleeper_lies_flat_with_its_head_toward_the_pillow() {
        let mut model = PlayerModel::new(false);
        model.setup_anim(&Default::default());
        let placement = Placement {
            feet: [0.0; 3],
            body_rot: 0.0,
            x_rot: 0.0,
            swim_amount: 0.0,
            in_water: false,
            visually_swimming: false,
            crouching: false,
            light: 0,
            skin: [0.0, 0.0, 1.0, 1.0],
            death_time: 0.0,
            red_overlay: false,
            fall_flying: None,
            auto_spin: None,
            upside_down: None,
            sleeping: Some(Sleep::in_bed(Some((0, 1)), 1.62, 0.0)),
        };
        let vertices = player_vertices(&model, &placement);
        let extent = |axis: usize| {
            vertices
                .iter()
                .fold((f32::MAX, f32::MIN), |(low, high), v| {
                    (low.min(v.position[axis]), high.max(v.position[axis]))
                })
        };
        // Lying on its back: no higher than the width of its body and legs' depth, and as long
        // as it was tall, running along z (the bed faces south, the head end is toward it).
        let (_, height) = extent(1);
        assert!(height < 0.5, "{height}");
        let (back, front) = extent(2);
        assert!(front - back > 1.7, "{back} {front}");
        // The shift puts the body 1.52 blocks away from the head end's side.
        assert!(back < -1.0 && front < 0.8, "{back} {front}");
    }

    #[test]
    fn an_upside_down_player_is_lifted_by_its_height_and_turned_over() {
        let mut model = PlayerModel::new(false);
        model.setup_anim(&Default::default());
        let placement = Placement {
            feet: [0.0; 3],
            body_rot: 0.0,
            x_rot: 0.0,
            swim_amount: 0.0,
            in_water: false,
            visually_swimming: false,
            crouching: false,
            light: 0,
            skin: [0.0, 0.0, 1.0, 1.0],
            death_time: 0.0,
            red_overlay: false,
            fall_flying: None,
            auto_spin: None,
            upside_down: None,
            sleeping: None,
        };
        let range = |placement: &Placement| {
            player_vertices(&model, placement)
                .iter()
                .fold((f32::MAX, f32::MIN), |(low, high), v| {
                    (low.min(v.position[1]), high.max(v.position[1]))
                })
        };
        let (low, high) = range(&placement);
        let (flipped_low, flipped_high) = range(&Placement {
            upside_down: Some(1.8),
            ..placement
        });
        assert!((flipped_high - (1.9 - low)).abs() < 1e-5);
        assert!((flipped_low - (1.9 - high)).abs() < 1e-5);
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
                skin: [0.0, 0.0, 1.0, 1.0],
                death_time: 0.0,
                red_overlay: false,
                fall_flying: None,
                auto_spin: None,
                upside_down: None,
                sleeping: None,
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
