//! The monsters, and the other mobs that are posed by one model class of their
//! own, the frame draws: spiders, the creeper, the enderman, silverfish, the
//! blaze, ghast, phantom, bat, vex and allay (`SpiderRenderer`,
//! `CreeperRenderer`, `EndermanRenderer` and the like).

use voxel_net::connection::PlaySession;
use voxel_render::entity_block::carried_block_pose;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::held_item::{allay_hand_pose, vex_hand_pose};
use voxel_render::mob_model::{
    AllayRenderState, BatRenderState, BreezeRenderState, CreakingRenderState, EndermanState,
    MobModel, MobPlacement, ParrotPose, ParrotRenderState, QuadrupedState, VexRenderState,
    WardenRenderState, WitherRenderState, creeper_scale, creeper_white_overlay, mob_vertices,
    mob_vertices_emissive, mob_vertices_emissive_faded, mob_vertices_glow, mob_vertices_scrolled,
    mob_vertices_swirl, placement_pose,
};
use voxel_render::player_model::{Arm, Part, PlayerRenderState};
use voxel_render::render_types::{POWER_COLOUR, POWER_TEXTURE, swirl_offset};
use voxel_render::scene::Vertex;
use voxel_world::creaking::CreakingState;
use voxel_world::{ClientEntity, EntitySnapshot, EquipmentSlot};

use crate::held_items::{HeldItem, Place};
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
    Monster {
        type_name: "minecraft:blaze",
        layer: "minecraft:blaze#main",
        texture: "entity/blaze/blaze",
        pose: Pose::Blaze,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:ghast",
        layer: "minecraft:ghast#main",
        texture: "entity/ghast/ghast",
        pose: Pose::Ghast,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:phantom",
        layer: "minecraft:phantom#main",
        texture: "entity/phantom/phantom",
        pose: Pose::Phantom,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:bat",
        layer: "minecraft:bat#main",
        texture: "entity/bat/bat",
        pose: Pose::Bat,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:vex",
        layer: "minecraft:vex#main",
        texture: "entity/illager/vex",
        pose: Pose::Vex,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:allay",
        layer: "minecraft:allay#main",
        texture: "entity/allay/allay",
        pose: Pose::Allay,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:parrot",
        layer: "minecraft:parrot#main",
        texture: "entity/parrot/parrot_red_blue",
        pose: Pose::Parrot,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:breeze",
        layer: "minecraft:breeze#main",
        texture: "entity/breeze/breeze",
        pose: Pose::Breeze,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:warden",
        layer: "minecraft:warden#main",
        texture: "entity/warden/warden",
        pose: Pose::Warden,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:creaking",
        layer: "minecraft:creaking#main",
        texture: "entity/creaking/creaking",
        pose: Pose::Creaking,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
    },
    Monster {
        type_name: "minecraft:wither",
        layer: "minecraft:wither#main",
        texture: "entity/wither/wither",
        pose: Pose::Wither,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
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
        .chain([POWER_TEXTURE, GHAST_SHOOTING_TEXTURE, VEX_CHARGING_TEXTURE])
        .chain(voxel_world::parrot::VARIANT_TEXTURES)
        .chain([BREEZE_WIND.1, BREEZE_EYES.1])
        .chain([CREAKING_EYES.1, WITHER_ARMOR.1, WITHER_INVULNERABLE_TEXTURE])
        .chain([
            WARDEN_BIOLUMINESCENT.1,
            WARDEN_SPOTS_1,
            WARDEN_SPOTS_2,
            WARDEN_HEART.1,
        ])
}

/// The texture a monster wears now: a ghast's mouth open to charge, a vex charging,
/// a parrot's variant, else its own.
fn texture_of(monster: &Monster, entity: &ClientEntity, partial: f32) -> &'static str {
    match monster.pose {
        Pose::Ghast if entity.is_ghast_charging() => GHAST_SHOOTING_TEXTURE,
        Pose::Vex if entity.is_vex_charging() => VEX_CHARGING_TEXTURE,
        Pose::Wither if wither_is_invulnerable_texture(entity, partial) => {
            WITHER_INVULNERABLE_TEXTURE
        }
        Pose::Parrot => voxel_world::parrot::variant_texture(entity.parrot_variant()),
        _ => monster.texture,
    }
}

