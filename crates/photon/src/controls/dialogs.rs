use super::*;
use photon_game::dialog::{Dialog, DialogRef, Next, run_action};
use photon_game::text::{ClickEvent, Component, Lang};
use photon_gui::screen::dialog::{DialogScreen, Press};
use photon_gui::screen::waiting::WaitingScreen;
use photon_gui::widgets::edit_box::Clipboard;
use photon_net::connection::DialogEvent;
use photon_render::gui::font::Font;

/// The screen a server dialog puts over the game.
pub enum DialogLayer {
    /// A notice or confirmation, and the data it was made from.
    Dialog {
        dialog: Box<Dialog>,
        screen: Box<DialogScreen>,
    },
    /// `WaitingForResponseScreen`, while the server answers a button.
    Waiting(Box<WaitingScreen>),
}

fn gui_size(surface: &WindowSurface) -> (i32, i32) {
    let size = surface.scale();
    (size.gui_scaled_width(), size.gui_scaled_height())
}

impl DialogLayer {
    /// `DialogScreens.createFromData`: none for a dialog whose screen is not built.
    fn of(dialog: Dialog, surface: &WindowSurface, assets: &ScreenAssets) -> Option<Self> {
        let screen = DialogScreen::new(
            &assets.creative.font,
            &assets.lang,
            gui_size(surface),
            &dialog,
            true,
        )?;
        Some(Self::Dialog {
            dialog: Box::new(dialog),
            screen: Box::new(screen),
        })
    }

    /// The screen again for a screen of another size (`repositionElements`): the controls keep
    /// what they hold, and the waiting screen its count.
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        match self {
            Self::Dialog { dialog, screen } => {
                let mut again = DialogScreen::new(font, lang, size, dialog, true)
                    .expect("a dialog that had a screen has one");
                again.keep_inputs_of(screen);
                Self::Dialog {
                    screen: Box::new(again),
                    dialog: dialog.clone(),
                }
            }
            Self::Waiting(waiting) => Self::Waiting(Box::new(waiting.resized(font, lang, size))),
        }
    }

    /// `setScreen(this)`: the screen starts over, its controls as the dialog first gave them.
    fn restarted(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        match self {
            Self::Dialog { dialog, .. } => Self::Dialog {
                screen: Box::new(
                    DialogScreen::new(font, lang, size, dialog, true)
                        .expect("a dialog that had a screen has one"),
                ),
                dialog: dialog.clone(),
            },
            Self::Waiting(_) => self.resized(font, lang, size),
        }
    }

    pub fn size(&self) -> (i32, i32) {
        match self {
            Self::Dialog { screen, .. } => screen.size(),
            Self::Waiting(waiting) => waiting.size(),
        }
    }

    pub fn draw(
        &self,
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font): (&photon_render::gui::GuiAtlas, &Font),
        item: &mut photon_render::gui::ItemDrawer<'_>,
        (mouse, blurriness): ((i32, i32), i32),
    ) {
        match self {
            Self::Dialog { screen, .. } => {
                screen.draw(frame, (atlas, font), item, (mouse, blurriness));
            }
            Self::Waiting(waiting) => waiting.draw(frame, (atlas, font), mouse, blurriness),
        }
    }
}

/// A key or character the focused text box takes before the screen does (Escape is the
/// screen's first). Whether it did.
fn text_event(
    screen: &mut DialogScreen,
    event: &glfw::WindowEvent,
    font: &Font,
    clipboard: &mut dyn Clipboard,
) -> bool {
    use glfw::{Action, Modifiers, WindowEvent};
    match *event {
        WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers)
            if key != glfw::Key::Escape =>
        {
            let control = modifiers.intersects(Modifiers::Control | Modifiers::Super);
            let shift = modifiers.contains(Modifiers::Shift);
            screen.edit_key(font, key, (control, shift), clipboard)
        }
        WindowEvent::Char(character) => screen.char_typed(font, character),
        _ => false,
    }
}

/// An event for the screen of a dialog: what it makes of a key or the mouse.
fn screen_event(
    screen: &mut DialogScreen,
    event: glfw::WindowEvent,
    (mouse, gui_mouse, previous_y): ((i32, i32), (f64, f64), f64),
    (font, clipboard): (&Font, &mut dyn Clipboard),
) -> Option<Press> {
    use glfw::{Action, Key, WindowEvent};
    if text_event(screen, &event, font, clipboard) {
        return None;
    }
    match event {
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => screen.escape(),
        WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
            screen.tab(!modifiers.contains(glfw::Modifiers::Shift));
            None
        }
        WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, modifiers) => {
            screen.set_shift(modifiers.contains(glfw::Modifiers::Shift));
            screen.pressed()
        }
        WindowEvent::Key(key @ (Key::Left | Key::Right), _, Action::Press | Action::Repeat, _) => {
            screen.arrow(key == Key::Right);
            None
        }
        WindowEvent::CursorPos(..) => {
            screen.dragged(gui_mouse, gui_mouse.1 - previous_y, font);
            None
        }
        WindowEvent::Scroll(_, y) => {
            screen.scrolled(mouse, y);
            None
        }
        WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, modifiers) => {
            screen.set_shift(modifiers.contains(glfw::Modifiers::Shift));
            screen.clicked(mouse, font)
        }
        WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
            screen.released();
            None
        }
        _ => None,
    }
}

