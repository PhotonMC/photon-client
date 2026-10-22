//! The chat, skin, mouse, controls and sound settings screens.

use serde_json::{Value, json};

use super::gui;
use super::setting::{at, click, drag, frame, key, shift_click, slide, times, wheel};

fn chat_layout() -> Vec<Value> {
    let options = |label: &str, options: Value| at(label).options(options).row();
    vec![
        at("in-world").row(),
        at("out-of-world").outside().row(),
        at("small").window(640, 480).row(),
        at("large").window(1280, 720).row(),
        at("tiny").window(640, 480).scale(3).row(),
        at("short").window(854, 300).row(),
        at("auto-large").window(1920, 1080).scale(0).row(),
        at("hover-visibility").mouse(100, 45).row(),
        at("hover-narrator-asleep").mouse(100, 195).row(),
        at("hover-slider").mouse(100, 95).row(),
        at("hover-done").mouse(213, 223).row(),
        at("scroll-1").steps(vec![wheel(-1)]).row(),
        at("scroll-5").steps(vec![wheel(-5)]).row(),
        at("scroll-end").steps(vec![wheel(-100)]).row(),
        at("scroll-end-short").window(854, 300).steps(vec![wheel(-100)]).row(),
        at("bar-drag").steps(slide((379, 60), (379, 90))).row(),
        at("bar-drag-end").steps(vec![click(379, 60), drag(379, 230)]).row(),
        options("opacity-zero", json!({"opacity": 0.0})),
        options("opacity-full", json!({"opacity": 1.0})),
        options("opacity-mid", json!({"opacity": 0.37})),
        options("background-zero", json!({"background_opacity": 0.0})),
        options("background-mid", json!({"background_opacity": 0.73})),
        options("scale-zero", json!({"scale": 0.0})),
        options("scale-mid", json!({"scale": 0.55})),
        options("spacing-full", json!({"line_spacing": 1.0})),
        options("spacing-mid", json!({"line_spacing": 0.25})),
        options("width-zero", json!({"width": 0.0})),
        options("width-mid", json!({"width": 0.5})),
        options("width-full", json!({"width": 1.0})),
        options("height-focused-low", json!({"height_focused": 0.0})),
        options("height-focused-mid", json!({"height_focused": 0.33})),
        options("height-unfocused-high", json!({"height_unfocused": 1.0})),
        options(
            "everything",
            json!({
                "opacity": 0.2, "background_opacity": 0.9, "scale": 0.7, "line_spacing": 0.6, "width": 0.3,
                "height_focused": 0.8, "height_unfocused": 0.1,
            }),
        ),
    ]
}

fn chat_presses() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    vec![
        press("press-visibility", vec![click(100, 45)]),
        press("press-visibility-twice", times(click(100, 45), 2)),
        press("press-visibility-thrice", times(click(100, 45), 3)),
        press("press-visibility-shift", vec![shift_click(100, 45)]),
        press("press-colors", vec![click(250, 45)]),
        press("press-links", vec![click(100, 70)]),
        press("press-links-prompt", vec![click(250, 70)]),
        press("press-narrator-asleep", vec![click(100, 195)]),
        press("press-gap", vec![click(212, 45)]),
        press("press-opacity", vec![click(100, 95)]),
        press("press-opacity-end", vec![click(205, 95)]),
        press("press-opacity-start", vec![click(60, 95)]),
        press("drag-opacity", slide((80, 95), (130, 95))),
        press("drag-opacity-held", vec![click(80, 95), drag(180, 95)]),
        press("drag-opacity-past", slide((80, 95), (400, 95))),
        press("press-background", vec![click(300, 95)]),
        press("drag-background", slide((230, 95), (300, 95))),
        press("press-scale", vec![click(100, 120)]),
        press("drag-scale-zero", slide((100, 120), (0, 120))),
        press("drag-scale-full", slide((100, 120), (300, 120))),
        press("press-spacing", vec![click(300, 120)]),
        press("press-delay", vec![click(100, 145)]),
        press("drag-delay", vec![click(70, 145), drag(120, 145), drag(160, 145), super::setting::release(160, 145)]),
        press("drag-delay-end", slide((70, 145), (300, 145))),
        press("press-width", vec![click(300, 145)]),
        press("drag-width", slide((230, 145), (300, 145))),
        press("drag-width-zero", slide((300, 145), (0, 145))),
        press("press-height-focused", vec![click(100, 170)]),
        press("drag-height-focused", slide((70, 170), (150, 170))),
        press("press-height-unfocused", vec![click(300, 170)]),
        press("drag-height-unfocused-zero", slide((300, 170), (0, 170))),
        press("press-autosuggest", vec![click(250, 195)]),
    ]
}

