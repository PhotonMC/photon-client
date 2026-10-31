//! The entries of the friends overlay's lists: their layout, drawing and buttons
//! (`FriendEntry`, `IncomingEntry`, `OutgoingEntry`).
//!
//! Not drawn: the skins other than a default one.

use super::draw::inside;
use super::*;
use crate::{FontRuns, FrameRuns};
use photon_render::entity::player_renderer::default_skin;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// Left, top, width and height.
pub(super) type Rect = (i32, i32, i32, i32);

const FACE_SIZE: i32 = 24;
/// `FocusableTextWidget.DEFAULT_PADDING`; a header is one line and the padding twice.
const TEXT_PADDING: i32 = 4;
const HEADER_HEIGHT: i32 = 9 + 2 * TEXT_PADDING;
const SCROLL_STEP: i32 = 9;
const ICON_BUTTON: i32 = 20;
const SCROLLBAR_WIDTH: i32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Button {
    Remove,
    Accept,
    Decline,
    Revoke,
}

/// What a row of a list is, and where it is from the list's top.
enum Row {
    Header(bool, i32),
    Entry(usize, i32),
    /// The text under the friends: how to manage the account.
    Footer(i32),
}

impl FriendsOverlay {
    /// The rows of the shown list from its top, and how tall they are together. The entries stand
    /// one on the other (the layouts have no spacing), a header is its text with a padding of 4
    /// around, and the friends list ends with the footer.
    fn rows(&self, entries: &[Entry]) -> (Vec<Row>, i32) {
        let mut rows = Vec::new();
        let mut y = 0;
        let mut last = None;
        for (index, entry) in entries.iter().enumerate() {
            if self.tab == Tab::Pending && last != Some(entry.kind) {
                rows.push(Row::Header(entry.kind == Tab::received_kind(), y));
                y += HEADER_HEIGHT;
            }
            last = Some(entry.kind);
            rows.push(Row::Entry(index, y));
            y += ENTRY_HEIGHT;
        }
        if self.tab == Tab::Friends && !entries.is_empty() {
            rows.push(Row::Footer(y));
            y += self.footer_lines.len() as i32 * 9 + 2 * TEXT_PADDING;
        }
        (rows, y)
    }

