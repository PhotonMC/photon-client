//! The screen a lost connection leaves (`DisconnectedScreen`): the title,
//! the reason wrapped and centred, and a button back.

use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::chat::split;
use crate::gui::font::Font;
use crate::gui::layout::{Arena, Grid, Item, Settings};
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

pub struct DisconnectedScreen {
    arena: Arena,
    title: Widget,
    /// The reason's lines, and the place of the box they stand in.
    reason: Vec<Vec<Run>>,
    reason_rect: usize,
    button: Widget,
    size: (i32, i32),
    focused: bool,
    /// What it was made for, to make it again for another size.
    cause: Component,
    heading: Component,
}

impl DisconnectedScreen {
    /// `DisconnectedScreen.init` for a reason, in a screen of `width` by
    /// `height` units.
    pub fn new(font: &Font, lang: &Lang, size: (i32, i32), reason: &Component) -> Self {
        Self::titled(
            font,
            lang,
            size,
            &Component::translatable("disconnect.lost"),
            reason,
        )
    }

    /// The screen with another title (`connect.failed` for a connection that never came).
    pub fn titled(
        font: &Font,
        lang: &Lang,
        (width, height): (i32, i32),
        heading: &Component,
        reason: &Component,
    ) -> Self {
        let mut arena = Arena::default();
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let title_text = runs(heading);
        let title_rect = arena.add(font.width_of_runs(&title_text), 9);
        let reason_runs = runs(reason);
        // `splitLines` of no text is no line.
        let lines = if reason_runs.iter().all(|run| run.text.is_empty()) {
            Vec::new()
        } else {
            split(&reason_runs, width - 50, font)
        };
        let widest = lines
            .iter()
            .map(|line| font.width_of_runs(line))
            .max()
            .unwrap_or(0);
        let reason_rect = arena.add((width - 50).min(widest), lines.len() as i32 * 9);
        let button_rect = arena.add(200, 20);
        let mut grid = Grid::new();
        let wide = Settings::default()
            .padding(10, 10, 10, 10)
            .align_horizontally(0.5);
        let narrow = Settings::default()
            .padding(2, 2, 2, 2)
            .align_horizontally(0.5);
        grid.add(Item::Leaf(title_rect), (0, 0), (1, 1), wide);
        grid.add(Item::Leaf(reason_rect), (1, 0), (1, 1), wide);
        grid.add(Item::Leaf(button_rect), (2, 0), (1, 1), narrow);
        grid.arrange(&mut arena);
        grid.align_in_rectangle(&mut arena, (0, 0, width, height), (0.5, 0.5));
        Self {
            title: Widget {
                rect: title_rect,
                active: false,
                kind: Kind::Text { text: title_text },
            },
            reason: lines,
            reason_rect,
            button: Widget {
                rect: button_rect,
                active: true,
                kind: Kind::Button {
                    label: runs(&Component::translatable("gui.toMenu")),
                },
            },
            arena,
            size: (width, height),
            focused: false,
            cause: reason.clone(),
            heading: heading.clone(),
        }
    }

    /// The screen again for a screen of another size.
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        Self::titled(font, lang, size, &self.heading, &self.cause)
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The menu background, the title, the reason and the button.
    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        frame.tile_texture(atlas, "menu_background", (32, 32), (0, 0, width, height));
        let highlighted = self.button.contains(&self.arena, mouse) || self.focused;
        self.button
            .draw(frame, atlas, font, &self.arena, highlighted);
        self.title.draw(frame, atlas, font, &self.arena, false);
        let rect = self.arena.0[self.reason_rect];
        let middle = rect.x + rect.width / 2;
        let mut y = rect.y;
        for line in &self.reason {
            let x = middle - font.width_of_runs(line) / 2;
            frame.styled_text(atlas, font, line, x, y, 0xFFFF_FFFF, true);
            y += 9;
        }
    }

    /// A press of the primary button: whether it was on the button.
    pub fn clicked(&self, mouse: (i32, i32)) -> bool {
        self.button.contains(&self.arena, mouse)
    }

    /// Tab focuses the button, the only one.
    pub fn tab(&mut self) {
        self.focused = true;
    }

    /// Enter or Space on the focused button.
    pub fn pressed(&self) -> bool {
        self.focused
    }
}
