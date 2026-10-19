//! The advancements the client holds (`ClientAdvancements`).

use serde_json::{Value, json};

use super::gui;
use crate::bits::f32_bits;

/// An advancement: the id, the parent, a display (or none) and the requirements.
fn advancement(id: &str, parent: Option<&str>, display: bool, requirements: &[&[&str]]) -> Value {
    let shown = display.then(|| {
        json!({"title": format!("{id} title"), "description": "d", "frame": 0, "show_toast": true, "hidden": false,
            "x": f32_bits(1.5), "y": f32_bits(-2.0)})
    });
    json!({"id": id, "parent": parent, "display": shown, "requirements": requirements})
}

/// A display that shows no toast.
fn quiet(mut value: Value) -> Value {
    value["display"]["show_toast"] = json!(false);
    value
}

fn update(reset: bool, added: Vec<Value>, removed: &[&str], progress: Vec<Value>, show: bool) -> Value {
    json!({"reset": reset, "added": added, "removed": removed, "progress": progress, "show": show})
}

fn done(id: &str, criteria: &[(&str, Option<i64>)]) -> Value {
    let criteria: Vec<Value> = criteria.iter().map(|(name, obtained)| json!([name, obtained])).collect();
    json!({"id": id, "criteria": criteria})
}

fn row(label: &str, steps: Vec<Value>) -> Value {
    json!({"label": label, "steps": steps})
}

/// `ClientAdvancements`: the tree built in any order, removed with what is under it, reset, and the
/// progress with its percent, text and the toasts.
pub fn client_advancements() -> Value {
    let rows = tree_rows().into_iter().chain(removal_rows()).chain(progress_rows()).collect();
    gui("client-advancements", "client_advancements", rows)
}

fn root() -> Value {
    advancement("test:root", None, true, &[&["a"]])
}

fn child() -> Value {
    advancement("test:child", Some("test:root"), true, &[&["a"], &["b"], &["c"]])
}

fn grand() -> Value {
    advancement("test:grand", Some("test:child"), true, &[&["a", "b"]])
}

/// The tree put together: in order, children before their parents, orphans, a second root.
fn tree_rows() -> Vec<Value> {
    let other = || advancement("test:other", None, true, &[&["x"]]);
    let lost = || advancement("test:lost", Some("test:missing"), true, &[&["x"]]);
    vec![
        row("empty", vec![update(false, vec![], &[], vec![], true)]),
        row("tree-in-order", vec![update(false, vec![root(), child(), grand()], &[], vec![], true)]),
        row("children-first", vec![update(false, vec![grand(), child(), root()], &[], vec![], true)]),
        row("two-roots", vec![update(false, vec![root(), other(), child()], &[], vec![], true)]),
        row("orphan", vec![update(false, vec![child(), lost()], &[], vec![], true)]),
        row(
            "orphan-found-later",
            vec![update(false, vec![child()], &[], vec![], true), update(false, vec![root()], &[], vec![], true)],
        ),
        row(
            "re-add",
            vec![
                update(false, vec![root(), child()], &[], vec![], true),
                update(false, vec![root(), child()], &[], vec![], true),
            ],
        ),
        row(
            "tab",
            vec![
                update(false, vec![root(), child()], &[], vec![], true),
                json!({"tab": "test:root"}),
                json!({"tab": "test:child"}),
                json!({"tab": "test:missing"}),
                json!({"tab": null}),
            ],
        ),
    ]
}

/// Removals and the reset.
fn removal_rows() -> Vec<Value> {
    let other = || advancement("test:other", None, true, &[&["x"]]);
    let tree = || update(false, vec![root(), child(), grand()], &[], vec![], true);
    vec![
        row("remove-parent", vec![tree(), update(false, vec![], &["test:child"], vec![], true)]),
        row(
            "remove-root",
            vec![
                update(false, vec![root(), child(), other()], &[], vec![], true),
                update(false, vec![], &["test:root"], vec![], true),
            ],
        ),
        row("remove-unknown", vec![update(false, vec![root()], &["test:nothing"], vec![], true)]),
        row(
            "remove-and-add-same-packet",
            vec![
                update(false, vec![root(), child()], &[], vec![], true),
                update(false, vec![child()], &["test:root"], vec![], true),
            ],
        ),
        row(
            "reset",
            vec![
                update(false, vec![root(), child()], &[], vec![done("test:root", &[("a", Some(1))])], true),
                update(true, vec![advancement("test:new", None, true, &[&["x"]])], &[], vec![], true),
            ],
        ),
    ]
}

