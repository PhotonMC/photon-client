//! The social interactions screen (`SocialInteractionsScreen`, `SocialInteractionsPlayerList`,
//! `PlayerEntry`): the players of the server on three pages (all, hidden, blocked), a search over
//! their names, and for each a button to hide them from the chat or show them again, and one to
//! report them.
//!
//! Not drawn: players the chat log remembers (their profiles come from the chat), the tooltips of
//! the entry buttons, and a skin other than a default one.

mod draw;
mod input;

use std::cell::Cell;
use std::time::Instant;

use photon_core::binary::Uuid;
use photon_game::text::{Component, Lang, Run, Style};

use crate::entity::player_renderer::default_skin;
use crate::gui::font::Font;
use crate::gui::layout::{Item, place_in_frame};
use crate::gui::widgets::button_set::ButtonSet;
use crate::gui::widgets::edit_box::EditBox;
use crate::gui::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

/// The header and footer the title and the Done button sit in (`HeaderAndFooterLayout`).
const HEADER_HEIGHT: i32 = 33;
const FOOTER_HEIGHT: i32 = 33;
/// The list's rows, `getRowWidth`, and where it starts.
const ENTRY_HEIGHT: i32 = 36;
const ROW_WIDTH: i32 = 220;
const LIST_START: i32 = 88;
/// `IMAGE_WIDTH`, and the window sprite's width.
const IMAGE_WIDTH: i32 = 238;
const BACKGROUND_WIDTH: i32 = 236;
const SKIN_SIZE: i32 = 24;
/// `PlayerEntry`'s colours: the row, the row of a player who has left, the status text, and the
/// shade over a face that has left.
const BG_FILL: u32 = 0xFF4A_4A4A;
const BG_FILL_REMOVED: u32 = 0xFF30_3030;
const PLAYER_STATUS_COLOUR: u32 = 0x8CFF_FFFF;
const SKIN_SHADE: u32 = 0xBE00_0000;

/// The pages of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    All,
    Hidden,
    Blocked,
}

/// What a button of the screen asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocialAction {
    Page(Page),
    Done,
}

/// What the screen asks the game for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocialEvent {
    /// The screen closes (`onClose`).
    Close,
    /// A player was hidden from the chat or shown again (`PlayerEntry`'s buttons).
    Hidden {
        id: Uuid,
        name: String,
        hidden: bool,
    },
    /// The report button of a player (the report screen needs the online services).
    Report(Uuid),
    /// The "Manage with Microsoft account" button: a link to confirm.
    BlockingHelp,
}

/// Where the keyboard is (`Screen.getFocused`): the search box at first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Search,
    /// A button of the screen's set, by its index.
    Button(usize),
    Hint,
    List,
}

/// A player the screen can list.
#[derive(Debug, Clone)]
pub struct SocialPlayer {
    pub id: Uuid,
    pub name: String,
    /// Whether the chat log holds a signed message of theirs (`hasRecentMessages`).
    pub recent_messages: bool,
    /// Whether a report of them is being drafted (`hasDraftReportFor`).
    pub draft_report: bool,
}

/// What the game knows of the players when the screen opens.
#[derive(Debug, Clone)]
pub struct Roster {
    /// The name of the server the player is on.
    pub server: String,
    /// The players of the server, the local player among them.
    pub online: Vec<SocialPlayer>,
    /// The players seen on the server who have left (`getSeenPlayers`).
    pub gone: Vec<SocialPlayer>,
    /// The players hidden from the chat (`getHiddenPlayers`).
    pub hidden: Vec<Uuid>,
    pub local: Uuid,
    /// Whether reports can be sent (`AbuseReportSender.isEnabled`).
    pub reporting: bool,
}

/// A row of the list.
#[derive(Debug, Clone)]
struct Entry {
    player: SocialPlayer,
    removed: bool,
}

/// Which of the row statuses a player has (`getStatusComponent`); none is the first.
fn status_index(hidden: bool, removed: bool) -> usize {
    usize::from(removed) + 2 * usize::from(hidden)
}

/// `String.compareToIgnoreCase`.
fn compare_ignore_case(a: &str, b: &str) -> std::cmp::Ordering {
    let fold = |c: char| {
        let upper = c.to_uppercase().next().unwrap_or(c);
        upper.to_lowercase().next().unwrap_or(upper)
    };
    a.chars().map(fold).cmp(b.chars().map(fold))
}

