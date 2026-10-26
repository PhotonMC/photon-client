use super::*;
use crate::widgets::navigation::{Candidate, Direction, Rectangle, next_in_direction};

/// `FocusNavigationEvent`: Tab (forward or back) or an arrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Nav {
    Tab(bool),
    Arrow(Direction),
}

/// Where the focus goes: an entry is named by its list and its place (0 is the heading).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    Search,
    Entry(usize, usize),
    Folder,
    Done,
}

impl PackScreen {
    /// The place of the list's selected entry (0 for the heading).
    fn selected_index(&self, list: usize) -> Option<usize> {
        match self.selected[list].as_ref()? {
            Selected::Header => Some(0),
            Selected::Pack(id) => self
                .shown(list)
                .iter()
                .position(|view| view.id == *id)
                .map(|place| place + 1),
        }
    }

    fn entry_focused(&self, list: usize) -> bool {
        self.focus == Some(Focus::List(list)) && self.selected[list].is_some()
    }

    /// `nextEntry`: the entry above or below one, or the first or last with none.
    fn next_entry(&self, list: usize, direction: Direction, start: Option<usize>) -> Option<usize> {
        let count = self.shown(list).len() + 1;
        let delta: isize = match direction {
            Direction::Up => -1,
            Direction::Down => 1,
            Direction::Left | Direction::Right => return None,
        };
        let index = match start {
            Some(start) => start as isize + delta,
            None if delta > 0 => 0,
            None => count as isize - 1,
        };
        (index >= 0 && (index as usize) < count).then_some(index as usize)
    }

    /// `ObjectSelectionList.nextFocusPath`; an arrow past the end drops focus and selection.
    fn list_next(&mut self, list: usize, nav: Nav, change: bool) -> Option<usize> {
        if self.entry_focused(list) {
            let Nav::Arrow(direction) = nav else {
                return None;
            };
            let next = self.next_entry(list, direction, self.selected_index(list));
            if next.is_none() && change {
                self.selected[list] = None;
            }
            return next;
        }
        let vertical = match nav {
            Nav::Tab(true) | Nav::Arrow(Direction::Down) => Direction::Down,
            Nav::Tab(false) | Nav::Arrow(Direction::Up) => Direction::Up,
            Nav::Arrow(_) => Direction::Down,
        };
        self.selected_index(list)
            .or_else(|| self.next_entry(list, vertical, None))
    }

    fn focused_next(&mut self, nav: Nav) -> Option<Target> {
        match self.focus {
            Some(Focus::List(list)) => self
                .list_next(list, nav, true)
                .map(|place| Target::Entry(list, place)),
            _ => None,
        }
    }

    /// `handleTabNavigation`: the search box, both lists, Open Folder and Done in order.
    fn root_tab(&mut self, forward: bool) -> Option<Target> {
        let order = [
            Focus::Search,
            Focus::List(0),
            Focus::List(1),
            Focus::Folder,
            Focus::Done,
        ];
        let at = self
            .focus
            .and_then(|focus| order.iter().position(|candidate| *candidate == focus));
        let start = match (at, forward) {
            (Some(at), true) => at + 1,
            (Some(at), false) => at,
            (None, true) => 0,
            (None, false) => order.len(),
        };
        let visit: Vec<Focus> = if forward {
            order[start..].to_vec()
        } else {
            order[..start].iter().rev().copied().collect()
        };
        for focus in visit {
            let found = match focus {
                Focus::Search => Some(Target::Search),
                Focus::List(list) => self
                    .list_next(list, Nav::Tab(forward), true)
                    .map(|place| Target::Entry(list, place)),
                Focus::Folder => self.open_folder.is_active().then_some(Target::Folder),
                Focus::Done => self.done.is_active().then_some(Target::Done),
            };
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// `handleArrowNavigation`: the widget that lies next to the focused one in the direction.
    fn root_arrow(&mut self, direction: Direction) -> Option<Target> {
        let entries: [Option<usize>; 2] = [0, 1].map(|list| {
            if self.focus == Some(Focus::List(list)) {
                None
            } else {
                self.list_next(list, Nav::Arrow(direction), false)
            }
        });
        let rectangle = |rect: usize| {
            let rect = self.arena.0[rect];
            Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        };
        let place = self.place();
        let list_rectangle = |list: usize| Rectangle {
            x: self.list_x(list),
            y: place.y,
            width: LIST_WIDTH,
            height: place.height,
        };
        let children = [
            (rectangle(self.title.rect), false),
            (rectangle(self.search_rect), true),
            (list_rectangle(0), entries[0].is_some()),
            (list_rectangle(1), entries[1].is_some()),
            (
                rectangle(self.open_folder.rect),
                self.open_folder.is_active(),
            ),
            (rectangle(self.done.rect), self.done.is_active()),
        ]
        .map(|(rectangle, focusable)| Candidate {
            rectangle,
            focusable,
        });
        let focused = self.focus.map(|focus| match focus {
            Focus::Search => 1,
            Focus::List(list) => 2 + list,
            Focus::Folder => 4,
            Focus::Done => 5,
        });
        let container = Rectangle {
            x: 0,
            y: 0,
            width: self.size.0,
            height: self.size.1,
        };
        Some(
            match next_in_direction(&children, focused, direction, container)? {
                1 => Target::Search,
                index @ (2 | 3) => Target::Entry(index - 2, entries[index - 2]?),
                4 => Target::Folder,
                _ => Target::Done,
            },
        )
    }

    /// `changeFocus`: the target takes the focus; an entry becomes its list's selection.
    fn apply(&mut self, target: Target) {
        match target {
            Target::Search => {
                if self.focus != Some(Focus::Search) {
                    self.searched_at = Instant::now();
                }
                self.focus = Some(Focus::Search);
            }
            Target::Folder => self.focus = Some(Focus::Folder),
            Target::Done => self.focus = Some(Focus::Done),
            Target::Entry(list, place) => {
                self.focus = Some(Focus::List(list));
                let entry = match place.checked_sub(1) {
                    None => Selected::Header,
                    Some(index) => Selected::Pack(self.shown(list)[index].id.clone()),
                };
                self.set_selected(list, entry);
            }
        }
    }

    /// `Screen.keyPressed`'s navigation: Tab and the arrows.
    pub(super) fn navigate(&mut self, nav: Nav) {
        let mut target = self.focused_next(nav);
        if target.is_none() {
            target = match nav {
                Nav::Tab(forward) => self.root_tab(forward),
                Nav::Arrow(direction) => self.root_arrow(direction),
            };
        }
        if target.is_none()
            && let Nav::Tab(forward) = nav
        {
            // Past the last widget the focus is cleared and goes round.
            self.focus = None;
            target = self.root_tab(forward);
        }
        if let Some(target) = target {
            self.apply(target);
        }
    }
}
