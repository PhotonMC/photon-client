use super::*;

impl LanguageSelectScreen {
    /// `ObjectSelectionList.nextFocusPath`: the entry the focus would go to.
    /// With an entry focused an arrow that finds no next one drops both the
    /// focus and the selection, as the reference does while asking.
    pub(super) fn list_next(&mut self, nav: Nav, change: bool) -> Option<usize> {
        if self.shown.is_empty() {
            return None;
        }
        if self.entry_focused {
            let Nav::Arrow(direction) = nav else {
                return None;
            };
            let next = self.next_entry(direction, self.selected_index());
            if next.is_none() && change {
                self.entry_focused = false;
                self.selected = None;
            }
            return next;
        }
        let vertical = match nav {
            Nav::Tab(forward) => {
                if forward {
                    Direction::Down
                } else {
                    Direction::Up
                }
            }
            Nav::Arrow(direction @ (Direction::Up | Direction::Down)) => direction,
            Nav::Arrow(_) => Direction::Down,
        };
        self.selected_index()
            .or_else(|| self.next_entry(vertical, None))
    }

    /// What the focused widget itself offers (`focus.nextFocusPath`).
    pub(super) fn focused_next(&mut self, nav: Nav) -> Option<Target> {
        match self.focus {
            Some(Focus::List) => self.list_next(nav, true).map(Target::Entry),
            _ => None,
        }
    }

    /// `handleTabNavigation` of the screen: the title, search box, list, the
    /// warning, the font button and Done in order; the texts take nothing.
    pub(super) fn root_tab(&mut self, forward: bool) -> Option<Target> {
        let order = [Focus::Search, Focus::List, Focus::Fonts, Focus::Done];
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
                Focus::List => self.list_next(Nav::Tab(forward), true).map(Target::Entry),
                Focus::Fonts => self.fonts.is_active().then_some(Target::Fonts),
                Focus::Done => self.done.is_active().then_some(Target::Done),
            };
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// `handleArrowNavigation` of the screen: the widget that lies next to
    /// the focused one in the direction.
    pub(super) fn root_arrow(&mut self, direction: Direction) -> Option<Target> {
        let list_entry = if self.focus == Some(Focus::List) {
            None
        } else {
            self.list_next(Nav::Arrow(direction), false)
        };
        let rectangle = |rect: usize| {
            let rect = self.arena.0[rect];
            Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        };
        let children = [
            (rectangle(self.title.rect), false),
            (rectangle(self.search_rect), true),
            (
                Rectangle {
                    x: 0,
                    y: self.place.y,
                    width: self.place.width,
                    height: self.place.height,
                },
                list_entry.is_some(),
            ),
            (rectangle(self.warning.rect), false),
            (rectangle(self.fonts.rect), self.fonts.is_active()),
            (rectangle(self.done.rect), self.done.is_active()),
        ]
        .map(|(rectangle, focusable)| Candidate {
            rectangle,
            focusable,
        });
        let focused = self.focus.map(|focus| match focus {
            Focus::Search => 1,
            Focus::List => 2,
            Focus::Fonts => 4,
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
                2 => Target::Entry(list_entry?),
                4 => Target::Fonts,
                _ => Target::Done,
            },
        )
    }

    /// `changeFocus`: what had the focus lets go, and the target takes it
    /// (an entry becomes the selected one, and the list shows it).
    pub(super) fn apply(&mut self, target: Target) {
        match target {
            Target::Search => self.set_focus(Some(Focus::Search)),
            Target::Fonts => self.set_focus(Some(Focus::Fonts)),
            Target::Done => self.set_focus(Some(Focus::Done)),
            Target::Entry(position) => {
                self.set_focus(Some(Focus::List));
                self.entry_focused = true;
                self.selected = Some(self.languages[self.shown[position]].0.clone());
                // A keyboard always shows the entry (`setSelected`).
                self.scroll_to_entry(position);
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
        if target.is_none() && matches!(nav, Nav::Tab(_)) {
            // Past the last widget the focus is cleared and goes round.
            self.entry_focused = false;
            self.focus = None;
            if let Nav::Tab(forward) = nav {
                target = self.root_tab(forward);
            }
        }
        if let Some(target) = target {
            self.apply(target);
        }
    }

    /// A character typed: the search box takes it, and the list shows what
    /// matches.
    pub fn char_typed(&mut self, font: &Font, character: char) {
        if self.focus == Some(Focus::Search) && self.search.char_typed(character, font) {
            self.filter();
        }
    }

    /// A key: the focused widget takes it (the search box edits, an entry of
    /// the list is chosen by Enter, Space), or Tab and the arrows move the
    /// focus.
    pub fn key_pressed(
        &mut self,
        font: &Font,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> LanguageInput {
        use glfw::Key;
        let selection = matches!(key, Key::Enter | Key::KpEnter | Key::Space);
        match (self.focus, key) {
            (Some(Focus::Search), key) if edit_key(key).is_some() => {
                let before = self.search.value();
                if let Some(edit) = edit_key(key) {
                    self.search
                        .key_pressed(edit, control, shift, font, clipboard);
                }
                if self.search.value() != before {
                    self.filter();
                }
                return LanguageInput::default();
            }
            (Some(Focus::List), _) if selection && self.entry_focused => return self.finish(),
            (Some(Focus::Done), _) if selection => {
                crate::ui_sound::play_down_sound();
                return self.finish();
            }
            _ => {}
        }
        match key {
            Key::Tab => self.navigate(Nav::Tab(!shift)),
            Key::Left => self.navigate(Nav::Arrow(Direction::Left)),
            Key::Right => self.navigate(Nav::Arrow(Direction::Right)),
            Key::Up => self.navigate(Nav::Arrow(Direction::Up)),
            Key::Down => self.navigate(Nav::Arrow(Direction::Down)),
            _ => {}
        }
        LanguageInput::default()
    }

    /// `filterEntries`: the languages whose name or region holds the text,
    /// in any case; nothing is selected after it.
    pub(super) fn filter(&mut self) {
        let text = self.search.value().to_lowercase();
        self.shown = self
            .languages
            .iter()
            .enumerate()
            .filter(|(_, (_, info))| {
                text.is_empty()
                    || info.name.to_lowercase().contains(&text)
                    || info.region.to_lowercase().contains(&text)
            })
            .map(|(index, _)| index)
            .collect();
        self.selected = None;
        self.set_scroll(self.scroll);
    }
}
