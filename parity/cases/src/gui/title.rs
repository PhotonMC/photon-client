//! The title screen, the server list and its forms, the join screens, and
//! the server list's pings and icons.

use serde_json::{Value, json};

use super::gui;
use super::icons::favicon;
use crate::ordered::{Ordered, object};

fn frame(ms: i64) -> Value {
    json!({"do": "frame", "ms": ms})
}

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

fn keys(name: &str, count: usize) -> Vec<Value> {
    vec![key(name); count]
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn type_text(text: &str) -> Value {
    json!({"do": "type", "text": text})
}

/// A screen's size, the mouse and the steps taken.
#[derive(Clone)]
struct View {
    size: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
    ms: i64,
}

fn view() -> View {
    View { size: (427, 240), mouse: (0, 0), steps: vec![], ms: 0 }
}

/// One title screen scenario.
struct Title {
    label: String,
    view: View,
    splash: &'static str,
    hide: bool,
}

fn title(label: impl Into<String>) -> Title {
    Title { label: label.into(), view: view(), splash: "Hello world!", hide: false }
}

impl Title {
    fn row(self) -> Value {
        let View { size, mouse, steps, ms } = self.view;
        json!({
            "label": self.label, "width": size.0, "height": size.1, "mouse": [mouse.0, mouse.1], "steps": steps,
            "ms": ms, "splash": self.splash, "hide_splash": self.hide,
        })
    }
}

/// The title screen in screens of several sizes, with the splash at
/// several moments of its beat, the mouse over each button, the icons'
/// tooltips and the keyboard focus.
pub fn title_screen() -> Value {
    let buttons = [
        ("singleplayer", (213, 115)),
        ("multiplayer", (213, 140)),
        ("realms", (213, 165)),
        ("friends", (189, 190)),
        ("language", (213, 190)),
        ("accessibility", (237, 190)),
        ("options", (160, 210)),
        ("quit", (265, 210)),
        ("copyright", (400, 233)),
    ];
    let sized =
        |label: &str, width: i64, height: i64| Title { view: View { size: (width, height), ..view() }, ..title(label) };
    let at = |label: String, mouse: (i64, i64), steps: Vec<Value>, ms: i64| Title {
        view: View { mouse, steps, ms, ..view() },
        ..title(label)
    };
    let stepped = |label: String, steps: Vec<Value>, ms: i64| at(label, (0, 0), steps, ms);
    let mut rows = vec![
        title("default"),
        sized("small", 320, 240),
        sized("wide", 640, 360),
        sized("tall", 427, 480),
        Title { hide: true, ..title("splash-hidden") },
        Title { splash: "Absolutely no memes!", ..title("long-splash") },
        stepped("beat-quarter".into(), vec![], 250),
        stepped("beat-half".into(), vec![], 500),
        stepped("beat-later".into(), vec![], 12_345),
    ];
    rows.extend(buttons.iter().map(|(name, point)| at(format!("hover-{name}"), *point, vec![], 0)));
    for name in ["friends", "language", "accessibility"] {
        let point = buttons.iter().find(|(button, _)| *button == name).map_or((0, 0), |(_, point)| *point);
        rows.push(at(format!("tip-{name}"), point, vec![frame(0)], 10));
    }
    rows.extend([1, 4, 9].map(|count| stepped(format!("tab-{count}"), keys("tab", count), 0)));
    rows.extend([
        stepped("tab-icon-tip".into(), [keys("tab", 5), vec![frame(0)]].concat(), 10),
        stepped("shift-tab".into(), keys("shift+tab", 2), 0),
        stepped("arrow-down".into(), keys("down", 3), 0),
        stepped("arrow-icons".into(), [keys("down", 4), vec![key("right")]].concat(), 0),
        stepped("arrow-to-quit".into(), [keys("down", 5), vec![key("right")]].concat(), 0),
    ]);
    gui("title-screen", "title_screen", rows.into_iter().map(Title::row).collect())
}

/// One server list scenario.
struct List {
    label: String,
    servers: Vec<String>,
    view: View,
}

fn list(label: impl Into<String>) -> List {
    let servers = ["Alpha", "Beta Server", "Gamma"].map(str::to_owned).to_vec();
    List { label: label.into(), servers, view: view() }
}

impl List {
    fn row(self) -> Value {
        let View { size, mouse, steps, ms } = self.view;
        json!({
            "label": self.label, "servers": self.servers, "width": size.0, "height": size.1,
            "mouse": [mouse.0, mouse.1], "steps": steps, "ms": ms,
        })
    }
}

/// The server list: saved servers being asked about (the bars turn with the
/// clock), the local network's header, the mouse over a server and its join
/// and move arrows, the status tooltip, clicks that select or move a
/// server, scrolling a long list, and several screen sizes.
pub fn join_multiplayer_screen() -> Value {
    let scroll = |dy: i64| json!({"do": "scroll", "x": 213, "y": 100, "dy": dy});
    let many: Vec<String> = (1..=6).map(|index| format!("Server {index}")).collect();
    let sized =
        |label: &str, width: i64, height: i64| List { view: View { size: (width, height), ..view() }, ..list(label) };
    let at = |label: &str, x: i64, y: i64| List { view: View { mouse: (x, y), ..view() }, ..list(label) };
    let stepped = |label: &str, steps: Vec<Value>| List { view: View { steps, ..view() }, ..list(label) };
    let crowd = |label: &str, steps: Vec<Value>| List { servers: many.clone(), ..stepped(label, steps) };
    let mut rows = vec![
        list("three"),
        List { servers: vec![], ..list("none") },
        List { servers: vec!["Only".to_owned()], ..list("one") },
        sized("small", 320, 240),
        sized("wide", 640, 360),
        sized("tall", 427, 480),
        List { servers: vec!["A server with a rather long name for its row".to_owned()], ..list("long-name") },
    ];
    rows.extend(
        [100, 250, 450, 700, 1234].map(|ms| List { view: View { ms, ..view() }, ..list(format!("bars-{ms}")) }),
    );
    rows.extend([
        at("hover-text", 200, 80),
        at("hover-join", 90, 80),
        at("hover-move-up", 70, 75),
        at("hover-move-down", 70, 95),
        at("hover-first-no-up", 70, 40),
        at("hover-last-no-down", 70, 125),
        at("hover-lan-header", 213, 150),
        at("tip-status", 352, 40),
        at("hover-back", 323, 222),
        stepped("select-second", vec![click(200, 80)]),
        List {
            view: View { mouse: (161, 222), steps: vec![click(200, 80)], ..view() },
            ..list("select-then-hover-delete")
        },
        stepped("select-lan-header", vec![click(213, 150)]),
        stepped("move-first-down", vec![click(70, 57)]),
        stepped("move-last-up", vec![click(70, 110)]),
        crowd("many", vec![]),
        crowd("many-scrolled", vec![scroll(-1)]),
        crowd("many-scrolled-end", vec![scroll(-10)]),
        crowd("many-scrolled-back", vec![scroll(-3), scroll(1)]),
    ]);
    gui("join-multiplayer-screen", "join_multiplayer_screen", rows.into_iter().map(List::row).collect())
}

/// One server form scenario.
struct Form {
    label: &'static str,
    kind: &'static str,
    name: &'static str,
    ip: &'static str,
    size: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
}

fn form(label: &'static str) -> Form {
    Form { label, kind: "add", name: "", ip: "", size: (427, 240), mouse: (0, 0), steps: vec![] }
}

impl Form {
    fn row(self, pack: &str) -> Value {
        json!({
            "label": self.label, "kind": self.kind, "name": self.name, "ip": self.ip, "pack": pack,
            "width": self.size.0, "height": self.size.1, "mouse": [self.mouse.0, self.mouse.1], "steps": self.steps,
        })
    }
}

/// The server list's forms: adding (an empty name shows its hint once the
/// box loses the focus), editing, joining directly; text typed and erased,
/// Tab through the widgets, the resource pack choice cycled, Done asleep
/// for an address the resolver refuses.
pub fn server_form() -> Value {
    let with = |label: &'static str, steps: Vec<Value>| Form { steps, ..form(label) };
    let rows = vec![
        (form("add-empty"), "PROMPT"),
        (Form { size: (320, 240), ..form("add-small") }, "PROMPT"),
        (Form { size: (640, 360), ..form("add-wide") }, "PROMPT"),
        (with("add-typed-name", vec![type_text("My Server")]), "PROMPT"),
        (with("add-tab-to-address", vec![key("tab")]), "PROMPT"),
        (with("add-typed-both", vec![type_text("Home"), key("tab"), type_text("localhost:25600")]), "PROMPT"),
        (with("add-bad-port", vec![key("tab"), type_text("host:99999")]), "PROMPT"),
        (with("add-erased", vec![type_text("abc"), key("backspace"), key("backspace")]), "PROMPT"),
        (with("add-tab-to-pack", keys("tab", 2)), "PROMPT"),
        (with("add-pack-cycled", vec![key("tab"), key("tab"), key("space")]), "PROMPT"),
        (with("add-pack-clicked", vec![click(213, 135)]), "PROMPT"),
        (with("add-click-address", vec![click(250, 115)]), "PROMPT"),
        (Form { mouse: (213, 205), ..form("add-hover-cancel") }, "PROMPT"),
        (Form { kind: "edit", name: "Old Name", ip: "old.example:25565", ..form("edit") }, "ENABLED"),
        (
            Form {
                kind: "edit",
                name: "N",
                ip: "a.very.long.server.address.that.does.not.fit.in.the.box.example.com",
                ..form("edit-long-address")
            },
            "PROMPT",
        ),
        (Form { kind: "edit", name: "N", ip: "x.example", ..with("edit-tab-to-done", keys("tab", 3)) }, "PROMPT"),
        (Form { kind: "direct", ..form("direct-empty") }, "PROMPT"),
        (Form { kind: "direct", ip: "127.0.0.1:25600", ..form("direct-last") }, "PROMPT"),
        (Form { kind: "direct", ..with("direct-typed", vec![type_text("example.org")]) }, "PROMPT"),
        (Form { kind: "direct", ip: "example.org", ..with("direct-tab", vec![key("tab")]) }, "PROMPT"),
        (Form { kind: "direct", size: (320, 240), ip: "example.org", ..form("direct-small") }, "PROMPT"),
    ];
    gui("server-form", "server_form", rows.into_iter().map(|(form, pack)| form.row(pack)).collect())
}

/// The screens of a join: connecting with each status the handshake shows,
/// the mouse on Cancel, and loading the terrain, in several screen sizes.
pub fn connect_screen() -> Value {
    let row = |label: &str, screen: &str, status: &str, size: (i64, i64), mouse: (i64, i64)| {
        json!({
            "label": label, "screen": screen, "status": status, "width": size.0, "height": size.1,
            "mouse": [mouse.0, mouse.1],
        })
    };
    let connect =
        |label: &str, status: &str, size: (i64, i64), mouse: (i64, i64)| row(label, "connect", status, size, mouse);
    let loading = |label: &str, size: (i64, i64)| row(label, "loading", "connect.connecting", size, (0, 0));
    let reconfiguring = |label: &str, size: (i64, i64), mouse: (i64, i64), ticks: i64| {
        let mut value = row(label, "reconfiguring", "connect.reconfiguring", size, mouse);
        value["ticks"] = json!(ticks);
        value
    };
    let rows = vec![
        reconfiguring("reconfiguring", (427, 240), (0, 0), 0),
        reconfiguring("reconfiguring-small", (320, 240), (0, 0), 0),
        reconfiguring("reconfiguring-wide", (640, 360), (0, 0), 0),
        reconfiguring("reconfiguring-active", (427, 240), (0, 0), 600),
        reconfiguring("reconfiguring-hover", (427, 240), (213, 130), 600),
        reconfiguring("reconfiguring-hover-early", (427, 240), (213, 130), 599),
        connect("connecting", "connect.connecting", (427, 240), (0, 0)),
        connect("transferring", "connect.transferring", (427, 240), (0, 0)),
        connect("joining", "connect.joining", (427, 240), (0, 0)),
        connect("negotiating", "connect.negotiating", (427, 240), (0, 0)),
        connect("connecting-small", "connect.connecting", (320, 240), (0, 0)),
        connect("connecting-wide", "connect.connecting", (640, 360), (0, 0)),
        connect("hover-cancel", "connect.connecting", (427, 240), (213, 200)),
        loading("loading", (427, 240)),
        loading("loading-small", (320, 240)),
        loading("loading-tall", (427, 480)),
    ];
    gui("connect-screen", "connect_screen", rows)
}

/// A server's answer to a ping: its description, version and players.
struct Answer {
    description: Ordered,
    online: i64,
    max: i64,
    sample: Vec<&'static str>,
    version: &'static str,
    protocol: i64,
    players: bool,
}

fn answer(description: impl Into<Ordered>) -> Answer {
    Answer {
        description: description.into(),
        online: 2,
        max: 20,
        sample: vec![],
        version: "26.2",
        protocol: 776,
        players: true,
    }
}

impl Answer {
    fn status(self) -> Ordered {
        let mut status = object(vec![
            ("description", self.description),
            ("version", object(vec![("name", self.version.into()), ("protocol", self.protocol.into())])),
        ]);
        if self.players {
            let sample = self
                .sample
                .iter()
                .zip(1..)
                .map(|(name, index): (&&str, i64)| {
                    object(vec![
                        ("name", (*name).into()),
                        ("id", format!("00000000-0000-0000-0000-00000000000{index}").into()),
                    ])
                })
                .collect::<Vec<_>>();
            status = status.with(
                "players",
                object(vec![("max", self.max.into()), ("online", self.online.into()), ("sample", sample.into())]),
            );
        }
        status
    }
}

fn ping_of(status: &Ordered, ms: i64) -> Value {
    json!({"json": status.dumps(), "ping": ms})
}

fn ping(answer: Answer, ms: i64) -> Value {
    ping_of(&answer.status(), ms)
}

fn unreachable(unknown_host: bool) -> Value {
    json!({"unreachable": true, "unknown_host": unknown_host})
}

/// A server list row of `pings` (the first servers' names), the mouse at
/// `mouse`.
pub(super) fn pings_row(label: &str, pings: Vec<Value>, mouse: (i64, i64)) -> Value {
    let servers: Vec<&str> = ["Alpha", "Beta Server", "Gamma"].into_iter().take(pings.len()).collect();
    json!({
        "label": label, "servers": servers, "pings": pings, "width": 427, "height": 240,
        "mouse": [mouse.0, mouse.1], "steps": [], "ms": 0,
    })
}

/// The server list after its pings: the answer's description in up to two
/// lines, the player count or the red version of an incompatible server,
/// the bars of the ping's time, a server the answer leaves fields out of,
/// servers that could not be reached, and the tooltips of the bars and of
/// the player count.
pub fn server_list_pings() -> Value {
    let row = |label: &str, pings: Vec<Value>| pings_row(label, pings, (0, 0));
    let at = |label: &str, pings: Vec<Value>, x: i64, y: i64| pings_row(label, pings, (x, y));
    let ok = || ping(Answer { sample: vec!["Alex", "Steve"], ..answer("A Minecraft Server") }, 40);
    let old = || ping(Answer { version: "1.8.9", protocol: 47, ..answer("old") }, 40);
    let styled = object(vec![
        ("text", "Red ".into()),
        ("color", "red".into()),
        ("extra", vec![object(vec![("text", "bold".into()), ("bold", true.into())])].into()),
    ]);
    let long = "A rather long description that goes on and on past the width of one line, \
                and then a third line that is not shown at all, no matter how long";
    let bare = object(vec![
        ("description", "x".into()),
        ("players", object(vec![("max", 1i64.into()), ("online", 0i64.into())])),
    ]);
    let rows = vec![
        row("answered", vec![ok()]),
        row("ping-times", vec![ping(answer("a"), 149), ping(answer("b"), 299), ping(answer("c"), 1000)]),
        row("ping-mid", vec![ping(answer("a"), 450), ping(answer("b"), 999)]),
        row("styled-motd", vec![ping(answer(styled), 40)]),
        row("two-line-motd", vec![ping(answer(long), 40)]),
        row("incompatible", vec![old()]),
        row("no-players", vec![ping(Answer { players: false, ..answer("quiet") }, 40)]),
        row("no-version", vec![ping_of(&bare, 40)]),
        row("unreachable", vec![unreachable(false), unreachable(true)]),
        row("mixed", vec![ok(), Value::Null, unreachable(false)]),
        at("tip-ping", vec![ok()], 352, 40),
        at("tip-players", vec![ping(Answer { online: 5, sample: vec!["Alex", "Steve"], ..answer("x") }, 40)], 320, 40),
        at("tip-incompatible", vec![old()], 352, 40),
        at("tip-unreachable", vec![unreachable(false)], 352, 40),
    ];
    gui("server-list-pings", "join_multiplayer_screen", rows)
}

/// The server list's icons: a 64x64 favicon the entry uploads and blits,
/// one the game rejects for its size (a valid PNG, so it is kept until the
/// entry fails to upload it), one past the 1024 limit, bytes that are no
/// PNG, a malformed data URL, a server without one, and icons on only some
/// of the servers.
pub fn server_list_icons() -> Value {
    let ping = |icon: Option<String>| {
        let mut status = object(vec![
            ("description", "icon".into()),
            ("version", object(vec![("name", "26.2".into()), ("protocol", 776i64.into())])),
            ("players", object(vec![("max", 20i64.into()), ("online", 1i64.into())])),
        ]);
        if let Some(icon) = icon {
            status = status.with("favicon", icon);
        }
        ping_of(&status, 40)
    };
    let icon = |width: u32, height: u32, seed: u32| Some(favicon(&super::icons::png(width, height, seed)));
    let row = |label: &str, pings: Vec<Value>| pings_row(label, pings, (0, 0));
    let rows = vec![
        row("icon-64", vec![ping(icon(64, 64, 0))]),
        row("icon-64-second", vec![ping(None), ping(icon(64, 64, 7))]),
        row("icon-32-rejected", vec![ping(icon(32, 32, 0))]),
        row("icon-too-large", vec![ping(icon(1025, 1, 0))]),
        row("icon-1024-rejected", vec![ping(icon(1024, 1, 0))]),
        row("icon-not-png", vec![ping(Some(favicon(b"not a png at all, just text")))]),
        row("icon-bad-prefix", vec![ping(Some("data:image/jpeg;base64,AAAA".to_owned()))]),
        row("icon-bad-base64", vec![ping(Some("data:image/png;base64,!!!".to_owned()))]),
        row("icon-none", vec![ping(None)]),
        row("icon-three", vec![ping(icon(64, 64, 1)), ping(icon(32, 32, 0)), ping(None)]),
    ];
    gui("server-list-icons", "join_multiplayer_screen", rows)
}
