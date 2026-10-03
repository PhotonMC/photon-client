//! The farm animals that are not drawn from a variant registry: the goat, the
//! mooshroom with its mushrooms and the rabbit (`GoatRenderer`,
//! `MushroomCowRenderer`, `MushroomCowMushroomLayer`, `RabbitRenderer`).

use voxel_net::connection::PlaySession;
use voxel_render::entity_block::mooshroom_mushroom_poses;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{
    GoatState, MobModel, MobPlacement, QuadrupedState, RabbitState, mob_vertices, placement_pose,
};
use voxel_render::scene::Vertex;
use voxel_world::ClientEntity;

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

/// The rabbit's variants by id (`Rabbit.Variant`): the id and the name its
/// textures carry, grown and baby. An id the variants lack is the first.
const RABBITS: [(i32, [&str; 2]); 7] = [
    (
        0,
        [
            "entity/rabbit/rabbit_brown",
            "entity/rabbit/rabbit_brown_baby",
        ],
    ),
    (
        1,
        [
            "entity/rabbit/rabbit_white",
            "entity/rabbit/rabbit_white_baby",
        ],
    ),
    (
        2,
        [
            "entity/rabbit/rabbit_black",
            "entity/rabbit/rabbit_black_baby",
        ],
    ),
    (
        3,
        [
            "entity/rabbit/rabbit_white_splotched",
            "entity/rabbit/rabbit_white_splotched_baby",
        ],
    ),
    (
        4,
        [
            "entity/rabbit/rabbit_gold",
            "entity/rabbit/rabbit_gold_baby",
        ],
    ),
    (
        5,
        [
            "entity/rabbit/rabbit_salt",
            "entity/rabbit/rabbit_salt_baby",
        ],
    ),
    (
        99,
        [
            "entity/rabbit/rabbit_caerbannog",
            "entity/rabbit/rabbit_caerbannog_baby",
        ],
    ),
];

/// The paths (under `textures/`) of the textures these animals wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    GOAT_TEXTURES
        .into_iter()
        .chain(MOOSHROOMS.into_iter().flat_map(|(textures, _)| textures))
        .chain(RABBITS.into_iter().flat_map(|(_, textures)| textures))
}

/// Which of these animals an entity is.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Animal {
    Goat,
    Mooshroom,
    Rabbit,
}

/// The texture and the model layer an animal is drawn in.
fn appearance(animal: Animal, entity: &ClientEntity) -> (&'static str, &'static str) {
    let baby = entity.is_baby();
    let age = usize::from(baby);
    match animal {
        Animal::Rabbit => {
            let variant = RABBITS
                .iter()
                .find(|(id, _)| *id == entity.rabbit_variant())
                .map_or(&RABBITS[0].1, |(_, textures)| textures);
            (
                variant[age],
                if baby {
                    "minecraft:rabbit_baby#main"
                } else {
                    "minecraft:rabbit#main"
                },
            )
        }
        Animal::Goat => (
            GOAT_TEXTURES[age],
            if baby {
                "minecraft:goat_baby#main"
            } else {
                "minecraft:goat#main"
            },
        ),
        Animal::Mooshroom => (
            MOOSHROOMS[mooshroom_variant(entity)].0[age],
            if baby {
                "minecraft:mooshroom_baby#main"
            } else {
                "minecraft:mooshroom#main"
            },
        ),
    }
}

fn mooshroom_variant(entity: &ClientEntity) -> usize {
    usize::try_from(entity.mooshroom_variant()).unwrap_or(0)
}

/// The model's `setupAnim` of the animal.
fn pose(
    model: &mut MobModel,
    animal: Animal,
    entity: &ClientEntity,
    look: QuadrupedState,
    partial: f32,
) {
    let baby = entity.is_baby();
    match animal {
        Animal::Rabbit => {
            let states = entity.rabbit();
            model.setup_rabbit(&RabbitState {
                quadruped: look,
                age_in_ticks: entity.tick_count() as f32 + partial,
                hop: states.map(|rabbit| rabbit.hop).unwrap_or_default(),
                idle_head_tilt: states
                    .map(|rabbit| rabbit.idle_head_tilt)
                    .unwrap_or_default(),
                baby,
            });
        }
        Animal::Goat => {
            let (has_left_horn, has_right_horn) = entity.goat_horns();
            model.setup_goat(&GoatState {
                quadruped: look,
                has_left_horn,
                has_right_horn,
                ramming_x_head_rot: entity.goat_ramming_x_head_rot(),
                baby,
            });
        }
        Animal::Mooshroom => model.setup_quadruped(&look),
    }
}

/// `MushroomCowMushroomLayer`: the mushrooms on a grown, visible mooshroom.
fn mushrooms(
    model: &MobModel,
    entity: &ClientEntity,
    placed: &MobPlacement,
    held: &mut Vec<HeldItem>,
) {
    if entity.is_baby() || entity.is_invisible() {
        return;
    }
    let (Some(state), Some(head)) = (
        voxel_data::block::registry::block(MOOSHROOMS[mooshroom_variant(entity)].1)
            .map(voxel_data::block::registry::Block::default_state_id),
        model
            .root()
            .children()
            .iter()
            .find(|part| part.name() == "head"),
    ) else {
        return;
    };
    for pose in mooshroom_mushroom_poses(&placement_pose(placed), head) {
        held.push(HeldItem::block(state, pose, placed.light));
    }
}

/// `GoatRenderer`, `MushroomCowRenderer` and `RabbitRenderer` for the goats,
/// mooshrooms and rabbits the server added. A mooshroom's mushrooms on an
/// invisible animal that glows would be drawn as outlines only, which is not
/// drawn yet.
pub fn remote_farm_animals(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let kinds: Vec<(i32, Animal)> = [
        ("minecraft:goat", Animal::Goat),
        ("minecraft:mooshroom", Animal::Mooshroom),
        ("minecraft:rabbit", Animal::Rabbit),
    ]
    .into_iter()
    .filter_map(|(name, animal)| Some((voxel_data::entity_type::id(name)?, animal)))
    .collect();
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let Some(animal) = kinds
            .iter()
            .find(|(id, _)| *id == entity.type_id)
            .map(|(_, animal)| *animal)
        else {
            continue;
        };
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let look = QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        };
        let (texture, layer) = appearance(animal, entity);
        let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
            continue;
        };
        pose(model, animal, entity, look, partial);
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
        if animal == Animal::Mooshroom {
            mushrooms(model, entity, &placed, held);
        }
    }
    vertices
}
