//! The world settings screen (`WorldOptionsScreen`): the game rules, the difficulty with its lock,
//! and the restrictions, between a title and Done. Photon has no integrated server, so the game
//! mode and allow-commands buttons never exist and the difficulty can never be changed here.

use std::cell::Cell;

use photon_game::text::{Component, Lang, Run, Style};

use crate::FontRuns;
use crate::layout::{Arena, Grid, Item, Settings, horizontal, place_in_frame};
use crate::options::text::generic_value;
use crate::tooltip::{Holder, Pending};
use crate::widgets::navigation::Rectangle;
use crate::widgets::slider::{InputType, SliderKey};
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `HeaderAndFooterLayout.DEFAULT_HEADER_AND_FOOTER_HEIGHT`.
const BAR: i32 = 33;
/// `CONTENT_MARGIN_TOP`.
const CONTENT_MARGIN_TOP: i32 = 30;

/// What the server told the client about the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldFacts {
    /// The difficulty's id: 0 peaceful, 1 easy, 2 normal, 3 hard.
    pub difficulty: u8,
    pub locked: bool,
    pub hardcore: bool,
    /// Whether the player holds the gamemaster permission.
    pub gamemaster: bool,
}

impl Default for WorldFacts {
    /// A new level's data: normal, unlocked, and no permission.
    fn default() -> Self {
        Self {
            difficulty: 2,
            locked: false,
            hardcore: false,
            gamemaster: false,
        }
    }
}

/// What a press asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldPress {
    GameRules,
    Restrictions,
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    GameRules,
    Difficulty,
    Lock,
    Restrictions,
    Done,
}

pub struct WorldOptionsScreen {
    arena: Arena,
    size: (i32, i32),
    in_world: bool,
    facts: WorldFacts,
    title: Widget,
    /// The widgets in the order the screen adds them (the order Tab goes through).
    widgets: Vec<(Part, Widget)>,
    tooltips: Vec<Option<Holder>>,
    focus: Option<usize>,
    input: Cell<InputType>,
}

fn runs(lang: &Lang, key: &str) -> Vec<Run> {
    lang.runs(&Component::translatable(key), &Style::default())
}

/// `Difficulty.getKey`.
fn difficulty_key(id: u8) -> &'static str {
    ["peaceful", "easy", "normal", "hard"][usize::from(id.min(3))]
}

/// The buttons, in the order the screen adds them. There is no integrated server: nothing is
/// hardcore here, the difficulty is the server's to change and the game rules wait for the
/// gamemaster permission.
fn widgets(arena: &mut Arena, lang: &Lang, facts: WorldFacts) -> Vec<(Part, Widget)> {
    let label = generic_value(
        &Component::translatable("options.difficulty"),
        Component::translatable(format!(
            "options.difficulty.{}",
            difficulty_key(facts.difficulty)
        )),
    );
    let button = |key: &str| Kind::Button {
        label: runs(lang, key),
    };
    let lock = Kind::Lock {
        locked: facts.locked || facts.hardcore,
    };
    // `difficultyButton.setWidth(width - lockButton.getWidth())`.
    let difficulty = Kind::Button {
        label: lang.runs(&label, &Style::default()),
    };
    [
        (
            Part::GameRules,
            (150, 20),
            facts.gamemaster,
            button("editGamerule.inGame.button"),
        ),
        (Part::Difficulty, (130, 20), false, difficulty),
        (Part::Lock, (20, 20), false, lock),
        (
            Part::Restrictions,
            (150, 20),
            true,
            button("restrictions_screen.button"),
        ),
        (Part::Done, (200, 20), true, button("gui.done")),
    ]
    .into_iter()
    .map(|(part, (width, height), active, kind)| {
        let rect = arena.add(width, height);
        (part, Widget { rect, active, kind })
    })
    .collect()
}

/// `repositionElements`: the title in the header, Done in the footer, and the grid of the
/// buttons between, 30 under the header or as high as the footer lets it.
fn arrange(
    arena: &mut Arena,
    title: usize,
    widgets: &[(Part, Widget)],
    (width, height): (i32, i32),
) {
    let rect = |part: Part| {
        widgets
            .iter()
            .find(|(other, _)| *other == part)
            .map_or(0, |(_, w)| w.rect)
    };
    let mut layout = horizontal(arena, &[rect(Part::Difficulty), rect(Part::Lock)], 0);
    layout.width = 150;
    let mut grid = Grid::new();
    grid.column_spacing = 8;
    grid.row_spacing = 4;
    let cell = Settings::default();
    grid.add_in_row(Item::Leaf(rect(Part::GameRules)), 2, 1, cell);
    grid.add_in_row(Item::Grid(Box::new(layout)), 2, 1, cell);
    grid.add_in_row(Item::Leaf(rect(Part::Restrictions)), 2, 1, cell);
    grid.arrange(arena);
    let (grid_width, grid_height) = (grid.width, grid.height);
    place_in_frame(arena, Item::Leaf(title), (0, 0, width, BAR), (0.5, 0.5));
    let done = Item::Leaf(rect(Part::Done));
    place_in_frame(arena, done, (0, height - BAR, width, BAR), (0.5, 0.5));
    let top = (BAR + CONTENT_MARGIN_TOP).min(height - BAR - grid_height);
    let frame = (0, top, width.max(grid_width), grid_height);
    place_in_frame(arena, Item::Grid(Box::new(grid)), frame, (0.5, 0.5));
}

