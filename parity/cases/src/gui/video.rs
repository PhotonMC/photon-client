//! The video settings screen: the list of display, quality and preference
//! options in several window sizes and GUI scales, scrolled, hovered,
//! pressed, Control and the wheel through the GUI scales, and the preset
//! turning to Custom when a quality option changes.

use serde_json::Value;

use super::gui;
use super::setting::{at, click, ctrl_wheel, drag, frame, release, shift_click, slide, times, wheel, wheel_at};

fn layout_rows() -> Vec<Value> {
    vec![
        at("in-world").row(),
        at("out-of-world").outside().row(),
        at("small").window(640, 480).row(),
        at("large").window(1280, 720).row(),
        at("tiny").window(640, 480).scale(3).row(),
        at("auto-large").window(1920, 1080).scale(0).row(),
        at("short").window(854, 300).row(),
        at("scale-1").scale(1).row(),
        at("very-short").window(640, 400).scale(4).row(),
        at("wide").window(1708, 480).outside().row(),
        at("hover-gui-scale").mouse(250, 110).row(),
        at("hover-fullscreen-resolution").mouse(200, 60).row(),
        at("hover-gamma").mouse(100, 160).row(),
        at("hover-done").mouse(213, 223).row(),
        at("hover-gap").mouse(212, 110).row(),
        at("hover-first-pixel-of-last").mouse(100, 206).row(),
        at("hover-under-header").mouse(100, 30).steps(vec![wheel(-2)]).row(),
        at("hover-header-line").mouse(100, 40).steps(vec![wheel(-1)]).row(),
    ]
}

fn scroll_rows() -> Vec<Value> {
    let bar = |label: &str, steps: Vec<Value>| at(label).steps(steps);
    vec![
        bar("scroll-1", vec![wheel(-1)]).row(),
        bar("scroll-5", times(wheel(-1), 5)).row(),
        bar("scroll-10", times(wheel(-1), 10)).row(),
        bar("scroll-end", vec![wheel(-100)]).row(),
        bar("scroll-up-past-top", vec![wheel(-3), wheel(10)]).row(),
        bar("scroll-fraction", vec![wheel(-0.5)]).row(),
        bar("scroll-fraction-up", vec![wheel(-4), wheel(1.5)]).row(),
        bar("scroll-over-header", vec![wheel_at(-3, 200, 10, false)]).row(),
        bar("scroll-over-widget", vec![wheel_at(-2, 250, 110, false)]).row(),
        bar("scroll-end-short", vec![wheel(-100)]).window(854, 300).row(),
        bar("scroll-end-very-short", vec![wheel(-100)]).window(640, 400).scale(4).row(),
        bar("scroll-end-large", vec![wheel(-100)]).window(1280, 720).row(),
        bar("scroll-end-out-of-world", vec![wheel(-100)]).outside().row(),
        bar("bar-drag", slide((379, 60), (379, 90))).row(),
        bar("bar-drag-held", vec![click(379, 60), drag(379, 90)]).row(),
        bar("bar-drag-past-bottom", vec![click(379, 60), drag(379, 230)]).row(),
        bar("bar-drag-above", vec![wheel(-5), click(379, 100), drag(379, 20)]).row(),
        bar("bar-drag-in-steps", vec![click(379, 50), drag(379, 70), drag(379, 100), drag(379, 130)]).row(),
        bar("bar-click-only", vec![click(379, 100), release(379, 100)]).row(),
        bar("bar-drag-short", vec![click(379, 50), drag(379, 80)]).window(854, 300).row(),
        bar("bar-drag-very-short", vec![click(100, 40), drag(100, 60)]).window(640, 400).scale(4).row(),
        bar("bar-edge-left", vec![click(376, 60), drag(376, 100)]).row(),
        bar("bar-edge-right", vec![click(382, 60), drag(382, 100)]).row(),
        bar("bar-beside", vec![click(383, 60), drag(383, 100)]).row(),
    ]
}

fn gui_scale_rows() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps);
    let large = |label: &str, steps: Vec<Value>| at(label).window(1920, 1080).scale(0).steps(steps);
    vec![
        press("gui-scale-press", vec![click(250, 110)]).row(),
        press("gui-scale-press-hover", vec![click(250, 110)]).mouse(250, 110).row(),
        press("gui-scale-press-twice", times(click(250, 110), 2)).row(),
        press("gui-scale-press-thrice", times(click(250, 110), 3)).row(),
        press("gui-scale-shift", vec![shift_click(250, 110)]).row(),
        large("gui-scale-from-auto", vec![click(300, 110)]).row(),
        large("gui-scale-large-auto-twice", times(click(300, 110), 2)).row(),
        press("gui-scale-then-scroll", vec![click(250, 110), wheel_at(-3, 300, 100, false)]).row(),
        press("gui-scale-scrolled", vec![wheel(-5), click(250, 50)]).row(),
        press("gui-scale-then-press-vsync", vec![click(250, 110), click(250, 85)]).row(),
        press("gui-scale-short", vec![click(250, 110)]).window(854, 300).row(),
        large("gui-scale-shift-wraps", vec![shift_click(300, 110)]).row(),
    ]
}

