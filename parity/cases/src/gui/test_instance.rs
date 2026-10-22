//! The test instance block's editing screen.

use serde_json::{Value, json};

use super::gui;

/// One scenario: a block, what is done to its screen, and the time it is drawn.
#[derive(Clone)]
struct Case {
    label: String,
    test: Option<&'static str>,
    size: (i64, i64, i64),
    rotation: &'static str,
    ignore_entities: bool,
    screen: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
}

fn case(label: &str) -> Case {
    Case {
        label: label.to_owned(),
        test: Some("minecraft:example"),
        size: (3, 4, 5),
        rotation: "none",
        ignore_entities: false,
        screen: (427, 240),
        mouse: (0, 0),
        steps: vec![],
    }
}

impl Case {
    fn row(self) -> Value {
        json!({
            "label": self.label, "test": self.test,
            "size": [self.size.0, self.size.1, self.size.2], "rotation": self.rotation,
            "ignore_entities": self.ignore_entities,
            "width": self.screen.0, "height": self.screen.1,
            "mouse": [self.mouse.0, self.mouse.1], "steps": self.steps, "ms": 0,
        })
    }

    fn after(self, steps: Vec<Value>) -> Self {
        Self { steps, ..self }
    }

    fn at(self, mouse: (i64, i64)) -> Self {
        Self { mouse, ..self }
    }

    fn testing(self, test: Option<&'static str>) -> Self {
        Self { test, ..self }
    }

