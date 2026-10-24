use super::*;
use photon_game::structure_block::Rotation;
use photon_game::test_instance::TestInstanceContents;
use photon_game::text::{Component, Lang};
use photon_gui::screen::test_instance_block::{InstanceOutcome, TestInstanceBlockScreen};

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

/// One step of a row: text, a key, a click, the wheel or a new size. Whether the screen closed.
fn step(
    (screen, error): (&mut TestInstanceBlockScreen, &mut Option<String>),
    lang: &Lang,
    step: &Value,
) -> Result<bool, String> {
    if status_step((screen, error), lang, step) {
        return Ok(false);
    }
    let at = (
        step["x"].as_f64().unwrap_or(0.0) as i32,
        step["y"].as_f64().unwrap_or(0.0) as i32,
    );
    let outcome = match step["do"].as_str() {
        Some("type") => {
            for character in step["text"].as_str().unwrap_or_default().chars() {
                screen.char_typed(lang, character);
            }
            InstanceOutcome::Stay
        }
        Some("key") => {
            let name = step["key"].as_str().unwrap_or_default();
            let control = name.starts_with("ctrl+");
            let name = name.trim_start_matches("ctrl+");
            let shift = name.starts_with("shift+");
            let key = key_of(name.trim_start_matches("shift+"))?;
            screen.key_pressed(lang, key, (control, shift), &mut crate::gui::NoClipboard)
        }
        Some("click") => screen.clicked(lang, at, step["shift"].as_bool() == Some(true)),
        Some("wheel") => {
            screen.scrolled(lang, step["amount"].as_f64().unwrap_or(0.0), at);
            InstanceOutcome::Stay
        }
        Some("resize") => {
            let size = |key: &str| step[key].as_i64().unwrap_or(0) as i32;
            screen.resize(lang, (size("width"), size("height")));
            InstanceOutcome::Stay
        }
        other => return Err(format!("unknown step {other:?}")),
    };
    Ok(outcome == InstanceOutcome::Close)
}

/// The steps that carry the server's side: a status line, and the mouse dragging the bar or
/// letting it go. Whether the step was one of them.
fn status_step(
    (screen, error): (&mut TestInstanceBlockScreen, &mut Option<String>),
    lang: &Lang,
    step: &Value,
) -> bool {
    let number = |key: &str| step[key].as_f64().unwrap_or(0.0);
    match step["do"].as_str() {
        Some("status") => {
            if let Some(text) = step["error"].as_str() {
                *error = Some(text.to_owned());
            }
            let size = step["size"].as_array().map(|size| {
                (
                    size[0].as_i64().unwrap_or(0) as i32,
                    size[1].as_i64().unwrap_or(0) as i32,
                    size[2].as_i64().unwrap_or(0) as i32,
                )
            });
            let error = error.as_deref().map(Component::literal);
            let text = Component::literal(step["text"].as_str().unwrap_or_default());
            screen.set_status(lang, error.as_ref(), &text, size);
        }
        Some("drag") => screen.dragged(number("y"), number("dy")),
        Some("release") => screen.released(),
        _ => return false,
    }
    true
}

fn rotation_named(name: &str) -> Result<Rotation, String> {
    Rotation::ALL
        .into_iter()
        .find(|rotation| rotation.name() == name)
        .ok_or_else(|| format!("unknown rotation {name}"))
}

/// `test_instance_screen`: the test instance block's screen after text, keys, clicks, the wheel
/// and new sizes, drawn once, and whether it closed and the packets it sent.
pub(super) fn test_instance_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
) -> Result<Value, String> {
    let lang = Lang::load(&ctx.root);
    let font = std::sync::Arc::new(crate::fonts::load_font(&ctx.root).ok_or("font")?);
    let int = |value: &Value| value.as_i64().unwrap_or(0) as i32;
    let size = (int(&row["width"]), int(&row["height"]));
    let contents = TestInstanceContents {
        // The block holds a parsed key, which writes as `Identifier.toString`.
        test: row["test"]
            .as_str()
            .map(|test| {
                photon_core::identifier::Identifier::parse(test)
                    .map(|id| id.as_string())
                    .map_err(|_| format!("unparsable test {test}"))
            })
            .transpose()?,
        size: (
            int(&row["size"][0]),
            int(&row["size"][1]),
            int(&row["size"][2]),
        ),
        rotation: rotation_named(row["rotation"].as_str().unwrap_or_default())?,
        ignore_entities: row["ignore_entities"].as_bool().unwrap_or(false),
    };
    let mut screen = TestInstanceBlockScreen::new((&font, &lang), &contents, size);
    let mut closed = false;
    let mut error = None;
    for each in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        closed |= step((&mut screen, &mut error), &lang, each)?;
    }
    let mouse = (int(&row["mouse"][0]), int(&row["mouse"][1]));
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        millis: row["ms"].as_u64().unwrap_or(0),
        ..GuiFrame::default()
    };
    screen.draw((&mut frame, &ctx.atlas, &ctx.font), mouse, &lang);
    let log = frame.log.take().unwrap_or_default();
    let packets: Vec<Value> = screen.take_sent().into_iter().map(packet_json).collect();
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&log),
        "closed": closed,
        "packets": packets,
    }))
}

/// A request as the oracle writes a packet: no test key when the box did not parse.
fn packet_json(request: photon_game::test_instance::InstanceRequest) -> Value {
    let mut packet = json!({
        "action": request.action.name(),
        "size": [request.size.0, request.size.1, request.size.2],
        "rotation": request.rotation.name(),
        "ignore_entities": request.ignore_entities,
    });
    if let Some(test) = request.test {
        packet["test"] = json!(test);
    }
    packet
}
