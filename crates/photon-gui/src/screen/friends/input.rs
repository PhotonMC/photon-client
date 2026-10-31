//! The friends overlay's input (`FriendsOverlayScreen`, `AddFriendWidget`): the tabs, the box and
//! its button, the keys.
//!
//! Not handled: focus travel with Tab and the arrows, a drag selecting text in the box, and the
//! entries' buttons.

use super::draw::inside;
use super::*;
use crate::FontRuns;
use crate::widgets::edit_box::{Clipboard, edit_key};

impl FriendsOverlay {
    fn set_focus(&mut self, focus: Focus) {
        if focus == Focus::Box && self.focus != Focus::Box {
            self.focused_at = Instant::now();
        }
        self.focus = focus;
    }

    /// Whether the box has the focus: the key that opens the overlay does not close it then.
    pub fn typing(&self) -> bool {
        self.focus == Focus::Box
    }

    /// The panel with its border, which a click outside of closes the screen.
    fn panel_rect(&self) -> (i32, i32, i32, i32) {
        let (left, top, width, height) = self.content();
        (
            left - BORDER,
            top - BORDER,
            width + 2 * BORDER,
            height + 2 * BORDER + 1,
        )
    }

    /// Where the profile name is, which a click copies.
    pub(super) fn name_rect(&self, font: &Font) -> (i32, i32, i32, i32) {
        let (left, top, ..) = self.content();
        let label = font.width_of_runs(&self.texts.profile_label);
        (
            left + INPUT_LEFT + label,
            top + INPUT_TOP + BOX_HEIGHT + PROFILE_TOP,
            font.width(&self.view.profile_name),
            9,
        )
    }

    /// `sendFriendRequest`: the trimmed name, when there is one and the box is usable.
    fn send(&mut self) -> Option<FriendsEvent> {
        if !self.button_active() {
            return None;
        }
        let name = self.edit.value().trim().to_owned();
        crate::ui_sound::play_down_sound();
        Some(FriendsEvent::Send(name))
    }

    /// The text of the box once the request is on its way (`AddFriendWidget.clear`).
    pub fn clear_box(&mut self, font: &Font) {
        self.edit.set_value("", font);
    }

    /// A press of the primary button at `mouse`.
    pub fn clicked(&mut self, mouse: (f64, f64), font: &Font) -> Option<FriendsEvent> {
        let at = (mouse.0.floor() as i32, mouse.1.floor() as i32);
        for tab in [Tab::Friends, Tab::Pending] {
            if inside(at, self.tab_rect(tab)) {
                crate::ui_sound::play_down_sound();
                self.tab = tab;
                self.scroll = 0;
                return None;
            }
        }
        let rect = self.box_rect();
        if inside(at, rect) {
            if self.box_active() {
                self.set_focus(Focus::Box);
                self.edit.click_at(at.0 - (rect.0 + 4), font);
            }
            return None;
        }
        if let Some(asked) = self.entry_pressed(at) {
            crate::ui_sound::play_down_sound();
            return Some(asked);
        }
        if inside(at, self.button_rect()) {
            return self.send();
        }
        if inside(at, self.name_rect(font)) {
            crate::ui_sound::play_down_sound();
            return Some(FriendsEvent::CopyName);
        }
        if !inside(at, self.panel_rect()) {
            return Some(FriendsEvent::Close);
        }
        self.set_focus(Focus::None);
        None
    }

    /// A key. `friends_key` is whether it matches the key that opens the overlay, which closes it
    /// unless the box has the focus.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        (control, shift): (bool, bool),
        friends_key: bool,
        font: &Font,
        clipboard: &mut dyn Clipboard,
    ) -> Option<FriendsEvent> {
        if key == glfw::Key::Escape || (friends_key && !self.typing()) {
            return Some(FriendsEvent::Close);
        }
        if self.focus != Focus::Box || !self.box_active() {
            return None;
        }
        if matches!(key, glfw::Key::Enter | glfw::Key::KpEnter) {
            return self.send();
        }
        if let Some(edit) = edit_key(key) {
            self.edit.key_pressed(edit, control, shift, font, clipboard);
        }
        None
    }

    /// A character typed: the box takes it while it has the focus.
    pub fn char_typed(&mut self, character: char, font: &Font) {
        if self.focus == Focus::Box && self.box_active() {
            self.edit.char_typed(character, font);
        }
    }
}
