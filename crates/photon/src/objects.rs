//! Entities that are not mobs: projectiles, vehicles, hanging and falling things.

pub(crate) mod arrows;
pub(crate) mod block_entity_items;
pub(crate) mod boats;
pub(crate) mod bullets;
pub(crate) mod charges;
pub(crate) mod displays;
pub(crate) mod dropped_items;
pub(crate) mod end_crystals;
pub(crate) mod falling_blocks;
pub(crate) mod fishing_hooks;
pub(crate) mod flames;
pub(crate) mod item_frames;
pub(crate) mod leashes;
pub(crate) mod lightning_bolts;
pub(crate) mod minecarts;
pub(crate) mod paintings;
pub(crate) mod piston_blocks;
pub(crate) mod thrown_items;
pub(crate) mod transient_blocks;

/// Every block drawn as a moving block this frame: the falling ones, the pistons' and the
/// transient ones, each a mesh in the frame's mesh units and the place to sort it from.
pub(crate) fn moving_blocks(
    session: &photon_net::connection::PlaySession,
    tints: (
        &photon_render::block::world_mesh::BlockRenderer,
        &photon_render::block::biome_tint::BiomeTints,
    ),
    view: crate::overlays::FrameView,
) -> Vec<(photon_render::scene::Mesh, [f32; 3])> {
    let mut moving = falling_blocks::falling_blocks(session, tints, view);
    moving.extend(piston_blocks::piston_blocks(session, tints, view));
    moving.extend(transient_blocks::transient_blocks(session, tints, view));
    moving
}
