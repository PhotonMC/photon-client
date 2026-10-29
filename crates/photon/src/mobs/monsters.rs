//! The monsters, and the other mobs that are posed by one model class of their
//! own, the frame draws: spiders, the creeper, the enderman, silverfish, the
//! blaze, ghast, phantom, bat, vex and allay (`SpiderRenderer`,
//! `CreeperRenderer`, `EndermanRenderer` and the like).

pub(crate) mod cube_mobs;
pub(crate) mod ender_dragons;
pub(crate) mod hoglins;
pub(crate) mod ravagers;
pub(crate) mod shulkers;
pub(crate) mod striders;

use photon_net::connection::PlaySession;
use photon_render::entity::block::carried_block_pose;
use photon_render::entity::held_item::{allay_hand_pose, vex_hand_pose};
use photon_render::entity::mob_model::{
    AllayRenderState, BatRenderState, BreezeRenderState, CreakingRenderState, EndermanState,
    MobModel, MobPlacement, ParrotPose, ParrotRenderState, QuadrupedState, VexRenderState,
    WardenRenderState, WitherRenderState, creeper_scale, creeper_white_overlay,
    mob_vertices_emissive, mob_vertices_emissive_faded, mob_vertices_glow, mob_vertices_scrolled,
    mob_vertices_swirl, placement_pose,
};
use photon_render::entity::player_model::{Arm, Part, PlayerRenderState};
use photon_render::entity::textures::EntityTextures;
use photon_render::render_types::{POWER_COLOUR, POWER_TEXTURE, swirl_offset};
use photon_render::scene::Vertex;
use photon_world::mobs::creaking::CreakingState;
use photon_world::{EntitySnapshot, EquipmentSlot};

use crate::equipment::held_items::{HeldItem, Place};
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
    /// `BlazeModel`, which a renderer lights fully by its blocks.
    Blaze,
    /// `GhastModel`, which opens its mouth (another texture) to charge.
    Ghast,
    /// `PhantomModel`, which grows with its size and tips with its pitch.
    Phantom,
    /// `BatModel`, which hangs or flies by two animations.
    Bat,
    /// `VexModel`, which a renderer lights fully and which charges (another texture).
    Vex,
    /// `AllayModel`, which a renderer lights fully.
    Allay,
    /// `ParrotModel`, in the texture of its variant.
    Parrot,
    /// `BreezeModel`, with its wind and its eyes drawn over it.
    Breeze,
    /// `WardenModel`, with its four glowing layers drawn over it.
    Warden,
    /// `CreakingModel`, with its eyes lit while it is active.
    Creaking,
    /// `WitherBossModel`, scaled, lit fully, with its armor's swirl while it is hurt.
    Wither,
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

const MONSTERS: [Monster; 17] = [
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
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
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
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:blaze",
        layer: "minecraft:blaze#main",
        texture: "entity/blaze/blaze",
        pose: Pose::Blaze,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:ghast",
        layer: "minecraft:ghast#main",
        texture: "entity/ghast/ghast",
        pose: Pose::Ghast,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:phantom",
        layer: "minecraft:phantom#main",
        texture: "entity/phantom/phantom",
        pose: Pose::Phantom,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:bat",
        layer: "minecraft:bat#main",
        texture: "entity/bat/bat",
        pose: Pose::Bat,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:vex",
        layer: "minecraft:vex#main",
        texture: "entity/illager/vex",
        pose: Pose::Vex,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:allay",
        layer: "minecraft:allay#main",
        texture: "entity/allay/allay",
        pose: Pose::Allay,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:parrot",
        layer: "minecraft:parrot#main",
        texture: "entity/parrot/parrot_red_blue",
        pose: Pose::Parrot,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:breeze",
        layer: "minecraft:breeze#main",
        texture: "entity/breeze/breeze",
        pose: Pose::Breeze,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:warden",
        layer: "minecraft:warden#main",
        texture: "entity/warden/warden",
        pose: Pose::Warden,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:creaking",
        layer: "minecraft:creaking#main",
        texture: "entity/creaking/creaking",
        pose: Pose::Creaking,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:wither",
        layer: "minecraft:wither#main",
        texture: "entity/wither/wither",
        pose: Pose::Wither,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
    },
];

/// The wind and the eyes of a breeze (`BreezeWindLayer`, `BreezeEyesLayer`): their
/// model layers and textures.
const BREEZE_WIND: (&str, &str) = ("minecraft:breeze#wind", "entity/breeze/breeze_wind");
const BREEZE_EYES: (&str, &str) = ("minecraft:breeze#eyes", "entity/breeze/breeze_eyes");

/// The glowing layers of a warden (`WardenRenderer`), in the order they are
/// added: each one's model layer and texture.
const WARDEN_BIOLUMINESCENT: (&str, &str) = (
    "minecraft:warden#bioluminescent",
    "entity/warden/warden_bioluminescent_layer",
);
const WARDEN_SPOTS: &str = "minecraft:warden#pulsating_spots";
const WARDEN_SPOTS_1: &str = "entity/warden/warden_pulsating_spots_1";
const WARDEN_SPOTS_2: &str = "entity/warden/warden_pulsating_spots_2";
const WARDEN_TENDRILS: (&str, &str) = ("minecraft:warden#tendrils", "entity/warden/warden");
const WARDEN_HEART: (&str, &str) = ("minecraft:warden#heart", "entity/warden/warden_heart");

/// A creaking's eyes (`CreakingRenderer`): their model layer and texture.
const CREAKING_EYES: (&str, &str) = ("minecraft:creaking#eyes", "entity/creaking/creaking_eyes");

/// A wither's armor (`WitherArmorLayer`) and the texture it wears while it is
/// invulnerable (`WitherBossRenderer.getTextureLocation`).
const WITHER_ARMOR: (&str, &str) = ("minecraft:wither#armor", "entity/wither/wither_armor");
const WITHER_INVULNERABLE_TEXTURE: &str = "entity/wither/wither_invulnerable";

/// The texture a vex charges in (`VexRenderer.getTextureLocation`).
const VEX_CHARGING_TEXTURE: &str = "entity/illager/vex_charging";

/// The texture a ghast opens its mouth in (`GhastRenderer.getTextureLocation`).
const GHAST_SHOOTING_TEXTURE: &str = "entity/ghast/ghast_shooting";

mod wither;
pub use wither::*;
mod remote;
pub use remote::*;