impl WorldOptionsScreen {
    /// `init`, in a screen of `size` units.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        facts: WorldFacts,
    ) -> Self {
        let mut arena = Arena::default();
        let text = runs(lang, "options.worldOptions.title");
        let title = Widget {
            rect: arena.add(font.width_of_runs(&text), 9),
            active: false,
            kind: Kind::Text { text },
        };
        let widgets = widgets(&mut arena, lang, facts);
        arrange(&mut arena, title.rect, &widgets, size);
        let tooltips = widgets
            .iter()
            .map(|(part, widget)| {
                (*part == Part::GameRules && !widget.active).then(|| {
                    Holder::of(
                        lang,
                        &Component::translatable("editGamerule.inGame.disabled.tooltip"),
                    )
                })
            })
            .collect();
        Self {
            arena,
            size,
            in_world,
            facts,
            title,
            widgets,
            tooltips,
            focus: None,
            input: Cell::new(InputType::Mouse),
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// What the screen was built from.
    pub fn facts(&self) -> WorldFacts {
        self.facts
    }

    /// The screen laid out again for another size or other facts (`repositionElements`,
    /// `onDifficultyChanged`, `onGamemasterPermissionChanged`); the focus stays.
    pub fn rebuild(&mut self, font: &Font, lang: &Lang, size: (i32, i32), facts: WorldFacts) {
        let focus = self.focus;
        *self = Self::new(font, lang, size, self.in_world, facts);
        self.focus = focus;
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        let mut pending: Option<Pending> = None;
        for (index, (_, widget)) in self.widgets.iter().enumerate() {
            let focused = self.focus == Some(index);
            let hovered = widget.contains(&self.arena, mouse);
            widget.draw(frame, atlas, font, &self.arena, hovered || focused);
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
            pending.draw(frame, atlas, font, mouse, self.size);
        }
    }

    fn press(&mut self, index: usize) -> Option<WorldPress> {
        crate::ui_sound::play_down_sound();
        match self.widgets[index].0 {
            Part::GameRules => Some(WorldPress::GameRules),
            Part::Restrictions => Some(WorldPress::Restrictions),
            Part::Done => Some(WorldPress::Done),
            Part::Difficulty | Part::Lock => None,
        }
    }

    /// A press of the primary button at `mouse`.
    pub fn clicked(&mut self, _lang: &Lang, mouse: (f64, f64), _shift: bool) -> Option<WorldPress> {
        self.input.set(InputType::Mouse);
        let mouse = (mouse.0 as i32, mouse.1 as i32);
        let index = self
            .widgets
            .iter()
            .position(|(_, widget)| widget.is_active() && widget.contains(&self.arena, mouse))?;
        self.focus = Some(index);
        self.press(index)
    }

    /// Tab (and Shift+Tab): the focus goes to the next active widget.
    pub fn tab(&mut self, forward: bool) {
        self.input.set(InputType::KeyboardTab);
        let count = self.widgets.len();
        let step = if forward { 1 } else { count - 1 };
        let mut at = self.focus.unwrap_or(if forward { count - 1 } else { 0 });
        for _ in 0..count {
            at = (at + step) % count;
            if self.widgets[at].1.is_active() {
                self.focus = Some(at);
                return;
            }
        }
    }

    /// Enter or Space: the focused button.
    pub fn key_pressed(
        &mut self,
        _lang: &Lang,
        key: SliderKey,
        _shift: bool,
    ) -> Option<WorldPress> {
        if key != SliderKey::Select {
            return None;
        }
        let index = self
            .focus
            .filter(|&index| self.widgets[index].1.is_active())?;
        self.press(index)
    }

    /// The screen has no mouse drag.
    pub fn dragged(&mut self, _lang: &Lang, _mouse: (f64, f64), _dy: f64) {}

    pub fn released(&mut self, _lang: &Lang) {}

    /// The screen has no list to scroll.
    pub fn scrolled(&mut self, _mouse: (f64, f64), _scroll_y: f64) {}

    pub fn scroll_amount(&self) -> f64 {
        0.0
    }
}
