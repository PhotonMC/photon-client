use super::*;
use crate::widgets::edit_box::{Clipboard, edit_key};
use crate::widgets::scroll_area::SCROLLBAR_WIDTH;

/// `SelectableEntry`: the parts of the 32 by 32 icon a press means something in.
fn over_icon((x, y): (i32, i32)) -> bool {
    (0..ICON).contains(&x) && (0..ICON).contains(&y)
}

fn over_left_half((x, y): (i32, i32)) -> bool {
    (0..ICON / 2).contains(&x) && (0..ICON).contains(&y)
}

fn over_top_right((x, y): (i32, i32)) -> bool {
    (ICON / 2..ICON).contains(&x) && (0..ICON / 2).contains(&y)
}

fn over_bottom_right((x, y): (i32, i32)) -> bool {
    (ICON / 2..ICON).contains(&x) && (ICON / 2..ICON).contains(&y)
}

/// What the pointer is over in an entry's icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    Select,
    Unselect,
    Up,
    Down,
}

/// What a list has under a point.
pub(super) enum Row<'a> {
    Header,
    /// A pack, and the point relative to its content.
    Pack(&'a PackView, (i32, i32)),
}

impl PackScreen {
    /// The row of a list under `at`.
    pub(super) fn row_at(&self, list: usize, at: (i32, i32)) -> Option<Row<'_>> {
        let place = self.place();
        let x = self.list_x(list);
        if at.0 < x || at.0 >= x + LIST_WIDTH || at.1 < place.y || at.1 >= place.bottom() {
            return None;
        }
        let left = x + (LIST_WIDTH - ROW_WIDTH) / 2;
        if at.0 < left || at.0 >= left + self.row_width(list) {
            return None;
        }
        let header = self.entry_y(list, 0);
        if at.1 >= header && at.1 < header + HEADER_ENTRY_HEIGHT {
            return Some(Row::Header);
        }
        self.shown(list)
            .into_iter()
            .enumerate()
            .find_map(|(index, view)| {
                let top = self.entry_y(list, index + 1);
                (at.1 >= top && at.1 < top + ENTRY_HEIGHT).then_some(Row::Pack(
                    view,
                    (at.0 - (left + PADDING), at.1 - (top + PADDING)),
                ))
            })
    }

    /// The pack entry of a list under `at`, with the point relative to its content.
    pub(super) fn entry_at(&self, list: usize, at: (i32, i32)) -> Option<(&PackView, (i32, i32))> {
        match self.row_at(list, at)? {
            Row::Pack(view, rel) => Some((view, rel)),
            Row::Header => None,
        }
    }

    /// The width of a list's rows: the bar takes some of it when the list scrolls.
    pub(super) fn row_width(&self, list: usize) -> i32 {
        ROW_WIDTH
            - if self.scrollable(list) {
                SCROLLBAR_WIDTH
            } else {
                0
            }
    }

    /// The part of the entry's icon the pointer is over, among those the pack offers
    /// (`extractContent`'s order: select, or unselect then up then down).
    pub(super) fn part_at(&self, view: &PackView, rel: (i32, i32)) -> Option<Part> {
        if view.fixed_position && view.required {
            return None;
        }
        let id = &view.id;
        if self.model.can_select(id) {
            return over_icon(rel).then_some(Part::Select);
        }
        if self.model.can_unselect(id) && over_left_half(rel) {
            return Some(Part::Unselect);
        }
        if self.model.can_move_up(id) && over_top_right(rel) {
            return Some(Part::Up);
        }
        if self.model.can_move_down(id) && over_bottom_right(rel) {
            return Some(Part::Down);
        }
        None
    }

    /// `handlePackSelection`: a pack that fits is selected, another is asked about.
    fn select(&mut self, id: &str) -> PackInput {
        let compatibility = self.views.get(id).map(|view| view.compatibility);
        match compatibility {
            Some(compatibility) if !compatibility.is_compatible() => PackInput {
                confirm: Some((id.to_owned(), compatibility)),
                ..PackInput::default()
            },
            _ => {
                self.select_confirmed(id);
                PackInput::default()
            }
        }
    }

    /// The pack is selected (it fits, or the player confirmed).
    pub fn select_confirmed(&mut self, id: &str) {
        self.model.toggle(id);
        self.transferred(id);
    }

    /// `populateLists` with the entry that moved: both lists are made again with nothing selected,
    /// and the list that holds the pack takes the focus and selects it.
    fn transferred(&mut self, id: &str) {
        self.update_done();
        self.selected = [None, None];
        for list in 0..2 {
            self.set_scroll(list, self.scroll[list]);
        }
        let found = (0..2).find(|&list| self.shown(list).iter().any(|view| view.id == id));
        if let Some(list) = found {
            self.focus = Some(Focus::List(list));
            self.set_selected(list, Selected::Pack(id.to_owned()));
        }
    }

    /// `setSelected`: the entry is the list's, and the list scrolls to it when it sticks out.
    fn set_selected(&mut self, list: usize, entry: Selected) {
        let index = match &entry {
            Selected::Header => 0,
            Selected::Pack(id) => {
                1 + self
                    .shown(list)
                    .iter()
                    .position(|view| view.id == *id)
                    .unwrap_or(0)
            }
        };
        self.selected[list] = Some(entry);
        let place = self.place();
        let top = self.entry_y(list, index);
        let height = self.entry_height(index);
        if top + PADDING < place.y || top + height - PADDING > place.bottom() {
            self.scroll_to_entry(list, index);
        }
    }

    fn entry_height(&self, index: usize) -> i32 {
        if index == 0 {
            HEADER_ENTRY_HEIGHT
        } else {
            ENTRY_HEIGHT
        }
    }

    /// `scrollToEntry`.
    fn scroll_to_entry(&mut self, list: usize, index: usize) {
        let place = self.place();
        let top = self.entry_y(list, index);
        let top_delta = top - place.y - PADDING;
        if top_delta < 0 {
            self.set_scroll(list, self.scroll[list] + f64::from(top_delta));
        }
        let top = self.entry_y(list, index);
        let bottom_delta = place.bottom() - top - self.entry_height(index) - PADDING;
        if bottom_delta < 0 {
            self.set_scroll(list, self.scroll[list] - f64::from(bottom_delta));
        }
    }

    fn press_part(&mut self, id: &str, part: Part) -> PackInput {
        match part {
            Part::Select => return self.select(id),
            Part::Unselect => self.model.toggle(id),
            Part::Up => self.model.move_up(id),
            Part::Down => self.model.move_down(id),
        }
        self.transferred(id);
        PackInput::default()
    }

    /// A press of the primary button at `mouse`.
    pub fn clicked(&mut self, font: &Font, mouse: (f64, f64)) -> PackInput {
        let at = (mouse.0 as i32, mouse.1 as i32);
        if self.click_search(font, at) {
            return PackInput::default();
        }
        if let Some(input) = self.click_lists(mouse) {
            return input;
        }
        let mut input = PackInput::default();
        if self.open_folder.is_active() && self.open_folder.contains(&self.arena, at) {
            crate::ui_sound::play_down_sound();
            input.open_folder = true;
        } else if self.done.is_active() && self.done.contains(&self.arena, at) {
            crate::ui_sound::play_down_sound();
            input.done = true;
        }
        input
    }

    /// A press on the search box: it takes the focus. Whether it was there.
    fn click_search(&mut self, font: &Font, at: (i32, i32)) -> bool {
        let search = self.arena.0[self.search_rect];
        let inside = at.0 >= search.x
            && at.0 < search.x + search.width
            && at.1 >= search.y
            && at.1 < search.y + search.height;
        if inside {
            if !self.search_focused() {
                self.searched_at = Instant::now();
            }
            self.focus = Some(Focus::Search);
            self.search.click_at(at.0 - (search.x + 4), font);
        }
        inside
    }

    /// A press over a list: its bar is taken, or a row pressed. None when the press was not over a
    /// list; a press over a list that finds no row changes nothing.
    fn click_lists(&mut self, mouse: (f64, f64)) -> Option<PackInput> {
        let at = (mouse.0 as i32, mouse.1 as i32);
        let place = self.place();
        let list = (0..2).find(|&list| {
            let x = f64::from(self.list_x(list));
            mouse.0 >= x && mouse.0 < x + f64::from(LIST_WIDTH) && place.contains((0.0, mouse.1))
        })?;
        let bar = f64::from(self.bar_x(list));
        if self.scrollable(list) && mouse.0 >= bar && mouse.0 <= bar + f64::from(SCROLLBAR_WIDTH) {
            self.scrolling = Some(list);
            self.focus = Some(Focus::List(list));
            return Some(PackInput::default());
        }
        Some(self.press_row(list, at))
    }

    /// A press on a row of a list (or between rows): the list takes the focus and the row is
    /// selected, or its icon part acts.
    fn press_row(&mut self, list: usize, at: (i32, i32)) -> PackInput {
        let pressed = match self.row_at(list, at) {
            None => return PackInput::default(),
            Some(Row::Header) => None,
            Some(Row::Pack(view, rel)) => Some((view.id.clone(), self.part_at(view, rel))),
        };
        self.focus = Some(Focus::List(list));
        match pressed {
            None => self.set_selected(list, Selected::Header),
            Some((id, None)) => self.set_selected(list, Selected::Pack(id)),
            Some((id, Some(part))) => {
                let input = self.press_part(&id, part);
                if input.confirm.is_some() {
                    self.set_selected(list, Selected::Pack(id));
                }
                return input;
            }
        }
        PackInput::default()
    }

    /// The pointer moved by `dy` with the primary button down.
    pub fn dragged(&mut self, mouse: (f64, f64), dy: f64) {
        if let Some(list) = self.scrolling {
            let to = self.place().dragged_scroll(
                self.scroll[list],
                mouse.1,
                dy,
                self.content_height(list),
            );
            self.set_scroll(list, to);
        }
    }

    pub fn released(&mut self) {
        self.scrolling = None;
    }

    /// The wheel over a list: the step of `AbstractScrollArea` is half the default entry height.
    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) {
        for list in 0..2 {
            let x = f64::from(self.list_x(list));
            if mouse.0 >= x
                && mouse.0 < x + f64::from(LIST_WIDTH)
                && self.place().contains((0.0, mouse.1))
            {
                self.set_scroll(
                    list,
                    self.scroll[list] - scroll_y * f64::from(ENTRY_HEIGHT / 2),
                );
            }
        }
    }

    /// The search text changed: both lists are made again from what matches, with nothing selected
    /// (`updateFilteredEntries`).
    pub(super) fn refilter(&mut self) {
        self.selected = [None, None];
        for list in 0..2 {
            self.set_scroll(list, self.scroll[list]);
        }
    }

    pub fn char_typed(&mut self, font: &Font, character: char) {
        if self.search_focused() && self.search.char_typed(character, font) {
            self.refilter();
        }
    }

    pub fn key_pressed(
        &mut self,
        font: &Font,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) {
        if self.search_focused()
            && let Some(edit) = edit_key(key)
        {
            let before = self.search.value();
            self.search
                .key_pressed(edit, control, shift, font, clipboard);
            if self.search.value() != before {
                self.refilter();
            }
        }
    }
}
