//! The statistics screen.

use serde_json::{Value, json};

use super::gui;

/// `StatFormatter`: the default's grouping, the tenths, and the distance and the time over their
/// unit thresholds, with values on the half and on the ties of two decimals.
pub fn stat_format() -> Value {
    let rows = vec![
        json!({"label": "default", "formatter": "default", "values": [
            0, 1, 12, 999, 1000, 1001, 12345, 999_999, 1_000_000, 123_456_789, 2_147_483_647, -1, -999, -1000, -2_147_483_648_i64]}),
        json!({"label": "divide-by-ten", "formatter": "divide_by_ten", "values": [
            0, 1, 2, 3, 5, 9, 10, 15, 25, 35, 45, 100, 125, 1234, 12345, 2_147_483_647, -1, -25]}),
        json!({"label": "distance", "formatter": "distance", "values": [
            0, 1, 49, 50, 51, 99, 100, 101, 149, 150, 151, 999, 1000, 12345, 49999, 50000, 50001, 50049, 50050, 99999,
            100_000, 100_001, 123_449, 123_450, 123_451, 1_234_500, 1_234_501, 2_147_483_647, -1, -100_000]}),
        json!({"label": "time", "formatter": "time", "values": [
            0, 1, 2, 3, 19, 20, 21, 599, 600, 601, 1199, 1200, 1201, 1220, 3000, 36_000, 36_001, 72_000, 72_001, 100_000,
            1_728_000, 1_728_001, 1_728_020, 2_000_000, 31_536_000, 315_360_000, 631_152_000, 631_152_001, 1_000_000_000,
            2_147_483_647, -1, -1200]}),
        json!({"label": "ties", "formatter": "distance", "values": [
            // 12.345 km (the exact binary value is just above the tie), 1.005 m, 0.125 km steps.
            1_234_500, 100_500, 112_500, 62_500, 50_001]}),
    ];
    gui("stat-format", "stat_format", rows)
}

/// One statistics screen row: a window, the statistics held, loaded or not, the mouse and the steps.
fn screen_row(label: &str, size: (i64, i64), stats: &Value, loaded: bool) -> Value {
    json!({"label": label, "width": size.0, "height": size.1, "mouse": [0, 0], "now": 0,
        "stats": stats, "loaded": loaded, "steps": []})
}

fn with(mut row: Value, changes: Value) -> Value {
    let fields = row.as_object_mut().expect("an object");
    for (key, value) in changes.as_object().expect("an object") {
        fields.insert(key.clone(), value.clone());
    }
    row
}

