//! The mobs the frame draws: which entity types have a model and a texture,
//! and where the server's mobs stand between two ticks.

use std::collections::HashMap;

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{
    ChickenState, MobModel, MobPlacement, PiglinState, QuadrupedState, SkeletonState, ZombieState,
    mob_vertices, placement_pose,
};
use voxel_render::player_model::{Arm, ArmPose, PlayerRenderState};
use voxel_render::scene::Vertex;
use voxel_world::EquipmentSlot;

use crate::held_items::{Hands, HeldItem};
use crate::overlays::{FrameView, packed_light};

/// A mob that wears a variant the server names: its entity type, the kind
/// of variant, and the registry that numbers the variants.
struct MobKind {
    type_name: &'static str,
    variant_kind: &'static str,
    registry: &'static str,
    pose: Pose,
}

/// Which model class poses a mob of a variant kind.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pose {
    /// `QuadrupedModel`.
    Quadruped,
    /// `ChickenModel`.
    Chicken,
}

/// The mobs drawn so far.
const KINDS: [MobKind; 3] = [
    MobKind {
        type_name: "minecraft:pig",
        variant_kind: "pig",
        registry: "minecraft:pig_variant",
        pose: Pose::Quadruped,
    },
    MobKind {
        type_name: "minecraft:cow",
        variant_kind: "cow",
        registry: "minecraft:cow_variant",
        pose: Pose::Quadruped,
    },
    MobKind {
        type_name: "minecraft:chicken",
        variant_kind: "chicken",
        registry: "minecraft:chicken_variant",
        pose: Pose::Chicken,
    },
];

/// Which model class poses a humanoid mob.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    /// `ZombieModel` (and `DrownedModel`, which swims).
    Zombie { drowned: bool },
    /// `SkeletonModel`.
    Skeleton,
    /// `PiglinModel`, or `ZombifiedPiglinModel`.
    Piglin { zombified: bool },
}

