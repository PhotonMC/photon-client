//! The beam of a beacon.

use serde_json::{Value, json};

use super::pose::entity;
use crate::bits::f32_bits;
use crate::line::float_text;

/// The real `BeaconRenderer` for a beam of one stretch and of several, at several times of its
/// clock and scales of its radius.
pub fn beacon_beam() -> Vec<Value> {
    let beam = |label: String, sections: &[(i64, i64)], time: f64, scale: f64| {
        let described: Vec<Value> =
            sections.iter().map(|(color, height)| json!({"color": color, "height": height})).collect();
        json!({"label": label, "sections": described, "animation_time": f32_bits(time), "radius_scale": f32_bits(scale)})
    };
    let white = 0xFFF9_FFFE_u32 as i32 as i64;
    let red = 0xFFB0_2E26_u32 as i32 as i64;
    let blue = 0xFF3C_44AA_u32 as i32 as i64;
    let mut rows = Vec::new();
    for time in [0.0, 0.25, 3.5, 17.75, 39.9] {
        rows.push(beam(format!("one stretch at {}", float_text(time)), &[(white, 5)], time, 1.0));
    }
    rows.push(beam("two stretches".to_owned(), &[(white, 2), (red, 7)], 12.5, 1.0));
    rows.push(beam("three stretches".to_owned(), &[(white, 1), (red, 3), (blue, 40)], 5.0, 1.0));
    for scale in [1.0, 1.7, 4.25] {
        rows.push(beam(format!("scaled {}", float_text(scale)), &[(red, 9), (blue, 3)], 20.25, scale));
    }
    vec![entity("beacon-beam", "beacon_beam", rows)]
}

/// The real `TheEndGatewayRenderer` for a beam of several heights, colours, scales and times of
/// its clock.
pub fn gateway_beam() -> Vec<Value> {
    let magenta = 0xFFC7_4EBD_u32 as i32 as i64;
    let purple = 0xFF89_32B8_u32 as i32 as i64;
    let beam = |label: &str, (scale, time): (f64, f64), height: i64, color: i64| json!({"label": label, "scale": f32_bits(scale), "animation_time": f32_bits(time), "height": height, "color": color});
    let rows = vec![
        beam("spawning, just begun", (0.0157, 0.0), 4, magenta),
        beam("spawning, high", (0.9511, 17.25), 301, magenta),
        beam("spawning, full", (1.0, 39.5), 319, magenta),
        beam("cooling, rising", (0.5, 3.75), 25, purple),
        beam("cooling, nearly done", (0.1564, 28.0), 7, purple),
        beam("cooling, tall", (0.9877, 12.5), 49, purple),
    ];
    vec![entity("gateway-beam", "gateway_beam", rows)]
}

/// The real `BrushableBlockRenderer` for an item against each face, at each stage of the dust.
pub fn brushable_pose() -> Vec<Value> {
    let mut rows = Vec::new();
    for direction in ["down", "up", "north", "south", "west", "east"] {
        for dusted in 0..4 {
            rows.push(
                json!({"label": format!("{direction} dusted {dusted}"), "direction": direction, "dusted": dusted}),
            );
        }
    }
    vec![entity("brushable-pose", "brushable_pose", rows)]
}

/// The real `ShelfRenderer` for items of several heights on a shelf facing each way.
pub fn shelf_pose() -> Vec<Value> {
    let mut rows = Vec::new();
    for facing in ["north", "south", "west", "east"] {
        for align in [false, true] {
            rows.push(json!({
                "label": format!("{facing} aligned {align}"), "facing": facing, "align": align,
                "min_y": f32_bits(0.0), "max_y": f32_bits(1.0), "gaps": [false, false, false],
            }));
        }
    }
    rows.push(json!({
        "label": "a small item and gaps", "facing": "south", "align": false,
        "min_y": f32_bits(0.125), "max_y": f32_bits(0.6875), "gaps": [false, true, false],
    }));
    rows.push(json!({
        "label": "a tall item, aligned", "facing": "west", "align": true,
        "min_y": f32_bits(-0.25), "max_y": f32_bits(1.5), "gaps": [true, false, true],
    }));
    vec![entity("shelf-pose", "shelf_pose", rows)]
}

