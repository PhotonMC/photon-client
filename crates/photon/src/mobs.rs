//! The mobs the frame draws: which entity types have a model and a texture,
//! and where the server's mobs stand between two ticks.

pub(crate) mod animals;
pub(crate) mod aquatic;
pub(crate) mod humanoids;
pub(crate) mod monsters;

use photon_core::ids::EntityTypeId;
use std::collections::HashMap;

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{
    ChickenState, MobModel, MobPlacement, PiglinArmPose, PiglinState, QuadrupedState,
    SkeletonState, ZombieState, mob_vertices, mob_vertices_coloured, placement_pose,
};
use photon_render::entity::player_model::{Arm, ArmPose, PlayerRenderState};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::EquipmentSlot;

use crate::equipment::armor::{Armor, Worn};
use crate::equipment::held_items::{HEAD_TRANSFORMS, Hands, HeldItem};
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
    /// The texture folder of a villager's profession layer, for the zombie
    /// villager (`VillagerProfessionLayer`).
    profession_folder: Option<&'static str>,
}

/// The zombies drawn so far.
const HUMANOIDS: [HumanoidKind; 13] = [
    HumanoidKind {
        type_name: "minecraft:zombie",
        adult_layer: "minecraft:zombie#main",
        baby_layer: "minecraft:zombie_baby#main",
        adult_texture: "entity/zombie/zombie",
        baby_texture: "entity/zombie/zombie_baby",
        outer: None,
        family: Family::Zombie { drowned: false },
        scale: 1.0,
        profession_folder: None,
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
        profession_folder: None,
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
        profession_folder: None,
    },
    // `EntityRenderers`: the giant's renderer is made with scale six.
    HumanoidKind {
        type_name: "minecraft:zombie_villager",
        adult_layer: "minecraft:zombie_villager#main",
        baby_layer: "minecraft:zombie_villager_baby#main",
        adult_texture: "entity/zombie_villager/zombie_villager",
        baby_texture: "entity/zombie_villager/zombie_villager_baby",
        outer: None,
        family: Family::Zombie { drowned: false },
        scale: 1.0,
        profession_folder: Some("zombie_villager"),
    },
    HumanoidKind {
        type_name: "minecraft:giant",
        adult_layer: "minecraft:giant#main",
        baby_layer: "minecraft:giant#main",
        adult_texture: "entity/zombie/zombie",
        baby_texture: "entity/zombie/zombie",
        outer: None,
        family: Family::Zombie { drowned: false },
        scale: 6.0,
        profession_folder: None,
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
        profession_folder: None,
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
        profession_folder: None,
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
        profession_folder: None,
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
        profession_folder: None,
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
        profession_folder: None,
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
        profession_folder: None,
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
        profession_folder: None,
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
        profession_folder: None,
    },
];

/// The variant a pig or a cow wears while the server has not set another
/// (`PigVariants.DEFAULT`, `CowVariants.TEMPERATE`).
const DEFAULT_VARIANT: &str = "minecraft:temperate";

/// The baked models of the mobs, which are posed anew for each one drawn.
pub struct MobModels {
    kinds: HashMap<EntityTypeId, &'static MobKind>,
    humanoids: HashMap<EntityTypeId, &'static HumanoidKind>,
    models: HashMap<String, MobModel>,
    /// What the glowing layers drew this frame (eyes), for the frame to hand
    /// to the renderer after the mobs.
    pub(crate) glow: Vec<Vertex>,
    /// What the energy swirls drew this frame (a charged creeper's).
    pub(crate) swirl: Vec<Vertex>,
    /// What the dissolving entities drew this frame (a dying dragon's body), and where a
    /// texel's mask is from it in the packed image.
    pub(crate) dissolve: Vec<Vertex>,
    pub(crate) dissolve_mask_offset: [f32; 2],
    /// A dying dragon's rays this frame.
    pub(crate) rays: Vec<Vertex>,
    /// The end portal and gateway surfaces this frame, and the game time and partial tick they
    /// slide by.
    pub(crate) portals: Vec<Vertex>,
    pub(crate) portal_time: (u64, f32),
    /// What shakes a creepy enderman: the renderer's own random source, which
    /// the reference does not seed.
    pub(crate) shake: photon_core::random::RandomSource,
    /// Where the skin of each player head in view is this frame.
    pub(crate) skull_skins: HashMap<(i32, i32, i32), photon_render::entity::textures::TextureRect>,
}

