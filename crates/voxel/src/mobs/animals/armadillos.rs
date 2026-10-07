//! The armadillo (`ArmadilloRenderer`): the adult's or the baby's model and
//! texture, posed from the state the server synced.

use voxel_net::connection::PlaySession;
use voxel_render::entity::mob_model::{ArmadilloRenderState, QuadrupedState, mob_vertices};
use voxel_render::entity::textures::EntityTextures;
use voxel_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const ADULT: (&str, &str) = ("minecraft:armadillo#main", "entity/armadillo/armadillo");
const BABY: (&str, &str) = (
    "minecraft:armadillo_baby#main",
    "entity/armadillo/armadillo_baby",
);

/// The paths (under `textures/`) of the textures armadillos wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [ADULT.1, BABY.1].into_iter()
}

/// `ArmadilloRenderer` for the armadillos the server added.
pub fn remote_armadillos(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let Some(armadillo) = voxel_data::entity_type::id("minecraft:armadillo") else {
        return Vec::new();
    };
    let mut vertices = Vec::new();
    for entity in world
        .entities()
        .all()
        .filter(|entity| entity.type_id == armadillo)
    {
        let (layer, texture) = if entity.is_baby() { BABY } else { ADULT };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let own = entity.armadillo().copied().unwrap_or_default();
        model.setup_armadillo(
            &ArmadilloRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: voxel_core::math::wrap_degrees_f32(
                        snapshot.head_yaw - snapshot.body_yaw,
                    ),
                    x_rot: snapshot.pitch,
                    walk_position: snapshot.walk_position,
                    walk_speed: snapshot.walk_speed,
                },
                age_in_ticks: entity.tick_count() as f32 + partial,
                is_hiding_in_shell: own.should_hide_in_shell(),
                roll_out: own.roll_out,
                roll_up: own.roll_up,
                peek: own.peek,
            },
            entity.is_baby(),
        );
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
    }
    vertices
}