/// The real `ConduitRenderer`: what its extraction makes of a conduit's ticks and turns, and the
/// parts it draws for a waking one in each phase of the wind and for a sleeping one.
pub fn conduit_pose() -> Vec<Value> {
    let mut rows = Vec::new();
    for (active, hunting) in [(false, false), (true, false), (true, true)] {
        for (tick_count, rotation, partial) in
            [(0, 0, 0.0), (65, 12, 0.5), (66, 40, 0.25), (140, 7, 0.75), (133, 100, 0.0)]
        {
            rows.push(json!({
                "kind": "state", "label": format!("state {active} {hunting} at {tick_count}/{rotation}"),
                "active": active, "hunting": hunting, "tick_count": tick_count, "rotation": rotation,
                "partial": f32_bits(partial),
            }));
        }
    }
    let pose = |label: String,
                active: bool,
                hunting: bool,
                (rotation, time, phase): (f64, f64, i64),
                (yaw, pitch): (f64, f64)| {
        json!({
            "kind": "pose", "label": label, "active": active, "hunting": hunting,
            "active_rotation": f32_bits(rotation), "anim_time": f32_bits(time), "phase": phase,
            "yaw": f32_bits(yaw), "pitch": f32_bits(pitch),
        })
    };
    rows.push(pose("asleep".to_owned(), false, false, (-0.45, 3.5, 0), (0.0, 0.0)));
    rows.push(pose("asleep turned".to_owned(), false, false, (-31.2, 70.25, 1), (10.0, 5.0)));
    for phase in 0..3 {
        rows.push(pose(
            format!("awake phase {phase}"),
            true,
            false,
            (-3.4, 12.5 + phase as f64 * 66.0, phase),
            (35.0, -20.0),
        ));
        rows.push(pose(format!("hunting phase {phase}"), true, true, (-12.75, 140.25, phase), (-120.0, 45.0)));
    }
    vec![entity("conduit-pose", "conduit_pose", rows)]
}

/// The real `CampfireRenderer` for food on a campfire facing each way, in every slot and with gaps.
pub fn campfire_pose() -> Vec<Value> {
    let mut rows = Vec::new();
    for facing in ["north", "south", "west", "east"] {
        rows.push(json!({"label": format!("{facing} full"), "facing": facing, "gaps": [false, false, false, false]}));
        rows.push(
            json!({"label": format!("{facing} with gaps"), "facing": facing, "gaps": [true, false, true, false]}),
        );
    }
    rows.push(json!({"label": "one in the last slot", "facing": "west", "gaps": [true, true, true, false]}));
    vec![entity("campfire-pose", "campfire_pose", rows)]
}

/// The real `VaultRenderer` for the item a vault shows: stacks of several sizes, thin and thick
/// models, and spins between ticks, the wrap included.
pub fn vault_pose() -> Vec<Value> {
    let row = |label: String,
               item: &str,
               count: i64,
               (min_y, max_y, depth): (f64, f64, f64),
               (previous, current, partial): (i64, i64, f64)| {
        json!({
            "label": label, "item": item, "count": count, "min_y": f32_bits(min_y), "max_y": f32_bits(max_y),
            "depth": f32_bits(depth), "previous": previous, "current": current, "partial": f32_bits(partial),
        })
    };
    let mut rows = Vec::new();
    for count in [1, 2, 16, 17, 33, 49, 64] {
        rows.push(row(format!("thin item, {count}"), "minecraft:diamond", count, (0.0, 1.0, 0.03125), (50, 60, 0.25)));
        rows.push(row(format!("thick block, {count}"), "minecraft:stone", count, (0.0, 1.0, 0.5), (50, 60, 0.75)));
    }
    rows.push(row("wrap".to_owned(), "minecraft:emerald", 3, (0.125, 0.875, 0.0625), (170, -180, 0.5)));
    rows.push(row("wrap back".to_owned(), "minecraft:emerald", 3, (0.125, 0.875, 0.0625), (-10, 0, 0.9)));
    rows.push(row("at rest".to_owned(), "minecraft:apple", 5, (0.0, 1.0, 0.125), (0, 0, 0.0)));
    vec![entity("vault-pose", "vault_pose", rows)]
}
