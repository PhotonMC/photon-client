//! The entries of the friends overlay's lists: their layout, drawing and buttons
//! (`FriendEntry`, `IncomingEntry`, `OutgoingEntry`).
//!
//! Not drawn: the buttons' loading states, the status text scrolling
//! when it is too long, and the skins other than a default one.

use super::draw::inside;
use super::*;
use crate::FrameRuns;
use photon_render::entity::player_renderer::default_skin;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// Left, top, width and height.
pub(super) type Rect = (i32, i32, i32, i32);

const ENTRY_WIDTH: i32 = OVERLAY_WIDTH - 16;
const FACE_SIZE: i32 = 24;
const ENTRY_GAP: i32 = 4;
const HEADER_HEIGHT: i32 = 9;
const SECTION_GAP: i32 = 8;
const SCROLL_STEP: i32 = 9;
const ICON_BUTTON: i32 = 20;
const SCROLLBAR_WIDTH: i32 = 6;
const GREY: u32 = 0xFFA0_A0A0;

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
}

impl FriendsOverlay {
    /// The rows of the shown list from its top, and how tall they are together.
    fn rows(&self, entries: &[Entry]) -> (Vec<Row>, i32) {
        let mut rows = Vec::new();
        let mut y = 0;
        let mut last = None;
        for (index, entry) in entries.iter().enumerate() {
            if self.tab == Tab::Pending && last != Some(entry.kind) {
                if last.is_some() {
                    y += SECTION_GAP - ENTRY_GAP;
                }
                rows.push(Row::Header(entry.kind == Tab::received_kind(), y));
                y += HEADER_HEIGHT + ENTRY_GAP;
            }
            last = Some(entry.kind);
            rows.push(Row::Entry(index, y));
            y += ENTRY_HEIGHT + ENTRY_GAP;
        }
        (rows, (y - ENTRY_GAP).max(0))
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
            top + ADD_WIDGET_HEIGHT,
            width,
            height - ADD_WIDGET_HEIGHT,
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
    pub(super) fn entry_pressed(&self, at: (i32, i32)) -> Option<FriendsEvent> {
        if self.view.sending {
            return None;
        }
        let (button, id, _) = self.button_under(at)?;
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
                    let at = (left, area.1 + y - self.scroll);
                    frame.styled_text(atlas, font, text, at, WHITE, true);
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
        let texture = format!("textures/{}", default_skin(entry.id).texture);
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
            frame.styled_text(
                atlas,
                font,
                &entry.status,
                (text_x, name_y + 9 + 2),
                GREY,
                true,
            );
        }
        for (button, rect) in Self::buttons_of(entry.kind, x, y) {
            self.draw_button(frame, atlas, button, rect, mouse);
        }
    }

    fn draw_button(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        button: Button,
        (x, y, w, h): (i32, i32, i32, i32),
        mouse: (i32, i32),
    ) {
        let sprite = if self.view.sending {
            "widget/button_disabled"
        } else if inside(mouse, (x, y, w, h)) {
            "widget/button_highlighted"
        } else {
            "widget/button"
        };
        frame.nine_slice(atlas, sprite, (200, 20), 1, false, (x, y, w, h));
        let (icon, size) = match button {
            Button::Remove => ("friends/remove", (13, 11)),
            Button::Accept => ("friends/accept", (18, 18)),
            Button::Decline => ("friends/reject", (18, 18)),
            Button::Revoke => ("friends/cancel", (12, 12)),
        };
        frame.sprite(
            atlas,
            icon,
            x + (w - size.0) / 2,
            y + (h - size.1) / 2,
            size.0,
            size.1,
        );
    }
}

impl Tab {
    /// The kind of entry the first section of the pending tab holds.
    fn received_kind() -> EntryKind {
        EntryKind::Incoming
    }
}

/// The scroller's offset from the list's top and its height (`scrollBarY`, `scrollerHeight`).
pub(super) fn scroller(scroll: i32, max: i32, (height, content): (i32, i32)) -> (i32, i32) {
    let bar = ((height * height) as f32 / content as f32) as i32;
    let bar = bar.clamp(32, height - 8);
    (scroll * (height - bar) / max, bar)
}
