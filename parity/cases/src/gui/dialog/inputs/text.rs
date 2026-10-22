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

pub(super) fn rows() -> Vec<Value> {
    [layout_rows(), editing_rows(), mouse_rows(), value_rows()].concat()
}
