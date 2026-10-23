//! The pause menu (`PauseScreen`): the buttons of the game menu laid out in
//! a grid, over the menu background.

use crate::FontRuns;
use std::cell::Cell;

use photon_game::text::{Component, Lang, Run, Style};

use crate::layout::{Arena, Grid, Item, Settings, horizontal};
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
        let options = add(
            &mut arena,
            204,
            button("menu.options"),
            PauseAction::Options,
        );
        grid.add_in_row(Item::Leaf(options), 2, 2, grid.defaults);
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

    /// The screen again for another size, with the same button.
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        Self::new(font, lang, size, self.custom.as_ref())
    }

    /// The screen size the layout was made for.
    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The texture the background is tiled with; over a game, the frame is blurred first.
    fn background(frame: &mut GuiFrame, world: Option<i32>) -> &'static str {
        let Some(blurriness) = world else {
            return "menu_background";
        };
        frame.blur_before_this_stratum(blurriness);
        "inworld_menu_background"
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
        let background = Self::background(frame, world);
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        for (index, (widget, _)) in self.widgets.iter().enumerate() {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == Some(index);
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
        self.title.draw(frame, atlas, font, &self.arena, false);
        // `extractDeferredElements`: the tooltip of the widget over the rest
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
