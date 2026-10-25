//! The creative and spectator flights: what the keys do while flying, the double tap that turns
//! flight on and off, and what stops it.

use serde_json::Value;

use super::scene::{Scene, keys};
use crate::movement::world::{cat, floor, put};

fn ceiling() -> Vec<Value> {
    (-2..3).flat_map(|x| (-2..3).map(move |z| put(x, 67, z, "minecraft:stone"))).collect()
}

/// A tap, a gap of `gap` ticks and a second tap: the double tap that toggles flight when the gap
/// is short enough, then `after` ticks of the keys in `held`.
fn double_tap(gap: usize, after: Vec<Value>) -> Vec<Value> {
    [keys(1, &["jump"]), keys(gap, &[]), keys(1, &["jump"]), after].concat()
}

/// A creative player takes off with the double tap and does `then`.
fn flying(label: &'static str, blocks: Vec<Value>, then: Vec<Value>) -> Scene {
    Scene::new(label, blocks, double_tap(1, then)).game_mode("creative")
}

/// The keys of a flight, the taps that start and end it, and the window between the taps.
pub fn rows() -> Vec<Value> {
    let mut rows = steady_rows();
    rows.extend(toggle_rows());
    rows.extend(speed_rows());
    rows
}

fn steady_rows() -> Vec<Value> {
    let level = |label, held: &[&str], count| flying(label, floor(), keys(count, held)).row();
    vec![
        level("fly-hover", &[], 10),
        level("fly-up", &["jump"], 15),
        level("fly-forward", &["forward"], 12),
        level("fly-backward", &["backward"], 12),
        level("fly-strafe-sprint", &["left", "sprint"], 12),
        level("fly-up-and-forward", &["jump", "forward"], 12),
        flying("fly-down", floor(), [keys(12, &["jump"]), keys(8, &["sneak"])].concat()).row(),
        flying("fly-sneak-forward", floor(), [keys(12, &["jump"]), keys(8, &["sneak", "forward"])].concat()).row(),
        flying("fly-into-ceiling", cat![floor(), ceiling()], keys(20, &["jump"])).row(),
        flying("fly-sprint-into-wall", cat![floor(), super::scene::wall()], keys(20, &["forward", "sprint"])).row(),
        Scene::new("spectator-sneak-sink", floor(), keys(10, &["sneak"])).game_mode("spectator").row(),
        Scene::new("spectator-sprint-forward", floor(), keys(10, &["forward", "sprint"])).game_mode("spectator").row(),
    ]
}

fn toggle_rows() -> Vec<Value> {
    let gap = |label: &'static str, ticks| {
        Scene::new(label, floor(), double_tap(ticks, keys(10, &["jump"]))).game_mode("creative").row()
    };
    vec![
        gap("fly-tap-gap-0", 0),
        gap("fly-tap-gap-5", 5),
        gap("fly-tap-gap-6", 6),
        gap("fly-tap-gap-7", 7),
        gap("fly-tap-gap-8", 8),
        flying(
            "fly-toggle-off",
            floor(),
            [keys(6, &["jump"]), keys(1, &[]), keys(1, &["jump"]), keys(1, &[]), keys(1, &["jump"]), keys(20, &[])]
                .concat(),
        )
        .row(),
        Scene::new("survival-double-tap", floor(), double_tap(1, keys(10, &["jump"]))).row(),
        Scene::new("adventure-double-tap", floor(), double_tap(1, keys(10, &["jump"]))).game_mode("adventure").row(),
        flying("fly-then-sneak-to-ground", floor(), [keys(4, &["jump"]), keys(30, &["sneak"])].concat()).row(),
    ]
}

/// The flying speed a server sets: the vertical push, the sprint doubling and the air control all
/// read it.
fn speed_rows() -> Vec<Value> {
    let take_off =
        |label: &'static str, speed: f64, then: Vec<Value>| flying(label, floor(), then).flying_speed(speed).row();
    vec![
        take_off("fly-speed-0.1-up", 0.1, keys(12, &["jump"])),
        take_off("fly-speed-0.07-sprint", 0.07, keys(12, &["forward", "sprint"])),
        take_off("fly-speed-0.07-strafe", 0.07, keys(12, &["left", "jump"])),
        take_off("fly-speed-0-hover", 0.0, keys(8, &["forward", "jump"])),
        Scene::new("spectator-speed-0.2-forward", floor(), keys(10, &["forward"]))
            .game_mode("spectator")
            .flying_speed(0.2)
            .row(),
        Scene::new("spectator-speed-0.03-sprint", floor(), keys(10, &["forward", "sprint", "jump"]))
            .game_mode("spectator")
            .flying_speed(0.03)
            .row(),
    ]
}
