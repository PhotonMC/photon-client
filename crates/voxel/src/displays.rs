//! The block and item displays (`DisplayRenderer`): the block or item, oriented as its billboard
//! constraint says and moved by its transformation (eased while it changes), lit by the world
//! unless the server overrode the light. Text displays, shadows and the glow outline are not here.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::biome_tint::BiomeTints;
use voxel_render::display::{DisplayTurns, display_pose, item_display_pose};
use voxel_render::item_entity::{entity_block_vertices, item_layer_vertices};
use voxel_render::joml::Pose;
use voxel_render::model::DisplayContext;
use voxel_render::scene::Vertex;
use voxel_render::world_mesh::BlockRenderer;
use voxel_world::display::NO_BRIGHTNESS_OVERRIDE;

use crate::item_layers::ItemLayerCache;
use crate::overlays::{FrameView, packed_light};

/// The layers of the items the displays show, by the `ItemDisplayContext` each names (by its id).
pub struct DisplayLayers {
    contexts: Vec<ItemLayerCache>,
}

impl DisplayLayers {
    pub fn new() -> Self {
        let contexts = [
            DisplayContext::None,
            DisplayContext::ThirdPersonLeftHand,
            DisplayContext::ThirdPersonRightHand,
            DisplayContext::FirstPersonLeftHand,
            DisplayContext::FirstPersonRightHand,
            DisplayContext::Head,
            DisplayContext::Gui,
            DisplayContext::Ground,
            DisplayContext::Fixed,
            DisplayContext::OnShelf,
        ];
        Self {
            contexts: contexts.map(ItemLayerCache::new).into(),
        }
    }
}

/// `ItemDisplayContext.BY_ID`'s order, with an id out of range the first.
fn context_index(id: u8) -> usize {
    usize::from(id).min(9) * usize::from(id <= 9)
}

/// `DisplayRenderer.getBlockLightLevel` and `getSkyLightLevel`: the light the server set, a level
/// at a time, else the world's.
fn displayed_light(world_light: i32, brightness_override: i32) -> i32 {
    if brightness_override == NO_BRIGHTNESS_OVERRIDE {
        return world_light;
    }
    (world_light & !(15 << 4) & !(15 << 20))
        | (brightness_override & (15 << 4))
        | (brightness_override & (15 << 20))
}

/// The triangles of every block and item display the server added.
pub fn displays(
    session: &PlaySession,
    layers: &mut DisplayLayers,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let Some(display) = entity.display() else {
            continue;
        };
        let (Some(state), Some(snapshot)) = (
            display.render_state(),
            world.entity_snapshot(entity.id(), partial),
        ) else {
            continue;
        };
        let (x, y, z) = snapshot.position;
        // `Display.shouldRenderAtSqrDistance`.
        let camera = (
            f64::from(view.origin[0]) + f64::from(view.camera[0]),
            f64::from(view.origin[1]) + f64::from(view.camera[1]),
            f64::from(view.origin[2]) + f64::from(view.camera[2]),
        );
        let distance_sqr = (x - camera.0).powi(2) + (y - camera.1).powi(2) + (z - camera.2).powi(2);
        let range = f64::from(display.view_range()) * 64.0;
        if distance_sqr >= range * range {
            continue;
        }
        let progress = display.interpolation_progress(entity.tick_count(), partial);
        let transformation = state.transformation.get(progress);
        let turns = DisplayTurns {
            entity: (entity.y_rot_at(partial), entity.x_rot_at(partial)),
            camera: (view.rotation.0, view.rotation.1),
        };
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        let pose = display_pose(&base, (state.billboard, &transformation), &turns);
        // `Entity.getLightProbePosition`: the eyes, which a display has at its feet.
        let light = displayed_light(
            packed_light(world, Vec3::new(x, y, z)),
            state.brightness_override,
        );
        if let Some(block) = display.block_state() {
            let cache = &mut layers.contexts[0];
            if let Some((layer, block_light)) = cache.block(block, renderer) {
                vertices.extend(entity_block_vertices(
                    layer,
                    &pose,
                    (
                        voxel_render::entity_block::submit_light(light, *block_light),
                        0.0,
                    ),
                ));
            }
        } else if let Some(context) = display.item_context()
            && let Some(stack) = entity.item_stack().filter(|stack| stack.item_count() > 0)
        {
            let cache = &mut layers.contexts[context_index(context)];
            vertices.extend(item_layer_vertices(
                cache.get(stack, renderer, tints),
                &[item_display_pose(&pose)],
                light,
                false,
            ));
        }
    }
    vertices
}
