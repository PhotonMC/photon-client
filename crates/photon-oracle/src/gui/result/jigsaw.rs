use super::*;
use photon_game::jigsaw::{JigsawContents, Joint};
use photon_game::text::Lang;
use photon_gui::screen::jigsaw::{JigsawAction, JigsawEdit, JigsawScreen};

/// The keys a row may name.
const KEYS: [(&str, glfw::Key); 13] = [
    ("tab", glfw::Key::Tab),
    ("backspace", glfw::Key::Backspace),
    ("delete", glfw::Key::Delete),
    ("enter", glfw::Key::Enter),
    ("escape", glfw::Key::Escape),
    ("space", glfw::Key::Space),
    ("left", glfw::Key::Left),
    ("right", glfw::Key::Right),
    ("up", glfw::Key::Up),
    ("down", glfw::Key::Down),
    ("home", glfw::Key::Home),
    ("end", glfw::Key::End),
    ("a", glfw::Key::A),
];

fn key_of(name: &str) -> Result<glfw::Key, String> {
    KEYS.iter()
        .find(|(key, _)| *key == name)
        .map(|(_, key)| *key)
        .ok_or_else(|| format!("unknown key {name}"))
}

fn joint_of(name: &str) -> Joint {
    if name == "aligned" {
        Joint::Aligned
    } else {
        Joint::Rollable
    }
}

fn contents_of(row: &Value) -> JigsawContents {
    let text = |key: &str| row[key].as_str().unwrap_or_default().to_owned();
    let number = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    JigsawContents {
        name: text("name"),
        target: text("target"),
        pool: text("pool"),
        final_state: text("final_state"),
        joint: joint_of(&text("joint")),
        selection_priority: number("selection_priority"),
        placement_priority: number("placement_priority"),
        // `FrontAndTop` names start with the front: up and down are the vertical ones.
        vertical: text("orientation").starts_with("up") || text("orientation").starts_with("down"),
    }
}

/// `ServerboundSetJigsawBlockPacket`, without the block's position.
fn set_packet(edit: &JigsawEdit) -> Value {
    json!({
        "type": "set", "name": edit.name, "target": edit.target, "pool": edit.pool,
        "final_state": edit.final_state, "joint": edit.joint.name(),
        "selection_priority": edit.selection_priority,
        "placement_priority": edit.placement_priority,
    })
}

/// What the screen did so far: whether it closed and what it sent.
struct Outcome {
    closed: bool,
    packets: Vec<Value>,
}

impl Outcome {
    fn settle(&mut self, action: JigsawAction) {
        match action {
            JigsawAction::Stay => {}
            JigsawAction::Cancel => self.closed = true,
            JigsawAction::Done(edit) => {
                self.closed = true;
                self.packets.push(set_packet(&edit));
            }
            JigsawAction::Generate(edit, levels, keep) => {
                self.closed = true;
                self.packets.push(set_packet(&edit));
                self.packets
                    .push(json!({"type": "generate", "levels": levels, "keep_jigsaws": keep}));
            }
        }
    }
}

/// One step of a row: text, a key, a click, a drag, a release, the wheel or a new size.
fn step(
    screen: &mut JigsawScreen,
    lang: &Lang,
    step: &Value,
    outcome: &mut Outcome,
) -> Result<(), String> {
    let at = (
        step["x"].as_f64().unwrap_or(0.0) as i32,
        step["y"].as_f64().unwrap_or(0.0) as i32,
    );
    match step["do"].as_str() {
        Some("type") => {
            for character in step["text"].as_str().unwrap_or_default().chars() {
                screen.char_typed(character);
            }
        }
        Some("key") => {
            let name = step["key"].as_str().unwrap_or_default();
            let control = name.starts_with("ctrl+");
            let name = name.trim_start_matches("ctrl+");
            let shift = name.starts_with("shift+");
            let key = key_of(name.trim_start_matches("shift+"))?;
            let action =
                screen.key_pressed(lang, key, (control, shift), &mut crate::gui::NoClipboard);
            outcome.settle(action);
        }
        Some("click") => {
            let shift = step["shift"].as_bool() == Some(true);
            outcome.settle(screen.clicked(lang, at, shift));
        }
        Some("drag") => screen.dragged(lang, at),
        Some("release") => screen.released(),
        Some("wheel") => screen.scrolled(lang, step["amount"].as_f64().unwrap_or(0.0), at),
        Some("resize") => {
            let size = |key: &str| step[key].as_i64().unwrap_or(0) as i32;
            screen.resize(lang, (size("width"), size("height")));
        }
        other => return Err(format!("unknown step {other:?}")),
    }
    Ok(())
}

/// `jigsaw_screen`: the jigsaw block's screen after text, keys, clicks, drags, the wheel and new
/// sizes, drawn twice (a tooltip waits for a second frame) with the second drawing recorded, and
/// whether it closed and the packets it would send, in order.
pub(super) fn jigsaw_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let lang = Lang::load(&ctx.root);
    let font = std::sync::Arc::new(crate::fonts::load_font(&ctx.root).ok_or("font")?);
    let int = |value: &Value| value.as_i64().unwrap_or(0) as i32;
    let size = (int(&row["width"]), int(&row["height"]));
    let mut outcome = Outcome {
        closed: false,
        packets: Vec::new(),
    };
    let mut screen = JigsawScreen::new((&font, &lang), &contents_of(row), size);
    for each in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        step(&mut screen, &lang, each, &mut outcome)?;
    }
    let mouse = (int(&row["mouse"][0]), int(&row["mouse"][1]));
    let ms = row["ms"].as_u64().unwrap_or(0);
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        millis: ms,
        ..GuiFrame::default()
    };
    screen.draw((&mut frame, &ctx.atlas, &ctx.font), mouse, &lang);
    let first = frame.log.as_ref().map_or(0, Vec::len);
    frame.millis = ms + 50;
    screen.draw((&mut frame, &ctx.atlas, &ctx.font), mouse, &lang);
    let log = frame.log.take().unwrap_or_default();
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&log[first..]),
        "closed": outcome.closed,
        "packets": outcome.packets,
    }))
}
