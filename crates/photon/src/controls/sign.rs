use super::*;

use photon_gui::screen::sign_edit::{SignBoard, SignEditScreen};

/// The sign the player is writing on: its screen, its block and the face.
pub struct SignEdit {
    pub screen: SignEditScreen,
    pub pos: (i32, i32, i32),
    pub front: bool,
}

/// `AbstractSignEditScreen.isValid`'s reach: `Player.isWithinBlockInteractionRange` with four
/// blocks to spare, from the eyes to the nearest point of the block.
fn within_reach(session: &PlaySession, (x, y, z): (i32, i32, i32)) -> bool {
    let Some(player) = session.player() else {
        return false;
    };
    let eye = crate::view::eye_position(player, 1.0);
    let range = photon_game::input::pick::interaction_ranges(session.world()).0 + 4.0;
    let axis = |at: f64, low: i32| (f64::from(low) - at).max(at - f64::from(low + 1)).max(0.0);
    let (dx, dy, dz) = (axis(eye.x, x), axis(eye.y, y), axis(eye.z, z));
    dx * dx + dy * dy + dz * dz < range * range
}

impl Controls {
    /// `handleOpenSignEditor` and `LocalPlayer.openTextEdit`: the screen of the sign's face opens
    /// over the game and the mouse is let go.
    pub fn open_sign_editor(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some((pos, front)) = session.take_sign_editor() else {
            return;
        };
        let Some(sign) = session.world().sign_view(pos) else {
            return;
        };
        let name = sign.block.trim_start_matches("minecraft:");
        let wood = ["_wall_hanging_sign", "_hanging_sign", "_wall_sign", "_sign"]
            .iter()
            .find_map(|suffix| name.strip_suffix(suffix))
            .unwrap_or(name);
        let board = if sign.hanging {
            SignBoard::Hanging
        } else if name.ends_with("_wall_sign") {
            SignBoard::Wall
        } else {
            SignBoard::Standing
        };
        let (front_text, back_text) = photon_game::sign::sign_texts(sign.tag);
        let text = if front { front_text } else { back_text };
        // The player's own text is not filtered: `isTextFilteringEnabled` is an online account's.
        let messages = text
            .messages(false)
            .clone()
            .map(|line| assets.lang.plain(&line));
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.sign_edit = Some(SignEdit {
            screen: SignEditScreen::new(
                &assets.lang,
                (board, wood),
                messages,
                (text.color, text.has_glowing_text),
                size,
                None,
            ),
            pos,
            front,
        });
        self.chat = None;
        self.pause = None;
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
        let (width, height) = surface.window_size();
        surface.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0);
    }

    /// `AbstractSignEditScreen.tick`: the screen closes once the sign is gone or out of reach.
    pub fn sign_tick(&mut self, session: &mut PlaySession, surface: &mut WindowSurface) -> Result {
        let Some(edit) = &self.sign_edit else {
            return Ok(());
        };
        if session.world().sign_view(edit.pos).is_none() || !within_reach(session, edit.pos) {
            self.close_sign(session, surface)?;
        }
        Ok(())
    }

    /// `removed`: the lines go to the server and the mouse is grabbed again.
    fn close_sign(&mut self, session: &mut PlaySession, surface: &mut WindowSurface) -> Result {
        if let Some(edit) = self.sign_edit.take() {
            session.sign_update(edit.pos, edit.front, edit.screen.messages())?;
        }
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
        Ok(())
    }

    /// An event for the sign's screen.
    pub(super) fn sign_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Modifiers, WindowEvent};
        let Some(edit) = self.sign_edit.as_mut() else {
            return Ok(());
        };
        let font = &assets.creative.font;
        let done = match event {
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let control = modifiers.intersects(Modifiers::Control | Modifiers::Super);
                let shift = modifiers.contains(Modifiers::Shift);
                edit.screen
                    .key_pressed(font, key, (control, shift), &mut WindowClipboard(surface))
            }
            WindowEvent::Char(character) => {
                edit.screen.char_typed(font, character);
                false
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                false
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => edit
                .screen
                .clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32)),
            _ => false,
        };
        if done {
            self.close_sign(session, surface)?;
        }
        Ok(())
    }
}