    fn turned(self, rotation: &'static str) -> Self {
        Self { rotation, ..self }
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

const ID: (i64, i64) = (100, 50);
const INFO: (i64, i64) = (100, 100);
const SIZE_X: (i64, i64) = (80, 170);
const SIZE_Y: (i64, i64) = (140, 170);
const SIZE_Z: (i64, i64) = (200, 170);
const ROTATION: (i64, i64) = (270, 170);
const ENTITIES: (i64, i64) = (330, 170);
const RESET: (i64, i64) = (100, 195);
const SAVE: (i64, i64) = (300, 195);
const RUN: (i64, i64) = (100, 220);
const DONE: (i64, i64) = (210, 220);
const CANCEL: (i64, i64) = (320, 220);

const ROTATIONS: [&str; 4] = ["none", "clockwise_90", "clockwise_180", "counterclockwise_90"];

fn opened() -> Vec<Case> {
    let mut rows = vec![
        case("opened"),
        case("opened unnamed").testing(None),
        case("opened namespaced").testing(Some("mod:path/to/test")),
        case("opened colon only").testing(Some(":")),
        case("opened trailing colon").testing(Some("a:")),
        case("opened leading colon").testing(Some(":abc")),
        case("opened empty").testing(Some("")),
        case("opened long").testing(Some("minecraft:a_very_long_test_identifier_that_goes_past_the_end_of_the_box_and_keeps_going_and_going_and_more")),
        Case { ignore_entities: true, ..case("opened ignoring entities") },
        Case { size: (48, 1, 20), ..case("opened sizes") },
        Case { size: (0, 100, -3), ..case("opened odd sizes") },
        Case { size: (123456789, 2, 3), ..case("opened wide size") },
    ];
    for rotation in &ROTATIONS[1..] {
        rows.push(case(&format!("opened {rotation}")).turned(rotation));
    }
    for (label, screen) in [("small", (320, 240)), ("wide", (640, 360)), ("tall", (427, 480)), ("tiny", (200, 150))] {
        rows.push(Case { screen, ..case(&format!("opened {label}")) });
    }
    rows
}

fn hovered() -> Vec<Case> {
    [
        ("id", ID),
        ("info", INFO),
        ("size", SIZE_X),
        ("rotation", ROTATION),
        ("entities", ENTITIES),
        ("reset", RESET),
        ("save", SAVE),
        ("run", RUN),
        ("done", DONE),
        ("cancel", CANCEL),
    ]
    .into_iter()
    .map(|(label, at)| case(&format!("hover {label}")).at(at))
    .collect()
}

fn focus() -> Vec<Case> {
    let mut rows = Vec::new();
    for count in 1..=13 {
        rows.push(case(&format!("tab {count}")).after(vec![key("tab"); count]));
    }
    for count in 1..=4 {
        rows.push(case(&format!("tab back {count}")).after(vec![key("shift+tab"); count]));
    }
    rows.push(case("tab skips disabled save").turned("clockwise_90").after(vec![key("tab"); 9]));
    for (label, keys) in [
        ("down", vec!["down"]),
        ("up", vec!["up"]),
        ("right", vec!["right"]),
        ("down down", vec!["down", "down"]),
        ("tab down", vec!["tab", "down"]),
        ("tab tab down", vec!["tab", "tab", "down"]),
        ("tab tab right", vec!["tab", "tab", "right"]),
        ("tab tab tab right", vec!["tab", "tab", "tab", "right"]),
        ("tab x5 down", vec!["tab", "tab", "tab", "tab", "tab", "down"]),
        ("tab x6 down", vec!["tab", "tab", "tab", "tab", "tab", "tab", "down"]),
        ("tab x6 up", vec!["tab", "tab", "tab", "tab", "tab", "tab", "up"]),
        ("tab x7 down", vec!["tab", "tab", "tab", "tab", "tab", "tab", "tab", "down"]),
        ("tab x8 left", vec!["tab", "tab", "tab", "tab", "tab", "tab", "tab", "tab", "left"]),
        ("tab x9 up", vec!["tab", "tab", "tab", "tab", "tab", "tab", "tab", "tab", "tab", "up"]),
    ] {
        rows.push(case(&format!("arrows {label}")).after(keys.into_iter().map(key).collect()));
    }
    rows.push(case("click info").after(vec![click(INFO)]));
    rows.push(case("click info then type").after(vec![click(INFO), text("abc")]));
    rows.push(case("click info then enter").after(vec![click(INFO), key("enter")]));
    rows.push(case("click size then tab").after(vec![click(SIZE_Y), key("tab")]));
    rows
}

fn identifier() -> Vec<Case> {
    let long = "x".repeat(130);
    vec![
        case("type").after(vec![text("abc")]),
        case("type one").after(vec![text("a")]),
        case("type uppercase").after(vec![text("A")]),
        case("type colon").after(vec![text(":")]),
        case("type second colon").after(vec![text(":x:")]),
        case("type space").after(vec![text(" ")]),
        case("type section sign").after(vec![text("\u{a7}")]),
        case("type limit").after(vec![key("ctrl+a"), text(&long)]),
        case("backspace").after(vec![key("backspace")]),
        case("backspace all").after(vec![key("ctrl+a"), key("backspace")]),
        case("backspace on empty").after(vec![key("ctrl+a"), key("backspace"), key("backspace")]),
        case("delete at end").after(vec![key("delete")]),
        case("delete at home").after(vec![key("home"), key("delete")]),
        case("cursor keys").after(vec![key("left"), key("home"), key("end")]),
        case("select all").after(vec![key("ctrl+a")]),
        case("select all type").after(vec![key("ctrl+a"), text("q")]),
        case("invalid then fixed").after(vec![text("A"), key("backspace")]),
        case("invalid then fixed then done").after(vec![text("A"), key("backspace"), click(DONE)]),
        case("invalid then done").after(vec![text("A"), click(DONE)]),
        case("empty is a name").after(vec![key("ctrl+a"), key("backspace")]),
        case("click in box").after(vec![click((120, 50)), text("Q")]),
        case("click in box then done").after(vec![click((120, 50)), text("q"), click(DONE)]),
        case("type then save").after(vec![text("z"), click(SAVE)]),
        case("type invalid then save").after(vec![text("Z"), click(SAVE)]),
        case("typed in unfocused").after(vec![key("tab"), key("tab"), text("1")]),
    ]
}

fn sizes() -> Vec<Case> {
    let mut rows = Vec::new();
    for (label, value) in [
        ("plus", "+5"),
        ("minus", "-3"),
        ("zero", "0"),
        ("big", "99"),
        ("limit", "48"),
        ("over", "49"),
        ("overflow", "99999999999"),
        ("text", "abc"),
        ("mixed", "5x"),
        ("space", " 7"),
        ("float", "2.5"),
        ("max", "2147483647"),
        ("min", "-2147483648"),
    ] {
        rows.push(case(&format!("size x {label}")).after(vec![click(SIZE_X), key("ctrl+a"), text(value), click(DONE)]));
        rows.push(case(&format!("size z {label}")).after(vec![click(SIZE_Z), key("ctrl+a"), text(value), click(DONE)]));
    }
    rows.extend([
        case("size blank").after(vec![click(SIZE_Y), key("ctrl+a"), key("backspace"), click(DONE)]),
        case("size length limit").after(vec![click(SIZE_Y), key("ctrl+a"), text("1234567890123456789")]),
        case("size no query").after(vec![click(SIZE_Y), text("1")]),
        case("size then run").after(vec![click(SIZE_Y), text("1"), click(RUN)]),
        case("size keyboard").after(vec![key("tab"), key("tab"), key("tab"), key("tab"), text("1")]),
        case("size enter ignored").after(vec![click(SIZE_X), key("enter")]),
    ]);
    rows
}

fn rotating() -> Vec<Case> {
    let mut rows = Vec::new();
    for count in 1..=5 {
        rows.push(case(&format!("rotate {count}")).after(vec![click(ROTATION); count]));
    }
    rows.extend([
        case("rotate back").after(vec![shift_click(ROTATION)]),
        case("rotate back twice").after(vec![shift_click(ROTATION), shift_click(ROTATION)]),
        case("rotate then save").after(vec![click(ROTATION), click(SAVE)]),
        case("rotate full circle then save").after(vec![click(ROTATION); 4].into_iter().chain([click(SAVE)]).collect()),
        case("rotate then done").after(vec![click(ROTATION), click(DONE)]),
        case("rotated block save").turned("clockwise_180").after(vec![click(SAVE)]),
        case("rotated block rotate").turned("clockwise_180").after(vec![click(ROTATION), click(ROTATION)]),
        case("rotated block back").turned("counterclockwise_90").after(vec![click(ROTATION)]),
        case("wheel rotation down").after(vec![wheel(ROTATION, -1)]),
        case("wheel rotation up").after(vec![wheel(ROTATION, 1)]),
        case("wheel elsewhere").after(vec![wheel(ID, -1)]),
        case("wheel entities").after(vec![wheel(ENTITIES, -1)]),
        case("wheel entities up").after(vec![wheel(ENTITIES, 1)]),
        case("wheel rotation then save").after(vec![wheel(ROTATION, -1), click(SAVE)]),
        case("enter on rotation").after(vec![
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("enter"),
        ]),
        case("space on rotation").after(vec![
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("space"),
        ]),
        case("shift enter on rotation").after(vec![
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("shift+enter"),
        ]),
        case("rotate then focus").after(vec![click(ROTATION), key("tab")]),
    ]);
    rows.extend(entities());
    rows
}

fn entities() -> Vec<Case> {
    vec![
        case("click entities").after(vec![click(ENTITIES)]),
        case("click entities twice").after(vec![click(ENTITIES), click(ENTITIES)]),
        case("entities then done").after(vec![click(ENTITIES), click(DONE)]),
        case("ignoring entities then done").after(vec![click(DONE)]),
        Case { ignore_entities: true, ..case("ignoring entities click") }.after(vec![click(ENTITIES), click(RUN)]),
        case("enter on entities").after(vec![
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("tab"),
            key("enter"),
        ]),
    ]
}

fn pressing() -> Vec<Case> {
    let tabs = |count: usize| vec![key("tab"); count];
    vec![
        case("click reset").after(vec![click(RESET)]),
        case("click save").after(vec![click(SAVE)]),
        case("click run").after(vec![click(RUN)]),
        case("click done").after(vec![click(DONE)]),
        case("click cancel").after(vec![click(CANCEL)]),
        case("click done twice").after(vec![click(DONE), click(DONE)]),
        case("click save disabled").turned("clockwise_90").after(vec![click(SAVE)]),
        case("click empty").after(vec![click((5, 5)), click(DONE)]),
        case("click info then done").after(vec![click(INFO), click(DONE)]),
        case("escape").after(vec![key("escape")]),
        case("escape after typing").after(vec![text("a"), key("escape")]),
        case("enter in box").after(vec![key("enter")]),
        case("space in box").after(vec![key("space")]),
        case("enter on reset").after([tabs(8), vec![key("enter")]].concat()),
        case("space on reset").after([tabs(8), vec![key("space")]].concat()),
        case("enter on save").after([tabs(9), vec![key("enter")]].concat()),
        case("enter on run").after([tabs(10), vec![key("enter")]].concat()),
        case("enter on done").after([tabs(11), vec![key("enter")]].concat()),
        case("space on done").after([tabs(11), vec![key("space")]].concat()),
        case("enter on cancel").after([tabs(12), vec![key("enter")]].concat()),
        case("enter on disabled save").turned("clockwise_90").after([tabs(9), vec![key("enter")]].concat()),
        case("enter on info").after(vec![key("tab"), key("enter")]),
        case("resend after reset").after(vec![click(RESET), click(RUN)]),
    ]
}

fn resized() -> Vec<Case> {
    vec![
        case("resize").after(vec![resize(640, 360)]),
        case("resize small").after(vec![resize(320, 240)]),
        case("resize typed").after(vec![text("zzz"), resize(640, 360)]),
        case("resize typed then done").after(vec![text("zzz"), resize(427, 240), click(DONE)]),
        case("resize invalid").after(vec![text("Z"), resize(640, 360)]),
        case("resize rotated").after(vec![click(ROTATION), resize(640, 360)]),
        case("resize rotated then done").after(vec![click(ROTATION), resize(427, 240), click(DONE)]),
        case("resize entities then done").after(vec![click(ENTITIES), resize(427, 240), click(DONE)]),
        case("resize size then done").after(vec![click(SIZE_X), text("9"), resize(427, 240), click(DONE)]),
        case("resize focus").after(vec![key("tab"), key("tab"), resize(640, 360)]),
        case("resize focus type").after(vec![key("tab"), key("tab"), resize(640, 360), text("Z")]),
        case("resize unnamed").testing(None).after(vec![resize(640, 360)]),
        case("resize twice").after(vec![resize(640, 360), resize(320, 240)]),
    ]
}

/// The test instance block's screen: the test, size, rotation and entities of a block, the
/// packets the screen sends on opening, on every edit of the identifier and on each action, the
/// save button that waits for a name that parses and no rotation, the sizes a number is clamped
/// to, and a screen made again at another size.
pub fn test_instance_screen() -> Value {
    let rows = [opened(), hovered(), focus(), identifier(), sizes(), rotating(), pressing(), resized()].concat();
    gui("test-instance-screen", "test_instance_screen", rows.into_iter().map(Case::row).collect())
}
