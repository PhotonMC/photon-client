//! The options screen (`OptionsScreen`): the field of view and one more
//! button under the title, the buttons that open the other settings screens
//! in two columns, and Done.
pub mod chat;
pub mod keybinds;
pub mod language;
pub mod list;
pub mod mouse;
pub mod sound;
pub mod text;
pub mod video;

use photon_game::options::values::IntRange;
use photon_game::text::{Component, Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::layout::{Arena, Grid, Item, Settings, place_in_frame};
use crate::gui::widgets::navigation::{Candidate, Direction, Rectangle, next_in_direction};
use crate::gui::widgets::slider::{InputType, OptionSlider, SliderKey};
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

/// `HeaderAndFooterLayout(this, 61, 33)`.
const HEADER_HEIGHT: i32 = 61;
const FOOTER_HEIGHT: i32 = 33;
/// `CONTENT_MARGIN_TOP`.
const CONTENT_MARGIN_TOP: i32 = 30;

/// What a press of a button asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionsAction {
    /// "Done", or Escape: back to the screen before.
    Done,
    /// "Online...", which a game outside a world has.
    Online,
    /// "Difficulty..." and the rest of a world's options, in a world.
    WorldOptions,
    SkinCustomisation,
    Sounds,
    Video,
    Controls,
    Language,
    Chat,
    ResourcePacks,
    Accessibility,
    Telemetry,
    CreditsAndAttribution,
}

/// What an input did: the screen to open or leave, and a new field of view.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OptionsInput {
    pub action: Option<OptionsAction>,
    pub fov: Option<i32>,
}

/// The widget the keyboard or the mouse has focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Fov,
    Button(usize),
}

pub struct OptionsScreen {
    arena: Arena,
    title: Widget,
    fov: OptionSlider<IntRange>,
    /// The header's second button, the grid's ten and Done, in the order
    /// they are added to the screen.
    buttons: Vec<(Widget, OptionsAction)>,
    size: (i32, i32),
    in_world: bool,
    focus: Option<Focus>,
    /// Whether the mouse holds the focused widget (`isDragging`).
    dragging: bool,
}

fn runs(lang: &Lang, key: &str) -> Vec<Run> {
    lang.runs(&Component::translatable(key), &Style::default())
}

