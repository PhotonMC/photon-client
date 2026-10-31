//! The friends of the title screen: the account is resolved off the menu's thread, and once it is
//! known the title's friends button opens the overlay the game opens with its key.

use std::sync::mpsc::{Receiver, channel};

use photon_gui::screen::title::TitleAction;
use photon_net::account::Account;

use super::{Choice, Client, Menu, Screen};

/// The account the title is looking up, and whether the friends work now.
pub(super) struct TitleFriends {
    account: Option<Receiver<Account>>,
    pub(super) ready: bool,
}

impl TitleFriends {
    /// Looks the signed-in account up on its own thread: the credential store may be slow.
    pub(super) fn look_up() -> Self {
        let (sender, receiver) = channel();
        let _ = std::thread::Builder::new()
            .name("Title Account".to_owned())
            .spawn(move || {
                if let Ok((_, Some(account))) = crate::profile_login::identity() {
                    let _ = sender.send(account);
                }
            });
        Self {
            account: Some(receiver),
            ready: false,
        }
    }
}

impl Menu {
    /// The account, when the lookup has found one: the friends start and the title's button works.
    pub(super) fn friends_account_polled(&mut self) {
        let Some(receiver) = self.friends_title.account.as_ref() else {
            return;
        };
        let Ok(account) = receiver.try_recv() else {
            return;
        };
        self.friends_title.account = None;
        self.host.start_friends_for(&account);
        self.friends_title.ready = true;
        if let Screen::Title(screen) = &mut self.screen {
            screen.friends_available(self.gui.lang());
        }
    }

    /// One frame of the friends: the answers of the service, and the key's request to open.
    pub(super) fn friends_ticked(&mut self, client: &mut Client) {
        for title in self.host.friends_tick(self.gui.lang()) {
            self.gui.add_friend_toast(&title);
        }
        let options = if self.host.host_open() {
            &self.host.options
        } else {
            &self.options
        };
        self.gui.update_menu_toasts(options);
        if self.host.take_friends_wanted() {
            self.open_friends(client);
        }
    }

    /// The press on the title's friends button: whether the overlay or its question is open.
    pub(super) fn open_friends(&mut self, client: &mut Client) -> bool {
        self.host.gui_mouse = (f64::from(self.mouse.0), f64::from(self.mouse.1));
        self.host.toggle_friends(&mut client.surface, &self.assets);
        self.host.host_open()
    }

    /// One window event for the title: the friends overlay over it takes it first.
    pub(super) fn title_input(
        &mut self,
        event: glfw::WindowEvent,
        client: &mut Client,
        size: (i32, i32),
    ) -> Option<Choice> {
        if self.host.host_open() {
            self.settings_event(event, client);
            return None;
        }
        let Screen::Title(screen) = &mut self.screen else {
            return None;
        };
        let chosen = super::input::title_event(screen, event, self.mouse);
        self.title_pressed(chosen, client, size)
    }

    /// A press on the title: an options screen, the friends, or what the title chose.
    pub(super) fn title_pressed(
        &mut self,
        chosen: Option<TitleAction>,
        client: &mut Client,
        size: (i32, i32),
    ) -> Option<Choice> {
        if self.open_settings(chosen.as_ref(), client) {
            return None;
        }
        if matches!(chosen, Some(TitleAction::Friends)) {
            self.open_friends(client);
            return None;
        }
        self.title_chosen(chosen, size)
    }
}
