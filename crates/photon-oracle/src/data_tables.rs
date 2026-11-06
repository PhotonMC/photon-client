//! Tables of the data crate that a movement case compares with the reference's own.

use serde_json::{Value, json};

/// The answer for the operation `operation` when it is one of these tables.
pub(crate) fn table_result(operation: &str) -> Option<Value> {
    match operation {
        "renderer_shadows" => Some(renderer_shadows()),
        "fire_flammability" => Some(fire_flammability()),
        "compostables" => Some(compostables()),
        "game_rules" => Some(game_rules()),
        "block_map_colors" => Some(block_map_colors()),
        "leaf_particles" => Some(leaf_particles()),
        _ => None,
    }
}

/// The renderers' shadows (`EntityRenderer.shadowRadius`, `shadowStrength`) and the types whose
/// renderer the oracle could not build.
fn renderer_shadows() -> Value {
    let types: Vec<Value> = photon_data::entity_shadow::all()
        .iter()
        .filter(|(name, ..)| {
            photon_data::entity_type::id(name).is_some_and(|id| {
                crate::case_ids::registries()
                    .entity_types
                    .to_wire(id.0 as u32)
                    .is_some()
            })
        })
        .map(|(name, radius, strength)| {
            json!({
                "name": name,
                "radius": crate::declarations::bits(*radius),
                "strength": crate::declarations::bits(*strength),
            })
        })
        .collect();
    let failed = photon_data::entity_shadow::unbuilt();
    json!({"rows": [{"label": "all", "types": types, "failed": failed}]})
}

/// `FireBlock`'s odds.
fn fire_flammability() -> Value {
    let blocks: Vec<Value> = photon_data::fire::all()
        .iter()
        .filter(|(name, ..)| crate::case_ids::registries().has_block(name))
        .map(|(name, ignite, burn)| json!({"name": name, "ignite": ignite, "burn": burn}))
        .collect();
    json!({"rows": [{"label": "all", "blocks": blocks}]})
}

/// `ComposterBlock.COMPOSTABLES`: the chance each item fills a layer.
fn compostables() -> Value {
    let items: Vec<Value> = photon_data::compost::all()
        .iter()
        .filter(|(name, _)| crate::case_ids::registries().has_item(name))
        .map(|(name, chance)| json!({"name": name, "chance": crate::declarations::bits(*chance)}))
        .collect();
    json!({"rows": [{"label": "all", "items": items}]})
}

/// The built-in game rules: what the in-world rule screen lists and bounds.
fn game_rules() -> Value {
    let rules: Vec<Value> = photon_data::game_rules::all()
        .iter()
        .map(|rule| {
            let mut entry = json!({
                "name": rule.name,
                "category": rule.category,
                "type": if rule.bounds.is_some() { "integer" } else { "boolean" },
                "default": rule.default,
                "vanilla": rule.vanilla,
            });
            if let Some((min, max)) = rule.bounds {
                entry["min"] = json!(min);
                entry["max"] = json!(max);
            }
            entry
        })
        .collect();
    json!({"rows": [{"label": "all", "rules": rules}]})
}

/// The map colours and dust colours, as the data crate carries them.
fn block_map_colors() -> Value {
    // The map colours and dust colours, as the data crate carries them.
    let (blocks, states, dust) = photon_data::map::map_color_tables();
    let rows: Vec<Value> = blocks
        .iter()
        .filter(|(name, _)| crate::case_ids::registries().has_block(name))
        .map(|(name, color)| {
            let others: Vec<Value> = states
                .iter()
                .filter(|(state, _)| {
                    photon_data::block::registry::block_for_state(crate::case_ids::block_state(
                        *state as i32,
                    ))
                    .is_some_and(|(block, _)| block.name == *name)
                })
                .map(|(state, color)| json!([state, color]))
                .collect();
            let mut entry = json!({"name": name, "color": color, "states": others});
            if let Some((_, color)) = dust.iter().find(|(block, _)| block == name) {
                entry["dust"] = json!(color);
            }
            entry
        })
        .collect();
    json!({"rows": [{"label": "all", "blocks": rows}]})
}

/// The leaves that let a leaf fall.
fn leaf_particles() -> Value {
    let blocks: Vec<Value> = photon_data::leaves::all()
        .iter()
        .filter(|(name, ..)| crate::case_ids::registries().has_block(name))
        .map(|(name, chance, particle, color)| {
            let mut entry = json!({
                "name": name,
                "chance": format!("{:08x}", chance.to_bits()),
                "particle": particle,
            });
            if let Some(color) = color {
                entry["color"] = json!(color);
            }
            entry
        })
        .collect();
    json!({"rows": [{"label": "all", "blocks": blocks}]})
}

/// The block sound types and which each block uses.
pub(crate) fn block_sounds() -> Value {
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let mut rows: Vec<Value> = photon_data::block_sound::sound_types()
        .filter(|row| sound_in_version(row.break_sound()))
            .map(|row| {
                json!({"label": format!("type:{}", row.name), "kind": "type", "name": row.name,
                    "volume": bits(row.volume), "pitch": bits(row.pitch),
                    "break": row.break_sound(), "step": row.step_sound(), "place": row.place_sound(),
                    "hit": row.hit_sound(), "fall": row.fall_sound()})
            })
            .collect();
    let overrides: Vec<(u32, &str)> = photon_data::block_sound::state_overrides()
        .map(|(state, row)| (state, row.name))
        .collect();
    for block in crate::case_ids::blocks() {
        let name = block.name;
        let Some(row) = photon_data::block_sound::of_block(name) else {
            continue;
        };
        let own: Vec<Value> = overrides
            .iter()
            .filter(|(state, _)| {
                photon_world::block::state_or_air(photon_core::ids::BlockStateId(*state as i32))
                    .block()
                    .name
                    == name
            })
            .map(|(state, type_name)| {
                json!([
                    crate::case_ids::wire_state(photon_core::ids::BlockStateId(*state as i32)),
                    type_name
                ])
            })
            .collect();
        rows.push(
            json!({"label": format!("block:{name}"), "kind": "block", "name": name,
                "type": row.name, "overrides": own}),
        );
    }
    json!({"rows": rows})
}

/// Whether the sound event exists in the oracle's version.
fn sound_in_version(name: &str) -> bool {
    photon_data::sound_event::id(name).is_some_and(|id| {
        crate::case_ids::registries()
            .sound_events
            .to_wire(id as u32)
            .is_some()
    })
}
