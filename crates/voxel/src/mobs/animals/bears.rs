//! The polar bear (`PolarBearRenderer`): its texture, grown or baby, and how
//! far it has stood up.

use voxel_net::connection::PlaySession;
use voxel_render::entity::mob_model::{PolarBearState, QuadrupedState, mob_vertices};
use voxel_render::entity::textures::EntityTextures;
use voxel_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The polar bear's textures, grown and baby.
const POLAR_BEAR: [&str; 2] = ["entity/bear/polarbear", "entity/bear/polarbear_baby"];

/// The paths (under `textures/`) of the textures the bears wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    POLAR_BEAR.into_iter()
}

/// `PolarBearRenderer` for the polar bears the server added.
pub fn remote_bears(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let Some(polar_bear) = voxel_data::entity_type::id("minecraft:polar_bear") else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world
        .entities()
        .all()
        .filter(|entity| entity.type_id == polar_bear)
    {
        let baby = entity.is_baby();
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(POLAR_BEAR[usize::from(baby)]),
            models.model(if baby {
                "minecraft:polar_bear_baby#main"
            } else {
                "minecraft:polar_bear#main"
            }),
        ) else {
            continue;
        };
        model.setup_polar_bear(&PolarBearState {
            quadruped: QuadrupedState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            stand_scale: entity.polar_bear().standing_animation_scale(partial),
            age_scale: if baby { 0.5 } else { 1.0 },
        });
        vertices.extend(mob_vertices(
            model,
            &placement(world, &snapshot, view, entity.scale()),
            rect,
        ));
    }
    vertices
}
