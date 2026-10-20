//! What keys, characters, the mouse and its wheel do on the structure block's screen.

use super::buttons::{TURNS, update_turns};
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

/// `Screen.isValidCharacterForName`: what a structure name's box takes at the cursor.
fn valid_for_name(current: &str, character: char, cursor: usize) -> bool {
    let colon = current.chars().position(|each| each == ':');
    let slash = current.chars().position(|each| each == '/');
    match character {
        ':' => slash.is_none_or(|slash| cursor <= slash) && colon.is_none(),
        // The reference compares with -1 for a colon that is not there.
        '/' => (cursor as i64) > colon.map_or(-1, |colon| colon as i64),
        '_' | '-' | '.' => true,
        'a'..='z' | '0'..='9' => true,
        _ => false,
    }
}

impl StructureBlockScreen {
    /// `charTyped` into the box that has the focus; the name's box takes only what a name may hold.
    pub fn char_typed(&mut self, character: char) {
        let Some(index) = box_index(self.focus) else {
            return;
        };
        if !self.is_shown(self.focus) {
            return;
        }
        let edit = &mut self.edits[index];
        if self.focus == Slot::Name && !valid_for_name(&edit.value(), character, edit.cursor()) {
            return;
        }
        edit.char_typed(character, &self.font);
    }

    /// `Screen.keyPressed` and then Enter as Done: Escape is Cancel, the focused widget takes its
    /// keys, Tab and the arrows move the focus.
    pub fn key_pressed(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> StructureBlockAction {
        use glfw::Key;
        if key == Key::Escape {
            return self.cancel();
        }
        if let Some(action) = self.focused_key(lang, key, (control, shift), clipboard) {
            return action;
        }
        match key {
            Key::Tab => self.tab(!shift),
            Key::Enter | Key::KpEnter => return self.done(UpdateType::UpdateData),
            _ => {
                if let Some(direction) = direction_of(key) {
                    self.arrow(direction);
                }
            }
        }
        StructureBlockAction::Stay
    }

    /// `onCancel`: the screen closes, and the block is to be given what it had.
    pub fn cancel(&self) -> StructureBlockAction {
        self.cancelled.set(true);
        StructureBlockAction::Cancel(self.initial)
    }

    fn done(&self, update: UpdateType) -> StructureBlockAction {
        StructureBlockAction::Done(Box::new(self.current(update)))
    }

    /// The focused widget's own `keyPressed`: a box edits, a button is pressed by Enter or Space.
    /// `None` when the widget does not take the key.
    fn focused_key(
        &mut self,
        lang: &Lang,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> Option<StructureBlockAction> {
        let Some(index) = box_index(self.focus) else {
            let select = matches!(
                key,
                glfw::Key::Enter | glfw::Key::KpEnter | glfw::Key::Space
            );
            let usable = self.is_shown(self.focus) && self.buttons.is_active(self.focus);
            return (select && usable).then(|| self.press(lang, self.focus, shift));
        };
        if !self.is_shown(self.focus) {
            return None;
        }
        let font = self.font.clone();
        let edit = &mut self.edits[index];
        edit_key(key)
            .is_some_and(|key| edit.press_key(key, control, shift, &font, clipboard))
            .then_some(StructureBlockAction::Stay)
    }

    /// A press of a button: Done, Save, Load and Detect send and close, Cancel closes, a turn sets
    /// the rotation, the others cycle.
    fn press(&mut self, lang: &Lang, slot: Slot, shift: bool) -> StructureBlockAction {
        match slot {
            Slot::Done => return self.done(UpdateType::UpdateData),
            Slot::Cancel => return self.cancel(),
            _ => {}
        }
        if let Some(update) = sends(slot, self.mode()) {
            return self.done(update);
        }
        if !self.buttons.press_cycle(slot, lang, shift) {
            self.turn(slot);
        }
        StructureBlockAction::Stay
    }

    /// A turn button: the block takes its rotation.
    fn turn(&mut self, slot: Slot) {
        if let Some((_, rotation, _)) = TURNS.iter().find(|(each, _, _)| *each == slot) {
            self.rotation = *rotation;
            update_turns(&mut self.buttons.turns, *rotation);
        }
    }

    fn set_focus(&mut self, focus: Slot) {
        if focus != self.focus && box_index(focus).is_some() {
            self.focused_at = Instant::now();
        }
        self.focus = focus;
        self.buttons.set_focus(focus);
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

    /// A press of the primary button at `mouse`: the first widget there that is shown and active
    /// takes the focus, and a box takes the cursor, a button is pressed.
    pub fn clicked(&mut self, lang: &Lang, mouse: (i32, i32), shift: bool) -> StructureBlockAction {
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
            return StructureBlockAction::Stay;
        };
        let slot = ORDER[hit];
        self.set_focus(slot);
        if let Some(index) = box_index(slot) {
            let font = self.font.clone();
            self.edits[index].click_at(mouse.0 - candidates[hit].rectangle.x - 4, &font);
            return StructureBlockAction::Stay;
        }
        self.press(lang, slot, shift)
    }

    /// The wheel: the cycle button under the mouse takes the next value.
    pub fn scrolled(&mut self, lang: &Lang, scroll_y: f64, mouse: (i32, i32)) {
        let over = (f64::from(mouse.0), f64::from(mouse.1));
        let mode = self.mode();
        for slot in CYCLES {
            if shown(slot, mode) && self.buttons.is_over(slot, &self.arena, over) {
                self.buttons.scroll_cycle(slot, lang, scroll_y);
                return;
            }
        }
    }
}

/// The update a button sends in a mode, if it is one of the three that send.
fn sends(slot: Slot, mode: StructureMode) -> Option<UpdateType> {
    match (slot, mode) {
        (Slot::Save, StructureMode::Save) => Some(UpdateType::SaveArea),
        (Slot::Load, StructureMode::Load) => Some(UpdateType::LoadArea),
        (Slot::Detect, StructureMode::Save) => Some(UpdateType::ScanArea),
        _ => None,
    }
}