impl OptionsScreen {
    /// `OptionsScreen.init`, for a field of view, in a world or not, in a
    /// screen of `width` by `height` units.
    #[expect(
        clippy::too_many_lines,
        reason = "`init` builds each widget and layout in turn"
    )]
    pub fn new(
        font: &Font,
        lang: &Lang,
        (width, height): (i32, i32),
        fov: i32,
        in_world: bool,
    ) -> Self {
        let mut arena = Arena::default();
        let title_text = runs(lang, "options.title");
        let title_rect = arena.add(font.width_of_runs(&title_text), 9);
        let fov_rect = arena.add(150, 20);
        let slider = OptionSlider::new(
            lang,
            fov_rect,
            IntRange::new(30, 110),
            Component::translatable("options.fov"),
            text::fov,
            fov,
        );
        let mut buttons = Vec::new();
        let mut button = |arena: &mut Arena, key: &str, action: OptionsAction, width: i32| {
            let rect = arena.add(width, 20);
            buttons.push((
                Widget {
                    rect,
                    active: true,
                    kind: Kind::Button {
                        label: runs(lang, key),
                    },
                },
                action,
            ));
            rect
        };
        let second = if in_world {
            button(
                &mut arena,
                "options.worldOptions.button",
                OptionsAction::WorldOptions,
                150,
            )
        } else {
            button(&mut arena, "options.online", OptionsAction::Online, 150)
        };
        let grid_rects: Vec<usize> = [
            (
                "options.skinCustomisation",
                OptionsAction::SkinCustomisation,
            ),
            ("options.sounds", OptionsAction::Sounds),
            ("options.video", OptionsAction::Video),
            ("options.controls", OptionsAction::Controls),
            ("options.language", OptionsAction::Language),
            ("options.chat", OptionsAction::Chat),
            ("options.resourcepack", OptionsAction::ResourcePacks),
            ("options.accessibility", OptionsAction::Accessibility),
            ("options.telemetry", OptionsAction::Telemetry),
            (
                "options.credits_and_attribution",
                OptionsAction::CreditsAndAttribution,
            ),
        ]
        .into_iter()
        .map(|(key, action)| button(&mut arena, key, action, 150))
        .collect();
        let done = button(&mut arena, "gui.done", OptionsAction::Done, 200);
        // A player outside Mojang's services is offline: no telemetry.
        buttons[9].0.active = false;

        // The header: the title over the two buttons, in the middle of a
        // frame as wide as the screen and 61 tall.
        let mut sub_header = Grid::new();
        sub_header.column_spacing = 8;
        sub_header.add(Item::Leaf(fov_rect), (0, 0), (1, 1), Settings::default());
        sub_header.add(Item::Leaf(second), (0, 1), (1, 1), Settings::default());
        let mut header = Grid::new();
        header.row_spacing = 8;
        header.add(
            Item::Leaf(title_rect),
            (0, 0),
            (1, 1),
            Settings::default().align_horizontally(0.5),
        );
        header.add(
            Item::Grid(Box::new(sub_header)),
            (1, 0),
            (1, 1),
            Settings::default(),
        );
        header.arrange(&mut arena);
        let frame = (
            0,
            0,
            width.max(header.width),
            HEADER_HEIGHT.max(header.height),
        );
        place_in_frame(&mut arena, Item::Grid(Box::new(header)), frame, (0.5, 0.5));

        // The contents: two columns of buttons, below the header unless the
        // screen is too short.
        let mut contents = Grid::new();
        contents.defaults = Settings::default()
            .padding(4, 0, 4, 4)
            .align_horizontally(0.5);
        for rect in grid_rects {
            contents.add_in_row(Item::Leaf(rect), 2, 1, contents.defaults);
        }
        contents.arrange(&mut arena);
        let top = (frame.3 + CONTENT_MARGIN_TOP).min(height - FOOTER_HEIGHT - contents.height);
        let frame = (0, top, width.max(contents.width), contents.height);
        place_in_frame(
            &mut arena,
            Item::Grid(Box::new(contents)),
            frame,
            (0.5, 0.5),
        );

        // The footer: Done, in the middle of the last 33 units.
        let frame = (0, height - FOOTER_HEIGHT, width, FOOTER_HEIGHT);
        place_in_frame(&mut arena, Item::Leaf(done), frame, (0.5, 0.5));

        Self {
            arena,
            title: Widget {
                rect: title_rect,
                active: false,
                kind: Kind::Text { text: title_text },
            },
            fov: slider,
            buttons,
            size: (width, height),
            in_world,
            focus: None,
            dragging: false,
        }
    }

    /// The screen again, for another size and the field of view as it is.
    #[must_use]
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32), fov: i32) -> Self {
        Self::new(font, lang, size, fov, self.in_world)
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The menu background, then each widget in the order they were added.
    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        self.fov.draw(frame, atlas, font, &self.arena, mouse);
        for (index, (widget, _)) in self.buttons.iter().enumerate() {
            let highlighted =
                widget.contains(&self.arena, mouse) || self.focus == Some(Focus::Button(index));
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
    }

    /// `ContainerEventHandler.setFocused`: the widget before loses it.
    fn focus_on(&mut self, focus: Option<Focus>, input: InputType) {
        if self.focus == focus {
            return;
        }
        if self.focus == Some(Focus::Fov) {
            self.fov.set_focused(false, input);
        }
        self.focus = focus;
        if focus == Some(Focus::Fov) {
            self.fov.set_focused(true, input);
        }
    }

    /// A press of the primary button at `mouse`: the first widget there that
    /// is active takes it, and the focus.
    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64)) -> OptionsInput {
        let mut input = OptionsInput::default();
        let at = (mouse.0 as i32, mouse.1 as i32);
        let hit = if self.fov.is_mouse_over(&self.arena, mouse) {
            Some(Focus::Fov)
        } else {
            self.buttons
                .iter()
                .position(|(widget, _)| widget.is_active() && widget.contains(&self.arena, at))
                .map(Focus::Button)
        };
        let Some(hit) = hit else {
            return input;
        };
        match hit {
            Focus::Fov => input.fov = self.fov.clicked(lang, &self.arena, mouse),
            Focus::Button(index) => input.action = Some(self.buttons[index].1),
        }
        self.focus_on(Some(hit), InputType::Mouse);
        self.dragging = true;
        input
    }

    /// The mouse moved with the primary button down.
    pub fn dragged(&mut self, lang: &Lang, x: f64) -> OptionsInput {
        let mut input = OptionsInput::default();
        if self.dragging && self.focus == Some(Focus::Fov) {
            input.fov = self.fov.dragged(lang, &self.arena, x);
        }
        input
    }

    /// The primary button let go.
    pub fn released(&mut self, lang: &Lang) {
        if self.dragging {
            self.dragging = false;
            if self.focus == Some(Focus::Fov) {
                self.fov.released(lang);
            }
        }
    }

    /// Tab (and Shift+Tab): the focus moves to the next widget that is
    /// active, round the end.
    pub fn tab(&mut self, forward: bool) {
        let order: Vec<Focus> = std::iter::once(Focus::Fov)
            .chain((0..self.buttons.len()).map(Focus::Button))
            .filter(|focus| match focus {
                Focus::Fov => true,
                Focus::Button(index) => self.buttons[*index].0.is_active(),
            })
            .collect();
        let at = self
            .focus
            .and_then(|focus| order.iter().position(|candidate| *candidate == focus));
        let next = match (at, forward) {
            (None, true) => order.first(),
            (None, false) => order.last(),
            (Some(at), true) => order.get(at + 1).or(order.first()),
            (Some(at), false) => at
                .checked_sub(1)
                .map_or(order.last(), |before| order.get(before)),
        };
        self.focus_on(next.copied(), InputType::KeyboardTab);
    }

    /// `Screen.keyPressed`'s arrow navigation: the focus goes to the widget
    /// next to the focused one in the direction.
    fn navigate(&mut self, direction: Direction) {
        let (width, height) = self.size;
        let rectangle = |rect: usize| {
            let rect = self.arena.0[rect];
            Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        };
        // The children in the order they were added: the title, the field
        // of view, then the buttons.
        let mut children = vec![
            Candidate {
                rectangle: rectangle(self.title.rect),
                focusable: false,
            },
            Candidate {
                rectangle: rectangle(self.fov.rect()),
                focusable: true,
            },
        ];
        children.extend(self.buttons.iter().map(|(widget, _)| Candidate {
            rectangle: rectangle(widget.rect),
            focusable: widget.is_active(),
        }));
        let focused = self.focus.map(|focus| match focus {
            Focus::Fov => 1,
            Focus::Button(index) => index + 2,
        });
        let container = Rectangle {
            x: 0,
            y: 0,
            width,
            height,
        };
        let next = next_in_direction(&children, focused, direction, container);
        let focus = next.map(|index| match index {
            1 => Focus::Fov,
            index => Focus::Button(index - 2),
        });
        if focus.is_some() {
            self.focus_on(focus, InputType::KeyboardArrow);
        }
    }

    /// An arrow or Enter key for the focused widget; an arrow it does not
    /// take moves the focus.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey) -> OptionsInput {
        let mut input = OptionsInput::default();
        let direction = match key {
            SliderKey::Left => Some(Direction::Left),
            SliderKey::Right => Some(Direction::Right),
            SliderKey::Up => Some(Direction::Up),
            SliderKey::Down => Some(Direction::Down),
            SliderKey::Select => None,
        };
        if let Some(direction) = direction {
            if self.focus == Some(Focus::Fov) && matches!(key, SliderKey::Left | SliderKey::Right) {
                let (taken, value) = self.fov.key_pressed(lang, &self.arena, key);
                input.fov = value;
                if taken {
                    return input;
                }
            }
            self.navigate(direction);
            return input;
        }
        match self.focus {
            Some(Focus::Fov) => input.fov = self.fov.key_pressed(lang, &self.arena, key).1,
            Some(Focus::Button(index)) if key == SliderKey::Select => {
                let (widget, action) = &self.buttons[index];
                if widget.is_active() {
                    input.action = Some(*action);
                }
            }
            _ => {}
        }
        input
    }
}