/// `ItemInHandLayer` for a vex or an allay: each hand's stack, the right hand's
/// first, with the pose its model's `translateToHand` leaves (a left-handed mob's
/// main hand is its left).
fn flier_hand_items(
    root: &Part,
    (monster, entity): (&Monster, &ClientEntity),
    placed: &MobPlacement,
    held: &mut Vec<HeldItem>,
) {
    fn child<'a>(part: &'a Part, name: &str) -> Option<&'a Part> {
        part.children().iter().find(|child| child.name() == name)
    }
    let base = placement_pose(placed);
    let Some(inner) = child(root, "root") else {
        return;
    };
    let Some(body) = child(inner, "body") else {
        return;
    };
    let (main, off) = (
        entity.equipment(EquipmentSlot::MainHand),
        entity.equipment(EquipmentSlot::OffHand),
    );
    let (right, left) = if entity.main_arm_is_left() {
        (off, main)
    } else {
        (main, off)
    };
    for (side, place, name, stack) in [
        (Arm::Right, Place::Right, "right_arm", right),
        (Arm::Left, Place::Left, "left_arm", left),
    ] {
        let (Some(stack), Some(arm)) = (stack, child(body, name)) else {
            continue;
        };
        let pose = if monster.pose == Pose::Vex {
            vex_hand_pose(&base, (inner, body, arm), side)
        } else {
            // The allay tilts the item as its right arm is, whichever hand holds it.
            let Some(right_arm) = child(body, "right_arm") else {
                continue;
            };
            allay_hand_pose(&base, (inner, body, right_arm), side)
        };
        held.push(HeldItem::stack(stack, pose, place, placed.light));
    }
}

/// `BreezeRenderer.extractRenderState`: the animation states the breeze runs.
fn breeze_state(entity: &ClientEntity, partial: f32) -> BreezeRenderState {
    let own = entity.breeze().copied().unwrap_or_default();
    BreezeRenderState {
        age_in_ticks: entity.tick_count() as f32 + partial,
        idle: own.idle,
        shoot: own.shoot,
        slide: own.slide,
        slide_back: own.slide_back,
        inhale: own.inhale,
        long_jump: own.long_jump,
    }
}

/// `WardenRenderer.extractRenderState`: the look, the walk, the pulses and the
/// animation states the warden runs.
fn warden_state(entity: &ClientEntity, look: QuadrupedState, partial: f32) -> WardenRenderState {
    let own = entity.warden().copied().unwrap_or_default();
    WardenRenderState {
        quadruped: look,
        age_in_ticks: entity.tick_count() as f32 + partial,
        tendril_animation: own.tendril_animation(partial),
        roar: own.roar,
        sniff: own.sniff,
        emerge: own.emerge,
        digging: own.digging,
        attack: own.attack,
        sonic_boom: own.sonic_boom,
    }
}

/// A warden's glowing layers over its body, in the order the renderer adds
/// them (`LivingEntityEmissiveLayer`): the bioluminescent parts, the two
/// pulsating spots that take turns, the tendrils and the heart, each faded by
/// its own alpha and posed as the body is.
fn warden_layers(
    models: &mut MobModels,
    textures: &EntityTextures,
    (entity, state, partial): (&ClientEntity, &WardenRenderState, f32),
    placed: &MobPlacement,
) -> Vec<Vertex> {
    let own = entity.warden().copied().unwrap_or_default();
    let age = state.age_in_ticks;
    let pi = std::f64::consts::PI as f32;
    let spots = |phase: f32| {
        (voxel_core::math::mth_cos_f64(f64::from(age * 0.045 + phase)) * 0.25).max(0.0)
    };
    let layers = [
        (WARDEN_BIOLUMINESCENT.0, WARDEN_BIOLUMINESCENT.1, 1.0),
        (WARDEN_SPOTS, WARDEN_SPOTS_1, spots(0.0)),
        (WARDEN_SPOTS, WARDEN_SPOTS_2, spots(pi)),
        (
            WARDEN_TENDRILS.0,
            WARDEN_TENDRILS.1,
            state.tendril_animation,
        ),
        (WARDEN_HEART.0, WARDEN_HEART.1, own.heart_animation(partial)),
    ];
    let mut vertices = Vec::new();
    for (layer, texture, alpha) in layers {
        // `LivingEntityEmissiveLayer.submit`: nothing below this alpha.
        if alpha <= 1.0e-5 {
            continue;
        }
        let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
            continue;
        };
        model.setup_warden(state);
        vertices.extend(mob_vertices_emissive_faded(model, placed, rect, alpha));
    }
    vertices
}

