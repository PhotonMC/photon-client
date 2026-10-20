//! What keys, characters, the mouse and its wheel do on the test block's screen.

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

impl TestBlockScreen {
    /// `charTyped` into the box when it has the focus and is shown.
    pub fn char_typed(&mut self, character: char) {
        if self.focus == Slot::Message && self.is_shown(Slot::Message) {
            self.edit.char_typed(character, &self.font);
        }
    }

    /// `Screen.keyPressed`: Escape is Cancel, the focused widget takes its keys, Tab and the
    /// arrows move the focus. Enter in the box does nothing.
    pub fn key_pressed(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> TestBlockAction {
        if key == glfw::Key::Escape {
            return TestBlockAction::Cancel;
        }
        if let Some(action) = self.focused_key(lang, key, (control, shift), clipboard) {
            return action;
        }
        if key == glfw::Key::Tab {
            self.tab(!shift);
        } else if let Some(direction) = direction_of(key) {
            self.arrow(direction);
        }
        TestBlockAction::Stay
    }

    /// The focused widget's own `keyPressed`: the box edits, a button is pressed by Enter or
    /// Space. `None` when the widget does not take the key.
    fn focused_key(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> Option<TestBlockAction> {
        if self.focus == Slot::Message {
            if !self.is_shown(Slot::Message) {
                return None;
            }
            let font = self.font.clone();
            return edit_key(key)
                .is_some_and(|key| self.edit.press_key(key, control, shift, &font, clipboard))
                .then_some(TestBlockAction::Stay);
        }
        let select = matches!(
            key,
            glfw::Key::Enter | glfw::Key::KpEnter | glfw::Key::Space
        );
        (select && self.focus != Slot::Nowhere).then(|| self.press(lang, self.focus, shift))
    }

    /// A press of a button: Done sends and closes, Cancel closes, the mode cycles.
    fn press(&mut self, lang: &Lang, slot: Slot, shift: bool) -> TestBlockAction {
        match slot {
            Slot::Done => return TestBlockAction::Done(self.current()),
            Slot::Cancel => return TestBlockAction::Cancel,
            Slot::Mode => drop(self.mode.press(lang, shift)),
            _ => {}
        }
        TestBlockAction::Stay
    }

    fn set_focus(&mut self, focus: Slot) {
        if focus != self.focus && focus == Slot::Message {
            self.focused_at = Instant::now();
        }
        self.focus = focus;
        self.mode.set_focused(focus == Slot::Mode);
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

    /// A press of the primary button at `mouse`: the first widget there that is shown takes the
    /// focus, and the box takes the cursor, a button is pressed.
    pub fn clicked(&mut self, lang: &Lang, mouse: (i32, i32), shift: bool) -> TestBlockAction {
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
            return TestBlockAction::Stay;
        };
        let slot = ORDER[hit];
        self.set_focus(slot);
        if slot == Slot::Message {
            let font = self.font.clone();
            self.edit
                .click_at(mouse.0 - candidates[hit].rectangle.x - 4, &font);
            return TestBlockAction::Stay;
        }
        self.press(lang, slot, shift)
    }

    /// The wheel: the mode button under the mouse takes the next value.
    pub fn scrolled(&mut self, lang: &Lang, scroll_y: f64, mouse: (i32, i32)) {
        let over = (f64::from(mouse.0), f64::from(mouse.1));
        if self.mode.is_mouse_over(&self.arena, over) {
            self.mode.scrolled(lang, scroll_y);
        }
    }
}
