//! Dialogs that ask for inputs: the checkbox and the actions that read it.

use serde_json::{Value, json};

use super::super::setting::{key, times};
use super::{Dialog, button, click, release, row};

fn checkbox(name: &str, label: &str, initial: bool) -> Value {
    json!({"type": "boolean", "key": name, "label": label, "initial": initial, "on_true": "yes", "on_false": "no"})
}

fn option(name: &str, label_visible: bool, options: &[(&str, Option<&str>, bool)]) -> Value {
    let options: Vec<Value> = options
        .iter()
        .map(|(id, display, initial)| {
            let mut entry = json!({"id": id, "initial": initial});
            if let Some(display) = display {
                entry["display"] = json!(display);
            }
            entry
        })
        .collect();
    json!({"type": "single_option", "key": name, "label": "Mode", "width": 200, "label_visible": label_visible, "options": options})
}

fn option_rows() -> Vec<Value> {
    let modes = || option("mode", true, &[("a", Some("Alpha"), false), ("b", None, true), ("c", Some("Gamma"), false)]);
    let send = || vec![button("Send", 150, None, Some(json!({"type": "template", "value": "say $(mode)"})))];
    let all = || vec![button("Send", 150, None, Some(json!({"type": "custom_all", "value": "minecraft:answer"})))];
    rows![
        Dialog { inputs: Some(vec![modes()]), ..row("option") },
        Dialog {
            inputs: Some(vec![option("mode", false, &[("a", Some("Alpha"), false), ("b", None, false)])]),
            ..row("option-value-only")
        },
        Dialog { inputs: Some(vec![modes()]), steps: vec![click(200, 98)], ..row("option-click") },
        Dialog { inputs: Some(vec![modes()]), mouse: (200, 98), ..row("option-hover") },
        Dialog { inputs: Some(vec![modes()]), steps: times(key("tab"), 3), ..row("option-focus") },
        Dialog {
            inputs: Some(vec![modes()]),
            steps: [times(key("tab"), 3), vec![key("enter")]].concat(),
            ..row("option-enter")
        },
        Dialog { inputs: Some(vec![modes()]), steps: vec![click(200, 98), click(200, 98)], ..row("option-wraps") },
        Dialog {
            inputs: Some(vec![modes()]),
            buttons: send(),
            steps: vec![click(213, 223)],
            ..row("option-template-initial")
        },
        Dialog {
            inputs: Some(vec![modes()]),
            buttons: send(),
            steps: vec![click(200, 98), click(213, 223)],
            ..row("option-template-cycled")
        },
        Dialog {
            inputs: Some(vec![modes()]),
            buttons: all(),
            steps: vec![click(200, 98), click(213, 223)],
            ..row("option-custom-all")
        },
        Dialog {
            inputs: Some(vec![modes()]),
            steps: vec![json!({"do": "scroll", "x": 200, "y": 98, "amount": 1})],
            ..row("option-scroll")
        },
    ]
}

fn range(start: f64, end: f64, initial: Option<f64>, step: Option<f64>) -> Value {
    let mut input =
        json!({"type": "number_range", "key": "n", "label": "Amount", "width": 200, "start": start, "end": end});
    if let Some(initial) = initial {
        input["initial"] = json!(initial);
    }
    if let Some(step) = step {
        input["step"] = json!(step);
    }
    input
}

fn drag_to(x: i64) -> Vec<Value> {
    vec![json!({"do": "drag", "x": x, "y": 100, "dx": 0, "dy": 0})]
}

