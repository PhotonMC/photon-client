//! The items thrown or shot through the air (`ThrownItemRenderer`, `FireworkEntityRenderer`):
//! snowballs, eggs, pearls, potions, bottles, eyes of ender, fireballs and firework rockets,
//! each its item's model facing the camera.

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::name_tag::camera_orientation;
use photon_render::joml::Pose;
use photon_render::objects::item::item_entity_vertices;
use photon_render::objects::projectile::{firework_pose, thrown_item_pose};
use photon_render::scene::Vertex;

use crate::equipment::item_layers::ItemLayerCache;
use crate::equipment::needle::Owner;
use crate::overlays::{FrameView, full_block_light, packed_light};
use photon_render::item_model::Holder;

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
    let rocket = photon_data::entity_type::id("minecraft:firework_rocket");
    for entity in world.entities().drawn_of_kind(|entity| {
        entity.thrown_item_look().is_some() || Some(entity.type_id) == rocket
    }) {
        let look = entity.thrown_item_look();
        if look.is_none() && Some(entity.type_id) != rocket {
            continue;
        }
        let (Some(stack), Some(snapshot)) = (
            entity.item_stack(),
            world.entity_snapshot_of(entity, partial),
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
        // `updateForNonLiving`: the thrown entity holds its stack, at its own heading.
        let (px, py, pz) = entity.position();
        layers.set_owner(Some(Owner {
            holder: Holder {
                position: [px, py, pz],
                yaw: entity.rotation().0,
            },
            seed: entity.id().0,
        }));
        let item_layers = layers.get(stack, renderer, tints);
        let drawn = item_entity_vertices(item_layers, &[pose], light);
        layers.extend_with_glint(stack, drawn, vertices);
    }
    layers.set_owner(None);
}
