//! The game rules screen's input (`AbstractGameRulesScreen`, `InWorldGameRulesScreen`): clicks on
//! the controls, buttons and scroll bar, the wheel, and the keys that edit the search box and the
//! number boxes.

use super::*;
use crate::widgets::edit_box::{Clipboard, edit_key};
use crate::widgets::scroll_area::SCROLLBAR_WIDTH;

impl GameRulesScreen {
    pub(super) fn set_focus(&mut self, focus: Option<Focus>) {
        if focus.is_some() && focus != self.focus {
            self.focused_at = Instant::now();
        }
        self.focus = focus;
    }

    pub(super) fn search_contains(&self, (x, y): (i32, i32)) -> bool {
        let rect = self.arena.0[self.search_rect];
        x >= rect.x && y >= rect.y && x < rect.x + rect.width && y < rect.y + rect.height
    }

    /// `getEntryAtPosition`: the row under `at`.
    pub(super) fn row_at(&self, (x, y): (i32, i32)) -> Option<usize> {
        let left = self.row_left();
        if x < left || x > left + ROW_WIDTH {
            return None;
        }
        (0..self.rows.len()).find(|&index| {
            let top = self.row_y(index);
            y >= top && y < top + ITEM_HEIGHT
        })
    }

    fn control_rect(&self, row: usize) -> Option<crate::layout::Rect> {
        match self.rows.get(row)? {
            Row::Rule {
                control: Control::Toggle(widget),
                ..
            } => Some(self.arena.0[widget.rect]),
            Row::Rule {
                control: Control::Input { rect, .. },
                ..
            } => Some(self.arena.0[*rect]),
            Row::Category(_, width) => Some(self.label_rect(row, *width)),
        }
    }

    /// The label widget of the category row `row`: the text, four pixels round it, centred.
    pub(super) fn label_rect(&self, row: usize, width: i32) -> crate::layout::Rect {
        crate::layout::Rect {
            x: self.row_left() + 2 + (ROW_WIDTH - 4) / 2 - width / 2 - 4,
            y: self.row_y(row) + 4,
            width: width + 8,
            height: 17,
        }
    }

    /// A press of the primary button at `mouse`: the first widget under it that takes it, in the
    /// order the screen added them (the search box, the buttons, then the list).
    pub fn clicked(&mut self, mouse: (f64, f64), font: &Font) -> Option<GameRulesEvent> {
        let at = (mouse.0.floor() as i32, mouse.1.floor() as i32);
        if self.search_contains(at) {
            self.set_focus(Some(Focus::Search));
            let rect = self.arena.0[self.search_rect];
            self.search.click_at(at.0 - (rect.x + 4), font);
            return None;
        }
        if self.done.is_active() && self.done.contains(&self.arena, at) {
            crate::ui_sound::play_down_sound();
            self.set_focus(Some(Focus::Done));
            return Some(GameRulesEvent::Apply(self.changes()));
        }
        if self.cancel.contains(&self.arena, at) {
            crate::ui_sound::play_down_sound();
            self.set_focus(Some(Focus::Cancel));
            return Some(self.close_requested());
        }
        if self.loading.is_none() && self.place.contains(mouse) {
            self.list_clicked(mouse, at, font);
        }
        None
    }

    /// `AbstractContainerWidget.mouseClicked` in the list: the scroll bar is taken hold of, or the
    /// control under the mouse takes the press.
    fn list_clicked(&mut self, mouse: (f64, f64), at: (i32, i32), font: &Font) {
        let bar = f64::from(self.row_left() + ROW_WIDTH + SCROLLBAR_WIDTH + 2);
        self.scrolling = self.max_scroll() > 0.0
            && mouse.0 >= bar
            && mouse.0 <= bar + f64::from(SCROLLBAR_WIDTH);
        let Some((row, rect)) = self
            .row_at(at)
            .and_then(|row| Some((row, self.control_rect(row)?)))
        else {
            return;
        };
        let inside = at.0 >= rect.x
            && at.0 < rect.x + rect.width
            && at.1 >= rect.y
            && at.1 < rect.y + rect.height;
        if inside && self.takes_focus_on_click(row) {
            self.set_focus(Some(Focus::Category(row)));
        } else if inside {
            self.press_control(row, at.0 - (rect.x + 4), font);
        }
    }

