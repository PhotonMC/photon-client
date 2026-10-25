//! The active effects' icons on the HUD: the two rows, their order, the ambient and hidden ones,
//! the blink of the last ten seconds and how far left they reach.

use serde_json::{Value, json};

use super::gui;

/// An effect as the server sends it: its name, amplifier, ticks left (-1 for infinite), and
/// whether it is ambient and shows its icon.
fn effect(name: &str, amplifier: i64, duration: i64, ambient: bool, show_icon: bool) -> Value {
    json!({"effect": format!("minecraft:{name}"), "amplifier": amplifier, "duration": duration,
        "ambient": ambient, "show_icon": show_icon})
}

fn shown(name: &str, duration: i64) -> Value {
    effect(name, 0, duration, false, true)
}

fn row(label: &str, (width, height): (i64, i64), effects: Vec<Value>) -> Value {
    json!({"label": label, "width": width, "height": height, "effects": effects})
}

fn blink_rows() -> Vec<Value> {
    [201, 200, 199, 190, 150, 101, 100, 37, 21, 20, 1, 0]
        .into_iter()
        .map(|ticks| row(&format!("blink-{ticks}"), (427, 240), vec![shown("speed", ticks)]))
        .collect()
}

/// `Hud.extractEffects` of a real local player's effects.
pub fn hud_effects() -> Value {
    let size = (427, 240);
    let many = [
        "speed",
        "haste",
        "strength",
        "jump_boost",
        "regeneration",
        "resistance",
        "fire_resistance",
        "water_breathing",
        "invisibility",
        "night_vision",
        "absorption",
        "saturation",
    ];
    let mut rows = vec![
        row("none", size, vec![]),
        row("one", size, vec![shown("speed", 600)]),
        row("one-small", (320, 180), vec![shown("speed", 600)]),
        row("both-rows", size, vec![shown("speed", 600), shown("poison", 300), shown("slowness", 900)]),
        // A neutral effect sits with the harmful ones.
        row("neutral", size, vec![shown("bad_omen", 600), shown("haste", 600)]),
        row("order-by-duration", size, vec![shown("speed", 900), shown("haste", 300), shown("strength", 600)]),
        row("order-by-colour", size, vec![shown("speed", 600), shown("haste", 600), shown("night_vision", 600)]),
        row("infinite", size, vec![shown("speed", -1), shown("haste", 600)]),
        row("ambient", size, vec![effect("haste", 1, 300, true, true), shown("speed", 600)]),
        row("long-ambient", size, vec![effect("haste", 0, 40000, true, true), effect("speed", 0, 50000, true, true)]),
        row("ambient-blinks-not", size, vec![effect("haste", 0, 100, true, true)]),
        row("hidden-icon", size, vec![effect("speed", 0, 600, false, false), shown("haste", 600)]),
        row("amplified", size, vec![effect("speed", 4, 600, false, true)]),
        row("many", size, many.iter().map(|name| shown(name, 600)).collect()),
        row("many-small", (320, 180), many.iter().map(|name| shown(name, 600)).collect()),
    ];
    rows.extend(blink_rows());
    gui("hud-effects", "hud_effects", rows)
}
