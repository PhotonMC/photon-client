//! The title screen's Options, Language and Accessibility buttons: the screens are the ones of a
//! level, kept in a `Controls` that the menu feeds the window's events.

use photon_gui::screen::title::TitleAction;

use super::{Menu, Screen};
use crate::play::Client;

impl Menu {
    /// `PHOTON_MENU_SCREEN=servers|options|video|language|credits|roll|accessibility`: open on that
    /// screen, for a capture of it.
    pub(super) fn open_requested_screen(&mut self, client: &Client) {
        let Ok(name) = std::env::var("PHOTON_MENU_SCREEN") else {
            return;
        };
        let action = match name.as_str() {
            "servers" => {
                self.screen = self.server_list(super::gui_size(client));
                return;
            }
            "video" => {
                self.host.options = crate::controls::load_options();
                self.host.open_title_video(&client.surface, &self.assets);
                self.screen = Screen::Settings;
                return;
            }
            "roll" => {
                self.show_credits(super::gui_size(client));
                return;
            }
            "options" => TitleAction::Options,
            "language" => TitleAction::Language,
            "credits" => TitleAction::Credits,
            "accessibility" => TitleAction::Accessibility,
            _ => return,
        };
        self.open_settings(Some(&action), client);
    }

    /// A press on one of the title's buttons that opens an options screen: whether it did.
    pub(super) fn open_settings(&mut self, chosen: Option<&TitleAction>, client: &Client) -> bool {
        if !matches!(
            chosen,
            Some(
                TitleAction::Options
                    | TitleAction::Language
                    | TitleAction::Accessibility
                    | TitleAction::Credits
            )
        ) {
            return false;
        }
        // The file may have changed since (the last server of a direct join).
        self.host.options = crate::controls::load_options();
        self.host.gui_mouse = (f64::from(self.mouse.0), f64::from(self.mouse.1));
        match chosen {
            Some(TitleAction::Options) => {
                self.host.open_title_options(&client.surface, &self.assets);
            }
            Some(TitleAction::Language) => {
                self.host.open_title_language(&client.surface, &self.assets);
            }
            Some(TitleAction::Credits) => {
                self.host
                    .open_title_credits_attribution(&client.surface, &self.assets);
            }
            _ => self
                .host
                .open_title_accessibility(&client.surface, &self.assets),
        }
        self.screen = Screen::Settings;
        true
    }

    /// One window event for the open options screens, and what they changed: the saved options
    /// are the menu's, a language chosen is loaded, and with every screen closed the title is back.
    pub(super) fn settings_event(&mut self, event: glfw::WindowEvent, client: &mut Client) {
        self.host
            .host_event(event, &mut client.surface, &self.assets);
        if let Some(code) = self.host.take_language_request() {
            self.gui.set_language(&code);
            self.assets = self.gui.screen_assets();
            self.host.rebuild_menus(&client.surface, &self.assets);
        }
        if std::mem::take(&mut self.host.options_saved) {
            self.options = self.host.options.clone();
        }
        if self.host.take_credits_request() {
            let size = super::gui_size(client);
            self.show_credits(size);
            return;
        }
        if !self.host.host_open() {
            self.screen = self.title(super::gui_size(client));
        }
    }
}
