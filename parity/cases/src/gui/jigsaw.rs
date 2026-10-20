//! The jigsaw block's editing screen.

use serde_json::{Value, json};

use super::gui;

/// One scenario: a block, what is done to its screen, and the time it is drawn.
#[derive(Clone)]
struct Case {
    label: String,
    orientation: &'static str,
    name: &'static str,
    target: &'static str,
    pool: &'static str,
    final_state: &'static str,
    joint: &'static str,
    priorities: (i64, i64),
    screen: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
    ms: u64,
}

fn case(label: &str) -> Case {
    Case {
        label: label.to_owned(),
        orientation: "up_north",
        name: "minecraft:bottom",
        target: "my:top",
        pool: "minecraft:empty",
        final_state: "minecraft:air",
        joint: "rollable",
        priorities: (3, 7),
        screen: (427, 240),
        mouse: (0, 0),
        steps: vec![],
        ms: 0,
    }
}

impl Case {
    fn row(self) -> Value {
        json!({
            "label": self.label, "orientation": self.orientation, "name": self.name,
            "target": self.target, "pool": self.pool, "final_state": self.final_state,
            "joint": self.joint, "selection_priority": self.priorities.0,
            "placement_priority": self.priorities.1,
            "width": self.screen.0, "height": self.screen.1,
            "mouse": [self.mouse.0, self.mouse.1], "steps": self.steps, "ms": self.ms,
        })
    }

    fn after(self, steps: Vec<Value>) -> Self {
        Self { steps, ..self }
    }

    fn at(self, mouse: (i64, i64)) -> Self {
        Self { mouse, ..self }
    }
}

fn text(text: &str) -> Value {
    json!({"do": "type", "text": text})
}

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

fn click(at: (i64, i64)) -> Value {
    json!({"do": "click", "x": at.0, "y": at.1})
}

fn shift_click(at: (i64, i64)) -> Value {
    json!({"do": "click", "x": at.0, "y": at.1, "shift": true})
}

fn drag(x: i64) -> Value {
    json!({"do": "drag", "x": x, "y": 195})
}

fn release(x: i64) -> Value {
    json!({"do": "release", "x": x, "y": 195})
}

fn wheel(at: (i64, i64), amount: i64) -> Value {
    json!({"do": "wheel", "x": at.0, "y": at.1, "amount": amount})
}

fn resize(width: i64, height: i64) -> Value {
    json!({"do": "resize", "width": width, "height": height})
}

/// The middle of a widget on the 427 by 240 screen: its left edge from the middle, top and size.
const fn middle(x: i64, y: i64, width: i64) -> (i64, i64) {
    (213 + x + width / 2, y + 10)
}

const POOL: (i64, i64) = middle(-153, 20, 300);
const NAME: (i64, i64) = middle(-153, 55, 300);
const TARGET: (i64, i64) = middle(-153, 90, 300);
const FINAL: (i64, i64) = middle(-153, 125, 300);
const SELECTION: (i64, i64) = middle(-153, 160, 98);
const PLACEMENT: (i64, i64) = middle(-50, 160, 98);
const JOINT: (i64, i64) = middle(54, 160, 100);
const LEVELS: (i64, i64) = middle(-154, 185, 100);
const KEEP: (i64, i64) = middle(-50, 185, 100);
const GENERATE: (i64, i64) = middle(54, 185, 100);
const DONE: (i64, i64) = middle(-154, 210, 150);
const CANCEL: (i64, i64) = middle(4, 210, 150);

/// A pool name longer than its box.
const LONG_POOL: &str = "very:long/pool/name/that/goes/on/and/on/and/on/and/on/and/on/and/on/and/on/and/on/and/on";

