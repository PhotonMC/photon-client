//! The chat box's suggestion list and its draft.

use serde_json::{Value, json};

use super::gui;

fn suggestion(
    label: &str,
    (text, start, length): (&str, i64, i64),
    matches: &[&str],
    keys: &[&str],
    size: (i64, i64),
) -> Value {
    json!({
        "label": label, "width": size.0, "height": size.1, "text": text, "start": start, "length": length,
        "matches": matches, "keys": keys,
    })
}

/// The chat box's list of matches: where it sits, which one is chosen, what
/// Tab, Shift+Tab, the arrow keys and Escape do to the list and the text,
/// and the grey text after the cursor.
pub fn suggestions() -> Value {
    let many: Vec<String> = (0..14).map(|n| format!("entry{n}")).collect();
    let many: Vec<&str> = many.iter().map(String::as_str).collect();
    let gam = ("/gam", 1, 3);
    let pair = ["gamemode", "gamerule"];
    let usual = (427, 240);
    let scrolling: Vec<&str> = [vec!["down"; 11], vec!["up"; 3]].concat();
    let rows = vec![
        suggestion("first-chosen", gam, &pair, &[], usual),
        suggestion("tab-twice", gam, &pair, &["tab", "tab", "tab"], usual),
        suggestion("shift-tab", gam, &pair, &["tab", "shift+tab", "tab"], usual),
        suggestion("arrows", gam, &["gamemode", "gamerule", "gametest"], &["down", "down", "up", "up", "up"], usual),
        suggestion("escape-hides", gam, &pair, &["escape"], usual),
        suggestion("prefix-first", ("/ga", 1, 2), &["zgame", "gamemode", "minecraft:gamerule"], &[], usual),
        suggestion("mid-text", ("/say hel", 5, 3), &["help", "hello", "hex"], &["tab"], usual),
        suggestion("scrolling-window", ("/e", 1, 1), &many, &scrolling, usual),
        suggestion("wraps-round", ("/e", 1, 1), &many, &["up", "up", "down"], usual),
        suggestion("narrow-screen", gam, &pair, &["tab"], (320, 240)),
    ];
    gui("suggestions", "suggestions", rows)
}

fn open(method: &str) -> Value {
    json!({"do": "open", "method": method})
}

fn message() -> Value {
    open("message")
}

/// `InBedChatScreen`: the box with the leave-bed button.
fn in_bed() -> Value {
    let mut event = open("message");
    event["bed"] = json!(true);
    event
}

fn typed(text: &str) -> Value {
    json!({"do": "type", "text": text})
}

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

fn close(reason: &str) -> Value {
    json!({"do": "close", "reason": reason})
}

fn draw() -> Value {
    json!({"do": "draw"})
}

fn draft(label: &str, events: Vec<Value>, save: bool) -> Value {
    json!({"label": label, "width": 427, "height": 240, "save": save, "events": events})
}

/// A draft kept by an interruption, then `more`.
fn kept(more: Vec<Value>) -> Vec<Value> {
    [vec![message(), typed("hello"), close("interrupted")], more].concat()
}

fn closing_rows() -> Vec<Value> {
    let command = || open("command");
    vec![
        draft("empty", vec![message()], false),
        draft("empty-command", vec![command()], false),
        draft("escape-discards", vec![message(), typed("hello"), close("intentional"), message()], false),
        draft("escape-saves", vec![message(), typed("hello"), close("intentional"), message()], true),
        draft("interrupted-saves", kept(vec![message()]), false),
        draft("interrupted-saves-with-option", kept(vec![message()]), true),
        draft("done-discards", vec![message(), typed("hello"), close("done"), message()], false),
        draft("done-discards-with-option", vec![message(), typed("hello"), close("done"), message()], true),
        draft("command-draft-to-command", vec![command(), typed("gamemode"), close("interrupted"), command()], false),
        draft("command-draft-to-message", vec![command(), typed("gamemode"), close("interrupted"), message()], false),
        draft("message-draft-to-command", kept(vec![command()]), false),
        draft("message-draft-to-command-then-message", kept(vec![command(), close("done"), message()]), false),
        draft("blank-is-nothing", vec![message(), typed("   "), close("interrupted"), message()], false),
        draft(
            "blank-clears-the-draft",
            kept(vec![message(), key("backspace"), close("interrupted"), message()]),
            false,
        ),
        draft("slash-only-is-kept", vec![command(), close("interrupted"), command()], false),
        draft("slash-only-blank-is-not", vec![command(), close("interrupted"), message()], false),
        draft("spaces-kept-as-typed", vec![message(), typed("  hi  "), close("interrupted"), message()], false),
    ]
}

