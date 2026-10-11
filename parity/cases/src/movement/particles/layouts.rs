//! Blocks laid across a plane, each in a state that varies with its place.

use serde_json::{Value, json};

const FACINGS: [&str; 4] = ["north", "south", "west", "east"];

/// Python-style modulo: never negative.
fn modulo(value: i64, by: i64) -> usize {
    usize::try_from(value.rem_euclid(by)).unwrap_or(0)
}

fn facing(x: i64, z: i64) -> &'static str {
    FACINGS[modulo(x * 7 + z, 4)]
}

fn at(x: i64, z: i64, name: &str, properties: Value) -> Value {
    json!({"x": x, "y": 64, "z": z, "block": format!("minecraft:{name}"), "properties": properties})
}

/// A torch of each kind across the plane: standing, on walls facing each
/// way, redstone lit or not.
pub fn torch_at(x: i64, z: i64) -> Value {
    let facing = facing(x, z);
    let kinds = [
        ("torch", json!({})),
        ("soul_torch", json!({})),
        ("copper_torch", json!({})),
        ("wall_torch", json!({"facing": facing})),
        ("soul_wall_torch", json!({"facing": facing})),
        ("copper_wall_torch", json!({"facing": facing})),
        ("redstone_torch", json!({"lit": "true"})),
        ("redstone_torch", json!({"lit": "false"})),
        ("redstone_wall_torch", json!({"facing": facing, "lit": "true"})),
    ];
    let (name, properties) = &kinds[modulo(x * 3 + z * 5, kinds.len() as i64)];
    at(x, z, name, properties.clone())
}

/// Light sources across the plane: campfires, furnaces facing each way,
/// candles, a brewing stand.
pub fn hearth_at(x: i64, z: i64) -> Value {
    let facing = facing(x, z);
    let candles = (1 + modulo(x + z, 4)).to_string();
    let kinds = [
        ("campfire", json!({"lit": "true"})),
        ("soul_campfire", json!({"lit": "true"})),
        ("campfire", json!({"lit": "false"})),
        ("furnace", json!({"lit": "true", "facing": facing})),
        ("blast_furnace", json!({"lit": "true", "facing": facing})),
        ("smoker", json!({"lit": "true", "facing": facing})),
        ("furnace", json!({"lit": "false", "facing": facing})),
        ("brewing_stand", json!({})),
        ("candle", json!({"lit": "true", "candles": candles})),
        ("red_candle", json!({"lit": "true", "candles": "4"})),
        ("candle", json!({"lit": "false", "candles": "2"})),
        ("candle_cake", json!({"lit": "true"})),
        ("blue_candle_cake", json!({"lit": "true"})),
    ];
    let (name, properties) = &kinds[modulo(x * 3 + z * 5, kinds.len() as i64)];
    at(x, z, name, properties.clone())
}

fn flag(on: bool) -> &'static str {
    if on { "true" } else { "false" }
}

/// Blocks of the redstone and water kinds across the plane: levers on each
/// face, lit and unlit ore, repeaters, dust with arms of each kind, wet
/// sponges, bubble columns, ender chests, sculk sensors in each phase.
pub fn redstone_at(x: i64, z: i64) -> Value {
    let facing = facing(x, z);
    let arms = ["up", "side", "none"];
    let arm = |value: i64| arms[modulo(value, 3)];
    let phase = |names: [&'static str; 3], value: i64| names[modulo(value, 3)];
    let face = ["floor", "wall", "ceiling"][modulo(x + z, 3)];
    let kinds = [
        (
            "lever",
            json!({
                "face": face, "facing": facing,
                "powered": flag(modulo(x + 2 * z, 3) != 0),
            }),
        ),
        ("redstone_ore", json!({"lit": "true"})),
        ("deepslate_redstone_ore", json!({"lit": flag(modulo(x + z, 4) != 0)})),
        (
            "repeater",
            json!({
                "delay": (1 + modulo(x + z, 4)).to_string(), "facing": facing, "locked": "false",
                "powered": flag(modulo(x * 3 + z, 5) != 0),
            }),
        ),
        (
            "redstone_wire",
            json!({
                "power": (1 + modulo(x * 5 + z * 3, 15)).to_string(), "north": arm(x + z), "east": arm(x * 2 + z),
                "south": arm(x + z * 2), "west": arm(x * 5 + z),
            }),
        ),
        ("redstone_wire", json!({"power": "0", "north": "side", "east": "none", "south": "none", "west": "none"})),
        ("wet_sponge", json!({})),
        ("bubble_column", json!({"drag": flag(modulo(x + z, 2) != 0)})),
        ("ender_chest", json!({"facing": facing, "waterlogged": "false"})),
        (
            "sculk_sensor",
            json!({
                "sculk_sensor_phase": phase(["active", "inactive", "cooldown"], x + z),
                "power": modulo(x + z, 16).to_string(), "waterlogged": "false",
            }),
        ),
        (
            "calibrated_sculk_sensor",
            json!({
                "facing": facing, "sculk_sensor_phase": phase(["active", "cooldown", "inactive"], x + 2 * z),
                "power": modulo(x * 3 + z, 16).to_string(), "waterlogged": "false",
            }),
        ),
    ];
    let (name, properties) = &kinds[modulo(x * 3 + z * 5, kinds.len() as i64)];
    at(x, z, name, properties.clone())
}

/// A square plane of one block at height `y`.
pub fn plane(y: i64, block: &str, properties: Option<&[(&str, &str)]>, size: i64) -> Vec<Value> {
    let properties: serde_json::Map<String, Value> =
        properties.unwrap_or(&[]).iter().map(|(name, value)| ((*name).to_owned(), json!(value))).collect();
    (-size..=size)
        .flat_map(|x| (-size..=size).map(move |z| (x, z)))
        .map(|(x, z)| json!({"x": x, "y": y, "z": z, "block": block, "properties": properties}))
        .collect()
}
