//! The shulker (`ShulkerRenderer`): its colour's texture, the lid that opens
//! by the peek the server says, and the turn that puts it on the face it is
//! attached to.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::ShulkerRenderState;
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const SHULKER: &str = "minecraft:shulker";
/// The textures: the default (a shulker of no colour), then one for each dye by
/// id (`DyeColor.getId`).
const TEXTURES: [&str; 17] = [
    "entity/shulker/shulker",
    "entity/shulker/shulker_white",
    "entity/shulker/shulker_orange",
    "entity/shulker/shulker_magenta",
    "entity/shulker/shulker_light_blue",
    "entity/shulker/shulker_yellow",
    "entity/shulker/shulker_lime",
    "entity/shulker/shulker_pink",
    "entity/shulker/shulker_gray",
    "entity/shulker/shulker_light_gray",
    "entity/shulker/shulker_cyan",
    "entity/shulker/shulker_purple",
    "entity/shulker/shulker_blue",
    "entity/shulker/shulker_brown",
    "entity/shulker/shulker_green",
    "entity/shulker/shulker_red",
    "entity/shulker/shulker_black",
];

/// The paths (under `textures/`) of the textures the shulkers wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES.into_iter()
}

/// `ShulkerRenderer.getTextureLocation`.
fn texture_of(color: Option<u8>) -> &'static str {
    color
        .and_then(|id| TEXTURES.get(usize::from(id) + 1))
        .copied()
        .unwrap_or(TEXTURES[0])
}

/// `ShulkerRenderer` for the shulkers the server added.
pub fn remote_shulkers(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let Some(type_id) = photon_data::entity_type::id(SHULKER) else {
        return;
    };
    for entity in world
        .entities()
        .drawn_with_invisible()
        .filter(|entity| entity.type_id == type_id)
    {
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture_of(entity.shulker_color())),
            models.model("minecraft:shulker#main"),
        ) else {
            continue;
        };
        model.setup_shulker(&ShulkerRenderState {
            peek_amount: entity.shulker().peek(partial),
            age_in_ticks: snapshot.age_in_ticks,
            x_rot: snapshot.pitch,
            y_head_rot: entity.rotation().2,
            y_body_rot: entity.body_y_rotation(),
        });
        let mut placed = placement(world, &snapshot, view, entity.scale());
        // `ShulkerRenderer.getRenderOffset`: where it slides in from.
        if let Some(offset) = entity.shulker_render_offset(partial) {
            for (feet, shift) in placed.feet.iter_mut().zip(offset) {
                *feet += shift as f32;
            }
        }
        // `ShulkerRenderer.setupRotations`: half a turn on the body, then the
        // face it is attached to.
        placed.body_rot += 180.0;
        placed.attach_face = Some(entity.shulker_attach_face());
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
    }
}
