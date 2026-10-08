//! The ravager (`RavagerRenderer`): its texture and the pose of its attack,
//! stun and roar.
//!
//! The roar's knockback of what stands near (`applyRoarKnockbackClient`) and
//! the particles of the stun and the roar are not made.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{QuadrupedState, RavagerState, mob_vertices};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const TEXTURE: &str = "entity/illager/ravager";

/// The paths (under `textures/`) of the textures a ravager wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [TEXTURE].into_iter()
}

/// `RavagerRenderer` for the ravagers the server added.
pub fn remote_ravagers(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(ravager) = photon_data::entity_type::id("minecraft:ravager") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world
        .entities()
        .drawn()
        .filter(|entity| entity.type_id == ravager)
    {
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(TEXTURE),
            models.model("minecraft:ravager#main"),
        ) else {
            continue;
        };
        let own = entity.ravager();
        let remaining = |ticks: i32| {
            if ticks > 0 {
                ticks as f32 - partial
            } else {
                0.0
            }
        };
        model.setup_ravager(&RavagerState {
            quadruped: QuadrupedState {
                head_y_rot: photon_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            stunned_ticks_remaining: remaining(own.stunned_tick()),
            attack_ticks_remaining: remaining(own.attack_tick()),
            roar_animation: if own.roar_tick() > 0 {
                (20 - own.roar_tick()) as f32 + partial
            } else {
                0.0
            } / 20.0,
        });
        vertices.extend(mob_vertices(
            model,
            &placement(world, &snapshot, view, entity.scale()),
            rect,
        ));
    }
}
