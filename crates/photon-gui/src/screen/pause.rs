//! The pause menu (`PauseScreen`): the buttons of the game menu laid out in
//! a grid, over the menu background.

use crate::FontRuns;
use std::cell::Cell;

use photon_game::text::{Component, Lang, Run, Style};

use crate::kit::{Painter, State};
use crate::layout::{Arena, Grid, Item, Rect, Settings, horizontal};
use crate::tokens::CLASSIC;
use crate::tooltip::{Holder, Pending};
use crate::widgets::navigation::{Candidate, Direction, Rectangle, next_in_direction};
use crate::widgets::slider::InputType;
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// What pressing a button asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseAction {
    /// "Back to Game".
    Resume,
    /// "Disconnect".
    Disconnect,
    Advancements,
    Statistics,
    Options,
    /// "World Options...".
    WorldOptions,
    ReportBugs,
    Feedback,
    Friends,
    PlayerReporting,
    /// The button a dialog of the server's puts in the menu.
    Custom,
}

pub struct PauseScreen {
    arena: Arena,
    widgets: Vec<(Widget, Option<PauseAction>)>,
    title: Widget,
    /// The widget the keyboard has focused, an index of `widgets`.
    focus: Option<usize>,
    /// The tooltip of each widget that has one.
    tooltips: Vec<Option<Holder>>,
    /// What the window last took in, which decides whether a focus shows its
    /// tooltip.
    input: Cell<InputType>,
    size: (i32, i32),
    /// The title of the dialog button, if the menu has one.
    custom: Option<Component>,
}

/// What the stilled buttons say (ours: the reference has no such state).
const NO_SERVICES: &str = "Photon does not have the online services this needs yet.";

fn runs(lang: &Lang, key: &str) -> Vec<Run> {
    lang.runs(
        &photon_game::text::Component::translatable(key),
        &Style::default(),
    )
}

/// The icon buttons say what they are when the mouse is on them, and the dialog button what it is.
fn tooltips(lang: &Lang, widgets: &[(Widget, Option<PauseAction>)]) -> Vec<Option<Holder>> {
    widgets
        .iter()
        .map(|(_, action)| {
            let key = match action {
                Some(PauseAction::ReportBugs) => "menu.reportBugs",
                Some(PauseAction::Feedback) => "menu.sendFeedback",
                Some(PauseAction::Friends) => "gui.friends.open.tooltip",
                Some(PauseAction::PlayerReporting) => "menu.playerReporting",
                Some(PauseAction::Custom) => "menu.custom_options.tooltip",
                _ => return None,
            };
            Some(Holder::of(lang, &Component::translatable(key)))
        })
        .collect()
}

