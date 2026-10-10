//! The reference's own sound registry: small `sounds.json` texts (weights,
//! borrowed events, missing files, the things a file can get wrong), picked
//! from with seeded random sources, and the game's real `sounds.json`.

use serde_json::{Value, json};

const FILES: &[&str] = &["a", "b", "c", "dir/d"];

/// Event, seed and pick count.
type Pick = (&'static str, i64, u32);

/// Label, `sounds.json` text and picks.
const SMALL: &[(&str, &str, &[Pick])] = &[
    ("plain-names", r#"{"one": {"sounds": ["a", "b", "c"]}}"#, &[("minecraft:one", 7, 12)]),
    (
        "weights",
        r#"{"w": {"sounds": [{"name": "a", "weight": 5}, {"name": "b"}, {"name": "c", "weight": 2}]}}"#,
        &[("minecraft:w", 1, 30), ("minecraft:w", -42, 30), ("minecraft:w", 123456789, 30)],
    ),
    (
        "fields",
        r#"{"f": {"subtitle": "subtitles.f", "sounds": [{"name": "a", "volume": 0.75, "pitch": 1.5, "stream": true, "preload": true, "attenuation_distance": 48}, {"name": "dir/d", "volume": 0.3}]}}"#,
        &[("minecraft:f", 3, 10)],
    ),
    (
        "event-entries",
        r#"{"base": {"sounds": [{"name": "a", "volume": 0.5, "weight": 3}, {"name": "b", "pitch": 2.0}]}, "wrap": {"sounds": [{"name": "base", "type": "event", "volume": 0.5, "pitch": 0.5, "weight": 9}, "c"]}, "wrap2": {"sounds": [{"name": "wrap", "type": "event", "stream": true}]}}"#,
        &[("minecraft:wrap", 5, 24), ("minecraft:wrap2", 11, 24)],
    ),
    (
        "missing-event-entry",
        r#"{"w": {"sounds": [{"name": "nowhere", "type": "event"}, "a"]}}"#,
        &[("minecraft:w", 2, 12)],
    ),
    ("missing-files", r#"{"m": {"sounds": ["gone", "a", {"name": "also/gone"}]}}"#, &[("minecraft:m", 4, 8)]),
    ("empty-event", r#"{"e": {"sounds": []}, "n": {}}"#, &[("minecraft:e", 1, 3), ("minecraft:n", 1, 3)]),
    ("replace", r#"{"r": {"sounds": ["a"], "replace": true}, "s": {"subtitle": "x"}}"#, &[("minecraft:r", 1, 4)]),
];

const INVALID: &[(&str, &str)] = &[
    ("zero-volume", r#"{"good": {"sounds": ["a"]}, "bad": {"sounds": [{"name": "a", "volume": 0}]}}"#),
    ("negative-pitch", r#"{"bad": {"sounds": [{"name": "a", "pitch": -1.0}]}}"#),
    ("zero-weight", r#"{"bad": {"sounds": [{"name": "a", "weight": 0}]}}"#),
    ("bad-type", r#"{"bad": {"sounds": [{"name": "a", "type": "loop"}]}}"#),
    ("no-name", r#"{"bad": {"sounds": [{"volume": 1.0}]}}"#),
    ("not-an-object", r#"{"bad": ["a"]}"#),
    ("sounds-not-array", r#"{"bad": {"sounds": "a"}}"#),
    ("subtitle-not-string", r#"{"bad": {"subtitle": 3, "sounds": ["a"]}}"#),
    ("bad-bool", r#"{"bad": {"replace": 1, "sounds": ["a"]}}"#),
];

const LATE: &[(&str, &str, &[Pick])] = &[
    ("float-weight", r#"{"w": {"sounds": [{"name": "a", "weight": 2.9}, "b"]}}"#, &[("minecraft:w", 6, 12)]),
    ("big-volume", r#"{"v": {"sounds": [{"name": "a", "volume": 3.5}]}}"#, &[("minecraft:v", 1, 2)]),
];

/// Events of the real `sounds.json` and the seeds each is picked with.
const REAL: &[(&str, &[i64])] = &[
    ("minecraft:block.stone.break", &[1, 2, 3]),
    ("minecraft:entity.pig.ambient", &[4, 5]),
    ("minecraft:ambient.cave", &[6, 7]),
    ("minecraft:entity.cow.step", &[8]),
    ("minecraft:block.note_block.harp", &[9]),
    ("minecraft:ui.button.click", &[10]),
];

fn picks(picks: &[Pick]) -> Vec<Value> {
    picks.iter().map(|(event, seed, count)| json!({"event": event, "seed": seed.to_string(), "count": count})).collect()
}

fn small(label: &str, text: &str, chosen: &[Pick], listed: &[&str]) -> Value {
    let files: Vec<String> = listed.iter().map(|name| format!("sounds/{name}.ogg")).collect();
    json!({"label": label, "json": text, "files": files, "picks": picks(chosen)})
}

pub fn registry() -> Value {
    let mut rows: Vec<Value> = SMALL.iter().map(|(label, text, chosen)| small(label, text, chosen, FILES)).collect();
    let listed = [FILES, &["thing"]].concat();
    let text = r#"{"n": {"sounds": [{"name": "minecraft:a"}, "other:thing"]}}"#;
    rows.push(small("namespaced-names", text, &[("minecraft:n", 9, 6)], &listed));
    rows.extend(INVALID.iter().map(|(label, text)| small(label, text, &[], FILES)));
    rows.extend(LATE.iter().map(|(label, text, chosen)| small(label, text, chosen, FILES)));
    let real: Vec<Value> = REAL
        .iter()
        .flat_map(|(event, seeds)| seeds.iter().map(move |seed| (*event, *seed, 40)))
        .map(|(event, seed, count)| json!({"event": event, "seed": seed.to_string(), "count": count}))
        .collect();
    rows.push(json!({"label": "real", "real": true, "picks": real}));
    crate::case::case("sound/registry", "sound", "registry", json!({ "rows": rows }))
}
