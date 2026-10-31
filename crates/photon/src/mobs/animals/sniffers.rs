//! The sniffer (`SnifferRenderer`): the adult's or the snifflet's model and
//! texture, posed from the state the server synced.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{QuadrupedState, SnifferRenderState};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const ADULT: (&str, &str) = ("minecraft:sniffer#main", "entity/sniffer/sniffer");
const BABY: (&str, &str) = ("minecraft:sniffer_baby#main", "entity/sniffer/snifflet");

/// The paths (under `textures/`) of the textures sniffers wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [ADULT.1, BABY.1].into_iter()
}

/// `SnifferRenderer` for the sniffers the server added.
pub fn remote_sniffers(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let Some(sniffer) = photon_data::entity_type::id("minecraft:sniffer") else {
        return;
    };
    for entity in world.drawn_mobs_where(|candidate| candidate == sniffer) {
        let (layer, texture) = if entity.is_baby() { BABY } else { ADULT };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let own = entity.sniffer().copied().unwrap_or_default();
        model.setup_sniffer(&SnifferRenderState {
            quadruped: QuadrupedState {
                head_y_rot: photon_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks: entity.tick_count() as f32 + partial,
            is_searching: own.is_searching(),
            digging: own.digging,
            sniffing: own.sniffing,
            rising: own.rising,
            feeling_happy: own.feeling_happy,
            scenting: own.scenting,
        });
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
    }
}
