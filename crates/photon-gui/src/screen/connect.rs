//! The screens of a join: `ConnectScreen` (what the connection is doing, and Cancel) and, once
//! the level has begun, `LevelLoadingScreen` ("Loading terrain..."; a server shows no progress).

use crate::{FontRuns, FrameRuns};
use photon_game::text::{Component, Lang, Run, Style};

use crate::layout::Arena;
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

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

/// `ServerReconfigScreen`: the title and a Disconnect button that stays inactive for the first
/// 600 ticks, centred in the window, over the menu background.
pub struct ReconfigScreen {
    arena: Arena,
    disconnect: Widget,
    title: Vec<Run>,
    title_at: (i32, i32),
    size: (i32, i32),
}

impl ReconfigScreen {
    /// The ticks after which the button works (`DISCONNECT_TIME`).
    pub const DISCONNECT_TIME: u32 = 600;

    /// `init` and `repositionElements`: a vertical layout of the title and the button, each with
    /// ten units around it, centred in the screen.
    pub fn new(lang: &Lang, font: &Font, size: (i32, i32)) -> Self {
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let title = runs("connect.reconfiguring");
        let title_width = font.width_of_runs(&title);
        // Cells of 10 units around the title (9 high) and the 150 by 20 button.
        let (layout_width, layout_height) = ((title_width + 20).max(170), 29 + 40);
        let left = (size.0 - layout_width) / 2;
        let top = (size.1 - layout_height) / 2;
        let mut arena = Arena::default();
        let rect = arena.add(150, 20);
        arena.0[rect].x = left + (layout_width - 150) / 2;
        arena.0[rect].y = top + 29 + 10;
        Self {
            arena,
            disconnect: Widget {
                rect,
                active: false,
                kind: Kind::Button {
                    label: runs("menu.disconnect"),
                },
            },
            title,
            title_at: (left + (layout_width - title_width) / 2, top + 10),
            size,
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// `tick`: the button works once the delay has passed.
    pub fn set_ticks(&mut self, ticks: u32) {
        self.disconnect.active = ticks >= Self::DISCONNECT_TIME;
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        menu_background(frame, atlas, self.size);
        frame.styled_text(
            atlas,
            font,
            &self.title,
            self.title_at.0,
            self.title_at.1,
            0xFFFF_FFFF,
            true,
        );
        let highlighted = self.disconnect.active && self.disconnect.contains(&self.arena, mouse);
        self.disconnect
            .draw(frame, atlas, font, &self.arena, highlighted);
    }

    /// Whether a press of the primary button at `mouse` disconnects.
    pub fn disconnected(&self, mouse: (i32, i32)) -> bool {
        self.disconnect.active && self.disconnect.contains(&self.arena, mouse)
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
