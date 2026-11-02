//! The friends overlay (`FriendsOverlayScreen`, `FriendsTab`, `PendingTab`, `AddFriendWidget`): a
//! panel over the screen it was opened from, with a tab for the friends and one for the pending
//! requests, and above the friends' list the box to send a request.
//!
//! The game gives the screen what the lists are (`FriendsView`) and acts on what it asks for
//! (`FriendsEvent`); the screen owns the tab, the focus and the text of the box.
//!
//! Not drawn: the entries' presence and skins other than a default one, and the Microsoft account
//! link of the empty state is plain text.

mod draw;
mod empty;
mod entries;
mod faces;
mod input;
mod tooltip;

use std::time::Instant;

use photon_core::binary::Uuid;
use photon_game::text::{Component, Lang, Run, Style};

use crate::widgets::edit_box::EditBox;
use photon_render::gui::font::Font;

/// `OVERLAY_WIDTH`, the tab buttons' and the panel's border.
const OVERLAY_WIDTH: i32 = 220;
/// What the entries and the texts between them are as wide as: the overlay less its scroll room.
const ENTRY_WIDTH: i32 = OVERLAY_WIDTH - 16;
const TAB_WIDTH: i32 = 110;
const TAB_HEIGHT: i32 = 20;
const BORDER: i32 = 8;
/// One entry of the lists (`AbstractFriendsEntryContainerWidget`).
const ENTRY_HEIGHT: i32 = 28;
/// What the box to add a friend takes: the box, its button and the rows around them
/// (`AddFriendWidget`).
const INPUT_LEFT: i32 = 8;
const INPUT_TOP: i32 = 3;
const BOX_HEIGHT: i32 = 20;
const BUTTON_SIZE: i32 = 20;
const INPUT_SPACING: i32 = 3;
const PROFILE_TOP: i32 = 6;
const ADD_WIDGET_HEIGHT: i32 = INPUT_TOP + BOX_HEIGHT + PROFILE_TOP + 9 + 4 + 2;
/// `PROFILE_NAME_LABEL`'s colour and the loading dots'.
const HINT_COLOUR: u32 = 0xFFE0_E0E0;
const HINT_OFF_COLOUR: u32 = 0xFF70_7070;
const DOTS_COLOUR: u32 = 0xFF80_8080;
const WHITE: u32 = 0xFFFF_FFFF;
/// Names are at most this long (`EditBox`'s default).
const MAX_NAME: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Friends,
    Pending,
}

/// What the lists show (`FriendsTab.showLoading`, `showError`, `showEmpty` and the entries).
#[derive(Debug, Clone, PartialEq)]
pub enum Listing {
    Loading,
    /// The message, as runs.
    Error(Vec<Run>),
    Empty,
    /// The friends, or the incoming and then the outgoing requests, in order.
    Entries(Vec<Entry>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Friend,
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: Uuid,
    pub name: String,
    pub kind: EntryKind,
    /// A friend's presence, as runs in its colour; empty for requests.
    pub status: Vec<Run>,
}

/// What the game knows for the screen.
#[derive(Debug, Clone, PartialEq)]
pub struct FriendsView {
    pub friends: Listing,
    pub pending: Listing,
    /// The incoming requests (`gui.friends.requests_count`).
    pub requests: usize,
    pub profile_name: String,
    /// The list is loaded, so a request can be sent (`State.SUCCESS`).
    pub ready: bool,
    /// A request is being sent.
    pub sending: bool,
}

/// What the screen asks the game for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FriendsEvent {
    /// The screen closes (`onClose`).
    Close,
    /// `sendFriendRequest` with the trimmed name.
    Send(String),
    /// The profile name is copied (`setClipboard`).
    CopyName,
    /// The remove button of a friend: the game asks first.
    Remove(Uuid),
    Accept(Uuid),
    Decline(Uuid),
    Revoke(Uuid),
    /// The Microsoft account link of the empty list: the game asks before opening it.
    OpenLink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Box,
    None,
}

pub struct FriendsOverlay {
    size: (i32, i32),
    /// `backgroundScreen`: a screen drawn under the overlay, which then has no background tile.
    over_screen: bool,
    faces: super::faces::Faces,
    tab: Tab,
    edit: EditBox,
    focus: Focus,
    view: FriendsView,
    texts: Texts,
    focused_at: Instant,
    scroll: i32,
    /// The scroller is held by the primary button.
    dragging: bool,
    /// The button whose action is on its way, which shows the loading state.
    pressed: Option<Pressed>,
    pending_label: Vec<Run>,
    /// The footer of the friends list, wrapped to the entries' width.
    footer_lines: Vec<Vec<Run>>,
}

/// A button that was pressed and has not been answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pressed {
    Send,
    Entry(entries::Button, Uuid),
}

/// The screen's texts as runs, made once.
struct Texts {
    friends_tab: Vec<Run>,
    hint: Vec<Run>,
    loading: [Vec<Run>; 2],
    profile_label: Vec<Run>,
    empty: Vec<Run>,
    footer: Vec<Run>,
    pending_empty: Vec<Run>,
    received: Vec<Run>,
    sent: Vec<Run>,
    tooltips: tooltip::TooltipTexts,
}

/// The headers of the pending lists: bold and underlined.
fn heading_style() -> Style {
    Style {
        bold: Some(true),
        underlined: Some(true),
        ..Style::default()
    }
}

