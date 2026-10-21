//! `item_condition`: the conditions an item model decides from a stack alone.

use super::*;
use photon_core::wire::write_varint;
use photon_data::item::component_types::{DYED_COLOR, LODESTONE_TRACKER};
use photon_render::item_model::{ItemState, TestedComponents};
use photon_world::ClientItemStackState;
use photon_world::item_components::ids;

/// The components a row's patch adds, as the wire carries them.
fn added_components(row: &Value) -> Result<Vec<(i32, Vec<u8>)>, String> {
    let mut added: Vec<(i32, Vec<u8>)> = Vec::new();
    for (key, id) in [("damage", ids::DAMAGE), ("max_damage", ids::MAX_DAMAGE)] {
        if let Some(value) = row[key].as_i64() {
            let mut bytes = Vec::new();
            write_varint(value as i32, &mut bytes);
            added.push((id, bytes));
        }
    }
    if row["unbreakable"].as_bool().unwrap_or(false) {
        added.push((ids::UNBREAKABLE, Vec::new()));
    }
    for name in row["components"].as_array().ok_or("components")? {
        match name.as_str().unwrap_or_default() {
            "minecraft:dyed_color" => added.push((DYED_COLOR, vec![0, 0xFF, 0, 0])),
            "minecraft:lodestone_tracker" => added.push((LODESTONE_TRACKER, vec![0, 1])),
            other => return Err(format!("unknown component {other}")),
        }
    }
    Ok(added)
}

/// The components a row's patch removes.
fn removed_components(row: &Value) -> Vec<i32> {
    row["remove"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|name| match name.as_str()? {
            "damage" => Some(ids::DAMAGE),
            "max_damage" => Some(ids::MAX_DAMAGE),
            _ => None,
        })
        .collect()
}

/// The stack a row describes: its item and its component patch.
fn stack_of(row: &Value) -> Result<ClientItemStackState, String> {
    let name = row["item"].as_str().ok_or("item")?;
    let item = photon_game::item::id_of(name).ok_or("unknown item")?;
    let added = added_components(row)?;
    let removed = removed_components(row);
    let mut patch = Vec::new();
    write_varint(added.len() as i32, &mut patch);
    write_varint(removed.len() as i32, &mut patch);
    for (id, bytes) in added {
        write_varint(id, &mut patch);
        patch.extend(bytes);
    }
    for id in removed {
        write_varint(id, &mut patch);
    }
    Ok(ClientItemStackState::new(1, item).with_patch_bytes(patch))
}

/// Each row's property decided from its stack, by the production conditions.
pub(super) fn item_condition_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let stack = stack_of(row)?;
        let state = ItemState {
            broken: stack.breaks_on_next_damage(),
            components: TestedComponents {
                dyed_color: stack.has_added_component(DYED_COLOR),
                lodestone_tracker: stack.has_added_component(LODESTONE_TRACKER),
            },
            ..ItemState::default()
        };
        let property = &row["property"];
        let component = property["component"].as_str().map(|name| {
            (
                name.to_owned(),
                property["ignore_default"].as_bool().unwrap_or(false),
            )
        });
        let value = state.condition(
            property["property"].as_str().ok_or("property")?,
            component.as_ref(),
        );
        rows.push(json!({"label": row["label"], "value": value}));
    }
    Ok(json!({"rows": rows}))
}
