//! The slime and the magma cube (`SlimeRenderer`, `MagmaCubeRenderer`): their
//! size and squish, the slime's translucent outer body, and the magma cube's
//! own light.

use photon_net::connection::PlaySession;
use photon_render::entity::block::sulfur_cube_block_pose;
use photon_render::entity::mob_model::{
    CubeMobScale, SulfurCubeScale, mob_vertices, overlay_code, placement_pose,
    sort_translucent_quads, sulfur_cube_overlay,
};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::EquipmentSlot;

use crate::equipment::held_items::{HeldItem, block_item_state};
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const SLIME: &str = "minecraft:slime";
const MAGMA_CUBE: &str = "minecraft:magma_cube";
const SLIME_TEXTURE: &str = "entity/slime/slime";
const MAGMA_CUBE_TEXTURE: &str = "entity/slime/magmacube";
const SULFUR_CUBE: &str = "minecraft:sulfur_cube";
/// The sulfur cube's textures: the outer shell and the inner body, each grown and small.
const SULFUR_OUTER: [&str; 2] = [
    "entity/sulfur_cube/sulfur_cube_outer",
    "entity/sulfur_cube/sulfur_cube_outer_small",
];
const SULFUR_INNER: [&str; 2] = [
    "entity/sulfur_cube/sulfur_cube_inner",
    "entity/sulfur_cube/sulfur_cube_inner_small",
];
/// The sulfur cube's layers: the shell and the inner body, grown and small.
const SULFUR_OUTER_LAYERS: [&str; 2] = [
    "minecraft:sulfur_cube#main",
    "minecraft:sulfur_cube_small#main",
];
const SULFUR_INNER_LAYERS: [&str; 2] = [
    "minecraft:sulfur_cube#inner",
    "minecraft:sulfur_cube_small#inner",
];

/// The paths (under `textures/`) of the textures the cubes wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [SLIME_TEXTURE, MAGMA_CUBE_TEXTURE]
        .into_iter()
        .chain(SULFUR_OUTER)
        .chain(SULFUR_INNER)
}

/// `AbstractCubeMobRenderer` for the slimes and the magma cubes the server added.
pub fn remote_cube_mobs(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for (name, texture_path) in [(SLIME, SLIME_TEXTURE), (MAGMA_CUBE, MAGMA_CUBE_TEXTURE)] {
        let Some(type_id) = photon_data::entity_type::id(name) else {
            continue;
        };
        let magma = name == MAGMA_CUBE;
        for entity in world
            .entities()
            .drawn()
            .filter(|entity| entity.type_id == type_id)
        {
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
                sulfur: None,
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
}

/// `SulfurCubeRenderer` for the sulfur cubes the server added: the inner body
/// first, then the shell, both translucent. A cube that holds a block draws
/// the block (`SulfurCubeInnerLayer`) in place of its inner body.
pub fn remote_sulfur_cubes(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let Some(type_id) = photon_data::entity_type::id(SULFUR_CUBE) else {
        return;
    };
    for entity in world
        .entities()
        .drawn()
        .filter(|entity| entity.type_id == type_id)
    {
        let baby = usize::from(entity.is_baby());
        let (Some(snapshot), Some(outer_rect), Some(inner_rect)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(SULFUR_OUTER[baby]),
            textures.rect(SULFUR_INNER[baby]),
        ) else {
            continue;
        };
        // `SulfurCubeRenderer.extractRenderState`.
        let fuse = if entity.sulfur_fuse() >= 0 {
            entity.sulfur_fuse() as f32 - partial + 1.0
        } else {
            0.0
        };
        // `Block.byItem(item).defaultBlockState()` of the body item; an item that is
        // no block gives air, which draws nothing.
        let block = entity
            .equipment(EquipmentSlot::Body)
            .and_then(block_item_state);
        let mut placed = placement(world, &snapshot, view, entity.scale());
        placed.cube_mob = Some(CubeMobScale {
            size: entity.cube_size(),
            squish: entity.cube_mob().squish(partial),
            downscale: true,
            sulfur: Some(SulfurCubeScale {
                fuse_remaining: fuse,
                baby: entity.is_baby(),
                holds_block: block.is_some(),
            }),
        });
        // `SulfurCubeInnerLayer`: a fuse that is lit flashes the body white.
        let mut inner_placed = placed;
        (inner_placed.red_overlay, inner_placed.white_overlay) =
            sulfur_cube_overlay(fuse, placed.red_overlay);
        if let Some(state) = block {
            let pose = sulfur_cube_block_pose(&placement_pose(&placed), entity.is_baby());
            // `SulfurCubeInnerLayer`: the overlay of the inner body's draw, so a lit fuse
            // flashes the block white.
            let overlay = overlay_code(inner_placed.red_overlay, inner_placed.white_overlay);
            held.push(HeldItem::block(state, pose, placed.light, overlay));
        } else if let Some(inner) = models.model(SULFUR_INNER_LAYERS[baby]) {
            let mut body = mob_vertices(inner, &inner_placed, inner_rect);
            sort_translucent_quads(&mut body, view.camera);
            vertices.extend(body);
        }
        if let Some(outer) = models.model(SULFUR_OUTER_LAYERS[baby]) {
            let mut shell = mob_vertices(outer, &placed, outer_rect);
            sort_translucent_quads(&mut shell, view.camera);
            vertices.extend(shell);
        }
    }
}
