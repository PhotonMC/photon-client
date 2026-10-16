//! The key binds screen (`KeyBindsScreen`), opened from the controls screen: the list of key
//! mappings, Reset Keys and Done. A mapping's button waits for the next key or mouse button,
//! which it is bound to (Escape unbinds it).

use crate::gui::FontRuns;
use std::cell::Cell;

use photon_data::keys::InputKey;
use photon_game::options::default_bindings;
use photon_game::text::{Component, Lang, Style};

use crate::gui::font::Font;
use crate::gui::layout::{Arena, Item, horizontal, place_in_frame};
use crate::gui::options::list::Nav;
use crate::gui::widgets::navigation::{Candidate, Direction, Rectangle, next_in_direction};
use crate::gui::widgets::slider::{InputType, SliderKey};
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

mod list;

use list::{Binds, Focus, KeyBindsList, Row};

/// `HeaderAndFooterLayout`'s default header and footer heights.
const HEADER_AND_FOOTER: i32 = 33;

/// A key given to the screen: Escape, Tab (forward or back), an arrow or Enter, or another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindKey {
    Escape,
    Tab(bool),
    Menu(SliderKey),
    Other,
}

/// What an input to the screen asks for.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct KeyBindsInput {
    /// Done, or Escape: back to the controls screen.
    pub done: bool,
}

/// Where the focus is on the screen: a widget of the list, Reset Keys or Done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    List(Focus),
    ResetAll,
    Done,
}

pub struct KeyBindsScreen {
    arena: Arena,
    title: Widget,
    reset_all: Widget,
    /// `resetButton.active`, which the screen sets after each frame it draws.
    reset_active: Cell<bool>,
    done: Widget,
    list: KeyBindsList,
    /// Which footer button has the focus (the list keeps its own).
    footer_focus: Option<Target>,
    keys: Vec<InputKey>,
    defaults: Vec<InputKey>,
    /// `selectedKey`: the mapping waiting for its key.
    selected: Option<usize>,
    /// `glfwGetKeyName`.
    names: fn(InputKey) -> Option<String>,
    size: (i32, i32),
    in_world: bool,
}

impl KeyBindsScreen {
    /// `init` of a screen `size` big over the keys each mapping is bound to, naming keys by
    /// `names`.
    pub fn new(
        (font, lang): (&Font, &Lang),
        size: (i32, i32),
        in_world: bool,
        keys: &[InputKey],
        names: fn(InputKey) -> Option<String>,
    ) -> Self {
        let mut arena = Arena::default();
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let text = runs("controls.keybinds.title");
        let title = Widget {
            rect: arena.add(font.width_of_runs(&text), 9),
            active: false,
            kind: Kind::Text { text },
        };
        let mut button = |key: &str| Widget {
            rect: arena.add(150, 20),
            active: true,
            kind: Kind::Button { label: runs(key) },
        };
        let (reset_all, done) = (button("controls.resetAll"), button("gui.done"));
        let defaults = default_bindings();
        let binds = Binds {
            keys,
            defaults: &defaults,
            selected: None,
            names,
        };
        let list = KeyBindsList::new(
            font,
            lang,
            (size.0, (HEADER_AND_FOOTER, size.1 - 2 * HEADER_AND_FOOTER)),
            in_world,
            &binds,
        );
        let mut screen = Self {
            arena,
            title,
            reset_all,
            reset_active: Cell::new(true),
            done,
            list,
            footer_focus: None,
            keys: keys.to_vec(),
            defaults,
            selected: None,
            names,
            size,
            in_world,
        };
        screen.arrange();
        screen
    }

    /// The keys each mapping is bound to now, in `keyMappings` order.
    pub fn bindings(&self) -> &[InputKey] {
        &self.keys
    }

    /// `repositionElements`: the title in the header, the buttons in the footer, the list
    /// between.
    fn arrange(&mut self) {
        let (width, height) = self.size;
        place_in_frame(
            &mut self.arena,
            Item::Leaf(self.title.rect),
            (0, 0, width, HEADER_AND_FOOTER),
            (0.5, 0.5),
        );
        let mut footer = horizontal(&self.arena, &[self.reset_all.rect, self.done.rect], 8);
        footer.arrange(&mut self.arena);
        place_in_frame(
            &mut self.arena,
            Item::Grid(Box::new(footer)),
            (0, height - HEADER_AND_FOOTER, width, HEADER_AND_FOOTER),
            (0.5, 0.5),
        );
        self.list
            .resize(width, (HEADER_AND_FOOTER, height - 2 * HEADER_AND_FOOTER));
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
        self.arrange();
    }

