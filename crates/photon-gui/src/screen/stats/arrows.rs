//! The arrow keys on the statistics screen (`Screen.keyPressed`, `handleArrowNavigation`): left
//! and right between the tabs and inside the items list, up and down between the bar, the list
//! and Done and from entry to entry.

use super::navigation::Arrow;
use super::{ScreenFocus, StatsScreen};

impl StatsScreen {
    /// An arrow: the child with the focus is asked first, then the screen looks for the child
    /// that lies in that direction.
    pub(super) fn arrow_focus(&mut self, arrow: Arrow) {
        let moved = match self.focus {
            ScreenFocus::Bar => self.bar_arrow(arrow),
            ScreenFocus::List => self.list_arrow(arrow),
            ScreenFocus::Done => false,
            ScreenFocus::None => return self.arrow_from_nothing(arrow),
        };
        if !moved {
            self.arrow_between_children(arrow);
        }
    }

    /// The bar without a focused tab focuses the current one; with one, left and right go to the
    /// next tab that can be chosen (which is then shown, with its sound).
    fn bar_arrow(&mut self, arrow: Arrow) -> bool {
        let Some(tab) = self.focused_tab else {
            return self.take_bar();
        };
        let step: isize = match arrow {
            Arrow::Left => -1,
            Arrow::Right => 1,
            Arrow::Up | Arrow::Down => return false,
        };
        let mut at = tab as isize + step;
        while (0..super::TABS as isize).contains(&at) {
            if self.tab_active[at as usize] {
                self.select_tab(at as usize);
                return true;
            }
            at += step;
        }
        // Nothing to the side: the arrow is spent.
        true
    }

    /// Whether the shown list has the focus inside it.
    pub(super) fn list_focused(&self) -> bool {
        let Some(lists) = &self.lists else {
            return false;
        };
        match self.selected {
            0 => lists.general.is_focused(),
            1 => lists.items.is_focused(),
            _ => lists.mobs.is_focused(),
        }
    }

    /// An arrow in the list: an unfocused one takes the focus; a focused one moves it.
    fn list_arrow(&mut self, arrow: Arrow) -> bool {
        let selected = self.selected;
        let focused = self.list_focused();
        let Some(lists) = &mut self.lists else {
            return false;
        };
        let down = arrow != Arrow::Up;
        match selected {
            1 => lists.items.arrow(arrow),
            _ if !focused => {
                let took = if selected == 0 {
                    lists.general.focus_entry(down)
                } else {
                    lists.mobs.focus_entry(down)
                };
                self.focused_tab = None;
                took
            }
            _ if matches!(arrow, Arrow::Left | Arrow::Right) => {
                if selected == 0 {
                    lists.general.clear();
                } else {
                    lists.mobs.clear();
                }
                true
            }
            0 => lists.general.arrow_step(down),
            _ => lists.mobs.arrow_step(down),
        }
    }

    /// The screen's own search for the child above or below the one that gave the arrow up.
    fn arrow_between_children(&mut self, arrow: Arrow) {
        match (self.focus, arrow) {
            (ScreenFocus::Bar, Arrow::Down) => {
                if !self.take_list_by_arrow(arrow) {
                    self.take_focus_done();
                }
            }
            (ScreenFocus::List, Arrow::Down) => self.take_focus_done(),
            // The rectangles put Done to the right of an item's slot.
            (ScreenFocus::List, Arrow::Right) if self.selected == 1 => self.take_focus_done(),
            (ScreenFocus::List, Arrow::Up) => {
                self.release_focus();
                self.take_bar();
            }
            (ScreenFocus::Done, Arrow::Up) => {
                let _ = self.take_list_by_arrow(arrow) || self.take_bar();
            }
            _ => {}
        }
    }

    fn take_focus_done(&mut self) {
        self.release_focus();
        self.focus = ScreenFocus::Done;
    }

    /// The list takes the focus from outside by an up or down arrow.
    fn take_list_by_arrow(&mut self, arrow: Arrow) -> bool {
        self.release_focus();
        let focus = self.focus;
        self.focus = ScreenFocus::List;
        if self.list_arrow(arrow) {
            return true;
        }
        self.focus = focus;
        false
    }

    /// No child has the focus: down starts at the top, up at the bottom.
    fn arrow_from_nothing(&mut self, arrow: Arrow) {
        match arrow {
            Arrow::Down => {
                self.take_bar();
            }
            Arrow::Up => self.take_focus_done(),
            Arrow::Left | Arrow::Right => {}
        }
    }
}
