//! The world settings screen (`WorldOptionsScreen`): a General header over the difficulty with its
//! lock, the game rules and the restrictions, between a title and the footer's Apply Changes and
//! Cancel. Photon has no integrated server, so the game mode, allow-commands and multiplayer
//! options never exist; the difficulty and its lock are the server's to change, by the gamemaster.

use std::cell::Cell;

use photon_game::text::{Component, Lang, Run, Style};

use crate::FontRuns;
use crate::layout::{Arena, Grid, Item, Settings, horizontal, place_in_frame};
use crate::options::text::generic_value;
use crate::tooltip::{Holder, Pending};
use crate::widgets::navigation::Rectangle;
use crate::widgets::scroll_area::Place;
use crate::widgets::slider::{InputType, SliderKey};
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `HeaderAndFooterLayout.DEFAULT_HEADER_AND_FOOTER_HEIGHT`.
const BAR: i32 = 33;
/// The padding of the content's cells (`defaultCellSetting().padding(8)`).
const CONTENT_PADDING: i32 = 8;
/// `HeaderAndFooterLayout.CONTENT_MARGIN_TOP`.
const CONTENT_MARGIN_TOP: i32 = 30;
/// `FocusableTextWidget.DEFAULT_PADDING`.
const HEADER_PADDING: i32 = 4;

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
    /// Apply Changes: the difficulty wanted (if it changed) and whether it is locked now.
    Apply {
        difficulty: Option<u8>,
        lock: bool,
    },
    /// Cancel.
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    General,
    Difficulty,
    Lock,
    GameRules,
    Restrictions,
    Apply,
    Cancel,
}

pub struct WorldOptionsScreen {
    arena: Arena,
    size: (i32, i32),
    in_world: bool,
    facts: WorldFacts,
    title: Widget,
    /// The widgets in the order the screen adds them (the order Tab goes through).
    widgets: Vec<(Part, Widget)>,
    /// The text inside the General header's widget.
    general: Widget,
    tooltips: Vec<Option<Holder>>,
    focus: Option<usize>,
    input: Cell<InputType>,
    /// The difficulty and the lock as the buttons have them now.
    wanted: (u8, bool),
}

fn runs(lang: &Lang, key: &str) -> Vec<Run> {
    lang.runs(&Component::translatable(key), &Style::default())
}

/// `Difficulty.getKey`.
fn difficulty_key(id: u8) -> &'static str {
    ["peaceful", "easy", "normal", "hard"][usize::from(id.min(3))]
}

/// The difficulty button's label for a difficulty.
fn difficulty_label(lang: &Lang, id: u8) -> Vec<Run> {
    let label = generic_value(
        &Component::translatable("options.difficulty"),
        Component::translatable(format!("options.difficulty.{}", difficulty_key(id))),
    );
    lang.runs(&label, &Style::default())
}

/// The buttons that change the difficulty are active for a gamemaster, unless it is locked.
fn difficulty_active(facts: WorldFacts) -> bool {
    // 26.2 lets only a singleplayer server's owner change it, and Photon has no such server.
    photon_core::versions::behaviour::world_options_is_sectioned()
        && facts.gamemaster
        && !facts.locked
        && !facts.hardcore
}

/// What the difficulty button and its lock are made of.
fn difficulty_pair(lang: &Lang, facts: WorldFacts) -> [(Part, (i32, i32), bool, Kind); 2] {
    let active = difficulty_active(facts);
    let label = difficulty_label(lang, facts.difficulty);
    let locked = facts.locked || facts.hardcore;
    [
        (Part::Difficulty, (130, 20), active, Kind::Button { label }),
        (Part::Lock, (20, 20), active, Kind::Lock { locked }),
    ]
}

/// The widgets, in the order the screen adds them, with each one's size.
fn widgets(
    arena: &mut Arena,
    lang: &Lang,
    facts: WorldFacts,
    general: &[Run],
) -> Vec<(Part, Widget)> {
    let button = |key: &str| Kind::Button {
        label: runs(lang, key),
    };
    let header = Kind::Text {
        text: general.to_vec(),
    };
    let header_width = Font::default().width_of_runs(general) + 2 * HEADER_PADDING;
    let sectioned = photon_core::versions::behaviour::world_options_is_sectioned();
    let general = (
        Part::General,
        if sectioned {
            (header_width, 9 + 2 * HEADER_PADDING)
        } else {
            (0, 0)
        },
        sectioned,
        header,
    );
    let rest = [
        (
            Part::GameRules,
            (150, 20),
            facts.gamemaster && !facts.hardcore,
            button("editGamerule.inGame.button"),
        ),
        (
            Part::Restrictions,
            (150, 20),
            true,
            button("restrictions_screen.button"),
        ),
        (
            Part::Apply,
            if sectioned { (150, 20) } else { (0, 0) },
            false,
            button("menu.multiplayerOptions.applyChanges"),
        ),
        if sectioned {
            (Part::Cancel, (150, 20), true, button("gui.cancel"))
        } else {
            (Part::Cancel, (200, 20), true, button("gui.done"))
        },
    ];
    std::iter::once(general)
        .chain(difficulty_pair(lang, facts))
        .chain(rest)
        .map(|(part, (width, height), active, kind)| {
            let rect = arena.add(width, height);
            (part, Widget { rect, active, kind })
        })
        .collect()
}

