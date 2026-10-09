//! The social interactions screen (`SocialInteractionsScreen`, `SocialInteractionsPlayerList`,
//! `PlayerEntry`): the players of the server on three pages (all, hidden, blocked), a search over
//! their names, and for each a button to hide them from the chat or show them again, and one to
//! report them.
//!
//! Not drawn: players the chat log remembers (their profiles come from the chat), the tooltips of
//! the entry buttons, and a skin other than a default one.

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
    page: Page,
) -> (ButtonSet<SocialAction>, Widget, Widget) {
    let runs = |component: &Component| lang.runs(component, &Style::default());
    let key = |key: &str| runs(&Component::translatable(key));
    let underlined = |name: &str| {
        runs(&Component::translatable(name).with_style(&Style {
            underlined: Some(true),
            ..Style::default()
        }))
    };
    let (width, height) = size;
    let window_height = (height - 128 - 16).max(52);
    let row_left = width / 2 - ROW_WIDTH / 2;
    let row_right = row_left + ROW_WIDTH;
    let tab_width = ROW_WIDTH / 3;
    let mut buttons = ButtonSet::new(size);
    let tabs = [
        ("gui.socialInteractions.tab_all", Page::All, row_left),
        (
            "gui.socialInteractions.tab_hidden",
            Page::Hidden,
            (row_left + row_right - tab_width) / 2 + 1,
        ),
        (
            "gui.socialInteractions.tab_blocked",
            Page::Blocked,
            row_right - tab_width + 1,
        ),
    ];
    for (name, tab, x) in tabs {
        let label = if tab == page {
            underlined(name)
        } else {
            key(name)
        };
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
        let (buttons, title, blocking_hint) = layout(font, lang, size, page);
        let mut screen = Self {
            buttons,
            title,
            blocking_hint,
            hint_visible: Cell::new(true),
            search: EditBox::new(200, 16).with_colour(0xFFFF_FFFF),
            focused_at: Instant::now(),
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

    /// The text typed into the search box, which filters the list when it changes
    /// (`checkSearchStringUpdate`).
    pub fn type_search(&mut self, text: &str, font: &Font) {
        for character in text.chars() {
            self.search.char_typed(character, font);
        }
        let lowered = self.search.value().to_lowercase();
        if lowered != self.last_search {
            self.last_search = lowered;
            self.show_page(self.page);
        }
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

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        // `extractBackground` in a game: the dimmed world behind, then the window.
        frame.tile_texture(
            atlas,
            "inworld_menu_background",
            (32, 32),
            (0, 0, width, height),
        );
        let window_x = self.margin_x() + 3;
        frame.sprite(
            atlas,
            "social_interactions/background",
            window_x,
            64,
            BACKGROUND_WIDTH,
            self.window_height() + 16,
        );
        frame.sprite(atlas, "icon/search", window_x + 10, 76, 12, 12);
        self.buttons.draw(frame, atlas, font, mouse);
        self.title
            .draw(frame, atlas, font, &self.buttons.arena, false);
        let (field_x, field_y) = (self.margin_x() + 28, 74);
        frame.nine_slice(
            atlas,
            "widget/text_field_highlighted",
            (200, 20),
            1,
            false,
            field_x,
            field_y,
            200,
            15,
        );
        let text_at = (field_x + 4, field_y + (15 - 8) / 2);
        self.search
            .draw_formatted(frame, atlas, font, text_at, Some(self.focused_at), None);
        if self.hint_visible.get() {
            let highlighted = self.blocking_hint.contains(&self.buttons.arena, mouse);
            self.blocking_hint
                .draw(frame, atlas, font, &self.buttons.arena, highlighted);
        }
        if !self.server_label.is_empty() {
            frame.styled_text(
                atlas,
                font,
                &self.server_label,
                self.margin_x() + 8,
                35,
                0xFFFF_FFFF,
                true,
            );
        }
        if !self.entries.is_empty() {
            self.draw_list(frame, atlas, font, mouse);
        } else {
            let empty = if !self.search.is_empty() {
                Some(&self.empty[0])
            } else {
                match self.page {
                    Page::All => None,
                    Page::Hidden => Some(&self.empty[1]),
                    Page::Blocked => Some(&self.empty[2]),
                }
            };
            if let Some(text) = empty {
                let x = width / 2 - font.width_of_runs(text) / 2;
                frame.styled_text(
                    atlas,
                    font,
                    text,
                    x,
                    (72 + self.list_end()) / 2,
                    0xFFFF_FFFF,
                    true,
                );
            }
        }
        self.hint_visible.set(self.page == Page::Blocked);
    }

    /// The list: its rows, cut to the list's window, and its scrollbar.
    fn draw_list(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let place = self.place;
        let left = self.row_left();
        let mark = frame.mark();
        let window = (0, place.y + 4, place.width, place.bottom());
        let in_window = mouse.1 >= window.1 && mouse.1 < window.3;
        for (index, entry) in self.entries.iter().enumerate() {
            let y = self.entry_y(index);
            if y + ENTRY_HEIGHT < place.y || y > place.bottom() {
                continue;
            }
            self.draw_entry(
                frame,
                atlas,
                font,
                entry,
                (left + 2, y + 2),
                (mouse, in_window),
            );
        }
        frame.clip_from(mark, window);
        place.draw_scrollbar(
            frame,
            atlas,
            left + ROW_WIDTH + SCROLLBAR_WIDTH + 2,
            self.scroll,
            self.content_height(),
        );
    }

    /// `PlayerEntry.extractContent`: the row, the face, the name and status, and the buttons.
    fn draw_entry(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        entry: &Entry,
        (x, y): (i32, i32),
        (mouse, in_window): ((i32, i32), bool),
    ) {
        let (content_width, content_height) = (ROW_WIDTH - 4, ENTRY_HEIGHT - 4);
        let skin_x = x + 4;
        let skin_y = y + (content_height - SKIN_SIZE) / 2;
        let text_x = skin_x + SKIN_SIZE + 4;
        let hidden = self.hidden(entry.player.id);
        let status = &self.statuses[status_index(hidden, entry.removed)];
        let rect = (x, y, x + content_width, y + content_height);
        let text_y = if status.is_empty() {
            frame.fill(atlas, rect, BG_FILL);
            y + (content_height - 9) / 2
        } else {
            frame.fill(atlas, rect, BG_FILL_REMOVED);
            let text_y = y + (content_height - 18) / 2;
            frame.styled_text(
                atlas,
                font,
                status,
                text_x,
                text_y + 12,
                PLAYER_STATUS_COLOUR,
                true,
            );
            text_y
        };
        let texture = format!("textures/{}", default_skin(entry.player.id).texture);
        for u in [8, 40] {
            frame.blit_texture_stretched(
                atlas,
                &texture,
                (64, 64),
                (u, 8, 8, 8),
                (skin_x, skin_y, SKIN_SIZE, SKIN_SIZE),
            );
        }
        let name = [Run {
            text: entry.player.name.clone(),
            style: Style::default(),
        }];
        frame.styled_text(atlas, font, &name, text_x, text_y, 0xFFFF_FFFF, true);
        if entry.removed {
            frame.fill(
                atlas,
                (skin_x, skin_y, skin_x + SKIN_SIZE, skin_y + SKIN_SIZE),
                SKIN_SHADE,
            );
        }
        if entry.player.id == self.roster.local {
            return;
        }
        let button_y = y + (content_height - 20) / 2;
        let report_x = x + content_width - 24;
        let hide_x = report_x - 24;
        let over = |button_x: i32| {
            in_window
                && mouse.0 >= button_x
                && mouse.0 < button_x + 20
                && mouse.1 >= button_y
                && mouse.1 < button_y + 20
        };
        let (hide_sprite, hide_lit) = if hidden {
            ("unmute_button", "unmute_button_highlighted")
        } else {
            ("mute_button", "mute_button_highlighted")
        };
        let sprite = if over(hide_x) { hide_lit } else { hide_sprite };
        frame.sprite(
            atlas,
            &format!("social_interactions/{sprite}"),
            hide_x,
            button_y,
            20,
            20,
        );
        let report = match (self.roster.reporting, over(report_x)) {
            (false, _) => "report_button_disabled",
            (true, true) => "report_button_highlighted",
            (true, false) => "report_button",
        };
        frame.sprite(
            atlas,
            &format!("social_interactions/{report}"),
            report_x,
            button_y,
            20,
            20,
        );
        if entry.player.draft_report {
            frame.sprite(
                atlas,
                "icon/draft_report",
                report_x + 5,
                button_y + 1,
                15,
                15,
            );
        }
    }
}
