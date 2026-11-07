//! The arrow keys on the debug options screen (`Screen.keyPressed`, `ContainerEventHandler`,
//! `ContainerObjectSelectionList.nextFocusPath`): the focus moves to the widget next to the focused
//! one, and inside the list from button to button.

use super::*;
use crate::widgets::navigation::{Candidate, Direction, Rectangle, next_in_direction};

/// The three buttons of an entry, as Left and Right go over them.
const SLOTS: usize = 3;

impl DebugOptionsScreen {
    /// An arrow key the search box does not take: what the focused widget offers, or else the
    /// widget that lies next to it.
    pub(super) fn arrow(&mut self, direction: Direction) {
        let in_list = matches!(
            self.focus,
            Focus::List | Focus::Button(..) | Focus::Category(_)
        );
        let target = if in_list {
            self.list_target(direction, self.focused_button())
        } else {
            None
        };
        let target = target.or_else(|| self.screen_target(direction));
        if let Some(focus) = target {
            self.set_focus(focus);
            match focus {
                Focus::Button(id, _) => self.scroll_to_entry(id),
                Focus::Category(index) => self.scroll_to_row(index),
                _ => {}
            }
        }
    }

    /// The row and slot of the widget that has the focus in the list: a category's label is the
    /// first and only child of its row.
    fn focused_button(&self) -> Option<(usize, usize)> {
        let (id, slot) = match self.focus {
            Focus::Button(id, slot) => (id, slot),
            Focus::Category(index) => return Some((index, 0)),
            _ => return None,
        };
        let row = self
            .rows
            .iter()
            .position(|row| matches!(row, Row::Entry { id: row_id, .. } if *row_id == id))?;
        Some((row, slot))
    }

    /// Whether the child `slot` of the row `row` can take the focus; the index is clamped to the
    /// row's children, so a label takes it whatever the slot.
    fn takes_focus(&self, (row, slot): (usize, usize)) -> bool {
        match &self.rows[row] {
            Row::Category(..) => photon_core::versions::behaviour::debug_option_categories_styled(),
            Row::Entry { buttons, .. } => buttons[slot].is_active(),
        }
    }

    fn focus_of(&self, (row, slot): (usize, usize)) -> Option<Focus> {
        match &self.rows[row] {
            Row::Entry { id, .. } => Some(Focus::Button(id, slot)),
            Row::Category(..) => Some(Focus::Category(row)),
        }
    }

    /// `AbstractSelectionList.nextEntry` over the entries that have buttons.
    fn next_entry(&self, direction: Direction, start: Option<usize>) -> Option<usize> {
        let delta: isize = match direction {
            Direction::Up => -1,
            Direction::Down => 1,
            _ => return None,
        };
        // Every row has a child to take the focus, but 26.2's category labels are plain text.
        let categories_focus = photon_core::versions::behaviour::debug_option_categories_styled();
        let has_buttons =
            |row: usize| categories_focus || matches!(self.rows[row], Row::Entry { .. });
        let mut index = match start {
            Some(row) => row as isize + delta,
            None if delta > 0 => 0,
            None => self.rows.len() as isize - 1,
        };
        while (0..self.rows.len() as isize).contains(&index) {
            if has_buttons(index as usize) {
                return Some(index as usize);
            }
            index += delta;
        }
        None
    }

    /// `ContainerObjectSelectionList.nextFocusPath` for an arrow key, with `focused` the button
    /// that has the focus, if one has.
    fn list_target(&self, direction: Direction, focused: Option<(usize, usize)>) -> Option<Focus> {
        if self.rows.is_empty() {
            return None;
        }
        let horizontal = matches!(direction, Direction::Left | Direction::Right);
        if let (true, Some((row, slot))) = (horizontal, focused) {
            return self.along_entry(direction, row, slot);
        }
        let (mut slot, direction) = match (focused, direction) {
            (Some((_, slot)), _) => (slot, direction),
            (None, Direction::Left) => (SLOTS - 1, Direction::Down),
            (None, Direction::Right) => (0, Direction::Down),
            (None, _) => (0, direction),
        };
        slot = slot.min(SLOTS - 1);
        let mut entry = focused.map(|(row, _)| row);
        loop {
            let row = self.next_entry(direction, entry)?;
            if self.takes_focus((row, slot)) {
                return self.focus_of((row, slot));
            }
            entry = Some(row);
        }
    }

