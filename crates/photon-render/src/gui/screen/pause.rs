//! The pause menu (`PauseScreen`): the buttons of the game menu laid out in
//! a grid, over the menu background.

use std::cell::Cell;

use photon_game::text::{Component, Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::layout::{Arena, Grid, Item, Settings, horizontal};
use crate::gui::tooltip::{Holder, Pending};
use crate::gui::widgets::navigation::{Candidate, Direction, Rectangle, next_in_direction};
use crate::gui::widgets::slider::InputType;
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

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
}

fn runs(lang: &Lang, key: &str) -> Vec<Run> {
    lang.runs(
        &photon_game::text::Component::translatable(key),
        &Style::default(),
    )
}

impl PauseScreen {
    /// `PauseScreen.init` and `createPauseMenu`, for a server (not a local
    /// one) in a screen of `width` by `height` units.
    #[expect(
        clippy::too_many_lines,
        reason = "`createPauseMenu` builds each button in turn"
    )]
    pub fn new(font: &Font, lang: &Lang, (width, height): (i32, i32)) -> Self {
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
        // The icon buttons say what they are when the mouse is on them.
        let tooltips = widgets
            .iter()
            .map(|(_, action)| {
                let key = match action {
                    Some(PauseAction::ReportBugs) => "menu.reportBugs",
                    Some(PauseAction::Feedback) => "menu.sendFeedback",
                    Some(PauseAction::Friends) => "gui.friends.open.tooltip",
                    Some(PauseAction::PlayerReporting) => "menu.playerReporting",
                    _ => return None,
                };
                Some(Holder::of(lang, &Component::translatable(key)))
            })
            .collect();
        Self {
            arena,
            widgets,
            title,
            focus: None,
            tooltips,
            input: Cell::new(InputType::Mouse),
            size: (width, height),
        }
    }

    /// The screen size the layout was made for.
    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The menu over the game: the background texture, the buttons (the
    /// highlighted one under the mouse or with the focus), the title. The
    /// reference also blurs what is behind; this does not.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
        (width, height): (i32, i32),
        in_world: bool,
    ) {
        let background = if in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
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
    pub fn clicked(&self, mouse: (i32, i32)) -> Option<PauseAction> {
        self.mouse_input();
        self.widgets
            .iter()
            .find(|(widget, _)| widget.is_active() && widget.contains(&self.arena, mouse))
            .and_then(|(_, action)| *action)
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
    pub fn pressed(&self) -> Option<PauseAction> {
        self.focus
            .and_then(|index| self.widgets.get(index))
            .filter(|(widget, _)| widget.is_active())
            .and_then(|(_, action)| *action)
    }
}
