use super::*;

use voxel_render::gui::font::Font;
use voxel_render::gui::{GuiAtlas, GuiFrame};

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

/// What the GUI scenarios share: the game's assets, the font and the atlas drawn from it, and
/// the language source, read once because its index has thousands of files.
pub(super) struct GuiContext {
    pub(super) root: std::path::PathBuf,
    pub(super) font: Font,
    pub(super) atlas: GuiAtlas,
    pub(super) languages: Option<voxel_game::language::LanguageSource>,
}

pub(crate) fn gui_result(request: &Value) -> Result<Value, String> {
    if request["operation"].as_str() == Some("name_tags") {
        return crate::name_tags_result(request);
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/assets");
    let font = Font::load(&root).ok_or("font")?;
    let alt = Font::load_alt(&root).ok_or("alt font")?;
    let (atlas, _) = GuiAtlas::load(&root, &[&font, &alt]).ok_or("gui atlas")?;
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
            "confirm_screen" => confirm_screen_case(&mut ctx, row)?,
            "widgets" => widgets_case(&mut ctx, row)?,
            "options_screen" => options_screen_case(&mut ctx, row)?,
            "video_settings_screen" => video_settings_screen_case(&mut ctx, row)?,
            "chat_options_screen"
            | "mouse_settings_screen"
            | "controls_screen"
            | "sound_options_screen" => chat_options_screen_case(&mut ctx, row, operation)?,
            "lang_text" => lang_text_case(&mut ctx, row)?,
            "language_screen" => language_screen_case(&mut ctx, row)?,
            "death_screen" => death_screen_case(&mut ctx, row)?,
            "disconnected_screen" => disconnected_screen_case(&mut ctx, row)?,
            "hud_hotbar" => hud_hotbar_case(&mut ctx, row)?,
            "hud_status" => hud_status_case(&mut ctx, row)?,
            "boss_bar" => boss_bar_case(&mut ctx, row)?,
            "locator_bar" => locator_bar_case(&mut ctx, row)?,
            "tab_list" => tab_list_case(&mut ctx, row)?,
            "container_screen" => container_screen_case(&mut ctx, row)?,
            _ => return Err("unknown gui operation".into()),
        });
    }
    Ok(json!({"rows": out}))
}
