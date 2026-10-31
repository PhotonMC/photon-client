//! The axolotl (`AxolotlRenderer`): its texture by variant, grown or baby, and
//! its pose: an adult's blend of swimming, hovering, crawling and lying still
//! by the eased factors of its tick, a baby's keyframe animations.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{AxolotlRenderState, QuadrupedState};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::EntityView;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The textures by variant (`lucy`, `wild`, `gold`, `cyan`, `blue`): grown, then baby.
const TEXTURES: [[&str; 2]; 5] = [
    [
        "entity/axolotl/axolotl_lucy",
        "entity/axolotl/axolotl_lucy_baby",
    ],
    [
        "entity/axolotl/axolotl_wild",
        "entity/axolotl/axolotl_wild_baby",
    ],
    [
        "entity/axolotl/axolotl_gold",
        "entity/axolotl/axolotl_gold_baby",
    ],
    [
        "entity/axolotl/axolotl_cyan",
        "entity/axolotl/axolotl_cyan_baby",
    ],
    [
        "entity/axolotl/axolotl_blue",
        "entity/axolotl/axolotl_blue_baby",
    ],
];

/// The paths (under `textures/`) of the textures the axolotls wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES.into_iter().flatten()
}

/// The state the axolotl renders in: its walk, its animation blends and the poses it is in.
fn render_state(
    entity: EntityView<'_>,
    snapshot: &photon_world::EntitySnapshot,
    partial: f32,
) -> AxolotlRenderState {
    let own = entity.axolotl().copied().unwrap_or_default();
    AxolotlRenderState {
        quadruped: QuadrupedState {
            head_y_rot: photon_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        },
        age_in_ticks: entity.tick_count() as f32 + partial,
        playing_dead_factor: own.playing_dead.factor(partial),
        moving_factor: own.moving.factor(partial),
        in_water_factor: own.in_water.factor(partial),
        on_ground_factor: own.on_ground.factor(partial),
        swim: own.swim,
        walk: own.walk,
        idle_under_water: own.idle_under_water,
        idle_under_water_on_ground: own.idle_under_water_on_ground,
        idle_on_ground: own.idle_on_ground,
        play_dead: own.play_dead,
    }
}

/// `AxolotlRenderer` for the axolotls the server added.
pub fn remote_axolotls(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(axolotl) = photon_data::entity_type::id("minecraft:axolotl") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.drawn_mobs_where(|candidate| candidate == axolotl) {
        let baby = entity.is_baby();
        let texture = TEXTURES[entity.axolotl_variant()][usize::from(baby)];
        let layer = if baby {
            "minecraft:axolotl_baby#main"
        } else {
            "minecraft:axolotl#main"
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot_of(entity, partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let state = render_state(entity, &snapshot, partial);
        if baby {
            model.setup_axolotl_baby(&state);
        } else {
            model.setup_axolotl_adult(&state);
        }
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
    }
}
