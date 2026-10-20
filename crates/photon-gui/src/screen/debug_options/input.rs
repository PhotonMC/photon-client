//! The debug options screen's input (`DebugOptionsScreen`, `OptionList`, `OptionEntry`): clicks on
//! the buttons, the scroll bar and wheel, the search box.
//!
//! Not handled: focus travel with Tab and the arrows, Enter on a focused button, a drag selecting
//! text in the search box, and the double click.

use super::*;
use crate::widgets::edit_box::{Clipboard, edit_key};
use crate::widgets::scroll_area::SCROLLBAR_WIDTH;

impl DebugOptionsScreen {
    fn set_focus(&mut self, focus: Focus) {
        if focus == Focus::Search && self.focus != Focus::Search {
            self.focused_at = Instant::now();
        }
        self.focus = focus;
    }

    pub(super) fn search_contains(&self, (x, y): (i32, i32)) -> bool {
        let rect = self.arena.0[self.search_rect];
        x >= rect.x && y >= rect.y && x < rect.x + rect.width && y < rect.y + rect.height
    }

    /// `OptionEntry.refreshEntry` for every entry, from the entry list the game keeps.
    pub fn refresh(&mut self, entries: &EntryList) {
        self.entries.clone_from(entries);
        self.refresh_rows();
    }

    /// A press of the primary button at `mouse`: the first widget under it that is active takes
    /// it, in the order the screen added them (the search box, the list, the footer's buttons).
    pub fn clicked(&mut self, mouse: (f64, f64), font: &Font) -> Option<DebugOptionsEvent> {
        let at = (mouse.0.floor() as i32, mouse.1.floor() as i32);
        if self.search_contains(at) {
            self.set_focus(Focus::Search);
            let rect = self.arena.0[self.search_rect];
            self.search.click_at(at.0 - (rect.x + 4), font);
            return None;
        }
        if self.place.contains(mouse) {
            return self.list_clicked(mouse, at);
        }
        let hit = [(0, &self.profiles[0]), (1, &self.profiles[1])]
            .into_iter()
            .find(|(_, widget)| widget.is_active() && widget.contains(&self.arena, at));
        if let Some((index, _)) = hit {
            self.set_focus(Focus::Profile(index));
            return Some(self.press_profile(Profile::ALL[index], index));
        }
        if self.done.contains(&self.arena, at) {
            self.set_focus(Focus::Done);
            return Some(DebugOptionsEvent::Close);
        }
        None
    }

    /// `addProfileButton`'s action: the profile is loaded, the rows show it, and only its button
    /// cannot be pressed.
    fn press_profile(&mut self, profile: Profile, index: usize) -> DebugOptionsEvent {
        self.entries.load_profile(profile);
        self.refresh_rows();
        for widget in &mut self.profiles {
            widget.active = true;
        }
        self.profiles[index].active = false;
        DebugOptionsEvent::Profile(profile)
    }

    /// `AbstractContainerWidget.mouseClicked`: the scroll bar is taken hold of, or the active
    /// button of the entry under the mouse is pressed.
    fn list_clicked(&mut self, mouse: (f64, f64), at: (i32, i32)) -> Option<DebugOptionsEvent> {
        let bar = f64::from(self.row_left() + ROW_WIDTH + SCROLLBAR_WIDTH + 2);
        self.scrolling = self.max_scroll() > 0.0
            && mouse.0 >= bar
            && mouse.0 <= bar + f64::from(SCROLLBAR_WIDTH);
        // A row under the mouse answers the click, a category's too, and the list takes the focus.
        let row = self.row_at(at);
        if row.is_some() || self.scrolling {
            self.set_focus(Focus::List);
        }
        let (id, slot) = self.pressed_button(row?, at)?;
        self.set_focus(Focus::Button(id, slot));
        let status = BUTTON_STATUSES[slot];
        self.entries.set_status(id, status);
        // `OptionEntry.setValue`: both profiles can be loaded again.
        for widget in &mut self.profiles {
            widget.active = true;
        }
        self.refresh_rows();
        Some(DebugOptionsEvent::Status { id, status })
    }

    /// `getEntryAtPosition`: the row under `at`.
    fn row_at(&self, (x, y): (i32, i32)) -> Option<usize> {
        let left = self.row_left();
        if x < left || x > left + ROW_WIDTH {
            return None;
        }
        (0..self.rows.len()).find(|&index| {
            let top = self.row_y(index);
            y >= top && y < top + ITEM_HEIGHT
        })
    }

    /// The entry and the button of the row `index` under `at`, if an active one is.
    fn pressed_button(&self, index: usize, at: (i32, i32)) -> Option<(&'static str, usize)> {
        let Row::Entry { id, buttons, .. } = &self.rows[index] else {
            return None;
        };
        let slot = buttons
            .iter()
            .position(|button| button.is_active() && button.contains(&self.arena, at))?;
        Some((*id, slot))
    }

    /// The mouse moved with the primary button down, `dy` further down.
    pub fn dragged(&mut self, mouse: (f64, f64), dy: f64) {
        if self.scrolling {
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

    /// A key: Escape closes the screen, and the search box edits while it has the focus.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        (control, shift): (bool, bool),
        font: &Font,
        clipboard: &mut dyn Clipboard,
    ) -> Option<DebugOptionsEvent> {
        if key == glfw::Key::Escape {
            return Some(DebugOptionsEvent::Close);
        }
        if self.focus == Focus::Search
            && let Some(edit) = edit_key(key)
            && self
                .search
                .key_pressed(edit, control, shift, font, clipboard)
            && self.search.value() != self.last_search
        {
            self.update_search();
        }
        None
    }
}
