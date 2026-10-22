//! Server dialogs: the notice and confirmation screen, and the waiting
//! screen.

use serde_json::{Value, json};

use super::gui;
use super::setting::{frame, key, times};

fn message(text: &str, width: i64) -> Value {
    json!({"text": text, "width": width})
}

fn item(width: i64, height: i64, description: Option<Value>) -> Value {
    let mut body = json!({"item": true, "width": width, "height": height});
    if let Some(description) = description {
        body["description"] = description;
    }
    body
}

fn button(label: &str, width: i64, tooltip: Option<&str>, action: Option<Value>) -> Value {
    let mut data = json!({"label": label, "width": width});
    if let Some(tooltip) = tooltip {
        data["tooltip"] = json!(tooltip);
    }
    if let Some(action) = action {
        data["action"] = action;
    }
    data
}

fn ok() -> Value {
    button("OK", 150, None, None)
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn scroll(x: i64, y: i64, amount: i64) -> Value {
    json!({"do": "scroll", "x": x, "y": y, "amount": amount})
}

fn drag(x: i64, y: i64, dy: i64) -> Value {
    json!({"do": "drag", "x": x, "y": y, "dx": 0, "dy": dy})
}

fn release(x: i64, y: i64) -> Value {
    json!({"do": "release", "x": x, "y": y})
}

fn ticks(count: i64) -> Value {
    json!({"do": "tick", "count": count})
}

fn command(text: &str) -> Value {
    json!({"type": "run_command", "value": text})
}

fn custom(name: &str) -> Value {
    json!({"type": "custom", "value": name})
}

/// A notice or confirmation scenario.
#[derive(Clone)]
struct Dialog {
    label: &'static str,
    size: (i64, i64),
    mouse: (i64, i64),
    in_world: bool,
    steps: Vec<Value>,
    ms: i64,
    body: Vec<Value>,
    buttons: Vec<Value>,
    title: &'static str,
    can_close: bool,
    after: &'static str,
    kind: Option<&'static str>,
    columns: Option<i64>,
    exit: Option<Value>,
    links: Option<Vec<Value>>,
    button_width: Option<i64>,
}

fn row(label: &'static str) -> Dialog {
    Dialog {
        label,
        size: (427, 240),
        mouse: (0, 0),
        in_world: true,
        steps: vec![],
        ms: 0,
        body: vec![message("Hello there.", 200)],
        buttons: vec![ok()],
        title: "Notice",
        can_close: true,
        after: "CLOSE",
        kind: None,
        columns: None,
        exit: None,
        links: None,
        button_width: None,
    }
}

impl Dialog {
    fn into_row(self) -> Value {
        let kind = self.kind.unwrap_or(if self.buttons.len() == 2 { "confirmation" } else { "notice" });
        let mut dialog = json!({
            "kind": kind, "title": self.title, "body": self.body, "buttons": self.buttons,
            "can_close": self.can_close, "after": self.after,
        });
        if let Some(columns) = self.columns {
            dialog["columns"] = json!(columns);
        }
        if let Some(exit) = self.exit {
            dialog["exit"] = exit;
        }
        if let Some(links) = self.links {
            dialog["links"] = json!(links);
            dialog["button_width"] = json!(self.button_width.unwrap_or(150));
        }
        json!({
            "label": self.label, "width": self.size.0, "height": self.size.1, "mouse": [self.mouse.0, self.mouse.1],
            "in_world": self.in_world, "dialog": dialog, "steps": self.steps, "ms": self.ms,
        })
    }
}

/// The rows of the dialogs listed.
macro_rules! rows {
    ($($dialog:expr),* $(,)?) => { vec![$($dialog.into_row()),*] };
}

fn waiting(label: &str, steps: Vec<Value>, mouse: (i64, i64), in_world: bool, size: (i64, i64)) -> Value {
    json!({
        "label": label, "width": size.0, "height": size.1, "mouse": [mouse.0, mouse.1], "in_world": in_world,
        "dialog": {"kind": "waiting"}, "steps": steps, "ms": 0,
    })
}

fn notice_rows() -> Vec<Value> {
    let long_text = "This message is long enough that it has to wrap onto several lines of the dialog \
                     before the button, and it keeps going so that more lines are needed too.";
    let confirm =
        vec![button("Yes", 100, None, Some(command("say yes"))), button("No", 100, None, Some(custom("minecraft:no")))];
    let text = |text: &str, width: i64| vec![message(text, width)];
    rows![
        row("notice"),
        Dialog { in_world: false, ..row("notice-out-of-world") },
        Dialog { body: vec![], ..row("no-body") },
        Dialog {
            body: vec![message("First.", 200), message("Second, a little longer than the first.", 200)],
            ..row("two-bodies")
        },
        Dialog { body: text(long_text, 200), ..row("wrapped") },
        Dialog { body: text(long_text, 100), ..row("wrapped-narrow") },
        Dialog { body: text(long_text, 400), ..row("wide-message") },
        Dialog { size: (320, 240), ..row("small") },
        Dialog { size: (854, 480), ..row("large") },
        Dialog { size: (200, 150), body: text(long_text, 200), ..row("tiny") },
        Dialog { body: text("", 200), ..row("empty-message") },
        Dialog { title: "A title that is long enough to push the warning button out of view", ..row("long-title") },
        Dialog { buttons: confirm.clone(), ..row("confirmation") },
        Dialog {
            buttons: vec![button("Accept", 200, None, None), button("Decline", 120, None, None)],
            ..row("confirmation-wide")
        },
        Dialog {
            mouse: (200, 224),
            steps: vec![frame(0)],
            ms: 700,
            buttons: vec![button("OK", 150, Some("Closes the dialog."), None)],
            ..row("button-tooltip-hover")
        },
        Dialog { mouse: (200, 224), ..row("hover-button") },
        Dialog { mouse: (230, 15), ..row("hover-warning") },
        Dialog { mouse: (230, 15), steps: vec![frame(0)], ms: 800, ..row("hover-warning-tooltip") },
        Dialog { body: vec![item(16, 16, None)], ..row("item") },
        Dialog { body: vec![item(64, 48, None)], ..row("item-large") },
        Dialog { body: vec![item(16, 16, Some(message("A shiny diamond.", 120)))], ..row("item-description") },
        Dialog {
            body: vec![message("Look:", 200), item(16, 16, None), message("That one.", 200)],
            ..row("item-and-text")
        },
    ]
    .into_iter()
    .collect()
}

fn confirm() -> Vec<Value> {
    vec![button("Yes", 100, None, Some(command("say yes"))), button("No", 100, None, Some(custom("minecraft:no")))]
}

fn run() -> Vec<Value> {
    vec![button("Run", 150, None, Some(command("say hi")))]
}

fn paragraphs() -> Vec<Value> {
    (0..14).map(|n| message(&format!("Paragraph {n}: the body of this dialog runs on for a while."), 200)).collect()
}

fn tabs(count: usize) -> Vec<Value> {
    times(key("tab"), count)
}

fn with(label: &'static str, steps: Vec<Value>) -> Dialog {
    Dialog { steps, ..row(label) }
}

fn scroll_rows() -> Vec<Value> {
    let confirm = confirm();
    let long_drag = vec![click(305, 80), drag(305, 140, 60)];
    let on = |label: &'static str, steps: Vec<Value>| Dialog { body: paragraphs(), steps, ..row(label) };
    rows![
        on("scrolling", vec![]),
        Dialog { size: (320, 180), ..on("scrolling-small", vec![]) },
        on("scroll-down", vec![scroll(213, 120, -1)]),
        on("scroll-down-far", vec![scroll(213, 120, -50)]),
        on("scroll-up-clamped", vec![scroll(213, 120, 3)]),
        on("scroll-outside", vec![scroll(5, 120, -3)]),
        with("scroll-no-room", vec![scroll(213, 100, -3)]),
        on("drag-bar", long_drag.clone()),
        on("drag-bar-released", [long_drag, vec![release(305, 140), drag(305, 200, 60)]].concat()),
        on("drag-above", vec![scroll(213, 120, -5), click(305, 80), drag(305, 10, -60)]),
        with("tab", tabs(1)),
        with("tab-2", tabs(2)),
        with("tab-3", tabs(3)),
        with("tab-4-round", tabs(4)),
        with("shift-tab", vec![key("shift+tab")]),
        with("shift-tab-2", times(key("shift+tab"), 2)),
        Dialog { buttons: confirm.clone(), ..with("tab-confirmation", tabs(4)) },
        Dialog { body: vec![item(16, 16, None), message("Text", 200)], ..with("tab-item", tabs(2)) },
        on("tab-into-view", tabs(8)),
        on("tab-into-view-end", tabs(15)),
        on("tab-back-into-view", times(key("shift+tab"), 3)),
    ]
}

fn press_rows() -> Vec<Value> {
    let confirm = confirm();
    let go = || vec![button("Go", 150, None, Some(custom("minecraft:go")))];
    let more = paragraphs()[..2].to_vec();
    let clicked = |label: &'static str, after: &'static str, buttons: Vec<Value>| Dialog {
        mouse: (200, 224),
        steps: vec![click(200, 224)],
        after,
        buttons,
        ..row(label)
    };
    rows![
        Dialog { body: more, mouse: (213, 100), ..with("click-text", vec![click(213, 100)]) },
        with("click-gap", vec![click(10, 10)]),
        with("enter-unfocused", vec![key("enter")]),
        Dialog { buttons: run(), ..with("enter-button", [tabs(3), vec![key("enter")]].concat()) },
        Dialog {
            buttons: vec![button("Run", 150, None, Some(command("/say hi")))],
            ..with("space-button", [tabs(3), vec![key("space")]].concat())
        },
        with("enter-no-action", [tabs(3), vec![key("enter")]].concat()),
        with("enter-on-text", vec![key("tab"), key("enter")]),
        Dialog { body: vec![], ..with("enter-warning", vec![key("tab"), key("enter")]) },
        clicked("click-button-run", "CLOSE", run()),
        clicked("click-button-custom", "CLOSE", go()),
        clicked("click-button-none", "NONE", run()),
        clicked("click-button-wait", "WAIT_FOR_RESPONSE", run()),
        clicked("click-button-wait-no-action", "WAIT_FOR_RESPONSE", vec![ok()]),
        clicked("click-button-none-no-action", "NONE", vec![ok()]),
        Dialog { mouse: (160, 224), buttons: confirm.clone(), ..with("click-yes", vec![click(160, 224)]) },
        Dialog { mouse: (268, 224), buttons: confirm.clone(), ..with("click-no", vec![click(268, 224)]) },
        Dialog { mouse: (230, 15), ..with("click-warning", vec![click(230, 15)]) },
        with("escape", vec![key("escape")]),
        Dialog { buttons: run(), ..with("escape-action", vec![key("escape")]) },
        Dialog { buttons: confirm.clone(), ..with("escape-confirmation", vec![key("escape")]) },
        Dialog { can_close: false, ..with("escape-shut", vec![key("escape")]) },
        Dialog { after: "WAIT_FOR_RESPONSE", buttons: confirm.clone(), ..with("escape-wait", vec![key("escape")]) },
        Dialog { after: "NONE", buttons: confirm, ..with("escape-none", vec![key("escape")]) },
    ]
}

fn waiting_rows() -> Vec<Value> {
    let asleep = |label: &str, count: i64| waiting(label, vec![ticks(count)], (0, 0), true, (427, 240));
    let at_rest = |label: &str, steps: Vec<Value>, mouse: (i64, i64)| waiting(label, steps, mouse, true, (427, 240));
    vec![
        waiting("waiting", vec![], (0, 0), true, (427, 240)),
        waiting("waiting-out-of-world", vec![], (0, 0), false, (427, 240)),
        asleep("waiting-19", 19),
        asleep("waiting-20", 20),
        asleep("waiting-21", 21),
        asleep("waiting-40", 40),
        asleep("waiting-60", 60),
        asleep("waiting-80", 80),
        asleep("waiting-99", 99),
        asleep("waiting-100", 100),
        asleep("waiting-101", 101),
        asleep("waiting-300", 300),
        waiting("waiting-small", vec![ticks(100)], (0, 0), true, (320, 240)),
        waiting("waiting-short", vec![ticks(100)], (0, 0), true, (427, 60)),
        at_rest("waiting-hover", vec![ticks(100)], (213, 73)),
        at_rest("waiting-click-asleep", vec![ticks(40), click(213, 73)], (213, 73)),
        at_rest("waiting-click-awake", vec![ticks(100), click(213, 73)], (213, 73)),
        at_rest("waiting-escape-asleep", vec![ticks(40), key("escape")], (0, 0)),
        at_rest("waiting-escape-awake", vec![ticks(100), key("escape")], (0, 0)),
    ]
}

impl Dialog {
    fn steps(self, steps: Vec<Value>) -> Self {
        Self { steps, ..self }
    }

    fn at(self, x: i64, y: i64) -> Self {
        Self { mouse: (x, y), ..self }
    }

    fn after(self, after: &'static str) -> Self {
        Self { after, ..self }
    }

    fn leave(self) -> Self {
        Self { exit: Some(button("Leave", 200, None, Some(custom("minecraft:leave")))), ..self }
    }
}

/// A dialog of `count` option buttons in `columns` columns.
fn listing(label: &'static str, count: usize, columns: i64) -> Dialog {
    let buttons = (0..count).map(|n| option(n, 150, None)).collect();
    Dialog {
        buttons,
        title: "Choose",
        kind: Some("multi_action"),
        columns: Some(columns),
        body: vec![message("Pick one.", 200)],
        ..row(label)
    }
}

fn option(n: usize, width: i64, tooltip: Option<&str>) -> Value {
    button(&format!("Option {n}"), width, tooltip, Some(command(&format!("say {n}"))))
}

fn list_rows() -> Vec<Value> {
    let tabs = |count: usize| times(key("tab"), count);
    let entered = |count: usize| [tabs(count), vec![key("enter")]].concat();
    let scrolled = |amount: i64| vec![scroll(213, 120, amount)];
    let escape = || vec![key("escape")];
    rows![
        listing("list", 3, 2),
        listing("list-one", 1, 2),
        listing("list-odd", 3, 2),
        listing("list-even", 4, 2),
        listing("list-three-columns", 7, 3),
        listing("list-one-column", 4, 1),
        listing("list-wide-columns", 5, 9),
        Dialog { buttons: (0..5).map(|n| option(n, 60, None)).collect(), ..listing("list-narrow-buttons", 5, 2) },
        listing("list-exit", 4, 2).leave(),
        listing("list-exit-odd", 3, 2).leave(),
        Dialog { body: vec![], ..listing("list-no-body", 4, 2) },
        listing("list-many", 30, 2),
        listing("list-many-exit", 30, 3).leave(),
        Dialog { size: (320, 180), ..listing("list-small", 9, 2) },
        Dialog { size: (854, 480), ..listing("list-tall", 12, 1) },
        listing("list-hover", 4, 2).at(150, 90),
        Dialog {
            buttons: (0..4).map(|n| option(n, 150, (n == 0).then_some("Tip 0"))).collect(),
            ..listing("list-hover-tooltip", 4, 2).at(150, 90).steps(vec![frame(0)])
        },
        listing("list-hover-exit", 4, 2).leave().at(213, 224),
        listing("list-scroll", 30, 2).steps(scrolled(-2)),
        listing("list-scroll-hover", 30, 2).steps(scrolled(-2)).at(150, 100),
        listing("list-scroll-far", 30, 2).steps(scrolled(-50)),
        listing("list-hover-clipped", 30, 2).at(150, 40),
        listing("list-tab", 4, 2).steps(tabs(2)),
        listing("list-tab-button", 4, 2).steps(tabs(4)),
        listing("list-tab-exit", 4, 2).leave().steps(tabs(6)),
        listing("list-tab-into-view", 30, 2).steps(tabs(12)),
        listing("list-tab-into-view-end", 30, 2).leave().steps(tabs(31)),
        listing("list-shift-tab", 30, 2).steps(times(key("shift+tab"), 2)),
        listing("list-enter", 4, 2).steps(entered(3)),
        listing("list-enter-exit", 4, 2).leave().steps(entered(7)),
        listing("list-click", 4, 2).steps(vec![click(150, 90)]).at(150, 90),
        listing("list-click-second", 4, 2).steps(vec![click(270, 90)]).at(270, 90),
        listing("list-click-last-row", 3, 2).steps(vec![click(213, 114)]).at(213, 114),
        listing("list-click-none", 4, 2).steps(vec![click(213, 115)]).after("NONE"),
        listing("list-click-wait", 4, 2).steps(vec![click(150, 90)]).after("WAIT_FOR_RESPONSE"),
        listing("list-click-scrolled", 30, 2).steps(vec![scroll(213, 120, -3), click(150, 100)]).at(150, 100),
        listing("list-click-clipped", 30, 2).steps(vec![click(150, 40)]).at(150, 40),
        listing("list-click-exit", 4, 2).leave().steps(vec![click(213, 224)]).at(213, 224),
        listing("list-escape", 4, 2).steps(escape()),
        listing("list-escape-exit", 4, 2).leave().steps(escape()),
        Dialog { can_close: false, ..listing("list-escape-shut", 4, 2).leave().steps(escape()) },
    ]
}

/// A `ServerLinksDialog` with the links given.
fn linked(label: &'static str, links: Vec<Value>, columns: i64) -> Dialog {
    Dialog {
        kind: Some("server_links"),
        columns: Some(columns),
        links: Some(links),
        buttons: vec![],
        title: "Server Links",
        body: vec![],
        ..row(label)
    }
}

fn site(n: usize) -> Value {
    json!({"known": n % 10, "link": format!("https://example.com/{n}")})
}

fn sites(count: usize) -> Vec<Value> {
    (0..count).map(site).collect()
}

/// `ServerLinksDialogScreen`: the buttons from the links, labelled by a known type's translation or
/// the server's own words, in the dialog's button width; a click asks about the address.
fn link_rows() -> Vec<Value> {
    let named = vec![
        json!({"text": "Our Wiki", "link": "https://wiki.example.com"}),
        json!({"known": 2, "link": "http://support.example.com/help?a=b"}),
        json!({"text": "Forum & Chat", "link": "https://forum.example.com"}),
    ];
    rows![
        linked("links-none", vec![], 2),
        linked("links-one", sites(1), 2),
        linked("links-two", sites(2), 2),
        linked("links-all-known", sites(10), 2),
        linked("links-named", named.clone(), 2),
        linked("links-one-column", sites(4), 1),
        linked("links-three-columns", sites(7), 3),
        linked("links-many", sites(25), 2).leave(),
        linked("links-exit", sites(3), 2).leave(),
        Dialog { button_width: Some(80), ..linked("links-narrow", sites(4), 2) },
        Dialog { button_width: Some(300), ..linked("links-wide", sites(4), 2) },
        linked("links-hover", sites(4), 2).at(150, 90),
        linked("links-tab", sites(4), 2).steps(times(key("tab"), 3)),
        linked("links-click", named.clone(), 2).steps(vec![click(150, 90)]).at(150, 90),
        linked("links-enter", sites(4), 2).steps([times(key("tab"), 2), vec![key("enter")]].concat()),
        linked("links-click-exit", sites(3), 2).leave().steps(vec![click(213, 224)]).at(213, 224),
        linked("links-escape", sites(3), 2).steps(vec![key("escape")]),
        linked("links-escape-exit", sites(3), 2).leave().steps(vec![key("escape")]),
    ]
}

/// `SimpleDialogScreen` for a notice and a confirmation: the title with its
/// warning button, the body in a scrolling box (wheel, scrollbar, Tab into
/// view), the buttons and what they tell the connection, Escape, and the
/// screen about custom screens; and `WaitingForResponseScreen`.
/// The blur is asked for from a blurriness of one, by a dialog and by the waiting screen over a
/// game.
fn blur_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for blurriness in [0, 10] {
        for mut base in [notice_rows().remove(0), waiting("waiting", vec![], (0, 0), true, (427, 240))] {
            base["in_world"] = json!(true);
            base["label"] = json!(format!("{}-blur-{blurriness}", base["label"].as_str().unwrap_or_default()));
            base["blurriness"] = json!(blurriness);
            rows.push(base);
        }
    }
    rows
}

pub fn dialog_screen() -> Value {
    let mut rows = [notice_rows(), scroll_rows(), press_rows(), waiting_rows(), list_rows(), link_rows()].concat();
    rows.extend(blur_rows());
    gui("dialog-screen", "dialog_screen", rows)
}
