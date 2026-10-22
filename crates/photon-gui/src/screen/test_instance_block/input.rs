//! What keys, characters, the mouse and its wheel do on the test instance block's screen.

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

impl TestInstanceBlockScreen {
    /// `charTyped` into the box that has the focus; a change of the identifier asks the server.
    pub fn char_typed(&mut self, lang: &Lang, character: char) {
        let Some(index) = box_index(self.focus) else {
            return;
        };
        let before = self.edits[0].changes();
        self.edits[index].char_typed(character, &self.font);
        self.after_edit(lang, before);
    }

    /// The identifier's box changed: `updateTestInfo`, as its responder does.
    fn after_edit(&mut self, lang: &Lang, before: u32) {
        for _ in before..self.edits[0].changes() {
            self.update_test_info(lang, false);
        }
    }

    /// `Screen.keyPressed`: Escape closes, the focused widget takes its keys, Tab and the arrows
    /// move the focus. Enter in a box does nothing.
    pub fn key_pressed(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> InstanceOutcome {
        self.keyboard = true;
        if key == glfw::Key::Escape {
            return InstanceOutcome::Close;
        }
        if let Some(outcome) = self.focused_key(lang, key, (control, shift), clipboard) {
            return outcome;
        }
        if key == glfw::Key::Tab {
            self.tab(!shift);
        } else if let Some(direction) = direction_of(key) {
            self.arrow(direction);
        }
        InstanceOutcome::Stay
    }

    /// The focused widget's own `keyPressed`: a box edits, a button is pressed by Enter or Space.
    /// `None` when the widget does not take the key.
    fn focused_key(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> Option<InstanceOutcome> {
        if let Some(index) = box_index(self.focus) {
            let font = self.font.clone();
            let before = self.edits[0].changes();
            let taken = edit_key(key).is_some_and(|key| {
                self.edits[index].press_key(key, control, shift, &font, clipboard)
            });
            self.after_edit(lang, before);
            return taken.then_some(InstanceOutcome::Stay);
        }
        let select = matches!(
            key,
            glfw::Key::Enter | glfw::Key::KpEnter | glfw::Key::Space
        );
        (select && !matches!(self.focus, Slot::Info | Slot::Nowhere) && self.is_usable(self.focus))
            .then(|| self.press(lang, self.focus, shift))
    }

    /// Whether a button can be pressed: the save button waits for a name and no rotation.
    fn is_usable(&self, slot: Slot) -> bool {
        slot != Slot::Save || self.save.active
    }

    /// A press of a button: Reset, Save and Run send and close, Done sends what was set and
    /// closes, Cancel closes, the cycle buttons take the next value.
    fn press(&mut self, lang: &Lang, slot: Slot, shift: bool) -> InstanceOutcome {
        crate::ui_sound::play_down_sound();
        let action = match slot {
            Slot::Reset => InstanceAction::Reset,
            Slot::Save => InstanceAction::Save,
            Slot::Run => InstanceAction::Run,
            Slot::Done => InstanceAction::Set,
            Slot::Cancel => return InstanceOutcome::Close,
            Slot::Rotation => {
                self.rotation.press(lang, shift);
                self.update_save_state();
                return InstanceOutcome::Stay;
            }
            Slot::Entities => {
                self.entities.press(lang, shift);
                return InstanceOutcome::Stay;
            }
            _ => return InstanceOutcome::Stay,
        };
        self.send(action);
        InstanceOutcome::Close
    }

    fn set_focus(&mut self, focus: Slot) {
        if focus != self.focus && focus == Slot::Id {
            self.focused_at = Instant::now();
        }
        self.focus = focus;
        self.rotation.set_focused(focus == Slot::Rotation);
        self.entities.set_focused(focus == Slot::Entities);
    }

    /// Tab (and Shift+Tab): the focus moves to the next widget that can take it, wrapping round.
    fn tab(&mut self, forward: bool) {
        let candidates = self.candidates();
        let count = ORDER.len();
        let place = ORDER.iter().position(|each| *each == self.focus);
        let place = place.unwrap_or(if forward { count - 1 } else { 0 });
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

    /// A press of the primary button at `mouse`: the first widget there that can take it takes
    /// the focus; a box takes the cursor, a button is pressed.
    pub fn clicked(&mut self, lang: &Lang, mouse: (i32, i32), shift: bool) -> InstanceOutcome {
        self.keyboard = false;
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
            return InstanceOutcome::Stay;
        };
        let slot = ORDER[hit];
        self.set_focus(slot);
        if let Some(index) = box_index(slot) {
            let font = self.font.clone();
            self.edits[index].click_at(mouse.0 - candidates[hit].rectangle.x - 4, &font);
            return InstanceOutcome::Stay;
        }
        if slot == Slot::Info {
            return InstanceOutcome::Stay;
        }
        self.press(lang, slot, shift)
    }

    /// The wheel: the cycle button under the mouse takes the next value.
    pub fn scrolled(&mut self, lang: &Lang, scroll_y: f64, mouse: (i32, i32)) {
        let over = (f64::from(mouse.0), f64::from(mouse.1));
        if self.rotation.is_mouse_over(&self.arena, over) {
            self.rotation.scrolled(lang, scroll_y);
            self.update_save_state();
        } else if self.entities.is_mouse_over(&self.arena, over) {
            self.entities.scrolled(lang, scroll_y);
        }
    }
}
