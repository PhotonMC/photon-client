//! Mob models: a layer of the data table baked into parts, posed the way the
//! reference's model classes pose them, and placed where a mob stands the
//! way `LivingEntityRenderer` places any living entity.

use voxel_core::math::{mth_cos_f64, mth_lerp_f32 as lerp_f32, mth_sin_f64};

use voxel_data::entity_model::ModelLayer;

use crate::entity::textures::{TextureRect, uv};

use crate::joml::{Pose, Quaternion};

use crate::entity::player_model::{
    ModelVertex, Part, PlayerRenderState, SwingAnimation, humanoid_anim, rot_lerp_rad,
};

use crate::scene::Vertex;

/// `Mth.triangleWave`.
pub(super) fn triangle_wave(index: f32, period: f32) -> f32 {
    ((index % period - period * 0.5).abs() - period * 0.25) / (period * 0.25)
}

/// A part of an adult axolotl by its path from the root.
pub(super) fn at<'a>(root: &'a mut Part, path: &str) -> &'a mut Part {
    path.split('/').fold(root, |part, name| {
        part.child_mut(name).expect("the axolotl has this part")
    })
}

/// `Mth.rotLerp` (which wraps the radians it is given as if they were degrees).
pub(super) fn rot_lerp(amount: f32, from: f32, to: f32) -> f32 {
    from + amount * voxel_core::math::wrap_degrees_f32(to - from)
}

/// A mob's model: the parts of one layer.
#[derive(Debug, Clone)]
pub struct MobModel {
    pub(super) root: Part,
    /// `FoxModel.legMotionPos`: the model's own count, which a faceplanted
    /// fox's legs follow and which grows with every pose of such a fox.
    pub(super) leg_motion_pos: f32,
}

mod amphibians;

pub use amphibians::*;

mod bears;

pub use bears::*;

mod bosses;

pub use bosses::*;

mod canines;

pub use canines::*;

mod equines;

pub use equines::*;

mod farm;

pub use farm::*;

mod felines;

pub use felines::*;

mod fish;

pub use fish::*;

mod flyers;

pub use flyers::*;

mod golems;

pub use golems::*;

mod humanoid;

pub use humanoid::*;

mod marine;

pub use marine::*;

mod monsters;

pub use monsters::*;

mod nether;

pub use nether::*;

mod objects;

pub use objects::*;

mod placement;

pub use placement::*;

mod vertices;

pub use vertices::*;

mod villagers;

pub use villagers::*;

impl MobModel {
    /// `LayerDefinition.bakeRoot`.
    pub fn bake(layer: &ModelLayer) -> Self {
        Self {
            root: Part::baked(&layer.root, (layer.texture_width, layer.texture_height)),
            leg_motion_pos: 0.0,
        }
    }

    pub fn root(&self) -> &Part {
        &self.root
    }

    /// `KeyframeAnimation.apply(millisSinceStart, targetScale)` of a definition
    /// on the pose the model is in.
    pub fn animate(
        &mut self,
        definition: &voxel_data::animation::Definition,
        millis: i64,
        target_scale: f32,
    ) {
        crate::entity::keyframe::apply(&mut self.root, definition, millis, target_scale);
    }

    /// `KeyframeAnimation.apply(animationState, ageInTicks)`: the animation
    /// runs if its state has started.
    pub(super) fn animate_state(
        &mut self,
        name: &str,
        running: voxel_world::animation_state::AnimationState,
        age_in_ticks: f32,
    ) {
        if !running.is_started() {
            return;
        }
        if let Some(definition) = voxel_data::animation::definition(name) {
            let millis = (running.time_in_millis(age_in_ticks) as f32 * 1.0) as i64;
            self.animate(definition, millis, 1.0);
        }
    }

    /// `ModelPart.render` of the whole model.
    pub fn render(&self, pose: &Pose) -> Vec<(String, [ModelVertex; 4])> {
        let mut out = Vec::new();
        self.root.render(pose, "", &mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel_data::entity_model::layer;

    #[test]
    fn a_quad_the_scroll_carries_over_the_edge_is_cut_there_and_keeps_its_area() {
        let breeze = MobModel::bake(layer("minecraft:breeze#wind").expect("the wind's layer"));
        let placement = MobPlacement::default();
        let area = |vertices: &[Vertex]| -> f32 {
            vertices
                .chunks_exact(3)
                .map(|triangle| {
                    let [a, b, c] = [triangle[0].uv, triangle[1].uv, triangle[2].uv];
                    ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
                })
                .sum()
        };
        let rect = [0.0, 0.0, 1.0, 1.0];
        let still = mob_vertices_scrolled(&breeze, &placement, rect, 0.0);
        let scrolled = mob_vertices_scrolled(&breeze, &placement, rect, 0.37);
        // Every texel stays inside the texture, and the picture keeps its size.
        for vertex in &scrolled {
            assert!((0.0..=1.0).contains(&vertex.uv[0]), "{:?}", vertex.uv);
        }
        assert!((area(&still) - area(&scrolled)).abs() < 1e-3);
        assert!(scrolled.len() > still.len());
    }

    #[test]
    fn a_faded_emissive_layer_carries_its_alpha_in_whole_255ths_and_the_overlay() {
        let heart = MobModel::bake(layer("minecraft:warden#heart").expect("the heart's layer"));
        let placement = MobPlacement {
            red_overlay: true,
            ..MobPlacement::default()
        };
        let vertices = mob_vertices_emissive_faded(&heart, &placement, [0.0, 0.0, 1.0, 1.0], 0.5);
        assert!(!vertices.is_empty());
        for vertex in &vertices {
            // `ARGB.white(0.5F)`: floor(127.5) = 127 of 255.
            assert_eq!(vertex.light, [-1.0, 127.0 / 255.0]);
            assert_eq!(vertex.shade, 1.0);
        }
        // The plain emissive layer has no alpha and no overlay.
        let plain = mob_vertices_emissive(&heart, &placement, [0.0, 0.0, 1.0, 1.0]);
        assert!(plain.iter().all(|v| v.light == [-1.0; 2] && v.shade == 0.0));
    }

    #[test]
    fn a_translucent_models_far_faces_are_drawn_before_its_near_ones() {
        let outer = MobModel::bake(layer("minecraft:slime#outer").expect("the slime's shell"));
        let placement = MobPlacement {
            feet: [0.0, 0.0, 5.0],
            ..MobPlacement::default()
        };
        let mut shell = mob_vertices(&outer, &placement, [0.0, 0.0, 1.0, 1.0]);
        sort_translucent_quads(&mut shell, [0.0; 3]);
        let distance = |quad: &[Vertex]| -> f32 {
            let centre = |axis: usize| (quad[0].position[axis] + quad[2].position[axis]) / 2.0;
            centre(0).powi(2) + centre(1).powi(2) + centre(2).powi(2)
        };
        let distances: Vec<f32> = shell.chunks_exact(6).map(distance).collect();
        assert_eq!(distances.len(), 6);
        assert!(
            distances.windows(2).all(|pair| pair[0] >= pair[1]),
            "{distances:?}"
        );
    }

    fn pig() -> MobModel {
        MobModel::bake(layer("minecraft:pig#main").expect("the pig's layer"))
    }

    #[test]
    fn a_pig_stands_on_the_ground_and_reaches_the_height_of_its_head() {
        let mut model = pig();
        model.setup_quadruped(&QuadrupedState::default());
        let vertices = mob_vertices(&model, &MobPlacement::default(), [0.0, 0.0, 1.0, 1.0]);
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
