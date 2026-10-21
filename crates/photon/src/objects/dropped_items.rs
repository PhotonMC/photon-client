//! The items lying in the world (`ItemEntityRenderer`): each stack's copies,
//! bobbing and spinning where the server says the item is.

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::objects::item::{
    item_entity_poses, item_entity_vertices, model_extent, rendered_amount, seed_for_stack,
};
use photon_render::scene::Vertex;
use photon_world::particles::{ItemPickup, PickedUp};

use crate::equipment::item_layers::{GlintFrame, ItemLayerCache};
use crate::equipment::needle::Owner;
use crate::objects::displays;
use crate::objects::item_frames;
use crate::objects::thrown_items;
use crate::overlays::{FrameView, packed_light};
use photon_render::item_model::Holder;

/// The triangles of every dropped item in the world. Which damage a stack
/// has is not read from its components yet, so every stack seeds its
/// copies' offsets as an undamaged one.
pub fn dropped_items(
    session: &PlaySession,
    pickups: &[&ItemPickup],
    layers: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(item_type) = photon_data::entity_type::id("minecraft:item") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.entities().drawn() {
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
        let item = (
            stack,
            entity.tick_count() as f32 + partial,
            entity.bob_offset(),
        );
        let light = packed_light(
            world,
            Vec3::new(
                snapshot.position.0,
                snapshot.position.1 + 0.2125,
                snapshot.position.2,
            ),
        );
        layers.set_owner(Some(ground_owner(entity)));
        draw_item(
            item,
            snapshot.position,
            light,
            (layers, renderer, tints),
            view,
            vertices,
        );
    }
    layers.set_owner(None);
    picked_up_items(session, pickups, layers, (renderer, tints), view, vertices);
}

/// `ItemPickupParticleGroup`: the items flying to whoever took them, as they were when taken,
/// lit where they were.
fn picked_up_items(
    session: &PlaySession,
    pickups: &[&ItemPickup],
    layers: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for pickup in pickups {
        if let PickedUp::Item {
            stack,
            age_in_ticks,
            bob_offset,
        } = &pickup.picked
        {
            let (x, y, z) = pickup.from;
            let light = packed_light(world, Vec3::new(x, y + 0.2125, z));
            let at = pickup.position_at(partial);
            let item = (stack, *age_in_ticks, *bob_offset);
            draw_item(item, at, light, (layers, renderer, tints), view, vertices);
        }
    }
}

/// A dropped item as the holder of its stack (`ItemEntity.getVisualRotationYInDegrees`): where
/// its entity is and the heading its spin has, without the frame's partial tick.
fn ground_owner(entity: &photon_world::ClientEntity) -> Owner {
    let spin = (entity.tick_count() as f32 + 0.5) / 20.0 + entity.bob_offset();
    let (x, y, z) = entity.position();
    Owner {
        holder: Holder {
            position: [x, y, z],
            yaw: 180.0 - spin / (std::f32::consts::PI * 2.0) * 360.0,
        },
        seed: entity.id().0,
    }
}

/// `ItemEntityRenderer.submit`: a stack's copies, bobbing and spinning at a place.
fn draw_item(
    (stack, age_in_ticks, bob_offset): (&photon_world::ClientItemStackState, f32, f32),
    (x, y, z): (f64, f64, f64),
    light: i32,
    (layers, renderer, tints): (&mut ItemLayerCache, &BlockRenderer, &BiomeTints),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let item_layers = layers.get(stack, renderer, tints);
    let mut start = Pose::default();
    start.translate(
        (x - f64::from(view.origin[0])) as f32,
        (y - f64::from(view.origin[1])) as f32,
        (z - f64::from(view.origin[2])) as f32,
    );
    let poses = item_entity_poses(
        start,
        age_in_ticks,
        bob_offset,
        rendered_amount(stack.item_count()),
        seed_for_stack(stack.item_registry_id(), 0),
        model_extent(item_layers),
    );
    let drawn = item_entity_vertices(item_layers, &poses, light);
    layers.extend_with_glint(stack, drawn, vertices);
}

/// The layers of the items in the world, by the display context they are drawn in.
pub struct WorldItemLayers {
    ground: ItemLayerCache,
    frames: item_frames::FrameState,
    on_shelf: ItemLayerCache,
    conduit: Option<Option<photon_render::block::world_mesh::ConduitMeshes>>,
    displays: displays::DisplayLayers,
}

