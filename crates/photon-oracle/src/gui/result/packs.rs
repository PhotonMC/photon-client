//! The Resource Packs screen over the packs a row lists.

use super::*;
use photon_assets::{
    Compatibility, PackEntry, PackSource, Position, Repository, SelectionConfig, SelectionModel,
    Title,
};
use photon_gui::options::packs::PackScreen;
use photon_render::window::scale::WindowScale;

struct NoClipboard;

impl photon_gui::widgets::edit_box::Clipboard for NoClipboard {
    fn get(&mut self) -> String {
        String::new()
    }
    fn set(&mut self, _text: &str) {}
}

fn int_at(value: &Value) -> i32 {
    value.as_i64().unwrap_or(0) as i32
}

/// One pack of the row, and whether the row has it selected.
fn pack_entry(pack: &Value) -> Result<(PackEntry, bool), String> {
    let flag = |key: &str| pack[key].as_bool().unwrap_or(false);
    let description = match pack["description_key"].as_str() {
        Some(key) => json!({"translate": key}).to_string(),
        None => json!(pack["description"].as_str().unwrap_or_default()).to_string(),
    };
    let title = match pack["title_key"].as_str() {
        Some(key) => Title::Translatable(key.to_owned()),
        None => Title::Literal(pack["title"].as_str().unwrap_or_default().to_owned()),
    };
    let compatibility = match pack["compatibility"].as_str().unwrap_or("COMPATIBLE") {
        "TOO_OLD" => Compatibility::TooOld,
        "TOO_NEW" => Compatibility::TooNew,
        "UNKNOWN" => Compatibility::Unknown,
        _ => Compatibility::Compatible,
    };
    let entry = PackEntry {
        id: pack["id"].as_str().ok_or("pack id")?.to_owned(),
        title,
        description: photon_core::json::parse(&description).map_err(|_| "description")?,
        compatibility,
        source: if flag("builtin") {
            PackSource::BuiltIn
        } else {
            PackSource::Default
        },
        config: SelectionConfig {
            required: flag("required"),
            fixed_position: flag("fixed"),
            default_position: if pack["top"].as_bool().unwrap_or(true) {
                Position::Top
            } else {
                Position::Bottom
            },
        },
        icon: None,
    };
    Ok((entry, flag("selected")))
}

/// The model over the row's packs.
fn pack_model(row: &Value) -> Result<SelectionModel, String> {
    let mut entries = Vec::new();
    let mut selected = Vec::new();
    for pack in row["packs"].as_array().ok_or("packs")? {
        let (entry, chosen) = pack_entry(pack)?;
        if chosen {
            selected.push(entry.id.clone());
        }
        entries.push(entry);
    }
    Ok(SelectionModel::new(Repository::new(entries, &selected)))
}

/// One input step of the row.
/// A key named in a row, and whether Shift is down.
fn pack_key(name: &str) -> Option<(glfw::Key, bool)> {
    let (shift, bare) = match name.strip_prefix("shift+") {
        Some(bare) => (true, bare),
        None => (false, name),
    };
    let key = match bare {
        "backspace" => glfw::Key::Backspace,
        "tab" => glfw::Key::Tab,
        "up" => glfw::Key::Up,
        "down" => glfw::Key::Down,
        "left" => glfw::Key::Left,
        "right" => glfw::Key::Right,
        "enter" => glfw::Key::Enter,
        "space" => glfw::Key::Space,
        _ => return None,
    };
    Some((key, shift))
}

fn pack_step(
    screen: &mut PackScreen,
    font: &Font,
    step: &Value,
    (at, last): ((f64, f64), (f64, f64)),
) -> Result<(), String> {
    match step["do"].as_str().unwrap_or_default() {
        "click" => {
            screen.clicked(font, at);
        }
        "char" => {
            if let Some(character) = step["char"].as_str().and_then(|text| text.chars().next()) {
                screen.char_typed(font, character);
            }
        }
        "key" => {
            let name = step["key"].as_str().unwrap_or_default();
            let (key, shift) = pack_key(name).ok_or_else(|| format!("unknown key {name}"))?;
            screen.key_pressed(font, key, (false, shift), &mut NoClipboard);
        }
        "drag" => screen.dragged(at, at.1 - last.1),
        "release" => screen.released(),
        "scroll" => screen.scrolled(at, step["dy"].as_f64().unwrap_or(0.0)),
        other => return Err(format!("unknown step {other}")),
    }
    Ok(())
}

/// The row's steps, with the frame the reference draws between one event and the next.
fn run_steps(
    screen: &mut PackScreen,
    (font, atlas, lang): (&Font, &GuiAtlas, &photon_game::text::Lang),
    row: &Value,
    mouse: (i32, i32),
) -> Result<(), String> {
    let mut last = (0.0, 0.0);
    let mut clock = 0_u64;
    for step in row["steps"].as_array().ok_or("steps")? {
        if step["do"] == "frame" {
            clock = step["ms"].as_u64().unwrap_or(0);
            let mut thrown = GuiFrame {
                millis: clock,
                ..GuiFrame::default()
            };
            screen.draw(&mut thrown, atlas, font, lang, mouse);
            continue;
        }
        // The reference draws a frame between one event and the next.
        let mut between = GuiFrame {
            millis: clock,
            ..GuiFrame::default()
        };
        screen.draw(&mut between, atlas, font, lang, mouse);
        let at = (
            step["x"].as_f64().unwrap_or(last.0),
            step["y"].as_f64().unwrap_or(last.1),
        );
        pack_step(screen, font, step, (at, last))?;
        last = at;
    }
    Ok(())
}

pub(super) fn pack_selection_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
) -> Result<Value, String> {
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let source = match &ctx.languages {
        Some(source) => source,
        None => ctx
            .languages
            .insert(photon_game::language::LanguageSource::open(&ctx.root).ok_or("languages")?),
    };
    let lang = source.lang("en_us");
    let gui_scale = int_at(&row["gui_scale"]);
    let mut window = WindowScale::new(
        int_at(&row["framebuffer"][0]),
        int_at(&row["framebuffer"][1]),
    );
    window.set_gui_scale(window.calculate_scale(gui_scale, false));
    let mut screen = PackScreen::new(
        font,
        &lang,
        (window.gui_scaled_width(), window.gui_scaled_height()),
        row["in_world"].as_bool().unwrap_or(false),
        pack_model(row)?,
        std::path::PathBuf::new(),
    );
    let mouse = (int_at(&row["mouse"][0]), int_at(&row["mouse"][1]));
    run_steps(&mut screen, (font, atlas, &lang), row, mouse)?;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        millis: row["ms"].as_u64().unwrap_or(0),
        ..GuiFrame::default()
    };
    screen.draw(&mut frame, atlas, font, &lang, mouse);
    Ok(json!({
        "label": row["label"],
        "width": window.gui_scaled_width(),
        "height": window.gui_scaled_height(),
        "gui_scale": gui_scale,
        "scroll": screen.scroll_amount(),
        "focus": screen.focus_name(),
        "selected": screen.selected_ids(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
