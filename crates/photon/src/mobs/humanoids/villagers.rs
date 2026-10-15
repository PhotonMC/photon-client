//! The villagers the frame draws: the villager with the type and profession
//! layer over it, the wandering trader (`VillagerRenderer`,
//! `WanderingTraderRenderer`, `VillagerProfessionLayer`) and the witch, whose
//! model is built on the villager's (`WitchRenderer`).

use photon_net::connection::PlaySession;
use photon_render::entity::held_item::{crossed_arms_item_pose, head_item_pose, witch_potion_pose};
use photon_render::entity::mob_model::{
    QuadrupedState, VillagerState, WitchState, mob_vertices, placement_pose,
};
use photon_render::entity::player_model::Part;
use photon_render::entity::textures::EntityTextures;
use photon_render::entity::villager::{Pass, profession_passes, profession_textures};
use photon_render::scene::Vertex;

use photon_world::{ClientEntity, EquipmentSlot};

use crate::equipment::held_items::{HeldItem, Place};
use crate::mobs::{MobModels, is_item, placement};
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
    .chain(profession_textures("zombie_villager"))
}

/// The model layer of a villager's model of the folder (`villager` or
/// `zombie_villager`), hatted or not, grown or baby.
pub(crate) fn layer(folder: &str, hatless: bool, baby: bool) -> String {
    let baby = if baby { "_baby" } else { "" };
    let hat = if hatless { "_no_hat" } else { "" };
    format!("minecraft:{folder}{baby}{hat}#main")
}

/// The passes of `VillagerProfessionLayer` for the entity, from the villager
/// data the server sent; none while it cannot be seen or names a type or
/// profession the registries lack.
pub(crate) fn profession_layer(entity: &ClientEntity, folder: &str, baby: bool) -> Vec<Pass> {
    if entity.is_invisible() {
        return Vec::new();
    }
    let [type_id, profession_id, level] = entity.villager_data();
    match (
        photon_data::villager::villager_type(type_id),
        photon_data::villager::profession(profession_id),
    ) {
        (Some((type_name, _)), Some((profession, _))) => {
            profession_passes(folder, type_name, profession, level, baby)
        }
        _ => Vec::new(),
    }
}

/// `VillagerRenderer` and `WanderingTraderRenderer` for the villagers the
/// server added, with what they hold in their crossed arms and wear on their
/// heads.
pub fn remote_villagers(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    let (Some(villager), Some(trader)) = (
        photon_data::entity_type::id("minecraft:villager"),
        photon_data::entity_type::id("minecraft:wandering_trader"),
    ) else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.entities().drawn() {
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
                head_y_rot: photon_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks: entity.tick_count() as f32 + partial,
            unhappy: entity.is_unhappy(),
        };
        let placed = placement(world, &snapshot, view, entity.scale());
        // The model and texture of each pass: the body first, then the layer.
        let mut passes = vec![if is_villager {
            (
                layer(FOLDER, false, baby),
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
        if is_villager {
            passes.extend(
                profession_layer(entity, FOLDER, baby)
                    .into_iter()
                    .map(|pass| (layer(FOLDER, pass.hatless, baby), pass.texture)),
            );
        }
        let main_layer = passes[0].0.clone();
        for (id, texture) in passes {
            let (Some(rect), Some(model)) = (textures.rect(&texture), models.model(&id)) else {
                continue;
            };
            model.setup_villager(&state);
            vertices.extend(mob_vertices(model, &placed, rect));
        }
        let Some(model) = models.model(&main_layer) else {
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
        if let (Some(stack), Some(arms)) = (entity.equipment(EquipmentSlot::MainHand), part("arms"))
        {
            let pose = crossed_arms_item_pose(&entity_pose, model.root(), arms);
            held.push(HeldItem::stack(stack, pose, Place::Ground, placed.light));
        }
        if let (Some(stack), Some(head)) = (entity.equipment(EquipmentSlot::Head), part("head")) {
            // `VillagerRenderer.CUSTOM_HEAD_TRANSFORMS`, or the default's.
            let transforms = if is_villager {
                (-0.117_187_5, 1.0)
            } else {
                (0.0, 1.0)
            };
            let pose = head_item_pose(&entity_pose, model.root(), head, transforms);
            held.push(HeldItem::stack(stack, pose, Place::Head, placed.light));
        }
    }
}

/// A part of a model's tree by its name.
fn child<'a>(from: &'a Part, name: &str) -> Option<&'a Part> {
    from.children().iter().find(|part| part.name() == name)
}

/// `WitchRenderer` for the witches the server added, with what they hold
/// (`WitchItemLayer`): a potion at the nose, anything else on the folded arms.
pub fn remote_witches(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    let Some(witch) = photon_data::entity_type::id("minecraft:witch") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.entities().drawn() {
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
                head_y_rot: photon_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks: entity.tick_count() as f32 + partial,
            entity_id: entity.id().0,
            holding_item: entity.equipment(EquipmentSlot::MainHand).is_some(),
        });
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
        let Some(stack) = entity.equipment(EquipmentSlot::MainHand) else {
            continue;
        };
        let part = child;
        let entity_pose = placement_pose(&placed);
        let root = model.root();
        let pose = if is_item(stack, "minecraft:potion")
            && let Some(head) = part(root, "head")
            && let Some(nose) = part(head, "nose")
        {
            witch_potion_pose(&entity_pose, root, head, nose)
        } else if let Some(arms) = part(root, "arms") {
            crossed_arms_item_pose(&entity_pose, root, arms)
        } else {
            continue;
        };
        held.push(HeldItem::stack(stack, pose, Place::Ground, placed.light));
    }
}
