//! Tables of the data crate that a movement case compares with the reference's own.

use serde_json::{Value, json};

/// The answer for the operation `operation` when it is one of these tables.
pub(crate) fn table_result(operation: &str) -> Option<Value> {
    match operation {
        "renderer_shadows" => Some(renderer_shadows()),
        "fire_flammability" => Some(fire_flammability()),
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