fn opened() -> Vec<Case> {
    let mut rows = vec![case("opened up")];
    for orientation in ["down_south", "north_up", "east_up", "south_up", "west_up"] {
        let joint = if orientation.ends_with("_up") { "aligned" } else { "rollable" };
        rows.push(Case { orientation, joint, ..case(&format!("opened {orientation}")) });
    }
    rows.push(Case { joint: "aligned", ..case("opened up aligned") });
    rows.push(Case { name: "minecraft:empty", target: "minecraft:empty", priorities: (0, 0), ..case("opened empty") });
    rows.push(Case {
        pool: "very:long/pool/name/that/goes/on/and/on/and/on/and/on/and/on/and/on/and/on/and/on/and/on",
        ..case("opened long pool")
    });
    rows.push(Case {
        final_state: "minecraft:oak_stairs[facing=east,half=top,shape=straight,waterlogged=false]",
        ..case("opened long state")
    });
    rows.push(Case { priorities: (-12, 100), ..case("opened priorities") });
    for (label, screen) in [("small", (320, 240)), ("wide", (640, 360)), ("tall", (427, 480))] {
        rows.push(Case { screen, ..case(&format!("opened {label}")) });
        rows.push(Case { screen, orientation: "north_up", joint: "aligned", ..case(&format!("opened flat {label}")) });
    }
    rows
}

fn hovered() -> Vec<Case> {
    let mut rows = Vec::new();
    for (label, at) in [
        ("pool", POOL),
        ("name", NAME),
        ("target", TARGET),
        ("state", FINAL),
        ("selection", SELECTION),
        ("placement", PLACEMENT),
        ("joint", JOINT),
        ("levels", LEVELS),
        ("keep", KEEP),
        ("generate", GENERATE),
        ("done", DONE),
        ("cancel", CANCEL),
    ] {
        rows.push(case(&format!("hover {label}")).at(at));
    }
    rows.push(Case { orientation: "north_up", joint: "aligned", ..case("hover where joint was") }.at(JOINT));
    rows.push(case("hover done invalid").after(vec![click(NAME), key("ctrl+a"), text("A B")]).at(DONE));
    rows
}

fn focus() -> Vec<Case> {
    let mut rows = Vec::new();
    for (label, orientation, joint) in [("up", "up_north", "rollable"), ("flat", "north_up", "aligned")] {
        let base = Case { orientation, joint, ..case("") };
        for count in 1..=13 {
            rows.push(Case { label: format!("tab {count} {label}"), ..base.clone() }.after(vec![key("tab"); count]));
        }
        for count in 1..=3 {
            rows.push(
                Case { label: format!("tab back {count} {label}"), ..base.clone() }
                    .after(vec![key("shift+tab"); count]),
            );
        }
        rows.push(
            Case { label: format!("tab to selection then click name {label}"), ..base.clone() }
                .after([vec![key("tab"); 4], vec![click(NAME)]].concat()),
        );
        rows.push(
            Case { label: format!("tab to selection then click done {label}"), ..base.clone() }
                .after([vec![key("tab"); 4], vec![click(CANCEL)]].concat()),
        );
    }
    let tabs = |count: usize| vec![key("tab"); count];
    rows.extend([
        case("arrow down").after(vec![key("down")]),
        case("arrow down twice").after(vec![key("down"), key("down")]),
        case("arrow right").after(vec![key("right")]),
        case("arrow in box").after(vec![key("left"), key("left")]),
        case("arrow up from name").after(vec![key("tab"), key("up")]),
        case("arrow down from keep").after([tabs(8), vec![key("down")]].concat()),
        case("arrow right from selection").after([tabs(4), vec![key("right"), key("right")]].concat()),
        case("arrow arrow tooltip").after([tabs(5), vec![key("up")]].concat()),
        case("space on joint").after([tabs(6), vec![key("space")]].concat()),
        case("enter on joint").after([tabs(6), vec![key("enter")]].concat()),
        case("space on keep").after([tabs(8), vec![key("space")]].concat()),
        case("space on cancel").after([tabs(11), vec![key("space")]].concat()),
        case("enter on generate").after([tabs(9), vec![key("enter")]].concat()),
        case("space on done").after([tabs(10), vec![key("space")]].concat()),
        case("shift enter joint").after([tabs(6), vec![key("shift+enter")]].concat()),
    ]);
    rows
}

