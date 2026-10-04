//! The monsters, and the other mobs that are posed by one model class of their
//! own, the frame draws: spiders, the creeper, the enderman, silverfish, the
//! blaze, ghast, phantom, bat, vex and allay (`SpiderRenderer`,
//! `CreeperRenderer`, `EndermanRenderer` and the like).

use voxel_net::connection::PlaySession;
use voxel_render::entity_block::carried_block_pose;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::held_item::{allay_hand_pose, vex_hand_pose};
use voxel_render::mob_model::{
    AllayRenderState, BatRenderState, BreezeRenderState, EndermanState, MobModel, MobPlacement,
    ParrotPose, ParrotRenderState, QuadrupedState, VexRenderState, creeper_scale,
    creeper_white_overlay, mob_vertices, mob_vertices_emissive, mob_vertices_glow,
    mob_vertices_scrolled, mob_vertices_swirl, placement_pose,
};
use voxel_render::player_model::{Arm, Part, PlayerRenderState};
use voxel_render::render_types::{POWER_COLOUR, POWER_TEXTURE, swirl_offset};
use voxel_render::scene::Vertex;
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

const MONSTERS: [Monster; 14] = [
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
];

/// The wind and the eyes of a breeze (`BreezeWindLayer`, `BreezeEyesLayer`): their
/// model layers and textures.
const BREEZE_WIND: (&str, &str) = ("minecraft:breeze#wind", "entity/breeze/breeze_wind");
const BREEZE_EYES: (&str, &str) = ("minecraft:breeze#eyes", "entity/breeze/breeze_eyes");

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
}

/// The texture a monster wears now: a ghast's mouth open to charge, a vex charging,
/// a parrot's variant, else its own.
fn texture_of(monster: &Monster, entity: &ClientEntity) -> &'static str {
    match monster.pose {
        Pose::Ghast if entity.is_ghast_charging() => GHAST_SHOOTING_TEXTURE,
        Pose::Vex if entity.is_vex_charging() => VEX_CHARGING_TEXTURE,
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
        Pose::Parrot => {
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
        let texture = texture_of(monster, entity);
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
        if monster.pose == Pose::Breeze {
            vertices.extend(breeze_layers(models, textures, (entity, partial), &placed));
        }
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
