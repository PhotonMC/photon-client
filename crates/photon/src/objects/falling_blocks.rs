//! The falling blocks (`FallingBlockRenderer`): the block's model tessellated as the world's is
//! (smooth light, its biome's colour) with air all about it, set where the entity is.

use photon_core::ids::BlockStateId;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::{BlockRenderer, MeshWorld};
use photon_render::scene::Mesh;

use crate::overlays::FrameView;

/// `EntityTypes.FALLING_BLOCK`'s height: the bounding box's top is where its block is read.
const HEIGHT: f32 = 0.98;

/// The falling blocks the server added, each a mesh in the frame's mesh units and the place
/// to sort it from.
pub fn falling_blocks(
    session: &PlaySession,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
) -> Vec<(Mesh, [f32; 3])> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut blocks = Vec::new();
    for entity in world.entities().drawn() {
        if photon_data::entity_type::name(entity.type_id) != Some("minecraft:falling_block") {
            continue;
        }
        // `ClientboundAddEntityPacket.getData`: the block state's number.
        let Some(state) = BlockStateId::non_negative(entity.data) else {
            continue;
        };
        let (x, y, z) = entity.position();
        // `FallingBlockRenderer.shouldRender`: not once the block has landed where it is.
        let at = [
            photon_core::math::floor_f64(x),
            photon_core::math::floor_f64(y),
            photon_core::math::floor_f64(z),
        ];
        if world.block_state_at(at[0], at[1], at[2]) == Some(state) {
            continue;
        }
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        // `extractRenderState`: the block at the top of the bounding box, seeded by where it started.
        let block = [
            at[0],
            photon_core::math::floor_f64(y + f64::from(HEIGHT)),
            at[2],
        ];
        let start = entity.start_block();
        let read = |px: i32, py: i32, pz: i32| {
            Some(if [px, py, pz] == block {
                state
            } else {
                photon_world::VANILLA_AIR_BLOCK_STATE_ID
            })
        };
        let light = |px: i32, py: i32, pz: i32| {
            (
                world.block_brightness(px, py, pz),
                world.sky_brightness(px, py, pz),
            )
        };
        // `MovingBlockRenderState.getBlockTint`: its biome's colour, not blended.
        let noise_biome = |px: i32, py: i32, pz: i32| world.noise_biome(px, py, pz);
        let biome = tints.biome_at(block[0], block[1], block[2], &noise_biome);
        let tint = |kind, position: [i32; 3]| tints.colour(kind, biome, position[0], position[2]);
        let mut mesh = renderer.mesh_moving_block(
            &MeshWorld {
                read: &read,
                light: &light,
                biome_tint: &tint,
                tags: world.block_state_registry().tags(),
            },
            block,
            [start.0, start.1, start.2],
        );
        // The entity's place, the block's corner half a block back.
        let (sx, sy, sz) = snapshot.position;
        let shift = [
            (sx - f64::from(view.origin[0])) as f32 - 0.5,
            (sy - f64::from(view.origin[1])) as f32,
            (sz - f64::from(view.origin[2])) as f32 - 0.5,
        ];
        for vertex in &mut mesh.vertices {
            for (coordinate, by) in vertex.position.iter_mut().zip(shift) {
                *coordinate += by;
            }
        }
        blocks.push((mesh, [shift[0] + 0.5, shift[1] + 0.5, shift[2] + 0.5]));
    }
    blocks
}
