//! The fox (`FoxRenderer`, `FoxHeldItemLayer`): its texture by variant and
//! sleep, its pose, how it tips when it pounces or has its face planted, and
//! what it holds in its mouth.

use voxel_net::connection::PlaySession;
use voxel_render::entity::held_item::fox_item_pose;
use voxel_render::entity::mob_model::{FoxState, QuadrupedState, mob_vertices, placement_pose};
use voxel_render::entity::textures::EntityTextures;
use voxel_render::scene::Vertex;
use voxel_world::EquipmentSlot;

use crate::equipment::held_items::{HeldItem, Place};
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The paths (under `textures/`) of the textures a fox wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [0, 1].into_iter().flat_map(|variant| {
        [false, true].into_iter().flat_map(move |sleeping| {
            [false, true]
                .into_iter()
                .map(move |baby| voxel_data::entity_type::fox_texture(variant, sleeping, baby))
        })
    })
}

/// `FoxRenderer` for the foxes the server added, with what they hold.
pub fn remote_foxes(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let Some(fox) = voxel_data::entity_type::id("minecraft:fox") else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities().filter(|entity| entity.type_id == fox) {
        let baby = entity.is_baby();
        let own = entity.fox();
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let texture = voxel_data::entity_type::fox_texture(own.variant(), own.is_sleeping(), baby);
        let (Some(rect), Some(model)) = (
            textures.rect(texture),
            models.model(if baby {
                "minecraft:fox_baby#main"
            } else {
                "minecraft:fox#main"
            }),
        ) else {
            continue;
        };
        let quadruped = QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        };
        let head_roll_angle = own.head_roll_angle(partial);
        model.setup_fox(&FoxState {
            quadruped,
            head_roll_angle,
            crouch_amount: own.crouch_amount(partial),
            crouching: own.is_crouching(),
            sleeping: own.is_sleeping(),
            sitting: own.is_sitting(),
            faceplanted: own.is_faceplanted(),
            pouncing: own.is_pouncing(),
            age_in_ticks: entity.tick_count() as f32 + partial,
            age_scale: if baby { 0.5 } else { 1.0 },
            baby,
        });
        let mut placed = placement(world, &snapshot, view, entity.scale());
        // `FoxRenderer.setupRotations`: a fox that pounces or has its face
        // planted tips with its pitch.
        if own.is_pouncing() || own.is_faceplanted() {
            placed.fox_tipping(snapshot.pitch);
        }
        vertices.extend(mob_vertices(model, &placed, rect));
        let head = model
            .root()
            .children()
            .iter()
            .find(|part| part.name() == "head");
        if let (Some(stack), Some(head)) = (entity.equipment(EquipmentSlot::MainHand), head) {
            let pose = fox_item_pose(
                &placement_pose(&placed),
                head,
                (baby, own.is_sleeping()),
                (head_roll_angle, quadruped.head_y_rot, quadruped.x_rot),
            );
            held.push(HeldItem::stack(stack, pose, Place::Ground, placed.light));
        }
    }
    vertices
}
