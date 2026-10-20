//! The command block's editing screen.

use serde_json::{Value, json};

use super::gui;

/// One scenario: a block, what is done to its screen, and the time it is drawn.
#[derive(Clone)]
struct Case {
    label: String,
    mode: &'static str,
    conditional: bool,
    automatic: bool,
    track_output: bool,
    command: &'static str,
    last_output: Option<&'static str>,
    /// The entity id of the command block minecart whose screen it is.
    minecart: Option<i64>,
    size: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
    ms: u64,
}

fn case(label: &str) -> Case {
    Case {
        label: label.to_owned(),
        mode: "redstone",
        conditional: false,
        automatic: false,
        track_output: true,
        command: "say hi",
        last_output: Some("Hello"),
        minecart: None,
        size: (427, 240),
        mouse: (0, 0),
        steps: vec![],
        ms: 0,
    }
}

impl Case {
    fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "mode": self.mode, "conditional": self.conditional,
            "automatic": self.automatic, "track_output": self.track_output,
            "command": self.command, "last_output": self.last_output,
            "width": self.size.0, "height": self.size.1, "mouse": [self.mouse.0, self.mouse.1],
            "steps": self.steps, "ms": self.ms,
        });
        if let Some(entity) = self.minecart {
            row["minecart"] = json!(true);
            row["entity_id"] = json!(entity);
        }
        row
    }
}

fn text(text: &str) -> Value {
    json!({"do": "type", "text": text})
}

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn shift_click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y, "shift": true})
}

fn wheel(x: i64, y: i64, amount: i64) -> Value {
    json!({"do": "wheel", "x": x, "y": y, "amount": amount})
}

fn update() -> Value {
    json!({"do": "update"})
}

fn resize(width: i64, height: i64) -> Value {
    json!({"do": "resize", "width": width, "height": height})
}

/// The buttons of the 427 by 240 screen.
const MODE: (i64, i64) = (100, 175);
/// The output button and the last output's box of the minecart's screen, which sit lower.
const MINECART_OUTPUT: (i64, i64) = (350, 160);
const MINECART_PREVIOUS: (i64, i64) = (200, 160);
const CONDITIONAL: (i64, i64) = (210, 175);
const AUTOMATIC: (i64, i64) = (310, 175);
const OUTPUT: (i64, i64) = (350, 145);
const DONE: (i64, i64) = (100, 202);
const CANCEL: (i64, i64) = (250, 202);
const COMMAND: (i64, i64) = (200, 60);
const PREVIOUS: (i64, i64) = (200, 145);

fn opened() -> Vec<Case> {
    let block = |label: &str, mode, conditional, automatic| Case {
        mode,
        conditional,
        automatic,
        steps: vec![update()],
        ..case(label)
    };
    vec![
        case("asleep"),
        Case { steps: vec![update()], ..case("woken") },
        block("sequence", "sequence", false, false),
        block("auto", "auto", false, true),
        block("conditional", "redstone", true, false),
        block("conditional chain", "sequence", true, true),
        Case { track_output: false, steps: vec![update()], ..case("untracked") },
        Case { track_output: false, ..case("untracked asleep") },
        Case { last_output: None, steps: vec![update()], ..case("no output") },
        Case { command: "", steps: vec![update()], ..case("empty") },
        Case { command: "give @p diamond 64", steps: vec![update()], ..case("woken command") },
        Case { size: (320, 240), steps: vec![update()], ..case("small") },
        Case { size: (640, 360), steps: vec![update()], ..case("wide") },
        Case { size: (427, 480), steps: vec![update()], ..case("tall") },
    ]
}

