//! The sign text scenarios and the sign editing screen.

use serde_json::{Value, json};

use super::gui;

const DYES: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

fn type_text(text: &str) -> Value {
    json!({"do": "type", "text": text})
}

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

fn face(lines: Value, color: &str, glowing: bool, filtered: Option<Value>) -> Value {
    json!({"messages": lines, "filtered": filtered, "color": color, "glowing": glowing})
}

/// One sign text scenario.
#[derive(Clone)]
struct Sign {
    label: String,
    kind: &'static str,
    wall: Option<&'static str>,
    rotation: i64,
    front: Value,
    back: Value,
    filtering: bool,
    draw_outline: bool,
    light: i64,
}

fn plain() -> Value {
    json!(["Hello", "", "sign", "x"])
}

fn row(label: impl Into<String>) -> Sign {
    Sign {
        label: label.into(),
        kind: "standing",
        wall: None,
        rotation: 0,
        front: face(plain(), "black", false, None),
        back: face(json!(["", "", "", ""]), "black", false, None),
        filtering: false,
        draw_outline: false,
        light: 0x00f0_00f0 - 17,
    }
}

impl Sign {
    fn row(self) -> Value {
        json!({
            "label": self.label, "kind": self.kind, "wall": self.wall, "rotation": self.rotation,
            "front": self.front, "back": self.back, "filtering": self.filtering,
            "draw_outline": self.draw_outline, "light": self.light,
        })
    }
}

fn placed_rows() -> Vec<Sign> {
    let mut rows = Vec::new();
    for kind in ["standing", "hanging"] {
        for rotation in 0..16 {
            rows.push(Sign {
                kind,
                rotation,
                back: face(json!(["back", "of", "the", "sign"]), "red", false, None),
                ..row(format!("{kind} floor {rotation}"))
            });
        }
        for wall in ["north", "south", "east", "west"] {
            rows.push(Sign { kind, wall: Some(wall), light: 240, ..row(format!("{kind} wall {wall}")) });
        }
    }
    rows
}

fn dye_rows() -> Vec<Sign> {
    let mut rows = Vec::new();
    for (index, dye) in DYES.iter().enumerate() {
        let kind = if index % 2 == 0 { "standing" } else { "hanging" };
        rows.push(Sign { kind, front: face(plain(), dye, false, None), ..row(format!("dye {dye}")) });
        rows.push(Sign { kind, front: face(plain(), dye, true, None), ..row(format!("dye {dye} glowing far")) });
        rows.push(Sign {
            kind,
            front: face(plain(), dye, true, None),
            draw_outline: true,
            ..row(format!("dye {dye} glowing near"))
        });
    }
    rows
}

fn text_rows() -> Vec<Sign> {
    let filtered = || Some(json!(["####", "", "clean", "x"]));
    let rude = || json!(["rude", "", "clean", "x"]);
    let wide = || {
        json!([
            "The quick brown fox jumps over the lazy dog",
            "WWWWWWWWWWWWWWWWWWWWWW",
            "iiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii",
            "nospacesatallinthislongline"
        ])
    };
    let styled = || {
        json!([
            {"text": "Red", "color": "red"},
            {"text": "Bold ", "bold": true, "extra": [{"text": "and", "color": "#123456"}]},
            ["", {"text": "list", "italic": true}, " tail"],
            {"text": "Glow", "color": "gold"}
        ])
    };
    vec![
        Sign { front: face(rude(), "black", false, filtered()), ..row("written lines") },
        Sign { front: face(rude(), "black", false, filtered()), filtering: true, ..row("filtered lines") },
        Sign { front: face(wide(), "black", false, None), back: face(wide(), "blue", true, None), ..row("too wide") },
        Sign { kind: "hanging", front: face(wide(), "black", false, None), ..row("too wide hanging") },
        Sign {
            front: face(styled(), "black", false, None),
            back: face(styled(), "white", true, None),
            draw_outline: true,
            ..row("styled")
        },
        Sign {
            kind: "hanging",
            wall: Some("east"),
            front: face(styled(), "cyan", true, None),
            ..row("styled hanging")
        },
    ]
}

/// What the reference's own sign renderers submit for a sign's text:
/// standing, wall and hanging signs at each turn and wall, each dye,
/// glowing text near and far, filtered lines, lines too wide for the sign
/// and styled ones.
pub fn sign_texts() -> Value {
    let rows = [placed_rows(), dye_rows(), text_rows()].concat();
    gui("sign-texts", "sign_texts", rows.into_iter().map(Sign::row).collect())
}

const WOODS: [&str; 12] = [
    "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "pale_oak", "mangrove", "cherry", "bamboo", "crimson",
    "warped",
];
const KINDS: [&str; 4] = ["standing", "wall", "hanging", "wall_hanging"];

