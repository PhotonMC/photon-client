//! The blocks the server asked to show for a moment (`LevelRenderer.submitTransientBlocks`),
//! drawn like a moving block where they stand.

use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::scene::Mesh;

use super::falling_blocks::moving_block_mesh;
use crate::overlays::FrameView;

/// The transient blocks that have not gone yet, each a mesh in the frame's mesh units and the
/// place to sort it from.
pub fn transient_blocks(
    session: &PlaySession,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
) -> Vec<(Mesh, [f32; 3])> {
    session
        .transient_blocks()
        .map(|block| {
            let at = block.position;
            let mut mesh = moving_block_mesh(
                session.world(),
                (renderer, tints),
                (block.state, at, at),
                view.ambient_occlusion,
            );
            let shift = [0, 1, 2].map(|axis| (at[axis] - view.origin[axis]) as f32);
            for vertex in &mut mesh.vertices {
                for (coordinate, by) in vertex.position.iter_mut().zip(shift) {
                    *coordinate += by;
                }
            }
            (mesh, shift.map(|coordinate| coordinate + 0.5))
        })
        .collect()
}
