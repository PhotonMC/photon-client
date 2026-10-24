//! The credits of the title screen and what the title's buttons open.

use std::env;
use std::time::Instant;

use glfw::{Action, Key, WindowEvent};
use photon_gui::screen::credits::CreditsScreen;
use photon_gui::screen::title::TitleAction;

use super::{Choice, Menu, Screen};

impl Menu {
    /// `WinScreen` of the title's Credits button: the credits alone, no poem.
    fn credits(&self, size: (i32, i32)) -> CreditsScreen {
        let user = env::var("PHOTON_USER").unwrap_or_else(|_| "Photon".to_owned());
        CreditsScreen::new(
            self.gui.text_assets().0,
            &*self.gui.resources,
            size,
            false,
            &user,
        )
    }

    /// What a press on the title screen chose.
    pub(super) fn title_chosen(
        &mut self,
        chosen: Option<TitleAction>,
        size: (i32, i32),
    ) -> Option<Choice> {
        match chosen {
            Some(TitleAction::Multiplayer) => self.screen = self.server_list(size),
            Some(TitleAction::Quit) => return Some(Choice::Quit),
            // The other screens are not here yet.
            _ => {}
        }
        None
    }

    /// The roll the credits and attribution screen's Credits button starts.
    pub(super) fn show_credits(&mut self, size: (i32, i32)) {
        self.screen = Screen::Credits(self.credits(size), Instant::now());
    }

    /// `setScreen(this)` of the roll's callback: the credits and attribution screen it came from.
    fn leave_credits(&mut self, size: (i32, i32)) {
        self.screen = if self.host.host_open() {
            Screen::Settings
        } else {
            self.title(size)
        };
    }

    /// `WinScreen.tick`: the roll that has run past its end closes.
    pub(super) fn credits_ticked(&mut self, size: (i32, i32)) {
        if matches!(&self.screen, Screen::Credits(screen, _) if screen.tick()) {
            self.leave_credits(size);
        }
    }

    /// The window's size changed: `init` again keeps the roll where it was.
    pub(super) fn refit_credits(&mut self, size: (i32, i32)) -> bool {
        let Screen::Credits(old, drawn) = &self.screen else {
            return false;
        };
        if old.size() == size {
            return false;
        }
        let mut screen = self.credits(size);
        screen.set_scroll(old.scroll());
        self.screen = Screen::Credits(screen, *drawn);
        true
    }

    /// An event for the open credits: Escape closes them (`onClose`), the speed keys steer the
    /// roll.
    pub(super) fn credits_event(&mut self, event: WindowEvent, size: (i32, i32)) -> Option<Choice> {
        let Screen::Credits(screen, _) = &mut self.screen else {
            return None;
        };
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press | Action::Repeat, _) => {
                self.leave_credits(size);
            }
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, _) => {
                screen.key_pressed(key as i32);
            }
            WindowEvent::Key(key, _, Action::Release, _) => screen.key_released(key as i32),
            _ => {}
        }
        None
    }
}
