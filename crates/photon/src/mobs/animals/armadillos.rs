//! The armadillo (`ArmadilloRenderer`): the adult's or the baby's model and
//! texture, posed from the state the server synced.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{ArmadilloRenderState, QuadrupedState};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;

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
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let Some(armadillo) = photon_data::entity_type::id("minecraft:armadillo") else {
        return;
    };
    for entity in world.drawn_mobs_where(|candidate| candidate == armadillo) {
        let (layer, texture) = if entity.is_baby() { BABY } else { ADULT };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot_of(entity, partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let own = entity.armadillo().copied().unwrap_or_default();
        model.setup_armadillo(
            &ArmadilloRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: photon_core::math::wrap_degrees_f32(
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
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
    }
}