fn pressed() -> Vec<Case> {
    let after = |label: &str, steps: Vec<Value>| Case { steps: [vec![update()], steps].concat(), ..case(label) };
    let asleep = |label: &str, steps: Vec<Value>| Case { steps, ..case(label) };
    vec![
        after("press mode", vec![click(MODE.0, MODE.1)]),
        after("press mode twice", vec![click(MODE.0, MODE.1), click(MODE.0, MODE.1)]),
        after("press mode round", vec![click(MODE.0, MODE.1), click(MODE.0, MODE.1), click(MODE.0, MODE.1)]),
        after("shift press mode", vec![shift_click(MODE.0, MODE.1)]),
        after("press conditional", vec![click(CONDITIONAL.0, CONDITIONAL.1)]),
        after("press automatic", vec![click(AUTOMATIC.0, AUTOMATIC.1)]),
        after("press output", vec![click(OUTPUT.0, OUTPUT.1)]),
        after("press output twice", vec![click(OUTPUT.0, OUTPUT.1), click(OUTPUT.0, OUTPUT.1)]),
        asleep("press mode asleep", vec![click(MODE.0, MODE.1)]),
        asleep("press output asleep", vec![click(OUTPUT.0, OUTPUT.1)]),
        asleep("press done asleep", vec![click(DONE.0, DONE.1)]),
        after("wheel mode", vec![wheel(MODE.0, MODE.1, -1)]),
        after("wheel mode up", vec![wheel(MODE.0, MODE.1, 1)]),
        after("wheel conditional", vec![wheel(CONDITIONAL.0, CONDITIONAL.1, -1)]),
        after("wheel automatic", vec![wheel(AUTOMATIC.0, AUTOMATIC.1, -1)]),
        after("wheel output", vec![wheel(OUTPUT.0, OUTPUT.1, -1)]),
        after("wheel elsewhere", vec![wheel(COMMAND.0, COMMAND.1, -1)]),
        Case { mouse: MODE, ..after("hover mode", vec![]) },
        Case { mouse: OUTPUT, ..after("hover output", vec![]) },
        Case { mouse: DONE, ..after("hover done", vec![]) },
        Case { mouse: DONE, ..asleep("hover done asleep", vec![]) },
        Case { mouse: CANCEL, ..after("hover cancel", vec![]) },
    ]
}

fn closing() -> Vec<Case> {
    let after = |label: &str, steps: Vec<Value>| Case { steps: [vec![update()], steps].concat(), ..case(label) };
    vec![
        after("done", vec![click(DONE.0, DONE.1)]),
        after("cancel", vec![click(CANCEL.0, CANCEL.1)]),
        after("escape", vec![key("escape")]),
        after("enter", vec![key("enter")]),
        after("done auto", vec![click(MODE.0, MODE.1), click(DONE.0, DONE.1)]),
        after(
            "done toggled",
            vec![click(CONDITIONAL.0, CONDITIONAL.1), click(AUTOMATIC.0, AUTOMATIC.1), click(DONE.0, DONE.1)],
        ),
        after("done untracked", vec![click(OUTPUT.0, OUTPUT.1), click(DONE.0, DONE.1)]),
        after("done typed", vec![text(" now"), click(DONE.0, DONE.1)]),
        Case { track_output: false, ..after("done untracked block", vec![click(DONE.0, DONE.1)]) },
        Case {
            mode: "sequence",
            conditional: true,
            automatic: true,
            ..after("done chain", vec![click(DONE.0, DONE.1)])
        },
        after("enter typed", vec![key("ctrl+a"), text("tp @s 0 0 0"), key("enter")]),
        Case { steps: vec![click(DONE.0, DONE.1)], ..case("done asleep") },
        Case { steps: vec![key("enter")], ..case("enter asleep") },
        after("tab to done enter", vec![key("shift+tab"), key("shift+tab"), key("enter")]),
    ]
}

fn typing() -> Vec<Case> {
    let after = |label: &str, steps: Vec<Value>| Case { steps: [vec![update()], steps].concat(), ..case(label) };
    vec![
        after("type", vec![text(" there")]),
        after("type slash", vec![key("ctrl+a"), text("/say hi")]),
        after("select all typed", vec![key("ctrl+a"), text("new")]),
        after("backspace", vec![key("backspace"), key("backspace")]),
        after("home", vec![key("home"), text("x")]),
        after("word left", vec![key("ctrl+left"), text("X")]),
        after("word erased", vec![key("ctrl+backspace")]),
        after("select left", vec![key("shift+left"), key("shift+left")]),
        after("delete inside", vec![key("home"), key("delete")]),
        after("type before data", vec![text("x")]),
        Case { steps: vec![text("abc"), update()], ..case("update replaces") },
        after("click in command", vec![click(80, COMMAND.1), text("Z")]),
        after("click far in command", vec![click(300, COMMAND.1), text("Z")]),
        after("click previous", vec![click(PREVIOUS.0, PREVIOUS.1), text("Z")]),
        after("type in previous", vec![click(PREVIOUS.0, PREVIOUS.1), key("backspace")]),
        after("previous select", vec![click(PREVIOUS.0, PREVIOUS.1), key("ctrl+a")]),
        Case {
            command: "teleport @a ~ ~ ~",
            ..after("long command", vec![text(" and some further words that run on")])
        },
        after("selector", vec![key("ctrl+a"), text("kill @")]),
        after("slash alone", vec![key("ctrl+a"), text("/")]),
        after("section sign", vec![text("a\u{a7}b")]),
    ]
}

