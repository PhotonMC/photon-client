//! The debug options screen's drawing (`DebugOptionsScreen.extractRenderState`,
//! `OptionEntry.extractContent`, `CategoryEntry.extractContent`).

use crate::{FontRuns, FrameRuns};

use super::*;
use photon_render::gui::{GuiAtlas, GuiFrame};

impl DebugOptionsScreen {
    /// The blurred world, the menu background over it, then the widgets in the order they were
    /// added: the title, the search box, the warning, the list and the footer. `under` draws what
    /// `extractBlurredBackground` puts in the screen's own stratum (the debug overlay), which the
    /// blur does not cover.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
        (blurriness, under): (i32, impl FnOnce(&mut GuiFrame)),
    ) {
        let (width, height) = self.size;
        frame.blur_before_this_stratum(blurriness);
        under(frame);
        frame.tile_texture(
            atlas,
            "inworld_menu_background",
            (32, 32),
            (0, 0, width, height),
        );
        self.title.draw(frame, atlas, font, &self.arena, false);
        self.draw_search(frame, atlas, font, mouse);
        let rect = self.arena.0[self.subtitle_rect];
        let middle = rect.x + rect.width / 2;
        for (line, y) in self.subtitle.iter().zip((rect.y..).step_by(9)) {
            let x = middle - font.width_of_runs(line) / 2;
            frame.styled_text(atlas, font, line, x, y, 0xFFFF_FFFF, true);
        }
        self.draw_list(frame, atlas, font, mouse);
        let footer = [
            (&self.profiles[0], Focus::Profile(0)),
            (&self.profiles[1], Focus::Profile(1)),
            (&self.done, Focus::Done),
        ];
        for (widget, focus) in footer {
            let lit = widget.contains(&self.arena, mouse) || self.focus == focus;
            widget.draw(frame, atlas, font, &self.arena, lit);
        }
        self.draw_not_allowed_tooltip(frame, atlas, font, mouse);
    }

    /// `OptionEntry.extractContent`'s `setTooltipForNextFrame`, drawn with the deferred elements:
    /// the mouse over the name side of a row whose entry reduced debug info hides.
    fn draw_not_allowed_tooltip(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
    ) {
        let over_list = self
            .place
            .contains((f64::from(mouse.0), f64::from(mouse.1)));
        let Some(index) = self.row_at(mouse).filter(|_| over_list) else {
            return;
        };
        let buttons_start = self.row_left() + 2 + (ROW_WIDTH - 4) - 3 * BUTTON_WIDTH;
        if matches!(self.rows[index], Row::Entry { allowed: false, .. }) && mouse.0 < buttons_start
        {
            frame.tooltip(
                atlas,
                font,
                std::slice::from_ref(&self.not_allowed),
                mouse,
                self.size,
            );
        }
    }

    /// `EditBox.extractWidgetRenderState`: the field, lit while it has the focus or the mouse,
    /// its text, and its hint while it is empty and without the focus.
    fn draw_search(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let rect = self.arena.0[self.search_rect];
        let focused = self.focus == Focus::Search;
        let lit = focused || self.search_contains(mouse);
        frame.nine_slice(
            atlas,
            if lit {
                "widget/text_field_highlighted"
            } else {
                "widget/text_field"
            },
            (200, 20),
            1,
            false,
            (rect.x, rect.y, rect.width, rect.height),
        );
        let text_at = (rect.x + 4, rect.y + (rect.height - 8) / 2);
        let focused = focused.then_some(self.focused_at);
        self.search
            .draw_formatted(frame, atlas, font, text_at, focused, None);
        if self.search.is_empty() && self.focus != Focus::Search {
            let hint = &self.search_hint;
            frame.styled_text(atlas, font, hint, text_at.0, text_at.1, 0xFFE0_E0E0, true);
        }
    }

    /// The list: its background, its rows cut to its window, the lines above and below it and
    /// its scrollbar.
    fn draw_list(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let place = self.place;
        place.draw_background(frame, atlas, self.scroll);
        let inside = place.contains((f64::from(mouse.0), f64::from(mouse.1)));
        let mouse = if inside { mouse } else { (i32::MIN, i32::MIN) };
        let mark = frame.mark();
        for (index, row) in self.rows.iter().enumerate() {
            let y = self.row_y(index);
            if y + ITEM_HEIGHT < place.y || y > place.bottom() {
                continue;
            }
            self.draw_row(frame, atlas, font, row, (self.row_left() + 2, y + 2));
            if let Row::Entry { id, buttons, .. } = row {
                self.draw_buttons(frame, atlas, font, (id, buttons), mouse);
            }
        }
        frame.clip_from(mark, (0, place.y, place.width, place.bottom()));
        let bar_x = self.row_left() + ROW_WIDTH + crate::widgets::scroll_area::SCROLLBAR_WIDTH + 2;
        place.draw_edges(frame, atlas, (self.scroll, self.content_height()), bar_x);
    }

    /// A row's label: a group's centred, an entry's name at the left.
    fn draw_row(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        row: &Row,
        (x, y): (i32, i32),
    ) {
        let content_width = ROW_WIDTH - 4;
        match row {
            Row::Category(text) => {
                let at = x + content_width / 2 - font.width_of_runs(text) / 2;
                frame.styled_text(atlas, font, text, at, y + 5, 0xFFFF_FFFF, true);
            }
            Row::Entry { name, allowed, .. } => {
                let colour = if *allowed {
                    0xFFFF_FFFF
                } else {
                    DISALLOWED_COLOUR
                };
                frame.styled_text(atlas, font, name, x, y + 5, colour, true);
            }
        }
    }

    /// An entry's three buttons, against the row's right edge, the last to draw the first.
    fn draw_buttons(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        (id, buttons): (&'static str, &[Widget; 3]),
        mouse: (i32, i32),
    ) {
        for (slot, button) in buttons.iter().enumerate().rev() {
            let lit = button.contains(&self.arena, mouse) || self.focus == Focus::Button(id, slot);
            button.draw(frame, atlas, font, &self.arena, lit);
        }
    }
}