fn typing() -> Vec<Case> {
    let replace = |label: &str, at: (i64, i64), value: &str, then: Vec<Value>| {
        case(label).after([vec![click(at), key("ctrl+a"), text(value)], then].concat())
    };
    let long_pool = |label: &str, steps: Vec<Value>| Case { pool: LONG_POOL, ..case(label) }.after(steps);
    vec![
        long_pool("long pool select all", vec![click(POOL), key("ctrl+a")]),
        long_pool("long pool home", vec![click(POOL), key("home")]),
        long_pool("long pool home then end", vec![click(POOL), key("home"), key("end")]),
        long_pool("long pool shift end", vec![click(POOL), key("home"), key("shift+end")]),
        long_pool("long pool shift left", vec![click(POOL), key("shift+left"), key("shift+left")]),
        long_pool("long pool insert in middle", vec![click((200, POOL.1)), text("zzzzzzzzzz")]),
        long_pool("long pool select all type", vec![click(POOL), key("ctrl+a"), text("q")]),
        replace("pool typed", POOL, "my:pool", vec![]),
        replace("pool typed done", POOL, "my:pool", vec![key("enter")]),
        replace("pool short done", POOL, "tower", vec![key("enter")]),
        replace("name invalid", NAME, "A B", vec![]),
        replace("name invalid done click", NAME, "A B", vec![click(DONE)]),
        replace("name invalid enter", NAME, "A B", vec![key("enter")]),
        replace("name invalid generate", NAME, "A B", vec![click(GENERATE)]),
        replace(
            "name invalid tab",
            NAME,
            "A B",
            vec![key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), key("tab")],
        ),
        replace("name invalid then fixed", NAME, "A B", vec![key("ctrl+a"), text("a:b"), click(DONE)]),
        replace("name empty", NAME, "", vec![click(DONE)]),
        replace("name empty enter", NAME, "", vec![key("enter")]),
        replace(
            "name backspace",
            NAME,
            "A B",
            vec![key("backspace"), key("backspace"), key("backspace"), key("backspace")],
        ),
        replace("target invalid", TARGET, "x y", vec![click(DONE)]),
        replace("pool invalid", POOL, "Pool", vec![click(DONE)]),
        replace("target slash", TARGET, "a/b", vec![click(DONE)]),
        replace("target colon twice", TARGET, "a:b:c", vec![click(DONE)]),
        replace("target uppercase namespace", TARGET, "A:b", vec![click(DONE)]),
        replace("state typed", FINAL, "minecraft:stone", vec![click(DONE)]),
        replace("state invalid still done", FINAL, "!!!", vec![click(DONE)]),
        replace("state limit", FINAL, &"s".repeat(300), vec![click(DONE)]),
        replace("name limit", NAME, &"n".repeat(200), vec![]),
        replace("selection typed", SELECTION, "12", vec![click(DONE)]),
        replace("selection invalid", SELECTION, "ab", vec![click(DONE)]),
        replace("selection negative", SELECTION, "-5", vec![click(DONE)]),
        replace("selection limit", SELECTION, "1234", vec![click(DONE)]),
        replace("selection plus", SELECTION, "+9", vec![click(DONE)]),
        replace("selection space", SELECTION, " 9", vec![click(DONE)]),
        replace("selection empty", SELECTION, "", vec![click(DONE)]),
        replace("placement typed", PLACEMENT, "99", vec![click(DONE)]),
        replace("placement invalid", PLACEMENT, "1.5", vec![click(DONE)]),
        replace("placement tooltip focus", PLACEMENT, "5", vec![]),
        case("click in pool middle").after(vec![click((100, POOL.1)), text("Z")]),
        case("type with no box").after(vec![click((213, 235)), text("Z")]),
        case("type into slider").after(vec![click(LEVELS), text("Z")]),
        case("type into button").after(vec![click(KEEP), text("Z")]),
    ]
}

