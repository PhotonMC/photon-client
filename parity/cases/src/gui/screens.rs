//! The pause, death and disconnected screens.

use serde_json::{Value, json};

use super::gui;

fn frame(ms: i64) -> Value {
    json!({"do": "frame", "ms": ms})
}

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

fn keys(name: &str, count: usize) -> Vec<Value> {
    vec![key(name); count]
}

fn pause(label: &str, (width, height): (i64, i64), mouse: (i64, i64), steps: Vec<Value>, ms: i64) -> Value {
    json!({"label": label, "width": width, "height": height, "mouse": [mouse.0, mouse.1], "steps": steps, "ms": ms})
}

const SIZE: (i64, i64) = (427, 240);
const ICONS: [(&str, (i64, i64)); 4] =
    [("report", (170, 120)), ("feedback", (193, 120)), ("friends", (215, 120)), ("reporting", (237, 120))];

fn hover_rows() -> Vec<Value> {
    let mut rows = vec![
        pause("default", SIZE, (0, 0), vec![], 0),
        pause("small", (320, 240), (0, 0), vec![], 0),
        pause("wide", (640, 360), (0, 0), vec![], 0),
        pause("tiny", (200, 150), (0, 0), vec![], 0),
    ];
    let buttons = [
        ("return", (213, 60)),
        ("advancements", (160, 90)),
        ("statistics", (265, 90)),
        ("report", (170, 120)),
        ("feedback", (193, 120)),
        ("friends", (215, 120)),
        ("reporting", (237, 120)),
        ("options", (213, 150)),
        ("disconnect", (213, 180)),
    ];
    for (name, point) in buttons {
        rows.push(pause(&format!("hover-{name}"), SIZE, point, vec![], 0));
    }
    // Tooltips: shown once the widget has been hovered a moment (a frame
    // earlier), and not on the first frame of it.
    for (name, point) in ICONS {
        rows.push(pause(&format!("tip-{name}"), SIZE, point, vec![frame(0)], 10));
        rows.push(pause(&format!("tip-first-frame-{name}"), SIZE, point, vec![], 0));
    }
    let tips = [
        ("tip-late", SIZE, (170, 120), vec![frame(0), frame(500)], 1000),
        ("tip-small-screen", (320, 240), (150, 120), vec![frame(0)], 10),
        ("tip-tiny-screen", (200, 150), (100, 95), vec![frame(0)], 10),
        ("tip-wide-screen", (640, 360), (270, 175), vec![frame(0)], 10),
        ("tip-edge-left", SIZE, (166, 112), vec![frame(0)], 10),
        ("tip-edge-bottom", SIZE, (170, 138), vec![frame(0)], 10),
        ("tip-edge-top", SIZE, (170, 101), vec![frame(0)], 10),
        ("tip-moves-between", SIZE, (193, 120), vec![frame(0), frame(5)], 10),
        ("tip-not-on-button", SIZE, (160, 90), vec![frame(0)], 10),
        ("tip-on-wide-button", SIZE, (213, 60), vec![frame(0)], 10),
    ];
    rows.extend(tips.into_iter().map(|(label, size, mouse, steps, ms)| pause(label, size, mouse, steps, ms)));
    rows
}

/// `count` presses of `name`, then a frame.
fn tipped(name: &str, count: usize) -> Vec<Value> {
    [keys(name, count), vec![frame(0)]].concat()
}

/// Keyboard focus: Tab and the arrows, then the tooltip below the widget.
fn focus_rows() -> Vec<Value> {
    let steps = |label: &str, steps: Vec<Value>, ms: i64| pause(label, SIZE, (0, 0), steps, ms);
    vec![
        steps("tab-1", keys("tab", 1), 0),
        steps("tab-2", keys("tab", 2), 0),
        steps("tab-4-icon", keys("tab", 4), 0),
        steps("tab-4-icon-tip", tipped("tab", 4), 10),
        steps("tab-5-icon-tip", tipped("tab", 5), 10),
        steps("tab-6-icon-tip", tipped("tab", 6), 10),
        steps("tab-7-icon-tip", tipped("tab", 7), 10),
        pause("tab-then-hover-tip", SIZE, (237, 120), tipped("tab", 4), 10),
        pause("tab-icon-small-tip", (320, 240), (0, 0), tipped("tab", 4), 10),
        pause("tab-icon-tiny-tip", (200, 150), (0, 0), tipped("tab", 4), 10),
        steps("shift-tab-tip", tipped("shift+tab", 4), 10),
        steps("arrow-down", keys("down", 1), 0),
        steps("arrow-up", keys("up", 1), 0),
        steps("arrow-down-twice", keys("down", 2), 0),
        steps("arrow-down-thrice", keys("down", 3), 0),
        steps("arrow-down-icons-tip", tipped("down", 3), 10),
        steps("arrow-right-from-first", vec![key("down"), key("right")], 0),
        steps("arrow-grid", vec![key("down"), key("down"), key("right"), key("down")], 0),
    ]
}

