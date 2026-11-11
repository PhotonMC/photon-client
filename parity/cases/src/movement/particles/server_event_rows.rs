//! What a 26.3 server tells the client instead of the client working it out: level events 2014 to
//! 2020, a particle packet with a randomization, and an explosion that makes no sound.

use serde_json::{Value, json};

use super::engine::*;
use super::events::{add, explode, level_event, level_event_block, with_fields};
use crate::bits::f32_bits;
use crate::movement::world::put;

/// A difference between two blocks as a level event carries it: each axis a byte, its radius added.
fn packed(difference: (i64, i64, i64), radii: (i64, i64, i64)) -> i64 {
    ((difference.0 + radii.0) << 16) | ((difference.1 + radii.1) << 8) | (difference.2 + radii.2)
}

/// The same rows run on both versions: 26.2 knows none of these events, 26.3 draws each.
pub fn rows() -> Vec<Value> {
    let stone = vec![put(0, 63, 0, "minecraft:stone"), put(3, 64, 3, "minecraft:stone")];
    let destroy = vec![
        level_event_block(0, 2014, (3, 64, 3), "minecraft:stone"),
        level_event_block(1, 2014, (-2, 64, 4), "minecraft:glass"),
        level_event_block(1, 2014, (4, 64, -3), "minecraft:air"),
    ];
    let teleports = vec![
        level_event(0, 2015, (2, 64, 2), packed((5, -3, 7), (16, 8, 16))),
        level_event(1, 2015, (-4, 64, 1), packed((-16, 8, 0), (16, 8, 16))),
        level_event(1, 2016, (3, 64, -3), packed((2, 2, -8), (8, 8, 8))),
        level_event(2, 2017, (1, 64, 5), packed((100, -20, 127), (127, 127, 127))),
        level_event(2, 2017, (-1, 64, 5), packed((-127, 0, 0), (127, 127, 127))),
    ];
    let endermen = vec![
        level_event(0, 2018, (2, 64, 2), packed((6, 0, -4), (127, 127, 127))),
        level_event(1, 2018, (-3, 70, 3), packed((-9, -3, 12), (127, 127, 127))),
    ];
    let faces: Vec<Value> = [-3, 0, 1, 2, 3, 4, 5, 9]
        .iter()
        .enumerate()
        .flat_map(|(index, data)| {
            let tick = index as i64 % 3;
            [level_event(tick, 2019, (3, 64, 3), *data), level_event(tick, 2020, (0, 63, 0), *data)]
        })
        .collect();
    let air_face = vec![level_event(0, 2019, (6, 64, 6), 1), level_event(0, 2020, (7, 64, 7), 1)];
    // Crumbs take their picture from the block's model (one blank sprite) and draw from an unseeded
    // random in the reference, so rows with crumbs compare only what they sound.
    let crumbs = |row: Engine| row.set("marker_block", json!("minecraft:barrier")).set("sounds_only", json!(true));
    vec![
        crumbs(engine_row("level-event-destroy-effect", destroy))
            .blocks(stone.clone())
            .ticks(4)
            .seeds(300)
            .describe(12)
            .row(),
        engine_row("level-event-block-teleport", teleports).ticks(4).seeds(600).describe(16).row(),
        engine_row("level-event-enderman-teleport", endermen).ticks(4).seeds(600).describe(16).row(),
        crumbs(engine_row("level-event-broken-from-face", faces))
            .blocks(stone.clone())
            .ticks(4)
            .seeds(600)
            .describe(16)
            .row(),
        crumbs(engine_row("level-event-broken-from-face-air", air_face))
            .blocks(stone)
            .ticks(3)
            .seeds(40)
            .describe(8)
            .row(),
    ]
}

/// A packet that names a randomization and a top speed for each axis.
fn randomized(tick: i64, particle: &str, randomization: i64, count: i64, speeds: (f64, f64, f64)) -> Value {
    json!({
        "tick": tick, "type": "packet", "particle": particle, "override": false, "always_show": false,
        "x": crate::bits::f64_bits(1.5), "y": crate::bits::f64_bits(65.0), "z": crate::bits::f64_bits(-2.5),
        "x_dist": f32_bits(0.5), "y_dist": f32_bits(0.25), "z_dist": f32_bits(1.0),
        "x_speed": f32_bits(speeds.0), "y_speed": f32_bits(speeds.1), "z_speed": f32_bits(speeds.2),
        "count": count, "randomization": randomization,
    })
}

/// Rows only 26.3 reads: a randomization has no place in 26.2's packet, and its explosion has no
/// flag.
pub fn newer_rows() -> Vec<Value> {
    let kinds = vec![("crit", 0.5, 1.0, 1), ("cloud", 1.0, 2.0, 3)];
    let blast = |tick: i64, at| explode(tick, at, 3.0, 5, "explosion", kinds.clone());
    let silent = |event: Value| with_fields(event, json!({"play_sound": false}));
    let speeds = (0.1, 0.2, 0.3);
    vec![
        engine_row(
            "packet-randomization",
            vec![
                randomized(0, "flame", 0, 4, speeds),
                randomized(0, "flame", 1, 4, speeds),
                randomized(1, "flame", 2, 4, speeds),
                randomized(1, "crit", 2, 3, (1.0, 2.0, 3.0)),
                randomized(2, "flame", 0, 0, speeds),
                randomized(2, "flame", 1, 0, speeds),
                randomized(2, "flame", 2, 0, speeds),
            ],
        )
        .ticks(4)
        .seeds(60)
        .describe(12)
        .row(),
        // An explosion with no sound draws none of the random numbers the sound takes; the
        // events after it show the level's random number generator where it stands.
        engine_row(
            "explosion-silent",
            vec![
                silent(blast(0, (4.0, 64.0, 4.0))),
                level_event(1, 2009, (1, 64, 1), 0),
                blast(1, (-4.0, 64.0, 4.0)),
                level_event(2, 2009, (1, 64, 1), 0),
                add(2, "flame", (0.0, 64.0, 0.0)),
            ],
        )
        .ticks(4)
        .seeds(100)
        .describe(12)
        .row(),
    ]
}