fn slider_rows() -> Vec<Value> {
    let plain = || vec![range(0.0, 10.0, None, None)];
    let stepped = || vec![range(0.0, 10.0, Some(3.0), Some(2.0))];
    let tab3 = || times(key("tab"), 3);
    rows![
        Dialog { inputs: Some(plain()), ..row("slider") },
        Dialog { inputs: Some(stepped()), ..row("slider-stepped") },
        Dialog { inputs: Some(plain()), steps: vec![click(213, 100)], ..row("slider-click") },
        Dialog { inputs: Some(plain()), mouse: (213, 100), ..row("slider-hover") },
        Dialog { inputs: Some(plain()), steps: tab3(), ..row("slider-focus") },
        Dialog { inputs: Some(plain()), steps: [tab3(), vec![key("enter")]].concat(), ..row("slider-focus-locked") },
        Dialog {
            inputs: Some(plain()),
            steps: [tab3(), vec![key("right"), key("right"), key("left")]].concat(),
            ..row("slider-arrows")
        },
        Dialog {
            inputs: Some(plain()),
            steps: [tab3(), vec![key("enter"), key("enter"), key("right")]].concat(),
            ..row("slider-lock-unlock")
        },
        Dialog { inputs: Some(plain()), steps: vec![click(100, 100)], ..row("slider-click-left-edge") },
        Dialog { inputs: Some(plain()), steps: vec![click(330, 100)], ..row("slider-click-right-edge") },
        Dialog {
            inputs: Some(plain()),
            steps: [vec![click(150, 100)], drag_to(250), vec![release(250, 100)]].concat(),
            ..row("slider-drag")
        },
        Dialog { inputs: Some(plain()), steps: [drag_to(250)].concat(), ..row("slider-drag-without-click") },
    ]
}

/// The rows where the range and the steps change the value the slider says.
fn range_rows() -> Vec<Value> {
    let plain = || vec![range(0.0, 10.0, None, None)];
    let stepped = || vec![range(0.0, 10.0, Some(3.0), Some(2.0))];
    let send = || vec![button("Send", 150, None, Some(json!({"type": "template", "value": "say $(n)"})))];
    let all = || vec![button("Send", 150, None, Some(json!({"type": "custom_all", "value": "minecraft:answer"})))];
    rows![
        Dialog {
            inputs: Some(vec![range(10.0, 0.0, None, None)]),
            steps: vec![click(150, 100)],
            ..row("slider-reversed")
        },
        Dialog {
            inputs: Some(vec![range(0.0, 1.0, None, Some(0.1))]),
            steps: vec![click(150, 100)],
            ..row("slider-tenths")
        },
        Dialog {
            inputs: Some(vec![range(0.0, 10.0, Some(0.0), Some(3.0))]),
            steps: vec![click(330, 100)],
            ..row("slider-step-overshoot")
        },
        Dialog {
            inputs: Some(vec![range(-5.0, 5.0, None, Some(1.0))]),
            steps: vec![click(213, 100)],
            ..row("slider-negative")
        },
        Dialog {
            inputs: Some(vec![range(0.0, 100000000.0, None, None)]),
            steps: vec![click(170, 100)],
            ..row("slider-large")
        },
        Dialog { inputs: Some(vec![range(5.0, 5.0, None, None)]), ..row("slider-flat") },
        Dialog {
            inputs: Some(vec![
                json!({"type": "number_range", "key": "n", "label": "Amount", "width": 120, "start": 0, "end": 1, "label_format": "options.percent_value"})
            ]),
            steps: vec![click(213, 100)],
            ..row("slider-format")
        },
        Dialog {
            inputs: Some(plain()),
            buttons: send(),
            steps: vec![click(150, 100), click(213, 223)],
            ..row("slider-template")
        },
        Dialog {
            inputs: Some(plain()),
            buttons: all(),
            steps: vec![click(150, 100), click(213, 223)],
            ..row("slider-custom-all")
        },
        Dialog {
            inputs: Some(stepped()),
            buttons: all(),
            steps: vec![click(330, 100), click(213, 223)],
            ..row("slider-stepped-custom")
        },
    ]
}

pub(super) fn rows() -> Vec<Value> {
    [checkbox_rows(), action_rows(), option_rows(), slider_rows(), range_rows()].concat()
}