/// The widgets `init` adds: the three tabs (the one for `page` underlined), the Done button in
/// the footer, the title in the header and the blocking hint under the window.
fn layout(
    font: &Font,
    lang: &Lang,
    size: (i32, i32),
    (page, labels): (Page, &[[Vec<Run>; 2]; 3]),
) -> (ButtonSet<SocialAction>, Widget, Widget) {
    let runs = |component: &Component| lang.runs(component, &Style::default());
    let key = |key: &str| runs(&Component::translatable(key));
    let (width, height) = size;
    let window_height = (height - 128 - 16).max(52);
    let row_left = width / 2 - ROW_WIDTH / 2;
    let row_right = row_left + ROW_WIDTH;
    let tab_width = ROW_WIDTH / 3;
    let mut buttons = ButtonSet::new(size);
    let tabs = [
        (Page::All, row_left),
        (Page::Hidden, (row_left + row_right - tab_width) / 2 + 1),
        (Page::Blocked, row_right - tab_width + 1),
    ];
    for (index, (tab, x)) in tabs.into_iter().enumerate() {
        let label = labels[index][usize::from(tab == page)].clone();
        buttons.add(
            (x, 45, tab_width, 20),
            (Kind::Button { label }, true),
            SocialAction::Page(tab),
            None,
        );
    }
    let hint_rect = buttons.arena.add(200, 20);
    buttons.arena.0[hint_rect].x = width / 2 - 100;
    buttons.arena.0[hint_rect].y = 64 + window_height;
    let blocking_hint = Widget {
        rect: hint_rect,
        active: true,
        kind: Kind::Button {
            label: key("gui.socialInteractions.blocking_hint"),
        },
    };
    let title_text = key("gui.socialInteractions.title");
    let title_rect = buttons.arena.add(font.width_of_runs(&title_text), 9);
    place_in_frame(
        &mut buttons.arena,
        Item::Leaf(title_rect),
        (0, 0, width, HEADER_HEIGHT.max(9)),
        (0.5, 0.5),
    );
    let title = Widget {
        rect: title_rect,
        active: false,
        kind: Kind::Text { text: title_text },
    };
    let done = buttons.add(
        (0, 0, 200, 20),
        (
            Kind::Button {
                label: key("gui.done"),
            },
            true,
        ),
        SocialAction::Done,
        None,
    );
    place_in_frame(
        &mut buttons.arena,
        Item::Leaf(done),
        (0, height - FOOTER_HEIGHT, width, FOOTER_HEIGHT),
        (0.5, 0.5),
    );
    (buttons, title, blocking_hint)
}

pub struct SocialInteractionsScreen {
    buttons: ButtonSet<SocialAction>,
    title: Widget,
    /// The "Manage with Microsoft account" button, which only the blocked page shows (after the
    /// first frame, which still draws it).
    blocking_hint: Widget,
    hint_visible: Cell<bool>,
    search: EditBox,
    focused_at: Instant,
    focus: Focus,
    /// The search box's hint, shown while it is empty and has not the focus.
    search_hint: Vec<Run>,
    /// The tab labels, plain and underlined, by tab.
    tab_labels: [[Vec<Run>; 2]; 3],
    /// Whether the list's scroll bar is held (`AbstractScrollArea.scrolling`).
    scrolling: bool,
    last_search: String,
    page: Page,
    roster: Roster,
    entries: Vec<Entry>,
    place: Place,
    scroll: f64,
    server_label: Vec<Run>,
    /// The two empty-list texts, for the search, the hidden page and the blocked page.
    empty: [Vec<Run>; 3],
    /// The status of a row, by whether the player is hidden and whether they have left.
    statuses: [Vec<Run>; 4],
    size: (i32, i32),
}

impl SocialInteractionsScreen {
    /// `init` over `roster`, on `page`.
    pub fn new(font: &Font, lang: &Lang, size: (i32, i32), roster: Roster, page: Page) -> Self {
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let status = |name: &str| runs(&Component::translatable(name).italic());
        let grey = |name: &str| runs(&Component::translatable(name).with_colour("gray"));
        let (width, height) = size;
        let list_end = 80 + (height - 128 - 16).max(52) - 8;
        let tab_labels = ["tab_all", "tab_hidden", "tab_blocked"].map(|name| {
            let name = format!("gui.socialInteractions.{name}");
            [
                runs(&Component::translatable(name.clone())),
                runs(&Component::translatable(name).with_style(&Style {
                    underlined: Some(true),
                    ..Style::default()
                })),
            ]
        });
        let (buttons, title, blocking_hint) = layout(font, lang, size, (page, &tab_labels));
        let search_hint = runs(
            &Component::translatable("gui.socialInteractions.search_hint")
                .with_colour("gray")
                .italic(),
        );
        let mut screen = Self {
            buttons,
            title,
            blocking_hint,
            hint_visible: Cell::new(true),
            // `EditBox.getInnerWidth` of the 200 wide box.
            search: EditBox::new(192, 16).with_colour(0xFFFF_FFFF),
            focused_at: Instant::now(),
            focus: Focus::Search,
            search_hint,
            tab_labels,
            scrolling: false,
            last_search: String::new(),
            page,
            roster,
            entries: Vec::new(),
            place: Place {
                y: LIST_START,
                width,
                height: list_end - LIST_START,
                in_world: true,
            },
            scroll: 0.0,
            server_label: Vec::new(),
            empty: [
                grey("gui.socialInteractions.search_empty"),
                grey("gui.socialInteractions.empty_hidden"),
                grey("gui.socialInteractions.empty_blocked"),
            ],
            statuses: [
                Vec::new(),
                status("gui.socialInteractions.status_offline"),
                status("gui.socialInteractions.status_hidden"),
                status("gui.socialInteractions.status_hidden_offline"),
            ],
            size,
        };
        screen.server_label = screen.label(lang);
        screen.show_page(page);
        screen
    }

