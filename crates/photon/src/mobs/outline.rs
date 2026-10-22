//! The silhouettes of the glowing mobs the frame drew, which the renderer turns into their
//! outlines (`RenderTypes.outline`, `Minecraft.shouldEntityAppearGlowing`).

use std::cell::RefCell;

use photon_game::scoreboard::Scoreboard;
use photon_render::scene::Vertex;

thread_local! {
    /// The body of each glowing mob drawn since the last hand-over, with its scoreboard name, which is its unique id.
    static SILHOUETTES: RefCell<Vec<(String, Vec<Vertex>)>> =
        const { RefCell::new(Vec::new()) };
}

/// Remember a glowing mob's body, as the vertices of its model.
pub(crate) fn record(entity: &photon_world::ClientEntity, body: Vec<Vertex>) {
    record_holder(entity.uuid.to_string_java(), body);
}

/// Remember the triangles of whatever has the scoreboard name `holder` and glows.
pub(crate) fn record_holder(holder: String, body: Vec<Vertex>) {
    SILHOUETTES.with(|list| list.borrow_mut().push((holder, body)));
}

/// Remember what a mob wears, when it glows: its armor and wings draw in the outline too
/// (`AFFECTS_OUTLINE`).
pub(crate) fn record_worn(entity: &photon_world::ClientEntity, worn: Vec<Vertex>) -> Vec<Vertex> {
    if entity.is_glowing() {
        record(entity, worn.clone());
    }
    worn
}

/// The vertices of a mob's layer in a model and a colour (a saddle, a collar, wool, a fish's
/// pattern), which join the outline when the mob glows.
pub(crate) fn coloured(
    entity: &photon_world::ClientEntity,
    model: &photon_render::entity::mob_model::MobModel,
    placement: &photon_render::entity::mob_model::MobPlacement,
    (rect, colour): (photon_render::entity::textures::TextureRect, Option<u32>),
) -> Vec<Vertex> {
    record_worn(
        entity,
        photon_render::entity::mob_model::mob_vertices_coloured(model, placement, rect, colour),
    )
}

/// The silhouettes recorded, each in its team's colour, and the list emptied. A black outline is
/// a negative colour, because zero is none.
pub(crate) fn take(board: &Scoreboard) -> Vec<Vertex> {
    let recorded = SILHOUETTES.with(|list| std::mem::take(&mut *list.borrow_mut()));
    let mut vertices = Vec::new();
    for (holder, body) in recorded {
        let rgb = photon_gui::entity_name::team_colour(&holder, board);
        let colour = if rgb == 0 {
            -1.0
        } else {
            photon_render::scene::entity_colour(rgb)
        };
        vertices.extend(body.into_iter().map(|mut vertex| {
            vertex.tint[3] = colour;
            vertex
        }));
    }
    vertices
}
