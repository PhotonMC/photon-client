//! Photon's own widgets (panel, button, sidebar row, card), drawn as flat quads from a design
//! skin. Corners are square for now, which is what the Classic skin wants; the light and dark
//! skins draw the same shapes until the renderer has a rounded primitive.

pub mod grid;

use crate::layout::Rect;
use crate::tokens::Skin;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// How a widget is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Normal,
    Hover,
    Selected,
    Disabled,
}

/// A frame, the atlas and font it draws with, and the skin whose tokens it reads.
pub struct Painter<'a> {
    pub frame: &'a mut GuiFrame,
    pub atlas: &'a GuiAtlas,
    pub font: &'a Font,
    pub skin: &'a Skin,
}

/// Design tokens are in CSS pixels; the GUI counts in units of two pixels, at least one.
pub fn unit(pixels: u32) -> i32 {
    (pixels / 2).max(1) as i32
}

/// Whether a point is inside a rectangle.
pub fn inside(rect: Rect, (x, y): (i32, i32)) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// `rect` moved in by `by` units on every side.
pub fn shrink(rect: Rect, by: i32) -> Rect {
    Rect {
        x: rect.x + by,
        y: rect.y + by,
        width: (rect.width - 2 * by).max(0),
        height: (rect.height - 2 * by).max(0),
    }
}

impl Painter<'_> {
    /// A flat rectangle in one colour.
    pub fn fill_rect(&mut self, rect: Rect, colour: u32) {
        let (x1, y1) = (rect.x + rect.width, rect.y + rect.height);
        self.frame
            .fill(self.atlas, (rect.x, rect.y, x1, y1), colour);
    }

    /// Text at a place, with the skin's text shadow under it when it has one.
    pub fn label(&mut self, text: &str, (x, y): (i32, i32), colour: u32) {
        if self.skin.text_shadow >> 24 != 0 {
            self.frame.text(
                self.atlas,
                self.font,
                text,
                x + 1,
                y + 1,
                self.skin.text_shadow,
            );
        }
        self.frame.text(self.atlas, self.font, text, x, y, colour);
    }

    /// Text in the middle of a rectangle.
    pub fn centred(&mut self, text: &str, rect: Rect, colour: u32) {
        let x = rect.x + (rect.width - self.font.width(text)) / 2;
        let y = rect.y + (rect.height - 8) / 2;
        self.label(text, (x, y), colour);
    }

    /// A surface with a one-unit edge in the skin's line colour.
    pub fn panel(&mut self, rect: Rect) {
        let edge = unit(self.skin.border);
        self.fill_rect(rect, self.skin.line);
        self.fill_rect(shrink(rect, edge), self.skin.card);
    }

    /// The button body: its border, face and bevel.
    fn plate(&mut self, rect: Rect, state: State) {
        let (skin, edge) = (self.skin, unit(self.skin.border));
        let border = match state {
            State::Hover | State::Selected => skin.button_border_hover,
            _ => skin.button_border,
        };
        self.fill_rect(rect, border);
        let face = shrink(rect, edge);
        self.fill_rect(face, skin.button);
        let light = Rect {
            height: edge,
            ..face
        };
        let left = Rect {
            width: edge,
            ..face
        };
        self.fill_rect(light, skin.bevel_light);
        self.fill_rect(left, skin.bevel_light);
        let foot = Rect {
            y: face.y + face.height - 2 * edge,
            height: 2 * edge,
            ..face
        };
        let side = Rect {
            x: face.x + face.width - edge,
            width: edge,
            ..face
        };
        self.fill_rect(foot, skin.bevel_dark);
        self.fill_rect(side, skin.bevel_dark);
        if state == State::Hover {
            self.fill_rect(face, skin.hover);
        }
    }

    /// A pressable button with a centred label.
    pub fn button(&mut self, rect: Rect, label: &str, state: State) {
        self.plate(rect, state);
        let ink = if state == State::Disabled {
            self.skin.muted
        } else {
            self.skin.button_ink
        };
        self.centred(label, rect, ink);
    }

    /// One row of the sidebar: the selected row has the accent as its face.
    pub fn row(&mut self, rect: Rect, label: &str, state: State) {
        let skin = self.skin;
        let (face, ink) = match state {
            State::Selected => (skin.accent, skin.accent_ink),
            State::Disabled => (0, skin.muted),
            _ => (0, skin.ink),
        };
        if face != 0 {
            self.fill_rect(rect, face);
        } else if state == State::Hover {
            self.fill_rect(rect, skin.hover);
        }
        self.label(label, (rect.x + 6, rect.y + (rect.height - 8) / 2), ink);
    }

    /// A game card: a coloured banner, a title and a detail line.
    pub fn card(&mut self, rect: Rect, (title, detail): (&str, &str), state: State) {
        self.plate(rect, state);
        let skin = self.skin;
        let inner = shrink(rect, unit(skin.border) + 1);
        let banner = Rect {
            height: inner.height * 3 / 5,
            ..inner
        };
        self.fill_rect(banner, skin.bg | 0xFF00_0000);
        let text_y = banner.y + banner.height + 3;
        self.label(title, (inner.x + 2, text_y), skin.button_ink);
        self.label(detail, (inner.x + 2, text_y + 10), skin.muted);
    }
}
