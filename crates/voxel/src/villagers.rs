//! The villagers the frame draws: the villager with the type and profession
//! layer over it, the wandering trader (`VillagerRenderer`,
//! `WanderingTraderRenderer`, `VillagerProfessionLayer`) and the witch, whose
//! model is built on the villager's (`WitchRenderer`).

use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{QuadrupedState, VillagerState, WitchState, mob_vertices};
use voxel_render::scene::Vertex;
use voxel_render::villager::{profession_passes, profession_textures};

use voxel_world::EquipmentSlot;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const WANDERING_TRADER_LAYER: &str = "minecraft:wandering_trader#main";
const WANDERING_TRADER_TEXTURE: &str = "entity/wandering_trader/wandering_trader";
const WITCH_LAYER: &str = "minecraft:witch#main";
const WITCH_TEXTURE: &str = "entity/witch/witch";

/// The texture folder the villager's layer reads (`VillagerRenderer`'s path).
const FOLDER: &str = "villager";

/// The paths (under `textures/`) of the textures the villagers wear.
pub fn textures() -> impl Iterator<Item = String> {
    [
        "entity/villager/villager",
        "entity/villager/villager_baby",
        WANDERING_TRADER_TEXTURE,
        WITCH_TEXTURE,
    ]
    .into_iter()
    .map(str::to_owned)
    .chain(profession_textures(FOLDER))
}

/// The model layer of a villager's model, hatted or not, grown or baby.
fn layer(hatless: bool, baby: bool) -> String {
    let baby = if baby { "_baby" } else { "" };
    let hat = if hatless { "_no_hat" } else { "" };
    format!("minecraft:villager{baby}{hat}#main")
}

/// `VillagerRenderer` and `WanderingTraderRenderer` for the villagers the
/// server added. What they hold in their crossed arms and wear on their heads
/// is not drawn yet.
pub fn remote_villagers(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let (Some(villager), Some(trader)) = (
        voxel_data::entity_type::id("minecraft:villager"),
        voxel_data::entity_type::id("minecraft:wandering_trader"),
    ) else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let is_villager = entity.type_id == villager;
        if !is_villager && entity.type_id != trader {
            continue;
        }
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let baby = is_villager && entity.is_baby();
        let state = VillagerState {
            quadruped: QuadrupedState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks: entity.tick_count() as f32 + partial,
            unhappy: entity.is_unhappy(),
        };
        let placement = placement(world, &snapshot, view, 1.0);
        // The model and texture of each pass: the body first, then the layer.
        let mut passes = vec![if is_villager {
            (
                layer(false, baby),
                if baby {
                    "entity/villager/villager_baby".to_owned()
                } else {
                    "entity/villager/villager".to_owned()
                },
            )
        } else {
            (
                WANDERING_TRADER_LAYER.to_owned(),
                WANDERING_TRADER_TEXTURE.to_owned(),
            )
        }];
        // The layer is not drawn on a villager that cannot be seen.
        if is_villager && !entity.is_invisible() {
            let [type_id, profession_id, level] = entity.villager_data();
            if let (Some((type_name, _)), Some((profession, _))) = (
                voxel_data::villager::villager_type(type_id),
                voxel_data::villager::profession(profession_id),
            ) {
                passes.extend(
                    profession_passes(FOLDER, type_name, profession, level, baby)
                        .into_iter()
                        .map(|pass| (layer(pass.hatless, baby), pass.texture)),
                );
            }
        }
        for (id, texture) in passes {
            let (Some(rect), Some(model)) = (textures.rect(&texture), models.model(&id)) else {
                continue;
            };
            model.setup_villager(&state);
            vertices.extend(mob_vertices(model, &placement, rect));
        }
    }
    vertices
}

/// `WitchRenderer` for the witches the server added. What a witch holds
/// (`WitchItemLayer`) is not drawn yet.
pub fn remote_witches(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let Some(witch) = voxel_data::entity_type::id("minecraft:witch") else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let (true, Some(snapshot), Some(rect), Some(model)) = (
            entity.type_id == witch,
            world.entity_snapshot(entity.id(), partial),
            textures.rect(WITCH_TEXTURE),
            models.model(WITCH_LAYER),
        ) else {
            continue;
        };
        model.setup_witch(&WitchState {
            quadruped: QuadrupedState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks: entity.tick_count() as f32 + partial,
            entity_id: entity.id(),
            holding_item: entity.equipment(EquipmentSlot::MainHand).is_some(),
        });
        vertices.extend(mob_vertices(
            model,
            &placement(world, &snapshot, view, 1.0),
            rect,
        ));
    }
    vertices
}