/// A humanoid mob of the families the humanoid models pose: its entity type,
/// its model layers and textures (grown and baby), the layer drawn over it
/// (the drowned's, the stray's) and its scale.
struct HumanoidKind {
    type_name: &'static str,
    adult_layer: &'static str,
    baby_layer: &'static str,
    adult_texture: &'static str,
    baby_texture: &'static str,
    /// The outer layer's model layers and textures, if it has one.
    outer: Option<[&'static str; 4]>,
    family: Family,
    scale: f32,
}

/// The zombies drawn so far.
const HUMANOIDS: [HumanoidKind; 12] = [
    HumanoidKind {
        type_name: "minecraft:zombie",
        adult_layer: "minecraft:zombie#main",
        baby_layer: "minecraft:zombie_baby#main",
        adult_texture: "entity/zombie/zombie",
        baby_texture: "entity/zombie/zombie_baby",
        outer: None,
        family: Family::Zombie { drowned: false },
        scale: 1.0,
    },
    HumanoidKind {
        type_name: "minecraft:husk",
        adult_layer: "minecraft:husk#main",
        baby_layer: "minecraft:husk_baby#main",
        adult_texture: "entity/zombie/husk",
        baby_texture: "entity/zombie/husk_baby",
        outer: None,
        family: Family::Zombie { drowned: false },
        scale: 1.0,
    },
    HumanoidKind {
        type_name: "minecraft:drowned",
        adult_layer: "minecraft:drowned#main",
        baby_layer: "minecraft:drowned_baby#main",
        adult_texture: "entity/zombie/drowned",
        baby_texture: "entity/zombie/drowned_baby",
        outer: Some([
            "minecraft:drowned#outer",
            "minecraft:drowned_baby#outer",
            "entity/zombie/drowned_outer_layer",
            "entity/zombie/drowned_outer_layer_baby",
        ]),
        family: Family::Zombie { drowned: true },
        scale: 1.0,
    },
    // `EntityRenderers`: the giant's renderer is made with scale six.
    HumanoidKind {
        type_name: "minecraft:giant",
        adult_layer: "minecraft:giant#main",
        baby_layer: "minecraft:giant#main",
        adult_texture: "entity/zombie/zombie",
        baby_texture: "entity/zombie/zombie",
        outer: None,
        family: Family::Zombie { drowned: false },
        scale: 6.0,
    },
    HumanoidKind {
        type_name: "minecraft:skeleton",
        adult_layer: "minecraft:skeleton#main",
        baby_layer: "minecraft:skeleton#main",
        adult_texture: "entity/skeleton/skeleton",
        baby_texture: "entity/skeleton/skeleton",
        outer: None,
        family: Family::Skeleton,
        scale: 1.0,
    },
    HumanoidKind {
        type_name: "minecraft:stray",
        adult_layer: "minecraft:stray#main",
        baby_layer: "minecraft:stray#main",
        adult_texture: "entity/skeleton/stray",
        baby_texture: "entity/skeleton/stray",
        outer: Some([
            "minecraft:stray#outer",
            "minecraft:stray#outer",
            "entity/skeleton/stray_overlay",
            "entity/skeleton/stray_overlay",
        ]),
        family: Family::Skeleton,
        scale: 1.0,
    },
    HumanoidKind {
        type_name: "minecraft:bogged",
        adult_layer: "minecraft:bogged#main",
        baby_layer: "minecraft:bogged#main",
        adult_texture: "entity/skeleton/bogged",
        baby_texture: "entity/skeleton/bogged",
        outer: Some([
            "minecraft:bogged#outer",
            "minecraft:bogged#outer",
            "entity/skeleton/bogged_overlay",
            "entity/skeleton/bogged_overlay",
        ]),
        family: Family::Skeleton,
        scale: 1.0,
    },
    HumanoidKind {
        type_name: "minecraft:parched",
        adult_layer: "minecraft:parched#main",
        baby_layer: "minecraft:parched#main",
        adult_texture: "entity/skeleton/parched",
        baby_texture: "entity/skeleton/parched",
        outer: None,
        family: Family::Skeleton,
        scale: 1.0,
    },
    HumanoidKind {
        type_name: "minecraft:wither_skeleton",
        adult_layer: "minecraft:wither_skeleton#main",
        baby_layer: "minecraft:wither_skeleton#main",
        adult_texture: "entity/skeleton/wither_skeleton",
        baby_texture: "entity/skeleton/wither_skeleton",
        outer: None,
        family: Family::Skeleton,
        scale: 1.0,
    },
    HumanoidKind {
        type_name: "minecraft:piglin",
        adult_layer: "minecraft:piglin#main",
        baby_layer: "minecraft:piglin_baby#main",
        adult_texture: "entity/piglin/piglin",
        baby_texture: "entity/piglin/piglin_baby",
        outer: None,
        family: Family::Piglin { zombified: false },
        scale: 1.0,
    },
    HumanoidKind {
        type_name: "minecraft:piglin_brute",
        adult_layer: "minecraft:piglin_brute#main",
        baby_layer: "minecraft:piglin_brute#main",
        adult_texture: "entity/piglin/piglin_brute",
        baby_texture: "entity/piglin/piglin_brute",
        outer: None,
        family: Family::Piglin { zombified: false },
        scale: 1.0,
    },
    HumanoidKind {
        type_name: "minecraft:zombified_piglin",
        adult_layer: "minecraft:zombified_piglin#main",
        baby_layer: "minecraft:zombified_piglin_baby#main",
        adult_texture: "entity/piglin/zombified_piglin",
        baby_texture: "entity/piglin/zombified_piglin_baby",
        outer: None,
        family: Family::Piglin { zombified: true },
        scale: 1.0,
    },
];

/// The variant a pig or a cow wears while the server has not set another
/// (`PigVariants.DEFAULT`, `CowVariants.TEMPERATE`).
const DEFAULT_VARIANT: &str = "minecraft:temperate";

/// The model layer a variant's model family asks for (`PigRenderer.models`,
/// `CowRenderer.models`): a pig's baby has one model whatever the family.
fn layer_name(kind: &MobKind, model: &str, baby: bool) -> String {
    match (kind.variant_kind, model, baby) {
        ("chicken", _, true) => "minecraft:chicken_baby#main".to_owned(),
        ("chicken", "cold", false) => "minecraft:cold_chicken#main".to_owned(),
        ("chicken", _, false) => "minecraft:chicken#main".to_owned(),
        ("pig", _, true) => "minecraft:pig_baby#main".to_owned(),
        ("pig", "cold", false) => "minecraft:cold_pig#main".to_owned(),
        ("pig", _, false) => "minecraft:pig#main".to_owned(),
        (cow, model, baby) => {
            let family = match model {
                "normal" => String::new(),
                other => format!("{other}_"),
            };
            let baby = if baby { "_baby" } else { "" };
            format!("minecraft:{family}{cow}{baby}#main")
        }
    }
}

/// An asset id of the data pack as the path under `textures/`.
fn texture_path(asset: &'static str) -> &'static str {
    asset.strip_prefix("minecraft:").unwrap_or(asset)
}

/// The paths (under `textures/`) of the textures the mobs wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    let variants = KINDS
        .iter()
        .flat_map(|kind| voxel_data::entity_type::variants(kind.variant_kind))
        .flat_map(|variant| {
            [
                texture_path(variant.asset),
                texture_path(variant.baby_asset),
            ]
        });
    let zombies = HUMANOIDS.iter().flat_map(|kind| {
        [kind.adult_texture, kind.baby_texture].into_iter().chain(
            kind.outer
                .into_iter()
                .flat_map(|outer| [outer[2], outer[3]]),
        )
    });
    variants.chain(zombies).chain(crate::sheep::textures())
}

/// The baked models of the mobs, which are posed anew for each one drawn.
pub struct MobModels {
    kinds: HashMap<i32, &'static MobKind>,
    humanoids: HashMap<i32, &'static HumanoidKind>,
    models: HashMap<String, MobModel>,
}

impl MobModels {
    pub fn new() -> Self {
        Self {
            kinds: KINDS
                .iter()
                .filter_map(|kind| Some((voxel_data::entity_type::id(kind.type_name)?, kind)))
                .collect(),
            humanoids: HUMANOIDS
                .iter()
                .filter_map(|kind| Some((voxel_data::entity_type::id(kind.type_name)?, kind)))
                .collect(),
            models: HashMap::new(),
        }
    }

