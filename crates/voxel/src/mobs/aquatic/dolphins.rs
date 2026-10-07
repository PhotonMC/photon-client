//! The dolphin (`DolphinRenderer`, `DolphinCarryingItemLayer`): its texture,
//! grown or baby, the undulation of its body while it moves, and what it
//! carries under its chin.

use voxel_net::connection::PlaySession;
use voxel_render::entity::mob_model::{
    DolphinState, dolphin_item_pose, mob_vertices, placement_pose,
};
use voxel_render::entity::textures::EntityTextures;
use voxel_render::scene::Vertex;
use voxel_world::EquipmentSlot;

use crate::equipment::held_items::{HeldItem, Place};
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The dolphin's textures and model layers: grown, then baby.
const TEXTURES: [&str; 2] = ["entity/dolphin/dolphin", "entity/dolphin/dolphin_baby"];
const LAYERS: [&str; 2] = ["minecraft:dolphin#main", "minecraft:dolphin_baby#main"];

/// The paths (under `textures/`) of the textures the dolphins wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES.into_iter()
}

/// `DolphinRenderer` for the dolphins the server added.
pub fn remote_dolphins(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    let Some(dolphin) = voxel_data::entity_type::id("minecraft:dolphin") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world
        .entities()
        .drawn()
        .filter(|entity| entity.type_id == dolphin)
    {
        let index = usize::from(entity.is_baby());
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(TEXTURES[index]),
            models.model(LAYERS[index]),
        ) else {
            continue;
        };
        model.setup_dolphin(&DolphinState {
            y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            age_in_ticks: entity.tick_count() as f32 + partial,
            is_moving: entity.is_moving_horizontally(),
        });
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
        if let Some(stack) = entity.equipment(EquipmentSlot::MainHand) {
            let pose = dolphin_item_pose(&placement_pose(&placed), snapshot.pitch);
            held.push(HeldItem::stack(stack, pose, Place::Ground, placed.light));
        }
    }
}