fn chat_scrolled() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    let tip = |label: &str, (x, y): (i64, i64), steps: Vec<Value>| at(label).mouse(x, y).steps(steps).ms(10).row();
    vec![
        press("scrolled-press-hide-matched", vec![wheel(-100), click(100, 100)]),
        press("scrolled-press-reduced-debug", vec![wheel(-100), click(250, 100)]),
        press("scrolled-press-secure", vec![wheel(-100), click(100, 125)]),
        press("scrolled-press-drafts", vec![wheel(-100), click(250, 125)]),
        press("scrolled-press-twice", vec![wheel(-100), click(250, 125), click(250, 125)]),
        press("press-clipped-bottom", vec![click(100, 206)]),
        press("press-two-in-turn", vec![click(100, 45), click(250, 70), click(100, 95)]),
        // Tooltips of the widgets that have one.
        tip("tip-hide-matched", (100, 167), vec![frame(0), wheel(-100)]),
        tip("tip-hide-matched-seen", (100, 167), vec![wheel(-100), frame(0)]),
        tip("tip-secure", (100, 192), vec![wheel(-100), frame(0)]),
        tip("tip-drafts", (300, 192), vec![wheel(-100), frame(0)]),
        tip("tip-none-on-opacity", (100, 95), vec![frame(0)]),
        tip("tip-drafts-pressed", (300, 192), vec![wheel(-100), frame(0), click(300, 192)]),
    ]
}

/// The chat settings screen: the eighteen options in a list, with the
/// options' own values shown, the narrator asleep, the list scrolled and
/// its bar dragged, and sliders and cycle buttons pressed.
pub fn chat_options_screen() -> Value {
    gui("chat-options-screen", "chat_options_screen", [chat_layout(), chat_presses(), chat_scrolled()].concat())
}

/// The skin customization screen: the seven parts and the main hand in two
/// columns, each part on or off from the options, pressed, with the main
/// hand cycled both ways.
pub fn skin_customization_screen() -> Value {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    let rows = vec![
        at("in-world").row(),
        at("out-of-world").outside().row(),
        at("small").window(640, 480).row(),
        at("short").window(854, 200).row(),
        at("hover-cape").mouse(100, 45).row(),
        at("none-shown").options(json!({"model_parts": 0})).row(),
        at("some-shown").options(json!({"model_parts": 0b1010101})).row(),
        at("left-handed").options(json!({"main_hand": "LEFT"})).row(),
        press("press-cape", vec![click(100, 45)]),
        press("press-hat", vec![click(100, 120)]),
        press("press-jacket-twice", times(click(250, 45), 2)),
        press("press-main-hand", vec![click(250, 120)]),
        press("press-main-hand-twice", times(click(250, 120), 2)),
        press("press-main-hand-shift", vec![shift_click(250, 120)]),
        press("press-sleeves", vec![click(100, 70), click(250, 70)]),
    ];
    gui("skin-customization-screen", "skin_customization_screen", rows)
}

/// The font settings screen: the two toggles, each pressed, and the hint of
/// the glyph variants.
pub fn font_options_screen() -> Value {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    let rows = vec![
        at("in-world").row(),
        at("out-of-world").outside().row(),
        at("small").window(640, 480).row(),
        at("hover-variants").mouse(250, 45).row(),
        press("press-unicode", vec![click(100, 45)]),
        press("press-variants", vec![click(250, 45)]),
        press("press-variants-twice", times(click(250, 45), 2)),
    ];
    gui("font-options-screen", "font_options_screen", rows)
}

