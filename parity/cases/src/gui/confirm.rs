//! The confirmation screen, the link screen and the server pack question.

use serde_json::{Value, json};

use super::gui;

/// One confirmation screen scenario.
#[derive(Clone)]
struct Confirm {
    size: (i64, i64),
    mouse: (i64, i64),
    death: bool,
    title: &'static str,
    message: String,
    yes: &'static str,
    no: &'static str,
    delay: i64,
    ticks: i64,
    keys: Vec<&'static str>,
    click: bool,
    in_world: bool,
    link: Option<Value>,
    pack: Option<Value>,
    command: Option<Value>,
}

fn confirm() -> Confirm {
    Confirm {
        size: (427, 240),
        mouse: (0, 0),
        death: false,
        title: "Are you sure?",
        message: "This cannot be undone.".to_owned(),
        yes: "Yes",
        no: "No",
        delay: 0,
        ticks: 0,
        keys: vec![],
        click: false,
        in_world: false,
        link: None,
        pack: None,
        command: None,
    }
}

impl Confirm {
    fn row(self, label: &str) -> Value {
        let mut row = json!({
            "label": label, "width": self.size.0, "height": self.size.1, "mouse": [self.mouse.0, self.mouse.1],
            "death": self.death, "title": self.title, "message": self.message, "yes": self.yes, "no": self.no,
            "delay": self.delay, "ticks": self.ticks, "keys": self.keys, "click": self.click, "in_world": self.in_world,
        });
        if let Some(link) = self.link {
            row["link"] = link;
        }
        if let Some(pack) = self.pack {
            row["pack"] = pack;
        }
        if let Some(command) = self.command {
            row["command"] = command;
        }
        row
    }
}

fn quit() -> Confirm {
    Confirm {
        death: true,
        title: "Are you sure you want to quit?",
        message: String::new(),
        yes: "Title Screen",
        no: "Respawn",
        delay: 20,
        ..confirm()
    }
}

fn lines(count: usize) -> String {
    (0..count).map(|n| format!("line {n}")).collect::<Vec<_>>().join("\n")
}

fn message_rows() -> Vec<Value> {
    let long = "This message is long enough that it has to wrap onto several lines of the screen before the \
                buttons, and it keeps going so that the third line is needed too.";
    let message = |text: String| Confirm { message: text, ..confirm() };
    vec![
        confirm().row("plain"),
        Confirm { in_world: true, ..confirm() }.row("plain-in-world"),
        message(String::new()).row("empty-message"),
        message(long.to_owned()).row("wrapped"),
        Confirm { size: (320, 240), ..message(long.to_owned()) }.row("wrapped-narrow"),
        Confirm { size: (854, 480), ..message(long.to_owned()) }.row("wrapped-wide"),
        message("one\ntwo\n\nfour".to_owned()).row("newlines"),
        message(lines(15)).row("fifteen-lines"),
        message(lines(16)).row("sixteen-lines"),
        message(lines(40)).row("many-lines"),
        Confirm { yes: "Open", no: "Cancel", ..confirm() }.row("custom-buttons"),
        Confirm { mouse: (100, 145), ..confirm() }.row("hover-yes"),
        Confirm { mouse: (300, 145), ..confirm() }.row("hover-no"),
        Confirm { mouse: (213, 145), ..confirm() }.row("hover-gap"),
        Confirm { mouse: (10, 10), ..confirm() }.row("hover-elsewhere"),
        quit().row("quit"),
        Confirm { ticks: 20, ..quit() }.row("quit-awake"),
        Confirm { mouse: (100, 140), ..quit() }.row("quit-hover-asleep"),
        Confirm { mouse: (100, 140), ticks: 20, ..quit() }.row("quit-hover-awake"),
        Confirm { size: (320, 240), ..quit() }.row("quit-small"),
    ]
}

fn input_rows() -> Vec<Value> {
    let delayed = |ticks: i64| Confirm { delay: 20, ticks, ..confirm() };
    let keys = |keys: Vec<&'static str>, base: Confirm| Confirm { keys, ..base };
    let click = |mouse: (i64, i64), base: Confirm| Confirm { mouse, click: true, ..base };
    vec![
        delayed(19).row("delay-one-short"),
        delayed(20).row("delay-done"),
        delayed(80).row("delay-long"),
        keys(vec!["tab"], confirm()).row("tab"),
        keys(vec!["tab", "tab"], confirm()).row("tab-twice"),
        keys(vec!["tab"; 3], confirm()).row("tab-thrice"),
        keys(vec!["shift+tab"], confirm()).row("shift-tab"),
        keys(vec!["shift+tab", "shift+tab"], confirm()).row("shift-tab-twice"),
        keys(vec!["tab"], delayed(5)).row("tab-asleep"),
        keys(vec!["tab"], delayed(20)).row("tab-awake"),
        keys(vec!["enter"], confirm()).row("enter-unfocused"),
        keys(vec!["tab", "enter"], confirm()).row("enter-yes"),
        keys(vec!["tab", "tab", "space"], confirm()).row("space-no"),
        keys(vec!["tab", "enter"], delayed(5)).row("enter-asleep"),
        keys(vec!["escape"], confirm()).row("escape"),
        keys(vec!["escape"], delayed(5)).row("escape-asleep"),
        keys(vec!["escape"], delayed(19)).row("escape-last-tick"),
        keys(vec!["escape"], delayed(20)).row("escape-awake"),
        keys(vec!["escape"], Confirm { ticks: 25, ..quit() }).row("escape-quit"),
        click((100, 145), confirm()).row("click-yes"),
        click((300, 145), confirm()).row("click-no"),
        click((213, 145), confirm()).row("click-gap"),
        click((100, 145), delayed(5)).row("click-asleep"),
        click((300, 145), delayed(20)).row("click-awake"),
    ]
}

