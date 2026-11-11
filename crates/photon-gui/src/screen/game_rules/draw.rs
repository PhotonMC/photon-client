//! The game rules screen's drawing (`AbstractGameRulesScreen`, `RuleList`, the entries'
//! `extractContent`, `LoadingDotsWidget`).

use crate::{FontRuns, FrameRuns};

use super::*;
use crate::screen::stats::loading_dots;
use crate::widgets::navigation::Rectangle;
use crate::widgets::scroll_area::SCROLLBAR_WIDTH;
use photon_render::gui::{GuiAtlas, GuiFrame};

impl GameRulesScreen {
    /// The blurred world, the menu background over it, then the title, the search box, the list
    /// (or the loading text) and the footer. `under` draws what the screen's own stratum holds
    /// below the background (the debug overlay).
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
        if let Some(loading) = &self.loading {
            self.draw_loading(frame, atlas, font, loading);
        }
        for (widget, focus) in [(&self.done, Focus::Done), (&self.cancel, Focus::Cancel)] {
            let lit = widget.contains(&self.arena, mouse) || self.focus == Some(focus);
            widget.draw(frame, atlas, font, &self.arena, lit);
        }
        if self.loading.is_none() {
            self.draw_list(frame, atlas, font, mouse);
        }
    }

    /// `LoadingDotsWidget.extractWidgetRenderState`: the text and, under it, the dots.
    fn draw_loading(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, loading: &Loading) {
        let rect = self.arena.0[loading.rect];
        let centre = rect.x + rect.width / 2;
        let text_x = centre - font.width_of_runs(&loading.text) / 2;
        frame.styled_text(
            atlas,
            font,
            &loading.text,
            (text_x, rect.y + 2),
            0xFFFF_FFFF,
            true,
        );
        let dots = loading_dots(frame.millis);
        let at = (centre - font.width(dots) / 2, rect.y + rect.height - 9 - 2);
        frame.text_with_shadow(atlas, font, dots, at.0, at.1, 0xFF80_8080);
    }

    /// `EditBox.extractWidgetRenderState`: the field, lit while it has the focus or the mouse,
    /// its text, and its hint while it is empty and without the focus.
    fn draw_search(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let rect = self.arena.0[self.search_rect];
        let focused = self.focus == Some(Focus::Search);
        let lit = focused || self.search_contains(mouse);
        self.draw_field(frame, atlas, rect, lit);
        let text_at = (rect.x + 4, rect.y + (rect.height - 8) / 2);
        let since = focused.then_some(self.focused_at);
        self.search
            .draw_formatted(frame, atlas, font, text_at, since, None);
        if self.search.is_empty() && !focused {
            let hint = &self.search_hint;
            frame.styled_text(atlas, font, hint, text_at, 0xFFE0_E0E0, true);
        }
    }

    fn draw_field(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        rect: crate::layout::Rect,
        lit: bool,
    ) {
        let sprite = if lit {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        let area = (rect.x, rect.y, rect.width, rect.height);
        frame.nine_slice(atlas, sprite, (200, 20), 1, false, area);
    }

    /// The list: its background, its rows cut to its window, the lines above and below it, its
    /// scrollbar, and the tooltip of the row under the mouse.
    fn draw_list(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let place = self.place;
        place.draw_background(frame, atlas, self.scroll);
        let inside = place.contains((f64::from(mouse.0), f64::from(mouse.1)));
        let mark = frame.mark();
        for (index, row) in self.rows.iter().enumerate() {
            let y = self.row_y(index);
            if y + ITEM_HEIGHT < place.y || y > place.bottom() {
                continue;
            }
            self.draw_row(
                frame,
                (atlas, font),
                (index, row),
                (self.row_left() + 2, y + 2),
                mouse,
            );
        }
        frame.clip_from(mark, (0, place.y, place.width, place.bottom()));
        let bar_x = self.row_left() + ROW_WIDTH + SCROLLBAR_WIDTH + 2;
        place.draw_edges(frame, atlas, (self.scroll, self.content_height()), bar_x);
        let hovered = self.row_at(mouse).filter(|_| inside);
        if let Some(Row::Rule { entry, .. }) = hovered.map(|index| &self.rows[index]) {
            let lines = &self.entries[*entry].tooltip;
            frame.tooltip(atlas, font, lines, mouse, self.size);
        }
    }

    /// A row: a category's label centred, or a rule's name and its control.
    fn draw_row(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        (index, row): (usize, &Row),
        (x, y): (i32, i32),
        mouse: (i32, i32),
    ) {
        let white = 0xFFFF_FFFF;
        match row {
            Row::Category(label, width) => {
                let at = x + (ROW_WIDTH - 4) / 2 - width / 2;
                if self.focus == Some(Focus::Category(index)) {
                    // `FocusableTextWidget`: black from its second column, outlined in white.
                    let rect = self.label_rect(index, *width);
                    let (right, bottom) = (rect.x + rect.width, rect.y + rect.height);
                    frame.fill(atlas, (rect.x + 1, rect.y, right, bottom), 0xFF00_0000);
                    let outline = Rectangle {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    };
                    crate::screen::dialog_body::outline(frame, atlas, outline);
                }
                // 26.3 centres the label in a focusable widget; 26.2 draws plain text five down.
                let top = if photon_core::versions::behaviour::debug_option_categories_styled() {
                    6
                } else {
                    5
                };
                frame.styled_text(atlas, font, label, (at, y + top), white, true);
            }
            Row::Rule { entry, control, .. } => {
                let lines = &self.entries[*entry].lines;
                if let [only] = lines.as_slice() {
                    frame.styled_text(atlas, font, only, (x, y + 5), white, true);
                } else if lines.len() >= 2 {
                    frame.styled_text(atlas, font, &lines[0], (x, y), white, true);
                    frame.styled_text(atlas, font, &lines[1], (x, y + 10), white, true);
                }
                let focused = self.focus == Some(Focus::Control(*entry));
                self.draw_control(frame, (atlas, font), (control, focused), mouse);
            }
        }
    }

    fn draw_control(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        (control, focused): (&Control, bool),
        mouse: (i32, i32),
    ) {
        match control {
            Control::Toggle(widget) => {
                let lit = widget.contains(&self.arena, mouse) || focused;
                widget.draw(frame, atlas, font, &self.arena, lit);
            }
            Control::Input { rect, edit, .. } => {
                let rect = self.arena.0[*rect];
                let inside = mouse.0 >= rect.x
                    && mouse.1 >= rect.y
                    && mouse.0 < rect.x + rect.width
                    && mouse.1 < rect.y + rect.height;
                self.draw_field(frame, atlas, rect, focused || inside);
                let text_at = (rect.x + 4, rect.y + (rect.height - 8) / 2);
                let since = focused.then_some(self.focused_at);
                edit.draw_formatted(frame, atlas, font, text_at, since, None);
            }
        }
    }
}
