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
        let in_list = matches!(self.focus, Focus::List | Focus::Button(..));
        let target = if in_list {
            self.list_target(direction, self.focused_button())
        } else {
            None
        };
        let target = target.or_else(|| self.screen_target(direction));
        if let Some(focus) = target {
            self.set_focus(focus);
            if let Focus::Button(id, _) = focus {
                self.scroll_to_entry(id);
            }
        }
    }

    /// The row and slot of the button that has the focus.
    fn focused_button(&self) -> Option<(usize, usize)> {
        let Focus::Button(id, slot) = self.focus else {
            return None;
        };
        let row = self
            .rows
            .iter()
            .position(|row| matches!(row, Row::Entry { id: row_id, .. } if *row_id == id))?;
        Some((row, slot))
    }

    /// Whether the button `slot` of the row `row` can take the focus.
    fn takes_focus(&self, (row, slot): (usize, usize)) -> bool {
        matches!(&self.rows[row], Row::Entry { buttons, .. } if buttons[slot].is_active())
    }

    fn focus_of(&self, (row, slot): (usize, usize)) -> Option<Focus> {
        match &self.rows[row] {
            Row::Entry { id, .. } => Some(Focus::Button(id, slot)),
            Row::Category(_) => None,
        }
    }

    /// `AbstractSelectionList.nextEntry` over the entries that have buttons.
    fn next_entry(&self, direction: Direction, start: Option<usize>) -> Option<usize> {
        let delta: isize = match direction {
            Direction::Up => -1,
            Direction::Down => 1,
            _ => return None,
        };
        let has_buttons = |row: usize| matches!(self.rows[row], Row::Entry { .. });
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
            candidate(self.rectangle(self.done.rect), self.done.is_active()),
        ];
        let focused = match self.focus {
            Focus::Search => 1,
            Focus::List | Focus::Button(..) => 3,
            Focus::Profile(index) => 4 + index,
            Focus::Done => 6,
        };
        // A button of the list gives the border: the list asks the entry, and it the button.
        let mut children = children;
        if let Some((row, slot)) = self.focused_button()
            && let Row::Entry { buttons, .. } = &self.rows[row]
        {
            children[3].rectangle = self.rectangle(buttons[slot].rect);
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
                index @ (4 | 5) => Focus::Profile(index - 4),
                _ => Focus::Done,
            },
        )
    }
}
