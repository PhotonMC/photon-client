//! Photon's home screen: a bar of destinations along the top and a page of game cards under it,
//! over the panorama.
//! It replaces the title screen; what a press chooses is a [`HomeAction`] the menu acts on.

mod page;
mod tabs;

pub use page::GameInfo;

use crate::kit::{Painter, State, grid, inside};
use crate::layout::Rect;
use crate::tokens::{CLASSIC, Skin};
use page::{Hit, Page, Words};
use photon_game::text::Lang;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};
use tabs::{GAP, Tab};

/// What pressing something on the home screen asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeAction {
    Servers,
    Friends,
    Language,
    Accessibility,
    Options,
    Quit,
    Credits,
    /// The play arrow of the directory's server at this index: straight into the game.
    Join(usize),
    /// The card of the directory's server at this index. The home screen opens its page itself.
    OpenGame(usize),
    /// Play on the open game page; [`HomeScreen::open_game_id`] says which game.
    PlayOpen,
}

const BAR_HEIGHT: i32 = 26;
const TAB_HEIGHT: i32 = 18;
const LOGO_WIDTH: i32 = 64;
const CARD_SIZE: (i32, i32) = (148, 88);
const MARGIN: i32 = 12;
const ARROW: i32 = 16;

/// How a target is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Look {
    Tab,
    /// The tab of the page that is open.
    Current,
    Quit,
    Card,
}

/// Something the player can point at: its place, what it does, and whether it can be pressed.
#[derive(Debug, Clone)]
struct Target {
    rect: Rect,
    label: String,
    detail: String,
    look: Look,
    action: Option<HomeAction>,
    enabled: bool,
}

/// The top bar's tabs: translation key and action (none for the page that is open, or not
/// available yet).
const DESTINATIONS: [(&str, Option<HomeAction>); 6] = [
    ("home", None),
    ("servers", Some(HomeAction::Servers)),
    ("friends", Some(HomeAction::Friends)),
    ("avatar", None),
    ("marketplace", None),
    ("settings", Some(HomeAction::Options)),
];

pub struct HomeScreen {
    size: (i32, i32),
    skin: Skin,
    version: String,
    nav: Vec<Target>,
    cards: Vec<Target>,
    quit: Target,
    friends_ready: bool,
    focus: Option<usize>,
    /// The page's title and the line under it.
    heading: (String, String),
    /// The cards' title, detail and what they do: the page's own, then the directory's.
    entries: Vec<(String, String, Option<HomeAction>)>,
    /// The directory's games, as the page shows them.
    games: Vec<GameInfo>,
    /// Each card's play arrow; only the directory's cards have one.
    arrows: Vec<Option<Rect>>,
    words: Words,
    /// The game page, when one is open in place of the cards.
    page: Option<Page>,
}

impl HomeScreen {
    /// The home screen of a window `size` units big in `lang`, its tabs as wide as their labels in
    /// `font`.
    pub fn new(size: (i32, i32), version: &str, font: &Font, lang: &Lang) -> Self {
        let quit_label = lang.text("photon.home.quit");
        let quit_width = font.width(&quit_label) + 14;
        let nav = Self::nav_targets(font, lang, size.0 - quit_width - 12 - LOGO_WIDTH);
        let quit = Target {
            rect: Rect {
                x: size.0 - quit_width - 6,
                y: (BAR_HEIGHT - TAB_HEIGHT) / 2,
                width: quit_width,
                height: TAB_HEIGHT,
            },
            label: quit_label,
            detail: String::new(),
            look: Look::Quit,
            action: Some(HomeAction::Quit),
            enabled: true,
        };
        let mut screen = Self {
            size,
            skin: CLASSIC,
            version: version.to_owned(),
            nav,
            cards: Vec::new(),
            quit,
            friends_ready: false,
            focus: None,
            entries: Vec::new(),
            games: Vec::new(),
            arrows: Vec::new(),
            words: Words::new(lang),
            page: None,
            heading: (
                lang.text("photon.home.heading"),
                lang.text("photon.home.subtitle"),
            ),
        };
        let text = |key: &str| lang.text(&format!("photon.home.{key}"));
        screen.entries = vec![
            (text("join"), text("join.detail"), Some(HomeAction::Servers)),
            (text("games"), text("games.detail"), None),
        ];
        screen.cards = screen.card_targets();
        screen.sync_friends();
        screen
    }