    pub(crate) fn model(&mut self, layer: &str) -> Option<&mut MobModel> {
        if !self.models.contains_key(layer) {
            let baked = MobModel::bake(voxel_data::entity_model::layer(layer)?);
            self.models.insert(layer.to_owned(), baked);
        }
        self.models.get_mut(layer)
    }
}

impl Default for MobModels {
    fn default() -> Self {
        Self::new()
    }
}

/// Where a mob is drawn and how it is lit, from where it is between two ticks.
pub(crate) fn placement(
    world: &voxel_world::ClientWorld,
    snapshot: &voxel_world::EntitySnapshot,
    view: FrameView,
    scale: f32,
) -> MobPlacement {
    let (x, y, z) = snapshot.position;
    MobPlacement {
        feet: [
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        ],
        body_rot: snapshot.body_yaw,
        scale,
        light: packed_light(world, Vec3::new(x, y + 0.5, z)),
        death_time: snapshot.death_time,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
        red_overlay: snapshot.red_overlay,
    }
}

/// `LivingEntityRenderer` for the mobs the server added: each posed from
/// where it is between two ticks.
pub fn remote_mobs(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let Some(kind) = models.kinds.get(&entity.type_id).copied() else {
            continue;
        };
        // The variant the server named by its place in the registry, or the
        // default while it has said nothing; one the registry lacks is not drawn.
        let name = match entity.variant() {
            None => Some(DEFAULT_VARIANT),
            Some(id) => usize::try_from(id)
                .ok()
                .and_then(|id| session.registry_names(kind.registry).get(id))
                .map(String::as_str),
        };
        let Some(variant) =
            name.and_then(|name| voxel_data::entity_type::variant(kind.variant_kind, name))
        else {
            continue;
        };
        let layer = layer_name(kind, variant.model, entity.is_baby());
        let texture = texture_path(if entity.is_baby() {
            variant.baby_asset
        } else {
            variant.asset
        });
        let (Some(snapshot), Some(rect)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
        ) else {
            continue;
        };
        let Some(model) = models.model(&layer) else {
            continue;
        };
        let look = QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        };
        match kind.pose {
            Pose::Quadruped => model.setup_quadruped(&look),
            Pose::Chicken => {
                let (flap, flap_speed) = entity.chicken_flap(partial);
                model.setup_chicken(&ChickenState {
                    head_y_rot: look.head_y_rot,
                    x_rot: look.x_rot,
                    walk_position: look.walk_position,
                    walk_speed: look.walk_speed,
                    flap,
                    flap_speed,
                });
            }
        }
        vertices.extend(mob_vertices(
            model,
            &placement(world, &snapshot, view, 1.0),
            rect,
        ));
    }
    vertices.extend(remote_humanoids(session, models, textures, (view, held)));
    vertices.extend(crate::sheep::remote_sheep(session, models, textures, view));
    vertices
}

/// `LivingEntityRenderer` for the humanoid mobs the server added (zombies,
/// skeletons, piglins), each posed by its model's `setupAnim` from where it is
/// between two ticks, with what it holds in its hands. The attack swing and
/// the arm poses the held items ask for (a piglin's weapon or crossbow, a
/// spear) are not read yet.
fn remote_humanoids(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let Some(kind) = models.humanoids.get(&entity.type_id).copied() else {
            continue;
        };
        let baby = entity.is_baby();
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let (humanoid, holding_bow) = humanoid_pose(entity, &snapshot, kind, partial);
        let aggressive = entity.is_aggressive();
        let placement = placement(world, &snapshot, view, kind.scale);
        let mut first = true;
        for (layer, texture) in model_layers(kind, baby).into_iter().flatten() {
            let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
                continue;
            };
            pose_humanoid(model, kind, (humanoid, aggressive, holding_bow), entity);
            if first {
                first = false;
                Hands {
                    base: &placement_pose(&placement),
                    root: model.root(),
                    hand_shift: if matches!(kind.family, Family::Skeleton) {
                        1.0
                    } else {
                        0.0
                    },
                    baby,
                    light: placement.light,
                    right: entity.item_held_by_arm(false),
                    left: entity.item_held_by_arm(true),
                }
                .collect(held);
            }
            vertices.extend(mob_vertices(model, &placement, rect));
        }
    }
    vertices
}