/// `StatsScreen`: loading (at times, in windows of different shapes), then the general tab with
/// numbers of each formatter, scrolled, with the mouse over a tab and over Done.
pub fn stats_screen() -> Value {
    let stats = json!([
        ["minecraft:custom", "minecraft:jump", 1234],
        ["minecraft:custom", "minecraft:play_time", 72_000],
        ["minecraft:custom", "minecraft:walk_one_cm", 123_450],
        ["minecraft:custom", "minecraft:damage_dealt", 215],
        ["minecraft:custom", "minecraft:deaths", 3],
        ["minecraft:custom", "minecraft:fly_one_cm", 12_345_678],
        ["minecraft:custom", "minecraft:time_since_death", 123_456_789],
    ]);
    let mut rows = Vec::new();
    for (size, label) in
        [((427, 240), "427"), ((320, 180), "320"), ((854, 480), "854"), ((240, 150), "240"), ((427, 241), "odd")]
    {
        rows.push(screen_row(&format!("loading-{label}"), size, &json!([]), false));
        rows.push(screen_row(&format!("general-{label}"), size, &stats, true));
    }
    for now in [300, 600, 900, 1200] {
        rows.push(with(screen_row(&format!("loading-{now}"), (427, 240), &json!([]), false), json!({"now": now})));
    }
    let general = screen_row("general", (427, 240), &stats, true);
    for (label, steps) in [
        ("scroll-1", json!([{"do": "scroll", "x": 200, "y": 100, "delta": -1.0}])),
        ("scroll-5", json!([{"do": "scroll", "x": 200, "y": 100, "delta": -5.0}])),
        ("scroll-all", json!([{"do": "scroll", "x": 200, "y": 100, "delta": -200.0}])),
        (
            "scroll-back",
            json!([{"do": "scroll", "x": 200, "y": 100, "delta": -200.0}, {"do": "scroll", "x": 200, "y": 100, "delta": 3.0}]),
        ),
        ("scroll-outside", json!([{"do": "scroll", "x": 200, "y": 10, "delta": -5.0}])),
        ("click-inactive-tab", json!([{"do": "click", "x": 200, "y": 10}])),
    ] {
        rows.push(with(general.clone(), json!({"label": format!("general-{label}"), "steps": steps})));
    }
    for (label, mouse) in
        [("tab-selected", [60, 10]), ("tab-other", [200, 10]), ("done", [200, 224]), ("list", [200, 100])]
    {
        rows.push(with(general.clone(), json!({"label": format!("hover-{label}"), "mouse": mouse})));
        rows.push(with(
            screen_row(&format!("hover-loading-{label}"), (427, 240), &json!([]), false),
            json!({"mouse": mouse}),
        ));
    }
    rows.extend(mob_rows());
    rows.extend(item_rows());
    gui("stats-screen", "stats_screen", rows)
}

/// The mobs tab: kills only, deaths only, both, many mobs scrolled, and the tab with nothing in it.
fn mob_rows() -> Vec<Value> {
    let few = json!([
        ["minecraft:killed", "minecraft:zombie", 12],
        ["minecraft:killed_by", "minecraft:creeper", 3],
        ["minecraft:killed", "minecraft:creeper", 1],
        ["minecraft:killed_by", "minecraft:creeper", 1],
        ["minecraft:killed", "minecraft:pig", 1000],
    ]);
    let mut many = Vec::new();
    for (index, name) in
        ["cow", "sheep", "chicken", "skeleton", "spider", "enderman", "witch", "bat", "squid", "cod", "salmon", "wolf"]
            .iter()
            .enumerate()
    {
        many.push(json!(["minecraft:killed", format!("minecraft:{name}"), index + 1]));
    }
    let many = Value::Array(many);
    let select_mobs = json!([{"do": "click", "x": 300, "y": 10}]);
    let mut rows = vec![
        with(screen_row("mobs-few", (427, 240), &few, true), json!({"steps": select_mobs})),
        with(screen_row("mobs-many", (427, 240), &many, true), json!({"steps": select_mobs})),
        with(
            screen_row("mobs-scrolled", (427, 240), &many, true),
            json!({"steps": [{"do": "click", "x": 300, "y": 10}, {"do": "scroll", "x": 200, "y": 100, "delta": -3.0}]}),
        ),
        with(screen_row("mobs-tab-hover", (427, 240), &few, true), json!({"mouse": [300, 10]})),
        screen_row("mobs-tab-open-but-selected-general", (427, 240), &few, true),
        with(screen_row("mobs-small", (320, 180), &many, true), json!({"steps": select_mobs})),
    ];
    rows.push(with(
        screen_row("mobs-none", (427, 240), &json!([["minecraft:custom", "minecraft:jump", 1]]), true),
        json!({"steps": select_mobs}),
    ));
    rows
}

