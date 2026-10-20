use super::*;

use photon_gui::screen::test_block::{TestBlockAction, TestBlockEdit as Done, TestBlockScreen};

/// The test block the player is editing: its screen and where the block is.
pub struct TestBlockEdit {
    pub screen: TestBlockScreen,
    pub pos: (i32, i32, i32),
}

impl Controls {
    /// `Player.openTestBlock`: the screen of the block just used opens over the game and the mouse
    /// is let go.
    pub fn open_test_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(pos) = session.take_opened_test_block() else {
            return;
        };
        let Some(contents) = session.test_block(pos) else {
            return;
        };
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let screen = TestBlockScreen::new((&assets.creative.font, &*assets.lang), &contents, size);
        self.test_block = Some(TestBlockEdit { screen, pos });
        self.chat = None;
        self.pause = None;
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
        let (width, height) = surface.window_size();
        surface.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0);
    }

    /// `onClose`: Done sends the mode and message; the mouse is grabbed again.
    fn close_test_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        done: Option<Done>,
    ) -> Result {
        if let (Some(edit), Some(done)) = (self.test_block.take(), done) {
            session.set_test_block(edit.pos, done.mode, &done.message)?;
        }
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
        Ok(())
    }

    /// An event for the test block's screen.
    pub(super) fn test_block_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Modifiers, MouseButton, WindowEvent};
        if let WindowEvent::CursorPos(x, y) = event {
            self.move_gui_mouse((x, y), surface);
            return Ok(());
        }
        let Some(edit) = self.test_block.as_mut() else {
            return Ok(());
        };
        let lang = &assets.lang;
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let action = match event {
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let control = modifiers.intersects(Modifiers::Control | Modifiers::Super);
                let shift = modifiers.contains(Modifiers::Shift);
                edit.screen
                    .key_pressed(lang, key, (control, shift), &mut WindowClipboard(surface))
            }
            WindowEvent::Char(character) => {
                edit.screen.char_typed(character);
                TestBlockAction::Stay
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => edit
                .screen
                .clicked(lang, mouse, modifiers.contains(Modifiers::Shift)),
            WindowEvent::Scroll(_, y) => {
                edit.screen.scrolled(lang, y, mouse);
                TestBlockAction::Stay
            }
            _ => TestBlockAction::Stay,
        };
        match action {
            TestBlockAction::Stay => Ok(()),
            TestBlockAction::Cancel => self.close_test_block(session, surface, None),
            TestBlockAction::Done(done) => self.close_test_block(session, surface, Some(done)),
        }
    }
}

impl TestBlockEdit {
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