    /// The footer's lines, centred in the entries' width, with where each starts.
    fn footer_at(&self, font: &Font, top: i32, left: i32) -> Vec<((i32, i32), &[Run])> {
        self.footer_lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                let x = left + ENTRY_WIDTH / 2 - font.width_of_runs(line) / 2;
                ((x, top + TEXT_PADDING + index as i32 * 9), line.as_slice())
            })
            .collect()
    }

    /// Whether `at` is on the link of the footer under the friends.
    pub(super) fn footer_link_at(&self, at: (i32, i32), font: &Font) -> bool {
        let area = self.list_area();
        if self.tab != Tab::Friends || !inside(at, area) {
            return false;
        }
        let (rows, _) = self.rows(self.shown());
        let Some(Row::Footer(y)) = rows.iter().find(|row| matches!(row, Row::Footer(_))) else {
            return false;
        };
        let top = area.1 + y - self.scroll;
        self.footer_at(font, top, area.0 + BORDER)
            .into_iter()
            .any(|((x, y), line)| {
                let mut left = x;
                line.iter().any(|run| {
                    let width = font.width_of_runs(std::slice::from_ref(run));
                    let hit = run.style.underlined == Some(true) && inside(at, (left, y, width, 9));
                    left += width;
                    hit
                })
            })
    }

    fn shown(&self) -> &[Entry] {
        let listing = match self.tab {
            Tab::Friends => &self.view.friends,
            Tab::Pending => &self.view.pending,
        };
        match listing {
            Listing::Entries(entries) => entries,
            _ => &[],
        }
    }

    /// Where the list is: the room under the box to add a friend.
    pub(super) fn list_area(&self) -> (i32, i32, i32, i32) {
        let (left, top, width, height) = self.content();
        (
            left,
            top + self.add_height(),
            width,
            height - self.add_height(),
        )
    }

    pub(super) fn max_scroll(&self) -> i32 {
        let (_, _, _, height) = self.list_area();
        (self.rows(self.shown()).1 - height).max(0)
    }

    /// A turn of the wheel over the list.
    pub fn scrolled(&mut self, mouse: (f64, f64), dy: f64) {
        let at = (mouse.0.floor() as i32, mouse.1.floor() as i32);
        if inside(at, self.list_area()) {
            let scroll = self.scroll - (dy * f64::from(SCROLL_STEP)).round() as i32;
            self.scroll = scroll.clamp(0, self.max_scroll());
        }
    }

    /// `isOverScrollbar`: the strip along the list's right edge, when the list scrolls.
    pub(super) fn over_scrollbar(&self, at: (i32, i32)) -> bool {
        let (x, y, width, height) = self.list_area();
        self.max_scroll() > 0
            && at.0 >= x + width - SCROLLBAR_WIDTH
            && at.0 <= x + width
            && at.1 >= y
            && at.1 < y + height
    }

    /// `AbstractScrollArea.mouseDragged`: the scroller follows the cursor by `dy`; above the list
    /// it goes to the top and below it to the end.
    pub fn dragged(&mut self, mouse_y: f64, dy: f64) {
        if !self.dragging {
            return;
        }
        let (_, y, _, height) = self.list_area();
        let max = self.max_scroll();
        let (_, content) = self.rows(self.shown());
        self.scroll = if mouse_y < f64::from(y) {
            0
        } else if mouse_y > f64::from(y + height) {
            max
        } else {
            let (_, bar) = scroller(0, max, (height, content));
            let scale = (f64::from(max.max(1)) / f64::from(height - bar)).max(1.0);
            (f64::from(self.scroll) + dy * scale).round() as i32
        }
        .clamp(0, max);
    }

    /// The primary button let go.
    pub fn released(&mut self) {
        self.dragging = false;
    }

    /// The buttons of an entry at `(x, y)`.
    fn buttons_of(kind: EntryKind, x: i32, y: i32) -> Vec<(Button, (i32, i32, i32, i32))> {
        let top = y + (ENTRY_HEIGHT - ICON_BUTTON) / 2;
        let right = x + ENTRY_WIDTH - ICON_BUTTON;
        let at = |x| (x, top, ICON_BUTTON, ICON_BUTTON);
        match kind {
            EntryKind::Friend => vec![(Button::Remove, at(right))],
            EntryKind::Outgoing => vec![(Button::Revoke, at(right))],
            EntryKind::Incoming => vec![
                (Button::Decline, at(right)),
                (Button::Accept, at(right - ICON_BUTTON - 4)),
            ],
        }
    }

    /// The press of a button of an entry at `at`, as what the screen asks of the game.
    pub(super) fn entry_pressed(&mut self, at: (i32, i32)) -> Option<FriendsEvent> {
        if self.view.sending {
            return None;
        }
        let (button, id, _) = self.button_under(at)?;
        // The remove button waits for the answer of its confirmation first (`removing`).
        if button != Button::Remove {
            self.pressed = Some(Pressed::Entry(button, id));
        }
        Some(match button {
            Button::Remove => FriendsEvent::Remove(id),
            Button::Accept => FriendsEvent::Accept(id),
            Button::Decline => FriendsEvent::Decline(id),
            Button::Revoke => FriendsEvent::Revoke(id),
        })
    }

    /// The button of an entry at `at`, and whose entry it is.
    pub(super) fn button_under(&self, at: (i32, i32)) -> Option<(Button, Uuid, Rect)> {
        let area = self.list_area();
        if !inside(at, area) {
            return None;
        }
        let entries = self.shown();
        let (rows, _) = self.rows(entries);
        let left = area.0 + BORDER;
        rows.iter().find_map(|row| {
            let Row::Entry(index, y) = row else {
                return None;
            };
            let entry = &entries[*index];
            let buttons = Self::buttons_of(entry.kind, left, area.1 + y - self.scroll);
            let (button, rect) = buttons.into_iter().find(|(_, rect)| inside(at, *rect))?;
            Some((button, entry.id, rect))
        })
    }

    pub(super) fn draw_entries(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
    ) {
        let area = self.list_area();
        let entries = self.shown();
        let (rows, _) = self.rows(entries);
        let left = area.0 + BORDER;
        let mark = frame.mark();
        for row in &rows {
            match row {
                Row::Header(received, y) => {
                    let text = if *received {
                        &self.texts.received
                    } else {
                        &self.texts.sent
                    };
                    // Centred in the entries' width, over a padding of the widget.
                    let x = left + (ENTRY_WIDTH - font.width_of_runs(text)) / 2;
                    let at = (x, area.1 + y - self.scroll + TEXT_PADDING);
                    frame.styled_text(atlas, font, text, at, WHITE, true);
                }
                Row::Footer(y) => {
                    for (at, line) in self.footer_at(font, area.1 + y - self.scroll, left) {
                        frame.styled_text(atlas, font, line, at, WHITE, true);
                    }
                }
                Row::Entry(index, y) => {
                    let at = (left, area.1 + y - self.scroll);
                    self.draw_entry(frame, (atlas, font), &entries[*index], at, mouse);
                }
            }
        }
        frame.clip_from(mark, (area.0, area.1, area.0 + area.2, area.1 + area.3));
        self.draw_scrollbar(frame, atlas, area);
    }

    /// `AbstractScrollArea.extractScrollbar`: drawn only when the list is taller than its room.
    fn draw_scrollbar(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, area: (i32, i32, i32, i32)) {
        let (_, content) = self.rows(self.shown());
        let max = self.max_scroll();
        if max == 0 {
            return;
        }
        let (x, y, width, height) = area;
        let bar_x = x + width - SCROLLBAR_WIDTH;
        let (bar_y, bar_height) = scroller(self.scroll, max, (height, content));
        frame.sprite(
            atlas,
            "widget/scroller_background",
            bar_x,
            y,
            SCROLLBAR_WIDTH,
            height,
        );
        frame.sprite(
            atlas,
            "widget/scroller",
            bar_x,
            y + bar_y,
            SCROLLBAR_WIDTH,
            bar_height,
        );
    }

    fn draw_entry(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        entry: &Entry,
        (x, y): (i32, i32),
        mouse: (i32, i32),
    ) {
        let face_y = y + (ENTRY_HEIGHT - FACE_SIZE) / 2;
        let texture = self.faces.texture(entry.id).map_or_else(
            || format!("textures/{}", default_skin(entry.id).texture),
            str::to_owned,
        );
        for u in [8, 40] {
            frame.blit_texture_stretched(
                atlas,
                &texture,
                (64, 64),
                (u, 8, 8, 8),
                (x, face_y, FACE_SIZE, FACE_SIZE),
            );
        }
        let text_x = x + FACE_SIZE + 4;
        let showing_status = entry.kind == EntryKind::Friend;
        let name_y = y + ENTRY_HEIGHT / if showing_status { 3 } else { 2 } - 9 / 2;
        frame.text_with_shadow(atlas, font, &entry.name, text_x, name_y, WHITE);
        if showing_status {
            let status_at = (text_x, name_y + 9 + 2);
            // `FriendEntry`: the room is up to the remove button, and a longer text slides.
            let room = x + ENTRY_WIDTH - ICON_BUTTON - text_x - 2;
            if font.width_of_runs(&entry.status) > room {
                let (top, bottom) = (status_at.1, status_at.1 + 9);
                crate::widgets::draw_label(
                    frame,
                    atlas,
                    font,
                    (&entry.status, true),
                    (text_x + 2, text_x + room - 2),
                    (top, bottom),
                );
            } else {
                frame.styled_text(atlas, font, &entry.status, status_at, WHITE, true);
            }
        }
        for (button, rect) in Self::buttons_of(entry.kind, x, y) {
            let loading = self.pressed == Some(Pressed::Entry(button, entry.id));
            self.draw_button(frame, atlas, (button, loading), rect, mouse);
        }
    }

    fn draw_button(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (button, loading): (Button, bool),
        (x, y, w, h): (i32, i32, i32, i32),
        mouse: (i32, i32),
    ) {
        let hovered = !self.view.sending && inside(mouse, (x, y, w, h));
        let sprite = if self.view.sending {
            "widget/button_disabled"
        } else if hovered {
            "widget/button_highlighted"
        } else {
            "widget/button"
        };
        frame.nine_slice(atlas, sprite, (200, 20), 1, false, (x, y, w, h));
        if loading {
            draw_loading_mark(frame, atlas, (x, y, w, h));
            return;
        }
        let (icon, size) = button_icon(button, hovered);
        frame.sprite(
            atlas,
            icon,
            x + w / 2 - size.0 / 2,
            y + h / 2 - size.1 / 2,
            size.0,
            size.1,
        );
    }
}

