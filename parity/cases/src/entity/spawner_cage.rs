//! Where a spawner puts the mob it shows.

use serde_json::{Value, json};

use super::pose::entity;
use crate::bits::{f32_bits, f64_bits};
use crate::line::float_text;

/// Where a spawner puts the mob it shows: the spin between two ticks, the scale that fits the
/// mob's box in the cage and the pose, for mobs that are small, tall, wide and huge.
pub fn spawner_cage() -> Vec<Value> {
    let mut rows = Vec::new();
    for mob in ["pig", "zombie", "spider", "enderman", "ghast", "chicken", "iron_golem"] {
        for partial in [0.0, 0.5, 1.0] {
            rows.push(json!({
                "label": format!("{mob} at {}", float_text(partial)),
                "type": format!("minecraft:{mob}"),
                "o_spin": f64_bits(1234.5),
                "spin": f64_bits(1238.25),
                "partial": f32_bits(partial),
            }));
        }
    }
    // What the tag of the shown entity sets: a baby, and the size of a cube mob.
    for (mob, extra) in [
        ("zombie", json!({"baby": true})),
        ("husk", json!({"baby": true})),
        ("piglin", json!({"baby": true})),
        ("zoglin", json!({"baby": true})),
        // Types whose loader does not read `IsBaby` stay adults.
        ("pig", json!({"baby": true})),
        ("piglin_brute", json!({"baby": true})),
        ("slime", json!({"size": 0})),
        ("slime", json!({"size": 1})),
        ("slime", json!({"size": 2})),
        ("magma_cube", json!({"size": 1})),
    ] {
        let mut row = json!({
            "label": format!("{mob} {extra}"),
            "type": format!("minecraft:{mob}"),
            "o_spin": f64_bits(40.0),
            "spin": f64_bits(41.5),
            "partial": f32_bits(0.5),
        });
        row.as_object_mut().expect("a row").extend(extra.as_object().expect("traits").clone());
        rows.push(row);
    }
    vec![entity("spawner-cage", "spawner_cage", rows)]
}
