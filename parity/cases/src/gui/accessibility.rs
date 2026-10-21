//! The accessibility settings screen.

use serde_json::{Value, json};

use super::gui;
use super::setting::{at, click, drag, frame, key, release, shift_click, slide, wheel};

/// The columns (left, right) and rows of the list as it opens.
const LEFT: i64 = 100;
const RIGHT: i64 = 300;
const ROWS: [i64; 7] = [45, 70, 95, 120, 145, 170, 195];
/// The rows once the list is scrolled to its end, the last at the bottom.
const END_ROWS: [i64; 7] = [45, 70, 95, 120, 145, 170, 195];

fn layout() -> Vec<Value> {
    let options = |label: &str, options: Value| at(label).options(options).row();
    vec![
        at("in-world").row(),
        at("out-of-world").outside().row(),
        at("small").window(640, 480).row(),
        at("large").window(1280, 720).row(),
        at("short").window(854, 300).row(),
        at("scroll-1").steps(vec![wheel(-1)]).row(),
        at("scroll-end").steps(vec![wheel(-100)]).row(),
        at("scroll-end-minecart").options(json!({"minecart": true})).steps(vec![wheel(-100)]).row(),
        at("scroll-end-out").outside().steps(vec![wheel(-100)]).row(),
        at("minecart-top").options(json!({"minecart": true})).row(),
        options("subtitles-on", json!({"subtitles": true})),
        options("text-background-zero", json!({"background_opacity": 0.0})),
        options("text-background-mid", json!({"background_opacity": 0.73})),
        options("opacity-zero", json!({"opacity": 0.0})),
        options("opacity-mid", json!({"opacity": 0.37})),
        options("spacing-mid", json!({"line_spacing": 0.25})),
        options("effects-zero", json!({"effects": 0.0})),
        options("effects-mid", json!({"effects": 0.43})),
        options("blurriness-zero", json!({"blurriness": 0})),
        options("blurriness-one", json!({"blurriness": 1})),
        options("blurriness-ten", json!({"blurriness": 10})),
    ]
}

fn presses() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    vec![
        press("press-narrator", vec![click(LEFT, ROWS[0])]),
        press("press-subtitles", vec![click(LEFT, ROWS[1])]),
        press("press-high-contrast", vec![click(RIGHT, ROWS[1])]),
        press("press-blurriness", vec![click(LEFT, ROWS[2])]),
        press("press-blurriness-start", vec![click(60, ROWS[2])]),
        press("drag-blurriness", slide((80, ROWS[2]), (130, ROWS[2]))),
        press("press-background-opacity", vec![click(RIGHT, ROWS[2])]),
        press("drag-background-opacity", slide((230, ROWS[2]), (300, ROWS[2]))),
        press("press-background-mode", vec![click(LEFT, ROWS[3])]),
        press("press-background-mode-twice", vec![click(LEFT, ROWS[3]), click(LEFT, ROWS[3])]),
        press("press-chat-opacity", vec![click(RIGHT, ROWS[3])]),
        press("drag-chat-opacity-zero", slide((300, ROWS[3]), (0, ROWS[3]))),
        press("press-spacing", vec![click(LEFT, ROWS[4])]),
        press("drag-spacing", slide((80, ROWS[4]), (130, ROWS[4]))),
        press("press-delay", vec![click(RIGHT, ROWS[4])]),
        press("drag-delay", vec![click(230, ROWS[4]), drag(260, ROWS[4]), drag(300, ROWS[4]), release(300, ROWS[4])]),
        press("press-notification-time", vec![click(LEFT, ROWS[5])]),
        press("drag-notification-time", slide((70, ROWS[5]), (150, ROWS[5]))),
        press("drag-notification-time-end", slide((70, ROWS[5]), (300, ROWS[5]))),
        press("press-bobbing", vec![click(RIGHT, ROWS[5])]),
        press("press-bobbing-shift", vec![shift_click(RIGHT, ROWS[5])]),
        press("press-effects", vec![click(LEFT, ROWS[6])]),
        press("drag-effects-zero", slide((100, ROWS[6]), (0, ROWS[6]))),
        press("press-fov-effects", vec![click(RIGHT, ROWS[6])]),
        press("drag-fov-effects", slide((230, ROWS[6]), (280, ROWS[6]))),
        press("press-gap", vec![click(212, ROWS[1])]),
    ]
}