/// The tree the progress rows share: a root with children of every sort of requirement.
fn progress_tree() -> Value {
    update(
        false,
        vec![
            advancement("test:root", None, true, &[&["a"]]),
            advancement("test:child", Some("test:root"), true, &[&["a"], &["b"], &["c"]]),
            quiet(advancement("test:quiet", Some("test:root"), true, &[&["a"]])),
            advancement("test:plain", Some("test:root"), false, &[&["a"]]),
            advancement("test:anyof", Some("test:root"), true, &[&["a", "b", "c"]]),
            advancement("test:empty", Some("test:root"), true, &[]),
        ],
        &[],
        vec![],
        true,
    )
}

fn progress_step(progress: Vec<Value>, reset: bool, show: bool) -> Value {
    update(reset, vec![], &[], progress, show)
}

/// Progress: how far a child is, when it is done and when its toast shows.
fn progress_rows() -> Vec<Value> {
    let child_progress = |criteria: &[(&str, Option<i64>)]| vec![done("test:child", criteria)];
    let all = [("a", Some(5)), ("b", Some(6)), ("c", Some(7))];
    vec![
        row(
            "none-done",
            vec![progress_tree(), progress_step(child_progress(&[("a", None), ("b", None), ("c", None)]), false, true)],
        ),
        row(
            "one-of-three",
            vec![
                progress_tree(),
                progress_step(child_progress(&[("a", Some(5)), ("b", None), ("c", None)]), false, true),
            ],
        ),
        row(
            "two-of-three",
            vec![
                progress_tree(),
                progress_step(child_progress(&[("a", Some(5)), ("b", Some(6)), ("c", None)]), false, true),
            ],
        ),
        row("all-done-toast", vec![progress_tree(), progress_step(child_progress(&all), false, true)]),
        row("all-done-hidden-show", vec![progress_tree(), progress_step(child_progress(&all), false, false)]),
        row(
            "all-done-on-reset",
            vec![update(
                true,
                vec![advancement("test:root", None, true, &[&["a"]])],
                &[],
                vec![done("test:root", &[("a", Some(1))])],
                true,
            )],
        ),
        row(
            "unknown-criterion-dropped",
            vec![progress_tree(), progress_step(child_progress(&[("zzz", Some(1)), ("a", Some(2))]), false, true)],
        ),
        row(
            "progress-twice",
            vec![
                progress_tree(),
                progress_step(child_progress(&[("a", Some(5))]), false, true),
                progress_step(child_progress(&[("b", Some(6))]), false, true),
            ],
        ),
        row(
            "progress-after-removal",
            vec![
                progress_tree(),
                progress_step(child_progress(&[("a", Some(5))]), false, true),
                update(false, vec![], &["test:child"], vec![], true),
            ],
        ),
    ]
    .into_iter()
    .chain(special_progress_rows())
    .collect()
}

/// The advancements that show no toast, or have no requirements, or are not in the tree.
fn special_progress_rows() -> Vec<Value> {
    let one = |id: &str, criteria: &[(&str, Option<i64>)]| {
        row(
            &format!("progress-{}", id.trim_start_matches("test:")),
            vec![progress_tree(), progress_step(vec![done(id, criteria)], false, true)],
        )
    };
    vec![
        one("test:quiet", &[("a", Some(1))]),
        one("test:plain", &[("a", Some(1))]),
        one("test:anyof", &[("a", None), ("b", Some(2)), ("c", Some(3))]),
        one("test:empty", &[("a", Some(1))]),
        one("test:nothing", &[("a", Some(1))]),
        row(
            "several-done-at-once",
            vec![
                progress_tree(),
                progress_step(
                    vec![done("test:root", &[("a", Some(1))]), done("test:anyof", &[("a", Some(1))])],
                    false,
                    true,
                ),
            ],
        ),
        row(
            "progress-with-the-parent-after",
            vec![update(
                false,
                vec![
                    advancement("test:child", Some("test:root"), true, &[&["a"]]),
                    advancement("test:root", None, true, &[&["a"]]),
                ],
                &[],
                vec![done("test:child", &[("a", Some(1))])],
                true,
            )],
        ),
    ]
}
