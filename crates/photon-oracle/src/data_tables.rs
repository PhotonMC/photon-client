//! Tables of the data crate that a movement case compares with the reference's own.

use serde_json::{Value, json};

/// The answer for the operation `operation` when it is one of these tables.
pub(crate) fn table_result(operation: &str) -> Option<Value> {
    match operation {
        "renderer_shadows" => Some(renderer_shadows()),
        "fire_flammability" => Some(fire_flammability()),
        "compostables" => Some(compostables()),
        "game_rules" => Some(game_rules()),
        _ => None,
    }
}

/// The renderers' shadows (`EntityRenderer.shadowRadius`, `shadowStrength`) and the types whose
/// renderer the oracle could not build.
fn renderer_shadows() -> Value {
    let types: Vec<Value> = photon_data::entity_shadow::all()
        .iter()
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
        .map(|(name, ignite, burn)| json!({"name": name, "ignite": ignite, "burn": burn}))
        .collect();
    json!({"rows": [{"label": "all", "blocks": blocks}]})
}

/// `ComposterBlock.COMPOSTABLES`: the chance each item fills a layer.
fn compostables() -> Value {
    let items: Vec<Value> = photon_data::compost::all()
        .iter()
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