fn mouse_rows() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    let options = |label: &str, options: Value| at(label).options(options).row();
    vec![
        at("in-world").row(),
        at("out-of-world").outside().row(),
        at("small").window(640, 480).row(),
        at("large").window(1280, 720).row(),
        at("tiny").window(640, 480).scale(3).row(),
        at("short").window(854, 240).row(),
        options("sensitivity-zero", json!({"sensitivity": 0.0})),
        options("sensitivity-full", json!({"sensitivity": 1.0})),
        options("sensitivity-mid", json!({"sensitivity": 0.37})),
        options("wheel-low", json!({"wheel": 0.01})),
        options("wheel-high", json!({"wheel": 10.0})),
        options("wheel-odd", json!({"wheel": 2.5})),
        options("inverted", json!({"invert_x": true, "invert_y": true})),
        options("inverted-x", json!({"invert_x": true})),
        at("hover-sensitivity").mouse(100, 45).row(),
        at("hover-done").mouse(213, 223).row(),
        at("hover-gap").mouse(212, 45).row(),
        press("press-sensitivity", vec![click(100, 45)]),
        press("press-sensitivity-start", vec![click(60, 45)]),
        press("press-sensitivity-end", vec![click(205, 45)]),
        press("drag-sensitivity", slide((80, 45), (150, 45))),
        press("drag-sensitivity-held", vec![click(80, 45), drag(190, 45)]),
        press("drag-sensitivity-zero", slide((100, 45), (0, 45))),
        press("drag-sensitivity-full", slide((100, 45), (400, 45))),
        press("press-wheel", vec![click(300, 45)]),
        press("drag-wheel", slide((230, 45), (260, 45))),
        press("drag-wheel-low", slide((300, 45), (0, 45))),
        press("drag-wheel-high", slide((300, 45), (500, 45))),
        press("press-discrete", vec![click(100, 70)]),
        press("press-discrete-twice", times(click(100, 70), 2)),
        press("press-invert-x", vec![click(250, 70)]),
        press("press-invert-y", vec![click(100, 95)]),
        press("press-cursor", vec![click(250, 95)]),
        press("press-shift", vec![shift_click(250, 70)]),
        press("press-gap", vec![click(212, 70)]),
        at("scroll-short").window(854, 240).steps(vec![wheel(-100)]).row(),
        press("press-in-turn", vec![click(250, 70), click(100, 95), click(100, 45)]),
        at("tip-cursor-changes").mouse(250, 97).steps(vec![frame(0)]).ms(10).row(),
        at("tip-cursor-first").mouse(250, 97).row(),
        at("tip-none-on-sensitivity").mouse(100, 45).steps(vec![frame(0)]).ms(10).row(),
    ]
}

fn control_rows() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    let dragged = [vec![frame(0)], slide((230, 120), (300, 120))].concat();
    vec![
        at("in-world").row(),
        at("out-of-world").outside().row(),
        at("small").window(640, 480).row(),
        at("large").window(1280, 720).row(),
        at("short").window(854, 240).row(),
        at("tiny").window(640, 480).scale(3).row(),
        at("hover-mouse-settings").mouse(100, 45).row(),
        at("hover-key-binds").mouse(300, 45).row(),
        at("hover-toggle").mouse(100, 70).row(),
        at("hover-done").mouse(213, 223).row(),
        press("press-toggle-sneak", vec![click(100, 70)]),
        press("press-toggle-sneak-twice", times(click(100, 70), 2)),
        press("press-toggle-sprint", vec![click(250, 70)]),
        press("press-toggle-attack", vec![click(100, 95)]),
        press("press-toggle-use", vec![click(250, 95)]),
        press("press-shift-toggle", vec![shift_click(250, 95)]),
        press("press-auto-jump", vec![click(100, 120)]),
        press("press-sprint-window", vec![click(250, 120)]),
        press("drag-sprint-window", slide((230, 120), (300, 120))),
        press("drag-sprint-window-zero", slide((230, 120), (0, 120))),
        press("drag-sprint-window-end", slide((230, 120), (400, 120))),
        press("press-operator-tab", vec![click(100, 145)]),
        press("press-gap", vec![click(212, 70)]),
        at("scroll-short").window(854, 240).steps(vec![wheel(-100)]).row(),
        press("press-in-turn", vec![click(100, 70), click(250, 95), click(100, 145)]),
        at("tip-sprint-window").mouse(250, 120).steps(vec![frame(0)]).ms(10).row(),
        at("tip-sprint-window-dragged").mouse(250, 120).steps(dragged).ms(10).row(),
        at("tip-none-on-buttons").mouse(100, 45).steps(vec![frame(0)]).ms(10).row(),
    ]
}

/// The mouse settings screen and the controls screen it opens from.
pub fn mouse_and_controls() -> Vec<Value> {
    vec![
        gui("mouse-settings-screen", "mouse_settings_screen", mouse_rows()),
        gui("controls-screen", "controls_screen", control_rows()),
    ]
}

/// The key mappings, their categories and the keys' names, as a fresh
/// `Options` has them.
pub fn key_mappings() -> Value {
    json!({"case_id": "gui/key-mappings", "family": "gui", "operation": "key_mappings"})
}

const CATEGORIES: [&str; 11] =
    ["master", "music", "record", "weather", "block", "hostile", "neutral", "player", "ambient", "voice", "ui"];

