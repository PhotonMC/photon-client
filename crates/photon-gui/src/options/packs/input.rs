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

impl PackScreen {
    /// The pack entry of a list under `at`, with the pointer relative to its content.
    pub(super) fn entry_at(&self, list: usize, at: (i32, i32)) -> Option<(&PackView, (i32, i32))> {
        let place = self.place();
        let x = self.list_x(list);
        if at.0 < x || at.0 >= x + LIST_WIDTH || at.1 < place.y || at.1 >= place.bottom() {
            return None;
        }
        let width = ROW_WIDTH
            - if self.scrollable(list) {
                SCROLLBAR_WIDTH
            } else {
                0
            };
        let left = x + (LIST_WIDTH - ROW_WIDTH) / 2;
        if at.0 < left || at.0 >= left + width {
            return None;
        }
        self.shown(list)
            .into_iter()
            .enumerate()
            .find_map(|(index, view)| {
                let top = self.entry_y(list, index + 1);
                (at.1 >= top && at.1 < top + ENTRY_HEIGHT)
                    .then_some((view, (at.0 - (left + PADDING), at.1 - (top + PADDING))))
            })
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
        self.after_change();
    }

    fn after_change(&mut self) {
        self.update_done();
        for list in 0..2 {
            self.set_scroll(list, self.scroll[list]);
        }
    }

    fn press_part(&mut self, id: &str, part: Part) -> PackInput {
        match part {
            Part::Select => return self.select(id),
            Part::Unselect => self.model.toggle(id),
            Part::Up => self.model.move_up(id),
            Part::Down => self.model.move_down(id),
        }
        self.after_change();
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
            if !self.search_focused {
                self.searched_at = Instant::now();
            }
            self.search_focused = true;
            self.search.click_at(at.0 - (search.x + 4), font);
        }
        inside
    }

    /// A press over a list: its bar is taken, or an entry's icon pressed. None when the press
    /// was not over a list.
    fn click_lists(&mut self, mouse: (f64, f64)) -> Option<PackInput> {
        let at = (mouse.0 as i32, mouse.1 as i32);
        let place = self.place();
        let list = (0..2).find(|&list| {
            let x = f64::from(self.list_x(list));
            mouse.0 >= x && mouse.0 < x + f64::from(LIST_WIDTH) && place.contains((0.0, mouse.1))
        })?;
        self.search_focused = false;
        let bar = f64::from(self.bar_x(list));
        if self.scrollable(list) && mouse.0 >= bar && mouse.0 <= bar + f64::from(SCROLLBAR_WIDTH) {
            self.scrolling = Some(list);
            return Some(PackInput::default());
        }
        let hit = self
            .entry_at(list, at)
            .and_then(|(view, rel)| Some((view.id.clone(), self.part_at(view, rel)?)));
        Some(match hit {
            Some((id, part)) => self.press_part(&id, part),
            None => PackInput::default(),
        })
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

    pub fn char_typed(&mut self, font: &Font, character: char) {
        if self.search_focused {
            self.search.char_typed(character, font);
            self.after_change();
        }
    }

    pub fn key_pressed(
        &mut self,
        font: &Font,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) {
        if self.search_focused
            && let Some(edit) = edit_key(key)
        {
            self.search
                .key_pressed(edit, control, shift, font, clipboard);
            self.after_change();
        }
    }
}
