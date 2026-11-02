use super::*;

/// `extractBlockDestroyAnimation`: each cracked block near the camera, at
/// the furthest stage anyone has it.
pub fn cracks(
    session: &PlaySession,
    (renderer, models, textures): (
        &BlockRenderer,
        &mut crate::mobs::MobModels,
        &photon_render::entity::textures::EntityTextures,
    ),
    camera: Vec3,
    view: FrameView,
) -> Vec<Vertex> {
    let mut cracked: std::collections::BTreeMap<(i32, i32, i32), i32> =
        std::collections::BTreeMap::new();
    for progress in session.world().destroying_blocks().values() {
        let centre = [progress.x, progress.y, progress.z].map(|v| f64::from(v) + 0.5);
        let distance = (centre[0] - camera.x).powi(2)
            + (centre[1] - camera.y).powi(2)
            + (centre[2] - camera.z).powi(2);
        if distance <= 1024.0 {
            let stage = cracked
                .entry((progress.x, progress.y, progress.z))
                .or_insert(-1);
            *stage = (*stage).max(progress.stage);
        }
    }
    let mut breaking = Vec::new();
    for (&(x, y, z), &stage) in &cracked {
        let Some(state) = session.world().block_state_at(x, y, z) else {
            continue;
        };
        let (quads, offset) = renderer.block_quads(state, [x, y, z]);
        // A block the model of a block entity draws has no quads of its own.
        if quads.is_empty() {
            breaking.extend(crate::world::block_entities::crumbling(
                session,
                (models, textures),
                view,
                ((x, y, z), stage),
            ));
        }
        breaking.extend(photon_render::block::breaking::breaking_vertices(
            &quads,
            offset,
            view.relative([x, y, z]),
            stage,
        ));
    }
    breaking
}