/// What a humanoid mob's model is posed from, and whether its main hand holds
/// a bow: the humanoid pose from where it is between two ticks, and the
/// arms' poses its held items ask for (`AbstractSkeletonRenderer.getArmPose`:
/// a bow drawn by the main arm of an aggressive skeleton).
fn humanoid_pose(
    entity: &voxel_world::ClientEntity,
    snapshot: &voxel_world::EntitySnapshot,
    kind: &HumanoidKind,
    partial: f32,
) -> (PlayerRenderState, bool) {
    let main_left = entity.main_arm_is_left();
    let holding_bow = entity
        .equipment(EquipmentSlot::MainHand)
        .is_some_and(|stack| is_item(stack, "minecraft:bow"));
    let bow_arm = |left: bool| {
        if matches!(kind.family, Family::Skeleton)
            && left == main_left
            && entity.is_aggressive()
            && holding_bow
        {
            ArmPose::BowAndArrow
        } else {
            ArmPose::Empty
        }
    };
    let humanoid = PlayerRenderState {
        head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
        x_rot: snapshot.pitch,
        walk_position: snapshot.walk_position,
        walk_speed: snapshot.walk_speed,
        age_in_ticks: entity.tick_count() as f32 + partial,
        swim_amount: snapshot.swim_amount,
        attack_time: snapshot.attack_time,
        attack_arm: if snapshot.attack_arm_left {
            Arm::Left
        } else {
            Arm::Right
        },
        right_arm_pose: bow_arm(false),
        left_arm_pose: bow_arm(true),
        main_arm: if main_left { Arm::Left } else { Arm::Right },
        ..Default::default()
    };
    (humanoid, holding_bow)
}

/// The model layers and textures a humanoid is drawn in: its own, and the
/// outer one (a stray's clothes) when it has one.
fn model_layers(kind: &HumanoidKind, baby: bool) -> [Option<(&'static str, &'static str)>; 2] {
    [
        Some(if baby {
            (kind.baby_layer, kind.baby_texture)
        } else {
            (kind.adult_layer, kind.adult_texture)
        }),
        kind.outer.map(|outer| {
            if baby {
                (outer[1], outer[3])
            } else {
                (outer[0], outer[2])
            }
        }),
    ]
}

/// The model's `setupAnim` of the family.
fn pose_humanoid(
    model: &mut MobModel,
    kind: &HumanoidKind,
    (humanoid, aggressive, holding_bow): (PlayerRenderState, bool, bool),
    entity: &voxel_world::ClientEntity,
) {
    match kind.family {
        Family::Zombie { drowned } => model.setup_zombie(&ZombieState {
            humanoid,
            aggressive,
            drowned,
        }),
        Family::Skeleton => model.setup_skeleton(&SkeletonState {
            humanoid,
            aggressive,
            holding_bow,
        }),
        Family::Piglin { zombified } => model.setup_piglin(&PiglinState {
            humanoid,
            aggressive,
            zombified,
            baby: entity.is_baby(),
            dancing: entity.is_dancing(),
        }),
    }
}

/// Whether a stack is of the item with this name.
fn is_item(stack: &voxel_world::ClientItemStackState, name: &str) -> bool {
    voxel_data::item::registry::index(name)
        .is_some_and(|index| usize::try_from(stack.item_registry_id()) == Ok(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant of the data pack asks for model layers the game builds,
    /// adult and baby, and the textures it names are the ones the frame loads.
    #[test]
    fn every_variant_has_its_layers_and_textures() {
        let loaded: Vec<&str> = textures().collect();
        for kind in &KINDS {
            for variant in voxel_data::entity_type::variants(kind.variant_kind) {
                for baby in [false, true] {
                    let layer = layer_name(kind, variant.model, baby);
                    assert!(voxel_data::entity_model::layer(&layer).is_some(), "{layer}");
                }
                assert!(loaded.contains(&texture_path(variant.asset)));
                assert!(loaded.contains(&texture_path(variant.baby_asset)));
            }
        }
        assert!(voxel_data::entity_type::variant("pig", DEFAULT_VARIANT).is_some());
        assert!(voxel_data::entity_type::variant("cow", DEFAULT_VARIANT).is_some());
    }
}