/// `ConfirmLinkScreen`: three buttons of 100 and, for a link that is not
/// trusted, a warning. Copy to Clipboard is not pressed.
fn link_rows() -> Vec<Value> {
    let url = "https://example.com/a/path?query=1";
    let link = |url: &str, trusted: bool| Confirm { link: Some(json!({"url": url, "trusted": trusted})), ..confirm() };
    let untrusted = || link(url, false);
    let long = format!("https://example.com/{}end", "segment/".repeat(12));
    let keyed = |keys: Vec<&'static str>| Confirm { keys, ..untrusted() };
    let clicked = |x: i64| Confirm { mouse: (x, 150), click: true, ..untrusted() };
    vec![
        untrusted().row("link"),
        link(url, true).row("link-trusted"),
        Confirm { in_world: true, ..untrusted() }.row("link-in-world"),
        Confirm { size: (320, 240), ..untrusted() }.row("link-small"),
        Confirm { size: (854, 480), ..untrusted() }.row("link-wide"),
        Confirm { size: (200, 150), ..untrusted() }.row("link-tiny"),
        link(&long, false).row("link-long-url"),
        link(&long, true).row("link-long-url-trusted"),
        link("", false).row("link-empty-url"),
        Confirm { mouse: (110, 150), ..untrusted() }.row("link-hover-yes"),
        Confirm { mouse: (213, 150), ..untrusted() }.row("link-hover-copy"),
        Confirm { mouse: (320, 150), ..untrusted() }.row("link-hover-no"),
        Confirm { mouse: (110, 145), ..link(url, true) }.row("link-hover-trusted"),
        keyed(vec!["tab"]).row("link-tab"),
        keyed(vec!["tab"; 2]).row("link-tab-2"),
        keyed(vec!["tab"; 3]).row("link-tab-3"),
        keyed(vec!["tab"; 4]).row("link-tab-4-round"),
        keyed(vec!["shift+tab"]).row("link-shift-tab"),
        keyed(vec!["tab", "enter"]).row("link-enter-yes"),
        keyed(vec!["tab", "tab", "tab", "enter"]).row("link-enter-no"),
        keyed(vec!["escape"]).row("link-escape"),
        clicked(110).row("link-click-yes"),
        clicked(320).row("link-click-no"),
        clicked(161).row("link-click-gap"),
    ]
}

/// The server resource pack question: wording of an optional and a
/// required pack, with and without the server's own prompt.
fn pack_rows() -> Vec<Value> {
    let prompt = "Get the server textures to see custom items.";
    let pack = |required: bool, prompted: bool| {
        let pack =
            if prompted { json!({"required": required, "prompt": prompt}) } else { json!({"required": required}) };
        Confirm { pack: Some(pack), ..confirm() }
    };
    vec![
        pack(false, false).row("pack"),
        pack(false, true).row("pack-prompt"),
        pack(true, false).row("pack-required"),
        pack(true, true).row("pack-required-prompt"),
        Confirm { size: (320, 240), ..pack(true, true) }.row("pack-required-small"),
        Confirm { in_world: true, ..pack(false, true) }.row("pack-in-world"),
        Confirm { mouse: (300, 145), ..pack(true, false) }.row("pack-hover-no"),
        Confirm { mouse: (100, 145), click: true, ..pack(true, false) }.row("pack-click-yes"),
        Confirm { mouse: (300, 145), click: true, ..pack(false, false) }.row("pack-click-no"),
    ]
}

/// The question about a command the player did not type: each wording, with the command in
/// yellow, and Back or Cancel by whether a screen lies below.
fn command_rows() -> Vec<Value> {
    let asking = |key: &str, accept: &str, back: bool| {
        let command = json!({
            "key": key, "command": "give @s stone 64", "accept": accept, "back": back,
        });
        Confirm { in_world: true, command: Some(command), ..confirm() }
    };
    let run = "multiplayer.confirm_command.run_command";
    vec![
        asking("multiplayer.confirm_command.parse_errors", run, true).row("command-parse-errors"),
        asking("multiplayer.confirm_command.permissions_required", run, true).row("command-permissions"),
        asking("multiplayer.confirm_command.permissions_required", run, false).row("command-cancel"),
        asking("multiplayer.confirm_command.signature_required", "chat.copy", true).row("command-signed-copy"),
        asking("multiplayer.confirm_command.signature_required", "multiplayer.confirm_command.suggest_command", false)
            .row("command-signed-suggest"),
        Confirm { size: (320, 240), ..asking("multiplayer.confirm_command.permissions_required", run, true) }
            .row("command-small"),
    ]
}

/// Over a game the background blurs from a blurriness of one; the death screen's does not.
fn blur_rows() -> Vec<Value> {
    [("blur-in-world-0", 0, false), ("blur-in-world-10", 10, false), ("blur-death", 5, true)]
        .into_iter()
        .map(|(label, blurriness, death)| {
            let mut row = Confirm { in_world: true, death, ..confirm() }.row(label);
            row["blurriness"] = json!(blurriness);
            row
        })
        .collect()
}

/// The confirmation screen (and the death screen's own): the message
/// wrapped and clipped to fifteen lines, the delay that keeps the buttons
/// asleep and Escape shut, Tab, Enter and clicks, and what the callback was
/// told.
pub fn confirm_screen() -> Value {
    gui(
        "confirm-screen",
        "confirm_screen",
        [message_rows(), input_rows(), link_rows(), pack_rows(), command_rows(), blur_rows()].concat(),
    )
}