fn press_rows() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(steps).row();
    vec![
        press("press-vsync", vec![click(250, 85)]),
        at("press-vsync-away").mouse(5, 5).steps(vec![click(250, 85)]).row(),
        press("press-vsync-twice", times(click(250, 85), 2)),
        press("press-vsync-shift", vec![shift_click(250, 85)]),
        press("press-inactivity", vec![click(100, 110)]),
        press("press-exclusive", vec![click(250, 135)]),
        press("press-backend", vec![click(250, 160)]),
        press("press-backend-3", times(click(250, 160), 3)),
        press("press-gap", vec![click(212, 85)]),
        press("press-header-line", vec![click(100, 40)]),
        press("press-outside-row", vec![click(30, 85)]),
        press("press-under-header", vec![click(100, 20)]),
        press("press-two-then-move-focus", vec![click(250, 85), click(100, 110)]),
        press("press-slider-framerate", vec![click(100, 85)]),
        press("press-slider-framerate-release", vec![click(100, 85), release(100, 85)]),
        press("drag-slider-framerate", slide((100, 85), (150, 85))),
        press("drag-slider-framerate-held", vec![click(100, 85), drag(190, 85)]),
        press("drag-slider-framerate-end", slide((100, 85), (300, 85))),
        press("drag-slider-framerate-start", slide((100, 85), (0, 85))),
        press("press-slider-gamma", vec![click(70, 160)]),
        press("drag-slider-gamma", vec![click(70, 160), drag(60, 160)]),
        press("drag-slider-gamma-100", slide((70, 160), (300, 160))),
        press("drag-slider-gamma-0", slide((70, 160), (10, 160))),
        press("press-resolution-slider", vec![click(100, 60)]),
        press("press-resolution-slider-drag", slide((100, 60), (200, 60))),
        press("press-preset-last-pixel", vec![click(170, 206)]),
        press("press-preset-last-pixel-release", vec![click(170, 206), release(170, 206)]),
        press("press-preset-scrolled-in", vec![wheel(-9), click(170, 100)]),
        press("press-ao-custom", vec![wheel(-10), click(100, 170)]),
        press("press-ao-custom-twice", vec![wheel(-10), click(100, 170), click(100, 170)]),
        press("press-shadows-custom", vec![wheel(-10), click(100, 180)]),
        press("press-particles-custom", vec![wheel(-10), click(300, 170)]),
        press("press-vsync-keeps-preset", vec![wheel(-10), click(250, 20)]),
    ]
}

/// Steps after scrolling to the end of the list.
fn bottom(steps: Vec<Value>) -> Vec<Value> {
    [vec![wheel(-100)], steps].concat()
}

fn bottom_rows() -> Vec<Value> {
    let press = |label: &str, steps: Vec<Value>| at(label).steps(bottom(steps)).row();
    let filtering = || click(100, 86);
    vec![
        press("press-weather-slider", vec![click(300, 110)]),
        press("press-weather-slider-drag", slide((300, 110), (250, 110))),
        press("press-cloud-range-clipped", vec![click(300, 40)]),
        press("press-cutout-leaves", vec![click(100, 60)]),
        press("press-transparency", vec![click(300, 60)]),
        press("anisotropy-asleep", vec![]),
        at("anisotropy-asleep-hover").mouse(300, 86).steps(bottom(vec![])).row(),
        press("anisotropy-asleep-press", vec![click(300, 86)]),
        press("filtering-rgss", vec![filtering()]),
        press("filtering-anisotropic", vec![filtering(), filtering()]),
        at("filtering-anisotropic-press-slider")
            .steps(vec![wheel(-100), filtering(), wheel(-100), filtering(), click(300, 86)])
            .row(),
        press(
            "filtering-anisotropic-drag-slider",
            [vec![filtering(), filtering()], slide((250, 86), (340, 86))].concat(),
        ),
        press("filtering-back-to-none", times(filtering(), 3)),
        press("press-autosave", vec![click(100, 165)]),
        press("press-vignette", vec![click(300, 165)]),
        press("press-attack", vec![click(100, 190)]),
        press("press-attack-shift", vec![shift_click(100, 190)]),
        press("press-chunk-fade", vec![click(300, 190)]),
        press("drag-chunk-fade", slide((300, 190), (240, 190))),
        press("drag-chunk-fade-zero", slide((300, 190), (0, 190))),
    ]
}

