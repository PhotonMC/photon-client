//! The animals that are not drawn from a variant registry: the goat, the
//! mooshroom with its mushrooms, the rabbit, the bee and the horse family
//! (`GoatRenderer`, `MushroomCowRenderer`, `MushroomCowMushroomLayer`,
//! `RabbitRenderer`, `BeeRenderer`, `HorseRenderer`, `DonkeyRenderer`,
//! `UndeadHorseRenderer`, `HorseMarkingLayer`).

use photon_net::connection::PlaySession;
use photon_render::entity::block::mooshroom_mushroom_poses;
use photon_render::entity::mob_model::{
    BeeState, CamelState, EquineKind, EquineState, GoatState, MobModel, MobPlacement,
    QuadrupedState, RabbitState, mob_vertices, overlay_code, placement_pose,
};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::ClientEntity;
use photon_world::animation_state::AnimationState;

use crate::equipment::held_items::HeldItem;
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

/// The horse's coats by variant id, grown and baby (`HorseRenderer`).
const HORSES: [[&str; 2]; 7] = [
    ["entity/horse/horse_white", "entity/horse/horse_white_baby"],
    [
        "entity/horse/horse_creamy",
        "entity/horse/horse_creamy_baby",
    ],
    [
        "entity/horse/horse_chestnut",
        "entity/horse/horse_chestnut_baby",
    ],
    ["entity/horse/horse_brown", "entity/horse/horse_brown_baby"],
    ["entity/horse/horse_black", "entity/horse/horse_black_baby"],
    ["entity/horse/horse_gray", "entity/horse/horse_gray_baby"],
    [
        "entity/horse/horse_darkbrown",
        "entity/horse/horse_darkbrown_baby",
    ],
];

/// The horse's markings by id, grown and baby; none is drawn for the first
/// (`HorseMarkingLayer`).
const MARKINGS: [Option<[&str; 2]>; 5] = [
    None,
    Some([
        "entity/horse/horse_markings_white",
        "entity/horse/horse_markings_white_baby",
    ]),
    Some([
        "entity/horse/horse_markings_whitefield",
        "entity/horse/horse_markings_whitefield_baby",
    ]),
    Some([
        "entity/horse/horse_markings_whitedots",
        "entity/horse/horse_markings_whitedots_baby",
    ]),
    Some([
        "entity/horse/horse_markings_blackdots",
        "entity/horse/horse_markings_blackdots_baby",
    ]),
];

/// The other equines' textures and model layers, grown and baby: donkey,
/// mule, skeleton horse and zombie horse.
const OTHER_EQUINES: [(&str, [&str; 2], [&str; 2], bool); 4] = [
    (
        "minecraft:donkey",
        ["entity/horse/donkey", "entity/horse/donkey_baby"],
        ["minecraft:donkey#main", "minecraft:donkey_baby#main"],
        true,
    ),
    (
        "minecraft:mule",
        ["entity/horse/mule", "entity/horse/mule_baby"],
        ["minecraft:mule#main", "minecraft:mule_baby#main"],
        true,
    ),
    (
        "minecraft:skeleton_horse",
        [
            "entity/horse/horse_skeleton",
            "entity/horse/horse_skeleton_baby",
        ],
        [
            "minecraft:skeleton_horse#main",
            "minecraft:skeleton_horse_baby#main",
        ],
        false,
    ),
    (
        "minecraft:zombie_horse",
        [
            "entity/horse/horse_zombie",
            "entity/horse/horse_zombie_baby",
        ],
        [
            "minecraft:zombie_horse#main",
            "minecraft:zombie_horse_baby#main",
        ],
        false,
    ),
];

/// The camel's textures, grown and baby, and the husk's (`CamelRenderer`,
/// `CamelHuskRenderer`: the husk is a grown camel's model in its own skin).
const CAMELS: [&str; 3] = [
    "entity/camel/camel",
    "entity/camel/camel_baby",
    "entity/camel/camel_husk",
];

/// The llama's coats by variant id, grown and baby (`LlamaRenderer`).
const LLAMAS: [[&str; 2]; 4] = [
    [
        "entity/llama/llama_creamy",
        "entity/llama/llama_creamy_baby",
    ],
    ["entity/llama/llama_white", "entity/llama/llama_white_baby"],
    ["entity/llama/llama_brown", "entity/llama/llama_brown_baby"],
    ["entity/llama/llama_gray", "entity/llama/llama_gray_baby"],
];

