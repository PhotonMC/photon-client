use super::*;
use photon_game::structure_block::{
    Mirror, Rotation, StructureBlockContents, StructureMode, structure_name,
};
use photon_game::text::Lang;
use photon_gui::screen::structure_block::{
    Initial, StructureBlockAction, StructureBlockEdit, StructureBlockScreen,
};

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

fn mode_of(name: &str) -> StructureMode {
    StructureMode::ALL
        .into_iter()
        .find(|mode| mode.name() == name)
        .unwrap_or(StructureMode::Data)
}

fn mirror_of(name: &str) -> Mirror {
    Mirror::ALL
        .into_iter()
        .find(|mirror| mirror.name() == name)
        .unwrap_or(Mirror::None)
}

fn rotation_of(name: &str) -> Rotation {
    match name {
        "clockwise_90" => Rotation::Clockwise90,
        "clockwise_180" => Rotation::Clockwise180,
        "counterclockwise_90" => Rotation::Counterclockwise90,
        _ => Rotation::None,
    }
}

fn rotation_name(rotation: Rotation) -> &'static str {
    match rotation {
        Rotation::None => "none",
        Rotation::Clockwise90 => "clockwise_90",
        Rotation::Clockwise180 => "clockwise_180",
        Rotation::Counterclockwise90 => "counterclockwise_90",
    }
}

fn triple(value: &Value) -> (i32, i32, i32) {
    let at = |index: usize| value[index].as_i64().unwrap_or(0) as i32;
    (at(0), at(1), at(2))
}

fn contents_of(row: &Value) -> StructureBlockContents {
    let flag = |key: &str| row[key].as_bool() == Some(true);
    StructureBlockContents {
        name: structure_name(row["name"].as_str().unwrap_or_default()),
        metadata: row["metadata"].as_str().unwrap_or_default().to_owned(),
        pos: triple(&row["pos"]),
        size: triple(&row["size"]),
        mirror: mirror_of(row["mirror"].as_str().unwrap_or_default()),
        rotation: rotation_of(row["rotation"].as_str().unwrap_or_default()),
        mode: mode_of(row["mode"].as_str().unwrap_or_default()),
        ignore_entities: flag("ignore_entities"),
        strict: flag("strict"),
        show_air: flag("show_air"),
        show_bounding_box: flag("show_bounding_box"),
        integrity: row["integrity"].as_f64().unwrap_or(1.0) as f32,
        seed: row["seed"].as_i64().unwrap_or(0),
    }
}

fn held_json(held: Initial) -> Value {
    json!({
        "mode": held.mode.name(),
        "mirror": held.mirror.name(),
        "rotation": rotation_name(held.rotation),
        "ignore_entities": held.ignore_entities,
        "strict": held.strict,
        "show_air": held.show_air,
        "show_bounding_box": held.show_bounding_box,
    })
}

/// `ServerboundSetStructureBlockPacket`, without the block's position.
fn packet(edit: &StructureBlockEdit) -> Value {
    let update = format!("{:?}", edit.update);
    let snake: String = update
        .chars()
        .enumerate()
        .flat_map(|(at, each)| {
            let lower = each.to_ascii_lowercase();
            if at > 0 && each.is_ascii_uppercase() {
                vec!['_', lower]
            } else {
                vec![lower]
            }
        })
        .collect();
    json!({
        "update": snake,
        "mode": edit.mode.name(),
        "name": edit.name,
        "offset": [edit.offset.0, edit.offset.1, edit.offset.2],
        "size": [edit.size.0, edit.size.1, edit.size.2],
        "mirror": edit.mirror.name(),
        "rotation": rotation_name(edit.rotation),
        "data": edit.data,
        "ignore_entities": edit.ignore_entities,
        "strict": edit.strict,
        "show_air": edit.show_air,
        "show_bounding_box": edit.show_bounding_box,
        "integrity_bits": edit.integrity.to_bits() as i32,
        "seed": edit.seed,
    })
}

/// What the screen did so far: whether it closed, what it sent, and the values the block ends with.
struct Outcome {
    closed: bool,
    packets: Vec<Value>,
    /// What the block was given when the screen was cancelled.
    restored: Option<Initial>,
}

impl Outcome {
    fn settle(&mut self, action: StructureBlockAction) {
        match action {
            StructureBlockAction::Stay => {}
            StructureBlockAction::Cancel(initial) => {
                self.closed = true;
                self.restored = Some(initial);
            }
            StructureBlockAction::Done(edit) => {
                self.closed = true;
                self.packets.push(packet(&edit));
                self.restored = None;
            }
        }
    }
}

/// One step of a row: text, a key, a click, the wheel or a new size.
fn step(
    screen: &mut StructureBlockScreen,
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

/// `structure_block_screen`: the structure block's screen after text, keys, clicks, the wheel and
/// new sizes, drawn at a time after it opened, whether it closed, the packets it would send and
/// the values the block holds in the end.
pub(super) fn structure_block_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
) -> Result<Value, String> {
    let lang = Lang::load(&ctx.root);
    let font = std::sync::Arc::new(Font::load(&ctx.root).ok_or("font")?);
    let int = |value: &Value| value.as_i64().unwrap_or(0) as i32;
    let size = (int(&row["width"]), int(&row["height"]));
    let mut outcome = Outcome {
        closed: false,
        packets: Vec::new(),
        restored: None,
    };
    let mut screen = StructureBlockScreen::new((&font, &lang), &contents_of(row), size);
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
    let held = outcome.restored.unwrap_or_else(|| screen.held());
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        "closed": outcome.closed,
        "sounds": sounds,
        "packets": outcome.packets,
        "held": held_json(held),
    }))
}
