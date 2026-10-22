//! Dialogs with a single-line text box: the label above it, what typing and editing do, and the
//! values a template and a payload take from it.

use serde_json::{Value, json};

use super::super::super::setting::{key, times};
use super::super::{Dialog, button, click, row};

fn text(label: &str, width: i64, extra: Value) -> Value {
    let mut input = json!({"type": "text", "key": "word", "label": label, "width": width});
    for (name, value) in extra.as_object().into_iter().flatten() {
        input[name] = value.clone();
    }
    input
}

fn typed(text: &str) -> Value {
    json!({"do": "type", "text": text})
}

fn tab3() -> Vec<Value> {
    times(key("tab"), 3)
}

fn layout_rows() -> Vec<Value> {
    rows![
        Dialog { inputs: Some(vec![text("Name", 200, json!({}))]), ..row("text") },
        Dialog {
            inputs: Some(vec![text("Name", 200, json!({"label_visible": false, "initial": "hello"}))]),
            ..row("text-no-label")
        },
        Dialog {
            inputs: Some(vec![text("A label wider than its box", 60, json!({"initial": "abc"}))]),
            ..row("text-wide-label")
        },
        Dialog { inputs: Some(vec![text("Name", 120, json!({"initial": "abc"}))]), ..row("text-narrow") },
        Dialog { inputs: Some(vec![text("Name", 200, json!({}))]), steps: tab3(), ..row("text-focus") },
    ]
}

fn editing_rows() -> Vec<Value> {
    let name = |extra: Value| vec![text("Name", 200, extra)];
    let focus = |steps: Vec<Value>| [tab3(), steps].concat();
    rows![
        Dialog { inputs: Some(name(json!({}))), steps: focus(vec![typed("abc")]), ..row("text-type") },
        Dialog {
            inputs: Some(name(json!({"max_length": 5}))),
            steps: focus(vec![typed("abcdefgh")]),
            ..row("text-max-length")
        },
        Dialog { inputs: Some(name(json!({}))), steps: focus(vec![typed("a b")]), ..row("text-space") },
        Dialog { inputs: Some(name(json!({}))), steps: vec![typed("abc")], ..row("text-type-unfocused") },
        Dialog {
            inputs: Some(name(json!({"initial": "hello"}))),
            steps: focus(vec![key("backspace"), key("backspace")]),
            ..row("text-backspace")
        },
        Dialog {
            inputs: Some(name(json!({"initial": "hello"}))),
            steps: focus(vec![key("home"), key("delete"), typed("J")]),
            ..row("text-home-delete")
        },
        Dialog {
            inputs: Some(name(json!({"initial": "hello"}))),
            steps: focus(vec![key("ctrl+a"), typed("x")]),
            ..row("text-select-all")
        },
        Dialog {
            inputs: Some(name(json!({"initial": "hello world"}))),
            steps: focus(vec![key("ctrl+left"), key("shift+left"), key("backspace")]),
            ..row("text-word-select")
        },
        Dialog {
            inputs: Some(name(json!({"initial": "hello"}))),
            steps: focus(vec![key("left"), key("left"), typed("-")]),
            ..row("text-left-keys")
        },
        Dialog {
            inputs: Some(name(json!({"initial": "hello"}))),
            steps: focus(vec![key("enter"), key("space")]),
            ..row("text-enter-space")
        },
        Dialog {
            inputs: Some(name(json!({"initial": "a very long text that does not fit in the box at all"}))),
            steps: focus(vec![key("home")]),
            ..row("text-scrolls")
        },
        Dialog {
            inputs: Some(name(json!({"initial": "hello", "max_length": 64}))),
            steps: focus(vec![typed("\u{e9}\u{20ac} zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz")]),
            ..row("text-scrolls-typing")
        },
        Dialog {
            inputs: Some(name(json!({"initial": "hello"}))),
            steps: [focus(vec![]), vec![key("tab")], vec![typed("x")]].concat(),
            ..row("text-tab-out")
        },
    ]
}

fn mouse_rows() -> Vec<Value> {
    let hello = || vec![text("Name", 200, json!({"initial": "hello"}))];
    rows![
        Dialog { inputs: Some(hello()), steps: vec![click(130, 113)], ..row("text-click") },
        Dialog { inputs: Some(hello()), steps: vec![click(130, 113), typed("-")], ..row("text-click-type") },
        Dialog { inputs: Some(hello()), steps: vec![click(300, 113), typed("!")], ..row("text-click-past-end") },
        Dialog { inputs: Some(hello()), steps: vec![click(113, 113), typed("<")], ..row("text-click-left-edge") },
        Dialog { inputs: Some(hello()), steps: vec![click(200, 95), typed("x")], ..row("text-click-label") },
        Dialog { inputs: Some(hello()), mouse: (200, 113), ..row("text-hover") },
        Dialog {
            inputs: Some(vec![text(
                "Name",
                200,
                json!({"initial": "a very long text that does not fit in the box at all"})
            )]),
            steps: vec![click(130, 113)],
            ..row("text-click-scrolled")
        },
    ]
}