/// The bee's textures: grown then baby, each calm or angry, without and with
/// nectar (`BeeRenderer`); indexed by baby, angry, nectar.
const BEES: [[[&str; 2]; 2]; 2] = [
    [
        ["entity/bee/bee", "entity/bee/bee_nectar"],
        ["entity/bee/bee_angry", "entity/bee/bee_angry_nectar"],
    ],
    [
        ["entity/bee/bee_baby", "entity/bee/bee_nectar_baby"],
        [
            "entity/bee/bee_angry_baby",
            "entity/bee/bee_angry_nectar_baby",
        ],
    ],
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
        .chain(BEES.into_iter().flatten().flatten())
        .chain(CAMELS)
        .chain(LLAMAS.into_iter().flatten())
        .chain(HORSES.into_iter().flatten())
        .chain(MARKINGS.into_iter().flatten().flatten())
        .chain(
            OTHER_EQUINES
                .into_iter()
                .flat_map(|(_, textures, _, _)| textures),
        )
}

/// Which of these animals an entity is.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Animal {
    Goat,
    Mooshroom,
    Rabbit,
    Bee,
    /// A horse (its coat and markings), or one of the other equines by the
    /// index of [`OTHER_EQUINES`].
    Horse,
    Equine(usize),
    /// The camel, or the camel husk.
    Camel(bool),
    /// The llama, or the trader's.
    Llama(bool),
}

/// Whether a bee is angry: its anger ends later than the game time now
/// (`NeutralMob.isAngry`).
fn bee_is_angry(entity: &ClientEntity, game_time: u64) -> bool {
    let end = entity.anger_end();
    end > 0 && end - game_time as i64 > 0
}