fn editing_rows() -> Vec<Value> {
    vec![
        draft("unedited-draft-stays", kept(vec![message(), close("interrupted"), message()]), false),
        draft("unedited-draft-stays-escape", kept(vec![message(), close("intentional"), message()]), true),
        draft("unedited-draft-lost-escape", kept(vec![message(), close("intentional"), message()]), false),
        draft("unedited-draft-lost-done", kept(vec![message(), close("done"), message()]), false),
        draft("edited-draft-kept", kept(vec![message(), typed("!"), close("interrupted"), message()]), false),
        draft("edited-draft-lost-on-done", kept(vec![message(), typed("!"), close("done"), message()]), false),
        draft("backspace-wipes-draft", kept(vec![message(), key("backspace")]), false),
        draft("backspace-wipes-then-types", kept(vec![message(), key("backspace"), typed("x")]), false),
        draft("backspace-twice", kept(vec![message(), key("backspace"), key("backspace")]), false),
        draft("backspace-on-own-text", vec![message(), typed("abc"), key("backspace")], false),
        draft(
            "cursor-keys-keep-draft",
            kept(vec![message(), key("left"), key("left"), key("home"), key("end")]),
            false,
        ),
        draft("delete-edits-draft", kept(vec![message(), key("home"), key("delete")]), false),
        draft("typing-ends-draft", kept(vec![message(), typed("a")]), false),
        draft(
            "draft-then-wiped-then-closed",
            kept(vec![message(), key("backspace"), close("intentional"), message()]),
            true,
        ),
        draft(
            "second-draft-replaces",
            kept(vec![message(), key("backspace"), typed("other"), close("interrupted"), message()]),
            false,
        ),
    ]
}

fn drawing_rows() -> Vec<Value> {
    let mut small = draft("draw-draft-small", kept(vec![message(), draw()]), false);
    small["width"] = json!(200);
    small["height"] = json!(120);
    let long = "a long draft that goes on ".repeat(6);
    vec![
        draft("draw-empty", vec![message(), draw()], false),
        draft("draw-in-bed", vec![in_bed(), draw()], false),
        draft("draw-in-bed-typed", vec![in_bed(), typed("hello"), draw()], false),
        draft("draw-own-text", vec![message(), typed("hello"), draw()], false),
        draft("draw-draft", kept(vec![message(), draw()]), false),
        draft("draw-draft-edited", kept(vec![message(), typed("!"), draw()]), false),
        draft("draw-draft-wiped", kept(vec![message(), key("backspace"), draw()]), false),
        draft("draw-draft-cursor-moved", kept(vec![message(), key("left"), draw()]), false),
        small,
        draft("draw-long-draft", vec![message(), typed(&long), close("interrupted"), message(), draw()], false),
        draft(
            "many-closes",
            vec![
                message(),
                typed("a"),
                close("interrupted"),
                message(),
                typed("b"),
                close("interrupted"),
                message(),
                typed("c"),
                close("done"),
                message(),
            ],
            false,
        ),
    ]
}

/// The chat box's draft: what a closed box keeps by the way it closed, a
/// blank text kept as nothing, which box takes which draft, a draft left as
/// it was when it is not edited, one Backspace wiping it, and a draft drawn
/// grey and slanted until it is edited.
pub fn chat_draft() -> Value {
    gui("chat-draft", "chat_draft", [closing_rows(), editing_rows(), drawing_rows()].concat())
}
