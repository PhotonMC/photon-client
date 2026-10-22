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
    rows.extend(selection_rows());
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

fn items_tab() -> Value {
    json!({"do": "click", "x": 200, "y": 10})
}

fn frame() -> Value {
    json!({"do": "frame"})
}

fn header(column: i64) -> Value {
    json!({"do": "click", "x": 140 + 40 * column, "y": 35})
}

fn scroll(delta: f64) -> Value {
    json!({"do": "scroll", "x": 200, "y": 100, "delta": delta})
}

fn resize(width: i64, height: i64) -> Value {
    json!({"do": "resize", "width": width, "height": height})
}

/// A row on the items tab with the statistics of many items, after the clicks. The header's buttons
/// take their places when drawn, so a frame comes between the clicks.
fn sorted(label: &str, clicks: Vec<Value>, mouse: [i64; 2]) -> Value {
    let mut steps = vec![items_tab(), frame()];
    for click in clicks {
        steps.push(click);
        steps.push(frame());
    }
    with(screen_row(label, (427, 240), &many_item_stats(), true), json!({"steps": steps, "mouse": mouse}))
}

/// Sorted once and the focus let go, so that the order is the game's and nothing is selected.
fn settle(clicks: Vec<Value>) -> Vec<Value> {
    let mut steps = vec![header(3), json!({"do": "click", "x": 10, "y": 100})];
    steps.extend(clicks);
    steps
}

/// The items tab. The game keeps the items in a set with no order until a column is sorted, so the
/// rows with more than one item are sorted first.
fn item_rows() -> Vec<Value> {
    let one = json!([["minecraft:used", "minecraft:diamond_sword", 5]]);
    let on_tab = |label: &str, size: (i64, i64), stats: &Value, changes: Value| {
        let row = with(screen_row(label, size, stats, true), json!({"steps": [items_tab()]}));
        with(row, changes)
    };
    let mut rows = vec![
        on_tab("items-one", (427, 240), &one, json!({})),
        on_tab("items-one-small", (320, 180), &one, json!({})),
        on_tab("items-one-hover-button", (427, 240), &one, json!({"mouse": [172, 35]})),
        with(screen_row("items-tab-hover", (427, 240), &one, true), json!({"mouse": [200, 10]})),
        on_tab("items-none", (427, 240), &json!([["minecraft:custom", "minecraft:jump", 1]]), json!({})),
    ];
    for column in 0..6 {
        rows.push(sorted(&format!("items-sort-{column}"), vec![header(column)], [0, 0]));
        rows.push(sorted(&format!("items-sort-{column}-up"), vec![header(column), header(column)], [0, 0]));
    }
    rows.push(sorted("items-sort-off", vec![header(2), header(2), header(2)], [0, 0]));
    rows.push(sorted("items-sort-switch", vec![header(2), header(3)], [0, 0]));
    rows.push(sorted("items-sort-hover", vec![header(0)], [132 + 40 * 3 + 5, 35]));
    rows.push(sorted("items-sort-scrolled", vec![header(3), scroll(-4.0)], [0, 0]));
    rows.push(sorted("items-sort-scrolled-far", vec![header(3), scroll(-100.0)], [0, 0]));
    rows.extend(item_header_rows());
    rows.extend(item_focus_rows());
    rows.extend(item_hover_rows());
    rows
}

/// The mouse over a row's item, over the numbers, and over the tabs of lists with nothing in them.
fn item_hover_rows() -> Vec<Value> {
    let nothing = json!([["minecraft:custom", "minecraft:jump", 1]]);
    let on_tab = |label: &str, tab: Value, stats: &Value, mouse: [i64; 2]| {
        with(screen_row(label, (427, 240), stats, true), json!({"steps": [tab], "mouse": mouse}))
    };
    let mobs_tab = json!({"do": "click", "x": 300, "y": 10});
    let bundle = json!([["minecraft:picked_up", "minecraft:bundle", 1]]);
    vec![
        on_tab("items-hover-bundle", items_tab(), &bundle, [80, 56]),
        sorted("items-hover-first-item", settle(vec![]), [80, 56]),
        sorted("items-hover-second-item", settle(vec![]), [80, 78]),
        sorted("items-hover-numbers", settle(vec![]), [200, 56]),
        sorted("items-hover-name", settle(vec![]), [110, 56]),
        sorted("items-hover-scrolled-item", settle(vec![scroll(-3.0)]), [80, 56]),
        on_tab("items-none-hover-items-tab", items_tab(), &nothing, [200, 10]),
        on_tab("items-none-hover-mobs-tab", items_tab(), &nothing, [300, 10]),
        on_tab("mobs-none-hover-mobs-tab", mobs_tab.clone(), &nothing, [300, 10]),
        on_tab("mobs-none-hover-items-tab", mobs_tab, &nothing, [200, 10]),
    ]
}

