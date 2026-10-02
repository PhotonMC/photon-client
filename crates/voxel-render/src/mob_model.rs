//! Mob models: a layer of the data table baked into parts, posed the way the
//! reference's model classes pose them, and placed where a mob stands the
//! way `LivingEntityRenderer` places any living entity.

use voxel_core::math::mth_cos_f64;
use voxel_data::entity_model::ModelLayer;

use crate::entity_textures::{TextureRect, uv};
use crate::joml::{Pose, Quaternion};
use crate::player_model::{ModelVertex, Part};
use crate::scene::Vertex;

/// What `QuadrupedModel.setupAnim` reads of a mob.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct QuadrupedState {
    /// The head's turn from the body and its pitch, in degrees.
    pub head_y_rot: f32,
    pub x_rot: f32,
    pub walk_position: f32,
    pub walk_speed: f32,
}

/// A mob's model: the parts of one layer.
#[derive(Debug, Clone)]
pub struct MobModel {
    root: Part,
}

impl MobModel {
    /// `LayerDefinition.bakeRoot`.
    pub fn bake(layer: &ModelLayer) -> Self {
        Self {
            root: Part::baked(&layer.root, (layer.texture_width, layer.texture_height)),
        }
    }

    pub fn root(&self) -> &Part {
        &self.root
    }

    /// `QuadrupedModel.setupAnim`: the head follows the look, and the legs
    /// swing in diagonal pairs with the walk.
    pub fn setup_quadruped(&mut self, state: &QuadrupedState) {
        // `EntityModel.setupAnim`: the pose is reset first.
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = state.head_y_rot * degrees;
        }
        let (position, speed) = (state.walk_position, state.walk_speed);
        let swing = [
            ("right_hind_leg", cos(position * 0.6662) * 1.4 * speed),
            ("left_hind_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("right_front_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("left_front_leg", cos(position * 0.6662) * 1.4 * speed),
        ];
        for (name, x_rot) in swing {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
    }

    /// `ModelPart.render` of the whole model.
    pub fn render(&self, pose: &Pose) -> Vec<(String, [ModelVertex; 4])> {
        let mut out = Vec::new();
        self.root.render(pose, "", &mut out);
        out
    }
}

/// Where a mob is and how it is lit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobPlacement {
    /// The feet, from the mesh origin.
    pub feet: [f32; 3],
    /// The body's facing, in degrees.
    pub body_rot: f32,
    /// The entity's scale (the attribute; one unless the server says other).
    pub scale: f32,
    /// Packed block and sky light at the mob.
    pub light: i32,
    /// `LivingEntityRenderState.deathTime`: zero while it lives.
    pub death_time: f32,
    /// `LivingEntityRenderer.getFlipDegrees`: how far a dead mob falls over.
    pub flip_degrees: f32,
    /// Drawn with the hurt overlay (`hasRedOverlay`).
    pub red_overlay: bool,
}

/// `LivingEntityRenderer.getFlipDegrees` of most mobs.
pub const FLIP_DEGREES: f32 = 90.0;

/// `LivingEntityRenderer.setupRotations`'s fall of a dying entity: the
/// turn about z, in degrees, once its death has begun. It eases out as the
/// square root of the time since the first tick, and stops at the full flip.
pub fn death_roll(death_time: f32, flip_degrees: f32) -> Option<f32> {
    (death_time > 0.0).then(|| {
        let fall = ((death_time - 1.0) / 20.0 * 1.6).sqrt();
        fall.min(1.0) * flip_degrees
    })
}

/// `LivingEntityRenderer.submit` for a mob standing on its feet: scaled,
/// turned to face away from the body's facing, flipped upright and lowered so
/// the feet meet the ground.
pub fn placement_pose(placement: &MobPlacement) -> Pose {
    let mut pose = Pose::default();
    pose.translate(placement.feet[0], placement.feet[1], placement.feet[2]);
    pose.scale(placement.scale, placement.scale, placement.scale);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        180.0 - placement.body_rot,
    )));
    if let Some(roll) = death_roll(placement.death_time, placement.flip_degrees) {
        pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
            roll,
        )));
    }
    pose.scale(-1.0, -1.0, 1.0);
    pose.translate(0.0, -1.501, 0.0);
    pose
}

/// The posed model's triangles, placed, each vertex carrying its normal in
/// its tint for the entity shader, with texture coordinates in the packed
/// entity textures.
pub fn mob_vertices(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
) -> Vec<Vertex> {
    let pose = placement_pose(placement);
    let light = crate::block_light::light_uv(placement.light);
    let mut vertices = Vec::new();
    for (_, corners) in model.render(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: f32::from(placement.red_overlay),
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
    use voxel_data::entity_model::layer;

    fn pig() -> MobModel {
        MobModel::bake(layer("minecraft:pig#main").expect("the pig's layer"))
    }

    #[test]
    fn a_pig_stands_on_the_ground_and_reaches_the_height_of_its_head() {
        let mut model = pig();
        model.setup_quadruped(&QuadrupedState::default());
        let vertices = mob_vertices(
            &model,
            &MobPlacement {
                feet: [0.0; 3],
                body_rot: 0.0,
                scale: 1.0,
                light: 0,
                death_time: 0.0,
                flip_degrees: FLIP_DEGREES,
                red_overlay: false,
            },
            [0.0, 0.0, 1.0, 1.0],
        );
        let lowest = vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::MAX, f32::min);
        let highest = vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::MIN, f32::max);
        // The model hangs from 24 sixteenths, a thousandth above the ground
        // (`-1.501`), and the pig's head tops it at 8 sixteenths below that.
        assert!((lowest - 0.001).abs() < 1e-5, "{lowest}");
        assert!((highest - (1.0 + 0.001)).abs() < 1e-5, "{highest}");
    }

    #[test]
    fn walking_swings_the_legs_in_diagonal_pairs() {
        let mut model = pig();
        model.setup_quadruped(&QuadrupedState {
            walk_position: 1.0,
            walk_speed: 0.5,
            ..QuadrupedState::default()
        });
        let x_rot = |name: &str| {
            model
                .root()
                .children()
                .iter()
                .find(|part| part.name() == name)
                .unwrap()
                .x_rot
        };
        assert!(x_rot("right_hind_leg") > 0.0);
        assert!((x_rot("right_hind_leg") + x_rot("left_hind_leg")).abs() < 1e-6);
        assert_eq!(x_rot("right_hind_leg"), x_rot("left_front_leg"));
        assert_eq!(x_rot("left_hind_leg"), x_rot("right_front_leg"));
    }
}
