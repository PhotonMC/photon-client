//! The slime and the magma cube (`SlimeRenderer`, `MagmaCubeRenderer`): their
//! size and squish, the slime's translucent outer body, and the magma cube's
//! own light.

use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{CubeMobScale, mob_vertices, sort_translucent_quads};
use voxel_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const SLIME: &str = "minecraft:slime";
const MAGMA_CUBE: &str = "minecraft:magma_cube";
const SLIME_TEXTURE: &str = "entity/slime/slime";
const MAGMA_CUBE_TEXTURE: &str = "entity/slime/magmacube";

/// The paths (under `textures/`) of the textures the cubes wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [SLIME_TEXTURE, MAGMA_CUBE_TEXTURE].into_iter()
}

/// `AbstractCubeMobRenderer` for the slimes and the magma cubes the server added.
pub fn remote_cube_mobs(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for (name, texture_path) in [(SLIME, SLIME_TEXTURE), (MAGMA_CUBE, MAGMA_CUBE_TEXTURE)] {
        let Some(type_id) = voxel_data::entity_type::id(name) else {
            continue;
        };
        let magma = name == MAGMA_CUBE;
        for entity in world.entities().filter(|entity| entity.type_id == type_id) {
            let (Some(snapshot), Some(rect)) = (
                world.entity_snapshot(entity.id(), partial),
                textures.rect(texture_path),
            ) else {
                continue;
            };
            let squish = entity.cube_mob().squish(partial);
            let mut placed = placement(world, &snapshot, view, entity.scale());
            placed.cube_mob = Some(CubeMobScale {
                size: entity.cube_size(),
                squish,
                downscale: !magma,
            });
            if magma {
                // `MagmaCubeRenderer.getBlockLightLevel`, on the packed light.
                placed.light |= 15 << 4;
                let Some(model) = models.model("minecraft:magma_cube#main") else {
                    continue;
                };
                model.setup_magma_cube(squish);
                vertices.extend(mob_vertices(model, &placed, rect));
                continue;
            }
            let Some(model) = models.model("minecraft:slime#main") else {
                continue;
            };
            vertices.extend(mob_vertices(model, &placed, rect));
            // `SlimeOuterLayer`: the outer body, translucent, drawn after.
            let Some(outer) = models.model("minecraft:slime#outer") else {
                continue;
            };
            let mut shell = mob_vertices(outer, &placed, rect);
            sort_translucent_quads(&mut shell, view.camera);
            vertices.extend(shell);
        }
    }
    vertices
}
