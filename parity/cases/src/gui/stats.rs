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
    rows.extend(key_rows());
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

const CTRL: i64 = 2;
const SHIFT: i64 = 1;
const TAB: i64 = 258;

fn key(code: i64, modifiers: i64) -> Value {
    json!({"do": "key", "key": code, "modifiers": modifiers})
}

fn digit(digit: i64) -> i64 {
    48 + digit
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn enter_after(mut steps: Vec<Value>) -> Vec<Value> {
    steps.push(key(257, 0));
    steps
}

fn enter_twice(steps: Vec<Value>) -> Vec<Value> {
    enter_after(enter_after(steps))
}

fn space_after(mut steps: Vec<Value>) -> Vec<Value> {
    steps.push(key(32, 0));
    steps
}

/// A loaded 427x240 screen of the statistics `stats`, and what is done to it.
fn on(label: &str, stats: &Value, loaded: bool, steps: Vec<Value>) -> Value {
    with(screen_row(label, (427, 240), stats, loaded), json!({"steps": steps}))
}

/// Every tab can be chosen: general, an item used and a mob killed.
fn all_stats() -> Value {
    json!([
        ["minecraft:custom", "minecraft:jump", 1234],
        ["minecraft:used", "minecraft:diamond_sword", 5],
        ["minecraft:killed", "minecraft:zombie", 12],
    ])
}

/// The Items tab chosen by a Ctrl+digit, then `count` presses of Tab with the modifiers.
fn items_then_tab(count: usize, modifiers: i64) -> Vec<Value> {
    let mut steps = vec![key(digit(2), CTRL)];
    steps.extend((0..count).map(|_| key(TAB, modifiers)));
    steps
}

/// The keys the tab bar takes (`TabNavigationBar.keyPressed`): Control with a digit or Tab, in a
/// loaded screen with every tab usable, one with the middle tab empty, and while loading.
fn key_rows() -> Vec<Value> {
    let all = all_stats();
    let no_items = json!([["minecraft:custom", "minecraft:jump", 1234], ["minecraft:killed", "minecraft:zombie", 12]]);
    let only_general = json!([["minecraft:custom", "minecraft:jump", 1234]]);
    let mut rows = vec![
        on("key-ctrl-2", &all, true, vec![key(digit(2), CTRL)]),
        on("key-ctrl-3", &all, true, vec![key(digit(3), CTRL)]),
        on("key-ctrl-1-stays", &all, true, vec![key(digit(1), CTRL)]),
        on("key-ctrl-0-last", &all, true, vec![key(digit(0), CTRL)]),
        on("key-ctrl-5-last", &all, true, vec![key(digit(5), CTRL)]),
        on("key-ctrl-3-inactive", &no_items, true, vec![key(digit(2), CTRL)]),
        on("key-ctrl-2-then-1", &all, true, vec![key(digit(2), CTRL), key(digit(1), CTRL)]),
        on("key-ctrl-tab", &all, true, vec![key(TAB, CTRL)]),
        on("key-ctrl-tab-twice", &all, true, vec![key(TAB, CTRL), key(TAB, CTRL)]),
        on("key-ctrl-tab-wraps", &all, true, vec![key(TAB, CTRL), key(TAB, CTRL), key(TAB, CTRL)]),
        on("key-ctrl-shift-tab", &all, true, vec![key(TAB, CTRL | SHIFT)]),
        on("key-ctrl-tab-skips-empty", &no_items, true, vec![key(TAB, CTRL)]),
        on("key-ctrl-shift-tab-skips-empty", &no_items, true, vec![key(TAB, CTRL | SHIFT), key(TAB, CTRL | SHIFT)]),
        on("key-ctrl-tab-loading", &only_general, false, vec![key(TAB, CTRL)]),
        on("key-ctrl-2-loading", &json!([]), false, vec![key(digit(2), CTRL)]),
        on("key-ctrl-tab-only-general", &only_general, true, vec![key(TAB, CTRL)]),
        on("key-digit-alone", &all, true, vec![key(digit(2), 0)]),
        on("key-shift-digit", &all, true, vec![key(digit(2), SHIFT)]),
        on("key-ctrl-keypad-digit", &all, true, vec![key(322, CTRL)]),
        on("key-ctrl-letter", &all, true, vec![key(65, CTRL)]),
        on("key-after-list-click", &all, true, vec![click(200, 40), key(digit(2), CTRL)]),
        on("key-after-list-click-back", &all, true, vec![click(200, 40), key(digit(2), CTRL), key(digit(1), CTRL)]),
        on("key-after-list-click-inactive", &no_items, true, vec![click(200, 40), key(digit(2), CTRL)]),
        on("key-focused-tab-inactive", &no_items, true, vec![click(70, 10), key(digit(2), CTRL)]),
        on(
            "key-focused-tab-inactive-keyboard",
            &no_items,
            true,
            vec![click(70, 10), key(TAB, CTRL), key(digit(2), CTRL)],
        ),
        on("key-after-tab-click", &all, true, vec![click(200, 10), key(digit(3), CTRL), key(TAB, CTRL)]),
    ];
    rows.extend(navigation_rows());
    rows
}

/// The keyboard focus (`Screen.keyPressed`): Tab around the bar, the list and Done, and Enter.
fn navigation_rows() -> Vec<Value> {
    let all = all_stats();
    let no_items = json!([["minecraft:custom", "minecraft:jump", 1234], ["minecraft:killed", "minecraft:zombie", 12]]);
    let mut rows = vec![
        on("nav-tab-1", &all, true, vec![key(TAB, 0)]),
        on("nav-tab-2", &all, true, vec![key(TAB, 0), key(TAB, 0)]),
        on("nav-tab-3", &all, true, vec![key(TAB, 0), key(TAB, 0), key(TAB, 0)]),
        on("nav-tab-4", &all, true, vec![key(TAB, 0), key(TAB, 0), key(TAB, 0), key(TAB, 0)]),
        on("nav-shift-tab-1", &all, true, vec![key(TAB, SHIFT)]),
        on("nav-shift-tab-2", &all, true, vec![key(TAB, SHIFT), key(TAB, SHIFT)]),
        on("nav-shift-tab-3", &all, true, vec![key(TAB, SHIFT), key(TAB, SHIFT), key(TAB, SHIFT)]),
        on("nav-list-click-tab", &all, true, vec![click(200, 40), key(TAB, 0)]),
        on("nav-list-click-tab-2", &all, true, vec![click(200, 40), key(TAB, 0), key(TAB, 0)]),
        on("nav-list-click-shift-tab", &all, true, vec![click(200, 40), key(TAB, SHIFT)]),
        on("nav-tab-click-tab", &all, true, vec![click(200, 10), key(TAB, 0)]),
        on("nav-tab-click-tab-2", &all, true, vec![click(200, 10), key(TAB, 0), key(TAB, 0)]),
        on("nav-tab-click-shift-tab", &all, true, vec![click(200, 10), key(TAB, SHIFT)]),
        on("nav-done-then-tab-click", &all, true, vec![key(TAB, 0), key(TAB, 0), key(TAB, 0), click(200, 10)]),
        on("nav-done-then-list-click", &all, true, vec![key(TAB, 0), key(TAB, 0), key(TAB, 0), click(200, 70)]),
    ];
    rows.extend(vec![
        on("nav-items-tab-1", &all, true, items_then_tab(1, 0)),
        on("nav-items-tab-2", &all, true, items_then_tab(2, 0)),
        on("nav-items-tab-6", &all, true, items_then_tab(6, 0)),
        on("nav-items-tab-7", &all, true, items_then_tab(7, 0)),
        on("nav-items-tab-9", &all, true, items_then_tab(9, 0)),
        on("nav-items-tab-10", &all, true, items_then_tab(10, 0)),
        on("nav-items-tab-12", &all, true, items_then_tab(12, 0)),
        on("nav-items-shift-tab-1", &all, true, items_then_tab(1, SHIFT)),
        on("nav-items-shift-tab-2", &all, true, items_then_tab(2, SHIFT)),
        on("nav-items-shift-tab-3", &all, true, items_then_tab(3, SHIFT)),
        on("nav-items-shift-tab-6", &all, true, items_then_tab(6, SHIFT)),
        on("nav-items-shift-tab-7", &all, true, items_then_tab(7, SHIFT)),
        on("nav-items-shift-tab-8", &all, true, items_then_tab(8, SHIFT)),
        on("nav-items-shift-tab-9", &all, true, items_then_tab(9, SHIFT)),
        on("nav-items-shift-tab-10", &all, true, items_then_tab(10, SHIFT)),
        on("nav-enter-load", &all, true, vec![key(257, 0)]),
        on("nav-items-enter-button", &all, true, enter_after(items_then_tab(2, 0))),
        on("nav-items-enter-twice", &all, true, enter_twice(items_then_tab(2, 0))),
        on("nav-items-space-button-2", &all, true, space_after(items_then_tab(3, 0))),
        on("nav-items-enter-slot", &all, true, enter_after(items_then_tab(8, 0))),
        on("nav-loading-tab", &all, false, vec![key(TAB, 0)]),
        on("nav-loading-shift-tab", &all, false, vec![key(TAB, SHIFT)]),
        on("nav-tab-enter", &all, true, vec![key(TAB, 0), key(257, 0)]),
        on("nav-tab-click-enter", &all, true, vec![click(200, 10), key(257, 0)]),
        on("nav-tab-click-space", &all, true, vec![click(200, 10), key(32, 0)]),
        on("nav-inactive-tab-enter", &no_items, true, vec![click(70, 10), key(digit(2), CTRL), key(257, 0)]),
        on("nav-list-enter", &all, true, vec![key(TAB, 0), key(TAB, 0), key(257, 0)]),
        on("nav-mobs-tab-2", &all, true, vec![click(330, 10), key(TAB, 0)]),
        on("nav-mobs-tab-3", &all, true, vec![click(330, 10), key(TAB, 0), key(TAB, 0)]),
        on("nav-mobs-shift-tab-2", &all, true, vec![click(330, 10), key(TAB, SHIFT), key(TAB, SHIFT)]),
        on("nav-mobs-list-click-tab", &all, true, vec![click(330, 10), click(200, 40), key(TAB, 0)]),
    ]);
    rows
}
