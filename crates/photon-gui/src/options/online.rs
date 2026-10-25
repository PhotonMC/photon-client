//! The online settings screen (`OnlineOptionsScreen`): the friends list and what comes with it,
//! the server listing and the Realms notifications. Without an account the friends list is off,
//! which leaves the three buttons that depend on it inactive; turning it on asks the player to
//! confirm in a screen that is not written yet.

use photon_game::options::{OnlineOptions, Options, PresenceSharing};
use photon_game::text::{Component, Lang};

use crate::options::list::{
    Build, SubScreen, SubScreenInput, boolean_tip, button, cycle, cycle_tip, toggle, translated,
};
use crate::widgets::cycle::{CycleButton, Tip};
use crate::widgets::slider::SliderKey;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// What the screen finds its widgets by.
const FRIENDS_LIST: u8 = 1;
const ALLOW_REQUESTS: u8 = 2;
const IN_GAME: u8 = 3;
const PRESENCE: u8 = 4;
const XBOX: u8 = 5;
const SERVER_LISTING: u8 = 6;
const REALMS: u8 = 7;

/// What the account says about the friends list (`PlayerSocialManager`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FriendFacts {
    pub enabled: bool,
    pub allow_requests: bool,
}

pub struct OnlineOptionsScreen {
    screen: SubScreen,
    friends: FriendFacts,
}

/// The four buttons of the friends section, two to a row.
fn friend_rows(online: OnlineOptions, friends: FriendFacts) -> Vec<(u8, Build)> {
    let presence_tip: Tip<&'static str> = Box::new(|key| {
        let tip = PresenceSharing::from_caption_key(key).map(PresenceSharing::tooltip_key);
        vec![tip.map(Component::translatable)]
    });
    vec![
        (
            FRIENDS_LIST,
            cycle(
                "options.friendsList",
                vec![true, false],
                toggle,
                friends.enabled,
            ),
        ),
        (
            ALLOW_REQUESTS,
            boolean_tip(
                "options.allowFriendRequests",
                friends.allow_requests,
                "options.allowFriendRequests.tooltip",
            ),
        ),
        (
            IN_GAME,
            boolean_tip(
                "options.inGameNotification",
                online.in_game_notification,
                "options.inGameNotification.tooltip",
            ),
        ),
        (
            PRESENCE,
            cycle_tip(
                "options.sharePresence",
                PresenceSharing::ALL
                    .map(PresenceSharing::caption_key)
                    .to_vec(),
                translated,
                online.share_presence.caption_key(),
                presence_tip,
            ),
        ),
    ]
}

impl OnlineOptionsScreen {
    /// `init`, with the options as they are.
    pub fn new(
        (font, lang): (&Font, &Lang),
        size: (i32, i32),
        in_world: bool,
        (options, friends): (&Options, FriendFacts),
    ) -> Self {
        let online = options.online;
        let screen = SubScreen::new(font, lang, size, "options.online.title", in_world, |list| {
            list.add_header(font, lang, "options.online.friends.header");
            list.add_small(lang, friend_rows(online, friends));
            list.add_big(lang, XBOX, button("options.online.xboxSettings"));
            list.add_header(font, lang, "options.online.servers.header");
            list.add_big(
                lang,
                SERVER_LISTING,
                boolean_tip(
                    "options.allowServerListing",
                    online.allow_server_listing,
                    "options.allowServerListing.tooltip",
                ),
            );
            list.add_header(font, lang, "options.online.realms.header");
            list.add_big(
                lang,
                REALMS,
                boolean_tip(
                    "options.realmsNotifications.button",
                    online.realms_notifications,
                    "options.realmsNotifications.tooltip",
                ),
            );
        });
        let mut this = Self { screen, friends };
        this.update_dependents();
        this
    }

    /// `updateFriendListDependentButtons`.
    fn update_dependents(&mut self) {
        let enabled = self.friends.enabled;
        for tag in [ALLOW_REQUESTS, IN_GAME, PRESENCE] {
            if let Some(control) = self.screen.list.find_mut(tag) {
                control.set_active(enabled);
            }
        }
    }

    fn flag(&self, tag: u8, fallback: bool) -> bool {
        self.screen
            .list
            .find(tag)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
            .map_or(fallback, |button| *button.value())
    }

    /// What the buttons say now, for the options to take.
    pub fn choices(&self, fallback: OnlineOptions) -> OnlineOptions {
        let presence = self
            .screen
            .list
            .find(PRESENCE)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<&'static str>>())
            .and_then(|button| PresenceSharing::from_caption_key(button.value()))
            .unwrap_or(fallback.share_presence);
        OnlineOptions {
            realms_notifications: self.flag(REALMS, fallback.realms_notifications),
            in_game_notification: self.flag(IN_GAME, fallback.in_game_notification),
            share_presence: presence,
            allow_server_listing: self.flag(SERVER_LISTING, fallback.allow_server_listing),
        }
    }

    /// The friends list button is the account's to change, so a press of it puts it back.
    fn settled(&mut self, lang: &Lang, input: SubScreenInput) -> SubScreenInput {
        if matches!(input.changed, Some(FRIENDS_LIST | ALLOW_REQUESTS)) {
            for (tag, value) in [
                (FRIENDS_LIST, self.friends.enabled),
                (ALLOW_REQUESTS, self.friends.allow_requests),
            ] {
                if let Some(button) =
                    self.screen.list.find_mut(tag).and_then(|control| {
                        control.as_any_mut().downcast_mut::<CycleButton<bool>>()
                    })
                {
                    button.set_value(lang, value);
                }
            }
        }
        input
    }

    pub fn size(&self) -> (i32, i32) {
        self.screen.size()
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.screen.resize(size);
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        self.screen.draw(frame, atlas, font, mouse);
    }

    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> SubScreenInput {
        let input = self.screen.clicked(lang, mouse, shift);
        self.settled(lang, input)
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) {
        self.screen.dragged(lang, mouse, dy);
    }

    /// Tab (and Shift+Tab).
    pub fn tab(&mut self, forward: bool) {
        self.screen.tab(forward);
    }

    /// An arrow, Enter or Space.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> SubScreenInput {
        let input = self.screen.key_pressed(lang, key, shift);
        self.settled(lang, input)
    }

    pub fn released(&mut self, lang: &Lang) {
        self.screen.released(lang);
    }

    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) {
        self.screen.list.scrolled(mouse, scroll_y);
    }

    pub fn scroll_amount(&self) -> f64 {
        self.screen.list.scroll_amount()
    }
}
