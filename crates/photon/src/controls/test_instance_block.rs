use super::*;

use photon_gui::screen::test_instance_block::{InstanceOutcome, TestInstanceBlockScreen};

/// The test instance block the player is editing: its screen and where the block is.
pub struct TestInstanceBlockEdit {
    pub screen: TestInstanceBlockScreen,
    pub pos: (i32, i32, i32),
    /// Whether the primary button is held, for the drag of the description's bar.
    held: bool,
}

impl Controls {
    /// `Player.openTestInstanceBlock`: the screen of the block just used opens over the game and
    /// the mouse is let go.
    pub fn open_test_instance_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(pos) = session.take_opened_test_instance_block() else {
            return;
        };
        let Some((contents, _)) = session.test_instance_block(pos) else {
            return;
        };
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let screen =
            TestInstanceBlockScreen::new((&assets.creative.font, &*assets.lang), &contents, size);
        self.test_instance_block = Some(TestInstanceBlockEdit {
            screen,
            pos,
            held: false,
        });
        self.chat = None;
        self.pause = None;
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
        let (width, height) = surface.window_size();
        surface.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0);
        if let Err(error) = self.send_test_instance_requests(session) {
            log::warn!("could not send the test instance block request: {error:?}");
        }
    }

    /// `handleTestInstanceBlockStatus`: the screen that is up shows the status; any other
    /// screen ignores it.
    pub fn show_test_instance_statuses(
        &mut self,
        session: &mut PlaySession,
        assets: &ScreenAssets,
    ) {
        let statuses = session.take_test_instance_statuses();
        let Some(edit) = self.test_instance_block.as_mut() else {
            return;
        };
        let error = session
            .test_instance_block(edit.pos)
            .and_then(|(_, error)| error);
        for status in statuses {
            edit.screen
                .set_status(&assets.lang, error.as_ref(), &status.status, status.size);
        }
    }

    /// The packets the screen made since the last look go to the server.
    fn send_test_instance_requests(&mut self, session: &mut PlaySession) -> Result {
        let Some(edit) = self.test_instance_block.as_mut() else {
            return Ok(());
        };
        for request in edit.screen.take_sent() {
            session.test_instance_action(edit.pos, &request)?;
        }
        Ok(())
    }

    /// `onClose`: the mouse is grabbed again.
    fn close_test_instance_block(&mut self, surface: &mut WindowSurface) {
        self.test_instance_block = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// An event for the test instance block's screen.
    pub(super) fn test_instance_block_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Modifiers, MouseButton, WindowEvent};
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse;
            self.move_gui_mouse((x, y), surface);
            if let Some(edit) = self.test_instance_block.as_mut().filter(|edit| edit.held) {
                edit.screen
                    .dragged(self.gui_mouse.1, self.gui_mouse.1 - before.1);
            }
            return Ok(());
        }
        let Some(edit) = self.test_instance_block.as_mut() else {
            return Ok(());
        };
        let lang = &assets.lang;
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let outcome = match event {
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let control = modifiers.intersects(Modifiers::Control | Modifiers::Super);
                let shift = modifiers.contains(Modifiers::Shift);
                edit.screen
                    .key_pressed(lang, key, (control, shift), &mut WindowClipboard(surface))
            }
            WindowEvent::Char(character) => {
                edit.screen.char_typed(lang, character);
                InstanceOutcome::Stay
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                edit.held = true;
                edit.screen
                    .clicked(lang, mouse, modifiers.contains(Modifiers::Shift))
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                edit.held = false;
                edit.screen.released();
                InstanceOutcome::Stay
            }
            WindowEvent::Scroll(_, y) => {
                edit.screen.scrolled(lang, y, mouse);
                InstanceOutcome::Stay
            }
            _ => InstanceOutcome::Stay,
        };
        self.send_test_instance_requests(session)?;
        if outcome == InstanceOutcome::Close {
            self.close_test_instance_block(surface);
        }
        Ok(())
    }
}

impl TestInstanceBlockEdit {
    /// The screen drawn at the size of the window.
    pub fn draw(
        &mut self,
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (
            &photon_render::gui::GuiAtlas,
            &photon_render::gui::font::Font,
            &photon_game::text::Lang,
        ),
        (mouse, size): ((i32, i32), (i32, i32)),
    ) {
        if self.screen.size() != size {
            self.screen.resize(lang, size);
        }
        self.screen.draw((frame, atlas, font), mouse, lang);
    }
}