/// The header scrolled up under the tab bar: where a press and the pointer count.
fn item_header_rows() -> Vec<Value> {
    let click_under_bar = |x: i64| json!({"do": "click", "x": x, "y": 20});
    vec![
        sorted("items-scrolled-click-active-tab", settle(vec![scroll(-1.0), click_under_bar(140)]), [0, 0]),
        sorted("items-scrolled-click-inactive-tab", settle(vec![scroll(-1.0), click_under_bar(340)]), [0, 0]),
        sorted("items-scrolled-hover-under-bar", settle(vec![scroll(-1.0)]), [340, 20]),
        sorted("items-scrolled-hover-in-list", settle(vec![scroll(-1.0)]), [340, 28]),
    ]
}

/// The focus of the slots and the header's buttons, and what resizing does to the scroll.
fn item_focus_rows() -> Vec<Value> {
    let slot = |row: i64| json!({"do": "click", "x": 80, "y": 55 + 22 * row});
    let click = |x: i64, y: i64| json!({"do": "click", "x": x, "y": y});
    vec![
        sorted("items-slot", settle(vec![slot(0)]), [0, 0]),
        sorted("items-slot-then-tab", settle(vec![slot(1), click(60, 10)]), [0, 0]),
        sorted("items-slot-numbers-click", settle(vec![click(200, 55)]), [0, 0]),
        sorted("items-slot-scrolled-resize", settle(vec![slot(0), scroll(-40.0), resize(320, 180)]), [0, 0]),
        sorted("items-slot-clipped", settle(vec![scroll(-1.0), slot(0)]), [0, 0]),
        sorted("items-resize", vec![header(3), scroll(-4.0), resize(320, 180)], [0, 0]),
        sorted("items-resize-scroll-only", vec![header(3), scroll(-4.0), scroll(-4.0), resize(427, 200)], [0, 0]),
        sorted("items-resize-back", vec![header(1), resize(240, 150), resize(427, 240)], [0, 0]),
        with(
            screen_row("general-resize", (427, 240), &many_item_stats(), true),
            json!({"steps": [scroll(-6.0), resize(300, 160)]}),
        ),
        with(screen_row("loading-resize", (427, 240), &json!([]), false), json!({"steps": [resize(200, 120)]})),
    ]
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

/// Selecting a row of the general and the mobs lists: the outline, the focus lost to the tab bar,
/// a row partly out of the list, and the selection kept over a resize.
fn selection_rows() -> Vec<Value> {
    let stats = json!([
        ["minecraft:custom", "minecraft:jump", 1234],
        ["minecraft:killed", "minecraft:zombie", 12],
        ["minecraft:killed_by", "minecraft:creeper", 3],
        ["minecraft:killed", "minecraft:pig", 4],
        ["minecraft:killed", "minecraft:cow", 5],
        ["minecraft:killed", "minecraft:sheep", 6],
    ]);
    let click = |x: i64, y: i64| json!({"do": "click", "x": x, "y": y});
    let scroll = |delta: f64| json!({"do": "scroll", "x": 200, "y": 100, "delta": delta});
    let resize = |width: i64, height: i64| json!({"do": "resize", "width": width, "height": height});
    let mobs = click(300, 10);
    let general =
        |label: &str, steps: Vec<Value>| with(screen_row(label, (427, 240), &stats, true), json!({"steps": steps}));
    let mut rows = vec![
        general("general-select", vec![click(200, 40)]),
        general("general-select-empty-area", vec![click(10, 40)]),
        general("general-select-then-tab", vec![click(200, 40), click(60, 10)]),
        general("general-select-then-done-area", vec![click(200, 40), click(200, 207)]),
        general("general-select-scrolled", vec![scroll(-2.0), click(200, 40)]),
        general("general-select-clipped-top", vec![scroll(-2.0), click(200, 25)]),
        general("general-select-clipped-bottom", vec![click(200, 200)]),
        general("general-select-resize", vec![scroll(-9.0), click(200, 100), resize(320, 150)]),
        general("general-select-resize-taller", vec![click(200, 200), resize(427, 400)]),
        general("general-select-twice", vec![click(200, 40), click(200, 70)]),
        general("general-select-under-bar", vec![click(200, 20)]),
    ];
    rows.push(general("mobs-select", vec![mobs.clone(), click(200, 50)]));
    rows.push(general("mobs-select-second", vec![mobs.clone(), click(200, 90)]));
    rows.push(general("mobs-select-then-tab", vec![mobs.clone(), click(200, 50), click(60, 10)]));
    rows.push(general("mobs-select-clipped", vec![mobs.clone(), scroll(-1.0), click(200, 25)]));
    rows
}