/// `WitherBossRenderer.extractRenderState`: what is left of the invulnerable
/// time, less the partial tick, or zero.
fn wither_invulnerable_ticks(entity: &ClientEntity, partial: f32) -> f32 {
    let ticks = entity.wither().map_or(0, |own| own.invulnerable_ticks());
    if ticks > 0 {
        ticks as f32 - partial
    } else {
        0.0
    }
}

/// `WitherBossRenderer.getTextureLocation`: the pale texture while it is
/// invulnerable, flashing the other for a tick in five over the last 80.
fn wither_is_invulnerable_texture(entity: &ClientEntity, partial: f32) -> bool {
    let ticks = wither_invulnerable_ticks(entity, partial).floor() as i32;
    ticks > 0 && (ticks > 80 || ticks / 5 % 2 != 1)
}

/// `WitherBoss.isPowered`: half its health or less.
fn wither_is_powered(entity: &ClientEntity) -> bool {
    match (entity.health(), entity.max_health()) {
        (Some(health), Some(max)) => health <= max / 2.0,
        _ => false,
    }
}

/// `CreakingRenderer.extractRenderState`: the look, the walk, and the animation
/// states the creaking runs; what a torn-down one does to the renderer's own
/// state is in [`creaking_tear_down`].
fn creaking_state(
    entity: &ClientEntity,
    look: QuadrupedState,
    partial: f32,
) -> CreakingRenderState {
    let own = entity.creaking();
    CreakingRenderState {
        quadruped: look,
        age_in_ticks: entity.tick_count() as f32 + partial,
        can_move: own.is_none_or(|own| own.can_move()),
        attack: own.map(|own| own.attack).unwrap_or_default(),
        invulnerable: own.map(|own| own.invulnerable).unwrap_or_default(),
        death: own.map(|own| own.death).unwrap_or_default(),
    }
}

/// A torn-down creaking does not fall over or flash red.
fn creaking_tear_down(entity: &ClientEntity, placed: &mut MobPlacement) {
    if entity
        .creaking()
        .is_some_and(CreakingState::is_tearing_down)
    {
        placed.death_time = 0.0;
        placed.red_overlay = false;
    }
}

/// Whether a creaking's eyes are lit: the blink of a torn-down one
/// (`hasGlowingEyes`), else while it is active.
fn creaking_eyes_glow(entity: &ClientEntity) -> bool {
    entity.creaking().is_some_and(|own| {
        if own.is_tearing_down() {
            own.has_glowing_eyes()
        } else {
            own.is_active()
        }
    })
}

/// The layers some monsters draw over their body: a breeze's wind and eyes, a
/// warden's glowing parts.
fn glowing_layers(
    models: &mut MobModels,
    textures: &EntityTextures,
    (pose, entity, look, partial): (Pose, &ClientEntity, QuadrupedState, f32),
    (placed, glow, swirl): (&MobPlacement, &mut Vec<Vertex>, &mut Vec<Vertex>),
) -> Vec<Vertex> {
    match pose {
        Pose::Wither => {
            // `WitherArmorLayer`: the armor's swirl while it is hurt, posed as the body is.
            if wither_is_powered(entity)
                && let (Some(rect), Some(armor)) =
                    (textures.rect(WITHER_ARMOR.1), models.model(WITHER_ARMOR.0))
            {
                let age = entity.tick_count() as f32 + partial;
                let own = entity.wither().copied().unwrap_or_default();
                armor.setup_wither(&WitherRenderState {
                    age_in_ticks: age,
                    y_rot: look.head_y_rot,
                    x_rot: look.x_rot,
                    body_rot: placed.body_rot,
                    x_head_rots: own.x_rot_heads(),
                    y_head_rots: own.y_rot_heads(),
                });
                // `WitherArmorLayer.xOffset`.
                let x = voxel_core::math::mth_cos_f64(f64::from(age * 0.02)) * 3.0 % 1.0;
                swirl.extend(mob_vertices_swirl(
                    armor,
                    placed,
                    rect,
                    [x, age * 0.01 % 1.0],
                    POWER_COLOUR as u32,
                ));
            }
            Vec::new()
        }
        Pose::Creaking => {
            // The eyes: the same posed model, drawn unlit when they glow.
            let state = creaking_state(entity, look, partial);
            if let (true, Some(rect), Some(model)) = (
                creaking_eyes_glow(entity),
                textures.rect(CREAKING_EYES.1),
                models.model(CREAKING_EYES.0),
            ) {
                model.setup_creaking(&state);
                glow.extend(mob_vertices_glow(model, placed, rect));
            }
            Vec::new()
        }
        Pose::Breeze => breeze_layers(models, textures, (entity, partial), placed),
        Pose::Warden => {
            let state = warden_state(entity, look, partial);
            warden_layers(models, textures, (entity, &state, partial), placed)
        }
        _ => Vec::new(),
    }
}