fn ctrl_rows() -> Vec<Value> {
    let ctrl = |label: &str, steps: Vec<Value>| at(label).steps(steps);
    let large =
        |label: &str, scale: i64, steps: Vec<Value>| at(label).window(1920, 1080).scale(scale).steps(steps).row();
    vec![
        ctrl("ctrl-wheel-down", vec![ctrl_wheel(-1)]).row(),
        ctrl("ctrl-wheel-up-at-top", vec![ctrl_wheel(1)]).row(),
        large("ctrl-wheel-auto-down", 0, vec![ctrl_wheel(-1)]),
        large("ctrl-wheel-auto-up", 0, vec![ctrl_wheel(1)]),
        large("ctrl-wheel-down-twice", 3, times(ctrl_wheel(-1), 2)),
        large("ctrl-wheel-down-thrice", 3, times(ctrl_wheel(-1), 3)),
        large("ctrl-wheel-up", 2, vec![ctrl_wheel(1)]),
        ctrl("ctrl-wheel-scrolled", vec![wheel(-5), ctrl_wheel(-1)]).row(),
        ctrl("ctrl-wheel-outside-list", vec![wheel_at(-1, 200, 10, true)]).row(),
        large("ctrl-wheel-fraction", 3, vec![ctrl_wheel(-0.5)]),
        ctrl("ctrl-wheel-focused", vec![click(250, 85), ctrl_wheel(-1)]).row(),
        ctrl("ctrl-wheel-then-wheel", vec![ctrl_wheel(-1), wheel(-3)]).row(),
        ctrl("ctrl-wheel-at-1", vec![ctrl_wheel(-1)]).scale(1).row(),
    ]
}

/// A tooltip a moment after the mouse is on the widget.
fn tip(label: &str, (x, y): (i64, i64), steps: Vec<Value>) -> super::setting::Setting {
    at(&format!("tip-{label}")).mouse(x, y).steps([vec![frame(0)], steps].concat()).ms(10)
}

fn tip_rows() -> Vec<Value> {
    let far = || vec![wheel(-100)];
    let up = || vec![wheel(-10)];
    let mut rows = vec![
        at("tip-first-frame").mouse(100, 110).row(),
        tip("inactivity", (100, 110), vec![]).row(),
        tip("exclusive", (250, 135), vec![]).row(),
        tip("backend", (250, 160), vec![]).row(),
        tip("gui-scale-none", (250, 110), vec![]).row(),
        tip("resolution-none", (200, 60), vec![]).row(),
        tip("preset", (150, 108), vec![wheel(-9)]).row(),
        tip("preset-last-pixel", (150, 206), vec![]).row(),
        tip("prioritize", (100, 146), up()).row(),
        tip("ao-none", (100, 170), up()).row(),
        tip("blurriness", (100, 40), far()).row(),
        tip("cutout-leaves", (100, 60), far()).row(),
        tip("transparency", (300, 60), far()).row(),
        tip("filtering", (100, 86), far()).row(),
        tip("anisotropy-asleep", (300, 86), far()).row(),
        tip("weather", (100, 110), far()).row(),
        tip("vignette", (300, 167), far()).row(),
        tip("chunk-fade", (300, 192), far()).row(),
        tip("attack-none", (100, 192), far()).row(),
    ];
    rows.extend(pressed_tips());
    rows
}

fn pressed_tips() -> Vec<Value> {
    let filtering = || click(100, 86);
    vec![
        tip("exclusive-restart", (250, 135), vec![click(250, 135)]).row(),
        tip("exclusive-back", (250, 135), times(click(250, 135), 2)).row(),
        tip("backend-restart", (250, 160), vec![click(250, 160)]).row(),
        tip("backend-vulkan", (250, 160), times(click(250, 160), 2)).row(),
        tip("backend-default", (250, 160), times(click(250, 160), 3)).row(),
        tip("inactivity-minimized", (100, 110), vec![click(100, 110)]).row(),
        tip("filtering-rgss", (100, 86), vec![wheel(-100), filtering()]).row(),
        tip("filtering-anisotropic", (100, 86), vec![wheel(-100), filtering(), filtering()]).row(),
        tip("anisotropy-awake", (300, 86), vec![wheel(-100), filtering(), filtering()]).row(),
        tip("prioritize-by-player", (100, 146), vec![wheel(-10), click(100, 146)]).row(),
        tip("prioritize-nearby", (100, 146), vec![wheel(-10), click(100, 146), click(100, 146)]).row(),
        tip("preset-after-custom", (150, 108), vec![wheel(-10), click(100, 170)]).row(),
        tip("small-screen", (100, 110), vec![]).window(640, 480).row(),
        tip("tiny-screen", (150, 70), vec![]).window(640, 480).scale(3).row(),
        tip("out-of-world", (100, 110), vec![]).outside().row(),
        at("tip-late").mouse(100, 110).steps(vec![frame(0), frame(300)]).ms(1000).row(),
        at("tip-moved-away").mouse(5, 5).steps(vec![frame(0)]).ms(10).row(),
        at("tip-clipped-hover").mouse(100, 30).steps(vec![frame(0), wheel(-2)]).ms(10).row(),
        at("tip-marquee-time").mouse(100, 110).steps(vec![frame(0), wheel(-100)]).ms(1234).row(),
    ]
}

pub fn video_settings_screen() -> Value {
    let rows =
        [layout_rows(), scroll_rows(), gui_scale_rows(), press_rows(), bottom_rows(), ctrl_rows(), tip_rows()].concat();
    gui("video-settings-screen", "video_settings_screen", rows)
}
