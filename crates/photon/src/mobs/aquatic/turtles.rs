//! The turtle (`TurtleRenderer`): its texture, grown or baby, the rowing of
//! its flippers on land and the paddling in water, and the egg on its belly.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{QuadrupedState, TurtleState};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The turtle's textures and model layers: grown, then baby.
const TEXTURES: [&str; 2] = ["entity/turtle/turtle", "entity/turtle/turtle_baby"];
const LAYERS: [&str; 2] = ["minecraft:turtle#main", "minecraft:turtle_baby#main"];

/// The paths (under `textures/`) of the textures the turtles wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES.into_iter()
}

/// `TurtleRenderer` for the turtles the server added.
pub fn remote_turtles(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(turtle) = photon_data::entity_type::id("minecraft:turtle") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.drawn_mobs_where(|candidate| candidate == turtle) {
        let baby = entity.is_baby();
        let index = usize::from(baby);
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot_of(entity, partial),
            textures.rect(TEXTURES[index]),
            models.model(LAYERS[index]),
        ) else {
            continue;
        };
        model.setup_turtle(&TurtleState {
            quadruped: QuadrupedState {
                head_y_rot: photon_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            is_on_land: !entity.is_in_water() && entity.on_ground(),
            is_laying_egg: entity.turtle_laying_egg(),
            has_egg: !baby && entity.turtle_has_egg(),
        });
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
    }
}
