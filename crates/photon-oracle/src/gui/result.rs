use super::*;

use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

mod chat;
use chat::*;
mod hud;
use hud::*;
mod screens;
use screens::*;
mod settings;
use settings::*;
mod container;
use container::*;
mod title;
use title::*;
mod signs;
use signs::*;
mod books;
use books::*;
mod keybinds;
use keybinds::*;
mod tooltip;
use tooltip::*;

/// What the GUI scenarios share: the game's assets, the font and the atlas drawn from it, and
/// the language source, read once because its index has thousands of files.
pub(super) struct GuiContext {
    pub(super) root: std::path::PathBuf,
    pub(super) font: Font,
    pub(super) atlas: GuiAtlas,
    /// The atlas as the GPU has it, which icons are put into.
    pub(super) atlas_image: photon_render::texture::png::DecodedImage,
    pub(super) languages: Option<photon_game::language::LanguageSource>,
}

pub(crate) fn gui_result(request: &Value) -> Result<Value, String> {
    if request["operation"].as_str() == Some("name_tags") {
        return crate::name_tags_result(request);
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/assets");
    let font = Font::load(&root).ok_or("font")?;
    let alt = Font::load_alt(&root).ok_or("alt font")?;
    let (atlas, atlas_image) = GuiAtlas::load(&root, &[&font, &alt]).ok_or("gui atlas")?;
    if request["operation"] == "key_mappings" {
        return Ok(key_mappings());
    }
    if request["operation"] == "panorama_view" {
        return panorama_view(request);
    }
    if request["operation"] == "command_parse" {
        return crate::command_parse_result(request, &root);
    }
    if request["operation"] == "command_suggest" {
        return crate::command_suggest_result(request);
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("gui rows must be an array")?;
    let mut out = Vec::new();
    let mut ctx = GuiContext {
        root,
        font,
        atlas,
        atlas_image,
        languages: None,
    };
    let operation = crate::string(request, "operation")?;
    for row in rows {
        out.push(match operation {
            "chat" => chat_case(&mut ctx, row)?,
            "suggestions" => suggestions_case(&mut ctx, row)?,
            "chat_draft" => chat_draft_case(&mut ctx, row)?,
            "titles" => titles_case(&mut ctx, row)?,
            "sidebar" => sidebar_case(&mut ctx, row)?,
            "pause_screen" => pause_screen_case(&mut ctx, row)?,
            "title_screen" => title_screen_case(&mut ctx, row)?,
            "join_multiplayer_screen" => join_multiplayer_screen_case(&mut ctx, row)?,
            "server_form" => server_form_case(&mut ctx, row)?,
            "connect_screen" => connect_screen_case(&mut ctx, row)?,
            "confirm_screen" => confirm_screen_case(&mut ctx, row)?,
            "widgets" => widgets_case(&mut ctx, row)?,
            "options_screen" => options_screen_case(&mut ctx, row)?,
            "video_settings_screen" => video_settings_screen_case(&mut ctx, row)?,
            "chat_options_screen"
            | "mouse_settings_screen"
            | "controls_screen"
            | "sound_options_screen" => chat_options_screen_case(&mut ctx, row, operation)?,
            "key_binds_screen" => key_binds_screen_case(&mut ctx, row)?,
            "lang_text" => lang_text_case(&mut ctx, row)?,
            "language_screen" => language_screen_case(&mut ctx, row)?,
            "death_screen" => death_screen_case(&mut ctx, row)?,
            "credits_screen" => credits_screen_case(&mut ctx, row)?,
            "disconnected_screen" => disconnected_screen_case(&mut ctx, row)?,
            "hud_hotbar" => hud_hotbar_case(&mut ctx, row)?,
            "hud_status" => hud_status_case(&mut ctx, row)?,
            "boss_bar" => boss_bar_case(&mut ctx, row)?,
            "locator_bar" => locator_bar_case(&mut ctx, row)?,
            "tab_list" => tab_list_case(&mut ctx, row)?,
            "container_screen" => container_screen_case(&mut ctx, row)?,
            "game_mode_switcher" => game_mode_switcher_case(&mut ctx, row)?,
            "sign_texts" => sign_texts_case(&mut ctx, row)?,
            "sign_edit_screen" => sign_edit_screen_case(&mut ctx, row)?,
            "book_edit_screen" => book_edit_screen_case(&mut ctx, row)?,
            "book_sign_screen" => book_sign_screen_case(&mut ctx, row)?,
            "book_view" => book_view_case(&mut ctx, row)?,
            "tooltip_lines" => tooltip_lines_case(&mut ctx, row)?,
            "bundle_tooltip" => bundle_tooltip_case(&mut ctx, row)?,
            _ => return Err("unknown gui operation".into()),
        });
    }
    Ok(json!({"rows": out}))
}

/// `key_mappings`: the key mappings, their categories and the named keys, as the data crate has them.
fn key_mappings() -> Value {
    let mappings: Vec<Value> = photon_data::keys::mappings()
        .map(|mapping| {
            json!({
                "name": mapping.name,
                "category": mapping.category,
                "key": mapping.default_key,
                "order": mapping.order,
            })
        })
        .collect();
    let keys: Vec<Value> = photon_data::keys::named_keys()
        .iter()
        .map(|(kind, code, name)| json!({"type": kind, "code": code, "name": name}))
        .collect();
    json!({
        "mappings": mappings,
        "categories": photon_data::keys::categories(),
        "keys": keys,
    })
}

/// `panorama_view`: the cube's model-view and the spin a frame adds, from the production module.
fn panorama_view(request: &Value) -> Result<Value, String> {
    use crate::{f32_field, f64_field};
    let matrix = photon_render::panorama::model_view(
        f32_field(request, "rot_x")?,
        f32_field(request, "rot_y")?,
    );
    let bits: Vec<String> = matrix
        .iter()
        .flatten()
        .map(|value| format!("{:08x}", value.to_bits()))
        .collect();
    let spin = photon_render::panorama::spin_after(
        f32_field(request, "spin")?,
        f32_field(request, "ticks")?,
        f64_field(request, "speed")?,
    );
    Ok(json!({"matrix": bits, "spin": format!("{:08x}", spin.to_bits())}))
}