/// The picture of a button and its size; accept and decline have another one under the cursor
/// (`WidgetSprites`).
fn button_icon(button: Button, hovered: bool) -> (&'static str, (i32, i32)) {
    match (button, hovered) {
        (Button::Remove, _) => ("friends/remove", (13, 11)),
        (Button::Accept, true) => ("friends/accept_highlighted", (18, 18)),
        (Button::Accept, false) => ("friends/accept", (18, 18)),
        (Button::Decline, true) => ("friends/reject_highlighted", (18, 18)),
        (Button::Decline, false) => ("friends/reject", (18, 18)),
        (Button::Revoke, _) => ("friends/cancel", (12, 12)),
    }
}

impl Tab {
    /// The kind of entry the first section of the pending tab holds.
    fn received_kind() -> EntryKind {
        EntryKind::Incoming
    }
}

/// `SpriteIconButton.extractLoadingStateIfLoading`: a small mark in the middle of the button.
pub(super) fn draw_loading_mark(frame: &mut GuiFrame, atlas: &GuiAtlas, (x, y, w, h): Rect) {
    frame.sprite(
        atlas,
        "friends/loading",
        x + (w - 5) / 2,
        y + (h - 2) / 2,
        5,
        2,
    );
}

/// The scroller's offset from the list's top and its height (`scrollBarY`, `scrollerHeight`).
pub(super) fn scroller(scroll: i32, max: i32, (height, content): (i32, i32)) -> (i32, i32) {
    let bar = ((height * height) as f32 / content as f32) as i32;
    let bar = bar.clamp(32, height - 8);
    (scroll * (height - bar) / max, bar)
}
