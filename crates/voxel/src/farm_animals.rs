//! The farm animals that are not drawn from a variant registry: the goat and
//! the mooshroom with its mushrooms (`GoatRenderer`, `MushroomCowRenderer`,
//! `MushroomCowMushroomLayer`).

use voxel_net::connection::PlaySession;
use voxel_render::entity_block::mooshroom_mushroom_poses;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{GoatState, QuadrupedState, mob_vertices, placement_pose};
use voxel_render::scene::Vertex;

use crate::held_items::HeldItem;
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The goat's textures, grown and baby.
const GOAT_TEXTURES: [&str; 2] = ["entity/goat/goat", "entity/goat/goat_baby"];

/// The mooshroom's textures by variant (red, brown), grown and baby, and the
/// mushroom each grows.
const MOOSHROOMS: [([&str; 2], &str); 2] = [
    (
        ["entity/cow/mooshroom_red", "entity/cow/mooshroom_red_baby"],
        "minecraft:red_mushroom",
    ),
    (
        [
            "entity/cow/mooshroom_brown",
            "entity/cow/mooshroom_brown_baby",
        ],
        "minecraft:brown_mushroom",
    ),
];

/// The paths (under `textures/`) of the textures these animals wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    GOAT_TEXTURES
        .into_iter()
        .chain(MOOSHROOMS.into_iter().flat_map(|(textures, _)| textures))
}

/// `GoatRenderer` and `MushroomCowRenderer` for the goats and mooshrooms the
/// server added. A mooshroom's mushrooms on an invisible animal that glows
/// would be drawn as outlines only, which is not drawn yet.
pub fn remote_farm_animals(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let (Some(goat), Some(mooshroom)) = (
        voxel_data::entity_type::id("minecraft:goat"),
        voxel_data::entity_type::id("minecraft:mooshroom"),
    ) else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let is_goat = entity.type_id == goat;
        if !is_goat && entity.type_id != mooshroom {
            continue;
        }
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let baby = entity.is_baby();
        let look = QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        };
        let variant = usize::try_from(entity.mooshroom_variant()).unwrap_or(0);
        let (texture, layer) = if is_goat {
            (
                GOAT_TEXTURES[usize::from(baby)],
                if baby {
                    "minecraft:goat_baby#main"
                } else {
                    "minecraft:goat#main"
                },
            )
        } else {
            (
                MOOSHROOMS[variant].0[usize::from(baby)],
                if baby {
                    "minecraft:mooshroom_baby#main"
                } else {
                    "minecraft:mooshroom#main"
                },
            )
        };
        let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
            continue;
        };
        if is_goat {
            let (has_left_horn, has_right_horn) = entity.goat_horns();
            model.setup_goat(&GoatState {
                quadruped: look,
                has_left_horn,
                has_right_horn,
                ramming_x_head_rot: entity.goat_ramming_x_head_rot(),
                baby,
            });
        } else {
            model.setup_quadruped(&look);
        }
        let placed = placement(world, &snapshot, view, 1.0);
        vertices.extend(mob_vertices(model, &placed, rect));
        if !is_goat
            && !baby
            && !entity.is_invisible()
            && let Some(state) = voxel_data::block::registry::block(MOOSHROOMS[variant].1)
                .map(voxel_data::block::registry::Block::default_state_id)
            && let Some(head) = model
                .root()
                .children()
                .iter()
                .find(|part| part.name() == "head")
        {
            for pose in mooshroom_mushroom_poses(&placement_pose(&placed), head) {
                held.push(HeldItem::block(state, pose, placed.light));
            }
        }
    }
    vertices
}
