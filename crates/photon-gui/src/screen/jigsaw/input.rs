//! What keys, characters, the mouse and its wheel do on the jigsaw block's screen.

use super::*;
use crate::widgets::edit_box::{Clipboard, edit_key};
use crate::widgets::navigation::{Direction, next_in_direction};

/// The way an arrow key moves the focus.
fn direction_of(key: glfw::Key) -> Option<Direction> {
    match key {
        glfw::Key::Right => Some(Direction::Right),
        glfw::Key::Left => Some(Direction::Left),
        glfw::Key::Down => Some(Direction::Down),
        glfw::Key::Up => Some(Direction::Up),
        _ => None,
    }
}

/// What the keyboard handler records of a key: Tab and the arrows say how the focus moved.
fn input_of(key: glfw::Key) -> Option<InputType> {
    match key {
        glfw::Key::Tab => Some(InputType::KeyboardTab),
        glfw::Key::Left | glfw::Key::Right | glfw::Key::Up | glfw::Key::Down => {
            Some(InputType::KeyboardArrow)
        }
        _ => None,
    }
}

impl JigsawScreen {
    /// `charTyped` into the box that has the focus.
    pub fn char_typed(&mut self, character: char) {
        let Some(index) = box_index(self.focus) else {
            return;
        };
        self.edits[index].char_typed(character, &self.font);
        self.update_validity();
    }

    /// `Screen.keyPressed` and then Enter as Done: Escape is Cancel, the focused widget takes its
    /// keys, Tab and the arrows move the focus.
    pub fn key_pressed(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> JigsawAction {
        use glfw::Key;
        if let Some(input) = input_of(key) {
            self.input = input;
        }
        if key == Key::Escape {
            return JigsawAction::Cancel;
        }
        if let Some(action) = self.focused_key(lang, key, (control, shift), clipboard) {
            return action;
        }
        match key {
            Key::Tab => self.tab(!shift),
            Key::Enter | Key::KpEnter if self.done.active => {
                return JigsawAction::Done(Box::new(self.current()));
            }
            _ => {
                if let Some(direction) = direction_of(key) {
                    self.arrow(direction);
                }
            }
        }
        JigsawAction::Stay
    }

    /// The focused widget's own `keyPressed`: a box edits, the slider moves, a button is pressed by
    /// Enter or Space. `None` when the widget does not take the key.
    fn focused_key(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> Option<JigsawAction> {
        if let Some(index) = box_index(self.focus) {
            let font = self.font.clone();
            let taken = edit_key(key).is_some_and(|key| {
                self.edits[index].press_key(key, control, shift, &font, clipboard)
            });
            self.update_validity();
            return taken.then_some(JigsawAction::Stay);
        }
        if self.focus == Slot::Levels {
            return self
                .slider
                .key_pressed(lang, &self.arena, key)
                .then_some(JigsawAction::Stay);
        }
        let select = matches!(
            key,
            glfw::Key::Enter | glfw::Key::KpEnter | glfw::Key::Space
        );
        (select && self.focusable(self.focus)).then(|| self.press(lang, self.focus, shift))
    }

    /// A press of a button: Done and Generate send and close (the first when valid), Cancel
    /// closes, the cycles take the next value.
    fn press(&mut self, lang: &Lang, slot: Slot, shift: bool) -> JigsawAction {
        if !matches!(slot, Slot::Levels) && box_index(slot).is_none() {
            crate::ui_sound::play_down_sound();
        }
        match slot {
            Slot::Done => JigsawAction::Done(Box::new(self.current())),
            Slot::Generate => JigsawAction::Generate(
                Box::new(self.current()),
                self.slider.levels(),
                *self.keep.value(),
            ),
            Slot::Cancel => JigsawAction::Cancel,
            Slot::Joint => {
                self.joint.press(lang, shift);
                JigsawAction::Stay
            }
            Slot::Keep => {
                self.keep.press(lang, shift);
                JigsawAction::Stay
            }
            _ => JigsawAction::Stay,
        }
    }

    fn set_focus(&mut self, focus: Slot) {
        if focus != self.focus && box_index(focus).is_some() {
            self.focused_at = Instant::now();
        }
        self.focus = focus;
        self.joint.set_focused(focus == Slot::Joint);
        self.keep.set_focused(focus == Slot::Keep);
        self.slider.set_focused(focus == Slot::Levels, self.input);
    }

    /// Tab (and Shift+Tab): the focus moves to the next widget that can take it, wrapping round.
    fn tab(&mut self, forward: bool) {
        let candidates = self.candidates();
        let count = ORDER.len();
        let place = ORDER
            .iter()
            .position(|each| *each == self.focus)
            .unwrap_or(if forward { count - 1 } else { 0 });
        for step in 1..=count {
            let index = if forward {
                (place + step) % count
            } else {
                (place + count - step % count) % count
            };
            if candidates[index].focusable {
                self.set_focus(ORDER[index]);
                return;
            }
        }
    }

    /// An arrow key the focused widget did not take: the focus goes to the widget beside it.
    fn arrow(&mut self, direction: Direction) {
        let candidates = self.candidates();
        let focused = ORDER.iter().position(|each| *each == self.focus);
        let container = Rectangle {
            x: 0,
            y: 0,
            width: self.size.0,
            height: self.size.1,
        };
        if let Some(next) = next_in_direction(&candidates, focused, direction, container) {
            self.set_focus(ORDER[next]);
        }
    }

    /// A press of the primary button at `mouse`: the first widget there that can be clicked takes
    /// the focus; a box takes the cursor, the slider the handle, a button is pressed.
    pub fn clicked(&mut self, lang: &Lang, mouse: (i32, i32), shift: bool) -> JigsawAction {
        self.input = InputType::Mouse;
        let candidates = self.candidates();
        let hit = candidates.iter().position(|candidate| {
            let rect = candidate.rectangle;
            candidate.focusable
                && mouse.0 >= rect.x
                && mouse.1 >= rect.y
                && mouse.0 < rect.x + rect.width
                && mouse.1 < rect.y + rect.height
        });
        let Some(hit) = hit else {
            return JigsawAction::Stay;
        };
        let slot = ORDER[hit];
        if slot == Slot::Levels {
            self.slider.clicked(lang, &self.arena, mouse.0);
        }
        self.set_focus(slot);
        if let Some(index) = box_index(slot) {
            let font = self.font.clone();
            self.edits[index].click_at(mouse.0 - candidates[hit].rectangle.x - 4, &font);
            return JigsawAction::Stay;
        }
        self.press(lang, slot, shift)
    }

    /// The mouse moved with the primary button down: the slider follows it.
    pub fn dragged(&mut self, lang: &Lang, mouse: (i32, i32)) {
        if self.focus == Slot::Levels {
            self.slider.dragged(lang, &self.arena, mouse.0);
        }
    }

    /// The primary button let go.
    pub fn released(&mut self) {
        self.slider.released();
    }

    /// The wheel: the cycle button under the mouse takes the next value.
    pub fn scrolled(&mut self, lang: &Lang, scroll_y: f64, mouse: (i32, i32)) {
        let over = (f64::from(mouse.0), f64::from(mouse.1));
        if self.joint.is_mouse_over(&self.arena, over) {
            self.joint.scrolled(lang, scroll_y);
        } else if self.keep.is_mouse_over(&self.arena, over) {
            self.keep.scrolled(lang, scroll_y);
        }
    }
}