/// One sign editing scenario.
#[derive(Clone)]
struct Edit {
    label: String,
    kind: &'static str,
    wood: &'static str,
    front: bool,
    lines: [&'static str; 4],
    color: &'static str,
    glowing: bool,
    size: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
    ms: i64,
}

const BLANK: [&str; 4] = ["", "", "", ""];
const WRITTEN: [&str; 4] = ["Hello", "", "sign", "x"];

fn edit(label: impl Into<String>) -> Edit {
    Edit {
        label: label.into(),
        kind: "standing",
        wood: "oak",
        front: true,
        lines: BLANK,
        color: "black",
        glowing: false,
        size: (427, 240),
        mouse: (0, 0),
        steps: vec![],
        ms: 0,
    }
}

impl Edit {
    fn row(self) -> Value {
        json!({
            "label": self.label, "kind": self.kind, "wood": self.wood, "front": self.front,
            "face": face(json!(self.lines), self.color, self.glowing, None),
            "width": self.size.0, "height": self.size.1, "mouse": [self.mouse.0, self.mouse.1],
            "steps": self.steps, "ms": self.ms,
        })
    }
}

fn edit_rows() -> Vec<Edit> {
    let mut rows = Vec::new();
    for (index, wood) in WOODS.iter().enumerate() {
        let kind = KINDS[index % 4];
        rows.push(Edit { kind, wood, lines: WRITTEN, ..edit(format!("{wood} {kind}")) });
    }
    for kind in KINDS {
        rows.push(Edit { kind, ..edit(format!("empty {kind}")) });
        rows.push(Edit { kind, front: false, lines: WRITTEN, ..edit(format!("back {kind}")) });
        rows.push(Edit {
            kind,
            steps: vec![type_text("Hi there"), key("down"), type_text("two")],
            ..edit(format!("typed {kind}"))
        });
        rows.push(Edit {
            kind,
            steps: vec![type_text("The quick brown fox jumps over it")],
            ..edit(format!("full {kind}"))
        });
        rows.push(Edit { kind, lines: WRITTEN, ms: 300, ..edit(format!("blink {kind}")) });
    }
    rows
}

fn typing_rows() -> Vec<Edit> {
    let words = ["one two three", "", "", ""];
    let written = |label: &str, steps: Vec<Value>| Edit { lines: WRITTEN, steps, ..edit(label) };
    let spoken = |label: &str, steps: Vec<Value>| Edit { lines: words, steps, ..edit(label) };
    vec![
        Edit { size: (320, 240), lines: WRITTEN, ..edit("small") },
        Edit { size: (640, 360), lines: WRITTEN, ..edit("wide") },
        Edit { kind: "hanging", size: (427, 480), lines: WRITTEN, ..edit("tall hanging") },
        written("up wraps", vec![key("up")]),
        written("enter twice", vec![key("enter"), key("enter")]),
        written("down to empty", vec![key("down")]),
        written("cursor inside", vec![key("left"), key("left")]),
        written("cursor home", vec![key("home")]),
        Edit { ms: 900, ..written("cursor inside blink", vec![key("left")]) },
        written("select left", vec![key("shift+left"), key("shift+left")]),
        written("select home", vec![key("left"), key("shift+home")]),
        spoken("select all", vec![key("ctrl+a")]),
        spoken("select all typed", vec![key("ctrl+a"), type_text("new")]),
        spoken("word left", vec![key("ctrl+left"), type_text("X")]),
        spoken("word right", vec![key("home"), key("ctrl+right"), type_text("X")]),
        spoken("word erased", vec![key("ctrl+backspace")]),
        written("backspace", vec![key("backspace"), key("backspace")]),
        written("delete inside", vec![key("home"), key("delete"), key("end")]),
        written("selection erased", vec![key("shift+left"), key("shift+left"), key("backspace")]),
        Edit { mouse: (213, 215), ..written("hover done", vec![]) },
        written("tab to done", vec![key("tab")]),
        Edit { color: "red", ..written("red", vec![]) },
        Edit { color: "red", glowing: true, ..written("red glowing", vec![]) },
        Edit { glowing: true, ..written("black glowing", vec![]) },
        Edit { kind: "hanging", color: "white", ..written("white hanging", vec![]) },
        Edit { steps: vec![type_text("a\u{a7}b")], ..edit("section sign refused") },
    ]
}

/// The sign editing screens: each wood, standing, on a wall and hanging,
/// front and back; the lines typed until the sign is full, moved between
/// with the arrows and Enter, the cursor in the text and at its end, its
/// blink, selections by Shift and Control, words erased, the Done button
/// under the mouse and focused by Tab, dyes and glowing text.
pub fn sign_edit_screen() -> Value {
    let rows = [edit_rows(), typing_rows()].concat();
    gui("sign-edit-screen", "sign_edit_screen", rows.into_iter().map(Edit::row).collect())
}
