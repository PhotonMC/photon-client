use super::*;

/// The map a row describes (`map`: id, scale, locked, colour runs, decorations) as the client
/// holds it, made the way the client makes one: from a map item data packet.
pub(super) fn seeded_map(row: &Value) -> Result<Option<(i32, photon_world::map::MapData)>, String> {
    let Some(map) = row.get("map") else {
        return Ok(None);
    };
    let id = map["id"].as_i64().ok_or("map id")? as i32;
    let mut colors = vec![0u8; photon_world::map::MAP_SIDE * photon_world::map::MAP_SIDE];
    for run in map["runs"].as_array().ok_or("map runs")? {
        let [start, length, packed] = [0, 1, 2].map(|at| run[at].as_i64().unwrap_or(0) as usize);
        colors[start..start + length].fill(packed as u8);
    }
    let mut decorations = Vec::new();
    for decoration in map["decorations"].as_array().ok_or("map decorations")? {
        let name = decoration["type"].as_str().ok_or("decoration type")?;
        let type_id = photon_data::map::decoration_types()
            .iter()
            .position(|kind| kind.name.trim_start_matches("minecraft:") == name)
            .ok_or("unknown decoration type")?;
        decorations.push(photon_world::map::MapDecoration {
            type_id: type_id as i32,
            x: decoration["x"].as_i64().unwrap_or(0) as i8,
            y: decoration["y"].as_i64().unwrap_or(0) as i8,
            rot: decoration["rot"].as_i64().unwrap_or(0) as i8,
            name: decoration["name"].as_str().map(name_tag),
        });
    }
    let mut world = photon_world::ClientWorld::default();
    world.apply_map_item_data(&photon_world::map::MapItemDataInput {
        map_id: id,
        scale: map["scale"].as_i64().unwrap_or(0) as i8,
        locked: map["locked"].as_bool().unwrap_or(false),
        decorations: Some(decorations),
        patch: Some(photon_world::map::MapPatch {
            start_x: 0,
            start_y: 0,
            width: photon_world::map::MAP_SIDE,
            height: photon_world::map::MAP_SIDE,
            colors,
        }),
    });
    Ok(world.map_data(id).cloned().map(|data| (id, data)))
}

/// The maps a screen asks for: the one the row seeded, as the GUI draws it, with the whole of a
/// sprite standing in for the picture's place and each decoration's sprite.
pub(super) fn drawer<'a>(
    seeded: &'a Option<(i32, photon_world::map::MapData)>,
    lang: &'a photon_game::text::Lang,
) -> impl FnMut(i32) -> Option<photon_render::gui::GuiMap> + 'a {
    move |id| {
        seeded
            .as_ref()
            .filter(|(seeded_id, _)| *seeded_id == id)
            .map(|(_, data)| {
                photon_gui::map_view::gui_map(
                    id,
                    data,
                    [0.0; 4],
                    &|_| Some([0.0, 0.0, 1.0, 1.0]),
                    lang,
                )
            })
    }
}

/// A name as the network carries a plain-text component: a string tag with no root name.
fn name_tag(text: &str) -> Vec<u8> {
    let mut bytes = vec![8];
    bytes.extend_from_slice(&(text.len() as u16).to_be_bytes());
    bytes.extend_from_slice(text.as_bytes());
    bytes
}