fn closing() -> Vec<Case> {
    let after = |label: &str, steps: Vec<Value>| case(label).after(steps);
    let flat =
        |label: &str, steps: Vec<Value>| Case { orientation: "north_up", joint: "aligned", ..case(label) }.after(steps);
    vec![
        after("done", vec![click(DONE)]),
        after("enter", vec![key("enter")]),
        after("cancel", vec![click(CANCEL)]),
        after("escape", vec![key("escape")]),
        after("generate", vec![click(GENERATE)]),
        after("generate keep off", vec![click(KEEP), click(GENERATE)]),
        after("generate levels", vec![click((130, 195)), click(GENERATE)]),
        after("generate typed", vec![click(NAME), key("ctrl+a"), text("tower"), click(GENERATE)]),
        after("done joint", vec![click(JOINT), click(DONE)]),
        after("done joint twice", vec![click(JOINT), click(JOINT), click(DONE)]),
        after("done shift joint", vec![shift_click(JOINT), click(DONE)]),
        after("wheel joint", vec![wheel(JOINT, -1), click(DONE)]),
        after("wheel joint up", vec![wheel(JOINT, 1), click(DONE)]),
        after("wheel keep", vec![wheel(KEEP, -1), click(GENERATE)]),
        after("wheel elsewhere", vec![wheel(POOL, -1), click(DONE)]),
        flat("done flat", vec![click(DONE)]),
        flat("click where joint was", vec![click(JOINT), click(DONE)]),
        flat("wheel where joint was", vec![wheel(JOINT, -1), click(DONE)]),
        after("click empty", vec![click((5, 5)), click(DONE)]),
        after("click label", vec![click((100, 10))]),
        after("press after close", vec![click(DONE), click(DONE)]),
        after("tab to done and enter", vec![key("tab"); 11].into_iter().chain([key("enter")]).collect()),
        after("tab to generate and space", vec![key("tab"); 10].into_iter().chain([key("space")]).collect()),
        after("tab to cancel and enter", vec![key("tab"); 12].into_iter().chain([key("enter")]).collect()),
        after("tab round", vec![key("tab"); 13]),
    ]
}

