//! The keyboard focus of the statistics screen: Tab and Shift+Tab around the tab bar, the list of
//! the tab shown and Done (`Screen.keyPressed`, `ContainerEventHandler.handleTabNavigation`), and
//! Enter or Space on the focused child.

use super::{SHIFT_MODIFIER, ScreenFocus, StatsAction, StatsScreen, TAB_KEY};

/// `GLFW_KEY_ENTER`, `GLFW_KEY_SPACE` and `GLFW_KEY_KP_ENTER`: `KeyEvent.isSelection`.
const SELECTION_KEYS: [i32; 3] = [257, 32, 335];

/// `GLFW_KEY_RIGHT`, `LEFT`, `DOWN` and `UP`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Arrow {
    Left,
    Right,
    Up,
    Down,
}

impl Arrow {
    fn of_key(key: i32) -> Option<Self> {
        match key {
            262 => Some(Self::Right),
            263 => Some(Self::Left),
            264 => Some(Self::Down),
            265 => Some(Self::Up),
            _ => None,
        }
    }
}

/// The children in tab order: the screen sorts them by tab order group (Done is in group 1) and
/// keeps the order they were added in within a group.
const ORDER: [ScreenFocus; 3] = [ScreenFocus::Bar, ScreenFocus::List, ScreenFocus::Done];

impl StatsScreen {
    /// `StatsScreen.keyPressed`: the tab bar first (Control with a digit or Tab), then the screen.
    pub fn key_pressed(&mut self, key: i32, modifiers: i32) -> Option<StatsAction> {
        if self.bar_key(key, modifiers) {
            return None;
        }
        if let Some(arrow) = Arrow::of_key(key) {
            self.arrow_focus(arrow);
            None
        } else if key == TAB_KEY {
            self.tab_focus(modifiers & SHIFT_MODIFIER == 0);
            None
        } else if SELECTION_KEYS.contains(&key) {
            self.press_focused()
        } else {
            None
        }
    }

    /// Enter or Space: the focused child is pressed. A tab is chosen; Done leaves.
    fn press_focused(&mut self) -> Option<StatsAction> {
        match (self.focus, self.focused_tab) {
            (ScreenFocus::Done, _) => {
                crate::ui_sound::play_down_sound();
                Some(StatsAction::Close)
            }
            (ScreenFocus::Bar, Some(tab)) if self.tab_active[tab] => {
                // A tab button makes no press sound of its own; choosing another tab does.
                self.press_tab(tab);
                None
            }
            (ScreenFocus::List, _) if self.selected == 1 => {
                if let Some(lists) = &mut self.lists {
                    lists.items.press_focused(&self.stats);
                }
                None
            }
            _ => None,
        }
    }

    /// `ContainerEventHandler.handleTabNavigation`: the child after the focused one that takes
    /// the focus; when there is none the focus is cleared and the search starts again.
    fn tab_focus(&mut self, forward: bool) {
        if self.focus == ScreenFocus::Bar && self.focused_tab.is_none() && self.take_bar() {
            return;
        }
        // A list the arrows let go of still has the screen's focus, and takes it back by Tab.
        if self.focus == ScreenFocus::List && !self.list_focused() && self.take_list(forward) {
            return;
        }
        if self.focus == ScreenFocus::List && self.list_step(forward) {
            return;
        }
        let at = ORDER.iter().position(|child| *child == self.focus);
        if self.focus_beyond(at, forward) {
            return;
        }
        self.clear_focus();
        self.focus_beyond(None, forward);
    }

    /// The first child after (or before) position `at` that takes the focus; with no position the
    /// search starts at the end it goes from.
    fn focus_beyond(&mut self, at: Option<usize>, forward: bool) -> bool {
        let count = ORDER.len();
        let candidates: Vec<usize> = match (at, forward) {
            (Some(at), true) => (at + 1..count).collect(),
            (Some(at), false) => (0..at).rev().collect(),
            (None, true) => (0..count).collect(),
            (None, false) => (0..count).rev().collect(),
        };
        candidates
            .into_iter()
            .any(|index| self.take_focus(ORDER[index], forward))
    }

    /// Whether the child takes the focus, and the focus moves to it if it does.
    fn take_focus(&mut self, child: ScreenFocus, forward: bool) -> bool {
        match child {
            ScreenFocus::Bar => self.focused_tab.is_none() && self.take_bar(),
            ScreenFocus::List => self.take_list(forward),
            ScreenFocus::Done => {
                self.release_focus();
                self.focus = ScreenFocus::Done;
                true
            }
            ScreenFocus::None => false,
        }
    }

    /// `TabNavigationBar.nextFocusPath`: a bar without a focused tab focuses the button of the
    /// tab shown (and that tab stays the current one).
    pub(super) fn take_bar(&mut self) -> bool {
        self.release_focus();
        self.focus = ScreenFocus::Bar;
        self.focused_tab = Some(self.selected);
        true
    }

    /// An unfocused list focuses its selected entry, or the first (the last, going backwards).
    pub(super) fn take_list(&mut self, forward: bool) -> bool {
        let selected = self.selected;
        let Some(lists) = &mut self.lists else {
            return false;
        };
        let took = match selected {
            0 => lists.general.focus_entry(forward),
            1 => lists.items.tab_step(forward),
            2 => lists.mobs.focus_entry(forward),
            _ => false,
        };
        if took {
            self.focused_tab = None;
            self.focus = ScreenFocus::List;
        }
        took
    }

    /// A focused list that has widgets of its own passes Tab on to the next one.
    fn list_step(&mut self, forward: bool) -> bool {
        let selected = self.selected;
        self.lists
            .as_mut()
            .is_some_and(|lists| selected == 1 && lists.items.tab_step(forward))
    }

    /// The focus the screen had lets go (`changeFocus` clears the old path).
    pub(super) fn release_focus(&mut self) {
        self.focused_tab = None;
        if let Some(lists) = &mut self.lists {
            lists.general.unfocus();
            lists.items.unfocus();
            lists.mobs.unfocus();
        }
    }

    /// `Screen.clearFocus`.
    pub(super) fn clear_focus(&mut self) {
        self.release_focus();
        self.focus = ScreenFocus::None;
    }
}