/// `repositionElements`: the title in the header, the footer's two buttons, and the content's
/// grid under the header with the cells' padding around it. Gives the General header's own text
/// its place, 4 units inside the widget.
fn arrange(
    arena: &mut Arena,
    (title, general): (usize, usize),
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
    let cell = Settings::default().align_horizontally(0.5);
    if photon_core::versions::behaviour::world_options_is_sectioned() {
        grid.add_in_row(Item::Leaf(rect(Part::General)), 2, 2, cell);
        grid.add_in_row(Item::Grid(Box::new(layout)), 2, 1, cell);
        grid.add_in_row(Item::Leaf(rect(Part::GameRules)), 2, 1, cell);
    } else {
        grid.add_in_row(Item::Leaf(rect(Part::GameRules)), 2, 1, cell);
        grid.add_in_row(Item::Grid(Box::new(layout)), 2, 1, cell);
    }
    grid.add_in_row(Item::Leaf(rect(Part::Restrictions)), 2, 1, cell);
    grid.arrange(arena);
    // The content is a column of one cell with the padding around it, centred in the screen.
    let area_width = grid.width + 2 * CONTENT_PADDING;
    let top = if photon_core::versions::behaviour::world_options_is_sectioned() {
        BAR + CONTENT_PADDING
    } else {
        // 26.2 puts the content a fixed margin under the header, as high as the footer allows.
        (BAR + CONTENT_MARGIN_TOP).min(height - BAR - grid.height)
    };
    let at = (width / 2 - area_width / 2 + CONTENT_PADDING, top);
    let frame = (at.0, at.1, grid.width, grid.height);
    place_in_frame(arena, Item::Grid(Box::new(grid)), frame, (0.0, 0.0));
    place_in_frame(arena, Item::Leaf(title), (0, 0, width, BAR), (0.5, 0.5));
    let apply = rect(Part::Apply);
    let buttons: &[usize] = if photon_core::versions::behaviour::world_options_is_sectioned() {
        &[apply, rect(Part::Cancel)]
    } else {
        &[rect(Part::Cancel)]
    };
    let mut footer = horizontal(arena, buttons, 8);
    footer.arrange(arena);
    let frame = (0, height - BAR, width, BAR);
    place_in_frame(arena, Item::Grid(Box::new(footer)), frame, (0.5, 0.5));
    let outer = arena.0[rect(Part::General)];
    arena.0[general].x = outer.x + HEADER_PADDING;
    arena.0[general].y = outer.y + HEADER_PADDING;
}

/// The key of the tooltip that says why the difficulty buttons are off.
fn difficulty_disabled(facts: WorldFacts) -> Option<&'static str> {
    if !facts.gamemaster {
        Some("options.worldOptions.difficulty.disabled.operator.tooltip")
    } else if facts.locked {
        Some("options.worldOptions.difficulty.disabled.locked.tooltip")
    } else if facts.hardcore {
        Some("options.worldOptions.difficulty.disabled.hardcore.tooltip")
    } else {
        None
    }
}

/// The key of the tooltip that says why the game rules button is off.
fn rules_disabled(facts: WorldFacts) -> Option<&'static str> {
    if facts.hardcore {
        Some("editGamerule.inGame.disabled.hardcore.tooltip")
    } else if !facts.gamemaster {
        Some("editGamerule.inGame.disabled.tooltip")
    } else {
        None
    }
}

/// The key of the tooltip that says why a widget is off, for the facts.
fn disabled_key(part: Part, facts: WorldFacts) -> Option<&'static str> {
    match part {
        Part::Difficulty => difficulty_disabled(facts),
        Part::GameRules => rules_disabled(facts),
        _ => None,
    }
}

