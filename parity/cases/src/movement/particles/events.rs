//! The events a particle engine row feeds the level, and the blocks it
//! animates.

use serde_json::{Map, Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::merge;

type Vec3 = (f64, f64, f64);
type Block = (i64, i64, i64);

/// The row with more fields set.
pub fn with_fields(mut event: Value, fields: Value) -> Value {
    merge(&mut event, fields);
    event
}

fn particle_event(tick: i64, kind: &str, particle: &str, at: Vec3, flags: (bool, bool)) -> Map<String, Value> {
    let Value::Object(map) = json!({
        "tick": tick, "type": kind, "particle": particle, "override": flags.0, "always_show": flags.1,
        "x": f64_bits(at.0), "y": f64_bits(at.1), "z": f64_bits(at.2),
    }) else {
        unreachable!("an object literal")
    };
    map
}

fn with_push(mut map: Map<String, Value>, push: Vec3) -> Value {
    map.insert("xa".to_owned(), json!(f64_bits(push.0)));
    map.insert("ya".to_owned(), json!(f64_bits(push.1)));
    map.insert("za".to_owned(), json!(f64_bits(push.2)));
    Value::Object(map)
}

/// A particle added to the level.
pub fn add(tick: i64, particle: &str, at: Vec3) -> Value {
    add_pushed(tick, particle, at, (0.0, 0.0, 0.0))
}

pub fn add_pushed(tick: i64, particle: &str, at: Vec3, push: Vec3) -> Value {
    with_push(particle_event(tick, "add", particle, at, (false, false)), push)
}

/// A particle added with the packet's override and always-show flags.
pub fn add_flagged(tick: i64, particle: &str, at: Vec3, over: bool, always_show: bool) -> Value {
    with_push(particle_event(tick, "add", particle, at, (over, always_show)), (0.0, 0.0, 0.0))
}

fn packet_with(tick: i64, particle: &str, at: Vec3, shape: (Vec3, f64, i64), over: bool) -> Value {
    let (spread, speed, count) = shape;
    let mut map = particle_event(tick, "packet", particle, at, (over, false));
    map.insert("x_dist".to_owned(), json!(f32_bits(spread.0)));
    map.insert("y_dist".to_owned(), json!(f32_bits(spread.1)));
    map.insert("z_dist".to_owned(), json!(f32_bits(spread.2)));
    map.insert("max_speed".to_owned(), json!(f32_bits(speed)));
    map.insert("count".to_owned(), json!(count));
    Value::Object(map)
}

/// A particle packet: a count, spread and speed.
pub fn packet(tick: i64, particle: &str, at: Vec3, spread: Vec3, speed: f64, count: i64) -> Value {
    packet_with(tick, particle, at, (spread, speed, count), false)
}

pub fn packet_override(tick: i64, particle: &str, at: Vec3, spread: Vec3, speed: f64, count: i64) -> Value {
    packet_with(tick, particle, at, (spread, speed, count), true)
}

/// An explosion packet: its radius, its block count and the particles it
/// scatters (name, scaling, speed, weight).
pub fn explode(
    tick: i64,
    at: Vec3,
    radius: f64,
    block_count: i64,
    particle: &str,
    kinds: Vec<(&str, f64, f64, i64)>,
) -> Value {
    let kinds: Vec<Value> = kinds
        .iter()
        .map(|(name, scaling, speed, weight)| {
            json!({"particle": name, "scaling": f32_bits(*scaling), "speed": f32_bits(*speed), "weight": weight})
        })
        .collect();
    json!({
        "tick": tick, "type": "explode", "x": f64_bits(at.0), "y": f64_bits(at.1), "z": f64_bits(at.2),
        "radius": f32_bits(radius), "block_count": block_count, "particle": particle, "kinds": kinds,
    })
}

/// A firework explosion's shape and colours.
pub fn explosion(shape: &str, colors: Vec<i64>, fades: Vec<i64>, twinkle: bool) -> Value {
    json!({"shape": shape, "colors": colors, "fades": fades, "trail": false, "twinkle": twinkle})
}

pub fn fireworks(tick: i64, at: Vec3, explosions: Vec<Value>, motion: Vec3) -> Value {
    json!({
        "tick": tick, "type": "fireworks", "x": f64_bits(at.0), "y": f64_bits(at.1), "z": f64_bits(at.2),
        "xd": f64_bits(motion.0), "yd": f64_bits(motion.1), "zd": f64_bits(motion.2), "explosions": explosions,
    })
}

pub fn level_event(tick: i64, event: i64, pos: Block, data: i64) -> Value {
    json!({"tick": tick, "type": "level_event", "event": event, "x": pos.0, "y": pos.1, "z": pos.2, "data": data})
}

pub fn global_level_event(tick: i64, event: i64, pos: Block, data: i64) -> Value {
    json!({"tick": tick, "type": "global_level_event", "event": event, "x": pos.0, "y": pos.1, "z": pos.2, "data": data})
}

pub fn block_event(tick: i64, pos: Block, b0: i64, b1: i64) -> Value {
    json!({"tick": tick, "type": "block_event", "x": pos.0, "y": pos.1, "z": pos.2, "b0": b0, "b1": b1})
}

/// A redstone ore touched (`BlockState.attack`): its dust, and it lights up.
pub fn ore_touch(tick: i64, pos: Block) -> Value {
    json!({"tick": tick, "type": "ore_touch", "x": pos.0, "y": pos.1, "z": pos.2})
}

/// The server's clock told to the level.
pub fn game_time(tick: i64, time: i64) -> Value {
    json!({"tick": tick, "type": "game_time", "time": time})
}

pub fn animate_tick(tick: i64, center: Block, count: i64, seed: i64) -> Value {
    json!({"tick": tick, "type": "animate_tick", "x": center.0, "y": center.1, "z": center.2, "count": count, "seed": seed})
}

/// A note block of an instrument and a note, unpowered.
pub fn note_block(pos: Block, instrument: &str, note: i64) -> Value {
    json!({
        "x": pos.0, "y": pos.1, "z": pos.2, "block": "minecraft:note_block",
        "properties": {"instrument": instrument, "note": note.to_string(), "powered": "false"},
    })
}

/// The note blocks of the note-blocks row: a place, an instrument and a note.
pub fn note_blocks() -> Vec<(Block, &'static str, i64)> {
    vec![
        ((0, 64, 0), "harp", 0),
        ((1, 64, 0), "harp", 12),
        ((2, 64, 0), "harp", 24),
        ((3, 64, 0), "basedrum", 5),
        ((4, 64, 0), "trumpet_oxidized", 19),
        ((5, 64, 0), "zombie", 7),
        ((6, 64, 0), "dragon", 3),
        ((7, 64, 0), "custom_head", 9),
    ]
}
