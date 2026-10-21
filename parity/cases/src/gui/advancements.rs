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

/// An advancement for the screen: the title, the description, the frame, the place and whether it
/// is hidden, with a background when it is a root.
struct Shown {
    id: &'static str,
    parent: Option<&'static str>,
    title: &'static str,
    description: &'static str,
    frame: i64,
    place: (f32, f32),
    hidden: bool,
    requirements: &'static [&'static [&'static str]],
}

impl Shown {
    fn new(id: &'static str, parent: Option<&'static str>, place: (f32, f32)) -> Self {
        Self {
            id,
            parent,
            title: "An Advancement",
            description: "Do the thing",
            frame: 0,
            place,
            hidden: false,
            requirements: &[&["a"]],
        }
    }

    fn json(&self) -> Value {
        let background = self.parent.is_none().then_some("minecraft:gui/advancements/backgrounds/stone");
        json!({
            "id": self.id, "parent": self.parent, "requirements": self.requirements,
            "display": {
                "title": self.title, "description": self.description, "frame": self.frame,
                "show_toast": true, "hidden": self.hidden, "background": background,
                "x": f32_bits(self.place.0), "y": f32_bits(self.place.1),
            },
        })
    }
}

fn screen_row(label: &str, before: Vec<Value>, steps: Vec<Value>) -> Value {
    json!({"label": label, "width": 427, "height": 240, "mouse": [0, 0], "before": before, "steps": steps})
}

/// The points the pointer is put on to find the widgets of the trees below.
const HOVER_POINTS: [(i32, i32); 8] =
    [(173, 124), (201, 124), (229, 151), (257, 124), (162, 113), (185, 136), (201, 97), (150, 124)];

fn with_mouse(mut row: Value, mouse: (i32, i32), steps: Vec<Value>) -> Value {
    row["mouse"] = json!([mouse.0, mouse.1]);
    row["steps"] = Value::Array(steps);
    row
}

fn ticks(point: (i32, i32), count: usize) -> Vec<Value> {
    let tick = json!({"do": "tick", "x": point.0, "y": point.1});
    std::iter::repeat_n(tick, count).collect()
}

/// A tree of a root and a few under it, of every frame, with the places spread.
fn spread_tree(prefix: &str, root_title: &'static str) -> Vec<Shown> {
    let root_id: &'static str = Box::leak(format!("{prefix}:root").into_boxed_str());
    let id = |name: &str| -> &'static str { Box::leak(format!("{prefix}:{name}").into_boxed_str()) };
    vec![
        Shown { title: root_title, ..Shown::new(root_id, None, (0.0, 0.0)) },
        Shown { frame: 1, title: "A Challenge", ..Shown::new(id("a"), Some(root_id), (1.0, 0.0)) },
        Shown { frame: 2, title: "A Goal", ..Shown::new(id("b"), Some(id("a")), (2.0, 1.0)) },
        Shown { hidden: true, title: "Hidden", ..Shown::new(id("c"), Some(root_id), (1.0, -1.0)) },
        Shown {
            title: "A Title That Is Long Enough To Wrap Over Several Lines Of The Box",
            description: "A description that is long as well, so that the lines of the box are broken in more than one place",
            requirements: &[&["a"], &["b"], &["c"], &["d"]],
            ..Shown::new(id("d"), Some(id("b")), (3.0, 0.0))
        },
    ]
}

/// The tree above, grown past the window on both axes so the pane can move.
fn wide_tree(prefix: &str, root_title: &'static str) -> Vec<Shown> {
    let mut shown = spread_tree(prefix, root_title);
    let id = |name: &str| -> &'static str { Box::leak(format!("{prefix}:{name}").into_boxed_str()) };
    shown.extend([
        Shown::new(id("e"), Some(id("d")), (6.0, 0.0)),
        Shown::new(id("f"), Some(id("e")), (9.0, 2.0)),
        Shown::new(id("g"), Some(id("a")), (4.0, -2.0)),
        Shown::new(id("h"), Some(id("b")), (5.0, 3.0)),
        Shown::new(id("i"), Some(id("f")), (10.0, -1.0)),
    ]);
    shown
}

/// Ten trees, so the tabs run past the three above onto the other sides.
fn many_roots() -> Vec<Value> {
    roots(10)
}

fn roots(count: usize) -> Vec<Value> {
    (0..count)
        .map(|index| {
            let prefix: &'static str = Box::leak(format!("r{index}").into_boxed_str());
            tree_update(&spread_tree(prefix, "Root"), vec![])
        })
        .collect()
}

fn tree_update(shown: &[Shown], progress: Vec<Value>) -> Value {
    update(false, shown.iter().map(Shown::json).collect(), &[], progress, true)
}

/// `AdvancementsScreen`: the window, the tabs, the tree, and the pointer over a widget.
pub fn advancements_screen() -> Value {
    let one = spread_tree("one", "Minecraft");
    let progress = vec![
        done("one:a", &[("a", Some(1))]),
        done("one:b", &[("a", None)]),
        done("one:d", &[("a", Some(2)), ("b", None), ("c", Some(3)), ("d", None)]),
    ];
    let progressed = tree_update(&one, progress);
    let tabs = vec![
        tree_update(&one, vec![]),
        tree_update(&spread_tree("two", "Nether"), vec![]),
        tree_update(&spread_tree("three", "The End"), vec![]),
    ];
    let mut rows = vec![
        screen_row("empty", vec![], vec![]),
        screen_row("tree", vec![tabs[0].clone()], vec![]),
        screen_row("progress", vec![progressed.clone()], vec![]),
        screen_row("tabs", tabs[..2].to_vec(), vec![]),
        screen_row("three-tabs", tabs.clone(), vec![]),
    ];
    rows.extend(hover_rows(&progressed));
    let wide =
        vec![tree_update(&wide_tree("one", "Minecraft"), vec![]), tree_update(&wide_tree("two", "Nether"), vec![])];
    rows.extend(motion_rows(&wide));
    rows.push(screen_row("many-roots", many_roots(), vec![]));
    rows.push(screen_row("roots-past-every-tab-slot", roots(40), vec![]));
    rows.extend(hidden_rows(&one));
    rows.extend(window_rows(&tabs));
    gui("advancements-screen", "advancements_screen", rows)
}

/// A hidden widget shows only once its own progress is done, not when its parent is.
fn hidden_rows(one: &[Shown]) -> Vec<Value> {
    let hidden_at = (201, 97);
    let after_root = tree_update(one, vec![done("one:root", &[("a", Some(1))])]);
    let mut rows = Vec::new();
    for (label, before) in [
        ("hidden-hover-none-done", tree_update(one, vec![])),
        ("hidden-hover-parent-done", after_root.clone()),
        ("hidden-hover-own-done", tree_update(one, vec![done("one:c", &[("a", Some(1))])])),
    ] {
        let mut steps = ticks(hidden_at, 12);
        steps.push(json!({"do": "frame"}));
        rows.push(with_mouse(screen_row(label, vec![before], vec![]), hidden_at, steps));
    }
    rows
}

/// The pointer left on points of the tree, long enough for the box to fade in.
fn hover_rows(progressed: &Value) -> Vec<Value> {
    HOVER_POINTS
        .into_iter()
        .enumerate()
        .map(|(index, point)| {
            let mut steps = ticks(point, 12);
            steps.push(json!({"do": "frame"}));
            with_mouse(screen_row(&format!("hover-{index}"), vec![progressed.clone()], vec![]), point, steps)
        })
        .collect()
}

fn drag(dx: f64, dy: f64) -> Value {
    json!({"do": "drag", "x": 200, "y": 118, "dx": dx, "dy": dy})
}

fn release() -> Value {
    json!({"do": "release", "x": 200, "y": 118})
}

fn scroll(sx: f64, sy: f64) -> Value {
    json!({"do": "scroll", "x": 200, "y": 118, "sx": sx, "sy": sy})
}

/// A progress for a tab's advancement, as an update step.
fn server_update(id: &str, reset: bool) -> Value {
    let progress = if reset { vec![] } else { vec![done(id, &[("a", Some(5))])] };
    let mut step = update(reset, vec![], &[], progress, false);
    step["do"] = json!("update");
    step
}

/// The tree moved by drags and the wheel, resized, changed by the server.
fn motion_rows(tabs: &[Value]) -> Vec<Value> {
    [
        ("drag", vec![drag(5.0, 5.0), drag(40.0, -20.0), release()]),
        ("drag-far", vec![drag(1.0, 1.0), drag(4000.0, 4000.0)]),
        ("drag-after-release", vec![drag(1.0, 1.0), release(), drag(30.0, 30.0)]),
        ("scroll", vec![scroll(0.0, -1.0)]),
        ("scroll-wide", vec![scroll(3.0, 2.0)]),
        ("resize", vec![json!({"do": "resize", "width": 640, "height": 360})]),
        ("resize-small", vec![json!({"do": "resize", "width": 200, "height": 120})]),
        ("tab-step", vec![json!({"do": "tab", "id": "two:root"})]),
        ("tab-unknown", vec![json!({"do": "tab", "id": "nowhere:root"})]),
        ("update-after", vec![server_update("two:a", false)]),
        ("reset", vec![server_update("", true)]),
    ]
    .into_iter()
    .map(|(label, steps)| screen_row(label, tabs.to_vec(), steps))
    .collect()
}

/// The window's own elements: the tab buttons and Done, pressed and hovered.
fn window_rows(tabs: &[Value]) -> Vec<Value> {
    let mut rows = Vec::new();
    for (label, point) in [("click-tab-1", (130, 45)), ("click-tab-2", (160, 45)), ("click-nothing", (10, 10))] {
        let steps = vec![json!({"do": "click", "x": point.0, "y": point.1})];
        rows.push(with_mouse(screen_row(label, tabs.to_vec(), vec![]), point, steps));
    }
    for (label, point) in
        [("tab-hover-0", (120, 45)), ("tab-hover-1", (150, 45)), ("tab-hover-2", (180, 45)), ("done-hover", (213, 215))]
    {
        rows.push(with_mouse(screen_row(label, tabs.to_vec(), vec![]), point, ticks(point, 3)));
    }
    rows
}