fn slider() -> Vec<Case> {
    let after = |label: &str, steps: Vec<Value>| case(label).after(steps);
    let flat =
        |label: &str, steps: Vec<Value>| Case { orientation: "north_up", joint: "aligned", ..case(label) }.after(steps);
    let tabs = |count: usize| vec![key("tab"); count];
    let mut rows = vec![
        after("slider click left", vec![click((59, 195))]),
        after("slider click start", vec![click((63, 195))]),
        after("slider click middle", vec![click((109, 195))]),
        after("slider click quarter", vec![click((86, 195))]),
        after("slider click end", vec![click((155, 195))]),
        after("slider click edge", vec![click((158, 195))]),
        after("slider click then generate", vec![click((109, 195)), click(GENERATE)]),
        after("slider drag", vec![click((70, 195)), drag(140)]),
        after("slider drag then release", vec![click((70, 195)), drag(140), release(140)]),
        after("slider drag past", vec![click((70, 195)), drag(400)]),
        after("slider drag before", vec![click((100, 195)), drag(-20)]),
        after("slider drag after release", vec![click((70, 195)), release(70), drag(140)]),
        after("slider drag unfocused", vec![drag(140)]),
        after("slider hover handle", vec![]).at((70, 195)),
        after("slider hover focused", vec![click((109, 195))]).at((300, 100)),
        after("slider focused by tab", tabs(7)),
        after("slider tab right", [tabs(7), vec![key("right")]].concat()),
        after("slider tab right many", [tabs(7), vec![key("right"); 20]].concat()),
        after("slider tab left", [tabs(7), vec![key("right"), key("right"), key("left")]].concat()),
        after("slider tab space then right", [tabs(7), vec![key("space"), key("right")]].concat()),
        after("slider tab space space right", [tabs(7), vec![key("space"), key("space"), key("right")]].concat()),
        after("slider tab enter then right", [tabs(7), vec![key("enter"), key("right")]].concat()),
        after("slider tab down", [tabs(7), vec![key("down")]].concat()),
        after("slider tab up", [tabs(7), vec![key("up")]].concat()),
        after("slider arrow focus", vec![key("down"), key("down")]),
        after("slider click then tab", vec![click((109, 195)), key("tab")]),
        after("slider click then right", vec![click((109, 195)), key("right"), key("right")]),
        after("slider click then space", vec![click((109, 195)), key("space"), key("right")]),
        after(
            "slider through arrows then right",
            vec![key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), key("down"), key("right")],
        ),
        flat("slider flat tab", tabs(6)),
        flat("slider flat tab right", [tabs(6), vec![key("right"); 5]].concat()),
        flat("slider flat tab twice", [tabs(6), vec![key("right")]].concat()),
        after("slider wheel", vec![wheel(LEVELS, -1)]),
        after("slider tab then click away", [tabs(7), vec![click(NAME)]].concat()),
        after("slider tab right then tab", [tabs(7), vec![key("right"), key("right"), key("tab")]].concat()),
    ];
    rows.push(after("slider click edge generate keep", vec![click((158, 195)), click(KEEP), click(GENERATE)]));
    rows
}

fn resized() -> Vec<Case> {
    let after = |label: &str, steps: Vec<Value>| case(label).after(steps);
    vec![
        after("resize", vec![resize(640, 360)]),
        after("resize small", vec![resize(320, 240)]),
        Case { orientation: "north_up", joint: "aligned", ..case("resize flat") }.after(vec![resize(640, 360)]),
        after("resize keeps text", vec![click(NAME), key("ctrl+a"), text("a:z"), resize(640, 360)]),
        after("resize keeps invalid", vec![click(NAME), key("ctrl+a"), text("A B"), resize(640, 360)]),
        after(
            "resize invalid done",
            vec![
                click(NAME),
                key("ctrl+a"),
                text("A B"),
                resize(640, 360),
                click((320 + 4 + 75 - 154 + 150 - 75, 220)),
            ],
        ),
        after("resize keeps joint", vec![click(JOINT), resize(640, 360)]),
        after("resize keeps keep", vec![click(KEEP), resize(640, 360)]),
        after("resize slider", vec![click((109, 195)), resize(640, 360)]),
        after("resize slider generate", vec![click((130, 195)), resize(640, 360), click((320 + 54 + 50, 195))]),
        after("resize slider again", vec![click((109, 195)), resize(640, 360), click((320 - 154 + 50, 195))]),
        after("resize focus", vec![key("tab"), key("tab"), resize(640, 360)]),
        after("resize focus type", vec![key("tab"), key("tab"), resize(640, 360), text("Z")]),
        after("resize keeps priorities", vec![click(SELECTION), key("ctrl+a"), text("42"), resize(640, 360)]),
        after("resize tooltip", vec![key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), resize(640, 360)]),
    ]
}

/// The jigsaw block's screen: the boxes, labels, the joint, the levels slider and the buttons for
/// blocks of each facing, the identifiers' validity gating Done and Generate, the priorities'
/// tooltips, the keys, clicks, drags and wheel, what Done and Generate send, and the screen made
/// again at another size.
pub fn jigsaw_screen() -> Value {
    let rows = [opened(), hovered(), focus(), typing(), closing(), slider(), resized()].concat();
    gui("jigsaw-screen", "jigsaw_screen", rows.into_iter().map(Case::row).collect())
}
