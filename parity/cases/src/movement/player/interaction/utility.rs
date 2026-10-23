//! The use key with an item whose `use` the client mostly leaves to the server: what the client
//! itself answers (swing, using state, count) is compared.

use serde_json::Value;

use super::placing::uses;
use super::{aim_at, using};
use crate::movement::world::{put, put_with};

const ITEMS: [&str; 6] = [
    "minecraft:ender_eye",
    "minecraft:oak_boat",
    "minecraft:fishing_rod",
    "minecraft:map",
    "minecraft:pig_spawn_egg",
    "minecraft:trident",
];

pub fn rows() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3), (0, 64, 1)];
    let run = |aim: Option<&Value>| [uses(1, aim, &around).clicks(1).go(), uses(3, aim, &around).go()].concat();
    let mut rows = Vec::new();
    for item in ITEMS {
        let name = item.trim_start_matches("minecraft:");
        let held = |label: String, world: Vec<Value>, x_rot: f64| {
            using(&label, world, run(Some(&top_of))).main_hand(item).count(2).x_rot(x_rot)
        };
        rows.push(held(format!("use-{name}-floor"), vec![], 39.0).row());
        rows.push(held(format!("use-{name}-water"), vec![put(0, 64, 2, "minecraft:water")], 39.0).row());
        rows.push(using(&format!("use-{name}-sky"), vec![], run(None)).main_hand(item).count(2).x_rot(-60.0).row());
    }
    // A main hand that passes leaves the off hand its turn.
    for (item, x_rot) in [("minecraft:ender_eye", 39.0), ("minecraft:oak_boat", -60.0), ("minecraft:map", 39.0)] {
        let name = item.trim_start_matches("minecraft:");
        rows.push(
            using(&format!("use-{name}-off-hand"), vec![], run(Some(&top_of)))
                .main_hand("minecraft:stick")
                .count(2)
                .off_hand(item)
                .x_rot(x_rot)
                .row(),
        );
    }
    rows.push(
        using(
            "use-ender_eye-portal-frame",
            vec![put_with(0, 64, 2, "minecraft:end_portal_frame", &[("eye", "false"), ("facing", "north")])],
            run(Some(&top_of)),
        )
        .main_hand("minecraft:ender_eye")
        .count(2)
        .x_rot(39.0)
        .row(),
    );
    // The target is the frame itself: without an eye it takes the item (a swing, no `use`), with
    // one `useOn` passes and the look passes too.
    let frame_top = aim_at((0, 64, 2), "up", (0.5, 64.8125, 2.5));
    for eye in ["false", "true"] {
        rows.push(
            using(
                &format!("use-ender_eye-on-frame-eye-{eye}"),
                vec![put_with(0, 64, 2, "minecraft:end_portal_frame", &[("eye", eye), ("facing", "north")])],
                run(Some(&frame_top)),
            )
            .main_hand("minecraft:ender_eye")
            .count(2)
            .x_rot(39.0)
            .row(),
        );
    }
    // The crosshair ignores the water, but the egg's own look finds the source.
    rows.push(
        using("use-pig_spawn_egg-over-water-no-target", vec![put(0, 64, 2, "minecraft:water")], run(None))
            .main_hand("minecraft:pig_spawn_egg")
            .count(2)
            .x_rot(39.0)
            .row(),
    );
    rows
}
