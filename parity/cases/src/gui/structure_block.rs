//! The structure block's editing screen.

use serde_json::{Value, json};

use super::gui;

/// One scenario: a block, what is done to its screen, and the time it is drawn.
#[derive(Clone)]
struct Case {
    label: String,
    mode: &'static str,
    name: &'static str,
    metadata: &'static str,
    pos: (i64, i64, i64),
    size: (i64, i64, i64),
    mirror: &'static str,
    rotation: &'static str,
    flags: [bool; 4],
    integrity: f64,
    seed: i64,
    screen: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
    ms: u64,
}

fn case(label: &str, mode: &'static str) -> Case {
    Case {
        label: label.to_owned(),
        mode,
        name: "minecraft:house",
        metadata: "chest",
        pos: (1, 2, -3),
        size: (4, 5, 6),
        mirror: "none",
        rotation: "none",
        flags: [true, false, false, true],
        integrity: 0.5,
        seed: 12345,
        screen: (427, 240),
        mouse: (0, 0),
        steps: vec![],
        ms: 0,
    }
}

impl Case {
    fn row(self) -> Value {
        json!({
            "label": self.label, "mode": self.mode, "name": self.name, "metadata": self.metadata,
            "pos": [self.pos.0, self.pos.1, self.pos.2],
            "size": [self.size.0, self.size.1, self.size.2],
            "mirror": self.mirror, "rotation": self.rotation,
            "ignore_entities": self.flags[0], "strict": self.flags[1],
            "show_air": self.flags[2], "show_bounding_box": self.flags[3],
            "integrity": self.integrity, "seed": self.seed,
            "width": self.screen.0, "height": self.screen.1,
            "mouse": [self.mouse.0, self.mouse.1], "steps": self.steps, "ms": self.ms,
        })
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

const DONE: (i64, i64) = middle(-154, 210, 150);
const CANCEL: (i64, i64) = middle(4, 210, 150);
const SAVE: (i64, i64) = middle(104, 185, 50);
const LOAD: (i64, i64) = middle(104, 185, 50);
const MODE: (i64, i64) = middle(-154, 185, 50);
const DETECT: (i64, i64) = middle(104, 120, 50);
const STRICT: (i64, i64) = middle(104, 120, 50);
const ENTITIES: (i64, i64) = middle(104, 160, 50);
const MIRROR: (i64, i64) = middle(-20, 185, 40);
const AIR: (i64, i64) = middle(104, 80, 50);
const BOUNDS: (i64, i64) = middle(104, 80, 50);
const ROTATIONS: [(i64, i64); 4] =
    [middle(-102, 185, 40), middle(-61, 185, 40), middle(21, 185, 40), middle(62, 185, 40)];
const NAME: (i64, i64) = middle(-152, 40, 300);
const POS_X: (i64, i64) = middle(-152, 80, 80);
const SIZE_X: (i64, i64) = middle(-152, 120, 80);
const SEED: (i64, i64) = middle(-72, 120, 80);
const DATA: (i64, i64) = middle(-152, 120, 240);

const MODES: [&str; 4] = ["save", "load", "corner", "data"];

fn every_mode(label: &str, change: impl Fn(Case) -> Case) -> Vec<Case> {
    MODES.iter().map(|mode| change(case(&format!("{label} {mode}"), mode))).collect()
}

fn opened() -> Vec<Case> {
    let mut rows = every_mode("opened", |case| case);
    rows.extend(every_mode("empty", |case| Case { name: "", metadata: "", pos: (0, 0, 0), size: (0, 0, 0), ..case }));
    rows.extend(every_mode("flags", |case| Case {
        flags: [false, true, true, false],
        mirror: "left_right",
        rotation: "clockwise_90",
        integrity: 1.0,
        seed: 0,
        ..case
    }));
    rows.extend(every_mode("limits", |case| Case {
        pos: (-48, 48, 0),
        size: (48, 0, 48),
        integrity: 0.123_456_7,
        seed: -9_000_000_000,
        mirror: "front_back",
        rotation: "counterclockwise_90",
        ..case
    }));
    for (label, rotation) in [("r180", "clockwise_180"), ("r90", "clockwise_90"), ("r270", "counterclockwise_90")] {
        rows.push(Case { rotation, ..case(&format!("rotation {label}"), "load") });
    }
    for (label, screen) in [("small", (320, 240)), ("wide", (640, 360)), ("tall", (427, 480))] {
        rows.extend(every_mode(label, |case| Case { screen, ..case }));
    }
    rows
}

fn hovered() -> Vec<Case> {
    let hover = |label: &str, mode, mouse| Case { mouse, ..case(label, mode) };
    vec![
        hover("hover done", "save", DONE),
        hover("hover cancel", "save", CANCEL),
        hover("hover save", "save", SAVE),
        hover("hover detect", "save", DETECT),
        hover("hover air", "save", AIR),
        hover("hover entities", "save", ENTITIES),
        hover("hover mode", "save", MODE),
        hover("hover load", "load", LOAD),
        hover("hover mirror", "load", MIRROR),
        hover("hover rotation", "load", ROTATIONS[1]),
        hover("hover strict", "load", STRICT),
        hover("hover bounds", "load", BOUNDS),
        hover("hover name", "load", NAME),
    ]
}

fn pressed() -> Vec<Case> {
    let after = |label: &str, mode, steps: Vec<Value>| Case { steps, ..case(label, mode) };
    vec![
        after("press mode", "data", vec![click(MODE)]),
        after("press mode twice", "data", vec![click(MODE); 2]),
        after("press mode thrice", "data", vec![click(MODE); 3]),
        after("press mode round", "data", vec![click(MODE); 4]),
        after("shift press mode", "data", vec![shift_click(MODE)]),
        after("wheel mode", "save", vec![wheel(MODE, -1)]),
        after("wheel mode up", "save", vec![wheel(MODE, 1)]),
        after("press entities", "save", vec![click(ENTITIES)]),
        after("press entities twice", "save", vec![click(ENTITIES); 2]),
        after("press air", "save", vec![click(AIR)]),
        after("press strict", "load", vec![click(STRICT)]),
        after("press bounds", "load", vec![click(BOUNDS)]),
        after("press mirror", "load", vec![click(MIRROR)]),
        after("press mirror twice", "load", vec![click(MIRROR); 2]),
        after("press mirror round", "load", vec![click(MIRROR); 3]),
        after("wheel mirror", "load", vec![wheel(MIRROR, -1)]),
        after("press rotation 0", "load", vec![click(ROTATIONS[0])]),
        after("press rotation 90", "load", vec![click(ROTATIONS[1])]),
        after("press rotation 180", "load", vec![click(ROTATIONS[2])]),
        after("press rotation 270", "load", vec![click(ROTATIONS[3])]),
        after("press where save was", "load", vec![click(SAVE)]),
        after("press where entities was", "data", vec![click(ENTITIES)]),
        after("click between", "save", vec![click((213, 100))]),
        after("mode then press entities", "data", vec![click(MODE), click(ENTITIES)]),
    ]
}

fn closing() -> Vec<Case> {
    let after = |label: &str, mode, steps: Vec<Value>| Case { steps, ..case(label, mode) };
    let mut rows = Vec::new();
    for mode in MODES {
        rows.push(after(&format!("done {mode}"), mode, vec![click(DONE)]));
        rows.push(after(&format!("enter {mode}"), mode, vec![key("enter")]));
        rows.push(after(&format!("cancel {mode}"), mode, vec![click(CANCEL)]));
        rows.push(after(&format!("escape {mode}"), mode, vec![key("escape")]));
    }
    rows.extend([
        after("save button", "save", vec![click(SAVE)]),
        after("load button", "load", vec![click(LOAD)]),
        after("detect button", "save", vec![click(DETECT)]),
        after("save then done", "save", vec![click(SAVE), click(DONE)]),
        after("detect then done", "save", vec![click(DETECT), click(DONE)]),
        after("cancel after mode", "data", vec![click(MODE), click(CANCEL)]),
        after("escape after mode", "data", vec![click(MODE), click(MODE), key("escape")]),
        after(
            "cancel after flags",
            "load",
            vec![click(STRICT), click(BOUNDS), click(MIRROR), click(ROTATIONS[3]), click(CANCEL)],
        ),
        after("cancel after entities", "save", vec![click(ENTITIES), click(AIR), click(CANCEL)]),
        after("done after flags", "load", vec![click(STRICT), click(MIRROR), click(ROTATIONS[2]), click(DONE)]),
        after("done after mode", "data", vec![click(MODE), click(DONE)]),
        after("done after mode round", "data", vec![click(MODE); 4]),
    ]);
    rows.push(Case {
        steps: [vec![click(MODE); 4], vec![click(DONE)]].concat(),
        ..case("done after mode round", "data")
    });
    rows
}

fn typing() -> Vec<Case> {
    let after = |label: &str, mode, steps: Vec<Value>| Case { steps, ..case(label, mode) };
    vec![
        after("type name", "save", vec![text("x")]),
        after("type name done", "save", vec![text("x"), key("enter")]),
        after("name replaced", "save", vec![key("ctrl+a"), text("my:castle"), key("enter")]),
        after("name bad", "save", vec![key("ctrl+a"), text("Bad Name"), key("enter")]),
        after("name empty", "save", vec![key("ctrl+a"), key("backspace"), key("enter")]),
        after("name no namespace", "save", vec![key("ctrl+a"), text("tower"), key("enter")]),
        after("name slash", "save", vec![key("ctrl+a"), text("a/b"), key("enter")]),
        after("name upper", "save", vec![key("ctrl+a"), text("ABC"), key("enter")]),
        after("pos x", "save", vec![click(POS_X), key("ctrl+a"), text("-7"), key("enter")]),
        after("pos x invalid", "save", vec![click(POS_X), key("ctrl+a"), text("abc"), key("enter")]),
        after("pos x empty", "save", vec![click(POS_X), key("ctrl+a"), key("backspace"), key("enter")]),
        after("pos x big", "save", vec![click(POS_X), key("ctrl+a"), text("999"), key("enter")]),
        after("pos x small", "save", vec![click(POS_X), key("ctrl+a"), text("-999"), key("enter")]),
        after("pos x plus", "save", vec![click(POS_X), key("ctrl+a"), text("+5"), key("enter")]),
        after("pos x decimal", "save", vec![click(POS_X), key("ctrl+a"), text("1.5"), key("enter")]),
        after("pos x spaces", "save", vec![click(POS_X), key("ctrl+a"), text(" 5 "), key("enter")]),
        after("size x", "save", vec![click(SIZE_X), key("ctrl+a"), text("30"), key("enter")]),
        after("size x big", "save", vec![click(SIZE_X), key("ctrl+a"), text("100"), key("enter")]),
        after("size x negative", "save", vec![click(SIZE_X), key("ctrl+a"), text("-4"), key("enter")]),
        after("integrity", "load", vec![click(SIZE_X), key("ctrl+a"), text("0.25"), key("enter")]),
        after("integrity big", "load", vec![click(SIZE_X), key("ctrl+a"), text("7"), key("enter")]),
        after("integrity negative", "load", vec![click(SIZE_X), key("ctrl+a"), text("-1"), key("enter")]),
        after("integrity invalid", "load", vec![click(SIZE_X), key("ctrl+a"), text("x"), key("enter")]),
        after("integrity percent", "load", vec![click(SIZE_X), key("ctrl+a"), text("50%"), key("enter")]),
        after("integrity comma", "load", vec![click(SIZE_X), key("ctrl+a"), text("0,5"), key("enter")]),
        after("integrity exponent", "load", vec![click(SIZE_X), key("ctrl+a"), text("1e-1"), key("enter")]),
        after("seed", "load", vec![click(SEED), key("ctrl+a"), text("77"), key("enter")]),
        after("seed invalid", "load", vec![click(SEED), key("ctrl+a"), text("7x"), key("enter")]),
        after("seed empty", "load", vec![click(SEED), key("ctrl+a"), key("backspace"), key("enter")]),
        after("seed big", "load", vec![click(SEED), key("ctrl+a"), text("9223372036854775807"), key("enter")]),
        after("seed overflow", "load", vec![click(SEED), key("ctrl+a"), text("9223372036854775808"), key("enter")]),
        after("seed negative", "load", vec![click(SEED), key("ctrl+a"), text("-5"), key("enter")]),
        after("data", "data", vec![text("!")]),
        after("data done", "data", vec![text("!"), key("enter")]),
        after("data replaced", "data", vec![key("ctrl+a"), text("chest_a"), key("enter")]),
        after("data long", "data", vec![text(&"long".repeat(40))]),
        after("data long done", "data", vec![text(&"long".repeat(40)), key("enter")]),
        after("data empty", "data", vec![key("ctrl+a"), key("backspace"), key("enter")]),
        after("click data", "data", vec![click(DATA), text("Z")]),
        after("click name", "load", vec![click(NAME), text("Z"), key("enter")]),
        after("click name middle", "load", vec![click(middle(-152, 40, 100)), text("Z")]),
    ]
}

fn focus() -> Vec<Case> {
    let after = |label: &str, mode, steps: Vec<Value>| Case { steps, ..case(label, mode) };
    let tabs = |count: usize| vec![key("tab"); count];
    let mut rows = Vec::new();
    for mode in MODES {
        for count in 1..=12 {
            rows.push(after(&format!("tab {count} {mode}"), mode, tabs(count)));
        }
        rows.push(after(&format!("tab back {mode}"), mode, vec![key("shift+tab")]));
        rows.push(after(&format!("tab back twice {mode}"), mode, vec![key("shift+tab"); 2]));
    }
    rows.extend([
        after("tab to done enter", "save", vec![key("enter")]),
        after("tab space mode", "data", vec![key("tab"), key("tab"), key("space")]),
        after("tab enter mode", "data", vec![key("tab"), key("tab"), key("enter")]),
        after("tab space cancel", "data", vec![key("tab"), key("space")]),
        after(
            "tab space entities",
            "save",
            vec![key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), key("space")],
        ),
        after("arrow down", "save", vec![key("down")]),
        after("arrow right", "save", vec![key("right")]),
        after("arrow left", "save", vec![key("left")]),
        after("arrow up", "save", vec![key("up")]),
        after("arrow down from name", "load", vec![key("tab"), key("tab"), key("tab"), key("down")]),
        after("arrow right from mode", "data", vec![key("tab"), key("tab"), key("right")]),
        after("arrow up from cancel", "data", vec![key("tab"), key("up")]),
        after("arrow left from cancel", "data", vec![key("tab"), key("left")]),
        after("click then tab", "save", vec![click(POS_X), key("tab")]),
        after("click then shift tab", "save", vec![click(POS_X), key("shift+tab")]),
    ]);
    rows
}

fn resized() -> Vec<Case> {
    let after = |label: &str, mode, steps: Vec<Value>| Case { steps, ..case(label, mode) };
    let mut rows = Vec::new();
    for mode in MODES {
        rows.push(after(&format!("resize {mode}"), mode, vec![resize(640, 360)]));
    }
    rows.extend([
        after("resize keeps name", "save", vec![text("z"), resize(640, 360)]),
        after("resize keeps mode", "data", vec![click(MODE), resize(500, 300)]),
        after("resize keeps flags", "load", vec![click(STRICT), click(MIRROR), resize(500, 300)]),
        after("resize keeps number", "save", vec![click(POS_X), key("ctrl+a"), text("9"), resize(500, 300)]),
        after("resize then done", "save", vec![resize(500, 300), click(DONE)]),
    ]);
    rows
}

/// The structure block's screen in each mode: the labels, boxes and buttons for a block's values,
/// the buttons pressed, scrolled and tabbed to, the numbers typed with what the game makes of
/// them, Done, Cancel and Escape with the packets sent and the values restored, and the screen
/// made again at another size.
pub fn structure_block_screen() -> Value {
    let rows = [opened(), hovered(), pressed(), closing(), typing(), focus(), resized()].concat();
    gui("structure-block-screen", "structure_block_screen", rows.into_iter().map(Case::row).collect())
}
