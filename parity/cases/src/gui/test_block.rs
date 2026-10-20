//! The test block's editing screen.

use serde_json::{Value, json};

use super::gui;

/// One scenario: a block, what is done to its screen, and the time it is drawn.
#[derive(Clone)]
struct Case {
    label: String,
    mode: &'static str,
    message: String,
    screen: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
}

fn case(label: &str, mode: &'static str) -> Case {
    Case {
        label: label.to_owned(),
        mode,
        message: "hello there".to_owned(),
        screen: (427, 240),
        mouse: (0, 0),
        steps: vec![],
    }
}

impl Case {
    fn row(self) -> Value {
        json!({
            "label": self.label, "mode": self.mode, "message": self.message,
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

    fn saying(self, message: &str) -> Self {
        Self { message: message.to_owned(), ..self }
    }
}

const MODES: [&str; 4] = ["start", "log", "fail", "accept"];

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

const MESSAGE: (i64, i64) = (181, 90);
const MODE: (i64, i64) = (84, 195);
const DONE: (i64, i64) = (134, 220);
const CANCEL: (i64, i64) = (292, 220);

fn opened() -> Vec<Case> {
    let long = "a long message that goes on past the end of its box and keeps going and going";
    let mut rows = Vec::new();
    for mode in MODES {
        rows.push(case(&format!("opened {mode}"), mode));
        rows.push(case(&format!("opened {mode} empty"), mode).saying(""));
        rows.push(case(&format!("opened {mode} long"), mode).saying(long));
        rows.push(case(&format!("opened {mode} limit"), mode).saying(&"m".repeat(130)));
        for (label, screen) in [("small", (320, 240)), ("wide", (640, 360)), ("tall", (427, 480))] {
            rows.push(Case { screen, ..case(&format!("opened {mode} {label}"), mode) });
        }
    }
    rows
}

fn hovered() -> Vec<Case> {
    let mut rows = Vec::new();
    for mode in ["start", "log"] {
        for (label, at) in [("message", MESSAGE), ("mode", MODE), ("done", DONE), ("cancel", CANCEL)] {
            rows.push(case(&format!("hover {label} {mode}"), mode).at(at));
        }
    }
    rows
}

fn focus() -> Vec<Case> {
    let mut rows = Vec::new();
    for mode in MODES {
        for count in 1..=5 {
            rows.push(case(&format!("tab {count} {mode}"), mode).after(vec![key("tab"); count]));
        }
        for count in 1..=3 {
            rows.push(case(&format!("tab back {count} {mode}"), mode).after(vec![key("shift+tab"); count]));
        }
        for (label, keys) in [
            ("down", vec!["down"]),
            ("up", vec!["up"]),
            ("right", vec!["right"]),
            ("down down", vec!["down", "down"]),
            ("tab down up", vec!["tab", "down", "up"]),
            ("tab right left", vec!["tab", "right", "left"]),
            ("tab tab right", vec!["tab", "tab", "right"]),
            ("tab tab tab right", vec!["tab", "tab", "tab", "right"]),
            ("tab tab tab tab left", vec!["tab", "tab", "tab", "tab", "left"]),
        ] {
            rows.push(case(&format!("arrows {label} {mode}"), mode).after(keys.into_iter().map(key).collect()));
        }
        rows.push(case(&format!("tab then click message {mode}"), mode).after(vec![key("tab"), click(MESSAGE)]));
    }
    rows
}

fn pressing() -> Vec<Case> {
    let tabs = |count: usize| vec![key("tab"); count];
    let mut rows = Vec::new();
    for mode in MODES {
        let first = usize::from(mode != "start");
        rows.extend([
            case(&format!("enter in box {mode}"), mode).after(vec![key("enter")]),
            case(&format!("space in box {mode}"), mode).after(vec![key("space")]),
            case(&format!("escape {mode}"), mode).after(vec![key("escape")]),
            case(&format!("enter on mode {mode}"), mode).after([tabs(first + 1), vec![key("enter")]].concat()),
            case(&format!("space on mode {mode}"), mode).after([tabs(first + 1), vec![key("space")]].concat()),
            case(&format!("shift enter on mode {mode}"), mode)
                .after([tabs(first + 1), vec![key("shift+enter")]].concat()),
            case(&format!("enter on done {mode}"), mode).after([tabs(first + 2), vec![key("enter")]].concat()),
            case(&format!("space on done {mode}"), mode).after([tabs(first + 2), vec![key("space")]].concat()),
            case(&format!("enter on cancel {mode}"), mode).after([tabs(first + 3), vec![key("enter")]].concat()),
            case(&format!("click done {mode}"), mode).after(vec![click(DONE)]),
            case(&format!("click cancel {mode}"), mode).after(vec![click(CANCEL)]),
            case(&format!("click done twice {mode}"), mode).after(vec![click(DONE), click(DONE)]),
            case(&format!("click empty {mode}"), mode).after(vec![click((5, 5)), click(DONE)]),
        ]);
    }
    rows
}

fn cycling() -> Vec<Case> {
    let mut rows = Vec::new();
    for mode in MODES {
        for count in 1..=4 {
            rows.push(case(&format!("click mode {count} {mode}"), mode).after(vec![click(MODE); count]));
        }
        rows.extend([
            case(&format!("shift click mode {mode}"), mode).after(vec![shift_click(MODE)]),
            case(&format!("shift click mode twice {mode}"), mode).after(vec![shift_click(MODE), shift_click(MODE)]),
            case(&format!("wheel down mode {mode}"), mode).after(vec![wheel(MODE, -1)]),
            case(&format!("wheel up mode {mode}"), mode).after(vec![wheel(MODE, 1)]),
            case(&format!("wheel elsewhere {mode}"), mode).after(vec![wheel(MESSAGE, -1)]),
            case(&format!("wheel then done {mode}"), mode).after(vec![wheel(MODE, -1), click(DONE)]),
            case(&format!("wheel then type {mode}"), mode).after(vec![wheel(MODE, -1), text("ab")]),
            case(&format!("wheel then tab {mode}"), mode).after(vec![wheel(MODE, -1), key("tab")]),
            case(&format!("wheel then arrow {mode}"), mode).after(vec![wheel(MODE, -1), key("down")]),
            case(&format!("click mode then type {mode}"), mode).after(vec![click(MODE), text("ab")]),
            case(&format!("click mode then tab {mode}"), mode).after(vec![click(MODE), key("tab")]),
            case(&format!("click mode then done {mode}"), mode).after(vec![click(MODE), click(DONE)]),
        ]);
    }
    rows
}

fn typing() -> Vec<Case> {
    let long = "a long message that goes on past the end of its box and keeps going and going";
    let mut rows = Vec::new();
    for mode in ["log", "accept"] {
        let typed = |label: &str, steps: Vec<Value>| case(&format!("{label} {mode}"), mode).after(steps);
        rows.extend([
            typed("type", vec![text("abc")]),
            typed("type then done", vec![text("abc"), click(DONE)]),
            typed("type limit", vec![text(&"z".repeat(140))]),
            typed("type uppercase", vec![text("ABC \u{a7} x")]),
            typed("backspace", vec![key("backspace"), key("backspace")]),
            typed("backspace then done", vec![key("backspace"), click(DONE)]),
            typed("select all", vec![key("ctrl+a")]),
            typed("select all type", vec![key("ctrl+a"), text("q")]),
            typed("select all done", vec![key("ctrl+a"), key("backspace"), click(DONE)]),
            typed("home", vec![key("home")]),
            typed("home type", vec![key("home"), text("x")]),
            typed("shift end", vec![key("home"), key("shift+end")]),
            typed("shift left", vec![key("shift+left"), key("shift+left")]),
            typed("click in box", vec![click((100, 90)), text("Q")]),
            typed("typed then tab then type", vec![text("a"), key("tab"), text("b")]),
            typed("typed then back to box", vec![text("a"), key("tab"), key("shift+tab"), text("b")]),
            typed("arrow keys in box", vec![key("left"), key("left"), text("-")]),
        ]);
        rows.extend([
            case(&format!("long select all {mode}"), mode).saying(long).after(vec![key("ctrl+a")]),
            case(&format!("long home {mode}"), mode).saying(long).after(vec![key("home")]),
            case(&format!("long shift left {mode}"), mode)
                .saying(long)
                .after(vec![key("shift+left"), key("shift+left")]),
            case(&format!("long click {mode}"), mode).saying(long).after(vec![click((200, 90)), text("zz")]),
        ]);
    }
    rows.extend([
        case("type in hidden box", "start").after(vec![text("abc")]),
        case("type in hidden box then cycle", "start").after(vec![text("abc"), click(MODE)]),
        case("click in hidden box", "start").after(vec![click(MESSAGE), text("abc")]),
        case("type then start then done", "log").after(vec![
            text("abc"),
            click(MODE),
            click(MODE),
            click(MODE),
            click(DONE),
        ]),
        case("type then start", "log").after(vec![text("abc"), click(MODE), click(MODE), click(MODE)]),
        case("start then done keeps message", "start").after(vec![click(DONE)]),
        case("start then cycle then type", "start").after(vec![click(MODE), click(MESSAGE), text("zz")]),
        case("start then cycle then tab", "start").after(vec![click(MODE), key("tab")]),
        case("start tab type", "start").after(vec![key("tab"), key("tab"), key("shift+tab"), text("t")]),
    ]);
    rows
}

fn resized() -> Vec<Case> {
    let mut rows = Vec::new();
    for mode in ["start", "log", "fail"] {
        rows.extend([
            case(&format!("resize {mode}"), mode).after(vec![resize(640, 360)]),
            case(&format!("resize small {mode}"), mode).after(vec![resize(320, 240)]),
            case(&format!("resize typed {mode}"), mode).after(vec![text("zzz"), resize(640, 360)]),
            case(&format!("resize typed then done {mode}"), mode).after(vec![
                text("zzz"),
                resize(427, 240),
                click(DONE),
            ]),
            case(&format!("resize mode kept {mode}"), mode).after(vec![click(MODE), resize(640, 360)]),
            case(&format!("resize mode kept then done {mode}"), mode).after(vec![
                click(MODE),
                resize(427, 240),
                click(DONE),
            ]),
            case(&format!("resize focus {mode}"), mode).after(vec![key("tab"), key("tab"), resize(640, 360)]),
            case(&format!("resize focus type {mode}"), mode).after(vec![
                key("tab"),
                key("tab"),
                resize(640, 360),
                text("Z"),
            ]),
            case(&format!("resize type message {mode}"), mode).after(vec![
                text("zzz"),
                click(MODE),
                resize(640, 360),
                text("y"),
            ]),
        ]);
    }
    rows
}

/// The test block's screen: the mode and message of a block in each mode, what the keys, clicks
/// and wheel do (Enter in the box does nothing), the box that a start block hides, what Done sends
/// and a screen made again at another size, which gives the message back as the block had it.
pub fn test_block_screen() -> Value {
    let rows = [opened(), hovered(), focus(), pressing(), cycling(), typing(), resized()].concat();
    gui("test-block-screen", "test_block_screen", rows.into_iter().map(Case::row).collect())
}