fn sound_layout() -> Vec<Value> {
    let options = |label: &str, options: Value| at(label).options(options).row();
    let silent: serde_json::Map<String, Value> =
        CATEGORIES.iter().map(|name| ((*name).to_owned(), json!(0.0))).collect();
    let mixed = json!({
        "music": 0.5, "record": 0.25, "weather": 0.0, "block": 0.75, "hostile": 0.1, "neutral": 0.9, "player": 0.33,
    });
    vec![
        at("in-world").row(),
        at("out-of-world").outside().row(),
        at("small").window(640, 480).row(),
        at("large").window(1280, 720).row(),
        at("tiny").window(640, 480).scale(3).row(),
        at("short").window(854, 240).row(),
        at("tall").window(854, 720).row(),
        options("master-off", json!({"volumes": {"master": 0.0}})),
        options("master-mid", json!({"volumes": {"master": 0.37}})),
        options("all-off", json!({ "volumes": silent })),
        options("mixed", json!({ "volumes": mixed })),
        options("subtitles-on", json!({"subtitles": true})),
        options("directional-on", json!({"directional": true})),
        options("frequent", json!({"music_frequency": "FREQUENT"})),
        options("constant", json!({"music_frequency": "CONSTANT"})),
        options("toast-pause", json!({"music_toast": "pause"})),
        options("toast-both", json!({"music_toast": "pause_and_toast"})),
        at("hover-master").mouse(100, 45).row(),
        at("hover-music").mouse(100, 70).row(),
        at("hover-gap").mouse(212, 70).row(),
        at("hover-done").mouse(213, 223).row(),
    ]
}

fn sound_presses() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    vec![
        press("press-master", vec![click(100, 45)]),
        press("press-master-start", vec![click(60, 45)]),
        press("press-master-end", vec![click(365, 45)]),
        press("drag-master", slide((100, 45), (200, 45))),
        press("drag-master-held", vec![click(100, 45), drag(300, 45)]),
        press("drag-master-zero", slide((100, 45), (0, 45))),
        press("drag-master-full", slide((100, 45), (500, 45))),
        press("press-music", vec![click(100, 70)]),
        press("press-record", vec![click(300, 70)]),
        press("drag-weather", slide((100, 95), (60, 95))),
        press("drag-block", slide((300, 95), (250, 95))),
        press("press-hostile", vec![click(100, 120)]),
        press("press-neutral", vec![click(300, 120)]),
        press("press-player", vec![click(100, 145)]),
        press("press-ambient", vec![click(300, 145)]),
        press("press-voice", vec![click(100, 170)]),
        press("press-ui", vec![click(300, 170)]),
        press("press-gap", vec![click(212, 70)]),
        press("scroll-bottom", vec![wheel(-100)]),
        at("scroll-short").window(854, 240).steps(vec![wheel(-100)]).row(),
        at("scroll-short-hover").window(854, 240).mouse(100, 60).steps(vec![wheel(-100)]).row(),
        at("scroll-tall-up").window(854, 240).steps(vec![wheel(-100), wheel(100)]).row(),
    ]
}

fn sound_bottom() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps([vec![wheel(-100)], steps].concat()).row();
    let tip = |label: &str, (x, y): (i64, i64)| at(label).mouse(x, y).steps(vec![wheel(-100), frame(0)]).ms(10);
    vec![
        press("press-device-scrolled", vec![click(100, 140)]),
        press("press-subtitles", vec![click(100, 165)]),
        press("press-directional", vec![click(300, 165)]),
        press("press-frequency", vec![click(100, 190)]),
        press("press-frequency-twice", times(click(100, 190), 2)),
        press("press-frequency-shift", vec![shift_click(100, 190)]),
        press("press-toast", vec![click(300, 190)]),
        press("press-toast-thrice", times(click(300, 190), 3)),
        tip("tip-subtitles", (100, 165)).row(),
        tip("tip-directional", (300, 165)).row(),
        tip("tip-directional-on", (300, 165)).options(json!({"directional": true})).row(),
        tip("tip-frequency", (100, 190)).row(),
        tip("tip-toast", (300, 190)).row(),
        tip("tip-toast-both", (300, 190)).options(json!({"music_toast": "pause_and_toast"})).row(),
        at("tip-none-on-slider").mouse(100, 45).steps(vec![frame(0)]).ms(10).row(),
        tip("tip-none-on-device", (100, 140)).row(),
        at("tab-first").steps(vec![key("tab")]).row(),
        at("tab-twice").steps(times(key("tab"), 2)).row(),
        at("tab-back").steps(vec![key("shift+tab")]).row(),
        at("arrow-right").steps(vec![key("tab"), key("right")]).row(),
        at("arrow-left").steps(vec![key("tab"), key("left")]).row(),
        at("arrow-down-twice").steps(vec![key("tab"), key("down"), key("down")]).row(),
        at("press-in-turn").steps(vec![click(100, 45), click(300, 70), click(100, 95)]).row(),
    ]
}

/// The sound settings screen: its list in several sizes, the sliders at
/// their ends and middle, the device, the subtitles, the directional audio
/// and the music choices at their values and pressed, and the tooltips of
/// the last three.
pub fn sound_options_screen() -> Value {
    gui("sound-options-screen", "sound_options_screen", [sound_layout(), sound_presses(), sound_bottom()].concat())
}