    /// The control of `row` pressed by the mouse, `offset` units into its field.
    fn press_control(&mut self, row: usize, offset: i32, font: &Font) {
        let Row::Rule { entry, control, .. } = &mut self.rows[row] else {
            return;
        };
        let entry = *entry;
        if let Control::Input { edit, .. } = control {
            edit.click_at(offset, font);
        } else {
            crate::ui_sound::play_down_sound();
            self.toggle(row);
        }
        self.set_focus(Some(Focus::Control(entry)));
    }

    /// The cycle button of the row pressed: the rule takes the other value.
    fn toggle(&mut self, row: usize) {
        let Row::Rule {
            entry,
            control: Control::Toggle(widget),
            ..
        } = &mut self.rows[row]
        else {
            return;
        };
        let rule = &mut self.entries[*entry];
        let value = !matches!(rule.current, Value::Bool(true));
        rule.current = Value::Bool(value);
        widget.kind = Kind::Button {
            label: self.on[usize::from(!value)].clone(),
        };
    }

    /// The row whose control belongs to the entry `entry`.
    fn row_of_entry(&self, entry: usize) -> Option<usize> {
        self.rows
            .iter()
            .position(|row| matches!(row, Row::Rule { entry: e, .. } if *e == entry))
    }

    /// `responder` of a number box: the rule takes the number when it is one, else the box turns
    /// red and Done cannot be pressed.
    fn validate(&mut self, row: usize) {
        let Row::Rule {
            entry,
            serial,
            control: Control::Input { edit, valid, .. },
        } = &mut self.rows[row]
        else {
            return;
        };
        let rule = &mut self.entries[*entry];
        if let Some(value) = deserialize(rule.rule, &edit.value()) {
            edit.set_colour(VALID_COLOUR);
            *valid = true;
            self.invalid.remove(serial);
            rule.current = value;
        } else {
            edit.set_colour(INVALID_COLOUR);
            *valid = false;
            self.invalid.insert(*serial);
        }
        self.refresh_done();
    }

    /// Every widget Tab visits, in the order the screen added them (the list last), and whether
    /// each takes the focus.
    /// Whether a click on the row's label focuses it: a category's does since 26.3.
    fn takes_focus_on_click(&self, row: usize) -> bool {
        matches!(self.rows[row], Row::Category(..))
            && photon_core::versions::behaviour::debug_option_categories_styled()
    }

    fn tab_order(&self) -> Vec<(Focus, bool)> {
        let mut order = vec![
            (Focus::Search, true),
            (Focus::Done, self.done.is_active()),
            (Focus::Cancel, true),
        ];
        for (index, row) in self.rows.iter().enumerate() {
            match row {
                Row::Rule { entry, .. } => order.push((Focus::Control(*entry), true)),
                // 26.2 labels are plain text that never takes the focus.
                Row::Category(..)
                    if photon_core::versions::behaviour::debug_option_categories_styled() =>
                {
                    order.push((Focus::Category(index), true));
                }
                Row::Category(..) => {}
            }
        }
        order
    }

    /// `Screen.keyPressed` with Tab: the next widget that takes the focus after the focused one, or
    /// before it with Shift, round to the other end.
    pub fn tab(&mut self, forward: bool) {
        let order = self.tab_order();
        let at = self
            .focus
            .and_then(|focus| order.iter().position(|(f, _)| *f == focus));
        let takes = |candidate: &&(Focus, bool)| candidate.1;
        let found = match (forward, at) {
            (true, Some(at)) => order[at + 1..].iter().find(takes),
            (false, Some(at)) => order[..at].iter().rev().find(takes),
            _ => None,
        };
        let found = found.or_else(|| {
            if forward {
                order.iter().find(takes)
            } else {
                order.iter().rev().find(takes)
            }
        });
        if let Some(&(focus, _)) = found {
            self.set_focus(Some(focus));
            self.scroll_to_focus(focus);
        }
    }

    /// The list scrolls to the row a focus in it stands on.
    pub(super) fn scroll_to_focus(&mut self, focus: Focus) {
        match focus {
            Focus::Control(entry) => self.scroll_to_entry(entry),
            Focus::Category(row) => self.scroll_to_row(row),
            _ => {}
        }
    }

