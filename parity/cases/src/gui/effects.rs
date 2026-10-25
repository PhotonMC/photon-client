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

/// One inventory list: a 176 by 166 image at `left` (the recipe book moves it), the mouse, the
/// effects and the tick rate.
fn listed(label: &str, (width, height): (i64, i64), left: i64, mouse: (i64, i64), effects: Vec<Value>) -> Value {
    json!({"label": label, "width": width, "height": height, "image_width": 176, "image_height": 166,
        "left": left, "mouse_x": mouse.0, "mouse_y": mouse.1, "effects": effects})
}

/// `EffectsInInventory` beside the survival inventory: full entries, icon-only ones, none, the
/// recipe book's shift, the squeeze past five, a cut name and the tooltips of cut entries.
pub fn inventory_effects() -> Value {
    let wide = (427, 240);
    let centred = |width: i64| (width - 176) / 2;
    let away = (0, 0);
    // The first entry's top left corner at the wide size, inside it.
    let first = (310, 45);
    let three = vec![shown("speed", 600), shown("poison", 300), effect("haste", 1, 1200, false, true)];
    let seven = ["speed", "haste", "strength", "jump_boost", "regeneration", "resistance", "fire_resistance"]
        .iter()
        .map(|name| shown(name, 600))
        .collect();
    let mut rows = vec![
        listed("none", wide, centred(427), away, vec![]),
        listed("three", wide, centred(427), away, three.clone()),
        listed("icons-only", (320, 240), centred(320), away, three.clone()),
        listed("icons-only-tooltip", (320, 240), centred(320), (260, 50), three.clone()),
        listed("no-room", (200, 240), centred(200), away, three.clone()),
        listed("book-open", wide, 177 + (427 - 376) / 2, away, three.clone()),
        listed("seven", wide, centred(427), away, seven),
        listed("full-entry-no-tooltip", wide, centred(427), first, vec![shown("speed", 600)]),
        listed("cut-name", wide, centred(427), away, vec![shown("hero_of_the_village", 600)]),
        listed("cut-name-tooltip", wide, centred(427), first, vec![shown("hero_of_the_village", 600)]),
        listed("infinite-and-hours", wide, centred(427), away, vec![shown("speed", -1), shown("haste", 80000)]),
        listed(
            "levels",
            wide,
            centred(427),
            away,
            vec![
                effect("speed", 1, 600, false, true),
                effect("haste", 9, 600, false, true),
                effect("strength", 10, 600, false, true),
            ],
        ),
        listed("ambient", wide, centred(427), away, vec![effect("haste", 0, 600, true, true), shown("speed", 600)]),
        listed("hidden-icon-listed", wide, centred(427), away, vec![effect("speed", 0, 600, false, false)]),
    ];
    let mut slow = listed("tick-rate", wide, centred(427), away, vec![shown("speed", 600)]);
    slow["tick_rate"] = json!(10.0);
    rows.push(slow);
    gui("inventory-effects", "inventory_effects", rows)
}
