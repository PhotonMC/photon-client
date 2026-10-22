//! Dialogs that ask for inputs: the checkbox and the actions that read it.

use serde_json::{Value, json};

use super::super::setting::{key, times};
use super::{Dialog, button, click, row};

fn checkbox(name: &str, label: &str, initial: bool) -> Value {
    json!({"type": "boolean", "key": name, "label": label, "initial": initial, "on_true": "yes", "on_false": "no"})
}

pub(super) fn rows() -> Vec<Value> {
    [checkbox_rows(), action_rows()].concat()
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
