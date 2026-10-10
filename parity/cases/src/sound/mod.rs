//! The `sound` family: the sound registry, packets, decoding and engine.

mod engine;
mod packets;
mod registry;

use serde_json::{Value, json};

use crate::case::case;

/// A whole-registry table with a single `all` row.
fn table(label: &str, operation: &str) -> Value {
    case(format!("sound/{label}"), "sound", operation, json!({"rows": [{"label": "all"}]}))
}

/// Real sound files decoded by the reference's own Ogg/Vorbis stream and
/// 16-bit conversion: short and long, the kinds of sound the game plays.
fn decode() -> Value {
    let objects = [
        ("click", "random/click"),
        ("dig-stone", "dig/stone1"),
        ("step-stone", "step/stone1"),
        ("pig", "mob/pig/say1"),
        ("cave", "ambient/cave/cave1"),
        ("villager", "mob/villager/idle1"),
        ("tiny-chicken-step", "mob/chicken/baby_chicken/step"),
        ("suspicious-gravel", "block/suspicious_gravel/break1"),
        ("music", "music/game/creative/dreiton"),
    ];
    let rows: Vec<Value> = objects
        .iter()
        .map(|(label, name)| json!({"label": label, "object": format!("minecraft/sounds/{name}.ogg")}))
        .collect();
    case("sound/decode", "sound", "decode", json!({ "rows": rows }))
}

pub fn cases() -> Vec<Value> {
    vec![
        registry::registry(),
        table("events", "sound_events"),
        packets::packets(),
        decode(),
        engine::engine(),
        table("block-sounds", "block_sounds"),
        table("damage-effects", "damage_effects"),
    ]
}