fn checkbox_rows() -> Vec<Value> {
    let agree = || vec![checkbox("agree", "Agree", false)];
    let tab3 = || times(key("tab"), 3);
    let toggle = || vec![click(200, 98)];
    rows![
        Dialog { inputs: Some(agree()), ..row("checkbox") },
        Dialog { inputs: Some(vec![checkbox("agree", "Agree", true)]), ..row("checkbox-initial") },
        Dialog { inputs: Some(agree()), steps: toggle(), ..row("checkbox-click") },
        Dialog { inputs: Some(agree()), steps: [toggle(), toggle()].concat(), ..row("checkbox-click-twice") },
        Dialog { inputs: Some(agree()), mouse: (200, 98), ..row("checkbox-hover") },
        Dialog { inputs: Some(agree()), steps: tab3(), ..row("checkbox-focus") },
        Dialog { inputs: Some(agree()), steps: [tab3(), vec![key("space")]].concat(), ..row("checkbox-space") },
        Dialog { inputs: Some(agree()), steps: [tab3(), vec![key("enter")]].concat(), ..row("checkbox-enter") },
        Dialog { inputs: Some(agree()), steps: times(key("tab"), 4), ..row("checkbox-tab-to-button") },
        Dialog {
            inputs: Some(vec![checkbox("agree", "A label that is much longer than the others around it", false)]),
            ..row("checkbox-long-label")
        },
        Dialog {
            inputs: Some(vec![checkbox("a", "First", false), checkbox("b", "Second", true)]),
            ..row("checkboxes")
        },
    ]
}

/// The rows where a button reads the inputs.
fn action_rows() -> Vec<Value> {
    let agree = || vec![checkbox("agree", "Agree", false)];
    let say = || vec![button("Send", 150, None, Some(json!({"type": "template", "value": "say $(agree)"})))];
    let answer = || vec![button("Send", 150, None, Some(json!({"type": "custom_all", "value": "minecraft:answer"})))];
    let toggle = || vec![click(200, 98)];
    rows![
        Dialog { inputs: Some(agree()), buttons: say(), steps: vec![click(213, 223)], ..row("template-default") },
        Dialog {
            inputs: Some(agree()),
            buttons: say(),
            steps: [toggle(), vec![click(213, 223)]].concat(),
            ..row("template-checked")
        },
        Dialog { inputs: Some(agree()), buttons: answer(), steps: vec![click(213, 223)], ..row("custom-all-default") },
        Dialog {
            inputs: Some(agree()),
            buttons: answer(),
            steps: [toggle(), vec![click(213, 223)]].concat(),
            ..row("custom-all-checked")
        },
        Dialog {
            inputs: Some(vec![checkbox("a", "First", false), checkbox("b", "Second", true)]),
            buttons: vec![button("Send", 150, None, Some(json!({"type": "template", "value": "say $(a) $(b) $(c)"})))],
            steps: vec![click(213, 223)],
            ..row("template-two-and-unknown")
        },
        Dialog {
            inputs: Some(vec![checkbox("a", "First", false), checkbox("b", "Second", true)]),
            buttons: answer(),
            steps: vec![click(213, 223)],
            ..row("custom-all-two")
        },
        Dialog {
            kind: Some("multi_action"),
            columns: Some(2),
            inputs: Some(agree()),
            buttons: vec![
                button("One", 100, None, Some(json!({"type": "template", "value": "say $(agree)"}))),
                button("Two", 100, None, Some(json!({"type": "custom_all", "value": "minecraft:answer"}))),
            ],
            steps: [times(key("tab"), 5), vec![key("enter")]].concat(),
            ..row("multi-action-checkbox")
        },
        Dialog {
            kind: Some("multi_action"),
            columns: Some(2),
            inputs: Some(agree()),
            buttons: vec![button("One", 100, None, Some(json!({"type": "template", "value": "say $(agree)"})))],
            steps: vec![click(200, 98), key("tab"), key("tab"), key("tab"), key("tab"), key("enter")],
            ..row("multi-action-checkbox-checked")
        },
    ]
}