/// The tooltip of each widget that has one, for the facts.
fn tooltips(lang: &Lang, widgets: &[(Part, Widget)], facts: WorldFacts) -> Vec<Option<Holder>> {
    widgets
        .iter()
        .map(|(part, _)| {
            disabled_key(*part, facts).map(|key| Holder::of(lang, &Component::translatable(key)))
        })
        .collect()
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
        let style = Style {
            bold: Some(true),
            underlined: Some(true),
            ..Style::default()
        };
        let general_text = lang.runs(
            &Component::translatable("options.worldOptions.general.title"),
            &style,
        );
        let widgets = widgets_of(&mut arena, lang, facts, (font, &general_text));
        let general = Widget {
            rect: arena.add(font.width_of_runs(&general_text), 9),
            active: false,
            kind: Kind::Text { text: general_text },
        };
        arrange(&mut arena, (title.rect, general.rect), &widgets, size);
        let tooltips = tooltips(lang, &widgets, facts);
        Self {
            arena,
            size,
            in_world,
            facts,
            title,
            widgets,
            general,
            tooltips,
            focus: None,
            input: Cell::new(InputType::Mouse),
            wanted: (facts.difficulty, facts.locked || facts.hardcore),
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
        let screen = Self::new(font, lang, size, self.in_world, facts);
        *self = screen;
        self.focus = focus;
    }

    fn place(&self) -> Place {
        Place {
            y: BAR,
            width: self.size.0,
            height: self.size.1 - 2 * BAR,
            in_world: self.in_world,
        }
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        let place = self.place();
        draw_bars(&place, frame, atlas);
        self.title.draw(frame, atlas, font, &self.arena, false);
        let mut pending: Option<Pending> = None;
        for (index, (part, widget)) in self.widgets.iter().enumerate() {
            if !is_drawn(*part) {
                continue;
            }
            draw_footer_bar(*part, &place, frame, atlas);
            let focused = self.focus == Some(index);
            let hovered = widget.contains(&self.arena, mouse);
            if *part == Part::General {
                self.draw_general(frame, atlas, font, focused);
            } else {
                widget.draw(frame, atlas, font, &self.arena, hovered || focused);
            }
            if let Some(asked) = self.ask_tooltip(index, frame.millis, (hovered, focused))
                && (pending.is_none() || focused)
            {
                pending = Some(asked);
            }
        }
        if let Some(pending) = pending {
            pending.draw(frame, atlas, font, mouse, self.size);
        }
    }

    /// The widget's area in the screen.
    fn area(&self, widget: &Widget) -> Rectangle {
        let rect = self.arena.0[widget.rect];
        Rectangle {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }

    /// What the tooltip of the widget at `index` asks to show now, if it has one.
    fn ask_tooltip(&self, index: usize, millis: u64, state: (bool, bool)) -> Option<Pending> {
        let tooltip = self.tooltips[index].as_ref()?;
        let area = self.area(&self.widgets[index].1);
        tooltip.refresh(millis, state, self.input.get(), area)
    }

    /// The General header: its text, in a black box with a white border while it has the focus.
    fn draw_general(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, focused: bool) {
        if focused {
            let area = self.area(&self.widgets[0].1);
            frame.fill(
                atlas,
                (
                    area.x + 1,
                    area.y,
                    area.x + area.width,
                    area.y + area.height,
                ),
                0xFF00_0000,
            );
            crate::screen::dialog_body::outline(frame, atlas, area);
        }
        self.general.draw(frame, atlas, font, &self.arena, false);
    }

    /// Whether a widget can take the focus or a press: the General header always can.
    fn focusable(part: Part, widget: &Widget) -> bool {
        (part == Part::General && photon_core::versions::behaviour::world_options_is_sectioned())
            || (part != Part::General && widget.is_active())
    }

    /// Whether the difficulty or the lock differ from what the server said.
    fn has_changes(&self) -> bool {
        self.wanted
            != (
                self.facts.difficulty,
                self.facts.locked || self.facts.hardcore,
            )
    }

    /// Apply Changes is active while something differs.
    fn refresh_apply(&mut self) {
        let changed = self.has_changes();
        if let Some((_, widget)) = self
            .widgets
            .iter_mut()
            .find(|(part, _)| *part == Part::Apply)
        {
            widget.active = changed;
        }
    }

    /// A press of the widget at `index`, going `step` through the difficulties.
    fn press(&mut self, lang: &Lang, index: usize, step: i32) -> Option<WorldPress> {
        crate::ui_sound::play_down_sound();
        match self.widgets[index].0 {
            Part::GameRules => Some(WorldPress::GameRules),
            Part::Restrictions => Some(WorldPress::Restrictions),
            Part::Cancel => Some(WorldPress::Done),
            Part::Apply => Some(WorldPress::Apply {
                difficulty: (self.wanted.0 != self.facts.difficulty).then_some(self.wanted.0),
                lock: self.wanted.1,
            }),
            Part::Difficulty => {
                self.wanted.0 = (i32::from(self.wanted.0) + step).rem_euclid(4) as u8;
                self.widgets[index].1.kind = Kind::Button {
                    label: difficulty_label(lang, self.wanted.0),
                };
                self.refresh_apply();
                None
            }
            Part::Lock => {
                self.wanted.1 = !self.wanted.1;
                self.widgets[index].1.kind = Kind::Lock {
                    locked: self.wanted.1,
                };
                self.refresh_apply();
                None
            }
            Part::General => None,
        }
    }

    /// A press of the primary button at `mouse`.
    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> Option<WorldPress> {
        self.input.set(InputType::Mouse);
        let mouse = (mouse.0 as i32, mouse.1 as i32);
        let index = self.widgets.iter().position(|(part, widget)| {
            Self::focusable(*part, widget) && widget.contains(&self.arena, mouse)
        })?;
        self.focus = Some(index);
        self.press(lang, index, if shift { -1 } else { 1 })
    }

    /// Tab (and Shift+Tab): the focus goes to the next active widget.
    pub fn tab(&mut self, forward: bool) {
        self.input.set(InputType::KeyboardTab);
        let count = self.widgets.len();
        let step = if forward { 1 } else { count - 1 };
        let mut at = self.focus.unwrap_or(if forward { count - 1 } else { 0 });
        for _ in 0..count {
            at = (at + step) % count;
            let (part, widget) = &self.widgets[at];
            if Self::focusable(*part, widget) {
                self.focus = Some(at);
                return;
            }
        }
    }

    /// Enter or Space: the focused button.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> Option<WorldPress> {
        if key != SliderKey::Select {
            return None;
        }
        let index = self.focus.filter(|&index| {
            let (part, widget) = &self.widgets[index];
            Self::focusable(*part, widget)
        })?;
        self.press(lang, index, if shift { -1 } else { 1 })
    }

    /// The screen has no mouse drag.
    pub fn dragged(&mut self, _lang: &Lang, _mouse: (f64, f64), _dy: f64) {}

    pub fn released(&mut self, _lang: &Lang) {}

    /// The content has no more than it shows.
    pub fn scrolled(&mut self, _mouse: (f64, f64), _scroll_y: f64) {}

    pub fn scroll_amount(&self) -> f64 {
        0.0
    }
}

