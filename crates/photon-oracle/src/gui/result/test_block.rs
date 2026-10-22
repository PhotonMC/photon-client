use super::*;
use photon_game::test_block::{TestBlockContents, TestBlockMode};
use photon_game::text::Lang;
use photon_gui::screen::test_block::{TestBlockAction, TestBlockScreen};

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

/// What the screen did so far: whether it closed and what it sent.
struct Outcome {
    closed: bool,
    packets: Vec<Value>,
}

impl Outcome {
    fn settle(&mut self, action: TestBlockAction) {
        match action {
            TestBlockAction::Stay => {}
            TestBlockAction::Cancel => self.closed = true,
            TestBlockAction::Done(edit) => {
                self.closed = true;
                self.packets
                    .push(json!({"mode": edit.mode.name(), "message": edit.message}));
            }
        }
    }
}

/// One step of a row: text, a key, a click, the wheel or a new size.
fn step(
    screen: &mut TestBlockScreen,
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
        Some("wheel") => screen.scrolled(lang, step["amount"].as_f64().unwrap_or(0.0), at),
        Some("resize") => {
            let size = |key: &str| step[key].as_i64().unwrap_or(0) as i32;
            screen.resize(lang, (size("width"), size("height")));
        }
        other => return Err(format!("unknown step {other:?}")),
    }
    Ok(())
}

/// `test_block_screen`: the test block's screen after text, keys, clicks, the wheel and new sizes,
/// drawn once, and whether it closed and the packets it would send.
pub(super) fn test_block_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let lang = Lang::load(&ctx.root);
    let font = std::sync::Arc::new(Font::load(&ctx.root).ok_or("font")?);
    let int = |value: &Value| value.as_i64().unwrap_or(0) as i32;
    let size = (int(&row["width"]), int(&row["height"]));
    let contents = TestBlockContents {
        mode: TestBlockMode::named(row["mode"].as_str().unwrap_or_default())
            .ok_or("unknown mode")?,
        message: row["message"].as_str().unwrap_or_default().to_owned(),
    };
    let mut outcome = Outcome {
        closed: false,
        packets: Vec::new(),
    };
    let mut screen = TestBlockScreen::new((&font, &lang), &contents, size);
    photon_gui::ui_sound::take();
    for each in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        step(&mut screen, &lang, each, &mut outcome)?;
    }
    let sounds = super::screens::asked_sounds();
    let mouse = (int(&row["mouse"][0]), int(&row["mouse"][1]));
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        millis: row["ms"].as_u64().unwrap_or(0),
        ..GuiFrame::default()
    };
    screen.draw((&mut frame, &ctx.atlas, &ctx.font), mouse, &lang);
    let log = frame.log.take().unwrap_or_default();
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&log),
        "closed": outcome.closed,
        "sounds": sounds,
        "packets": outcome.packets,
    }))
}
