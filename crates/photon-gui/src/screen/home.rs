//! Photon's home screen: a sidebar of destinations and a page of game cards over the panorama.
//! It replaces the title screen; what a press chooses is a [`HomeAction`] the menu acts on.

use crate::kit::{Painter, State, grid, inside};
use crate::layout::Rect;
use crate::tokens::{CLASSIC, Skin};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

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
}

const SIDEBAR_WIDTH: i32 = 96;
const ROW_HEIGHT: i32 = 16;
const CARD_SIZE: (i32, i32) = (148, 88);
const MARGIN: i32 = 12;

/// Something the player can point at: its place, what it does, and whether it can be pressed.
#[derive(Debug, Clone)]
struct Target {
    rect: Rect,
    label: &'static str,
    detail: &'static str,
    action: Option<HomeAction>,
    enabled: bool,
}

/// The sidebar rows: label and action (none for the page that is open, or not available yet).
const DESTINATIONS: [(&str, Option<HomeAction>); 6] = [
    ("Home", None),
    ("Servers", Some(HomeAction::Servers)),
    ("Friends", Some(HomeAction::Friends)),
    ("Avatar", None),
    ("Marketplace", None),
    ("Settings", Some(HomeAction::Options)),
];

pub struct HomeScreen {
    size: (i32, i32),
    skin: Skin,
    version: String,
    sidebar: Vec<Target>,
    cards: Vec<Target>,
    quit: Target,
    friends_ready: bool,
    focus: Option<usize>,
}

impl HomeScreen {
    /// The home screen of a window `size` units big.
    pub fn new(size: (i32, i32), version: &str) -> Self {
        let rows = grid::column(
            Rect {
                x: 6,
                y: 44,
                width: SIDEBAR_WIDTH - 12,
                height: size.1,
            },
            DESTINATIONS.len(),
            (ROW_HEIGHT, 2),
        );
        let sidebar = rows
            .into_iter()
            .zip(DESTINATIONS)
            .map(|(rect, (label, action))| Target {
                rect,
                label,
                detail: "",
                action,
                enabled: action.is_some(),
            })
            .collect();
        let quit_rect = Rect {
            x: 6,
            y: size.1 - 24,
            width: SIDEBAR_WIDTH - 12,
            height: 18,
        };
        let quit = Target {
            rect: quit_rect,
            label: "Quit",
            detail: "",
            action: Some(HomeAction::Quit),
            enabled: true,
        };
        let mut screen = Self {
            size,
            skin: CLASSIC,
            version: version.to_owned(),
            sidebar,
            cards: Vec::new(),
            quit,
            friends_ready: false,
            focus: None,
        };
        screen.cards = screen.card_targets();
        screen.sync_friends();
        screen
    }

    /// The page's cards. The directory is not connected yet, so the first one adds a server by
    /// address and the second says what is coming.
    fn card_targets(&self) -> Vec<Target> {
        let area = Rect {
            x: SIDEBAR_WIDTH + MARGIN,
            y: 52,
            width: self.size.0 - SIDEBAR_WIDTH - 2 * MARGIN,
            height: self.size.1 - 64,
        };
        let content = [
            (
                "Join a server",
                "Add one by address",
                Some(HomeAction::Servers),
            ),
            ("Photon games", "Directory coming soon", None),
        ];
        grid::cards(area, content.len(), CARD_SIZE, 8)
            .into_iter()
            .zip(content)
            .map(|(rect, (label, detail, action))| Target {
                rect,
                label,
                detail,
                action,
                enabled: action.is_some(),
            })
            .collect()
    }

    fn sync_friends(&mut self) {
        let ready = self.friends_ready;
        for target in &mut self.sidebar {
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
        self.sidebar
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

    /// A mouse press: what the thing under it chose.
    pub fn clicked(&mut self, mouse: (i32, i32)) -> Option<HomeAction> {
        let index = self.target_at(mouse)?;
        self.focus = Some(index);
        self.targets().nth(index).and_then(|target| target.action)
    }

    /// Tab (or arrow keys): focus moves to the next pressable thing, or the previous one.
    pub fn step_focus(&mut self, forward: bool) {
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
    pub fn pressed(&self) -> Option<HomeAction> {
        let focus = self.focus?;
        self.targets()
            .nth(focus)
            .filter(|target| target.enabled)
            .and_then(|target| target.action)
    }

    /// The panorama's dimming, the sidebar, the page and the version line.
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
        let bar = Rect {
            x: 0,
            y: 0,
            width: SIDEBAR_WIDTH,
            height,
        };
        painter.panel(bar);
        let ink = painter.skin.ink;
        painter.label("PHOTON", (10, 12), ink);
        let muted = painter.skin.muted;
        painter.label(&self.version, (10, 24), muted);
        let hovered = self.target_at(mouse);
        for (index, target) in self.targets().enumerate() {
            let focused = self.focus == Some(index) || hovered == Some(index);
            Self::draw_target(&mut painter, target, index < self.sidebar.len(), focused);
        }
        painter.label("Home", (SIDEBAR_WIDTH + MARGIN, 14), ink);
        painter.label("Pick a game to play", (SIDEBAR_WIDTH + MARGIN, 28), muted);
    }

    fn draw_target(painter: &mut Painter, target: &Target, in_sidebar: bool, focused: bool) {
        let state = match (target.enabled, focused) {
            (false, _) => State::Disabled,
            (true, true) => State::Hover,
            (true, false) => State::Normal,
        };
        if !in_sidebar && target.label != "Quit" {
            painter.card(target.rect, (target.label, target.detail), state);
        } else if target.label == "Quit" {
            painter.button(target.rect, target.label, state);
        } else {
            let state = if target.label == "Home" {
                State::Selected
            } else {
                state
            };
            painter.row(target.rect, target.label, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> HomeScreen {
        HomeScreen::new((427, 240), "Photon 0.4.0")
    }

    fn centre(rect: Rect) -> (i32, i32) {
        (rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    #[test]
    fn pressing_the_servers_row_asks_for_the_server_list() {
        let mut home = screen();
        let at = centre(home.sidebar[1].rect);
        assert_eq!(home.clicked(at), Some(HomeAction::Servers));
    }

    #[test]
    fn a_row_that_is_not_available_does_nothing() {
        let mut home = screen();
        let avatar = centre(home.sidebar[3].rect);
        assert_eq!(home.clicked(avatar), None);
        // Friends waits for an account.
        let friends = centre(home.sidebar[2].rect);
        assert_eq!(home.clicked(friends), None);
        home.friends_available();
        assert_eq!(home.clicked(friends), Some(HomeAction::Friends));
    }

    #[test]
    fn quit_and_settings_are_pressable() {
        let mut home = screen();
        let settings = centre(home.sidebar[5].rect);
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
    fn nothing_is_pressed_before_anything_is_focused() {
        assert_eq!(screen().pressed(), None);
    }
}
