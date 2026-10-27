//! The use key with an item whose `use` the client mostly leaves to the server: what the client
//! itself answers (swing, using state, count) is compared.

use serde_json::Value;

use super::placing::uses;
use super::{aim_at, using};
use crate::components::{
    BINDING_CURSE, DAMAGE, ENCHANTMENTS, INSTRUMENT, PROTECTION, RECIPES, RIPTIDE, Tag, UNBREAKABLE, enchantments, nbt,
    patch, patch_of, vi,
};
use crate::movement::player::scene::keys;
use crate::movement::world::{put, put_with};

const ITEMS: [&str; 15] = [
    "minecraft:iron_boots",
    "minecraft:elytra",
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

/// The cooldown of a call holds a second click, in survival and in creative alike, and in the off
/// hand all the same.
fn horn_cooldown_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let click = || uses(1, None, &[]).clicks(1).go();
    for (label, mode) in [("use-goat_horn-twice", "survival"), ("use-goat_horn-twice-creative", "creative")] {
        let ticks = [click(), uses(6, None, &[]).go(), click(), uses(2, None, &[]).go()].concat();
        rows.push(using(label, vec![], ticks).main_hand("minecraft:goat_horn").count(1).game_mode(mode).row());
    }
    let ticks = [click(), uses(6, None, &[]).go(), click(), uses(2, None, &[]).go()].concat();
    rows.push(
        using("use-goat_horn-off-hand-twice", vec![], ticks)
            .main_hand("minecraft:stick")
            .off_hand("minecraft:goat_horn")
            .count(1)
            .row(),
    );
    rows
}

/// A trident one point from breaking fails the use; an unbreakable one starts it, and one with
/// room left starts it too.
fn trident_rows() -> Vec<Value> {
    let ticks = [uses(1, None, &[]).clicks(1).go(), uses(3, None, &[]).go()].concat();
    let row = |label: &str, added: &[(i64, String)]| {
        using(label, vec![], ticks.clone()).main_hand("minecraft:trident").count(1).patch(patch(added)).row()
    };
    // Riptide fails on dry land and starts in water; the clicks come after the water has been felt.
    let spin = patch(&[(ENCHANTMENTS, enchantments(&[(RIPTIDE, 1)]))]);
    let wet = [uses(3, None, &[]).go(), ticks.clone()].concat();
    let riptide = |label: &str, world: Vec<Value>, ticks: Vec<Value>| {
        using(label, world, ticks).main_hand("minecraft:trident").count(1).patch(spin.clone()).row()
    };
    let release =
        [uses(1, None, &[]).clicks(1).down().go(), uses(13, None, &[]).down().go(), uses(12, None, &[]).go()].concat();
    let wet_release = [uses(3, None, &[]).go(), release.clone()].concat();
    let rain_row = |label: &str, world: Vec<Value>, ticks: Vec<Value>, rain: f64| {
        using(label, world, ticks).main_hand("minecraft:trident").count(1).patch(spin.clone()).raining(rain).row()
    };
    vec![
        riptide("use-trident-riptide-dry", vec![], ticks.clone()),
        riptide("use-trident-riptide-water", vec![put(0, 64, 0, "minecraft:water")], wet.clone()),
        // `isInWaterOrRain`: the rain counts where it reaches the player, and only above a fifth.
        rain_row("use-trident-riptide-rain", vec![], ticks.clone(), 1.0),
        rain_row("use-trident-riptide-rain-under-roof", vec![put(0, 67, 0, "minecraft:stone")], ticks.clone(), 1.0),
        rain_row("use-trident-riptide-light-rain", vec![], ticks.clone(), 0.15),
        // Held past ten ticks and let go: the riptide throws the player along the look.
        rain_row("use-trident-riptide-rain-release", vec![], release.clone(), 1.0),
        riptide("use-trident-riptide-water-release", vec![put(0, 64, 0, "minecraft:water")], wet_release),
        row("use-trident-last-point", &[(DAMAGE, vi(249))]),
        row("use-trident-room-left", &[(DAMAGE, vi(248))]),
        row("use-trident-unbreakable-last-point", &[(DAMAGE, vi(249)), (UNBREAKABLE, String::new())]),
    ]
}

