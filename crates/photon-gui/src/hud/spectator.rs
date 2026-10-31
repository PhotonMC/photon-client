//! The spectator menu over the hotbar (`SpectatorGui.extractHotbar` and `extractAction`): the
//! hotbar sprites faded by the menu's age, an icon in each slot, the key that picks it, and the
//! selected entry's name (or the category's prompt) above.
//!
//! Not drawn: a player's face (the icon of a player entry, and of a team entry over its colour).

use crate::runs::{FontRuns, FrameRuns};
use photon_game::text::Run;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// What a slot's icon is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Icon {
    Sprite(&'static str),
    /// A team's colour, `0xRRGGBB`, behind its player's face.
    Team(Option<u32>),
    /// A player's face.
    Player,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpectatorSlot {
    pub icon: Icon,
    pub enabled: bool,
    /// The hotbar key's label, shown over an enabled slot.
    pub key: Vec<Run>,
}

/// The menu as one frame draws it.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectatorHud {
    /// `getHotbarAlpha`.
    pub alpha: f32,
    /// The selected slot, or -1.
    pub selected: i32,
    /// The nine slots; `None` is `EMPTY_SLOT`.
    pub slots: Vec<Option<SpectatorSlot>>,
    /// The selected entry's name, or the prompt of the category.
    pub action: Vec<Run>,
}

/// `ARGB.white(alpha)`.
fn white(alpha: f32) -> u32 {
    (((alpha * 255.0) as u32).min(255) << 24) | 0x00FF_FFFF
}

/// `ARGB.colorFromFloat(alpha, b, b, b)`.
fn grey(alpha: f32, brightness: f32) -> u32 {
    let channel = ((brightness * 255.0) as u32).min(255);
    (((alpha * 255.0) as u32).min(255) << 24) | (channel << 16) | (channel << 8) | channel
}

/// `ARGB.scaleRGB` of a team colour, opaque.
fn scaled(rgb: u32, brightness: f32) -> u32 {
    let part = |shift: u32| (((rgb >> shift & 0xFF) as f32 * brightness) as u32).min(255);
    0xFF00_0000 | part(16) << 16 | part(8) << 8 | part(0)
}

/// `SpectatorGui.extractHotbar`.
pub fn draw_hotbar(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    (width, height): (i32, i32),
    menu: &SpectatorHud,
) {
    let alpha = menu.alpha;
    let centre = width / 2;
    let y = (height as f32 - 22.0 * alpha).floor() as i32;
    let colour = white(alpha);
    frame.sprite_tinted(atlas, "hud/hotbar", (centre - 91, y, 182, 22), colour);
    if menu.selected >= 0 {
        let x = centre - 91 - 1 + menu.selected * 20;
        frame.sprite_tinted(atlas, "hud/hotbar_selection", (x, y - 1, 24, 23), colour);
    }
    for (slot, entry) in menu.slots.iter().enumerate() {
        if let Some(entry) = entry {
            let x = centre - 90 + slot as i32 * 20 + 2;
            slot_icon(frame, (atlas, font), (x, y + 3), alpha, entry);
        }
    }
}

/// `extractSlot`: the icon at its brightness, then the key of an enabled slot.
fn slot_icon(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    (x, y): (i32, i32),
    alpha: f32,
    slot: &SpectatorSlot,
) {
    let brightness = if slot.enabled { 1.0 } else { 0.25 };
    match &slot.icon {
        Icon::Sprite(name) => {
            frame.sprite_tinted(atlas, name, (x, y, 16, 16), grey(alpha, brightness));
        }
        Icon::Team(Some(rgb)) => {
            frame.fill(
                atlas,
                (x + 1, y + 1, x + 15, y + 15),
                scaled(*rgb, brightness),
            );
        }
        Icon::Team(None) | Icon::Player => {}
    }
    if alpha > 0.0 && slot.enabled {
        let width = font.width_of_runs(&slot.key);
        frame.styled_text(
            atlas,
            font,
            &slot.key,
            (x + 19 - 2 - width, y + 6 + 3),
            white(alpha),
            true,
        );
    }
}

/// `SpectatorGui.extractAction`: the name above the hotbar on a backdrop. `backdrop` is the text
/// background's opacity.
pub fn draw_action(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    (width, height): (i32, i32),
    (menu, backdrop): (&SpectatorHud, f32),
) {
    if menu.alpha <= 0.0 {
        return;
    }
    let text_width = font.width_of_runs(&menu.action);
    let (x, y) = ((width - text_width) / 2, height - 35);
    let alpha = ((menu.alpha * 255.0) as i32).min(255);
    let background = (backdrop * 255.0).floor() as i32;
    if background != 0 {
        frame.fill(
            atlas,
            (x - 2, y - 2, x + text_width + 2, y + 9 + 2),
            ((background * alpha / 255) as u32) << 24,
        );
    }
    frame.styled_text(
        atlas,
        font,
        &menu.action,
        (x, y),
        (alpha as u32) << 24 | 0x00FF_FFFF,
        true,
    );
}