    /// The tabs of the top bar that fit in `room`, side by side after the logo.
    fn nav_targets(font: &Font, lang: &Lang, room: i32) -> Vec<Target> {
        let tabs = DESTINATIONS
            .iter()
            .map(|&(key, action)| Tab {
                key,
                label: lang.text(&format!("photon.home.tab.{key}")),
                action,
            })
            .collect();
        let fit = tabs::fit(tabs, &|text| font.width(text), room);
        let mut x = LOGO_WIDTH;
        fit.tabs
            .into_iter()
            .map(|tab| {
                let width = font.width(&tab.label) + fit.padding;
                let rect = Rect {
                    x,
                    y: (BAR_HEIGHT - TAB_HEIGHT) / 2,
                    width,
                    height: TAB_HEIGHT,
                };
                x += width + GAP;
                Target {
                    rect,
                    look: if tab.key == "home" {
                        Look::Current
                    } else {
                        Look::Tab
                    },
                    label: tab.label,
                    detail: String::new(),
                    action: tab.action,
                    enabled: tab.action.is_some(),
                }
            })
            .collect()
    }

    /// The page's cards. The directory is not connected yet, so the first one adds a server by
    /// address and the second says what is coming.
    fn card_targets(&mut self) -> Vec<Target> {
        let area = Rect {
            x: MARGIN,
            y: BAR_HEIGHT + 40,
            width: self.size.0 - 2 * MARGIN,
            height: self.size.1 - BAR_HEIGHT - 52,
        };
        let rects = grid::cards(area, self.entries.len(), CARD_SIZE, 8);
        self.arrows = rects
            .iter()
            .zip(&self.entries)
            .map(|(rect, (_, _, action))| {
                matches!(action, Some(HomeAction::OpenGame(_))).then_some(Rect {
                    x: rect.x + rect.width - ARROW - 4,
                    y: rect.y + rect.height - ARROW - 4,
                    width: ARROW,
                    height: ARROW,
                })
            })
            .collect();
        rects
            .into_iter()
            .zip(self.entries.iter().cloned())
            .map(|(rect, (label, detail, action))| Target {
                rect,
                label,
                detail,
                look: Look::Card,
                action,
                enabled: action.is_some(),
            })
            .collect()
    }

    /// The directory's servers, as the cards after the first. Without any the page keeps its
    /// placeholder card. A game page that is open keeps the game it was opened with.
    pub fn set_directory(&mut self, games: Vec<GameInfo>) {
        if games.is_empty() {
            return;
        }
        self.entries.truncate(1);
        self.entries
            .extend(games.iter().enumerate().map(|(index, game)| {
                (
                    game.name.clone(),
                    game.detail.clone(),
                    Some(HomeAction::OpenGame(index)),
                )
            }));
        self.games = games;
        self.cards = self.card_targets();
        self.focus = None;
    }

    /// A directory that could not be read: the placeholder card says so instead of "coming soon".
    pub fn set_directory_unavailable(&mut self, detail: &str) {
        if let Some((_, text, None)) = self.entries.get_mut(1) {
            *text = detail.to_owned();
            self.cards = self.card_targets();
        }
    }

    fn sync_friends(&mut self) {
        let ready = self.friends_ready;
        for target in &mut self.nav {
            if target.action == Some(HomeAction::Friends) {
                target.enabled = ready;
            }
        }
    }

    /// The friends service has an account: the Friends row can be pressed.
    pub fn friends_available(&mut self) {
        self.friends_ready = true;
        self.sync_friends();
    }

    /// The window size the layout was made for.
    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// Everything that can be pressed, in the order Tab walks through it.
    fn targets(&self) -> impl Iterator<Item = &Target> {
        self.nav
            .iter()
            .chain(self.cards.iter())
            .chain(std::iter::once(&self.quit))
    }

    fn enabled_indices(&self) -> Vec<usize> {
        self.targets()
            .enumerate()
            .filter(|(_, target)| target.enabled)
            .map(|(index, _)| index)
            .collect()
    }

    /// The pressable thing under the mouse, if any.
    fn target_at(&self, mouse: (i32, i32)) -> Option<usize> {
        self.targets()
            .position(|target| target.enabled && inside(target.rect, mouse))
    }

    /// The id of the game whose page is open: what [`HomeAction::PlayOpen`] plays.
    pub fn open_game_id(&self) -> Option<&str> {
        self.page.as_ref().map(|page| page.game.id.as_str())
    }

    /// The video of the open game page, which the app plays into the surface texture.
    pub fn wanted_video(&self) -> Option<&str> {
        self.page.as_ref()?.game.video.as_deref()
    }

