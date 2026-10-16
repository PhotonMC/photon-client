//! The moving pistons (`PistonHeadRenderer`): the block each carries, or the piston's head, drawn
//! as it has moved by now, and the base a retracting arm leaves behind.

use photon_core::ids::BlockStateId;
use photon_data::block::blocks::id;
use photon_data::block::registry::BLOCKS;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::scene::Mesh;
use photon_world::MovingPiston;
use photon_world::block::rules;

use super::falling_blocks::moving_block_mesh;
use crate::overlays::FrameView;

/// The state of a block named by its properties.
fn state_of(block: usize, properties: &[(&str, &str)]) -> Option<BlockStateId> {
    let chosen = properties
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    BLOCKS[block].state_id(&chosen)
}

fn flag(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

/// What a moving piston shows: the block, and the base it leaves behind, each with where it stands.
fn shown(moving: &MovingPiston, progress: f32) -> (Option<BlockStateId>, Option<BlockStateId>) {
    let moved = photon_world::block::state_or_air(moving.moved_state);
    if usize::from(moved.block) == id::PISTON_HEAD && progress <= 4.0 {
        let head = rules::with_property(moving.moved_state, "short", flag(progress <= 0.5));
        return (head, None);
    }
    if moving.source && !moving.extending {
        let kind = if usize::from(moved.block) == id::STICKY_PISTON {
            "sticky"
        } else {
            "normal"
        };
        let facing = rules::property(moving.moved_state, "facing").unwrap_or("north");
        let head = state_of(
            id::PISTON_HEAD,
            &[
                ("type", kind),
                ("facing", facing),
                ("short", flag(progress >= 0.5)),
            ],
        );
        let base = rules::with_property(moving.moved_state, "extended", "true");
        return (head, base);
    }
    (Some(moving.moved_state), None)
}

/// The moving pistons of the level, each block a mesh in the frame's mesh units and the place to
/// sort it from.
pub fn piston_blocks(
    session: &PlaySession,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
) -> Vec<(Mesh, [f32; 3])> {
    let world = session.world();
    let partial = (view.partial as f32).min(1.0);
    let mut blocks = Vec::new();
    for (pos, moving) in world.moving_pistons() {
        if photon_world::block::state_or_air(moving.moved_state)
            .block()
            .air
        {
            continue;
        }
        // `getProgress`, and how far the block still is from where it ends.
        let progress = moving.progress_old + (moving.progress - moving.progress_old) * partial;
        let extended = if moving.extending {
            progress - 1.0
        } else {
            1.0 - progress
        };
        let (dx, dy, dz) = moving.direction.step();
        let offset = [dx, dy, dz].map(|step| step as f32 * extended);
        // The block is read and lit from the place it came from, back against its movement.
        let (bx, by, bz) = moving.movement_direction().opposite().step();
        let from = [pos.0 + bx, pos.1 + by, pos.2 + bz];
        let (block, base) = shown(&moving, progress);
        let drawn = [
            (block, from, offset),
            (base, [pos.0, pos.1, pos.2], [0.0; 3]),
        ];
        for (state, at, by) in drawn {
            let Some(state) = state else { continue };
            let mut mesh = moving_block_mesh(world, (renderer, tints), (state, at, at));
            let shift = [
                (pos.0 - view.origin[0]) as f32 + by[0],
                (pos.1 - view.origin[1]) as f32 + by[1],
                (pos.2 - view.origin[2]) as f32 + by[2],
            ];
            for vertex in &mut mesh.vertices {
                for (coordinate, by) in vertex.position.iter_mut().zip(shift) {
                    *coordinate += by;
                }
            }
            blocks.push((mesh, [shift[0] + 0.5, shift[1] + 0.5, shift[2] + 0.5]));
        }
    }
    blocks
}
