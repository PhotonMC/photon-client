use super::*;
use photon_game::chat_abilities::RestrictionAction;
use photon_gui::screen::confirm::ConfirmBackground;
use photon_gui::screen::restrictions::{Press, RestrictionsScreen};

/// `CommonLinks.ACCOUNT_SETTINGS`.
const ACCOUNT_SETTINGS: &str = "https://aka.ms/JavaAccountSettings";

impl Controls {
    /// `ChatScreen.handleComponentClicked` on the restricted line: the restrictions screen opens
    /// over the chat.
    pub(super) fn open_restrictions(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.restrictions = Some(RestrictionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            self.chat_log.abilities(),
            true,
        ));
    }

    /// What a restriction's button does (`ChatRestriction.Action`): the chat settings open, or the
    /// account page is asked about, over the chat.
    fn restriction_action(
        &mut self,
        action: RestrictionAction,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        match action {
            RestrictionAction::GoToChatSettings => self.open_chat_options(surface, assets),
            RestrictionAction::GoToAccount => {
                let screen = ConfirmScreen::link(
                    &assets.creative.font,
                    &assets.lang,
                    Self::gui_size(surface),
                    ACCOUNT_SETTINGS,
                    true,
                )
                .with_background(ConfirmBackground::Menu { in_world: true });
                self.link_confirm = Some((screen, ACCOUNT_SETTINGS.to_owned()));
            }
        }
    }

    /// An event for the open restrictions screen; a press leaves it, for the chat or a screen
    /// over the chat.
    pub(super) fn restrictions_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::WindowEvent;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(screen) = self.restrictions.as_mut() {
                screen.dragged(self.gui_mouse.1, self.gui_mouse.1 - before);
            }
            return;
        }
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let Some(screen) = self.restrictions.as_mut() else {
            return;
        };
        let press = screen_press(screen, &event, mouse, self.options.mouse_wheel_sensitivity);
        match press {
            Some(Press::Done) => self.restrictions = None,
            Some(Press::Action(action)) => {
                self.restrictions = None;
                self.restriction_action(action, surface, assets);
            }
            None => {}
        }
    }
}

/// What an event does on the screen: the press it made, if any.
fn screen_press(
    screen: &mut RestrictionsScreen,
    event: &glfw::WindowEvent,
    mouse: (i32, i32),
    wheel: f64,
) -> Option<Press> {
    use glfw::{Action, Key, MouseButton, WindowEvent};
    match event {
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => Some(screen.escape()),
        WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
            screen.tab(!modifiers.contains(glfw::Modifiers::Shift));
            None
        }
        WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
            screen.pressed()
        }
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => screen.clicked(mouse),
        WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
            screen.released();
            None
        }
        WindowEvent::Scroll(_, y) => {
            screen.scrolled(mouse, y * wheel);
            None
        }
        _ => None,
    }
}

impl Controls {
    /// An event for the chat screen: the restrictions screen, or the chat settings it sent the
    /// player to, when one is up over the chat; else the chat box.
    pub(super) fn chat_screen_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if self.restrictions.is_some() {
            self.restrictions_event(event, surface, assets);
        } else if self.chat_options_screen.is_some() {
            self.chat_options_event(event, surface, assets);
        } else {
            return self.chat_event(event, session, surface, assets);
        }
        Ok(())
    }
}