    /// Says whether the surface texture holds a picture of that video, to be drawn on the page.
    pub fn set_video_live(&mut self, live: bool) {
        if let Some(page) = self.page.as_mut() {
            page.live = live;
        }
    }

    /// Escape: an open game page closes. Whether there was one.
    pub fn escape(&mut self) -> bool {
        self.page.take().is_some()
    }

    /// Opens the page of the game with this id. Whether the home screen has one.
    pub fn open_game_by_id(&mut self, id: &str) -> bool {
        let Some(index) = self.games.iter().position(|game| game.id == id) else {
            return false;
        };
        self.open_game(index);
        true
    }

    fn open_game(&mut self, index: usize) {
        if let Some(game) = self.games.get(index).cloned() {
            self.page = Some(Page::new(self.size, game, &self.games, self.words.clone()));
        }
    }

    /// What a press on the open page chose: another page, the way back, or playing.
    fn page_hit(&mut self, hit: Hit) -> Option<HomeAction> {
        crate::ui_sound::play_down_sound();
        match hit {
            Hit::Play => return Some(HomeAction::PlayOpen),
            Hit::Back => self.page = None,
            Hit::Other(slot) => {
                let other = self
                    .page
                    .as_ref()
                    .and_then(|page| page.other(slot))
                    .cloned();
                if let Some(game) = other {
                    self.page = Some(Page::new(self.size, game, &self.games, self.words.clone()));
                }
            }
        }
        None
    }

    /// Opening a game's page is the home screen's own business; the rest goes on to the menu.
    fn resolved(&mut self, action: Option<HomeAction>) -> Option<HomeAction> {
        if let Some(HomeAction::OpenGame(index)) = action {
            self.open_game(index);
            return None;
        }
        action
    }

    /// The play arrow under the mouse: its game's place in the directory.
    fn arrow_at(&self, mouse: (i32, i32)) -> Option<usize> {
        if self.page.is_some() {
            return None;
        }
        // Entry 0 is the page's own card; the directory's games follow it.
        let at = self
            .arrows
            .iter()
            .position(|arrow| arrow.is_some_and(|rect| inside(rect, mouse)))?;
        at.checked_sub(1)
    }

    /// A mouse press: what the thing under it chose.
    pub fn clicked(&mut self, mouse: (i32, i32)) -> Option<HomeAction> {
        if let Some(hit) = self.page.as_ref().and_then(|page| page.hit_at(mouse)) {
            return self.page_hit(hit);
        }
        if let Some(game) = self.arrow_at(mouse) {
            crate::ui_sound::play_down_sound();
            return Some(HomeAction::Join(game));
        }
        let index = self.target_at(mouse)?;
        if self.page.is_some()
            && self
                .targets()
                .nth(index)
                .is_some_and(|t| t.look == Look::Card)
        {
            return None;
        }
        self.focus = Some(index);
        crate::ui_sound::play_down_sound();
        let action = self.targets().nth(index).and_then(|target| target.action);
        self.resolved(action)
    }

    /// Tab (or arrow keys): focus moves to the next pressable thing, or the previous one.
    pub fn step_focus(&mut self, forward: bool) {
        if let Some(page) = &mut self.page {
            page.step(forward);
            return;
        }
        let order = self.enabled_indices();
        let (Some(first), Some(last)) = (order.first().copied(), order.last().copied()) else {
            return;
        };
        let current = self
            .focus
            .and_then(|focus| order.iter().position(|&i| i == focus));
        self.focus = Some(match (current, forward) {
            (None, true) => first,
            (None, false) => last,
            (Some(at), true) => order[(at + 1) % order.len()],
            (Some(at), false) => order[(at + order.len() - 1) % order.len()],
        });
    }

    /// Enter or Space: what the focused thing chose.
    pub fn pressed(&mut self) -> Option<HomeAction> {
        if let Some(page) = &self.page {
            let hit = page.pressed();
            return self.page_hit(hit);
        }
        let focus = self.focus?;
        let action = self
            .targets()
            .nth(focus)
            .filter(|target| target.enabled)
            .and_then(|target| target.action)?;
        crate::ui_sound::play_down_sound();
        self.resolved(Some(action))
    }

