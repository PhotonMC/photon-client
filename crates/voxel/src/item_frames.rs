//! The item frames (`ItemFrameRenderer`): the frame's model on the wall (none for an invisible
//! one) and the item in it, turned by eighths. A frame that holds a map shows the map's item, as
//! one whose map data has not come: the map's own drawing is not here yet.

use voxel_core::direction::Direction;
use voxel_core::math::{Vec3, floor_f64};
use voxel_net::connection::PlaySession;
use voxel_render::biome_tint::BiomeTints;
use voxel_render::item_entity::{entity_block_vertices, item_layer_vertices};
use voxel_render::item_frame::{item_frame_poses, item_frame_position, render_offset};
use voxel_render::joml::Pose;
use voxel_render::scene::Vertex;
use voxel_render::world_mesh::BlockRenderer;

use crate::item_layers::ItemLayerCache;
use crate::overlays::{FrameView, packed_light};

/// `ItemFrameRenderer.getLightCoords`' light for the item of a glow frame
/// (`LightCoordsUtil.FULL_BRIGHT`): full block and sky light.
const GLOW_ITEM_LIGHT: i32 = 15_728_880;

/// The block light a glow frame is at least lit by (`ItemFrameRenderer.GLOW_FRAME_BRIGHTNESS`).
const GLOW_FRAME_BRIGHTNESS: i32 = 5;

/// The triangles of every item frame the server added, with the frame's model and the item in it.
pub fn item_frames(
    session: &PlaySession,
    layers: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let mut vertices = Vec::new();
    for entity in world.entities().filter(|entity| entity.is_item_frame()) {
        let glowing =
            voxel_data::entity_type::name(entity.type_id) == Some("minecraft:glow_item_frame");
        // `ClientboundAddEntityPacket.getData`: the way it faces; the block is its position.
        let direction = Direction::from_3d_data_value(entity.data);
        let (x, y, z) = entity.position();
        let (x, y, z) = item_frame_position((floor_f64(x), floor_f64(y), floor_f64(z)), direction);
        // `Entity.getLightProbePosition`: the eyes, which an item frame has at its feet; a glow
        // frame has at least a little block light (`getBlockLightLevel`).
        let mut light = packed_light(world, Vec3::new(x, y, z));
        if glowing {
            let block = ((light >> 4) & 15).max(GLOW_FRAME_BRIGHTNESS);
            light = (light & !(15 << 4)) | (block << 4);
        }
        let offset = render_offset(direction);
        let mut base = Pose::default();
        base.translate(
            ((x - f64::from(view.origin[0])) + offset.0) as f32,
            ((y - f64::from(view.origin[1])) + offset.1) as f32,
            ((z - f64::from(view.origin[2])) + offset.2) as f32,
        );
        let poses = item_frame_poses(
            &base,
            direction,
            entity.is_invisible(),
            entity.item_frame_rotation(),
        );
        if let Some(pose) = poses.frame {
            // `updateForItemFrame`: the map variant is for a frame whose map data is here (never yet).
            let state = renderer.item_frame_state(glowing, false);
            if let Some((layer, block_light)) = layers.block(state, renderer) {
                vertices.extend(entity_block_vertices(
                    layer,
                    &pose,
                    (
                        voxel_render::entity_block::submit_light(light, *block_light),
                        0.0,
                    ),
                ));
            }
        }
        if let Some(stack) = entity.item_stack().filter(|stack| stack.item_count() > 0) {
            let item_light = if glowing { GLOW_ITEM_LIGHT } else { light };
            vertices.extend(item_layer_vertices(
                layers.get(stack, renderer, tints),
                &[poses.item],
                item_light,
                false,
            ));
        }
    }
    vertices
}