/// A breeze's wind and eyes over its body: the wind scrolled along its texture
/// and lit by the world, the eyes lit by faces only; both posed as the body is.
fn breeze_layers(
    models: &mut MobModels,
    textures: &EntityTextures,
    (entity, partial): (&ClientEntity, f32),
    placed: &MobPlacement,
) -> Vec<Vertex> {
    let state = breeze_state(entity, partial);
    let mut vertices = Vec::new();
    // `BreezeWindLayer.xOffset`: a fiftieth of the age, within one texture.
    let scroll = (state.age_in_ticks * 0.02) % 1.0;
    for (layer, texture) in [BREEZE_WIND, BREEZE_EYES] {
        let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
            continue;
        };
        model.setup_breeze(&state);
        vertices.extend(if layer == BREEZE_WIND.0 {
            mob_vertices_scrolled(model, placed, rect, scroll)
        } else {
            mob_vertices_emissive(model, placed, rect)
        });
    }
    vertices
}

/// `ParrotModel.setupAnim`, posed by what the parrot is doing.
fn pose_parrot(model: &mut MobModel, entity: &ClientEntity, look: QuadrupedState, partial: f32) {
    let own = entity.parrot().copied();
    // `ParrotModel.getPose`: sitting, else flying in the air (the party
    // pose needs a jukebox the client does not track yet).
    let pose = if entity.is_sitting() {
        ParrotPose::Sitting
    } else if !entity.on_ground() {
        ParrotPose::Flying
    } else {
        ParrotPose::Standing
    };
    model.setup_parrot(&ParrotRenderState {
        quadruped: look,
        age_in_ticks: entity.tick_count() as f32 + partial,
        flap_angle: own.map_or(0.0, |own| own.flap_angle(partial)),
        pose,
    });
}

/// `WitherBossModel.setupAnim`, and what `WitherBossRenderer` adds to the pose: twice the
/// size (less while it is invulnerable), and a block light of fifteen.
fn pose_wither(
    model: &mut MobModel,
    entity: &ClientEntity,
    (look, snapshot, partial): (QuadrupedState, &EntitySnapshot, f32),
    placed: &mut MobPlacement,
) {
    let invulnerable = wither_invulnerable_ticks(entity, partial);
    let own = entity.wither().copied().unwrap_or_default();
    model.setup_wither(&WitherRenderState {
        age_in_ticks: entity.tick_count() as f32 + partial,
        y_rot: look.head_y_rot,
        x_rot: look.x_rot,
        body_rot: snapshot.body_yaw,
        x_head_rots: own.x_rot_heads(),
        y_head_rots: own.y_rot_heads(),
    });
    // `WitherBossRenderer.scale` and `getBlockLightLevel`.
    let scale = 2.0
        - if invulnerable > 0.0 {
            invulnerable / 220.0 * 0.5
        } else {
            0.0
        };
    placed.hook_scale = Some([scale; 3]);
    placed.light = (placed.light & !0xFFFF) | (15 << 4);
}

