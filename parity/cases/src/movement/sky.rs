//! The sky flash cases.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;

/// The sky colour and the sky light factor with and without a lightning
/// flash, with the option that hides it, from the real client level's
/// attribute system.
pub fn sky_flash() -> Value {
    let mut rows = Vec::new();
    for flash in 0..3 {
        for hide in [false, true] {
            rows.push(json!({
                "label": format!("flash-{flash}-hide-{}", if hide { "True" } else { "False" }),
                "blocks": [], "chunk_radius": 0, "flash": flash, "hide": hide,
                "x": f64_bits(8.5), "y": f64_bits(64.0), "z": f64_bits(8.5),
            }));
        }
    }
    for y in [-60.0, 200.0, 320.0] {
        rows.push(json!({
            "label": format!("flash-2-y-{y:?}"), "blocks": [], "chunk_radius": 0, "flash": 2, "hide": false,
            "x": f64_bits(-300.25), "y": f64_bits(y), "z": f64_bits(1000.5),
        }));
    }
    case("movement/sky-flash", "movement", "sky_flash", json!({"rows": rows}))
}
