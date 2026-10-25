//! The settings screens that take the same inputs, and the rescale the font settings cause.

use super::settings::{row_options, sound_screen};
use super::*;

/// `Minecraft.resizeGui` after the font settings toggled `forceUnicodeFont`: the scale is chosen
/// again and the open screen laid out at the new size.
pub(super) fn rescale_font(
    screen: &mut Shell,
    window: &mut WindowScale,
    (gui_scale, unicode): (i32, &mut bool),
) {
    let Shell::Font(font) = screen else {
        return;
    };
    let shown = FontChoices {
        force_unicode_font: *unicode,
        japanese_glyph_variants: false,
    };
    let now = font.choices(shown).force_unicode_font;
    if now != *unicode {
        *unicode = now;
        window.set_gui_scale(window.calculate_scale(gui_scale, now));
        font.resize((window.gui_scaled_width(), window.gui_scaled_height()));
    }
}

use photon_gui::options::accessibility::AccessibilityScreen;
use photon_gui::options::chat::ChatOptionsScreen;
use photon_gui::options::font::{FontChoices, FontOptionsScreen};
use photon_gui::options::mouse::{ControlsScreen, MouseSettingsScreen};
use photon_gui::options::skin::SkinCustomizationScreen;
use photon_gui::options::sound::SoundOptionsScreen;
use photon_gui::options::world::{WorldFacts, WorldOptionsScreen};
use photon_render::window::scale::WindowScale;

// The settings screens that have the same inputs.
pub(super) enum Shell {
    Chat(ChatOptionsScreen),
    Accessibility(AccessibilityScreen),
    Mouse(MouseSettingsScreen),
    Controls(ControlsScreen),
    Sound(SoundOptionsScreen),
    Skin(SkinCustomizationScreen),
    Font(FontOptionsScreen),
    World(WorldOptionsScreen),
}

/// The screen of a settings operation, over the options the row has.
pub(super) fn settings_shell(
    operation: &str,
    row: &Value,
    (font, lang): (&photon_render::gui::font::Font, &photon_game::text::Lang),
    (size, in_world): ((i32, i32), bool),
) -> Shell {
    let options = row_options(row).unwrap_or_default();
    match operation {
        "skin_customization_screen" => Shell::Skin(SkinCustomizationScreen::new(
            font, lang, size, in_world, &options,
        )),
        "world_options_screen" => Shell::World(WorldOptionsScreen::new(
            font,
            lang,
            size,
            in_world,
            world_facts(&row["world"]),
        )),
        "font_options_screen" => {
            Shell::Font(FontOptionsScreen::new(font, lang, size, in_world, &options))
        }
        "chat_options_screen" => {
            Shell::Chat(ChatOptionsScreen::new(font, lang, size, in_world, &options))
        }
        "accessibility_options_screen" => Shell::Accessibility(AccessibilityScreen::new(
            font,
            lang,
            size,
            in_world,
            (
                &options,
                row["options"]["minecart"].as_bool().unwrap_or(false),
            ),
        )),
        "mouse_settings_screen" => Shell::Mouse(MouseSettingsScreen::new(
            font, lang, size, in_world, &options,
        )),
        "sound_options_screen" => {
            Shell::Sound(sound_screen(font, lang, (size, in_world), &options))
        }
        _ => Shell::Controls(ControlsScreen::new(font, lang, size, in_world)),
    }
}

/// What the row says the server told the client about the world.
fn world_facts(world: &Value) -> WorldFacts {
    let difficulty = match world["difficulty"].as_str().unwrap_or("normal") {
        "peaceful" => 0,
        "easy" => 1,
        "hard" => 3,
        _ => 2,
    };
    WorldFacts {
        difficulty,
        locked: world["locked"].as_bool().unwrap_or(false),
        hardcore: world["hardcore"].as_bool().unwrap_or(false),
        gamemaster: world["permission"].as_i64().unwrap_or(0) >= 2,
    }
}