fn focus() -> Vec<Case> {
    let after = |label: &str, steps: Vec<Value>| Case { steps: [vec![update()], steps].concat(), ..case(label) };
    vec![
        after("tab 1", vec![key("tab")]),
        after("tab 2", vec![key("tab"), key("tab")]),
        after("tab 3", vec![key("tab"), key("tab"), key("tab")]),
        after("tab 4", vec![key("tab"), key("tab"), key("tab"), key("tab")]),
        after("tab 5", vec![key("tab"), key("tab"), key("tab"), key("tab"), key("tab")]),
        after("tab 6", vec![key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), key("tab")]),
        after("tab 7", vec![key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), key("tab"), key("tab")]),
        after("tab round", vec![key("tab"); 8]),
        after("tab back", vec![key("shift+tab")]),
        after("tab back twice", vec![key("shift+tab"), key("shift+tab")]),
        after("tab asleep", vec![key("tab"), key("tab"), key("tab")]),
        after("tab to output space", vec![key("tab"), key("tab"), key("space")]),
        after("tab to mode space", vec![key("tab"), key("tab"), key("tab"), key("space")]),
        after("tab to mode enter", vec![key("tab"), key("tab"), key("tab"), key("enter")]),
        after("arrow down", vec![key("down")]),
        after("arrow up", vec![key("up")]),
        after("arrow down from previous", vec![key("tab"), key("down")]),
        after("arrow right from mode", vec![key("tab"), key("tab"), key("tab"), key("right")]),
        after("arrow left from conditional", vec![key("tab"), key("tab"), key("tab"), key("tab"), key("left")]),
        after("arrow up from done", vec![key("shift+tab"), key("shift+tab"), key("up")]),
        after("arrow right from done", vec![key("shift+tab"), key("shift+tab"), key("right")]),
        after("arrow left from cancel", vec![key("shift+tab"), key("left")]),
        Case { steps: vec![key("tab"), key("tab"), key("tab")], ..case("tab asleep woken not") },
    ]
}

fn resized() -> Vec<Case> {
    let after = |label: &str, steps: Vec<Value>| Case { steps: [vec![update()], steps].concat(), ..case(label) };
    vec![
        after("resize", vec![resize(640, 360)]),
        after("resize small", vec![resize(320, 240)]),
        after("resize keeps text", vec![text(" more"), resize(640, 360)]),
        after("resize keeps values", vec![click(MODE.0, MODE.1), click(AUTOMATIC.0, AUTOMATIC.1), resize(500, 300)]),
        Case { steps: vec![resize(640, 360)], ..case("resize wakes") },
        Case { steps: vec![text("abc"), resize(640, 360)], ..case("resize asleep keeps text") },
        after("resize untracked", vec![click(OUTPUT.0, OUTPUT.1), resize(640, 360)]),
    ]
}