    /// The panorama's dimming, the top bar, the page and the version.
    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        let mut painter = Painter {
            frame,
            atlas,
            font,
            skin: &self.skin,
        };
        let dim = (self.skin.bg & 0x00FF_FFFF) | 0x3300_0000;
        painter.frame.fill(atlas, (0, 0, width, height), dim);
        painter.panel(Rect {
            x: 0,
            y: 0,
            width,
            height: BAR_HEIGHT,
        });
        let (ink, muted) = (painter.skin.ink, painter.skin.muted);
        painter.label("PHOTON", (8, 4), ink);
        painter.label(&self.version, (8, 14), muted);
        let hovered = self.target_at(mouse);
        for (index, target) in self.targets().enumerate() {
            if self.page.is_some() && target.look == Look::Card {
                continue;
            }
            let focused = self.focus == Some(index) || hovered == Some(index);
            Self::draw_target(&mut painter, target, focused);
        }
        if let Some(page) = &self.page {
            page.draw(&mut painter, mouse);
            return;
        }
        self.draw_arrows(&mut painter, mouse);
        painter.label(&self.heading.0, (MARGIN, BAR_HEIGHT + 10), ink);
        painter.label(&self.heading.1, (MARGIN, BAR_HEIGHT + 24), muted);
    }

    /// The play arrow of the card the mouse or the focus is on.
    fn draw_arrows(&self, painter: &mut Painter, mouse: (i32, i32)) {
        let focused = self.focus.and_then(|at| at.checked_sub(self.nav.len()));
        for (at, (card, arrow)) in self.cards.iter().zip(&self.arrows).enumerate() {
            let Some(rect) = *arrow else { continue };
            if !inside(card.rect, mouse) && focused != Some(at) {
                continue;
            }
            let state = if inside(rect, mouse) {
                State::Hover
            } else {
                State::Normal
            };
            painter.button(rect, "", state);
            let ink = painter.skin.button_ink;
            for row in 0..8_i32 {
                let width = (9 - (2 * row - 7).abs()) / 2;
                let (x, y) = (rect.x + 6, rect.y + 4 + row);
                painter.fill_rect(
                    Rect {
                        x,
                        y,
                        width,
                        height: 1,
                    },
                    ink | 0xFF00_0000,
                );
            }
        }
    }

    fn draw_target(painter: &mut Painter, target: &Target, focused: bool) {
        let state = match (target.enabled, focused) {
            (false, _) => State::Disabled,
            (true, true) => State::Hover,
            (true, false) => State::Normal,
        };
        match target.look {
            Look::Quit => painter.button(target.rect, &target.label, state),
            Look::Current => painter.row(target.rect, &target.label, State::Selected),
            Look::Tab => painter.row(target.rect, &target.label, state),
            Look::Card => painter.card(target.rect, (&target.label, &target.detail), state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> HomeScreen {
        HomeScreen::new((427, 240), "v0.1.0", &Font::default(), &Lang::default())
    }

    fn centre(rect: Rect) -> (i32, i32) {
        (rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    #[test]
    fn a_narrow_window_drops_unavailable_tabs_before_it_overlaps_quit() {
        let home = HomeScreen::new((180, 240), "v0.1.0", &Font::default(), &Lang::default());
        let last = home.nav.last().expect("tabs");
        assert!(last.rect.x + last.rect.width <= home.quit.rect.x);
        assert!(
            home.nav
                .iter()
                .any(|tab| tab.action == Some(HomeAction::Options))
        );
        assert!(home.nav.len() < DESTINATIONS.len());
    }

    #[test]
    fn pressing_the_servers_row_asks_for_the_server_list() {
        let mut home = screen();
        let at = centre(home.nav[1].rect);
        assert_eq!(home.clicked(at), Some(HomeAction::Servers));
    }

    #[test]
    fn a_row_that_is_not_available_does_nothing() {
        let mut home = screen();
        let avatar = centre(home.nav[3].rect);
        assert_eq!(home.clicked(avatar), None);
        // Friends waits for an account.
        let friends = centre(home.nav[2].rect);
        assert_eq!(home.clicked(friends), None);
        home.friends_available();
        assert_eq!(home.clicked(friends), Some(HomeAction::Friends));
    }

    #[test]
    fn quit_and_settings_are_pressable() {
        let mut home = screen();
        let settings = centre(home.nav[5].rect);
        assert_eq!(home.clicked(settings), Some(HomeAction::Options));
        let quit = centre(home.quit.rect);
        assert_eq!(home.clicked(quit), Some(HomeAction::Quit));
    }

    #[test]
    fn tab_walks_the_enabled_things_and_wraps() {
        let mut home = screen();
        // Enabled: Servers, Settings, the first card, Quit.
        let mut chosen = Vec::new();
        for _ in 0..5 {
            home.step_focus(true);
            chosen.push(home.pressed());
        }
        assert_eq!(
            chosen,
            [
                Some(HomeAction::Servers),
                Some(HomeAction::Options),
                Some(HomeAction::Servers),
                Some(HomeAction::Quit),
                Some(HomeAction::Servers),
            ]
        );
        home.step_focus(false);
        assert_eq!(home.pressed(), Some(HomeAction::Quit));
    }

    #[test]
    fn a_press_clicks_once_and_a_dead_row_stays_silent() {
        let mut home = screen();
        crate::ui_sound::take();
        home.clicked(centre(home.nav[3].rect));
        assert!(crate::ui_sound::take().is_empty());
        home.clicked(centre(home.nav[1].rect));
        let sounds = crate::ui_sound::take();
        assert_eq!(sounds.len(), 1);
        home.step_focus(true);
        home.pressed();
        assert_eq!(crate::ui_sound::take().len(), 1);
    }

    #[test]
    fn nothing_is_pressed_before_anything_is_focused() {
        assert_eq!(screen().pressed(), None);
    }

    fn game(name: &str) -> GameInfo {
        GameInfo {
            id: name.to_lowercase(),
            name: name.to_owned(),
            detail: "0 online".to_owned(),
            description: String::new(),
            videos: 0,
            video: None,
            images: 0,
        }
    }

    #[test]
    fn a_directory_card_opens_its_page_and_its_arrow_joins() {
        let mut home = screen();
        home.set_directory(Vec::new());
        assert_eq!(home.cards.len(), 2);
        home.set_directory(vec![game("Skyblock"), game("Arena")]);
        assert_eq!(home.cards.len(), 3);
        // The arrow sits inside the card and wins over it.
        let arrow = home.arrows[2].expect("arrow");
        assert!(inside(home.cards[2].rect, centre(arrow)));
        assert_eq!(home.clicked(centre(arrow)), Some(HomeAction::Join(1)));
        // The rest of the card opens the page and nothing leaves the screen.
        let body = (home.cards[2].rect.x + 3, home.cards[2].rect.y + 3);
        assert_eq!(home.clicked(body), None);
        assert_eq!(home.open_game_id(), Some("arena"));
        let first = centre(home.cards[0].rect);
        assert_eq!(
            home.clicked(first),
            None,
            "the cards are hidden under a page"
        );
        assert!(home.escape());
        assert_eq!(home.clicked(first), Some(HomeAction::Servers));
        // The placeholder cards have no arrow.
        assert!(home.arrows[0].is_none());
    }

    #[test]
    fn a_click_on_an_arrow_sounds_once_and_enter_opens_the_page() {
        let mut home = screen();
        home.set_directory(vec![game("Arena")]);
        crate::ui_sound::take();
        home.clicked(centre(home.arrows[1].expect("arrow")));
        assert_eq!(crate::ui_sound::take().len(), 1);
        // Tab does not stop on arrows; Enter on the card opens the page.
        for _ in 0..4 {
            home.step_focus(true);
        }
        assert_eq!(home.focus, Some(home.nav.len() + 1));
        assert_eq!(home.pressed(), None);
        assert_eq!(home.open_game_id(), Some("arena"));
        // On the page Enter plays, and Back returns.
        assert_eq!(home.pressed(), Some(HomeAction::PlayOpen));
        assert!(home.escape());
        assert_eq!(home.open_game_id(), None);
    }

    #[test]
    fn a_page_keeps_its_game_when_the_directory_changes() {
        let mut home = screen();
        home.set_directory(vec![game("Arena"), game("Skyblock")]);
        home.clicked((home.cards[1].rect.x + 3, home.cards[1].rect.y + 3));
        home.set_directory(vec![game("Skyblock")]);
        assert_eq!(home.open_game_id(), Some("arena"));
    }

    #[test]
    fn an_unavailable_directory_changes_only_the_placeholder_card() {
        let mut home = screen();
        home.set_directory_unavailable("Directory unavailable");
        assert_eq!(home.entries[1].1, "Directory unavailable");
        assert_eq!(home.entries[1].2, None);
        home.set_directory(vec![game("Arena")]);
        home.set_directory_unavailable("Directory unavailable");
        assert_eq!(home.entries[1].0, "Arena");
        assert_eq!(home.entries[1].1, "0 online");
    }
}
