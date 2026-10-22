//! The keys a dialog row presses: the focused text box takes what it edits with, the screen the rest.

use photon_gui::screen::dialog::{DialogScreen, Press};
use photon_gui::widgets::edit_box::Clipboard;
use photon_render::gui::font::Font;

/// What a box does with the clipboard: nothing the reference can be asked about.
struct NoClip;

impl Clipboard for NoClip {
    fn get(&mut self) -> String {
        String::new()
    }

    fn set(&mut self, _: &str) {}
}

/// The key a key name stands for, when a text box may take it.
fn edit_of(base: &str) -> Option<glfw::Key> {
    use glfw::Key;
    const KEYS: [(&str, Key); 12] = [
        ("backspace", Key::Backspace),
        ("delete", Key::Delete),
        ("left", Key::Left),
        ("right", Key::Right),
        ("home", Key::Home),
        ("end", Key::End),
        ("a", Key::A),
        ("enter", Key::Enter),
        ("up", Key::Up),
        ("down", Key::Down),
        ("pageup", Key::PageUp),
        ("pagedown", Key::PageDown),
    ];
    KEYS.iter()
        .find(|(name, _)| *name == base)
        .map(|(_, key)| *key)
}

/// A key the reference's screen was given, with the modifiers its name carries
/// (`shift+`, `ctrl+`): the focused box takes it first.
pub(super) fn key_press(
    screen: &mut DialogScreen,
    font: &Font,
    key: &str,
) -> Result<Option<Press>, String> {
    let base = key.rsplit('+').next().unwrap_or(key);
    let modifiers = (key.contains("ctrl+"), key.contains("shift+"));
    if let Some(edit) = edit_of(base) {
        if screen.edit_key(font, edit, modifiers, &mut NoClip) {
            return Ok(None);
        }
        // A key the box would have edited with, and none has the focus: nothing happens.
        if !matches!(base, "left" | "right" | "enter") {
            return Ok(None);
        }
    }
    plain_key(screen, key)
}

/// A key the screen itself takes.
fn plain_key(screen: &mut DialogScreen, key: &str) -> Result<Option<Press>, String> {
    Ok(match key {
        "tab" | "shift+tab" => {
            screen.tab(key == "tab");
            None
        }
        "enter" | "space" => screen.pressed(),
        "shift+enter" => {
            screen.set_shift(true);
            let press = screen.pressed();
            screen.set_shift(false);
            press
        }
        "escape" => screen.escape(),
        "left" | "right" => {
            screen.arrow(key == "right");
            None
        }
        other => return Err(format!("unknown key {other}")),
    })
}
