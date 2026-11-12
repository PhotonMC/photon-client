//! A map the client holds, as the GUI draws it.

use photon_render::gui::{GuiMap, GuiMapDecoration};
use photon_world::map::MapData;

/// The map `id` as [`GuiFrame::map`](photon_render::gui::GuiFrame::map) draws it: its colours, its
/// cell of the maps' image and its decorations (the sprite of each looked up by its texture's
/// path under `textures/`, as `[u0, v0, u1, v1]`).
pub fn gui_map(
    id: i32,
    data: &MapData,
    cell: [f32; 4],
    sprite_uv: &dyn Fn(&str) -> Option<[f32; 4]>,
) -> GuiMap {
    let decorations = data
        .decorations()
        .iter()
        .filter_map(|decoration| {
            let kind = photon_data::map::decoration_type(decoration.type_id)?;
            Some(GuiMapDecoration {
                x: decoration.x,
                y: decoration.y,
                rot: decoration.rot,
                shows: kind.show_on_item_frame,
                sprite: kind
                    .asset
                    .rsplit(':')
                    .next()
                    .unwrap_or(kind.asset)
                    .to_owned(),
                uv: sprite_uv(kind.texture).unwrap_or([0.0; 4]),
                name: None,
            })
        })
        .collect();
    GuiMap {
        id,
        scale: data.scale,
        locked: data.locked,
        colors: data.colors().to_vec(),
        cell,
        decorations,
    }
}

/// A map drawer for screens with no map data to draw.
pub fn no_maps(_: i32) -> Option<GuiMap> {
    None
}
