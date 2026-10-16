use super::*;

use photon_gui::screen::book::BookScreen;
use photon_gui::screen::book_edit::{BookEditAction, BookEditScreen};
use photon_gui::screen::book_sign::{BookSignAction, BookSignScreen};

/// The book and quill being written in: its screen, the signing screen while it is over it, and
/// the hand that holds it.
pub struct BookEdit {
    pub screen: BookEditScreen,
    pub sign: Option<BookSignScreen>,
    /// The title typed on the signing screen when it was last left (`titleValue`).
    title: String,
    /// The player's name, which signs the book.
    owner: String,
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
            sign: None,
            title: String::new(),
            owner: session.local_name().to_owned(),
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

    /// `handleOpenBook`: the book in the hand the server named shows over the game.
    pub fn open_held_book(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(pages) = session.take_open_book() else {
            return;
        };
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.screen = Some(Box::new(BookScreen::held(
            size,
            assets.lang.clone(),
            assets.creative.font.clone(),
            pages,
        )));
        self.book_edit = None;
        self.sign_edit = None;
        self.chat = None;
        self.pause = None;
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
    }

    /// The screen closes; Done sends the pages (`saveChanges`), signing sends the pages as they
    /// are with the title, Escape nothing.
    fn close_book(
        &mut self,
        save: Option<Option<String>>,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        if let (Some(edit), Some(title)) = (self.book_edit.take(), save) {
            match title {
                None => session.edit_book(edit.hand, &edit.screen.saved_pages(), None)?,
                Some(title) => session.edit_book(edit.hand, edit.screen.pages(), Some(&title))?,
            }
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
        if edit.sign.is_some() {
            return self.book_sign_event(event, session, surface, assets);
        }
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
            Some(BookEditAction::Done) => self.close_book(Some(None), session, surface),
            Some(BookEditAction::Close) => self.close_book(None, session, surface),
            Some(BookEditAction::Sign) => {
                if let Some(edit) = self.book_edit.as_mut() {
                    edit.sign = Some(BookSignScreen::new(
                        texts,
                        &edit.title,
                        &edit.owner,
                        edit.screen.size(),
                    ));
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    /// An event for the signing screen over the book's.
    fn book_sign_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Modifiers, WindowEvent};
        let font = &assets.creative.font;
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let Some(sign) = self.book_edit.as_mut().and_then(|edit| edit.sign.as_mut()) else {
            return Ok(());
        };
        let action = match event {
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let control = modifiers.intersects(Modifiers::Control | Modifiers::Super);
                let shift = modifiers.contains(Modifiers::Shift);
                sign.key_pressed(font, key, (control, shift), &mut WindowClipboard(surface))
            }
            WindowEvent::Char(character) => {
                sign.char_typed(font, character);
                None
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                sign.clicked(font, mouse)
            }
            _ => None,
        };
        match action {
            Some(BookSignAction::Finalize(title)) => {
                self.close_book(Some(Some(title)), session, surface)
            }
            Some(BookSignAction::Close) => self.close_book(None, session, surface),
            Some(BookSignAction::Cancel(title)) => {
                // Back to the pages, which init again (`rebuildWidgets`).
                if let Some(edit) = self.book_edit.as_mut() {
                    edit.title = title;
                    edit.sign = None;
                    edit.screen.reopened(&assets.creative.font, &assets.lang);
                }
                Ok(())
            }
            None => Ok(()),
        }
    }
}
