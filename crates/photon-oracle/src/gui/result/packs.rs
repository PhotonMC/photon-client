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
    let literal = json!(pack["description"].as_str().unwrap_or_default()).to_string();
    let compatibility = match pack["compatibility"].as_str().unwrap_or("COMPATIBLE") {
        "TOO_OLD" => Compatibility::TooOld,
        "TOO_NEW" => Compatibility::TooNew,
        "UNKNOWN" => Compatibility::Unknown,
        _ => Compatibility::Compatible,
    };
    let entry = PackEntry {
        id: pack["id"].as_str().ok_or("pack id")?.to_owned(),
        title: Title::Literal(pack["title"].as_str().unwrap_or_default().to_owned()),
        description: photon_core::json::parse(&literal).map_err(|_| "description")?,
        compatibility,
        source: PackSource::Default,
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
        "key" => match step["key"].as_str().unwrap_or_default() {
            "backspace" => {
                screen.key_pressed(font, glfw::Key::Backspace, (false, false), &mut NoClipboard);
            }
            other => return Err(format!("unknown key {other}")),
        },
        "drag" => screen.dragged(at, at.1 - last.1),
        "release" => screen.released(),
        "scroll" => screen.scrolled(at, step["dy"].as_f64().unwrap_or(0.0)),
        other => return Err(format!("unknown step {other}")),
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
    let mut last = (0.0, 0.0);
    for step in row["steps"].as_array().ok_or("steps")? {
        let at = (
            step["x"].as_f64().unwrap_or(last.0),
            step["y"].as_f64().unwrap_or(last.1),
        );
        pack_step(&mut screen, font, step, (at, last))?;
        last = at;
    }
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let mouse = (int_at(&row["mouse"][0]), int_at(&row["mouse"][1]));
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
