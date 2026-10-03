//! The monsters of the arthropod, creeper and enderman kind the frame draws:
//! spiders, the creeper and the enderman (`SpiderRenderer`, `CreeperRenderer`,
//! `EndermanRenderer`).

use voxel_net::connection::PlaySession;
use voxel_render::entity_block::carried_block_pose;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{
    EndermanState, QuadrupedState, creeper_scale, creeper_white_overlay, mob_vertices,
    mob_vertices_glow, placement_pose,
};
use voxel_render::player_model::PlayerRenderState;
use voxel_render::scene::Vertex;

use crate::held_items::HeldItem;
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// Which model class poses a monster.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pose {
    /// `SpiderModel`.
    Spider,
    /// `CreeperModel`, which swells.
    Creeper,
    /// `EndermanModel`.
    Enderman,
    /// `SilverfishModel`, `EndermiteModel`: the segments' wave (true for the silverfish).
    Segmented(bool),
}

/// A monster drawn here: its entity type, model layer and texture, how it is
/// posed and how far it falls over when it dies (`getFlipDegrees`).
struct Monster {
    type_name: &'static str,
    layer: &'static str,
    texture: &'static str,
    pose: Pose,
    flip_degrees: f32,
}

const MONSTERS: [Monster; 6] = [
    Monster {
        type_name: "minecraft:spider",
        layer: "minecraft:spider#main",
        texture: "entity/spider/spider",
        pose: Pose::Spider,
        flip_degrees: 180.0,
    },
    Monster {
        type_name: "minecraft:cave_spider",
        layer: "minecraft:cave_spider#main",
        texture: "entity/spider/cave_spider",
        pose: Pose::Spider,
        flip_degrees: 180.0,
    },
    Monster {
        type_name: "minecraft:creeper",
        layer: "minecraft:creeper#main",
        texture: "entity/creeper/creeper",
        pose: Pose::Creeper,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:silverfish",
        layer: "minecraft:silverfish#main",
        texture: "entity/silverfish/silverfish",
        pose: Pose::Segmented(true),
        flip_degrees: 180.0,
    },
    Monster {
        type_name: "minecraft:endermite",
        layer: "minecraft:endermite#main",
        texture: "entity/endermite/endermite",
        pose: Pose::Segmented(false),
        flip_degrees: 180.0,
    },
    Monster {
        type_name: "minecraft:enderman",
        layer: "minecraft:enderman#main",
        texture: "entity/enderman/enderman",
        pose: Pose::Enderman,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
];

/// The paths (under `textures/`) of the textures the monsters wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    MONSTERS
        .iter()
        .flat_map(|monster| {
            [
                Some(monster.texture),
                voxel_render::render_types::eyes_texture(monster.type_name),
            ]
        })
        .flatten()
}

/// `LivingEntityRenderer` for the spiders, creepers and endermen the server
/// added, with the block an enderman carries. The spiders' and the enderman's
/// glowing eyes and the charged creeper's swirl are not drawn yet.
pub fn remote_monsters(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let ids: Vec<(i32, &Monster)> = MONSTERS
        .iter()
        .filter_map(|monster| Some((voxel_data::entity_type::id(monster.type_name)?, monster)))
        .collect();
    let mut vertices = Vec::new();
    let mut glow = Vec::new();
    for entity in world.entities() {
        let Some(monster) = ids
            .iter()
            .find(|(id, _)| *id == entity.type_id)
            .map(|(_, monster)| monster)
        else {
            continue;
        };
        // `EndermanRenderer.getRenderOffset`: a creepy enderman shakes, by a
        // little in x and in z (two draws of the renderer's random source).
        let shake = (monster.pose == Pose::Enderman && entity.is_creepy()).then(|| {
            let spread = 0.02;
            [
                models.shake.next_gaussian() * spread,
                models.shake.next_gaussian() * spread,
            ]
        });
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(monster.texture),
            models.model(monster.layer),
        ) else {
            continue;
        };
        let look = QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        };
        let mut placed = placement(world, &snapshot, view, entity.scale());
        placed.flip_degrees = monster.flip_degrees;
        if let Some([x, z]) = shake {
            placed.feet[0] += x as f32;
            placed.feet[2] += z as f32;
        }
        match monster.pose {
            Pose::Spider => model.setup_spider(&look),
            Pose::Segmented(silverfish) => {
                model.setup_segmented(entity.tick_count() as f32 + partial, silverfish);
            }
            Pose::Enderman => model.setup_enderman(&EndermanState {
                humanoid: PlayerRenderState {
                    head_y_rot: look.head_y_rot,
                    x_rot: look.x_rot,
                    walk_position: look.walk_position,
                    walk_speed: look.walk_speed,
                    age_in_ticks: entity.tick_count() as f32 + partial,
                    swim_amount: snapshot.swim_amount,
                    attack_time: snapshot.attack_time,
                    ..Default::default()
                },
                carrying: entity.is_carrying_block(),
                creepy: entity.is_creepy(),
            }),
            Pose::Creeper => {
                let swelling = entity.creeper_swelling(partial);
                model.setup_creeper(&look);
                placed.hook_scale = Some(creeper_scale(swelling));
                placed.white_overlay = creeper_white_overlay(swelling);
            }
        }
        vertices.extend(mob_vertices(model, &placed, rect));
        // The eyes, over the model as it is posed, after the other layers.
        if let Some(eyes) = voxel_render::render_types::eyes_texture(monster.type_name)
            .and_then(|eyes| textures.rect(eyes))
        {
            glow.extend(mob_vertices_glow(model, &placed, eyes));
        }
        if let Some(state) = entity
            .carried_block_state()
            .filter(|_| monster.pose == Pose::Enderman)
        {
            held.push(HeldItem::block(
                state,
                carried_block_pose(&placement_pose(&placed)),
                placed.light,
            ));
        }
    }
    models.glow.extend(glow);
    vertices
}