/// The texture and the model layer an animal is drawn in.
fn appearance(
    animal: Animal,
    entity: &ClientEntity,
    game_time: u64,
) -> (&'static str, &'static str) {
    let baby = entity.is_baby();
    let age = usize::from(baby);
    match animal {
        Animal::Llama(trader) => (
            LLAMAS[usize::try_from(entity.llama_variant()).unwrap_or(0)][age],
            match (trader, baby) {
                (false, false) => "minecraft:llama#main",
                (false, true) => "minecraft:llama_baby#main",
                (true, false) => "minecraft:trader_llama#main",
                (true, true) => "minecraft:trader_llama_baby#main",
            },
        ),
        Animal::Camel(husk) => (
            CAMELS[if husk { 2 } else { age }],
            if baby && !husk {
                "minecraft:camel_baby#main"
            } else {
                "minecraft:camel#main"
            },
        ),
        Animal::Horse => (
            HORSES[usize::try_from(entity.horse_look().0).unwrap_or(0)][age],
            if baby {
                "minecraft:horse_baby#main"
            } else {
                "minecraft:horse#main"
            },
        ),
        Animal::Equine(index) => (OTHER_EQUINES[index].1[age], OTHER_EQUINES[index].2[age]),
        Animal::Bee => (
            BEES[age][usize::from(bee_is_angry(entity, game_time))]
                [usize::from(entity.bee_has_nectar())],
            if baby {
                "minecraft:bee_baby#main"
            } else {
                "minecraft:bee#main"
            },
        ),
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
    (look, partial, game_time): (QuadrupedState, f32, u64),
) {
    let baby = entity.is_baby();
    match animal {
        // A baby llama carries no chest (`LlamaRenderer.extractRenderState`).
        Animal::Llama(_) => model.setup_llama(&look, !baby && entity.horse_has_chest()),
        Animal::Camel(_) => {
            let camel = entity.horse().map(|horse| &horse.camel);
            let state = |pick: fn(&photon_world::mobs::horse::CamelState) -> AnimationState| {
                camel.map(pick).unwrap_or_default()
            };
            model.setup_camel(&CamelState {
                quadruped: look,
                age_in_ticks: entity.tick_count() as f32 + partial,
                // The cooldown is only counted for a camel its own player rides.
                jump_cooldown: 0.0,
                sit: state(|camel| camel.sit),
                sit_pose: state(|camel| camel.sit_pose),
                sit_up: state(|camel| camel.sit_up),
                idle: state(|camel| camel.idle),
                dash: state(|camel| camel.dash),
                baby,
            });
        }
        Animal::Horse | Animal::Equine(_) => {
            let donkey = matches!(animal, Animal::Equine(index) if OTHER_EQUINES[index].3);
            let anim = entity.horse();
            model.setup_equine(&EquineState {
                kind: match (donkey, baby) {
                    (false, false) => EquineKind::Horse,
                    (false, true) => EquineKind::BabyHorse,
                    (true, false) => EquineKind::Donkey,
                    (true, true) => EquineKind::BabyDonkey,
                },
                quadruped: look,
                age_in_ticks: entity.tick_count() as f32 + partial,
                eat_animation: anim.map_or(0.0, |horse| horse.eat_animation(partial)),
                stand_animation: anim.map_or(0.0, |horse| horse.stand_animation(partial)),
                feeding_animation: anim.map_or(0.0, |horse| horse.mouth_animation(partial)),
                animate_tail: anim.is_some_and(|horse| horse.tail_counter() > 0),
                // Whether the animal is in water is not tracked for mobs yet.
                in_water: false,
                age_scale: if baby { 0.5 } else { 1.0 },
                has_chest: entity.horse_has_chest(),
            });
        }
        Animal::Bee => {
            let (x, y, z) = entity.motion();
            model.setup_bee(&BeeState {
                age_in_ticks: entity.tick_count() as f32 + partial,
                roll_amount: entity.bee_roll_amount(partial),
                has_stinger: !entity.bee_has_stung(),
                on_ground: entity.on_ground() && x * x + y * y + z * z < 1.0e-7,
                angry: bee_is_angry(entity, game_time),
                baby,
            });
        }
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
        photon_data::block::registry::block(MOOSHROOMS[mooshroom_variant(entity)].1)
            .map(photon_data::block::registry::Block::default_state_id),
        model
            .root()
            .children()
            .iter()
            .find(|part| part.name() == "head"),
    ) else {
        return;
    };
    for pose in mooshroom_mushroom_poses(&placement_pose(placed), head) {
        // `MushroomCowMushroomLayer` takes the entity's overlay.
        held.push(HeldItem::block(
            state,
            pose,
            placed.light,
            overlay_code(placed.red_overlay, 0.0),
        ));
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
    vertices: &mut Vec<Vertex>,
) {
    let kinds: Vec<(i32, Animal)> = [
        ("minecraft:goat", Animal::Goat),
        ("minecraft:mooshroom", Animal::Mooshroom),
        ("minecraft:rabbit", Animal::Rabbit),
        ("minecraft:bee", Animal::Bee),
        ("minecraft:llama", Animal::Llama(false)),
        ("minecraft:trader_llama", Animal::Llama(true)),
        ("minecraft:camel", Animal::Camel(false)),
        ("minecraft:camel_husk", Animal::Camel(true)),
        ("minecraft:horse", Animal::Horse),
        ("minecraft:donkey", Animal::Equine(0)),
        ("minecraft:mule", Animal::Equine(1)),
        ("minecraft:skeleton_horse", Animal::Equine(2)),
        ("minecraft:zombie_horse", Animal::Equine(3)),
    ]
    .into_iter()
    .filter_map(|(name, animal)| Some((photon_data::entity_type::id(name)?, animal)))
    .collect();
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.entities().drawn() {
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
            head_y_rot: photon_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        };
        let (texture, layer) = appearance(animal, entity, world.game_time());
        let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
            continue;
        };
        pose(model, animal, entity, (look, partial, world.game_time()));
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
        if animal == Animal::Mooshroom {
            mushrooms(model, entity, &placed, held);
        }
        // `HorseMarkingLayer`: the markings over the coat, not on an invisible horse.
        if animal == Animal::Horse
            && !entity.is_invisible()
            && let Some(markings) = usize::try_from(entity.horse_look().1)
                .ok()
                .and_then(|index| MARKINGS[index])
            && let Some(marking) = textures.rect(markings[usize::from(entity.is_baby())])
        {
            vertices.extend(mob_vertices(model, &placed, marking));
        }
    }
}
