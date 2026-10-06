//! The items lying in the world (`ItemEntityRenderer`): each stack's copies,
//! bobbing and spinning where the server says the item is.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::biome_tint::BiomeTints;
use voxel_render::item_entity::{
    item_entity_poses, item_entity_vertices, model_extent, rendered_amount, seed_for_stack,
};
use voxel_render::joml::Pose;
use voxel_render::scene::Vertex;
use voxel_render::world_mesh::BlockRenderer;

use crate::item_layers::ItemLayerCache;
use crate::overlays::{FrameView, packed_light};

/// The triangles of every dropped item in the world. Which damage a stack
/// has is not read from its components yet, so every stack seeds its
/// copies' offsets as an undamaged one.
pub fn dropped_items(
    session: &PlaySession,
    layers: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
) -> Vec<Vertex> {
    let Some(item_type) = voxel_data::entity_type::id("minecraft:item") else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        if entity.type_id != item_type {
            continue;
        }
        let (Some(stack), Some(snapshot)) = (
            entity.item_stack(),
            world.entity_snapshot(entity.id(), partial),
        ) else {
            continue;
        };
        if stack.item_count() <= 0 {
            continue;
        }
        let item_layers = layers.get(stack, renderer, tints);
        let (x, y, z) = snapshot.position;
        let mut start = Pose::default();
        start.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        let poses = item_entity_poses(
            start,
            entity.tick_count() as f32 + partial,
            entity.bob_offset(),
            rendered_amount(stack.item_count()),
            seed_for_stack(stack.item_registry_id(), 0),
            model_extent(item_layers),
        );
        let light = packed_light(world, Vec3::new(x, y + 0.2125, z));
        vertices.extend(item_entity_vertices(item_layers, &poses, light));
    }
    vertices
}

/// The layers of the items in the world, by the display context they are drawn in.
pub struct WorldItemLayers {
    ground: ItemLayerCache,
    frames: ItemLayerCache,
    displays: crate::displays::DisplayLayers,
}

impl WorldItemLayers {
    pub fn new() -> Self {
        Self {
            ground: ItemLayerCache::new(voxel_render::model::DisplayContext::Ground),
            frames: ItemLayerCache::new(voxel_render::model::DisplayContext::Fixed),
            displays: crate::displays::DisplayLayers::new(),
        }
    }
}

/// The triangles of every item in the world as the ground shows it: those lying about and those
/// thrown or shot through the air, and those in item frames, and the displays'.
pub fn world_items(
    session: &PlaySession,
    layers: &mut WorldItemLayers,
    tints: (&BlockRenderer, &BiomeTints),
    view: FrameView,
) -> Vec<Vertex> {
    let mut items = dropped_items(session, &mut layers.ground, tints, view);
    items.extend(crate::thrown_items::thrown_items(
        session,
        &mut layers.ground,
        tints,
        view,
    ));
    items.extend(crate::item_frames::item_frames(
        session,
        &mut layers.frames,
        tints,
        view,
    ));
    items.extend(crate::displays::displays(
        session,
        &mut layers.displays,
        tints,
        view,
    ));
    items
}