    /// `AbstractSelectionList.scrollToEntry`: the list scrolls the least that shows the whole row.
    pub(super) fn scroll_to_entry(&mut self, entry: usize) {
        if let Some(index) = self.row_of_entry(entry) {
            self.scroll_to_row(index);
        }
    }

    /// `AbstractSelectionList.scrollToEntry` for the row at `index`.
    pub(super) fn scroll_to_row(&mut self, index: usize) {
        let top_delta = self.row_y(index) - self.place.y - 2;
        if top_delta < 0 {
            self.set_scroll(self.scroll + f64::from(top_delta));
        }
        let bottom_delta = self.place.bottom() - self.row_y(index) - ITEM_HEIGHT - 2;
        if bottom_delta < 0 {
            self.set_scroll(self.scroll - f64::from(bottom_delta));
        }
    }

    /// The wheel over the list (`AbstractSelectionList.mouseScrolled`).
    pub fn scrolled(&mut self, mouse: (f64, f64), dy: f64) {
        if self.loading.is_none() && self.place.contains(mouse) {
            self.set_scroll(self.scroll - dy * SCROLL_RATE);
        }
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

    /// A character typed into the box that has the focus.
    pub fn char_typed(&mut self, character: char, font: &Font) {
        match self.focus {
            Some(Focus::Search) => {
                if self.search.char_typed(character, font) {
                    self.search_changed(font);
                }
            }
            Some(Focus::Control(entry)) => {
                let Some(row) = self.row_of_entry(entry) else {
                    return;
                };
                if let Row::Rule {
                    control: Control::Input { edit, .. },
                    ..
                } = &mut self.rows[row]
                    && edit.char_typed(character, font)
                {
                    self.validate(row);
                }
            }
            _ => {}
        }
    }

    fn search_changed(&mut self, font: &Font) {
        if self.search.value() != self.last_search && self.loading.is_none() {
            self.update_search(font);
        }
    }

    /// A key: Escape asks to close, Tab and the arrows move the focus, Enter and Space press the focused
    /// button, and the box with the focus edits.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        (control, shift): (bool, bool),
        font: &Font,
        clipboard: &mut dyn Clipboard,
    ) -> Option<GameRulesEvent> {
        if key == glfw::Key::Escape {
            return Some(self.close_requested());
        }
        if self.move_focus(key, shift) {
            return None;
        }
        let enter = matches!(
            key,
            glfw::Key::Enter | glfw::Key::KpEnter | glfw::Key::Space
        );
        if enter && let Some(event) = self.press_focused() {
            return event;
        }
        let edit = edit_key(key)?;
        match self.focus {
            Some(Focus::Search) => {
                if self
                    .search
                    .key_pressed(edit, control, shift, font, clipboard)
                {
                    self.search_changed(font);
                }
            }
            Some(Focus::Control(entry)) => {
                let row = self.row_of_entry(entry)?;
                if let Row::Rule {
                    control: Control::Input { edit: input, .. },
                    ..
                } = &mut self.rows[row]
                    && input.key_pressed(edit, control, shift, font, clipboard)
                {
                    self.validate(row);
                }
            }
            _ => {}
        }
        None
    }

    /// Enter or Space on the focused button: the press of the mouse. `None` when the focus is on
    /// no button; `Some(None)` when a press changed the rows only.
    fn press_focused(&mut self) -> Option<Option<GameRulesEvent>> {
        match self.focus? {
            Focus::Done if self.done.is_active() => {
                crate::ui_sound::play_down_sound();
                Some(Some(GameRulesEvent::Apply(self.changes())))
            }
            Focus::Cancel => {
                crate::ui_sound::play_down_sound();
                Some(Some(self.close_requested()))
            }
            Focus::Control(entry) => {
                let row = self.row_of_entry(entry)?;
                if matches!(
                    self.rows[row],
                    Row::Rule {
                        control: Control::Toggle(_),
                        ..
                    }
                ) {
                    crate::ui_sound::play_down_sound();
                    self.toggle(row);
                    return Some(None);
                }
                None
            }
            _ => None,
        }
    }

    /// The text typed into the search box, a character at a time.
    pub fn type_search(&mut self, text: &str, font: &Font) {
        for character in text.chars() {
            self.char_typed(character, font);
        }
    }
}
