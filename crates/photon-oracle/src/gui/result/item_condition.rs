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
    if let Some(properties) = row["block_state"].as_object() {
        let mut bytes = Vec::new();
        write_varint(properties.len() as i32, &mut bytes);
        for (name, value) in properties {
            for text in [name.as_str(), value.as_str().unwrap_or_default()] {
                write_varint(text.len() as i32, &mut bytes);
                bytes.extend(text.as_bytes());
            }
        }
        added.push((photon_data::item::component_types::BLOCK_STATE, bytes));
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
    let mut removed: Vec<i32> = row["remove"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|name| match name.as_str()? {
            "damage" => Some(ids::DAMAGE),
            "max_damage" => Some(ids::MAX_DAMAGE),
            _ => None,
        })
        .collect();
    if row["remove_block_state"].as_bool().unwrap_or(false) {
        removed.push(photon_data::item::component_types::BLOCK_STATE);
    }
    removed
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

/// Each row's `block_state` select value read from its stack, by the production select.
pub(super) fn item_select_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let stack = stack_of(row)?;
        let state = ItemState {
            block_state: photon_render::item_model::block_state_key(
                &photon_game::item::values::Values::new(
                    stack.item_registry_id(),
                    stack.patch_bytes(),
                )
                .block_state(),
            ),
            ..ItemState::default()
        };
        let property = format!(
            "{}{}",
            photon_render::item_model::BLOCK_STATE_SELECT,
            row["property"].as_str().ok_or("property")?
        );
        let value = photon_render::item_model::select_value(
            &property,
            (photon_render::model::DisplayContext::Gui, &state),
        );
        // A property the stack lacks is no value at all, as in the Java record.
        let mut out = json!({"label": row["label"]});
        if let Some(value) = value {
            out["value"] = json!(value);
        }
        rows.push(out);
    }
    Ok(json!({"rows": rows}))
}

/// Each row's compass needle by the production wobbler and turn.
pub(super) fn item_needle_result(request: &Value) -> Result<Value, String> {
    use photon_render::item_model::{Holder, Wobbler, rotation_towards};
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let mut out = json!({"label": row["label"]});
        if row["kind"] == "wobble" {
            let mut wobbler = Wobbler::new(
                row["wobble"].as_bool().ok_or("wobble")?,
                row["factor"].as_f64().ok_or("factor")? as f32,
            );
            let mut steps = Vec::new();
            for step in row["steps"].as_array().ok_or("steps")? {
                let tick = step["tick"].as_i64().ok_or("tick")?;
                let update = wobbler.should_update(tick);
                if update {
                    wobbler.update(tick, step["target"].as_f64().ok_or("target")? as f32);
                }
                steps.push(
                    json!({"updated": update, "rotation": wobbler.rotation().to_bits() as i32}),
                );
            }
            out["steps"] = json!(steps);
        } else {
            let owner = &row["owner"];
            let holder = Holder {
                position: ["x", "y", "z"].map(|axis| owner[axis].as_f64().unwrap_or_default()),
                yaw: owner["yaw"].as_f64().ok_or("yaw")? as f32,
            };
            let block = row["block"].as_array().ok_or("block")?;
            let block = [0, 1, 2].map(|axis| block[axis].as_i64().unwrap_or_default() as i32);
            out["rotation"] = json!(rotation_towards(&holder, block, None).to_bits() as i32);
        }
        rows.push(out);
    }
    Ok(json!({"rows": rows}))
}
