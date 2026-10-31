use super::*;
use photon_gui::options::online::{FriendFacts, FriendPress, OnlineOptionsScreen};

/// `CommonLinks.PRIVACY_AND_ONLINE_SETTINGS`.
const PRIVACY_AND_ONLINE_SETTINGS: &str = "https://aka.ms/MinecraftJavaXboxPrivacyAndSafety";

impl Controls {
    /// `OnlineOptionsScreen`, over the options screen. There is no account yet, so the friends
    /// list is off.
    pub(super) fn open_online(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let screen = OnlineOptionsScreen::new(
            (&assets.creative.font, &assets.lang),
            Self::gui_size(surface),
            !self.title_host,
            (&self.options, self.friend_facts()),
        );
        // The friends buttons work once the connection has an account to ask the service as.
        self.online_screen = Some(if self.friends_list.is_some() {
            screen
        } else {
            screen.without_services(&assets.lang)
        });
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen, and the options are
    /// saved.
    pub(super) fn close_online(&mut self) {
        self.online_screen = None;
        self.save_options();
    }

    /// What the account says of the friends list.
    pub(super) fn friend_facts(&self) -> FriendFacts {
        self.friends_list
            .as_ref()
            .map_or_else(FriendFacts::default, |social| FriendFacts {
                enabled: social.list_enabled(),
                allow_requests: social.allow_requests(),
            })
    }

    /// `OnlineOptionsScreen`'s friends buttons: the list button asks to turn the list on or turns
    /// it off with the invites and the in-game notifications, the requests button changes only
    /// whether requests are allowed. The service has the last word; the screen follows it.
    fn friend_pressed(
        &mut self,
        press: FriendPress,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        let facts = self.friend_facts();
        let Some(social) = self.friends_list.as_ref() else {
            return;
        };
        match press {
            FriendPress::List if facts.enabled => {
                self.options.online.in_game_notification = false;
                social.update_settings(false, false);
            }
            FriendPress::List => self.ask_friends_list(surface, assets),
            FriendPress::Requests => social.update_settings(facts.enabled, !facts.allow_requests),
        }
    }

    /// The switches the screen shows are the options'.
    fn take_online_choices(&mut self) {
        let Some(screen) = self.online_screen.as_ref() else {
            return;
        };
        self.options.online = screen.choices(self.options.online);
    }

    /// An event for the open online settings.
    pub(super) fn online_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let lang = &assets.lang;
        if let glfw::WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(screen) = self.online_screen.as_mut() {
                screen.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            return;
        }
        let wheel = self.options.mouse_wheel_sensitivity;
        let Some(screen) = self.online_screen.as_mut() else {
            return;
        };
        let (done, xbox, friend) = screen_input(screen, &event, (lang, self.gui_mouse, wheel));
        self.take_online_choices();
        if let Some(press) = friend {
            self.friend_pressed(press, surface, assets);
        }
        if xbox {
            // `PrivacyConfirmLinkScreen.confirmLinkNow`: the page is asked about, over the screen.
            let screen = ConfirmScreen::link(
                &assets.creative.font,
                &assets.lang,
                Self::gui_size(surface),
                PRIVACY_AND_ONLINE_SETTINGS,
                true,
            )
            .with_background(ConfirmBackground::Menu {
                in_world: !self.title_host,
            });
            self.link_confirm = Some((
                screen,
                ConfirmTarget::Link(PRIVACY_AND_ONLINE_SETTINGS.to_owned()),
            ));
        }
        if done {
            self.close_online();
        }
    }
}

/// What an event does on the screen: whether the screen is done.
fn screen_input(
    screen: &mut OnlineOptionsScreen,
    event: &glfw::WindowEvent,
    (lang, mouse, wheel): (&photon_game::text::Lang, (f64, f64), f64),
) -> (bool, bool, Option<FriendPress>) {
    use glfw::{Action, Key, MouseButton, WindowEvent};
    let result = |input: photon_gui::options::list::SubScreenInput| {
        (
            input.done,
            OnlineOptionsScreen::xbox_pressed(&input),
            OnlineOptionsScreen::friend_pressed(&input),
        )
    };
    match (menu_key(event), event) {
        (Some(MenuKey::Tab(forward)), _) => screen.tab(forward),
        (Some(MenuKey::Key(key, shift)), _) => return result(screen.key_pressed(lang, key, shift)),
        (None, WindowEvent::Key(Key::Escape, _, Action::Press, _)) => return (true, false, None),
        (None, WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers)) => {
            let shift = modifiers.contains(glfw::Modifiers::Shift);
            return result(screen.clicked(lang, mouse, shift));
        }
        (None, WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _)) => {
            screen.released(lang);
        }
        (None, WindowEvent::Scroll(_, y)) => screen.scrolled(mouse, y * wheel),
        _ => {}
    }
    (false, false, None)
}
