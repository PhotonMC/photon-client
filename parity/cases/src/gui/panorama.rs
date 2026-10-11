//! The title screen's panorama.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;

fn view(id: String, rot_y: f64, spin: f64, ticks: f64, speed: f64) -> Value {
    case(
        format!("gui/panorama-view/{id}"),
        "gui",
        "panorama_view",
        json!({
            "rot_x": f32_bits(10.0), "rot_y": f32_bits(rot_y), "spin": f32_bits(spin),
            "ticks": f32_bits(ticks), "speed": f64_bits(speed),
        }),
    )
}

/// The cube's model-view at the title screen's pitch and a spread of spins,
/// and the spin one frame adds at several frame lengths and speeds.
pub fn panorama() -> Vec<Value> {
    let views = [
        ("zero", 0.0),
        ("first-tick", -0.1),
        ("quarter", -90.0),
        ("fraction", -45.37),
        ("wrapped-limit", 179.95),
        ("negative-limit", -180.0),
        ("small", 0.0123),
    ];
    let spins = [
        ("one-tick", 0.0, 1.0, 1.0),
        ("half-speed", 12.5, 0.4, 0.5),
        ("zero-speed", 33.0, 1.0, 0.0),
        ("wraps-up", 179.9, 2.0, 1.0),
        ("wraps-down", -179.9, 3.0, -1.0),
        ("fractional", 7.77, 0.37, 0.83),
    ];
    let views = views.iter().map(|(label, spin)| view(format!("model-view/{label}"), *spin, 0.0, 1.0, 1.0));
    let spins =
        spins.iter().map(|(label, spin, ticks, speed)| view(format!("spin/{label}"), 0.0, *spin, *ticks, *speed));
    views.chain(spins).collect()
}