fn arrow_rows() -> Vec<Value> {
    let steps = |label: &str, steps: Vec<Value>, ms: i64| pause(label, SIZE, (0, 0), steps, ms);
    let three_down = || keys("down", 3);
    vec![
        steps("arrow-icons-across", [three_down(), vec![key("right"), key("right"), frame(0)]].concat(), 10),
        steps("arrow-icons-left", [three_down(), vec![key("right"), key("left"), frame(0)]].concat(), 10),
        steps("arrow-down-from-icons", keys("down", 4), 0),
        steps("arrow-down-to-disconnect", keys("down", 6), 0),
        steps("arrow-down-past", keys("down", 8), 0),
        steps("arrow-up-from-disconnect", [keys("down", 6), vec![key("up")]].concat(), 0),
        steps("arrow-up-to-return", [three_down(), keys("up", 3)].concat(), 0),
        steps("arrow-left-from-nothing", vec![key("left")], 0),
        steps("arrow-right-from-nothing", vec![key("right")], 0),
        pause("arrow-small", (320, 240), (0, 0), [three_down(), vec![key("right")]].concat(), 0),
        pause("arrow-wide", (640, 360), (0, 0), vec![key("down"), key("right"), key("down")], 0),
        pause("arrow-tiny", (200, 150), (0, 0), [three_down(), vec![key("right"), key("down")]].concat(), 0),
    ]
}

/// The pause menu in screens of several sizes, with the mouse away and over
/// each of its buttons, its tooltips and keyboard focus.
/// The blur behind the menu is asked for from a blurriness of one (`Screen.extractBlurredBackground`).
fn blur_rows() -> Vec<Value> {
    [("blur-0", 0), ("blur-1", 1), ("blur-10", 10)]
        .into_iter()
        .map(|(label, blurriness)| {
            let mut row = pause(label, SIZE, (0, 0), vec![], 0);
            row["blurriness"] = json!(blurriness);
            row
        })
        .collect()
}

pub fn pause_screen() -> Value {
    gui("pause-screen", "pause_screen", [hover_rows(), focus_rows(), arrow_rows(), blur_rows()].concat())
}

/// One death screen scenario.
struct Death {
    size: (i64, i64),
    mouse: (i64, i64),
    ticks: i64,
    hardcore: bool,
    cause: Option<&'static str>,
    score: i64,
}

const DEATH: Death =
    Death { size: SIZE, mouse: (0, 0), ticks: 0, hardcore: false, cause: Some("Steve was slain by Zombie"), score: 0 };

fn death(label: &str, death: Death) -> Value {
    let mut row = json!({
        "label": label, "width": death.size.0, "height": death.size.1, "mouse": [death.mouse.0, death.mouse.1],
        "ticks": death.ticks, "hardcore": death.hardcore, "score": death.score,
    });
    if let Some(cause) = death.cause {
        row["cause"] = json!(cause);
    }
    row
}

/// The death screen: before and after its buttons wake (20 ticks), with and
/// without a cause, hardcore, scores of several widths, and the mouse over
/// each button.
pub fn death_screen() -> Value {
    let awake = Death { ticks: 20, ..DEATH };
    let long = "Alex was blown up by Creeper whilst fighting Skeleton and more";
    let rows = vec![
        death("fresh", DEATH),
        death("awake", Death { ..awake }),
        death("one-short", Death { ticks: 19, ..DEATH }),
        death("late", Death { ticks: 80, ..DEATH }),
        death("no-cause", Death { cause: None, ..awake }),
        death("hardcore", Death { hardcore: true, ..awake }),
        death("big-score", Death { score: 123456, ..awake }),
        death("small-screen", Death { size: (320, 240), ..awake }),
        death("wide", Death { size: (854, 480), ..awake }),
        death("long-cause", Death { cause: Some(long), ..awake }),
        death("hover-respawn-asleep", Death { mouse: (213, 132), ..DEATH }),
        death("hover-respawn", Death { mouse: (213, 132), ..awake }),
        death("hover-title", Death { mouse: (213, 156), ..awake }),
        death("hover-elsewhere", Death { mouse: (10, 10), ..awake }),
    ];
    gui("death-screen", "death_screen", rows)
}

/// The disconnected screen for short and long reasons (wrapped at the
/// screen's width less 50), in several screen sizes, and with the mouse
/// over the button.
pub fn disconnected_screen() -> Value {
    let row = |label: &str, reason: &str, (width, height): (i64, i64), mouse: (i64, i64)| json!({"label": label, "width": width, "height": height, "mouse": [mouse.0, mouse.1], "reason": reason});
    let long = "You are not whitelisted on this server and the reason is long enough to wrap ".repeat(3);
    let rows = vec![
        row("short", "Disconnected", SIZE, (0, 0)),
        row("kicked", "Kicked for flying", SIZE, (0, 0)),
        row("long", &long, SIZE, (0, 0)),
        row("small", "Timed out", (320, 240), (0, 0)),
        row("wide", &long, (854, 480), (0, 0)),
        row("tiny", &long, (200, 150), (0, 0)),
        row("empty", "", SIZE, (0, 0)),
        row("lines", "one\ntwo\nthree", SIZE, (0, 0)),
        row("hover", "Disconnected", SIZE, (213, 140)),
    ];
    gui("disconnected-screen", "disconnected_screen", rows)
}
