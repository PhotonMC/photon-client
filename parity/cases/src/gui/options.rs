//! The options screen.

use serde_json::{Value, json};

use super::gui;
use super::setting::key;

/// One options screen scenario.
struct Options {
    size: (i64, i64),
    mouse: (i64, i64),
    in_world: bool,
    fov: i64,
    steps: Vec<Value>,
}

fn options() -> Options {
    Options { size: (427, 240), mouse: (0, 0), in_world: true, fov: 70, steps: vec![] }
}

impl Options {
    fn row(self, label: &str) -> Value {
        json!({
            "label": label, "width": self.size.0, "height": self.size.1, "mouse": [self.mouse.0, self.mouse.1],
            "in_world": self.in_world, "fov": self.fov, "steps": self.steps,
        })
    }
}

fn steps(steps: Vec<Value>) -> Options {
    Options { steps, ..options() }
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn drag(x: i64, y: i64) -> Value {
    json!({"do": "drag", "x": x, "y": y})
}

fn release(x: i64, y: i64) -> Value {
    json!({"do": "release", "x": x, "y": y})
}

fn tabs(count: usize) -> Vec<Value> {
    vec![key("tab"); count]
}

fn layout_rows() -> Vec<Value> {
    let hover = |x: i64, y: i64| Options { mouse: (x, y), ..options() };
    vec![
        options().row("in-world"),
        Options { in_world: false, ..options() }.row("out-of-world"),
        Options { size: (320, 240), ..options() }.row("small"),
        Options { size: (640, 360), ..options() }.row("large"),
        Options { size: (200, 150), ..options() }.row("tiny"),
        Options { size: (854, 480), in_world: false, ..options() }.row("wide"),
        Options { size: (427, 200), ..options() }.row("short"),
        Options { fov: 30, ..options() }.row("fov-min"),
        Options { fov: 110, ..options() }.row("fov-max"),
        Options { fov: 90, ..options() }.row("fov-90"),
        hover(100, 38).row("hover-fov"),
        Options { in_world: false, ..hover(300, 38) }.row("hover-online"),
        hover(300, 38).row("hover-world-options"),
        hover(100, 95).row("hover-skin"),
        hover(300, 119).row("hover-controls"),
        hover(100, 190).row("hover-telemetry-asleep"),
        hover(200, 220).row("hover-done"),
        hover(212, 95).row("hover-gap"),
        hover(5, 5).row("hover-elsewhere"),
    ]
}

fn key_rows() -> Vec<Value> {
    let back = || key("shift+tab");
    vec![
        steps(tabs(1)).row("tab-1"),
        steps(tabs(2)).row("tab-2"),
        steps(tabs(3)).row("tab-3"),
        steps(tabs(9)).row("tab-9"),
        steps(tabs(10)).row("tab-10-past-telemetry"),
        steps(tabs(12)).row("tab-12-done"),
        steps(tabs(13)).row("tab-13-round"),
        steps(vec![back()]).row("shift-tab"),
        steps(vec![back(); 2]).row("shift-tab-2"),
        steps(vec![key("tab"), key("tab"), back()]).row("tab-then-back"),
        steps(vec![key("tab"), back()]).row("tab-back-round"),
        steps(vec![key("tab"), key("right"), key("right"), key("right")]).row("keys-right"),
        Options { fov: 40, ..steps(vec![key("tab"), key("left"), key("left"), key("left")]) }.row("keys-left"),
        Options { fov: 110, ..steps(vec![key("tab"), key("right")]) }.row("keys-at-max"),
        steps(vec![key("tab"), key("enter"), key("enter"), key("right")]).row("keys-enter-twice"),
        steps(vec![key("tab"), key("enter")]).row("keys-enter-locks-shown"),
        steps(vec![key("enter")]).row("enter-unfocused"),
    ]
}

fn mouse_rows() -> Vec<Value> {
    let y = 38;
    vec![
        steps(vec![click(130, y)]).row("click-fov"),
        steps(vec![click(130, y), release(130, y)]).row("click-fov-release"),
        steps(vec![click(59, y), release(59, y)]).row("click-fov-left"),
        steps(vec![click(208, y), release(208, y)]).row("click-fov-right"),
        steps(vec![click(70, y), drag(110, y), drag(150, y)]).row("drag-fov"),
        steps(vec![click(70, y), drag(110, y), drag(150, y), release(150, y)]).row("drag-fov-release"),
        steps(vec![click(70, y), drag(900, y), release(900, y)]).row("drag-past-end"),
        steps(vec![drag(150, y), release(150, y)]).row("drag-without-click"),
        steps(vec![click(5, 5), release(5, 5)]).row("click-empty"),
        steps(vec![click(100, 190), release(100, 190)]).row("click-telemetry-asleep"),
        steps(vec![click(130, y), release(130, y), key("right")]).row("click-fov-then-keys"),
        steps(vec![click(130, y), release(130, y), key("tab")]).row("click-fov-then-tab"),
        steps(vec![key("tab"), key("tab"), click(130, y), release(130, y)]).row("tab-then-click-fov"),
    ]
}

/// Arrows the slider does not take move the focus to the neighbour.
fn arrow_rows() -> Vec<Value> {
    let tab = || key("tab");
    let down = || key("down");
    let after_tab = |keys: Vec<Value>| steps([vec![tab()], keys].concat());
    vec![
        steps(vec![down()]).row("arrow-down-from-nothing"),
        steps(vec![key("up")]).row("arrow-up-from-nothing"),
        steps(vec![key("right")]).row("arrow-right-from-nothing"),
        steps(vec![key("left")]).row("arrow-left-from-nothing"),
        after_tab(vec![down()]).row("arrow-down-from-fov"),
        after_tab(vec![down(), down()]).row("arrow-down-twice"),
        after_tab(vec![down(), down(), down()]).row("arrow-down-thrice"),
        after_tab(vec![down(); 7]).row("arrow-down-to-done"),
        after_tab(vec![down(); 9]).row("arrow-down-past-done"),
        after_tab(vec![key("shift+tab"), key("up")]).row("arrow-up-from-done"),
        after_tab(vec![key("up")]).row("arrow-up-from-fov"),
        after_tab(vec![down(), key("right")]).row("arrow-right-from-button"),
        after_tab(vec![down(), key("right"), key("left")]).row("arrow-left-from-button"),
        after_tab(vec![down(), down(), down(), key("up")]).row("arrow-grid-down"),
        after_tab(vec![down(), down(), key("right"), down(), key("left")]).row("arrow-grid-across"),
        after_tab([vec![down(); 6], vec![key("right"), down()]].concat()).row("arrow-over-asleep"),
        after_tab(vec![key("enter"), key("right"), key("right")]).row("arrow-slider-takes-right"),
        after_tab(vec![key("enter"), key("enter"), key("left")]).row("arrow-slider-left-not-locked"),
        steps(vec![click(130, 38), release(130, 38), down()]).row("arrow-after-click"),
        Options { size: (200, 150), ..after_tab(vec![down(), down(), key("right")]) }.row("arrow-wide-small"),
        Options { size: (640, 360), ..after_tab(vec![down(), key("right"), down()]) }.row("arrow-wide-large"),
        Options { in_world: false, ..steps(vec![down(), down(), key("right")]) }.row("arrow-out-of-world"),
    ]
}

/// The options screen: the field of view, the header's second button, the
/// grid of settings screens and Done, in several screen sizes; hover, Tab
/// through every widget and round, the slider moved with the mouse and the
/// keys.
pub fn options_screen() -> Value {
    gui("options-screen", "options_screen", [layout_rows(), key_rows(), mouse_rows(), arrow_rows()].concat())
}