/// A horn names its call by the `instrument` component: the item's own is the ponder call, a patched
/// one is another call (sound and all), and a removed one leaves nothing to play.
fn horn_rows() -> Vec<Value> {
    let ticks = [uses(1, None, &[]).clicks(1).go(), uses(3, None, &[]).go()].concat();
    let row = |label: &str, hex: String| {
        using(label, vec![], ticks.clone()).main_hand("minecraft:goat_horn").count(1).patch(hex).row()
    };
    // The sing call is the seventh entry of the registry, a holder written as its id plus one.
    vec![
        row("use-goat_horn-sing", patch(&[(INSTRUMENT, vi(7))])),
        row("use-goat_horn-no-instrument", patch_of(&[], &[INSTRUMENT])),
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
    // A patched empty list is read by its count: the book is consumed and the use fails.
    rows.push(
        using("use-knowledge_book-empty-list", vec![], ticks)
            .main_hand("minecraft:knowledge_book")
            .count(1)
            .patch(patch(&[(RECIPES, nbt(&Tag::List(vec![])))]))
            .row(),
    );
    rows
}

/// The rows that need a target, a world or a clock of their own.
/// A wearable item used while something is worn in its slot (`swapWithEquipmentSlot`).
fn swapping_rows() -> Vec<Value> {
    let sky = |label: &str| using(label, vec![], run(None)).x_rot(-60.0);
    let plate = "minecraft:iron_chestplate";
    let worn = |id: i64| patch(&[(ENCHANTMENTS, enchantments(&[(id, 4)]))]);
    vec![
        sky("use-chestplate-swap-protection")
            .main_hand(plate)
            .chest("minecraft:diamond_chestplate")
            .chest_patch(worn(PROTECTION))
            .row(),
        sky("use-chestplate-swap-binding")
            .main_hand(plate)
            .chest("minecraft:diamond_chestplate")
            .chest_patch(worn(BINDING_CURSE))
            .row(),
        sky("use-chestplate-swap-binding-creative")
            .main_hand(plate)
            .chest("minecraft:diamond_chestplate")
            .chest_patch(worn(BINDING_CURSE))
            .game_mode("creative")
            .row(),
        sky("use-chestplate-swap-single").main_hand(plate).chest("minecraft:diamond_chestplate").row(),
        sky("use-chestplate-swap-stack").main_hand(plate).count(3).chest("minecraft:diamond_chestplate").row(),
        sky("use-chestplate-swap-empty-slot").main_hand(plate).row(),
        sky("use-chestplate-swap-same").main_hand(plate).chest(plate).row(),
        sky("use-chestplate-swap-single-creative")
            .main_hand(plate)
            .chest("minecraft:diamond_chestplate")
            .game_mode("creative")
            .row(),
        sky("use-chestplate-swap-stack-creative")
            .main_hand(plate)
            .count(3)
            .chest("minecraft:diamond_chestplate")
            .game_mode("creative")
            .row(),
        sky("use-chestplate-swap-empty-creative").main_hand(plate).game_mode("creative").row(),
    ]
}

/// A boat aimed at the side of a block, where its box would meet the block.
fn boat_rows() -> Vec<Value> {
    let side = aim_at((0, 65, 2), "north", (0.5, 65.62, 2.0));
    let beside = |label: &str, block: &str| {
        using(label, vec![put(0, 65, 2, block)], run(Some(&side))).main_hand("minecraft:oak_boat").count(2).x_rot(0.0)
    };
    vec![
        beside("use-oak_boat-into-wall", "minecraft:stone").row(),
        beside("use-oak_boat-into-slab", "minecraft:stone_slab").row(),
        beside("use-oak_boat-into-glass-pane", "minecraft:glass_pane").row(),
        beside("use-oak_boat-into-water", "minecraft:water").row(),
    ]
}

fn special_rows() -> Vec<Value> {
    let top_of = aim_at((0, 63, 2), "up", (0.5, 64.0, 2.5));
    let mut rows = [swapping_rows(), boat_rows()].concat();
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
    rows.extend([recipe_rows(), trident_rows(), horn_rows(), horn_cooldown_rows()].concat());
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
