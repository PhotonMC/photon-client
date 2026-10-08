//! How the focus moves in `KeyBindsList`: by the mouse, Tab and the arrows.

use super::*;

impl KeyBindsList {
    /// `setFocused` down the list: a row that sticks out of it, or is reached by the keyboard,
    /// is scrolled into it.
    pub(in crate::gui::options::keybinds) fn focus_on(
        &mut self,
        focus: Option<Focus>,
        input: InputType,
    ) {
        let old = self.focus;
        if old == focus {
            return;
        }
        self.focus = focus;
        if let Some((row, _)) = focus {
            let changed_row = !matches!(old, Some((before, _)) if before == row);
            let top = self.row_y(row) + 2;
            let bottom = self.row_y(row) + ITEM_HEIGHT - 2;
            let keyboard = matches!(input, InputType::KeyboardArrow | InputType::KeyboardTab);
            if changed_row && (keyboard || top < self.place.y || bottom > self.place.bottom()) {
                self.scroll_to_row(row);
            }
        }
    }

    /// The focus moves by a key.
    pub(in crate::gui::options::keybinds) fn move_focus(
        &mut self,
        focus: Option<Focus>,
        input: InputType,
    ) {
        self.focus_on(None, input);
        self.focus_on(focus, input);
        self.screen_focused = focus.is_some();
        self.input = input;
    }

    /// How many widgets a row has, and whether one of them takes the focus.
    fn widgets_in(&self, row: usize) -> usize {
        match &self.rows[row] {
            Row::Category { .. } => 1,
            Row::Key { .. } => 2,
        }
    }

    fn is_active(&self, (row, child): Focus) -> bool {
        match &self.rows[row] {
            Row::Category { .. } => true,
            Row::Key { change, reset, .. } => [change, reset][child].is_active(),
        }
    }

    fn first_active(&self, row: usize, order: impl Iterator<Item = usize>) -> Option<Focus> {
        order
            .map(|child| (row, child))
            .find(|&at| self.is_active(at))
    }

    /// `ContainerObjectSelectionList.nextFocusPath`.
    pub(in crate::gui::options::keybinds) fn next_focus(&self, nav: Nav) -> Option<Focus> {
        match nav {
            Nav::Tab(forward) => self.tab_focus(forward),
            Nav::Arrow(direction) => self.arrow_focus(direction),
        }
    }

    /// Tab: on along the row's widgets, then the rows after (or before) it.
    fn tab_focus(&self, forward: bool) -> Option<Focus> {
        if let Some((row, child)) = self.focus {
            let found = if forward {
                self.first_active(row, child + 1..self.widgets_in(row))
            } else {
                self.first_active(row, (0..child).rev())
            };
            if found.is_some() {
                return found;
            }
        }
        let rows: Vec<usize> = match (self.focus, forward) {
            (Some((row, _)), true) => (row + 1..self.rows.len()).collect(),
            (Some((row, _)), false) => (0..row).rev().collect(),
            (None, true) => (0..self.rows.len()).collect(),
            (None, false) => (0..self.rows.len()).rev().collect(),
        };
        rows.into_iter().find_map(|row| {
            let count = self.widgets_in(row);
            if forward {
                self.first_active(row, 0..count)
            } else {
                self.first_active(row, (0..count).rev())
            }
        })
    }

    /// An arrow: left and right along the row; up and down to the row above or below, at the
    /// same place in it (or the last it has).
    fn arrow_focus(&self, direction: Direction) -> Option<Focus> {
        if let (Direction::Left | Direction::Right, Some((row, child))) = (direction, self.focus) {
            return if direction == Direction::Right {
                self.first_active(row, child + 1..self.widgets_in(row))
            } else {
                self.first_active(row, (0..child).rev())
            };
        }
        let (index, direction) = match (self.focus, direction) {
            (Some((_, child)), direction) => (child, direction),
            (None, Direction::Left) => (usize::MAX, Direction::Down),
            (None, Direction::Right) => (0, Direction::Down),
            (None, direction) => (0, direction),
        };
        let down = direction == Direction::Down;
        let mut row = self.focus.map(|(row, _)| row);
        loop {
            row = match (row, down) {
                (None, true) => Some(0),
                (None, false) => self.rows.len().checked_sub(1),
                (Some(row), true) => Some(row + 1),
                (Some(row), false) => row.checked_sub(1),
            }
            .filter(|row| *row < self.rows.len());
            let row = row?;
            let at = (row, index.min(self.widgets_in(row) - 1));
            if self.is_active(at) {
                return Some(at);
            }
        }
    }
}
