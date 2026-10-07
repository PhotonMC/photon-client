//! The chests of the world (`ChestRenderer`): each block entity's chest model, in the texture of
//! its kind, turned to face, its lid as far open as the server's block events have it.

use voxel_core::direction::Direction;
use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::chest::{ChestKind, ChestMaterial, chest_pose, eased_open};
use voxel_render::entity_textures::EntityTextures;
use voxel_render::joml::Pose;
use voxel_render::mob_model::mob_vertices_posed;
use voxel_render::scene::Vertex;

use crate::mobs::MobModels;
use crate::overlays::{FrameView, packed_light};

/// How far from the camera, in chunks, chests are drawn.
const RADIUS_CHUNKS: i32 = 8;

/// The paths (under `textures/`) of every chest texture.
const TEXTURES: [&str; 19] = [
    "entity/chest/normal",
    "entity/chest/normal_left",
    "entity/chest/normal_right",
    "entity/chest/trapped",
    "entity/chest/trapped_left",
    "entity/chest/trapped_right",
    "entity/chest/copper",
    "entity/chest/copper_left",
    "entity/chest/copper_right",
    "entity/chest/copper_exposed",
    "entity/chest/copper_exposed_left",
    "entity/chest/copper_exposed_right",
    "entity/chest/copper_weathered",
    "entity/chest/copper_weathered_left",
    "entity/chest/copper_weathered_right",
    "entity/chest/copper_oxidized",
    "entity/chest/copper_oxidized_left",
    "entity/chest/copper_oxidized_right",
    "entity/chest/ender",
];

pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES.into_iter()
}

/// `ChestRenderer.getChestMaterial` from the block's name.
fn material_of(name: &str) -> ChestMaterial {
    match name {
        "minecraft:ender_chest" => ChestMaterial::Ender,
        "minecraft:trapped_chest" => ChestMaterial::Trapped,
        "minecraft:copper_chest" | "minecraft:waxed_copper_chest" => {
            ChestMaterial::CopperUnaffected
        }
        "minecraft:exposed_copper_chest" | "minecraft:waxed_exposed_copper_chest" => {
            ChestMaterial::CopperExposed
        }
        "minecraft:weathered_copper_chest" | "minecraft:waxed_weathered_copper_chest" => {
            ChestMaterial::CopperWeathered
        }
        "minecraft:oxidized_copper_chest" | "minecraft:waxed_oxidized_copper_chest" => {
            ChestMaterial::CopperOxidized
        }
        _ => ChestMaterial::Regular,
    }
}

/// `LightCoordsUtil.max`: each of the block and sky light the greater.
fn max_light(first: i32, second: i32) -> i32 {
    let block = |packed: i32| packed >> 4 & 15;
    let sky = |packed: i32| packed >> 20 & 15;
    block(first).max(block(second)) << 4 | sky(first).max(sky(second)) << 20
}

/// The chests near the camera.
pub fn chests(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let centre = (
        (view.origin[0] + view.camera[0].floor() as i32).div_euclid(16),
        (view.origin[2] + view.camera[2].floor() as i32).div_euclid(16),
    );
    let mut vertices = Vec::new();
    for (x, y, z) in world.chest_positions_near(centre, RADIUS_CHUNKS) {
        let Some(chest) = world.chest_view(x, y, z) else {
            continue;
        };
        let kind = match chest.kind {
            1 => ChestKind::Left,
            2 => ChestKind::Right,
            _ => ChestKind::Single,
        };
        // The ender chest has one half only, whatever its state says.
        let material = material_of(chest.name);
        let kind = if material == ChestMaterial::Ender {
            ChestKind::Single
        } else {
            kind
        };
        let (Some(rect), Some(model)) = (
            textures.rect(&material.texture(kind)),
            models.model(kind.layer()),
        ) else {
            continue;
        };
        let at =
            |pos: (i32, i32, i32)| Vec3::new(f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        let mut light = packed_light(world, at((x, y, z)));
        if let Some(other) = chest.partner {
            light = max_light(light, packed_light(world, at(other)));
        }
        let open = world.chest_openness((x, y, z), chest.partner, partial);
        model.setup_chest(eased_open(open));
        let mut base = Pose::default();
        base.translate(
            (x - view.origin[0]) as f32,
            (y - view.origin[1]) as f32,
            (z - view.origin[2]) as f32,
        );
        let facing: Direction = chest.facing;
        let pose = chest_pose(&base, facing);
        vertices.extend(mob_vertices_posed(model, &pose, rect, (light, false)));
    }
    vertices
}