impl WorldItemLayers {
    /// The triangles of the totem's animation, and its glint's: the stack in the `FIXED` context
    /// at a pose, with the glint moving as the frame's `glint` says.
    pub fn activation(
        &mut self,
        stack: &photon_world::ClientItemStackState,
        pose: photon_render::joml::Pose,
        materials: (&BlockRenderer, &BiomeTints),
        glint: &GlintFrame,
    ) -> (Vec<Vertex>, Vec<Vertex>) {
        self.frames.layers.set_glint(glint);
        self.frames.fixed_item(stack, pose, materials)
    }

    /// Set how the frame's item glints move and where their texture is.
    pub fn set_glint(&mut self, frame: &GlintFrame) {
        for cache in [
            &mut self.ground,
            &mut self.frames.layers,
            &mut self.on_shelf,
        ] {
            cache.set_glint(frame);
        }
        self.displays.set_glint(frame);
    }

    /// The glints of the enchanted items drawn this frame.
    pub fn take_glint(&mut self) -> Vec<Vertex> {
        let mut glint = Vec::new();
        for cache in [
            &mut self.ground,
            &mut self.frames.layers,
            &mut self.on_shelf,
        ] {
            glint.extend(cache.take_glint());
        }
        glint.extend(self.displays.take_glint());
        glint
    }

    pub fn new() -> Self {
        Self {
            ground: ItemLayerCache::new(photon_render::model::DisplayContext::Ground),
            frames: item_frames::FrameState::new(),
            on_shelf: ItemLayerCache::new(photon_render::model::DisplayContext::OnShelf),
            conduit: None,
            displays: displays::DisplayLayers::new(),
        }
    }
}

impl WorldItemLayers {
    /// Hand the items on the ground and what the frames' maps need to the renderer.
    pub fn hand_over(
        &mut self,
        backend: &mut photon_render::VulkanBackend,
        items: &[Vertex],
    ) -> Result<(), photon_render::VulkanError> {
        backend.set_draw(photon_render::vulkan::DrawKind::GroundItems, items)?;
        self.frames.hand_over(backend)
    }
}

/// The triangles of every item in the world as the ground shows it: those lying about and those
/// thrown or shot through the air, and those in item frames, and the displays'.
pub fn world_items(
    (session, pickups): (&PlaySession, &[&ItemPickup]),
    layers: &mut WorldItemLayers,
    (renderer, tints, textures): (&BlockRenderer, &BiomeTints, &EntityTextures),
    view: FrameView,
    items: &mut Vec<Vertex>,
) {
    dropped_items(
        session,
        pickups,
        &mut layers.ground,
        (renderer, tints),
        view,
        items,
    );
    thrown_items::thrown_items(session, &mut layers.ground, (renderer, tints), view, items);
    item_frames::item_frames(
        session,
        &mut layers.frames,
        (renderer, tints, textures),
        view,
        items,
    );
    block_entity_items(session, layers, (renderer, tints), view, items);
    super::flames::flames(session, renderer, view, items);
    displays::displays(
        session,
        &mut layers.displays,
        (renderer, tints),
        view,
        items,
    );
}

/// The items block entities show: what suspicious blocks, vaults, campfires and shelves hold, and
/// the conduits' shells.
fn block_entity_items(
    session: &PlaySession,
    layers: &mut WorldItemLayers,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
    items: &mut Vec<Vertex>,
) {
    super::block_entity_items::brushable_items(
        session,
        &mut layers.frames.layers,
        (renderer, tints),
        view,
        items,
    );
    super::block_entity_items::vault_items(
        session,
        &mut layers.ground,
        (renderer, tints),
        view,
        items,
    );
    super::block_entity_items::campfire_items(
        session,
        &mut layers.frames.layers,
        (renderer, tints),
        view,
        items,
    );
    super::block_entity_items::shelf_items(
        session,
        &mut layers.on_shelf,
        (renderer, tints),
        view,
        items,
    );
    if let Some(meshes) = layers
        .conduit
        .get_or_insert_with(|| renderer.conduit_meshes())
    {
        super::block_entity_items::conduits(session, meshes, view, items);
    }
}
