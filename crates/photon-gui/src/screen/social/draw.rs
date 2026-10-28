//! The social interactions screen's drawing (`SocialInteractionsScreen.extractRenderState`,
//! `PlayerEntry.extractContent`).

use crate::{FontRuns, FrameRuns};
use photon_game::text::{Run, Style};

use super::*;

impl SocialInteractionsScreen {
    /// `extractBackground` in a game: the blurred world behind, dimmed, then the window.
    fn draw_background(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, blurriness: i32) {
        let (width, height) = self.size;
        frame.blur_before_this_stratum(blurriness);
        frame.tile_texture(
            atlas,
            "inworld_menu_background",
            (32, 32),
            (0, 0, width, height),
        );
        let window_x = self.margin_x() + 3;
        frame.sprite(
            atlas,
            "social_interactions/background",
            window_x,
            64,
            BACKGROUND_WIDTH,
            self.window_height() + 16,
        );
        frame.sprite(atlas, "icon/search", window_x + 10, 76, 12, 12);
    }

    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
        blurriness: i32,
    ) {
        self.draw_background(frame, atlas, blurriness);
        self.buttons.draw(frame, atlas, font, mouse);
        self.title
            .draw(frame, atlas, font, &self.buttons.arena, false);
        let (field_x, field_y) = (self.margin_x() + 28, 74);
        frame.nine_slice(
            atlas,
            if self.focus == Focus::Search {
                "widget/text_field_highlighted"
            } else {
                "widget/text_field"
            },
            (200, 20),
            1,
            false,
            (field_x, field_y, 200, 15),
        );
        let text_at = (field_x + 4, field_y + (15 - 8) / 2);
        let focused = self.focus == Focus::Search;
        self.search.draw_formatted(
            frame,
            atlas,
            font,
            text_at,
            focused.then_some(self.focused_at),
            None,
        );
        if self.search.is_empty() && !focused {
            frame.styled_text(
                atlas,
                font,
                &self.search_hint,
                text_at.0,
                text_at.1,
                0xFFFF_FFFF,
                true,
            );
        }
        if self.hint_visible.get() {
            let highlighted = self.focus == Focus::Hint
                || self.blocking_hint.contains(&self.buttons.arena, mouse);
            self.blocking_hint
                .draw(frame, atlas, font, &self.buttons.arena, highlighted);
        }
        if !self.server_label.is_empty() {
            frame.styled_text(
                atlas,
                font,
                &self.server_label,
                self.margin_x() + 8,
                35,
                0xFFFF_FFFF,
                true,
            );
        }
        if self.entries.is_empty() {
            let empty = if self.search.is_empty() {
                match self.page {
                    Page::All => None,
                    Page::Hidden => Some(&self.empty[1]),
                    Page::Blocked => Some(&self.empty[2]),
                }
            } else {
                Some(&self.empty[0])
            };
            if let Some(text) = empty {
                let x = self.size.0 / 2 - font.width_of_runs(text) / 2;
                frame.styled_text(
                    atlas,
                    font,
                    text,
                    x,
                    (72 + self.list_end()) / 2,
                    0xFFFF_FFFF,
                    true,
                );
            }
        } else {
            self.draw_list(frame, atlas, font, mouse);
        }
        self.hint_visible.set(self.page == Page::Blocked);
    }

    /// The list: its rows, cut to the list's window, and its scrollbar.
    fn draw_list(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let place = self.place;
        let left = self.row_left();
        let mark = frame.mark();
        let window = (0, place.y + 4, place.width, place.bottom());
        let in_window = mouse.1 >= window.1 && mouse.1 < window.3;
        for (index, entry) in self.entries.iter().enumerate() {
            let y = self.entry_y(index);
            if y + ENTRY_HEIGHT < place.y || y > place.bottom() {
                continue;
            }
            self.draw_entry(
                frame,
                atlas,
                font,
                entry,
                (left + 2, y + 2),
                (mouse, in_window),
            );
        }
        frame.clip_from(mark, window);
        place.draw_scrollbar(
            frame,
            atlas,
            left + ROW_WIDTH + SCROLLBAR_WIDTH + 2,
            self.scroll,
            self.content_height(),
        );
    }

    /// `PlayerEntry.extractContent`: the row, the face, the name and status, and the buttons.
    fn draw_entry(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        entry: &Entry,
        (x, y): (i32, i32),
        (mouse, in_window): ((i32, i32), bool),
    ) {
        let (content_width, content_height) = (ROW_WIDTH - 4, ENTRY_HEIGHT - 4);
        let skin_x = x + 4;
        let skin_y = y + (content_height - SKIN_SIZE) / 2;
        let text_x = skin_x + SKIN_SIZE + 4;
        let hidden = self.hidden(entry.player.id);
        let status = &self.statuses[status_index(hidden, entry.removed)];
        let rect = (x, y, x + content_width, y + content_height);
        let text_y = if status.is_empty() {
            frame.fill(atlas, rect, BG_FILL);
            y + (content_height - 9) / 2
        } else {
            frame.fill(atlas, rect, BG_FILL_REMOVED);
            let text_y = y + (content_height - 18) / 2;
            frame.styled_text(
                atlas,
                font,
                status,
                text_x,
                text_y + 12,
                PLAYER_STATUS_COLOUR,
                true,
            );
            text_y
        };
        let texture = format!("textures/{}", default_skin(entry.player.id).texture);
        for u in [8, 40] {
            frame.blit_texture_stretched(
                atlas,
                &texture,
                (64, 64),
                (u, 8, 8, 8),
                (skin_x, skin_y, SKIN_SIZE, SKIN_SIZE),
            );
        }
        let name = [Run {
            text: entry.player.name.clone(),
            style: Style::default(),
        }];
        frame.styled_text(atlas, font, &name, text_x, text_y, 0xFFFF_FFFF, true);
        if entry.removed {
            frame.fill(
                atlas,
                (skin_x, skin_y, skin_x + SKIN_SIZE, skin_y + SKIN_SIZE),
                SKIN_SHADE,
            );
        }
        if entry.player.id == self.roster.local {
            return;
        }
        let button_y = y + (content_height - 20) / 2;
        let report_x = x + content_width - 24;
        let hide_x = report_x - 24;
        let over = |button_x: i32| {
            in_window
                && mouse.0 >= button_x
                && mouse.0 < button_x + 20
                && mouse.1 >= button_y
                && mouse.1 < button_y + 20
        };
        let (hide_sprite, hide_lit) = if hidden {
            ("unmute_button", "unmute_button_highlighted")
        } else {
            ("mute_button", "mute_button_highlighted")
        };
        let sprite = if over(hide_x) { hide_lit } else { hide_sprite };
        frame.sprite(
            atlas,
            &format!("social_interactions/{sprite}"),
            hide_x,
            button_y,
            20,
            20,
        );
        let report = match (self.roster.reporting, over(report_x)) {
            (false, _) => "report_button_disabled",
            (true, true) => "report_button_highlighted",
            (true, false) => "report_button",
        };
        frame.sprite(
            atlas,
            &format!("social_interactions/{report}"),
            report_x,
            button_y,
            20,
            20,
        );
        if entry.player.draft_report {
            frame.sprite(
                atlas,
                "icon/draft_report",
                report_x + 5,
                button_y + 1,
                15,
                15,
            );
        }
    }
}