impl FriendsOverlay {
    pub fn new(lang: &Lang, font: &Font, size: (i32, i32), view: FriendsView) -> Self {
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let key = |name: &str| runs(&Component::translatable(name));
        let grey = |name: &str| runs(&Component::translatable(name).with_colour("gray"));
        let link = Component::translatable("gui.friends.empty_state.link").with_style(&Style {
            underlined: Some(true),
            colour: Some(0xAAAAAA),
            ..Style::default()
        });
        let heading = heading_style();
        let texts = Texts {
            friends_tab: key("gui.friends.tab_friends"),
            hint: runs(
                &Component::translatable("gui.friends.enter_nickname").with_colour("dark_gray"),
            ),
            loading: [
                key("gui.friends.loading_friends"),
                key("gui.friends.loading_requests"),
            ],
            profile_label: runs(
                &Component::translatable("gui.friends.my_profile_name").with_style(&Style {
                    colour: Some(0xA0A0A0),
                    ..Style::default()
                }),
            ),
            empty: runs(
                &Component::translatable_with("gui.friends.empty_state", vec![link.clone()])
                    .with_colour("gray"),
            ),
            footer: runs(
                &Component::translatable_with("gui.friends.manage_account_footer", vec![link])
                    .with_colour("gray"),
            ),
            pending_empty: grey("gui.friends.pending.empty"),
            received: runs(
                &Component::translatable("gui.friends.pending.received").with_style(&heading),
            ),
            sent: runs(&Component::translatable("gui.friends.pending.sent").with_style(&heading)),
            tooltips: tooltip::TooltipTexts::new(lang),
        };
        Self {
            footer_lines: crate::chat::split(&texts.footer, ENTRY_WIDTH - 8, font),
            pending_label: pending_label(lang, view.requests),
            focused_at: Instant::now(),
            scroll: 0,
            dragging: false,
            over_screen: false,
            faces: super::faces::Faces::default(),
            pressed: None,
            size,
            tab: Tab::Friends,
            edit: EditBox::new(
                OVERLAY_WIDTH - BUTTON_SIZE - INPUT_SPACING - 2 * INPUT_LEFT,
                MAX_NAME,
            ),
            focus: Focus::None,
            view,
            texts,
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The overlay opened over a screen of its own (the title), which it blurs and draws under.
    pub fn set_over_screen(&mut self, over_screen: bool) {
        self.over_screen = over_screen;
    }

    /// `repositionElements` for a window of another size.
    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
    }

    /// The lists as the game has them now.
    pub fn set_view(&mut self, lang: &Lang, view: FriendsView) {
        if view.requests != self.view.requests {
            self.pending_label = pending_label(lang, view.requests);
        }
        if !view.sending {
            self.pressed = None;
        }
        self.view = view;
        self.scroll = self.scroll.clamp(0, self.max_scroll());
    }

    /// `FriendEntry.confirmRemoveFriend`: the remove button of `id` waits for the answer.
    pub fn removing(&mut self, id: Uuid) {
        self.pressed = Some(Pressed::Entry(entries::Button::Remove, id));
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }

    /// The panel's content: the layout `FriendsOverlayScreen.repositionElements` centres, which is
    /// as wide as the overlay and 80 shorter than the window.
    fn content(&self) -> (i32, i32, i32, i32) {
        let height = (self.size.1 - 80).max(ADD_WIDGET_HEIGHT);
        let left = (self.size.0 - OVERLAY_WIDTH) / 2;
        (left, (self.size.1 - height) / 2, OVERLAY_WIDTH, height)
    }

    /// The height of `AddFriendWidget`, which only the friends tab has: the pending tab's list
    /// starts at the top of the panel.
    fn add_height(&self) -> i32 {
        if self.tab == Tab::Friends {
            ADD_WIDGET_HEIGHT
        } else {
            0
        }
    }

    /// `AddFriendWidget`'s box and its button.
    fn box_rect(&self) -> (i32, i32, i32, i32) {
        let (left, top, ..) = self.content();
        let width = OVERLAY_WIDTH - BUTTON_SIZE - INPUT_SPACING - 2 * INPUT_LEFT;
        (left + INPUT_LEFT, top + INPUT_TOP, width, BOX_HEIGHT)
    }

    fn button_rect(&self) -> (i32, i32, i32, i32) {
        let (x, y, width, _) = self.box_rect();
        (x + width + INPUT_SPACING, y, BUTTON_SIZE, BUTTON_SIZE)
    }

    /// The box can be typed in: the list is loaded and no request is on its way.
    fn box_active(&self) -> bool {
        self.view.ready && !self.view.sending
    }

    /// The send button shows the loading mark (`State.SENDING`): a request is on its way, or the
    /// list is loading.
    pub(super) fn button_loading(&self) -> bool {
        self.pressed == Some(Pressed::Send) || self.view.friends == Listing::Loading
    }

    /// The button can be pressed (`AddFriendWidget.State.READY`).
    fn button_active(&self) -> bool {
        self.box_active() && !self.edit.value().trim().is_empty()
    }
}

/// The pending tab's title (`gui.friends.requests_count`).
fn pending_label(lang: &Lang, requests: usize) -> Vec<Run> {
    let count = Component::literal(requests.to_string());
    lang.runs(
        &Component::translatable_with("gui.friends.requests_count", vec![count]),
        &Style::default(),
    )
}

#[cfg(test)]
mod tests;