fn value_rows() -> Vec<Value> {
    let send = || vec![button("Send", 150, None, Some(json!({"type": "template", "value": "say $(word)"})))];
    let all = || vec![button("Send", 150, None, Some(json!({"type": "custom_all", "value": "minecraft:answer"})))];
    let name = |initial: &str| vec![text("Name", 200, json!({"initial": initial}))];
    let press = || vec![click(213, 223)];
    rows![
        Dialog { inputs: Some(name("plain")), buttons: send(), steps: press(), ..row("text-template") },
        Dialog {
            inputs: Some(name("a\"b'c\\d\te\u{1}\n")),
            buttons: send(),
            steps: press(),
            ..row("text-template-escapes")
        },
        Dialog { inputs: Some(name("plain")), buttons: all(), steps: press(), ..row("text-custom-all") },
        Dialog { inputs: Some(name("say \"hi\"")), buttons: all(), steps: press(), ..row("text-custom-all-double") },
        Dialog { inputs: Some(name("it's")), buttons: all(), steps: press(), ..row("text-custom-all-single") },
        Dialog { inputs: Some(name("'\"")), buttons: all(), steps: press(), ..row("text-custom-all-both") },
        Dialog { inputs: Some(name("a\\b\tc\u{1}")), buttons: all(), steps: press(), ..row("text-custom-all-escapes") },
        Dialog {
            inputs: Some(name("")),
            buttons: send(),
            steps: [tab3(), vec![typed("it\"s"), key("tab"), key("enter")]].concat(),
            ..row("text-template-typed")
        },
    ]
}

fn area(label_visible: bool, extra: Value) -> Value {
    let mut input = text("Notes", 200, json!({"multiline": {}, "label_visible": label_visible}));
    for (name, value) in extra.as_object().into_iter().flatten() {
        input[name] = value.clone();
    }
    input
}

fn area_layout_rows() -> Vec<Value> {
    let initial = "first line\nsecond line\nthird";
    rows![
        Dialog { inputs: Some(vec![area(true, json!({}))]), ..row("area") },
        Dialog { inputs: Some(vec![area(false, json!({"initial": initial}))]), ..row("area-no-label") },
        Dialog { inputs: Some(vec![area(true, json!({"initial": initial}))]), ..row("area-initial") },
        Dialog {
            inputs: Some(vec![area(true, json!({"multiline": {"height": 80}, "initial": initial}))]),
            ..row("area-height")
        },
        Dialog {
            inputs: Some(vec![area(true, json!({"multiline": {"max_lines": 2}, "initial": "one\ntwo\nthree"}))]),
            ..row("area-max-lines")
        },
        Dialog {
            inputs: Some(vec![area(true, json!({"multiline": {"max_lines": 6}, "max_length": 200}))]),
            ..row("area-tall")
        },
        Dialog {
            inputs: Some(vec![area(
                true,
                json!({"initial": "a very long text that has to wrap around the edge of the box more than once"})
            )]),
            ..row("area-wraps")
        },
        Dialog {
            inputs: Some(vec![area(true, json!({"initial": "1\n2\n3\n4\n5\n6\n7", "max_length": 50}))]),
            ..row("area-scrolls")
        },
        Dialog {
            inputs: Some(vec![area(true, json!({"initial": "1\n2\n3\n4\n5\n6\n7", "max_length": 50}))]),
            steps: vec![json!({"do": "scroll", "x": 200, "y": 120, "amount": -1})],
            ..row("area-wheel")
        },
        Dialog { inputs: Some(vec![area(true, json!({}))]), steps: tab3(), ..row("area-focus") },
    ]
}

fn area_editing_rows() -> Vec<Value> {
    let notes = |extra: Value| vec![area(true, extra)];
    let focus = |steps: Vec<Value>| [tab3(), steps].concat();
    let lines = || json!({"initial": "alpha\nbeta\ngamma", "max_length": 50});
    rows![
        Dialog { inputs: Some(notes(json!({}))), steps: focus(vec![typed("abc")]), ..row("area-type") },
        Dialog {
            inputs: Some(notes(json!({}))),
            steps: focus(vec![typed("a"), key("enter"), typed("b"), key("enter")]),
            ..row("area-enter")
        },
        Dialog {
            inputs: Some(notes(json!({"multiline": {"max_lines": 2}}))),
            steps: focus(vec![typed("a"), key("enter"), typed("b"), key("enter"), typed("c")]),
            ..row("area-line-limit")
        },
        Dialog {
            inputs: Some(notes(json!({"max_length": 6}))),
            steps: focus(vec![typed("abcdefgh")]),
            ..row("area-char-limit")
        },
        Dialog { inputs: Some(notes(lines())), steps: focus(vec![key("up"), typed("-")]), ..row("area-up") },
        Dialog {
            inputs: Some(notes(lines())),
            steps: focus(vec![key("up"), key("up"), key("down"), typed("-")]),
            ..row("area-up-down")
        },
        Dialog { inputs: Some(notes(lines())), steps: focus(vec![key("pageup"), typed("<")]), ..row("area-page-up") },
        Dialog {
            inputs: Some(notes(lines())),
            steps: focus(vec![key("pageup"), key("pagedown"), typed(">")]),
            ..row("area-page-down")
        },
        Dialog { inputs: Some(notes(lines())), steps: focus(vec![key("home"), typed("^")]), ..row("area-home") },
        Dialog {
            inputs: Some(notes(lines())),
            steps: focus(vec![key("up"), key("end"), typed("$")]),
            ..row("area-end")
        },
    ]
}

