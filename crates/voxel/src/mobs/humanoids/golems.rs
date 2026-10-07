//! The golems the frame draws: the iron golem with its cracks over it and the
//! flower it offers, and the snow golem with its pumpkin (`IronGolemRenderer`,
//! `IronGolemCrackinessLayer`, `IronGolemFlowerLayer`, `SnowGolemRenderer`,
//! `SnowGolemHeadLayer`).

use voxel_net::connection::PlaySession;
use voxel_render::entity::block::{iron_golem_flower_pose, snow_golem_head_pose};
use voxel_render::entity::mob_model::{
    Crackiness, IronGolemState, QuadrupedState, golem_crackiness, iron_golem_roll, mob_vertices,
    overlay_code, placement_pose,
};
use voxel_render::entity::textures::EntityTextures;
use voxel_render::scene::Vertex;

use crate::equipment::held_items::HeldItem;
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const IRON_GOLEM_LAYER: &str = "minecraft:iron_golem#main";
const IRON_GOLEM_TEXTURE: &str = "entity/iron_golem/iron_golem";
const SNOW_GOLEM_LAYER: &str = "minecraft:snow_golem#main";
const SNOW_GOLEM_TEXTURE: &str = "entity/snow_golem/snow_golem";

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

/// The default state of a block, which is what the layers draw
/// (`Blocks.CARVED_PUMPKIN.defaultBlockState()`).
fn default_state(name: &str) -> Option<u32> {
    voxel_data::block::registry::block(name)
        .map(voxel_data::block::registry::Block::default_state_id)
}

/// `IronGolemRenderer` and `SnowGolemRenderer` for the golems the server
/// added, with the flower and the pumpkin. A pumpkin on an invisible golem
/// that glows would be drawn as an outline only, which is not drawn yet.
pub fn remote_golems(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
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
        let mut placed = placement(world, &snapshot, view, entity.scale());
        let mut passes = Vec::new();
        if is_iron {
            placed.roll = iron_golem_roll(look.walk_position, look.walk_speed);
            passes.push(IRON_GOLEM_TEXTURE);
            // What the data holds until the server says otherwise is one
            // health (`LivingEntity.DATA_HEALTH_ID`).
            let fraction = entity.health().unwrap_or(1.0) / entity.max_health().unwrap_or(1.0);
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
        let layer = if is_iron {
            IRON_GOLEM_LAYER
        } else {
            SNOW_GOLEM_LAYER
        };
        let Some(model) = models.model(layer) else {
            continue;
        };
        let part = |name: &str| {
            model
                .root()
                .children()
                .iter()
                .find(|part| part.name() == name)
        };
        let entity_pose = placement_pose(&placed);
        // `SnowGolemHeadLayer` takes the entity's overlay; the flower takes none.
        let (pose, state, overlay) = if is_iron && entity.iron_golem_ticks().1 > 0 {
            (
                part("right_arm").map(|arm| iron_golem_flower_pose(&entity_pose, arm)),
                default_state("minecraft:poppy"),
                0.0,
            )
        } else if !is_iron && entity.has_pumpkin() && !entity.is_invisible() {
            (
                part("head").map(|head| snow_golem_head_pose(&entity_pose, head)),
                default_state("minecraft:carved_pumpkin"),
                overlay_code(placed.red_overlay, 0.0),
            )
        } else {
            (None, None, 0.0)
        };
        if let (Some(pose), Some(state)) = (pose, state) {
            held.push(HeldItem::block(state, pose, placed.light, overlay));
        }
    }
    vertices
}
