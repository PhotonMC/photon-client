//! The golems the frame draws: the iron golem with its cracks over it, and
//! the snow golem (`IronGolemRenderer`, `IronGolemCrackinessLayer`,
//! `SnowGolemRenderer`).

use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{
    Crackiness, IronGolemState, QuadrupedState, golem_crackiness, iron_golem_roll, mob_vertices,
};
use voxel_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const IRON_GOLEM_LAYER: &str = "minecraft:iron_golem#main";
const IRON_GOLEM_TEXTURE: &str = "entity/iron_golem/iron_golem";
const SNOW_GOLEM_LAYER: &str = "minecraft:snow_golem#main";
const SNOW_GOLEM_TEXTURE: &str = "entity/snow_golem/snow_golem";

/// The most health an iron golem has (`IronGolem.createAttributes`); the
/// attribute the server may change is not read for mobs yet.
const IRON_GOLEM_MAX_HEALTH: f32 = 100.0;

/// The texture the cracks of each level are drawn in.
fn crack_texture(level: Crackiness) -> Option<&'static str> {
    match level {
        Crackiness::None => None,
        Crackiness::Low => Some("entity/iron_golem/iron_golem_crackiness_low"),
        Crackiness::Medium => Some("entity/iron_golem/iron_golem_crackiness_medium"),
        Crackiness::High => Some("entity/iron_golem/iron_golem_crackiness_high"),
    }
}

/// The paths (under `textures/`) of the textures the golems wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [
        IRON_GOLEM_TEXTURE,
        "entity/iron_golem/iron_golem_crackiness_low",
        "entity/iron_golem/iron_golem_crackiness_medium",
        "entity/iron_golem/iron_golem_crackiness_high",
        SNOW_GOLEM_TEXTURE,
    ]
    .into_iter()
}

/// `IronGolemRenderer` and `SnowGolemRenderer` for the golems the server
/// added. The flower an iron golem offers and the pumpkin a snow golem wears
/// are blocks, which are not drawn on entities yet.
pub fn remote_golems(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let (Some(iron), Some(snow)) = (
        voxel_data::entity_type::id("minecraft:iron_golem"),
        voxel_data::entity_type::id("minecraft:snow_golem"),
    ) else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let is_iron = entity.type_id == iron;
        if !is_iron && entity.type_id != snow {
            continue;
        }
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let look = QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        };
        let mut placed = placement(world, &snapshot, view, 1.0);
        let mut passes = Vec::new();
        if is_iron {
            placed.roll = iron_golem_roll(look.walk_position, look.walk_speed);
            passes.push(IRON_GOLEM_TEXTURE);
            // What the data holds until the server says otherwise is one
            // health (`LivingEntity.DATA_HEALTH_ID`).
            let fraction = entity.health().unwrap_or(1.0) / IRON_GOLEM_MAX_HEALTH;
            if !entity.is_invisible()
                && let Some(texture) = crack_texture(golem_crackiness(fraction))
            {
                passes.push(texture);
            }
        } else {
            passes.push(SNOW_GOLEM_TEXTURE);
        }
        for texture in passes {
            let layer = if is_iron {
                IRON_GOLEM_LAYER
            } else {
                SNOW_GOLEM_LAYER
            };
            let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
                continue;
            };
            if is_iron {
                let (attack, offer) = entity.iron_golem_ticks();
                model.setup_iron_golem(&IronGolemState {
                    quadruped: look,
                    attack_ticks_remaining: if attack > 0 {
                        attack as f32 - partial
                    } else {
                        0.0
                    },
                    offer_flower_tick: offer,
                });
            } else {
                model.setup_snow_golem(&look);
            }
            vertices.extend(mob_vertices(model, &placed, rect));
        }
    }
    vertices
}
