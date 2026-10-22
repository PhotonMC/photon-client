//! What keys, characters, the mouse and its wheel do on the command block's screen.

use super::*;
use crate::chat::suggestions::ListKey;
use crate::widgets::edit_box::{Clipboard, edit_key};
use crate::widgets::navigation::{Direction, next_in_direction};

/// The key the list of suggestions answers to.
fn list_key_of(key: glfw::Key, shift: bool) -> Option<ListKey> {
    match key {
        glfw::Key::Up => Some(ListKey::Up),
        glfw::Key::Down => Some(ListKey::Down),
        glfw::Key::Tab => Some(ListKey::Tab { shift }),
        glfw::Key::Escape => Some(ListKey::Escape),
        _ => None,
    }
}

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

impl CommandBlockScreen {
    /// `charTyped` into the box that has the focus.
    pub fn char_typed(&mut self, character: char) {
        if self.focus == Focus::Command && self.command.char_typed(character, &self.font) {
            self.suggest.edited(&mut self.command);
        }
    }

    /// `AbstractCommandBlockEditScreen.keyPressed`: the list of suggestions first, then the
    /// screen (Escape, the focused widget, Tab and the arrows), then Enter as Done.
    pub fn key_pressed(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> CommandBlockAction {
        use glfw::Key;
        if let Some(list_key) = list_key_of(key, shift)
            && self.suggest.list_key(&mut self.command, list_key)
        {
            return CommandBlockAction::Stay;
        }
        if key == Key::Escape {
            return CommandBlockAction::Close;
        }
        if let Some(action) = self.focused_key(lang, key, (control, shift), clipboard) {
            return action;
        }
        match key {
            Key::Tab => self.tab(!shift),
            Key::Enter | Key::KpEnter => return CommandBlockAction::Done(self.edit()),
            _ => {
                if let Some(direction) = direction_of(key) {
                    self.arrow(direction);
                }
            }
        }
        CommandBlockAction::Stay
    }

    /// The focused widget's own `keyPressed`: a box edits, a button is pressed by Enter or Space.
    /// `None` when the widget does not take the key.
    fn focused_key(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> Option<CommandBlockAction> {
        let edit = match self.focus {
            Focus::Command => Some(&mut self.command),
            Focus::Previous => Some(&mut self.previous),
            _ => None,
        };
        let Some(edit) = edit else {
            let select = matches!(
                key,
                glfw::Key::Enter | glfw::Key::KpEnter | glfw::Key::Space
            );
            return (select && self.is_active(self.focus)).then(|| self.press(lang, shift));
        };
        let before = edit.value();
        let taken = edit_key(key)
            .is_some_and(|key| edit.press_key(key, control, shift, &self.font, clipboard));
        if self.focus == Focus::Command && self.command.value() != before {
            self.suggest.edited(&mut self.command);
        }
        taken.then_some(CommandBlockAction::Stay)
    }

    /// Whether the widget can be focused and pressed.
    fn is_active(&self, focus: Focus) -> bool {
        self.candidate_of(focus).focusable
    }

    /// The focused button pressed (`onPress`): Done sends, Cancel closes, the others cycle.
    fn press(&mut self, lang: &Lang, shift: bool) -> CommandBlockAction {
        if !matches!(self.focus, Focus::Command | Focus::Previous) {
            crate::ui_sound::play_down_sound();
        }
        let buttons = &mut self.buttons;
        match self.focus {
            Focus::Output => {
                let tracking = buttons.output.press(lang, shift);
                self.update_previous_output(tracking);
            }
            Focus::Mode | Focus::Conditional | Focus::Automatic => {
                if let Some(extra) = &mut buttons.extra {
                    extra.press(self.focus, lang, shift);
                }
            }
            Focus::Done => return CommandBlockAction::Done(self.edit()),
            Focus::Cancel => return CommandBlockAction::Close,
            Focus::Command | Focus::Previous => {}
        }
        CommandBlockAction::Stay
    }

    fn set_focus(&mut self, focus: Focus) {
        if focus != self.focus && matches!(focus, Focus::Command | Focus::Previous) {
            self.focused_at = Instant::now();
        }
        self.focus = focus;
        self.buttons.output.set_focused(focus == Focus::Output);
        if let Some(extra) = &mut self.buttons.extra {
            extra.mode.set_focused(focus == Focus::Mode);
            extra.conditional.set_focused(focus == Focus::Conditional);
            extra.automatic.set_focused(focus == Focus::Automatic);
        }
    }

    /// Tab (and Shift+Tab): the focus moves to the next widget that can take it, wrapping round.
    fn tab(&mut self, forward: bool) {
        let candidates = self.candidates();
        let count = self.kind.order().len();
        let place = self
            .kind
            .order()
            .iter()
            .position(|each| *each == self.focus)
            .unwrap_or(0);
        for step in 1..=count {
            let index = if forward {
                (place + step) % count
            } else {
                (place + count - step % count) % count
            };
            if candidates[index].focusable {
                self.set_focus(self.kind.order()[index]);
                return;
            }
        }
    }

    /// An arrow key the focused widget did not take: the focus goes to the widget beside it.
    fn arrow(&mut self, direction: Direction) {
        let candidates = self.candidates();
        let focused = self
            .kind
            .order()
            .iter()
            .position(|each| *each == self.focus);
        let container = Rectangle {
            x: 0,
            y: 0,
            width: self.size.0,
            height: self.size.1,
        };
        if let Some(next) = next_in_direction(&candidates, focused, direction, container) {
            self.set_focus(self.kind.order()[next]);
        }
    }

    /// A press of the primary button at `mouse`: the list of suggestions first, then a box takes
    /// the focus and the cursor, or a button is pressed.
    pub fn clicked(&mut self, lang: &Lang, mouse: (i32, i32), shift: bool) -> CommandBlockAction {
        if self.suggest.mouse_clicked(&mut self.command, mouse) {
            return CommandBlockAction::Stay;
        }
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
            return CommandBlockAction::Stay;
        };
        self.set_focus(self.kind.order()[hit]);
        match self.kind.order()[hit] {
            Focus::Command => self
                .command
                .click_at(mouse.0 - candidates[hit].rectangle.x - 4, &self.font),
            Focus::Previous => self
                .previous
                .click_at(mouse.0 - candidates[hit].rectangle.x - 4, &self.font),
            _ => return self.press(lang, shift),
        }
        CommandBlockAction::Stay
    }

    /// The wheel: the list of suggestions scrolls when the mouse is over it, otherwise a cycle
    /// button under the mouse takes the next value.
    pub fn scrolled(&mut self, lang: &Lang, scroll_y: f64, mouse: (i32, i32)) {
        if self.suggest.mouse_scrolled(scroll_y, mouse) {
            return;
        }
        let buttons = &mut self.buttons;
        let arena = &buttons.arena;
        let over = (f64::from(mouse.0), f64::from(mouse.1));
        if buttons.output.is_mouse_over(arena, over) {
            if let Some(tracking) = buttons.output.scrolled(lang, scroll_y) {
                self.update_previous_output(tracking);
            }
        } else if let Some(extra) = &mut buttons.extra {
            extra.scrolled(lang, scroll_y, (arena, over));
        }
    }
}

impl Extra {
    /// A press of the button the focus is on.
    fn press(&mut self, focus: Focus, lang: &Lang, shift: bool) {
        match focus {
            Focus::Mode => drop(self.mode.press(lang, shift)),
            Focus::Conditional => drop(self.conditional.press(lang, shift)),
            _ => drop(self.automatic.press(lang, shift)),
        }
    }

    /// The wheel over one of the buttons.
    fn scrolled(&mut self, lang: &Lang, scroll_y: f64, (arena, over): (&Arena, (f64, f64))) {
        if self.mode.is_mouse_over(arena, over) {
            self.mode.scrolled(lang, scroll_y);
        } else if self.conditional.is_mouse_over(arena, over) {
            self.conditional.scrolled(lang, scroll_y);
        } else if self.automatic.is_mouse_over(arena, over) {
            self.automatic.scrolled(lang, scroll_y);
        }
    }
}