fn area_navigation_rows() -> Vec<Value> {
    let notes = |extra: Value| vec![area(true, extra)];
    let focus = |steps: Vec<Value>| [tab3(), steps].concat();
    let lines = || json!({"initial": "alpha\nbeta\ngamma", "max_length": 50});
    rows![
        Dialog {
            inputs: Some(notes(lines())),
            steps: focus(vec![key("ctrl+home"), key("ctrl+end"), typed("!")]),
            ..row("area-ctrl-home-end")
        },
        Dialog {
            inputs: Some(notes(lines())),
            steps: focus(vec![key("up"), key("home"), key("backspace")]),
            ..row("area-backspace-joins")
        },
        Dialog {
            inputs: Some(notes(lines())),
            steps: focus(vec![key("ctrl+a"), typed("x")]),
            ..row("area-select-all")
        },
        Dialog {
            inputs: Some(notes(lines())),
            steps: focus(vec![key("shift+up"), key("shift+left"), key("backspace")]),
            ..row("area-shift-select")
        },
        Dialog {
            inputs: Some(notes(lines())),
            steps: focus(vec![key("ctrl+left"), key("ctrl+left"), typed("_")]),
            ..row("area-word-left")
        },
        Dialog {
            inputs: Some(notes(json!({"initial": "1\n2\n3\n4\n5\n6\n7", "max_length": 50}))),
            steps: focus(vec![key("pageup"), key("down"), key("down"), key("down"), key("down"), key("down")]),
            ..row("area-follows-cursor")
        },
        Dialog { inputs: Some(notes(lines())), steps: focus(vec![key("tab")]), ..row("area-tab-out") },
    ]
}

fn area_mouse_rows() -> Vec<Value> {
    let notes = || vec![area(true, json!({"initial": "alpha\nbeta\ngamma", "max_length": 50}))];
    rows![
        Dialog { inputs: Some(notes()), steps: vec![click(130, 120)], ..row("area-click") },
        Dialog { inputs: Some(notes()), steps: vec![click(130, 120), typed("-")], ..row("area-click-type") },
        Dialog { inputs: Some(notes()), steps: vec![click(300, 140), typed("!")], ..row("area-click-below") },
        Dialog {
            inputs: Some(notes()),
            steps: vec![
                click(115, 107),
                json!({"do": "drag", "x": 140, "y": 125, "dx": 0, "dy": 0}),
                json!({"do": "release", "x": 140, "y": 125})
            ],
            ..row("area-drag-select")
        },
        Dialog {
            inputs: Some(notes()),
            steps: vec![click(130, 120), json!({"do": "click", "x": 150, "y": 107, "shift": true}), typed("#")],
            ..row("area-shift-click")
        },
    ]
}

fn area_value_rows() -> Vec<Value> {
    let send = || vec![button("Send", 150, None, Some(json!({"type": "template", "value": "say $(word)"})))];
    let all = || vec![button("Send", 150, None, Some(json!({"type": "custom_all", "value": "minecraft:answer"})))];
    let notes = |initial: &str| vec![area(true, json!({"initial": initial, "max_length": 50}))];
    let press = || vec![click(213, 247)];
    rows![
        Dialog { inputs: Some(notes("a\nb\"c")), buttons: send(), steps: press(), ..row("area-template") },
        Dialog { inputs: Some(notes("a\nb\"c")), buttons: all(), steps: press(), ..row("area-custom-all") },
        Dialog {
            inputs: Some(notes("")),
            buttons: send(),
            steps: [tab3(), vec![typed("x"), key("enter"), typed("y"), key("tab"), key("enter")]].concat(),
            ..row("area-template-typed")
        },
    ]
}

pub(super) fn rows() -> Vec<Value> {
    [
        layout_rows(),
        editing_rows(),
        mouse_rows(),
        value_rows(),
        area_layout_rows(),
        area_editing_rows(),
        area_navigation_rows(),
        area_mouse_rows(),
        area_value_rows(),
    ]
    .concat()
}
