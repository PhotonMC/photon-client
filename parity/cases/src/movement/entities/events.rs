//! The server's packets between ticks, as events at a tick.

use serde_json::{Map, Value, json};

use crate::bits::{f32_bits, f64_bits};

/// A relative move, counted from where the last packet put the entity.
pub fn move_item(tick: i64, xa: i64, ya: i64, za: i64) -> Value {
    json!({"tick": tick, "type": "move", "xa": xa, "ya": ya, "za": za, "on_ground": false})
}

/// A relative move that turns the entity to the packed angles the packet carries.
pub fn move_turn(tick: i64, delta: (i64, i64, i64), turn: (i64, i64)) -> Value {
    json!({
        "tick": tick, "type": "move", "xa": delta.0, "ya": delta.1, "za": delta.2,
        "y_rot_byte": turn.0, "x_rot_byte": turn.1, "on_ground": false,
    })
}

/// A relative move that leaves the entity on the ground.
pub fn move_ground(tick: i64, xa: i64, ya: i64, za: i64) -> Value {
    json!({"tick": tick, "type": "move", "xa": xa, "ya": ya, "za": za, "on_ground": true})
}

/// An absolute position and motion.
pub fn sync(tick: i64, position: (f64, f64, f64), motion: (f64, f64, f64), on_ground: bool) -> Value {
    json!({
        "tick": tick, "type": "sync", "x": f64_bits(position.0), "y": f64_bits(position.1),
        "z": f64_bits(position.2), "motion_x": f64_bits(motion.0), "motion_y": f64_bits(motion.1),
        "motion_z": f64_bits(motion.2), "on_ground": on_ground,
    })
}

/// A motion the server sent.
pub fn motion_event(tick: i64, velocity: (f64, f64, f64)) -> Value {
    json!({
        "tick": tick, "type": "motion", "motion_x": f64_bits(velocity.0), "motion_y": f64_bits(velocity.1),
        "motion_z": f64_bits(velocity.2),
    })
}

/// An entity's motion (`entity_motion`).
pub fn motion(tick: i64, x: f64, y: f64, z: f64) -> Value {
    json!({"tick": tick, "type": "entity_motion", "x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)})
}

/// A move packet that sets only whether the entity is on the ground.
pub fn on_ground(tick: i64, ground: bool) -> Value {
    json!({"tick": tick, "type": "move", "xa": 0, "ya": 0, "za": 0, "on_ground": ground})
}

/// A move packet without the ground flag.
pub fn move_by(tick: i64, xa: i64, ya: i64, za: i64) -> Value {
    json!({"tick": tick, "type": "move", "xa": xa, "ya": ya, "za": za})
}

/// A teleport to a place, at rest.
pub fn sync_to(tick: i64, x: f64, y: f64, z: f64) -> Value {
    sync(tick, (x, y, z), (0.0, 0.0, 0.0), false)
}

/// The swing of an arm (`Animate`).
pub fn animate(tick: i64, action: i64) -> Value {
    json!({"tick": tick, "type": "animate", "action": action})
}

pub fn passengers(tick: i64, vehicle: i64, riders: &[i64]) -> Value {
    json!({"tick": tick, "type": "passengers", "vehicle": vehicle, "riders": riders})
}

pub fn fish_data(tick: i64, fish: &str, value: i64) -> Value {
    json!({"tick": tick, "type": "fish_data", "fish": fish, "value": value})
}

pub fn cube_size(tick: i64, size: i64) -> Value {
    json!({"tick": tick, "type": "cube_size", "size": size})
}

pub fn head_turn(tick: i64, angle: i64) -> Value {
    json!({"tick": tick, "type": "head_rotation", "angle": angle})
}

/// The shulker's data: the fields given, after the tick and type.
pub fn shulker_data(tick: i64, fields: Value) -> Value {
    let mut event = json!({"tick": tick, "type": "shulker_data"});
    crate::case::merge(&mut event, fields);
    event
}

pub fn guardian_random(tail: f64, seed: i64) -> Value {
    json!({"tick": 0, "type": "guardian_random", "tail": f32_bits(tail), "seed": seed})
}

/// A display's data the server sent: floats and vectors in their bits,
/// ints as they are.
pub fn display_data(tick: i64, fields: Value) -> Value {
    let mut event = Map::new();
    event.insert("tick".to_owned(), json!(tick));
    event.insert("type".to_owned(), json!("display_data"));
    if let Value::Object(fields) = fields {
        for (key, value) in fields {
            event.insert(key.clone(), display_field(&key, value));
        }
    }
    Value::Object(event)
}

fn display_field(key: &str, value: Value) -> Value {
    let number = |part: &Value| part.as_f64().unwrap_or(0.0);
    match key {
        "translation" | "scale" | "left_rotation" | "right_rotation" => {
            let parts: Vec<String> = value
                .as_array()
                .map_or_else(Vec::new, |parts| parts.iter().map(|part| f32_bits(number(part))).collect());
            json!(parts)
        }
        "view_range" | "shadow_radius" | "shadow_strength" | "width" | "height" => json!(f32_bits(number(&value))),
        _ => value,
    }
}

/// A step along the track the server sent (the turns in the byte the wire
/// packs them in).
pub fn cart_step(position: (f64, f64, f64), motion: (f64, f64, f64), turn: (i64, i64), weight: f64) -> Value {
    json!({
        "x": f64_bits(position.0), "y": f64_bits(position.1), "z": f64_bits(position.2),
        "mx": f64_bits(motion.0), "my": f64_bits(motion.1), "mz": f64_bits(motion.2),
        "y_rot": turn.0, "x_rot": turn.1, "weight": f32_bits(weight),
    })
}

/// A move of ticks every entity hops by: on the ground at 2, off at 9, on
/// at 21, off at 30 and on at 33.
pub fn hops() -> Vec<Value> {
    vec![on_ground(2, true), on_ground(9, false), on_ground(21, true), on_ground(30, false), on_ground(33, true)]
}