fn scrolled() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    let end = |label: &str, (x, row): (i64, usize)| press(label, vec![wheel(-100), click(x, END_ROWS[row])]);
    let minecart = |label: &str, steps: Vec<Value>| at(label).options(json!({"minecart": true})).steps(steps).row();
    vec![
        end("end-press-darkness", (LEFT, 0)),
        end("end-press-damage-tilt", (RIGHT, 0)),
        end("end-press-glint-speed", (LEFT, 1)),
        end("end-press-glint-strength", (RIGHT, 1)),
        end("end-press-lightning", (LEFT, 2)),
        end("end-press-monochrome-logo", (RIGHT, 2)),
        end("end-press-panorama", (LEFT, 3)),
        end("end-press-splash", (RIGHT, 3)),
        end("end-press-hotkey", (LEFT, 4)),
        end("end-press-minecart", (RIGHT, 4)),
        end("end-press-outline", (LEFT, 5)),
        minecart("end-press-minecart-allowed", vec![wheel(-100), click(RIGHT, END_ROWS[4])]),
        minecart("end-press-minecart-twice", vec![wheel(-100), click(RIGHT, END_ROWS[4]), click(RIGHT, END_ROWS[4])]),
        press("scrolled-press-two-in-turn", vec![wheel(-100), click(LEFT, END_ROWS[2]), click(RIGHT, END_ROWS[3])]),
    ]
}

fn footer() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    vec![
        press("press-between-footer", vec![click(213, 223)]),
        at("hover-guide").mouse(134, 223).row(),
        at("hover-done").mouse(293, 223).row(),
        press("tab-1", vec![key("tab")]),
        press("tab-2", vec![key("tab"), key("tab")]),
        press("tab-back", vec![key("shift+tab")]),
        press("tab-back-twice", vec![key("shift+tab"), key("shift+tab")]),
        press("tab-to-footer", vec![key("shift+tab"), key("shift+tab"), key("shift+tab")]),
        press("tab-round", vec![key("shift+tab"), key("shift+tab"), key("shift+tab"), key("tab")]),
        press("arrow-down", vec![key("tab"), key("down")]),
        press("arrow-right", vec![key("tab"), key("right")]),
        press("arrow-from-done-up", vec![key("shift+tab"), key("up")]),
        press("arrow-from-done-left", vec![key("shift+tab"), key("left")]),
        press("arrow-from-guide-right", vec![key("shift+tab"), key("shift+tab"), key("right")]),
    ]
}

fn tips() -> Vec<Value> {
    let tip = |label: &str, (x, y): (i64, i64), steps: Vec<Value>| at(label).mouse(x, y).steps(steps).ms(10).row();
    vec![
        tip("tip-subtitles", (LEFT, ROWS[1]), vec![frame(0)]),
        tip("tip-high-contrast-inactive", (RIGHT, ROWS[1]), vec![frame(0)]),
        tip("tip-blurriness", (LEFT, ROWS[2]), vec![frame(0)]),
        tip("tip-notification-time", (LEFT, ROWS[5]), vec![frame(0)]),
        tip("tip-effects", (LEFT, ROWS[6]), vec![frame(0)]),
        tip("tip-none-on-narrator", (LEFT, ROWS[0]), vec![frame(0)]),
        tip("tip-minecart-inactive", (RIGHT, END_ROWS[4]), vec![wheel(-100), frame(0)]),
    ]
}

/// The accessibility screen: the narrator and the Controls link, twenty-three options two to a
/// row, the guide link and Done, with the options' own values shown, the list scrolled, and
/// sliders, cycle buttons and the footer pressed.
pub fn accessibility_options_screen() -> Value {
    gui(
        "accessibility-options-screen",
        "accessibility_options_screen",
        [layout(), presses(), scrolled(), footer(), tips()].concat(),
    )
}