impl PauseScreen {
    /// `PauseScreen.init` and `createPauseMenu`, for a server (not a local
    /// one) in a screen of `width` by `height` units.
    pub fn new(
        font: &Font,
        lang: &Lang,
        (width, height): (i32, i32),
        custom: Option<&Component>,
    ) -> Self {
        let mut arena = Arena::default();
        let mut widgets: Vec<(Widget, Option<PauseAction>)> = Vec::new();
        let mut add = |arena: &mut Arena, w: i32, kind: Kind, action: PauseAction| -> usize {
            let rect = arena.add(w, 20);
            widgets.push((
                Widget {
                    rect,
                    active: true,
                    kind,
                },
                Some(action),
            ));
            rect
        };
        let button = |key: &str| Kind::Button {
            label: runs(lang, key),
        };
        let icon = |sprite: &str| Kind::Icon {
            sprite: sprite.to_owned(),
            size: (15, 15),
        };
        let mut grid = Grid::new();
        grid.defaults = Settings::default().padding(4, 4, 4, 0);
        let back = add(
            &mut arena,
            204,
            button("menu.returnToGame"),
            PauseAction::Resume,
        );
        grid.add_in_row(Item::Leaf(back), 2, 2, grid.defaults.padding_top(50));
        let advancements = add(
            &mut arena,
            98,
            button("gui.advancements"),
            PauseAction::Advancements,
        );
        grid.add_in_row(Item::Leaf(advancements), 2, 1, grid.defaults);
        let statistics = add(&mut arena, 98, button("gui.stats"), PauseAction::Statistics);
        grid.add_in_row(Item::Leaf(statistics), 2, 1, grid.defaults);
        let icons = [
            add(
                &mut arena,
                20,
                icon("pause_menu/bug"),
                PauseAction::ReportBugs,
            ),
            add(
                &mut arena,
                20,
                icon("pause_menu/social_interactions"),
                PauseAction::Feedback,
            ),
            add(
                &mut arena,
                20,
                icon("friends/friends"),
                PauseAction::Friends,
            ),
            add(
                &mut arena,
                20,
                icon("pause_menu/player_reporting"),
                PauseAction::PlayerReporting,
            ),
        ];
        let row = horizontal(&arena, &icons, 4);
        grid.add_in_row(
            Item::Grid(Box::new(row)),
            2,
            2,
            grid.defaults.align_horizontally(0.5),
        );
        if let Some(title) = custom {
            let label = lang.runs(title, &Style::default());
            let custom = add(&mut arena, 204, Kind::Button { label }, PauseAction::Custom);
            grid.add_in_row(Item::Leaf(custom), 2, 2, grid.defaults);
        }
        for (key, action) in [
            ("menu.options", PauseAction::Options),
            ("options.worldOptions.button", PauseAction::WorldOptions),
        ] {
            let rect = add(&mut arena, 98, button(key), action);
            grid.add_in_row(Item::Leaf(rect), 2, 1, grid.defaults);
        }
        let disconnect = add(
            &mut arena,
            204,
            button("menu.disconnect"),
            PauseAction::Disconnect,
        );
        grid.add_in_row(Item::Leaf(disconnect), 2, 2, grid.defaults);
        grid.arrange(&mut arena);
        grid.align_in_rectangle(&mut arena, (0, 0, width, height), (0.5, 0.25));
        // The title: a string widget as wide as its text, at the top.
        let text = runs(lang, "menu.game");
        let text_width = font.width_of_runs(&text);
        let rect = arena.add(text_width, 9);
        arena.0[rect].x = width / 2 - text_width / 2;
        arena.0[rect].y = 40;
        let title = Widget {
            rect,
            active: false,
            kind: Kind::Text { text },
        };
        let tooltips = tooltips(lang, &widgets);
        Self {
            arena,
            widgets,
            title,
            focus: None,
            tooltips,
            input: Cell::new(InputType::Mouse),
            size: (width, height),
            custom: custom.cloned(),
        }
    }

    /// The menu of a client without the reference's friends service: the friends button is shown
    /// but still, and says why. The reference's
    /// menu has them active, so the parity cases build the menu without this.
    pub fn without_services(mut self, lang: &Lang) -> Self {
        for (index, (widget, action)) in self.widgets.iter_mut().enumerate() {
            let (key, fallback) = match action {
                Some(PauseAction::Friends) => ("photon.pause.no_services", NO_SERVICES),
                _ => continue,
            };
            widget.active = false;
            let reason = Component::translatable_or(key, fallback);
            self.tooltips[index] = Some(Holder::of(lang, &reason));
        }
        self
    }