/// The separators and the list background around the content (26.3; 26.2 has none).
fn draw_bars(place: &Place, frame: &mut GuiFrame, atlas: &GuiAtlas) {
    if photon_core::versions::behaviour::world_options_is_sectioned() {
        place.draw_header_separator(frame, atlas);
        place.draw_background(frame, atlas, 0.0);
    }
}

/// The footer's separator, drawn before its first button (Apply Changes).
fn draw_footer_bar(part: Part, place: &Place, frame: &mut GuiFrame, atlas: &GuiAtlas) {
    if part == Part::Apply {
        place.draw_footer_separator(frame, atlas);
    }
}

/// Whether a widget is drawn: 26.2 has neither the General header nor Apply Changes.
fn is_drawn(part: Part) -> bool {
    photon_core::versions::behaviour::world_options_is_sectioned()
        || !matches!(part, Part::General | Part::Apply)
}

/// The widgets for a screen, the header's width from the font.
fn widgets_of(
    arena: &mut Arena,
    lang: &Lang,
    facts: WorldFacts,
    (font, general): (&Font, &[Run]),
) -> Vec<(Part, Widget)> {
    let mut all = widgets(arena, lang, facts, general);
    // The header is as wide as its text and the padding around it.
    let (_, header) = &mut all[0];
    arena.0[header.rect].width = font.width_of_runs(general) + 2 * HEADER_PADDING;
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gamemaster() -> WorldFacts {
        WorldFacts {
            gamemaster: true,
            ..WorldFacts::default()
        }
    }

    fn screen(facts: WorldFacts) -> WorldOptionsScreen {
        let (font, lang) = (Font::default(), Lang::default());
        WorldOptionsScreen::new(&font, &lang, (427, 240), true, facts)
    }

    /// The centre of a button.
    fn centre(screen: &WorldOptionsScreen, wanted: Part) -> (f64, f64) {
        let (_, widget) = screen
            .widgets
            .iter()
            .find(|(part, _)| *part == wanted)
            .expect("the screen has the button");
        let rect = screen.arena.0[widget.rect];
        (
            f64::from(rect.x + rect.width / 2),
            f64::from(rect.y + rect.height / 2),
        )
    }

    #[test]
    fn a_gamemasters_button_is_active_in_the_reference_screen() {
        let mut screen = screen(gamemaster());
        let at = centre(&screen, Part::GameRules);
        assert_eq!(
            screen.clicked(&Lang::default(), at, false),
            Some(WorldPress::GameRules)
        );
    }

    #[test]
    fn a_new_difficulty_makes_apply_changes_active() {
        let mut screen = screen(gamemaster());
        let lang = Lang::default();
        let at = centre(&screen, Part::Difficulty);
        assert_eq!(screen.clicked(&lang, at, false), None);
        let at = centre(&screen, Part::Apply);
        assert_eq!(
            screen.clicked(&lang, at, false),
            Some(WorldPress::Apply {
                difficulty: Some(3),
                lock: false,
            })
        );
    }
}
