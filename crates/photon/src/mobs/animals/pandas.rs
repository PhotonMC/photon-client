//! The panda (`PandaRenderer`, `PandaHoldsItemLayer`): its texture by gene,
//! its pose, how it somersaults, sits and lies on its back, and what it holds
//! while it sits.
//!
//! Whether a panda is in water is not tracked (a worried panda in a thunder
//! storm would not sit down in the water), and its eating and sneezing make
//! no particles or sounds.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{
    PandaRotation, PandaState, QuadrupedState, panda_item_pose, placement_pose,
};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::EquipmentSlot;

use crate::equipment::held_items::{HeldItem, Place};
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The paths (under `textures/`) of the textures a panda wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    (0..7).flat_map(|gene| {
        [false, true]
            .into_iter()
            .map(move |baby| photon_data::entity_type::panda_texture(gene, baby))
    })
}

/// `PandaRenderer` for the pandas the server added, with what they hold.
pub fn remote_pandas(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    let Some(panda) = photon_data::entity_type::id("minecraft:panda") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.drawn_mobs().filter(|entity| entity.type_id == panda) {
        let baby = entity.is_baby();
        let own = entity.panda();
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let (Some(rect), Some(model)) = (
            textures.rect(photon_data::entity_type::panda_texture(own.variant(), baby)),
            models.model(if baby {
                "minecraft:panda_baby#main"
            } else {
                "minecraft:panda#main"
            }),
        ) else {
            continue;
        };
        let age_in_ticks = entity.tick_count() as f32 + partial;
        let scared = own.is_scared(world.is_thundering());
        let (sit_amount, lie_on_back_amount) =
            (own.sit_amount(partial), own.lie_on_back_amount(partial));
        model.setup_panda(&PandaState {
            quadruped: QuadrupedState {
                head_y_rot: photon_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks,
            unhappy: own.is_unhappy(),
            sneezing: own.is_sneezing(),
            sneeze_time: own.sneeze_counter(),
            eating: own.is_eating(),
            scared,
            sit_amount,
            lie_on_back_amount,
            // A baby never shows the roll in its pose.
            roll_amount: if baby { 0.0 } else { own.roll_amount(partial) },
            baby,
        });
        let mut placed = placement(world, &snapshot, view, entity.scale());
        placed.panda = Some(PandaRotation {
            roll_time: if own.roll_counter() > 0 {
                own.roll_counter() as f32 + partial
            } else {
                0.0
            },
            sit_amount,
            lie_on_back_amount,
            x_rot: snapshot.pitch,
            age_in_ticks,
            scared,
            baby,
        });
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
        if let Some(stack) = entity.equipment(EquipmentSlot::MainHand)
            && own.is_sitting()
            && !scared
        {
            let pose = panda_item_pose(&placement_pose(&placed), age_in_ticks, own.is_eating());
            held.push(HeldItem::stack(stack, pose, Place::Ground, placed.light));
        }
    }
}
