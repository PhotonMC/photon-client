//! Dialogs that ask for inputs: the checkbox and the actions that read it.

use serde_json::{Value, json};

use super::super::setting::{key, times};
use super::{Dialog, button, click, row};

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

pub(super) fn rows() -> Vec<Value> {
    [checkbox_rows(), action_rows(), option_rows()].concat()
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