impl MobModels {
    pub fn new() -> Self {
        Self {
            kinds: KINDS
                .iter()
                .filter_map(|kind| Some((photon_data::entity_type::id(kind.type_name)?, kind)))
                .collect(),
            humanoids: HUMANOIDS
                .iter()
                .filter_map(|kind| Some((photon_data::entity_type::id(kind.type_name)?, kind)))
                .collect(),
            models: HashMap::new(),
            glow: Vec::new(),
            swirl: Vec::new(),
            dissolve: Vec::new(),
            dissolve_mask_offset: [0.0; 2],
            rays: Vec::new(),
            portals: Vec::new(),
            portal_time: (0, 0.0),
            shake: photon_core::random::RandomSource::create(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |elapsed| elapsed.as_nanos() as i64),
            ),
            skull_skins: HashMap::new(),
        }
    }

    pub(crate) fn model(&mut self, layer: &str) -> Option<&mut MobModel> {
        if !self.models.contains_key(layer) {
            let baked = MobModel::bake(photon_data::entity_model::layer(layer)?);
            self.models.insert(layer.to_owned(), baked);
        }
        self.models.get_mut(layer)
    }
}

impl MobModels {}

impl Default for MobModels {
    fn default() -> Self {
        Self::new()
    }
}

/// `LivingEntityRenderer.isBodyVisible`: an invisible mob's model, and the clothing layers drawn
/// with it, are left out; a spectator sees it as a ghost instead (`forceTransparent`, the colour
/// `0x26FFFFFF`).
pub(crate) fn visible_body(
    entity: &photon_world::ClientEntity,
    model: &MobModel,
    (placement, rect): (&MobPlacement, photon_render::entity::textures::TextureRect),
) -> Vec<Vertex> {
    if entity.is_glowing() {
        outline::record(entity, mob_vertices(model, placement, rect));
    }
    if !entity.is_invisible() {
        return mob_vertices(model, placement, rect);
    }
    placement.ghost.map_or_else(Vec::new, |camera| {
        photon_render::entity::mob_model::mob_vertices_blended(
            model,
            &placement_pose(placement),
            rect,
            (placement.light, GHOST_ALPHA),
            camera,
        )
    })
}

/// The alpha byte of `0x26FFFFFF`, the colour a ghost's model is drawn with.
const GHOST_ALPHA: u8 = 0x26;

pub(crate) mod outline;
pub(crate) use outline::{
    record_item as record_item_outline, record_item_holder, record_worn as with_outline,
    take as take_outlines, take_items as take_item_outlines,
};
mod wild;
pub use wild::*;
mod pose;
pub(crate) use pose::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant of the data pack asks for model layers the game builds,
    /// adult and baby, and the textures it names are the ones the frame loads.
    #[test]
    fn every_variant_has_its_layers_and_textures() {
        let loaded: Vec<&str> = textures().collect();
        for kind in &KINDS {
            for variant in photon_data::entity_type::variants(kind.variant_kind) {
                for baby in [false, true] {
                    let layer = layer_name(kind, variant.model, baby);
                    assert!(
                        photon_data::entity_model::layer(&layer).is_some(),
                        "{layer}"
                    );
                }
                assert!(loaded.contains(&texture_path(variant.asset)));
                assert!(loaded.contains(&texture_path(variant.baby_asset)));
            }
        }
        assert!(photon_data::entity_type::variant("pig", DEFAULT_VARIANT).is_some());
        assert!(photon_data::entity_type::variant("cow", DEFAULT_VARIANT).is_some());
    }
}