    pub fn scroll_amount(&self) -> f64 {
        self.list.scroll_amount()
    }

    /// `resetMappingAndUpdateButtons`.
    fn refresh(&mut self, lang: &Lang) {
        let binds = Binds {
            keys: &self.keys,
            defaults: &self.defaults,
            selected: self.selected,
            names: self.names,
        };
        self.list.refresh(lang, &binds);
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
        let pending = self.list.draw(frame, atlas, font, mouse);
        let mut reset_all = self.reset_all.clone();
        reset_all.active = self.reset_active.get();
        for (widget, target) in [(&reset_all, Target::ResetAll), (&self.done, Target::Done)] {
            let lit = widget.contains(&self.arena, mouse) || self.footer_focus == Some(target);
            widget.draw(frame, atlas, font, &self.arena, lit);
        }
        if let Some(pending) = pending {
            pending.draw(frame, atlas, font, mouse, self.size);
        }
        self.reset_active.set(self.keys != self.defaults);
    }

    /// A press of a widget: a mapping's button waits for its key, a row's reset and Reset Keys
    /// bind the defaults again, Done goes back.
    fn press(&mut self, lang: &Lang, target: Target) -> KeyBindsInput {
        match target {
            Target::List((row, child)) => {
                let Row::Key { mapping, .. } = self.list.rows[row] else {
                    return KeyBindsInput::default();
                };
                if child == 0 {
                    self.selected = Some(mapping);
                } else {
                    self.keys[mapping] = self.defaults[mapping];
                }
            }
            Target::ResetAll => self.keys.clone_from(&self.defaults),
            Target::Done => return KeyBindsInput { done: true },
        }
        self.refresh(lang);
        KeyBindsInput::default()
    }

    /// Whether a target can take the focus and a press.
    fn footer_active(&self, target: Target) -> bool {
        match target {
            Target::ResetAll => self.reset_active.get(),
            _ => true,
        }
    }

