//! The screens of a join: `ConnectScreen` (what the connection is doing, and Cancel) and, once
//! the level has begun, `LevelLoadingScreen` ("Loading terrain..."; a server shows no progress).

use photon_game::text::{Component, Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

/// `Screen.extractBackground` with no level: the panorama's overlay and the menu's background.
fn menu_background(frame: &mut GuiFrame, atlas: &GuiAtlas, (width, height): (i32, i32)) {
    frame.blit_texture_stretched(
        atlas,
        "title/background/panorama_overlay",
        (16, 128),
        (0, 0, 16, 128),
        (0, 0, width, height),
    );
    frame.tile_texture(atlas, "menu_background", (32, 32), (0, 0, width, height));
}

/// Centred text with its shadow (`GuiGraphics.centeredText`).
fn centred(frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, text: &[Run], (x, y): (i32, i32)) {
    let width = font.width_of_runs(text);
    frame.styled_text(atlas, font, text, x - width / 2, y, 0xFFFF_FFFF, true);
}

pub struct ConnectScreen {
    arena: Arena,
    cancel: Widget,
    status: Vec<Run>,
    size: (i32, i32),
}

impl ConnectScreen {
    /// `init`, the status `connect.connecting` until told another.
    pub fn new(lang: &Lang, size: (i32, i32)) -> Self {
        let mut arena = Arena::default();
        let rect = arena.add(200, 20);
        arena.0[rect].x = size.0 / 2 - 100;
        arena.0[rect].y = size.1 / 4 + 120 + 12;
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        Self {
            arena,
            cancel: Widget {
                rect,
                active: true,
                kind: Kind::Button {
                    label: runs("gui.cancel"),
                },
            },
            status: runs("connect.connecting"),
            size,
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// `updateStatus` to the text `key` names.
    pub fn set_status(&mut self, lang: &Lang, key: &str) {
        self.status = lang.runs(&Component::translatable(key), &Style::default());
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        menu_background(frame, atlas, self.size);
        let highlighted = self.cancel.contains(&self.arena, mouse);
        self.cancel
            .draw(frame, atlas, font, &self.arena, highlighted);
        centred(
            frame,
            atlas,
            font,
            &self.status,
            (self.size.0 / 2, self.size.1 / 2 - 50),
        );
    }

    /// Whether a press of the primary button at `mouse` cancels.
    pub fn cancelled(&self, mouse: (i32, i32)) -> bool {
        self.cancel.contains(&self.arena, mouse)
    }
}

/// `LevelLoadingScreen` of a server's level (`Reason.OTHER`, no progress, no chunk map).
pub fn draw_level_loading(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    font: &Font,
    lang: &Lang,
    size: (i32, i32),
) {
    menu_background(frame, atlas, size);
    let text = lang.runs(
        &Component::translatable("multiplayer.downloadingTerrain"),
        &Style::default(),
    );
    centred(frame, atlas, font, &text, (size.0 / 2, size.1 / 2 - 50));
}
