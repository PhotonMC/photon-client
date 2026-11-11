//! The sky flash cases.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
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
    clock_rows(&mut rows);
    case("movement/sky-flash", "movement", "sky_flash", json!({"rows": rows}))
}

/// Every colour attribute between its keyframes: the day's clock at ticks inside the segments
/// (sunrise, dusk, night), dry, in rain, in thunder and in a part of each, and with the flash.
fn clock_rows(rows: &mut Vec<Value>) {
    const CLOCKS: [i64; 18] = [
        0, 450, 1000, 3000, 6000, 11000, 12000, 12600, 13000, 13300, 13800, 15000, 18000, 21000, 22500, 22900, 23500,
        23999,
    ];
    const WEATHER: [(&str, f32, f32); 4] =
        [("dry", 0.0, 0.0), ("rain", 1.0, 0.0), ("thunder", 1.0, 1.0), ("part", 0.37, 0.21)];
    for (name, rain, thunder) in WEATHER {
        for (index, clock) in CLOCKS.iter().enumerate() {
            // The dry day is read at every tick; the weather at every other one.
            if name != "dry" && index % 2 == 1 {
                continue;
            }
            rows.push(json!({
                "label": format!("clock-{clock}-{name}"), "blocks": [], "chunk_radius": 0, "flash": 0, "hide": false,
                "clock": clock, "weather": [f32_bits(rain), f32_bits(thunder)],
                "x": f64_bits(8.5), "y": f64_bits(64.0), "z": f64_bits(8.5),
            }));
        }
    }
    for clock in [13300, 22900] {
        rows.push(json!({
            "label": format!("clock-{clock}-flash"), "blocks": [], "chunk_radius": 0, "flash": 2, "hide": false,
            "clock": clock, "weather": [f32_bits(0.6), f32_bits(0.0)],
            "x": f64_bits(8.5), "y": f64_bits(64.0), "z": f64_bits(8.5),
        }));
    }
}
