//! The illagers the frame draws: the vindicator, pillager, evoker and
//! illusioner (`IllagerRenderer` and its subclasses).

use photon_core::ids::EntityTypeId;
use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{
    IllagerArmPose, IllagerState, QuadrupedState, mob_vertices,
};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::{EntityView, EquipmentSlot};

use crate::equipment::held_items::{HEAD_TRANSFORMS, Hands, HeldItem};
use crate::mobs::{MobModels, is_item, placement};
use crate::overlays::FrameView;

/// Which illager, which decides its arm pose and when its hands show what
/// they hold.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Vindicator,
    Pillager,
    Evoker,
    Illusioner,
}

struct Illager {
    type_name: &'static str,
    layer: &'static str,
    texture: &'static str,
    kind: Kind,
}

const ILLAGERS: [Illager; 4] = [
    Illager {
        type_name: "minecraft:vindicator",
        layer: "minecraft:vindicator#main",
        texture: "entity/illager/vindicator",
        kind: Kind::Vindicator,
    },
    Illager {
        type_name: "minecraft:pillager",
        layer: "minecraft:pillager#main",
        texture: "entity/illager/pillager",
        kind: Kind::Pillager,
    },
    Illager {
        type_name: "minecraft:evoker",
        layer: "minecraft:evoker#main",
        texture: "entity/illager/evoker",
        kind: Kind::Evoker,
    },
    Illager {
        type_name: "minecraft:illusioner",
        layer: "minecraft:illusioner#main",
        texture: "entity/illager/illusioner",
        kind: Kind::Illusioner,
    },
];

/// The paths (under `textures/`) of the textures the illagers wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    ILLAGERS.iter().map(|illager| illager.texture)
}

/// `getArmPose` of each illager. A pillager charging its crossbow is shown
/// holding it: the charge needs how long the item has been in use, which is
/// not tracked for mobs yet.
fn arm_pose(kind: Kind, entity: EntityView<'_>) -> IllagerArmPose {
    let celebrating = || {
        if entity.is_celebrating() {
            IllagerArmPose::Celebrating
        } else {
            IllagerArmPose::Crossed
        }
    };
    match kind {
        Kind::Vindicator if entity.is_aggressive() => IllagerArmPose::Attacking,
        Kind::Vindicator => celebrating(),
        Kind::Pillager => {
            let crossbow = [EquipmentSlot::MainHand, EquipmentSlot::OffHand]
                .into_iter()
                .any(|slot| {
                    entity
                        .equipment(slot)
                        .is_some_and(|stack| is_item(stack, "minecraft:crossbow"))
                });
            if crossbow {
                IllagerArmPose::CrossbowHold
            } else if entity.is_aggressive() {
                IllagerArmPose::Attacking
            } else {
                IllagerArmPose::Neutral
            }
        }
        Kind::Evoker if entity.is_casting_spell() => IllagerArmPose::Spellcasting,
        Kind::Evoker => celebrating(),
        Kind::Illusioner if entity.is_casting_spell() => IllagerArmPose::Spellcasting,
        Kind::Illusioner if entity.is_aggressive() => IllagerArmPose::BowAndArrow,
        Kind::Illusioner => IllagerArmPose::Crossed,
    }
}

/// When the layer that draws the hands' items shows them.
fn shows_hand_items(kind: Kind, entity: EntityView<'_>) -> bool {
    match kind {
        Kind::Pillager => true,
        Kind::Vindicator => entity.is_aggressive(),
        Kind::Evoker => entity.is_casting_spell(),
        Kind::Illusioner => entity.is_casting_spell() || entity.is_aggressive(),
    }
}

/// `IllagerRenderer` for the illagers the server added, with what they hold
/// and wear on the head. The illusioner's mirror images while it cannot be
/// seen and the pillager's crossbow charge are not drawn yet.
pub fn remote_illagers(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let ids: Vec<(EntityTypeId, &Illager)> = ILLAGERS
        .iter()
        .filter_map(|illager| Some((photon_data::entity_type::id(illager.type_name)?, illager)))
        .collect();
    for entity in world.drawn_mobs() {
        let Some(illager) = ids
            .iter()
            .find(|(id, _)| *id == entity.type_id)
            .map(|(_, illager)| *illager)
        else {
            continue;
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(illager.texture),
            models.model(illager.layer),
        ) else {
            continue;
        };
        model.setup_illager(&illager_state(illager, entity, &snapshot, partial));
        let placed = placement(world, &snapshot, view, entity.scale());
        let hands = shows_hand_items(illager.kind, entity);
        Hands {
            base: &photon_render::entity::mob_model::placement_pose(&placed),
            root: model.root(),
            hand_shift: 0.0,
            baby: false,
            light: placed.light,
            right: entity.item_held_by_arm(false).filter(|_| hands),
            left: entity.item_held_by_arm(true).filter(|_| hands),
            head: entity.equipment(EquipmentSlot::Head),
            head_transforms: HEAD_TRANSFORMS,
            eye: None,
            motions: Default::default(),
        }
        .collect_outlined(held, HeldItem::glow_holder(entity));
        // `IllusionerRenderer.isBodyVisible` is always true.
        if illager.kind == Kind::Illusioner {
            vertices.extend(mob_vertices(model, &placed, rect));
        } else {
            vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
        }
    }
}

/// What the illager's model is posed from.
fn illager_state(
    illager: &Illager,
    entity: EntityView<'_>,
    snapshot: &photon_world::EntitySnapshot,
    partial: f32,
) -> IllagerState {
    let main_arm_left = entity.main_arm_is_left();
    IllagerState {
        quadruped: QuadrupedState {
            head_y_rot: photon_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        },
        age_in_ticks: entity.tick_count() as f32 + partial,
        riding: entity.vehicle_id().is_some(),
        arm_pose: arm_pose(illager.kind, entity),
        main_arm_left,
        attack_time: snapshot.attack_time,
        has_item: entity.item_held_by_arm(main_arm_left).is_some(),
        max_crossbow_charge: 0,
        ticks_using_item: 0.0,
        hat_visible: illager.kind == Kind::Illusioner,
    }
}
