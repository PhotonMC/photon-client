//! The social interactions screen's input (`SocialInteractionsScreen`, `SocialInteractionsPlayerList`
//! and `PlayerEntry`): clicks, the scroll bar and wheel, the search box.
//!
//! Not handled: focus travel with Tab and the arrows, Enter on a focused button, a drag selecting
//! text in the search box, the double click, and the report button's focus, which the report
//! screen would show.

use super::*;
use crate::widgets::edit_box::{Clipboard, edit_key};

/// `AbstractSelectionList`'s scroll per wheel step: half an entry.
const SCROLL_RATE: f64 = (ENTRY_HEIGHT / 2) as f64;
/// The size of an entry's buttons, and the room around them.
const BUTTON_SIZE: i32 = 20;

impl SocialInteractionsScreen {
    fn set_focus(&mut self, focus: Focus) {
        if focus == Focus::Search && self.focus != Focus::Search {
            self.focused_at = Instant::now();
        }
        self.focus = focus;
        self.buttons.set_focus(match focus {
            Focus::Button(index) => Some(index),
            _ => None,
        });
    }

    fn max_scroll(&self) -> f64 {
        f64::from(self.place.max_scroll(self.content_height()))
    }

    pub(super) fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, self.max_scroll());
    }

    /// Where the search box is: `(x, y, width, height)`.
    fn search_rect(&self) -> (i32, i32, i32, i32) {
        (self.margin_x() + 28, 74, 200, 15)
    }

    fn inside((x, y): (i32, i32), (left, top, width, height): (i32, i32, i32, i32)) -> bool {
        x >= left && x < left + width && y >= top && y < top + height
    }

    /// The buttons of the entry under `mouse`, if it has any: the entry is found, and which button
    /// is under the mouse, the hide or show button (`false`) or the report button (`true`).
    fn entry_at(&self, (x, y): (i32, i32)) -> Option<(usize, Option<bool>)> {
        let left = self.row_left();
        let index = (0..self.entries.len()).find(|index| {
            Self::inside(
                (x, y),
                (left, self.entry_y(*index), ROW_WIDTH, ENTRY_HEIGHT),
            )
        })?;
        let entry = &self.entries[index];
        if entry.player.id == self.roster.local {
            return Some((index, None));
        }
        let (content_x, content_y) = (left + 2, self.entry_y(index) + 2);
        let content_width = ROW_WIDTH - 4;
        let button_y = content_y + (ENTRY_HEIGHT - 4 - BUTTON_SIZE) / 2;
        let report_x = content_x + content_width - 24;
        let hide_x = report_x - 24;
        let over =
            |button_x: i32| Self::inside((x, y), (button_x, button_y, BUTTON_SIZE, BUTTON_SIZE));
        let button = if over(hide_x) {
            Some(false)
        } else if self.roster.reporting && over(report_x) {
            Some(true)
        } else {
            None
        };
        Some((index, button))
    }

    /// A press of the primary button at `mouse`: the first widget under it takes it, in the order
    /// the screen added them (the tabs, the search box, the blocking hint, the list, Done), and
    /// takes the focus.
    pub fn clicked(&mut self, mouse: (f64, f64), font: &Font) -> Option<SocialEvent> {
        self.buttons.mouse_input();
        let at = (mouse.0.floor() as i32, mouse.1.floor() as i32);
        let hit = self.buttons.button_at(at);
        if let Some((index, SocialAction::Page(page))) = hit {
            self.set_focus(Focus::Button(index));
            self.show_page(page);
            self.relabel_tabs();
            return None;
        }
        let search = self.search_rect();
        if Self::inside(at, search) {
            self.set_focus(Focus::Search);
            self.search.click_at(at.0 - (search.0 + 4), font);
            return None;
        }
        if self.hint_visible.get() && self.blocking_hint.contains(&self.buttons.arena, at) {
            self.set_focus(Focus::Hint);
            return Some(SocialEvent::BlockingHelp);
        }
        if self.place.contains(mouse) {
            return self.list_clicked(mouse, at);
        }
        match hit {
            Some((index, SocialAction::Done)) => {
                self.set_focus(Focus::Button(index));
                Some(SocialEvent::Close)
            }
            _ => None,
        }
    }

    /// `AbstractContainerWidget.mouseClicked`: the scroll bar is taken hold of, or the buttons of
    /// the entry under the mouse are pressed.
    fn list_clicked(&mut self, mouse: (f64, f64), at: (i32, i32)) -> Option<SocialEvent> {
        let bar = f64::from(self.row_left() + ROW_WIDTH + SCROLLBAR_WIDTH + 2);
        self.scrolling = self.max_scroll() > 0.0
            && mouse.0 >= bar
            && mouse.0 <= bar + f64::from(SCROLLBAR_WIDTH);
        let entry = self.entry_at(at);
        if entry.is_some() || self.scrolling {
            self.set_focus(Focus::List);
        }
        let (index, Some(report)) = entry? else {
            return None;
        };
        let player = self.entries[index].player.clone();
        if report {
            return Some(SocialEvent::Report(player.id));
        }
        let hidden = !self.hidden(player.id);
        if hidden {
            self.roster.hidden.push(player.id);
        } else {
            self.roster.hidden.retain(|id| *id != player.id);
        }
        Some(SocialEvent::Hidden {
            id: player.id,
            name: player.name,
            hidden,
        })
    }

    /// `showPage` marks the page's tab with an underline.
    fn relabel_tabs(&mut self) {
        let pages = [Page::All, Page::Hidden, Page::Blocked];
        for (index, labels) in self.tab_labels.iter().enumerate() {
            let label = labels[usize::from(pages[index] == self.page)].clone();
            self.buttons.set_label(index, label);
        }
    }

    /// The mouse moved with the primary button down, `dy` further down.
    pub fn dragged(&mut self, mouse: (f64, f64), dy: f64) {
        if self.scrolling && self.focus == Focus::List {
            let scroll = self
                .place
                .dragged_scroll(self.scroll, mouse.1, dy, self.content_height());
            self.set_scroll(scroll);
        }
    }

    /// The primary button went up.
    pub fn released(&mut self) {
        self.scrolling = false;
    }

    /// The wheel turned `dy` steps with the mouse at `mouse`: the list scrolls unless another
    /// widget is the first under it.
    pub fn scrolled(&mut self, mouse: (f64, f64), dy: f64) {
        let at = (mouse.0.floor() as i32, mouse.1.floor() as i32);
        let over_other = self
            .buttons
            .button_at(at)
            .is_some_and(|(_, action)| matches!(action, SocialAction::Page(_)))
            || Self::inside(at, self.search_rect())
            || (self.hint_visible.get() && self.blocking_hint.contains(&self.buttons.arena, at));
        if !over_other && self.place.contains(mouse) {
            self.set_scroll(self.scroll - dy * SCROLL_RATE);
        }
    }

    /// A key. `social_key` is whether it matches the social interactions key, which closes the
    /// screen unless the search box has the focus.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        (control, shift): (bool, bool),
        social_key: bool,
        font: &Font,
        clipboard: &mut dyn Clipboard,
    ) -> Option<SocialEvent> {
        if self.focus != Focus::Search && social_key || key == glfw::Key::Escape {
            return Some(SocialEvent::Close);
        }
        if self.focus == Focus::Search
            && let Some(edit) = edit_key(key)
            && self
                .search
                .key_pressed(edit, control, shift, font, clipboard)
        {
            self.search_changed();
        }
        None
    }

    /// A character typed: the search box takes it while it has the focus.
    pub fn char_typed(&mut self, character: char, font: &Font) {
        if self.focus == Focus::Search && self.search.char_typed(character, font) {
            self.search_changed();
        }
    }

    /// The text typed into the search box, a character at a time.
    pub fn type_search(&mut self, text: &str, font: &Font) {
        for character in text.chars() {
            self.char_typed(character, font);
        }
    }

    /// `checkSearchStringUpdate`: the list is filtered again when the lowercased text changed.
    fn search_changed(&mut self) {
        let lowered = self.search.value().to_lowercase();
        if lowered != self.last_search {
            self.last_search = lowered;
            self.show_page(self.page);
        }
    }
}
