//! The friends overlay's drawing.

use super::*;
use crate::{FontRuns, FrameRuns};
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `LoadingDotsText.FRAMES`, three tenths of a second each.
fn loading_dots(millis: u64) -> &'static str {
    const FRAMES: [&str; 4] = ["O o o", "o O o", "o o O", "o O o"];
    FRAMES[(millis / 300 % 4) as usize]
}

impl FriendsOverlay {
    /// Where the lists are: under the box to add a friend.
    fn list_rect(&self) -> (i32, i32, i32, i32) {
        let (left, top, width, height) = self.content();
        (
            left,
            top + ADD_WIDGET_HEIGHT,
            width,
            height - ADD_WIDGET_HEIGHT,
        )
    }

    /// The tab buttons, side by side above the panel.
    pub(super) fn tab_rect(&self, tab: Tab) -> (i32, i32, i32, i32) {
        let (left, top, ..) = self.content();
        let index = i32::from(tab == Tab::Pending);
        (
            left + index * TAB_WIDTH,
            top - TAB_HEIGHT - 7,
            TAB_WIDTH,
            TAB_HEIGHT,
        )
    }

    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
        (blurriness, millis): (i32, u64),
    ) {
        let (left, top, width, height) = self.content();
        frame.blur_before_this_stratum(blurriness);
        frame.tile_texture(
            atlas,
            "inworld_menu_background",
            (32, 32),
            (0, 0, self.size.0, self.size.1),
        );
        frame.nine_slice(
            atlas,
            "friends/background",
            (236, 34),
            BORDER,
            false,
            (
                left - BORDER,
                top - BORDER,
                width + 2 * BORDER,
                height + 2 * BORDER + 1,
            ),
        );
        for tab in [Tab::Friends, Tab::Pending] {
            self.draw_tab(frame, (atlas, font), mouse, tab);
        }
        self.draw_add_widget(frame, (atlas, font), mouse);
        let (view, texts) = (&self.view, &self.texts);
        let (listing, loading) = match self.tab {
            Tab::Friends => (&view.friends, &texts.loading[0]),
            Tab::Pending => (&view.pending, &texts.loading[1]),
        };
        let area = self.list_rect();
        match listing {
            Listing::Loading => draw_loading(frame, (atlas, font), (area, loading), millis),
            Listing::Error(message) => {
                self.draw_centered(frame, (atlas, font), area, (message, false));
            }
            Listing::Empty => {
                let runs = match self.tab {
                    Tab::Friends => &texts.empty,
                    Tab::Pending => &texts.pending_empty,
                };
                self.draw_centered(frame, (atlas, font), area, (runs, self.tab == Tab::Friends));
            }
            Listing::Entries(_) => self.draw_entries(frame, (atlas, font), mouse),
        }
        self.draw_tooltip(frame, (atlas, font), mouse);
    }

    fn draw_tab(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
        tab: Tab,
    ) {
        let (x, y, width, height) = self.tab_rect(tab);
        let selected = self.tab == tab;
        let hovered = inside(mouse, (x, y, width, height));
        let sprite = match (selected, hovered) {
            (_, true) => "friends/button_highlighted",
            (true, false) => "friends/button",
            (false, false) => "friends/button_disabled",
        };
        frame.nine_slice(atlas, sprite, (200, 20), 3, false, (x, y, width, height));
        let label = match tab {
            Tab::Friends => &self.texts.friends_tab,
            Tab::Pending => &self.pending_label,
        };
        let text_width = font.width_of_runs(label);
        let top = y + i32::from(!selected);
        let (inner_left, inner_right) = (x + 1, x + width - 1);
        let text_x = inner_left + (inner_right - inner_left - text_width) / 2;
        frame.styled_text(
            atlas,
            font,
            label,
            (text_x, (top + y + height - 9) / 2),
            WHITE,
            true,
        );
        if selected {
            let underline = text_width.min(width - 4);
            let from = x + (width - underline) / 2;
            frame.fill(
                atlas,
                (from, y + height - 2, from + underline, y + height - 1),
                WHITE,
            );
        }
    }

    fn draw_add_widget(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
    ) {
        let (bx, by, bw, bh) = self.box_rect();
        let focused = self.focus == Focus::Box && self.box_active();
        let sprite = if focused {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        frame.nine_slice(atlas, sprite, (200, 20), 1, false, (bx, by, bw, bh));
        let text_at = (bx + 4, by + (bh - 8) / 2);
        self.edit.draw_formatted(
            frame,
            atlas,
            font,
            text_at,
            focused.then_some(self.focused_at),
            None,
        );
        if self.edit.is_empty() && !focused {
            frame.styled_text(atlas, font, &self.texts.hint, text_at, WHITE, true);
        }
        let (sx, sy, sw, sh) = self.button_rect();
        let state = if !self.button_active() {
            "widget/button_disabled"
        } else if inside(mouse, (sx, sy, sw, sh)) {
            "widget/button_highlighted"
        } else {
            "widget/button"
        };
        frame.nine_slice(atlas, state, (200, 20), 1, false, (sx, sy, sw, sh));
        frame.sprite(atlas, "friends/send_request", sx + 2, sy + 2, 15, 15);
        self.draw_profile_row(frame, (atlas, font));
    }

    /// The row under the box: the profile name label and the name, then the separator.
    fn draw_profile_row(&self, frame: &mut GuiFrame, (atlas, font): (&GuiAtlas, &Font)) {
        let (left, top, width, _) = self.content();
        let row_y = top + INPUT_TOP + BOX_HEIGHT + PROFILE_TOP;
        let label_width = font.width_of_runs(&self.texts.profile_label);
        frame.styled_text(
            atlas,
            font,
            &self.texts.profile_label,
            (left + INPUT_LEFT, row_y),
            LABEL_COLOUR,
            true,
        );
        frame.text_with_shadow(
            atlas,
            font,
            &self.view.profile_name,
            left + INPUT_LEFT + label_width,
            row_y,
            WHITE,
        );
        frame.sprite(
            atlas,
            "friends/list_separator_top",
            left,
            row_y + 9 + 4,
            width,
            2,
        );
    }

    /// The lines of a text, centred in the room of the list, with the picture over them.
    fn draw_centered(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        area: (i32, i32, i32, i32),
        (runs, picture): (&[Run], bool),
    ) {
        let block = self.centred_block(font, area, runs, picture);
        if let Some((x, y)) = block.picture {
            frame.sprite(atlas, "friends/illustrations_00", x, y, 128, 48);
        }
        for (x, y, line) in &block.lines {
            frame.styled_text(atlas, font, line, (*x, *y), WHITE, true);
        }
    }
}

fn draw_loading(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    ((x, y, width, height), message): ((i32, i32, i32, i32), &[Run]),
    millis: u64,
) {
    let text_width = font.width_of_runs(message);
    let centre = x + width / 2;
    // `LoadingDotsWidget` is 28 tall: 2, the text, 6, the dots and 2.
    let top = y + (height - 28) / 2;
    frame.styled_text(
        atlas,
        font,
        message,
        (centre - text_width / 2, top + 2),
        WHITE,
        true,
    );
    let dots = loading_dots(millis);
    frame.text_with_shadow(
        atlas,
        font,
        dots,
        centre - font.width(dots) / 2,
        top + 28 - 9 - 2,
        DOTS_COLOUR,
    );
}

pub(super) fn inside((x, y): (i32, i32), (left, top, width, height): (i32, i32, i32, i32)) -> bool {
    x >= left && x < left + width && y >= top && y < top + height
}