/// The items tab. The game keeps the items in a set with no order until a column is sorted, so the
/// rows with more than one item are sorted first.
fn item_rows() -> Vec<Value> {
    let one = json!([["minecraft:used", "minecraft:diamond_sword", 5]]);
    let many = many_item_stats();
    let items_tab = json!({"do": "click", "x": 200, "y": 10});
    let frame = json!({"do": "frame"});
    let header = |column: i64| json!({"do": "click", "x": 140 + 40 * column, "y": 35});
    let sorted = |label: &str, clicks: Vec<Value>, mouse: [i64; 2]| {
        // The header's buttons take their places when drawn: a frame comes between the clicks.
        let mut steps = vec![items_tab.clone(), frame.clone()];
        for click in clicks {
            steps.push(click);
            steps.push(frame.clone());
        }
        with(screen_row(label, (427, 240), &many, true), json!({"steps": steps, "mouse": mouse}))
    };
    let mut rows = vec![
        with(screen_row("items-one", (427, 240), &one, true), json!({"steps": [items_tab.clone()]})),
        with(screen_row("items-one-small", (320, 180), &one, true), json!({"steps": [items_tab.clone()]})),
        with(
            screen_row("items-one-hover-button", (427, 240), &one, true),
            json!({"steps": [items_tab.clone()], "mouse": [172, 35]}),
        ),
        with(screen_row("items-tab-hover", (427, 240), &one, true), json!({"mouse": [200, 10]})),
        with(
            screen_row("items-none", (427, 240), &json!([["minecraft:custom", "minecraft:jump", 1]]), true),
            json!({"steps": [items_tab.clone()]}),
        ),
    ];
    for column in 0..6 {
        rows.push(sorted(&format!("items-sort-{column}"), vec![header(column)], [0, 0]));
        rows.push(sorted(&format!("items-sort-{column}-up"), vec![header(column), header(column)], [0, 0]));
    }
    rows.push(sorted("items-sort-off", vec![header(2), header(2), header(2)], [0, 0]));
    rows.push(sorted("items-sort-switch", vec![header(2), header(3)], [0, 0]));
    rows.push(sorted("items-sort-hover", vec![header(0)], [132 + 40 * 3 + 5, 35]));
    rows.push(sorted(
        "items-sort-scrolled",
        vec![header(3), json!({"do": "scroll", "x": 200, "y": 100, "delta": -4.0})],
        [0, 0],
    ));
    rows.push(sorted(
        "items-sort-scrolled-far",
        vec![header(3), json!({"do": "scroll", "x": 200, "y": 100, "delta": -100.0})],
        [0, 0],
    ));
    rows
}

/// The statistics of a good many items: blocks mined, items crafted, used, broken, picked up.
fn many_item_stats() -> Value {
    let mut stats = vec![
        json!(["minecraft:mined", "minecraft:stone", 100]),
        json!(["minecraft:mined", "minecraft:dirt", 2500]),
        json!(["minecraft:mined", "minecraft:oak_log", 12]),
        json!(["minecraft:crafted", "minecraft:oak_planks", 48]),
        json!(["minecraft:crafted", "minecraft:stick", 100]),
        json!(["minecraft:used", "minecraft:stone", 3]),
        json!(["minecraft:used", "minecraft:diamond_sword", 12345]),
        json!(["minecraft:broken", "minecraft:wooden_pickaxe", 2]),
        json!(["minecraft:picked_up", "minecraft:dirt", 7]),
        json!(["minecraft:dropped", "minecraft:oak_log", 1]),
        json!(["minecraft:picked_up", "minecraft:apple", 1_000_000]),
        json!(["minecraft:mined", "minecraft:redstone_wire", 4]),
        json!(["minecraft:mined", "minecraft:water", 9]),
    ];
    let names = [
        "cobblestone",
        "sand",
        "gravel",
        "glass",
        "torch",
        "bread",
        "bone",
        "arrow",
        "bow",
        "iron_ingot",
        "coal",
        "string",
        "feather",
        "flint",
        "leather",
        "wheat",
        "carrot",
        "potato",
        "beef",
        "porkchop",
    ];
    for (index, name) in names.iter().enumerate() {
        stats.push(json!(["minecraft:used", format!("minecraft:{name}"), index + 1]));
    }
    Value::Array(stats)
}