fn minecart() -> Vec<Case> {
    let cart = |label: &str, steps: Vec<Value>| Case { minecart: Some(7), steps, ..case(label) };
    let at = |label: &str, mouse: (i64, i64)| Case { mouse, ..cart(label, vec![]) };
    let with = |label: &str, change: fn(Case) -> Case| change(cart(label, vec![]));
    let tabs = |count: usize| vec![key("tab"); count];
    vec![
        cart("opened", vec![]),
        with("no output", |case| Case { last_output: None, ..case }),
        with("untracked", |case| Case { track_output: false, ..case }),
        with("empty", |case| Case { command: "", ..case }),
        with("long", |case| Case { command: "teleport @a ~ ~ ~ and some further words", ..case }),
        with("small", |case| Case { size: (320, 240), ..case }),
        with("wide", |case| Case { size: (640, 360), ..case }),
        with("tall", |case| Case { size: (427, 480), ..case }),
        cart("press output", vec![click(MINECART_OUTPUT.0, MINECART_OUTPUT.1)]),
        cart("press output twice", vec![click(MINECART_OUTPUT.0, MINECART_OUTPUT.1); 2]),
        cart("wheel output", vec![wheel(MINECART_OUTPUT.0, MINECART_OUTPUT.1, -1)]),
        cart("press where mode was", vec![click(MODE.0, MODE.1)]),
        cart("wheel where mode was", vec![wheel(MODE.0, MODE.1, -1)]),
        at("hover output", MINECART_OUTPUT),
        at("hover done", DONE),
        at("hover cancel", CANCEL),
        cart("done", vec![click(DONE.0, DONE.1)]),
        cart("cancel", vec![click(CANCEL.0, CANCEL.1)]),
        cart("escape", vec![key("escape")]),
        cart("enter", vec![key("enter")]),
        cart("done untracked", vec![click(MINECART_OUTPUT.0, MINECART_OUTPUT.1), click(DONE.0, DONE.1)]),
        cart("done typed", vec![text(" now"), click(DONE.0, DONE.1)]),
        cart("enter replaced", vec![key("ctrl+a"), text("tp @s 0 0 0"), key("enter")]),
        Case { minecart: Some(300), steps: vec![click(DONE.0, DONE.1)], ..case("done another cart") },
        cart("type", vec![text(" there")]),
        cart("type slash", vec![key("ctrl+a"), text("/say hi")]),
        cart("click in command", vec![click(80, COMMAND.1), text("Z")]),
        cart("click previous", vec![click(MINECART_PREVIOUS.0, MINECART_PREVIOUS.1), text("Z")]),
        cart("previous select", vec![click(MINECART_PREVIOUS.0, MINECART_PREVIOUS.1), key("ctrl+a")]),
        cart("click between", vec![click(200, 100)]),
        cart("tab 1", tabs(1)),
        cart("tab 2", tabs(2)),
        cart("tab 3", tabs(3)),
        cart("tab 4", tabs(4)),
        cart("tab round", tabs(5)),
        cart("tab back", vec![key("shift+tab")]),
        cart("tab back twice", vec![key("shift+tab"), key("shift+tab")]),
        cart("tab to output space", vec![key("tab"), key("tab"), key("space")]),
        cart("tab to done enter", vec![key("tab"), key("tab"), key("tab"), key("enter")]),
        cart("arrow down", vec![key("down")]),
        cart("arrow down from previous", vec![key("tab"), key("down")]),
        cart("arrow right from output", vec![key("tab"), key("tab"), key("right")]),
        cart("arrow down from output", vec![key("tab"), key("tab"), key("down")]),
        cart("arrow up from done", vec![key("shift+tab"), key("shift+tab"), key("up")]),
        cart("arrow left from cancel", vec![key("shift+tab"), key("left")]),
        cart("resize", vec![resize(640, 360)]),
        cart("resize keeps text", vec![text(" more"), resize(640, 360)]),
        cart("resize keeps output", vec![click(MINECART_OUTPUT.0, MINECART_OUTPUT.1), resize(500, 300)]),
    ]
}

/// The command block's screen: the box and the buttons asleep and woken by the block's data, each
/// mode, the condition and the trigger, the output shown or a dash, the buttons pressed, scrolled
/// and tabbed to, Done, Cancel, Escape and Enter with what they send, the box typed into,
/// and the screen made again at another size; then the command block minecart's screen, the same
/// without the three extra buttons, a little lower and awake from the start.
pub fn command_block_screen() -> Value {
    let rows = [opened(), pressed(), closing(), typing(), focus(), resized(), minecart()].concat();
    gui("command-block-screen", "command_block_screen", rows.into_iter().map(Case::row).collect())
}
