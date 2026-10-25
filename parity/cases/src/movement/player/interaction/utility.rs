//! The use key with an item whose `use` the client mostly leaves to the server: what the client
//! itself answers (swing, using state, count) is compared.

use serde_json::Value;

use super::placing::uses;
use super::{aim_at, using};
use crate::components::{DAMAGE, RECIPES, Tag, UNBREAKABLE, nbt, patch, vi};
use crate::movement::player::scene::keys;
use crate::movement::world::{put, put_with};

const ITEMS: [&str; 13] = [
    "minecraft:ender_eye",
    "minecraft:oak_boat",
    "minecraft:fishing_rod",
    "minecraft:map",
    "minecraft:pig_spawn_egg",
    "minecraft:trident",
    "minecraft:goat_horn",
    "minecraft:carrot_on_a_stick",
    "minecraft:warped_fungus_on_a_stick",
    "minecraft:firework_rocket",
    "minecraft:knowledge_book",
    "minecraft:written_book",
    "minecraft:bundle",
];

/// One click, then three quiet ticks, watching the blocks around the target.
fn run(aim: Option<&Value>) -> Vec<Value> {
    let around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3), (0, 64, 1)];
    [uses(1, aim, &around).clicks(1).go(), uses(3, aim, &around).go()].concat()
}

pub fn rows() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let mut rows = special_rows();
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
    for (item, x_rot) in [
        ("minecraft:ender_eye", 39.0),
        ("minecraft:oak_boat", -60.0),
        ("minecraft:map", 39.0),
        ("minecraft:goat_horn", 39.0),
        ("minecraft:carrot_on_a_stick", 39.0),
        ("minecraft:firework_rocket", 39.0),
        ("minecraft:knowledge_book", 39.0),
    ] {
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
    rows
}

/// A trident one point from breaking fails the use; an unbreakable one starts it, and one with
/// room left starts it too.
fn trident_rows() -> Vec<Value> {
    let ticks = [uses(1, None, &[]).clicks(1).go(), uses(3, None, &[]).go()].concat();
    let row = |label: &str, added: &[(i64, String)]| {
        using(label, vec![], ticks.clone()).main_hand("minecraft:trident").count(1).patch(patch(added)).row()
    };
    vec![
        row("use-trident-last-point", &[(DAMAGE, vi(249))]),
        row("use-trident-room-left", &[(DAMAGE, vi(248))]),
        row("use-trident-unbreakable-last-point", &[(DAMAGE, vi(249)), (UNBREAKABLE, String::new())]),
    ]
}

/// A book with a recipe is a success (a swing) and is consumed on the client too; creative keeps
/// the book.
fn recipe_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let ticks = [uses(1, None, &[]).clicks(1).inventory().go(), uses(2, None, &[]).inventory().go()].concat();
    for (label, mode) in [("use-knowledge_book-recipe", "survival"), ("use-knowledge_book-recipe-creative", "creative")]
    {
        rows.push(
            using(label, vec![], ticks.clone())
                .main_hand("minecraft:knowledge_book")
                .count(1)
                .patch(patch(&[(RECIPES, nbt(&Tag::List(vec![Tag::from("minecraft:oak_planks")])))]))
                .game_mode(mode)
                .row(),
        );
    }
    rows
}

/// The rows that need a target, a world or a clock of their own.
fn special_rows() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let mut rows = Vec::new();
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
    // A rocket in the hand of a glider is used (a swing); the server spawns it.
    let rocket =
        [keys(1, &["jump"]), keys(3, &[]), uses(1, None, &[]).clicks(1).go(), uses(3, None, &[]).go()].concat();
    rows.push(
        using("use-firework_rocket-gliding", vec![], rocket)
            .main_hand("minecraft:firework_rocket")
            .count(2)
            .chest("minecraft:elytra")
            .start(0.5, 100.0, 0.5)
            .x_rot(20.0)
            .row(),
    );
    // The cooldown of a call holds a second click, in survival and in creative alike.
    let click = || uses(1, None, &[]).clicks(1).go();
    for (label, mode) in [("use-goat_horn-twice", "survival"), ("use-goat_horn-twice-creative", "creative")] {
        let ticks = [click(), uses(6, None, &[]).go(), click(), uses(2, None, &[]).go()].concat();
        rows.push(using(label, vec![], ticks).main_hand("minecraft:goat_horn").count(1).game_mode(mode).row());
    }
    rows.extend([recipe_rows(), trident_rows()].concat());
    // `ItemStack.use` begins a `use_cooldown` when the use duration is 0 and the result is a
    // success: the eye starts a use all the same; a stick that passes gets none.
    for (item, x_rot) in [("minecraft:ender_eye", -60.0), ("minecraft:ender_eye", 39.0), ("minecraft:stick", 39.0)] {
        let name = item.trim_start_matches("minecraft:");
        let label = format!("use-{name}-with-cooldown-{}", if x_rot < 0.0 { "sky" } else { "floor" });
        let aim = (x_rot > 0.0).then_some(&top_of);
        rows.push(using(&label, vec![], run(aim)).main_hand(item).count(2).use_cooldown(2.0).x_rot(x_rot).row());
    }
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
