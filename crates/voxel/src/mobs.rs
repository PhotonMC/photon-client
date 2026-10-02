//! The mobs the frame draws: which entity types have a model and a texture,
//! and where the server's mobs stand between two ticks.

use std::collections::HashMap;

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{MobModel, MobPlacement, QuadrupedState, mob_vertices};
use voxel_render::scene::Vertex;

use crate::overlays::{FrameView, packed_light};

/// A mob's model layers (grown and baby) and their textures.
struct MobKind {
    type_name: &'static str,
    adult_layer: &'static str,
    baby_layer: &'static str,
    adult_texture: &'static str,
    baby_texture: &'static str,
}

/// The mobs drawn so far. The variant a pig or cow wears is the temperate
/// one: the variant registry is not read yet.
const KINDS: [MobKind; 2] = [
    MobKind {
        type_name: "minecraft:pig",
        adult_layer: "minecraft:pig#main",
        baby_layer: "minecraft:pig_baby#main",
        adult_texture: "entity/pig/pig_temperate",
        baby_texture: "entity/pig/pig_temperate_baby",
    },
    MobKind {
        type_name: "minecraft:cow",
        adult_layer: "minecraft:cow#main",
        baby_layer: "minecraft:cow_baby#main",
        adult_texture: "entity/cow/cow_temperate",
        baby_texture: "entity/cow/cow_temperate_baby",
    },
];

/// The paths (under `textures/`) of the textures the mobs wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    KINDS
        .iter()
        .flat_map(|kind| [kind.adult_texture, kind.baby_texture])
}

/// The baked models of the mobs, which are posed anew for each one drawn.
pub struct MobModels {
    kinds: HashMap<i32, &'static MobKind>,
    models: HashMap<&'static str, MobModel>,
}

impl MobModels {
    pub fn new() -> Self {
        Self {
            kinds: KINDS
                .iter()
                .filter_map(|kind| Some((voxel_data::entity_type::id(kind.type_name)?, kind)))
                .collect(),
            models: HashMap::new(),
        }
    }

    fn model(&mut self, layer: &'static str) -> Option<&mut MobModel> {
        if !self.models.contains_key(layer) {
            let baked = MobModel::bake(voxel_data::entity_model::layer(layer)?);
            self.models.insert(layer, baked);
        }
        self.models.get_mut(layer)
    }
}

impl Default for MobModels {
    fn default() -> Self {
        Self::new()
    }
}

/// `LivingEntityRenderer` for the mobs the server added: each posed from
/// where it is between two ticks.
pub fn remote_mobs(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let Some(kind) = models.kinds.get(&entity.type_id).copied() else {
            continue;
        };
        let (layer, texture) = if entity.is_baby() {
            (kind.baby_layer, kind.baby_texture)
        } else {
            (kind.adult_layer, kind.adult_texture)
        };
        let (Some(snapshot), Some(rect)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
        ) else {
            continue;
        };
        let Some(model) = models.model(layer) else {
            continue;
        };
        model.setup_quadruped(&QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        });
        let (x, y, z) = snapshot.position;
        vertices.extend(mob_vertices(
            model,
            &MobPlacement {
                feet: [
                    (x - f64::from(view.origin[0])) as f32,
                    (y - f64::from(view.origin[1])) as f32,
                    (z - f64::from(view.origin[2])) as f32,
                ],
                body_rot: snapshot.body_yaw,
                scale: 1.0,
                light: packed_light(world, Vec3::new(x, y + 0.5, z)),
                death_time: snapshot.death_time,
                flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
                red_overlay: snapshot.red_overlay,
            },
            rect,
        ));
    }
    vertices
}