impl Controls {
    /// The dialogs the server showed or cleared since the last frame.
    pub fn receive_dialogs(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if self.pause.is_none() {
            self.pause_additions = session.pause_screen_additions();
        }
        for event in session.take_dialog_events() {
            match event {
                DialogEvent::Show(dialog) => {
                    let dialog = dialog.resolved(&*session);
                    self.show_dialog(dialog, surface, assets);
                }
                DialogEvent::Clear => self.clear_dialog(),
            }
        }
    }

    /// The dialog a click event names: written out, or by the server's registry.
    pub(super) fn resolve_dialog(
        &self,
        dialog: &DialogRef,
        session: &PlaySession,
    ) -> Option<Dialog> {
        photon_game::dialog::DialogContext::dialog(session, dialog)
            .map(|dialog| dialog.resolved(session))
    }

    /// `ClientCommonPacketListenerImpl.showDialog`: the dialog takes the place of the screen it
    /// is shown over. The mouse is let go.
    pub(super) fn show_dialog(
        &mut self,
        dialog: Dialog,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(layer) = DialogLayer::of(dialog, surface, assets) else {
            log::warn!("failed to show a dialog whose screen is not built");
            return;
        };
        self.drop_light_screens();
        self.dialog = Some(layer);
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
    }

    /// `ClientCommonPacketListenerImpl.clearDialog`: a dialog on screen goes; a waiting screen
    /// stays.
    fn clear_dialog(&mut self) {
        if matches!(self.dialog, Some(DialogLayer::Dialog { .. })) {
            self.dialog = None;
        }
    }

    /// `DialogScreen.WarningScreen`: the question about screens a server made.
    fn open_dialog_warning(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let translated = Component::translatable;
        self.dialog_warning = Some(
            ConfirmScreen::new(
                &assets.creative.font,
                &assets.lang,
                gui_size(surface),
                &translated("menu.custom_screen_info.title"),
                &translated("menu.custom_screen_info.contents"),
                &translated("menu.disconnect"),
                &translated("gui.back"),
            )
            .with_background(ConfirmBackground::Menu { in_world: true }),
        );
    }

    /// An event for the warning screen: Yes leaves the server, No and Escape go back.
    pub(super) fn dialog_warning_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) {
        use glfw::{Action, Key, WindowEvent};
        let Some(warning) = self.dialog_warning.as_mut() else {
            return;
        };
        let action = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => warning.escape(),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                warning.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                warning.pressed()
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                warning.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32))
            }
            _ => None,
        };
        match action {
            Some(ConfirmAction::Yes) => {
                self.dialog_warning = None;
                session.disconnect(Component::translatable(
                    "menu.custom_screen_info.disconnect",
                ));
            }
            Some(_) => self.dialog_warning = None,
            None => {}
        }
    }

    /// An event for the dialog on screen.
    pub(super) fn dialog_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let previous_y = self.gui_mouse.1;
        if let WindowEvent::CursorPos(x, y) = event {
            self.move_gui_mouse((x, y), surface);
        }
        let press = match (self.dialog.as_mut(), event) {
            (Some(DialogLayer::Dialog { screen, .. }), event) => screen_event(
                screen,
                event,
                (mouse, self.gui_mouse, previous_y),
                (&assets.creative.font, &mut WindowClipboard(surface)),
            ),
            (Some(DialogLayer::Waiting(waiting)), event) => {
                let closed = match event {
                    WindowEvent::Key(Key::Escape, _, Action::Press, _) => waiting.escape(),
                    WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                        waiting.clicked(mouse)
                    }
                    _ => false,
                };
                if closed {
                    self.dialog = None;
                }
                None
            }
            (None, _) => None,
        };
        match press {
            Some(Press::Warning) => {
                self.open_dialog_warning(surface, assets);
                Ok(())
            }
            Some(press) => self.dialog_pressed(press, session, surface, assets),
            None => Ok(()),
        }
    }

    /// `DialogScreen.runAction`: the screen it leaves, then the button's click event.
    fn dialog_pressed(
        &mut self,
        press: Press,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let Some(DialogLayer::Dialog { dialog, screen }) = self.dialog.as_ref() else {
            return Ok(());
        };
        let values = screen.values();
        let (action, after) = match press {
            Press::Button(index) => (
                dialog.main_actions()[index].action.as_ref(),
                dialog.common().after_action,
            ),
            _ => (dialog.on_cancel(), photon_game::dialog::AfterAction::Close),
        };
        let outcome = run_action(action, after, &values);
        // A command is checked before the screen changes (`handleDialogClickEvent` leaves it to
        // `sendUnattendedCommand`); the other events act on the screen it leaves.
        if let Some(ClickEvent::RunCommand(command)) = &outcome.event {
            let command = command.strip_prefix('/').unwrap_or(command);
            return self.send_unattended(command, Some(outcome.next), session, surface, assets);
        }
        self.leave_dialog(outcome.next, surface, assets);
        match &outcome.event {
            Some(event) => self.click_event(event, session, surface, assets),
            None => Ok(()),
        }
    }

    /// The screen a used button leaves (`DialogScreen.runAction`).
    pub(super) fn leave_dialog(
        &mut self,
        next: Next,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        match next {
            Next::Previous => self.dialog = None,
            // `setScreen(this)` starts the screen over.
            Next::Same => {
                if let Some(layer) = self.dialog.take() {
                    self.dialog = Some(layer.restarted(
                        &assets.creative.font,
                        &assets.lang,
                        gui_size(surface),
                    ));
                }
            }
            Next::Waiting => {
                self.dialog = Some(DialogLayer::Waiting(Box::new(WaitingScreen::new(
                    &assets.creative.font,
                    &assets.lang,
                    gui_size(surface),
                    true,
                ))));
            }
        }
    }
}
