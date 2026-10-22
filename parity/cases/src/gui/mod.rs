//! The `gui` family: screens and heads-up displays drawn by the reference's
//! own GUI code, with their glyphs left out.

mod accessibility;
mod advancements;
mod books;
mod chat;
mod command_block;
mod commands;
mod confirm;
mod container;
mod credits;
mod debug;
mod debug_entry_rows;
mod dialog;
mod draft;
mod hud;
mod icons;
mod item_conditions;
mod jigsaw;
mod keybinds;
mod language;
mod locator;
mod mob_tags;
mod options;
mod panels;
mod panorama;
mod restrictions;
mod screens;
mod setting;
mod signs;
mod social;
mod stats;
mod status;
mod structure_block;
mod suggest;
mod test_block;
mod test_instance;
mod title;
mod toasts;
mod tooltips;
mod tree;
mod tutorial;
mod video;
mod widgets;

use serde_json::{Value, json};

use crate::case::case;

/// A `gui` case of `operation` with its `rows`; `name` is the case's id.
pub(crate) fn gui(name: &str, operation: &str, rows: Vec<Value>) -> Value {
    case(format!("gui/{name}"), "gui", operation, json!({"rows": rows}))
}

type Generator = fn() -> Vec<Value>;

/// The generators in the order the cases are listed.
const ORDER: &[Generator] = &[
    || vec![chat::chat_case()],
    || vec![draft::suggestions()],
    || vec![hud::titles()],
    || vec![hud::boss_bar()],
    || vec![hud::sidebar()],
    || vec![commands::command_parse()],
    || vec![suggest::command_suggest()],
    || vec![commands::command_verify()],
    || vec![item_conditions::item_condition(), item_conditions::item_select(), item_conditions::item_needle()],
    || vec![screens::pause_screen()],
    || vec![screens::death_screen()],
    || vec![screens::disconnected_screen()],
    || vec![credits::credits_screen()],
    || vec![social::social_interactions_screen()],
    || vec![hud::tab_list()],
    || vec![container::container_screen()],
    || vec![widgets::game_mode_switcher()],
    || vec![confirm::confirm_screen()],
    || vec![dialog::dialog_screen()],
    || vec![restrictions::restrictions_screen()],
    || vec![widgets::widgets()],
    || vec![options::options_screen()],
    || vec![video::video_settings_screen()],
    || vec![panels::chat_options_screen()],
    || vec![accessibility::accessibility_options_screen()],
    || vec![panels::skin_customization_screen()],
    || vec![crate::sound::music_manager()],
    panels::mouse_and_controls,
    || vec![panels::sound_options_screen()],
    || vec![status::hud_hotbar()],
    || vec![status::hud_sleep()],
    || vec![debug::hud_debug_overlay()],
    || vec![debug::hud_debug_lines()],
    || vec![debug::debug_entries()],
    || vec![debug::debug_options_screen()],
    || vec![stats::stat_format()],
    || vec![stats::stats_screen()],
    || vec![advancements::client_advancements()],
    || vec![advancements::advancements_screen()],
    || vec![status::subtitles()],
    || vec![toasts::toasts()],
    || vec![tutorial::tutorial()],
    || vec![status::hud_status()],
    language::language_cases,
    || vec![locator::locator_bar()],
    language::settings_keys,
    || vec![draft::chat_draft()],
    || vec![locator::name_tags()],
    || vec![title::title_screen()],
    || vec![title::join_multiplayer_screen()],
    || vec![title::server_form()],
    || vec![title::connect_screen()],
    || vec![title::server_list_pings()],
    || vec![title::server_list_icons()],
    || vec![panels::key_mappings()],
    panorama::panorama,
    || vec![keybinds::key_binds_screen()],
    || vec![signs::sign_texts()],
    || vec![signs::sign_edit_screen()],
    || vec![command_block::command_block_screen()],
    || vec![structure_block::structure_block_screen()],
    || vec![jigsaw::jigsaw_screen()],
    || vec![test_block::test_block_screen()],
    || vec![test_instance::test_instance_screen()],
    || vec![books::book_edit_screen()],
    || vec![books::book_sign_screen()],
    || vec![books::book_view()],
    || vec![tooltips::tooltip_lines()],
    || vec![tooltips::bundle_tooltip()],
];

pub fn cases() -> Vec<Value> {
    ORDER.iter().flat_map(|generate| generate()).collect()
}