    /// `updateServerLabel`: the server and how many are on it, none while nobody is.
    fn label(&self, lang: &Lang) -> Vec<Run> {
        let count = self.roster.online.len();
        let key = if count > 1 {
            "gui.socialInteractions.server_label.multiple"
        } else {
            "gui.socialInteractions.server_label.single"
        };
        if count == 0 {
            return Vec::new();
        }
        let label = Component::translatable_with(
            key,
            vec![
                Component::literal(self.roster.server.clone()),
                Component::literal(count.to_string()),
            ],
        );
        lang.runs(&label, &Style::default())
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The screen laid out again for another window size (`Screen.resize`): the page, the roster
    /// as it is now and the search text are kept, the scroll and the focus are not. (The reference
    /// leaves the list unfiltered until the text changes again.)
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        let mut screen = Self::new(font, lang, size, self.roster.clone(), self.page);
        screen.search.set_value(&self.search.value(), font);
        screen.last_search.clone_from(&self.last_search);
        screen.show_page(self.page);
        screen
    }

    fn window_height(&self) -> i32 {
        (self.size.1 - 128 - 16).max(52)
    }

    fn list_end(&self) -> i32 {
        80 + self.window_height() - 8
    }

    fn margin_x(&self) -> i32 {
        (self.size.0 - IMAGE_WIDTH) / 2
    }

    fn row_left(&self) -> i32 {
        self.size.0 / 2 - ROW_WIDTH / 2
    }

    fn hidden(&self, id: Uuid) -> bool {
        self.roster.hidden.contains(&id)
    }

    /// `showPage`: the list for `page` from the roster, filtered by the search.
    fn show_page(&mut self, page: Page) {
        self.page = page;
        let online = |ids: &[Uuid]| -> Vec<Entry> {
            self.roster
                .online
                .iter()
                .filter(|player| ids.contains(&player.id))
                .map(|player| Entry {
                    player: player.clone(),
                    removed: false,
                })
                .collect()
        };
        let mut entries = match page {
            Page::All => {
                let ids: Vec<Uuid> = self.roster.online.iter().map(|player| player.id).collect();
                let mut entries = online(&ids);
                for player in &self.roster.gone {
                    if !ids.contains(&player.id) {
                        entries.push(Entry {
                            player: player.clone(),
                            removed: true,
                        });
                    }
                }
                entries
            }
            Page::Hidden => online(&self.roster.hidden),
            // Blocking needs the online services.
            Page::Blocked => Vec::new(),
        };
        entries.sort_by(|a, b| self.compare(&a.player, &b.player));
        let filter = &self.last_search;
        entries.retain(|entry| entry.player.name.to_lowercase().contains(filter.as_str()));
        self.entries = entries;
        self.set_scroll(self.scroll);
    }

    /// `sortPlayerEntries`: the local player, a draft report, those who spoke, the rest, offline
    /// profiles last; names that start with a letter, digit or `_` before the others.
    fn compare(&self, a: &SocialPlayer, b: &SocialPlayer) -> std::cmp::Ordering {
        let rank = |player: &SocialPlayer| {
            if player.id == self.roster.local {
                0
            } else if player.draft_report {
                1
            } else if (player.id.most >> 12) & 0xF == 2 {
                4
            } else if player.recent_messages {
                2
            } else {
                3
            }
        };
        let plain = |player: &SocialPlayer| {
            let first = player.name.chars().next();
            let named = first.is_some_and(|c| c == '_' || c.is_ascii_alphanumeric())
                && !player.name.trim().is_empty();
            i32::from(!named)
        };
        rank(a)
            .cmp(&rank(b))
            .then(plain(a).cmp(&plain(b)))
            .then_with(|| compare_ignore_case(&a.name, &b.name))
    }

    fn content_height(&self) -> i32 {
        self.entries.len() as i32 * ENTRY_HEIGHT + 4
    }

    fn entry_y(&self, index: usize) -> i32 {
        self.place.y + 2 - self.scroll as i32 + index as i32 * ENTRY_HEIGHT
    }
}