    /// A press of mouse button `button`: the mapping waiting for its key is bound to it; else a
    /// press of the primary button goes to the widget under it.
    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), button: i32) -> KeyBindsInput {
        if let Some(mapping) = self.selected.take() {
            self.keys[mapping] = InputKey::Mouse(button);
            self.refresh(lang);
            return KeyBindsInput::default();
        }
        if button != 0 {
            return KeyBindsInput::default();
        }
        let (took, pressed) = self.list.clicked(mouse);
        if took {
            self.footer_focus = None;
            return pressed.map_or_else(KeyBindsInput::default, |focus| {
                self.press(lang, Target::List(focus))
            });
        }
        let at = (mouse.0 as i32, mouse.1 as i32);
        let hit = [
            (self.reset_all.rect, Target::ResetAll),
            (self.done.rect, Target::Done),
        ]
        .into_iter()
        .find(|&(rect, target)| {
            let rect = self.arena.0[rect];
            self.footer_active(target)
                && at.0 >= rect.x
                && at.1 >= rect.y
                && at.0 < rect.x + rect.width
                && at.1 < rect.y + rect.height
        });
        let Some((_, target)) = hit else {
            return KeyBindsInput::default();
        };
        self.apply(target, InputType::Mouse);
        self.press(lang, target)
    }

    pub fn dragged(&mut self, mouse: (f64, f64), dy: f64) {
        self.list.dragged(mouse, dy);
    }

    pub fn released(&mut self) {
        self.list.released();
    }

    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) {
        self.list.scrolled(mouse, scroll_y);
    }

    /// A key `key` pressed: the mapping waiting for its key is bound to it (Escape unbinds);
    /// else Escape goes back, Tab and the arrows move the focus, Enter presses.
    pub fn key_pressed(&mut self, lang: &Lang, key: InputKey, kind: BindKey) -> KeyBindsInput {
        if let Some(mapping) = self.selected.take() {
            self.keys[mapping] = if kind == BindKey::Escape {
                InputKey::Keysym(-1)
            } else {
                key
            };
            self.refresh(lang);
            return KeyBindsInput::default();
        }
        match kind {
            BindKey::Escape => return KeyBindsInput { done: true },
            BindKey::Tab(forward) => self.tab(forward),
            BindKey::Menu(SliderKey::Select) => {
                let target = if self.list.screen_focused {
                    self.list.focus.map(Target::List)
                } else {
                    self.footer_focus
                };
                if let Some(target) = target
                    && self.is_active(target)
                {
                    return self.press(lang, target);
                }
            }
            BindKey::Menu(SliderKey::Left) => self.arrow(Direction::Left),
            BindKey::Menu(SliderKey::Right) => self.arrow(Direction::Right),
            BindKey::Menu(SliderKey::Up) => self.arrow(Direction::Up),
            BindKey::Menu(SliderKey::Down) => self.arrow(Direction::Down),
            BindKey::Other => {}
        }
        KeyBindsInput::default()
    }

    /// Whether the focused target takes a press (a category's name does not).
    fn is_active(&self, target: Target) -> bool {
        match target {
            Target::List((row, child)) => match &self.list.rows[row] {
                Row::Key { change, reset, .. } => [change, reset][child].is_active(),
                Row::Category { .. } => false,
            },
            footer => self.footer_active(footer),
        }
    }

    /// The focus moves to `target`; what had it lets go.
    fn apply(&mut self, target: Target, input: InputType) {
        self.footer_focus = None;
        match target {
            Target::List(focus) => self.list.move_focus(Some(focus), input),
            footer => {
                self.list.move_focus(None, input);
                self.footer_focus = Some(footer);
            }
        }
    }

    /// `handleTabNavigation` over the screen's widgets in order: the list, Reset Keys, Done.
    fn tab_target(&self, forward: bool) -> Option<Target> {
        if self.list.screen_focused
            && let Some(focus) = self.list.next_focus(Nav::Tab(forward))
        {
            return Some(Target::List(focus));
        }
        let at = if self.list.screen_focused {
            Some(0)
        } else {
            match self.footer_focus {
                Some(Target::ResetAll) => Some(1),
                Some(_) => Some(2),
                None => None,
            }
        };
        let order: Vec<usize> = match (at, forward) {
            (Some(at), true) => (at + 1..3).collect(),
            (Some(at), false) => (0..at).rev().collect(),
            (None, true) => (0..3).collect(),
            (None, false) => (0..3).rev().collect(),
        };
        order.into_iter().find_map(|child| match child {
            0 => self.list.next_focus(Nav::Tab(forward)).map(Target::List),
            1 => self
                .footer_active(Target::ResetAll)
                .then_some(Target::ResetAll),
            _ => Some(Target::Done),
        })
    }

    /// Tab (and Shift+Tab): the focus moves on, and past the last widget round to the first.
    fn tab(&mut self, forward: bool) {
        let mut target = self.tab_target(forward);
        if target.is_none() {
            self.footer_focus = None;
            self.list.move_focus(None, InputType::None);
            target = self.tab_target(forward);
        }
        if let Some(target) = target {
            self.apply(target, InputType::KeyboardTab);
        }
    }

    /// `handleArrowNavigation`: the list's own step, else the widget next to the focused one.
    fn arrow(&mut self, direction: Direction) {
        let list_step = self.list.next_focus(Nav::Arrow(direction));
        if self.list.screen_focused
            && let Some(focus) = list_step
        {
            self.apply(Target::List(focus), InputType::KeyboardArrow);
            return;
        }
        let rectangle = |rect: usize| {
            let rect = self.arena.0[rect];
            Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        };
        let children = [
            Candidate {
                rectangle: rectangle(self.title.rect),
                focusable: false,
            },
            Candidate {
                rectangle: self.list.rectangle(),
                focusable: list_step.is_some(),
            },
            Candidate {
                rectangle: rectangle(self.reset_all.rect),
                focusable: self.footer_active(Target::ResetAll),
            },
            Candidate {
                rectangle: rectangle(self.done.rect),
                focusable: true,
            },
        ];
        let focused = if self.list.screen_focused {
            Some(1)
        } else {
            match self.footer_focus {
                Some(Target::ResetAll) => Some(2),
                Some(_) => Some(3),
                None => None,
            }
        };
        let container = Rectangle {
            x: 0,
            y: 0,
            width: self.size.0,
            height: self.size.1,
        };
        match next_in_direction(&children, focused, direction, container) {
            Some(1) => {
                if let Some(focus) = list_step {
                    self.apply(Target::List(focus), InputType::KeyboardArrow);
                }
            }
            Some(2) => self.apply(Target::ResetAll, InputType::KeyboardArrow),
            Some(3) => self.apply(Target::Done, InputType::KeyboardArrow),
            _ => {}
        }
    }
}
