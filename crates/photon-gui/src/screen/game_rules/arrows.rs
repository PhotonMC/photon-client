//! The arrow keys: `Screen.keyPressed` moves the focus by the position of the widgets. Inside the
//! list the entries with a control are walked in order; from the list, the buttons under it are
//! the ones that overlap its column.

use super::{Control, Focus, GameRulesScreen, Row};

/// An arrow key.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Arrow {
    Up,
    Down,
    Left,
    Right,
}

impl Arrow {
    pub(super) fn of(key: glfw::Key) -> Option<Self> {
        match key {
            glfw::Key::Up => Some(Self::Up),
            glfw::Key::Down => Some(Self::Down),
            glfw::Key::Left => Some(Self::Left),
            glfw::Key::Right => Some(Self::Right),
            _ => None,
        }
    }
}

impl GameRulesScreen {
    /// Tab and the arrow keys: whether the key moved the focus (or was Tab).
    pub(super) fn move_focus(&mut self, key: glfw::Key, shift: bool) -> bool {
        if key == glfw::Key::Tab {
            self.tab(!shift);
            return true;
        }
        Arrow::of(key).is_some_and(|arrow| self.arrow(arrow))
    }

    /// The rule entries, in list order, and whether each is a toggle.
    fn control_entries(&self) -> Vec<(usize, bool)> {
        self.rows
            .iter()
            .filter_map(|row| match row {
                Row::Rule { entry, control, .. } => {
                    Some((*entry, matches!(control, Control::Toggle(_))))
                }
                Row::Category { .. } => None,
            })
            .collect()
    }

    /// An arrow key. `false` when the focused widget keeps the key for itself (a text box moves
    /// its cursor with Left and Right) or nothing moved.
    pub(super) fn arrow(&mut self, arrow: Arrow) -> bool {
        let entries = self.control_entries();
        let target = match (self.focus, arrow) {
            (Some(Focus::Search), Arrow::Down) => entries.first().map(|e| Focus::Control(e.0)),
            (Some(Focus::Search), _) => None,
            (Some(Focus::Control(entry)), _) => self.control_target(&entries, entry, arrow),
            (Some(Focus::Done), Arrow::Right) => Some(Focus::Cancel),
            (Some(Focus::Cancel), Arrow::Left) if self.done.is_active() => Some(Focus::Done),
            (Some(Focus::Done | Focus::Cancel), Arrow::Up) => {
                entries.last().map(|e| Focus::Control(e.0))
            }
            _ => None,
        };
        let Some(target) = target else {
            return false;
        };
        self.set_focus(Some(target));
        if let Focus::Control(entry) = target {
            self.scroll_to_entry(entry);
        }
        true
    }

    /// The arrow from a control: the neighbour entries, the search box above the first, the
    /// button under the list column below the last, and the left button from a toggle.
    fn control_target(
        &self,
        entries: &[(usize, bool)],
        entry: usize,
        arrow: Arrow,
    ) -> Option<Focus> {
        let at = entries.iter().position(|e| e.0 == entry)?;
        match arrow {
            Arrow::Up if at == 0 => Some(Focus::Search),
            Arrow::Up => Some(Focus::Control(entries[at - 1].0)),
            Arrow::Down => Some(
                entries
                    .get(at + 1)
                    .map_or(Focus::Cancel, |e| Focus::Control(e.0)),
            ),
            Arrow::Left if entries[at].1 && self.done.is_active() => Some(Focus::Done),
            Arrow::Left | Arrow::Right => None,
        }
    }
}