    /// The screen again for another size, with the same button.
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        let screen = Self::new(font, lang, size, self.custom.as_ref());
        let stilled = self
            .widgets
            .iter()
            .any(|(widget, action)| *action == Some(PauseAction::Friends) && !widget.active);
        if stilled {
            screen.without_services(lang)
        } else {
            screen
        }
    }

    /// The screen size the layout was made for.
    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The alpha of the background colour: see-through over a game.
    fn veil(world: Option<i32>) -> u32 {
        if world.is_some() {
            0x8000_0000
        } else {
            0xFF00_0000
        }
    }

    /// One widget in the Photon skin: buttons through the kit, the rest as they are.
    fn draw_widget(&self, painter: &mut Painter, widget: &Widget, highlighted: bool) {
        let rect = self.arena.0[widget.rect];
        let state = match (widget.active, highlighted) {
            (false, _) => State::Disabled,
            (true, true) => State::Hover,
            (true, false) => State::Normal,
        };
        match &widget.kind {
            Kind::Button { label } => {
                let text: String = label.iter().map(|run| run.text.as_str()).collect();
                painter.button(rect, &text, state);
            }
            Kind::Icon { sprite, size } => {
                painter.button(rect, "", state);
                let (x, y) = (
                    rect.x + (rect.width - size.0) / 2,
                    rect.y + (rect.height - size.1) / 2,
                );
                painter
                    .frame
                    .sprite(painter.atlas, sprite, x, y, size.0, size.1);
            }
            _ => widget.draw(
                painter.frame,
                painter.atlas,
                painter.font,
                &self.arena,
                highlighted,
            ),
        }
    }

    /// The menu: the background texture, the buttons (the highlighted one under the mouse or with
    /// the focus), the title. Over a game (`world` holds the menu blurriness) what is behind is
    /// blurred first (`PauseScreen.extractBackground`).
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
        (width, height): (i32, i32),
        world: Option<i32>,
    ) {
        if let Some(blurriness) = world {
            frame.blur_before_this_stratum(blurriness);
        }
        let skin = CLASSIC;
        let mut painter = Painter {
            frame,
            atlas,
            font,
            skin: &skin,
        };
        let dim = (skin.bg & 0x00FF_FFFF) | Self::veil(world);
        painter.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            dim,
        );
        for (index, (widget, _)) in self.widgets.iter().enumerate() {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == Some(index);
            self.draw_widget(&mut painter, widget, highlighted);
        }
        self.title
            .draw(painter.frame, atlas, font, &self.arena, false);
        self.draw_tooltip(painter.frame, atlas, font, mouse, (width, height));
    }

    /// `extractDeferredElements`: the tooltip of the widget over the rest.
    fn draw_tooltip(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
        (width, height): (i32, i32),
    ) {
        // (the first asked for, but the focused one's replaces it).
        let mut pending: Option<Pending> = None;
        for (index, (widget, _)) in self.widgets.iter().enumerate() {
            let Some(tooltip) = &self.tooltips[index] else {
                continue;
            };
            let rect = self.arena.0[widget.rect];
            let rectangle = Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            };
            let focused = self.focus == Some(index);
            let hovered = widget.contains(&self.arena, mouse);
            if let Some(asked) = tooltip.refresh(
                frame.millis,
                (hovered, focused),
                self.input.get(),
                rectangle,
            ) && (pending.is_none() || focused)
            {
                pending = Some(asked);
            }
        }
        if let Some(pending) = pending {
            pending.draw(frame, atlas, font, mouse, (width, height));
        }
    }

    /// The mouse moved or was pressed: the window's last input is the mouse.
    pub fn mouse_input(&self) {
        self.input.set(InputType::Mouse);
    }

    /// An arrow key: the focus goes to the button next to the focused one.
    pub fn arrow(&mut self, direction: Direction) {
        self.input.set(InputType::KeyboardArrow);
        let rectangle = |rect: usize| {
            let rect = self.arena.0[rect];
            Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        };
        // The children in the order they are added: the title is last.
        let mut children: Vec<Candidate> = self
            .widgets
            .iter()
            .map(|(widget, _)| Candidate {
                rectangle: rectangle(widget.rect),
                focusable: widget.is_active(),
            })
            .collect();
        children.push(Candidate {
            rectangle: rectangle(self.title.rect),
            focusable: false,
        });
        let container = Rectangle {
            x: 0,
            y: 0,
            width: self.size.0,
            height: self.size.1,
        };
        if let Some(next) = next_in_direction(&children, self.focus, direction, container) {
            self.focus = Some(next);
        }
    }

    /// A press of the primary button at `mouse`: what it asks for.
    pub fn clicked(&mut self, mouse: (i32, i32)) -> Option<PauseAction> {
        self.mouse_input();
        let index = self
            .widgets
            .iter()
            .position(|(widget, _)| widget.is_active() && widget.contains(&self.arena, mouse))?;
        self.press(index)
    }

    /// The press of a button: it sounds, and "Disconnect" turns itself off while it leaves.
    fn press(&mut self, index: usize) -> Option<PauseAction> {
        crate::ui_sound::play_down_sound();
        let action = self.widgets[index].1;
        if action == Some(PauseAction::Disconnect) {
            self.widgets[index].0.active = false;
        }
        action
    }

    /// Tab (and Shift+Tab): the focus moves to the next button.
    pub fn tab(&mut self, forward: bool) {
        self.input.set(InputType::KeyboardTab);
        let count = self.widgets.len();
        let next = match (self.focus, forward) {
            (None, true) => 0,
            (None, false) => count - 1,
            (Some(index), true) => (index + 1) % count,
            (Some(index), false) => (index + count - 1) % count,
        };
        self.focus = Some(next);
    }

    /// Enter or Space: what the focused button asks for.
    pub fn pressed(&mut self) -> Option<PauseAction> {
        let index = self.focus.filter(|&index| {
            self.widgets
                .get(index)
                .is_some_and(|(widget, _)| widget.is_active())
        })?;
        self.press(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The centre of the widget of `action`.
    fn centre(screen: &PauseScreen, action: PauseAction) -> (i32, i32) {
        let (widget, _) = screen
            .widgets
            .iter()
            .find(|(_, found)| *found == Some(action))
            .expect("the menu has the button");
        let rect = screen.arena.0[widget.rect];
        (rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    fn screen(without_services: bool) -> PauseScreen {
        let (font, lang) = (Font::default(), Lang::default());
        let screen = PauseScreen::new(&font, &lang, (427, 240), None);
        if without_services {
            screen.without_services(&lang)
        } else {
            screen
        }
    }

    #[test]
    fn the_reference_menu_has_every_button_active() {
        let mut screen = screen(false);
        for action in [
            PauseAction::ReportBugs,
            PauseAction::Feedback,
            PauseAction::Friends,
            PauseAction::PlayerReporting,
        ] {
            let at = centre(&screen, action);
            assert_eq!(screen.clicked(at), Some(action));
        }
    }

    #[test]
    fn without_services_stills_friends_only() {
        let mut screen = screen(true);
        let at = centre(&screen, PauseAction::Friends);
        assert_eq!(screen.clicked(at), None);
        for action in [
            PauseAction::ReportBugs,
            PauseAction::Feedback,
            PauseAction::PlayerReporting,
            PauseAction::Options,
        ] {
            let at = centre(&screen, action);
            assert_eq!(screen.clicked(at), Some(action));
        }
    }

    #[test]
    fn a_resized_menu_keeps_the_buttons_still() {
        let (font, lang) = (Font::default(), Lang::default());
        let mut screen = screen(true).resized(&font, &lang, (320, 180));
        let at = centre(&screen, PauseAction::Friends);
        assert_eq!(screen.clicked(at), None);
    }

    #[test]
    fn each_main_button_asks_for_its_action() {
        let mut screen = screen(false);
        for action in [
            PauseAction::Resume,
            PauseAction::Advancements,
            PauseAction::Statistics,
            PauseAction::Options,
        ] {
            let at = centre(&screen, action);
            assert_eq!(screen.clicked(at), Some(action));
        }
    }

    #[test]
    fn a_click_on_nothing_asks_for_nothing() {
        assert_eq!(screen(false).clicked((1, 1)), None);
    }

    #[test]
    fn disconnect_asks_once_whether_clicked_or_pressed() {
        let mut screen = screen(false);
        let at = centre(&screen, PauseAction::Disconnect);
        assert_eq!(screen.clicked(at), Some(PauseAction::Disconnect));
        assert_eq!(screen.clicked(at), None);
        let mut keyed = self::screen(false);
        for _ in 0..keyed.widgets.len() {
            keyed.tab(false);
            if keyed.focus
                == keyed
                    .widgets
                    .iter()
                    .position(|(_, a)| *a == Some(PauseAction::Disconnect))
            {
                break;
            }
        }
        assert_eq!(keyed.pressed(), Some(PauseAction::Disconnect));
        assert_eq!(keyed.pressed(), None);
    }

    #[test]
    fn tab_walks_every_button_and_wraps() {
        let mut screen = screen(false);
        let count = screen.widgets.len();
        assert_eq!(screen.pressed(), None);
        let mut seen = Vec::new();
        for _ in 0..count {
            screen.tab(true);
            seen.push(screen.focus.expect("focused"));
        }
        assert_eq!(seen, (0..count).collect::<Vec<_>>());
        screen.tab(true);
        assert_eq!(screen.focus, Some(0));
        screen.tab(false);
        assert_eq!(screen.focus, Some(count - 1));
    }
}
