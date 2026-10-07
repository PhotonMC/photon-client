//! The squid and the glow squid (`SquidRenderer`, `GlowSquidRenderer`): their
//! textures, grown or baby, the beat of their tentacles and the turns of
//! their bodies in the water, and the light a glow squid gives itself.

use voxel_net::connection::PlaySession;
use voxel_render::entity::mob_model::{SquidTurns, glow_squid_block_light, mob_vertices};
use voxel_render::entity::textures::EntityTextures;
use voxel_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The squids: the entity type, its textures and its model layers, grown and baby.
const SQUIDS: [(&str, [&str; 2], [&str; 2]); 2] = [
    (
        "minecraft:squid",
        ["entity/squid/squid", "entity/squid/squid_baby"],
        ["minecraft:squid#main", "minecraft:squid_baby#main"],
    ),
    (
        "minecraft:glow_squid",
        ["entity/squid/glow_squid", "entity/squid/glow_squid_baby"],
        [
            "minecraft:glow_squid#main",
            "minecraft:glow_squid_baby#main",
        ],
    ),
];

/// The paths (under `textures/`) of the textures the squids wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    SQUIDS.into_iter().flat_map(|(_, textures, _)| textures)
}

/// `SquidRenderer` for the squids the server added.
pub fn remote_squids(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for (name, texture_paths, layers) in SQUIDS {
        let Some(type_id) = voxel_data::entity_type::id(name) else {
            continue;
        };
        for entity in world
            .entities()
            .drawn()
            .filter(|entity| entity.type_id == type_id)
        {
            let baby = entity.is_baby();
            let index = usize::from(baby);
            let (Some(snapshot), Some(rect), Some(model)) = (
                world.entity_snapshot(entity.id(), partial),
                textures.rect(texture_paths[index]),
                models.model(layers[index]),
            ) else {
                continue;
            };
            // Until it has ticked, a squid hangs as it was modelled.
            let pose = entity.squid().map(|squid| squid.pose(partial));
            model.setup_squid(pose.map_or(0.0, |pose| pose.tentacle_angle));
            let mut placed = placement(world, &snapshot, view, entity.scale());
            placed.squid = Some(SquidTurns {
                x_body_rot: pose.map_or(0.0, |pose| pose.x_body_rot),
                z_body_rot: pose.map_or(0.0, |pose| pose.z_body_rot),
                baby,
            });
            if name == "minecraft:glow_squid" {
                // `GlowSquidRenderer.getBlockLightLevel`, on the packed light.
                let block = (placed.light >> 4) & 15;
                let block = glow_squid_block_light(entity.glow_dark_ticks(), block);
                placed.light = (placed.light & !(15 << 4)) | (block << 4);
            }
            vertices.extend(mob_vertices(model, &placed, rect));
        }
    }
}
