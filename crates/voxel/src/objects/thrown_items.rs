//! The items thrown or shot through the air (`ThrownItemRenderer`, `FireworkEntityRenderer`):
//! snowballs, eggs, pearls, potions, bottles, eyes of ender, fireballs and firework rockets,
//! each its item's model facing the camera.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::block::biome_tint::BiomeTints;
use voxel_render::block::world_mesh::BlockRenderer;
use voxel_render::entity::name_tag::camera_orientation;
use voxel_render::joml::Pose;
use voxel_render::objects::item::item_entity_vertices;
use voxel_render::objects::projectile::{firework_pose, thrown_item_pose};
use voxel_render::scene::Vertex;

use crate::equipment::item_layers::ItemLayerCache;
use crate::overlays::{FrameView, full_block_light, packed_light};

/// The items of the entities that draw as an item facing the camera, as the ground shows them
/// (`ItemDisplayContext.GROUND`), each lit by where it is (fully, for those the renderer lights so).
pub fn thrown_items(
    session: &PlaySession,
    layers: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let orientation = camera_orientation(view.rotation.0, view.rotation.1);
    let rocket = voxel_data::entity_type::id("minecraft:firework_rocket");
    for entity in world.entities().drawn() {
        let look = entity.thrown_item_look();
        if look.is_none() && Some(entity.type_id) != rocket {
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
        let (x, y, z) = snapshot.position;
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        let pose = match look {
            Some((scale, _)) => thrown_item_pose(&base, scale, &orientation),
            None => firework_pose(&base, &orientation, entity.is_shot_at_angle()),
        };
        // `Entity.getLightProbePosition`: the eyes; a fireball's block light is fifteen.
        let mut light = packed_light(world, Vec3::new(x, y + f64::from(entity.eye_height()), z));
        if look.is_some_and(|(_, full)| full) {
            light = full_block_light(light);
        }
        let item_layers = layers.get(stack, renderer, tints);
        vertices.extend(item_entity_vertices(item_layers, &[pose], light));
    }
}