/// The model's `setupAnim` of the monster, and what its renderer adds to the
/// pose (a swelling creeper's scale).
fn pose_monster(
    model: &mut MobModel,
    monster: &Monster,
    entity: &ClientEntity,
    (look, snapshot, partial): (QuadrupedState, &EntitySnapshot, f32),
    placed: &mut MobPlacement,
) {
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
        Pose::Blaze => {
            model.setup_blaze(
                entity.tick_count() as f32 + partial,
                look.head_y_rot,
                look.x_rot,
            );
            // `BlazeRenderer.getBlockLightLevel`: fifteen, whatever the blocks say.
            placed.light = (placed.light & !0xFFFF) | (15 << 4);
        }
        Pose::Ghast => model.setup_ghast(entity.tick_count() as f32 + partial),
        Pose::Bat => {
            let own = entity.bat().copied().unwrap_or_default();
            model.setup_bat(&BatRenderState {
                head_y_rot: look.head_y_rot,
                age_in_ticks: entity.tick_count() as f32 + partial,
                is_resting: entity.is_bat_resting(),
                fly: own.fly,
                rest: own.rest,
            });
        }
        Pose::Vex => {
            // `ArmedEntityRenderState`: the right hand is the main one for a
            // right-handed mob.
            let (main, off) = (
                entity.equipment(EquipmentSlot::MainHand).is_some(),
                entity.equipment(EquipmentSlot::OffHand).is_some(),
            );
            let left_handed = entity.main_arm_is_left();
            model.setup_vex(&VexRenderState {
                head_y_rot: look.head_y_rot,
                x_rot: look.x_rot,
                age_in_ticks: entity.tick_count() as f32 + partial,
                is_charging: entity.is_vex_charging(),
                right_hand_item: if left_handed { off } else { main },
                left_hand_item: if left_handed { main } else { off },
            });
            // `VexRenderer.getBlockLightLevel`.
            placed.light = (placed.light & !0xFFFF) | (15 << 4);
        }
        Pose::Breeze => model.setup_breeze(&breeze_state(entity, partial)),
        Pose::Warden => model.setup_warden(&warden_state(entity, look, partial)),
        Pose::Wither => pose_wither(model, entity, (look, snapshot, partial), placed),
        Pose::Creaking => {
            model.setup_creaking(&creaking_state(entity, look, partial));
            creaking_tear_down(entity, placed);
        }
        Pose::Parrot => pose_parrot(model, entity, look, partial),
        Pose::Allay => {
            let own = entity.allay().copied().unwrap_or_default();
            model.setup_allay(&AllayRenderState {
                quadruped: look,
                age_in_ticks: entity.tick_count() as f32 + partial,
                is_dancing: entity.is_allay_dancing(),
                is_spinning: own.is_spinning(),
                spinning_progress: own.spinning_progress(partial),
                holding_animation_progress: own.holding_item_progress(partial),
            });
            // `AllayRenderer.getBlockLightLevel`.
            placed.light = (placed.light & !0xFFFF) | (15 << 4);
        }
        Pose::Phantom => {
            let age_in_ticks = entity.tick_count() as f32 + partial;
            model.setup_phantom((entity.id() * 3) as f32 + age_in_ticks);
            // `PhantomRenderer.scale` and `setupRotations`: it grows with its size,
            // is lifted, and tips with its pitch.
            let scale = 1.0 + 0.15 * entity.phantom_size() as f32;
            placed.hook_scale = Some([scale; 3]);
            placed.hook_translate = Some([0.0, 1.3125, 0.1875]);
            placed.pitch = Some(snapshot.pitch);
        }
        Pose::Creeper => {
            let swelling = entity.creeper_swelling(partial);
            model.setup_creeper(&look);
            placed.hook_scale = Some(creeper_scale(swelling));
            placed.white_overlay = creeper_white_overlay(swelling);
        }
    }
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
    let mut swirl = Vec::new();
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
        let texture = texture_of(monster, entity, partial);
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
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
        pose_monster(
            model,
            monster,
            entity,
            (look, &snapshot, partial),
            &mut placed,
        );
        vertices.extend(mob_vertices(model, &placed, rect));
        if matches!(monster.pose, Pose::Vex | Pose::Allay) {
            flier_hand_items(model.root(), (monster, entity), &placed, held);
        }
        // The eyes, over the model as it is posed, after the other layers.
        if let Some(eyes) = voxel_render::render_types::eyes_texture(monster.type_name)
            .and_then(|eyes| textures.rect(eyes))
        {
            glow.extend(mob_vertices_glow(model, &placed, eyes));
        }
        vertices.extend(glowing_layers(
            models,
            textures,
            (monster.pose, entity, look, partial),
            (&placed, &mut glow, &mut swirl),
        ));
        // A charged creeper's swirl: the armor model, posed as the creeper is.
        if monster.pose == Pose::Creeper
            && entity.is_powered()
            && let (Some(rect), Some(armor)) = (
                textures.rect(POWER_TEXTURE),
                models.model("minecraft:creeper#armor"),
            )
        {
            armor.setup_creeper(&look);
            swirl.extend(mob_vertices_swirl(
                armor,
                &placed,
                rect,
                swirl_offset(entity.tick_count() as f32 + partial),
                POWER_COLOUR as u32,
            ));
        }
        if let Some(state) = entity
            .carried_block_state()
            .filter(|_| monster.pose == Pose::Enderman)
        {
            held.push(HeldItem::block(
                state,
                carried_block_pose(&placement_pose(&placed)),
                placed.light,
                0.0,
            ));
        }
    }
    models.glow.extend(glow);
    models.swirl.extend(swirl);
    vertices
}
