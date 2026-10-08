use super::*;

use photon_render::gui::screen::book_edit::{BookEditAction, BookEditScreen};

/// The book and quill being written in: its screen and the hand that holds it.
pub struct BookEdit {
    pub screen: BookEditScreen,
    pub hand: u8,
    /// While the primary button is down over the screen, which drags the selection: whether
    /// Shift was down when it went down.
    held: Option<bool>,
}

impl Controls {
    /// `LocalPlayer.openItemGui` for a book and quill: its screen opens over the game and the
    /// mouse is let go.
    pub fn open_book_editor(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some((hand, pages)) = session.take_opened_book() else {
            return;
        };
        let pages = pages.iter().map(|page| assets.lang.plain(page)).collect();
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.book_edit = Some(BookEdit {
            screen: BookEditScreen::new((&assets.creative.font, &assets.lang), pages, size, None),
            hand,
            held: None,
        });
        self.chat = None;
        self.pause = None;
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
        let (width, height) = surface.window_size();
        surface.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0);
    }

    /// The screen closes; Done sends the pages (`saveChanges`), Escape nothing.
    fn close_book(
        &mut self,
        save: bool,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        if let Some(edit) = self.book_edit.take()
            && save
        {
            session.edit_book(edit.hand, &edit.screen.saved_pages(), None)?;
        }
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
        Ok(())
    }

    /// An event for the book's screen.
    pub(super) fn book_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Modifiers, WindowEvent};
        let font = &assets.creative.font;
        let texts = (&**font, &*assets.lang);
        let mouse = self.gui_mouse;
        let Some(edit) = self.book_edit.as_mut() else {
            return Ok(());
        };
        let action = match event {
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let control = modifiers.intersects(Modifiers::Control | Modifiers::Super);
                let shift = modifiers.contains(Modifiers::Shift);
                edit.screen
                    .key_pressed(texts, key, (control, shift), &mut WindowClipboard(surface))
            }
            WindowEvent::Char(character) => {
                edit.screen.char_typed(font, character);
                None
            }
            WindowEvent::CursorPos(x, y) => {
                if let Some(shift) = edit.held {
                    self.move_gui_mouse((x, y), surface);
                    if let Some(edit) = self.book_edit.as_mut() {
                        edit.screen.dragged(font, self.gui_mouse, shift);
                    }
                } else {
                    self.move_gui_mouse((x, y), surface);
                }
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(Modifiers::Shift);
                edit.held = Some(shift);
                edit.screen.clicked(texts, mouse, (shift, false), None)
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                edit.held = None;
                None
            }
            _ => None,
        };
        match action {
            Some(BookEditAction::Done) => self.close_book(true, session, surface),
            Some(BookEditAction::Close) => self.close_book(false, session, surface),
            // The signing screen is not written yet (T88b).
            Some(BookEditAction::Sign) | None => Ok(()),
        }
    }
}
