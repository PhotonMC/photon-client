//! The death and disconnected screens.

use serde_json::{Value, json};

use super::gui;

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

const SIZE: (i64, i64) = (427, 240);

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

/// Presses of the buttons: asleep they stay silent, awake they click. "Respawn" is left out: Java
/// asks the player to respawn, which needs a live connection, and the button goes off after that.
fn press_rows(awake: Death) -> Vec<Value> {
    let click = |x: i64, y: i64| json!({"do": "click", "x": x, "y": y});
    let with = |label: &str, death: Death, steps: Vec<Value>| {
        let mut row = self::death(label, death);
        row["steps"] = json!(steps);
        row
    };
    vec![
        with("click-title-asleep", Death { ticks: 19, ..DEATH }, vec![click(213, 156)]),
        with("click-title", Death { mouse: (213, 156), ..awake }, vec![click(213, 156)]),
        with("click-respawn-asleep", Death { ticks: 19, ..DEATH }, vec![click(213, 132)]),
        with("click-elsewhere", Death { ..awake }, vec![click(10, 10)]),
        with("click-title-hardcore", Death { hardcore: true, ..awake }, vec![click(213, 156)]),
        with(
            "tab-enter-asleep",
            Death { ticks: 19, ..DEATH },
            vec![json!({"do": "key", "key": "tab"}), json!({"do": "key", "key": "enter"})],
        ),
        with(
            "tab-tab-enter",
            Death { ..awake },
            vec![
                json!({"do": "key", "key": "tab"}),
                json!({"do": "key", "key": "tab"}),
                json!({"do": "key", "key": "enter"}),
            ],
        ),
        with("enter-nothing-focused", Death { ..awake }, vec![json!({"do": "key", "key": "enter"})]),
    ]
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
    gui("death-screen", "death_screen", [rows, press_rows(awake)].concat())
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
    let press = |label: &str, size: (i64, i64), steps: Vec<Value>| {
        let mut row = row(label, "Disconnected", size, (0, 0));
        row["steps"] = json!(steps);
        row
    };
    let click = |x: i64, y: i64| json!({"do": "click", "x": x, "y": y});
    let rows = [
        rows,
        vec![
            press("click-button", SIZE, vec![click(213, 140)]),
            press("click-elsewhere", SIZE, vec![click(10, 10)]),
            press("click-small", (320, 240), vec![click(160, 140)]),
            press("enter-nothing-focused", SIZE, vec![key("enter")]),
            press("tab-enter", SIZE, vec![key("tab"), key("enter")]),
            press("tab-tab-enter", SIZE, vec![key("tab"), key("tab"), key("enter")]),
        ],
    ]
    .concat();
    gui("disconnected-screen", "disconnected_screen", rows)
}
