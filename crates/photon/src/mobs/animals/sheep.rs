//! The sheep the frame draws: the body, and over it the undercoat and the
//! wool in the colour of the dye (`SheepRenderer`, `SheepWoolUndercoatLayer`,
//! `SheepWoolLayer`).

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{MobPlacement, QuadrupedState, SheepState};
use photon_render::entity::textures::EntityTextures;
use photon_render::entity::wool::wool_colour;
use photon_render::scene::Vertex;
use photon_world::EntityView;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The undercoat's dye is white: nothing is drawn under the white wool.
const WHITE_DYE: u8 = 0;

/// What one of the sheep's layers draws: its model layer and texture, its
/// colour, and whether it shows for a baby, a grown sheep, a sheared one.
struct Layer {
    adult: Option<(&'static str, &'static str)>,
    baby: Option<(&'static str, &'static str)>,
    coloured: bool,
    shows_sheared: bool,
    /// Drawn only for a dyed sheep.
    only_dyed: bool,
}

/// The layers, in the order they are drawn.
const LAYERS: [Layer; 3] = [
    Layer {
        adult: Some(("minecraft:sheep#main", "entity/sheep/sheep")),
        baby: Some(("minecraft:sheep_baby#main", "entity/sheep/sheep_baby")),
        coloured: false,
        shows_sheared: true,
        only_dyed: false,
    },
    Layer {
        adult: Some((
            "minecraft:sheep#wool_undercoat",
            "entity/sheep/sheep_wool_undercoat",
        )),
        baby: None,
        coloured: true,
        shows_sheared: true,
        only_dyed: true,
    },
    Layer {
        adult: Some(("minecraft:sheep#wool", "entity/sheep/sheep_wool")),
        baby: Some(("minecraft:sheep_baby#wool", "entity/sheep/sheep_wool_baby")),
        coloured: true,
        shows_sheared: false,
        only_dyed: false,
    },
];

/// The paths (under `textures/`) of the textures the sheep wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    LAYERS
        .iter()
        .flat_map(|layer| [layer.adult, layer.baby])
        .flatten()
        .map(|(_, texture)| texture)
}

/// `SheepRenderer` for the sheep the server added. The dye of a `jeb_` sheep
/// (its custom name) is not read, and the eating starts with the server's
/// entity event.
pub fn remote_sheep(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(sheep) = photon_data::entity_type::id("minecraft:sheep") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.drawn_mobs_where(|candidate| candidate == sheep) {
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let baby = entity.is_baby();
        let (eat_position, eat_angle) = entity.sheep_head_eat_scales(partial, snapshot.pitch);
        let state = SheepState {
            quadruped: QuadrupedState {
                head_y_rot: photon_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            head_eat_position_scale: eat_position,
            head_eat_angle_scale: eat_angle,
            age_scale: if baby { 0.5 } else { 1.0 },
        };
        let placement = placement(world, &snapshot, view, entity.scale());
        for (index, layer) in LAYERS.iter().enumerate() {
            vertices.extend(layer_vertices(
                (index, layer),
                (entity, &state, &placement),
                (models, textures),
            ));
        }
    }
}

/// What one layer of a sheep draws: the body (a ghost for a spectator when the sheep is
/// invisible) or a wool layer, which is not drawn on a sheep that cannot be seen.
fn layer_vertices(
    (index, layer): (usize, &Layer),
    (entity, state, placement): (EntityView<'_>, &SheepState, &MobPlacement),
    (models, textures): (&mut MobModels, &EntityTextures),
) -> Vec<Vertex> {
    let dye = entity.sheep_dye_id();
    let shown = (layer.shows_sheared || !entity.is_sheared())
        && (!layer.only_dyed || dye != WHITE_DYE)
        && (index == 0 || !entity.is_invisible());
    let Some((id, texture)) = (if entity.is_baby() {
        layer.baby
    } else {
        layer.adult
    }) else {
        return Vec::new();
    };
    let (true, Some(rect), Some(model)) = (shown, textures.rect(texture), models.model(id)) else {
        return Vec::new();
    };
    model.setup_sheep(state);
    if index == 0 {
        return crate::mobs::visible_body(entity, model, (placement, rect));
    }
    let colour = layer.coloured.then(|| wool_colour(usize::from(dye)));
    crate::mobs::outline::coloured(entity, model, placement, (rect, colour))
}