    /// `OptionEntry.nextFocusPath` for Left and Right: the next button of the entry that takes it.
    fn along_entry(&self, direction: Direction, row: usize, slot: usize) -> Option<Focus> {
        if matches!(self.rows[row], Row::Category(..)) {
            return None;
        }
        let delta: isize = if direction == Direction::Left { -1 } else { 1 };
        let mut index = (slot as isize + delta).clamp(0, SLOTS as isize - 1);
        while (0..SLOTS as isize).contains(&index) {
            if index as usize != slot && self.takes_focus((row, index as usize)) {
                return self.focus_of((row, index as usize));
            }
            index += delta;
        }
        None
    }

    /// The label widget of the category row `index`: the text, four pixels round it, centred.
    pub(super) fn label_rectangle(&self, index: usize, width: i32) -> Rectangle {
        Rectangle {
            x: self.row_left() + 2 + (ROW_WIDTH - 4) / 2 - width / 2 - 4,
            y: self.row_y(index) + 2,
            width: width + 8,
            height: 17,
        }
    }

    fn rectangle(&self, rect: usize) -> Rectangle {
        let rect = self.arena.0[rect];
        Rectangle {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }

    /// `ContainerEventHandler.handleArrowNavigation` of the screen: its widgets in the order they
    /// were added (the title, search box, warning, list, the two profiles and Done).
    fn screen_target(&self, direction: Direction) -> Option<Focus> {
        let list = Rectangle {
            x: 0,
            y: self.place.y,
            width: self.place.width,
            height: self.place.height,
        };
        let list_target = self.list_target(direction, None);
        let candidate = |rectangle, focusable| Candidate {
            rectangle,
            focusable,
        };
        let children = [
            candidate(self.rectangle(self.title.rect), false),
            candidate(self.rectangle(self.search_rect), true),
            candidate(self.rectangle(self.subtitle_rect), false),
            candidate(list, list_target.is_some()),
            candidate(
                self.rectangle(self.profiles[0].rect),
                self.profiles[0].is_active(),
            ),
            candidate(
                self.rectangle(self.profiles[1].rect),
                self.profiles[1].is_active(),
            ),
            candidate(
                self.rectangle(self.gui_scale.rect),
                self.gui_scale.is_active() && self.has_gui_scale(),
            ),
            candidate(self.rectangle(self.done.rect), self.done.is_active()),
        ];
        let focused = focus_child(self.focus);
        // A button of the list gives the border: the list asks the entry, and it the button.
        let mut children = children;
        if let Some((row, slot)) = self.focused_button()
            && let Row::Entry { buttons, .. } = &self.rows[row]
        {
            children[3].rectangle = self.rectangle(buttons[slot].rect);
        }
        if let Focus::Category(index) = self.focus
            && let Row::Category(_, width) = &self.rows[index]
        {
            children[3].rectangle = self.label_rectangle(index, *width);
        }
        let container = Rectangle {
            x: 0,
            y: 0,
            width: self.size.0,
            height: self.size.1,
        };
        Some(
            match next_in_direction(&children, Some(focused), direction, container)? {
                1 => Focus::Search,
                3 => list_target?,
                index => child_focus(index),
            },
        )
    }
}

/// The screen child a focus belongs to: the search box, the list, a footer button.
fn focus_child(focus: Focus) -> usize {
    match focus {
        Focus::Search => 1,
        Focus::List | Focus::Button(..) | Focus::Category(_) => 3,
        Focus::Profile(index) => 4 + index,
        Focus::GuiScale => 6,
        Focus::Done => 7,
    }
}

/// The focus of a footer child.
fn child_focus(index: usize) -> Focus {
    match index {
        4 | 5 => Focus::Profile(index - 4),
        6 => Focus::GuiScale,
        _ => Focus::Done,
    }
}
