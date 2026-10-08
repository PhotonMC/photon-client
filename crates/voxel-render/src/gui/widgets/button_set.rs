//! The buttons of a screen laid out by hand (`Screen.addRenderableWidget` in `init`), with what a
//! screen does with them: drawing them with the one under the mouse or the focus highlighted, the
//! tooltip of the one asked about, Tab and the arrows moving the focus, and a press asking for the
//! button's action.

use std::cell::Cell;

use voxel_game::text::{Component, Lang};

use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::tooltip::{Holder, Pending};
use crate::gui::widgets::navigation::{Candidate, Direction, Rectangle, next_in_direction};
use crate::gui::widgets::slider::InputType;
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

pub struct ButtonSet<A> {
    pub arena: Arena,
    buttons: Vec<(Widget, A, Option<Holder>)>,
    /// The button the keyboard has focused, an index of `buttons`.
    focus: Option<usize>,
    /// What the window last took in, which decides whether a focus shows its tooltip.
    input: Cell<InputType>,
    size: (i32, i32),
}

impl<A: Copy> ButtonSet<A> {
    pub fn new(size: (i32, i32)) -> Self {
        Self {
            arena: Arena::default(),
            buttons: Vec::new(),
            focus: None,
            input: Cell::new(InputType::Mouse),
            size,
        }
    }

    /// A button at `(x, y, width, height)`, with the tooltip `tooltip` names if any; its place
    /// in the arena.
    pub fn add(
        &mut self,
        (x, y, width, height): (i32, i32, i32, i32),
        (kind, active): (Kind, bool),
        action: A,
        tooltip: Option<(&Lang, &str)>,
    ) -> usize {
        let rect = self.arena.add(width, height);
        self.arena.0[rect].x = x;
        self.arena.0[rect].y = y;
        let holder = tooltip.map(|(lang, key)| Holder::of(lang, &Component::translatable(key)));
        self.buttons
            .push((Widget { rect, active, kind }, action, holder));
        rect
    }

    /// Wake or still the button at `index` of those added.
    pub fn set_active(&mut self, index: usize, active: bool) {
        self.buttons[index].0.active = active;
    }

    /// The screen size the buttons were laid out for.
    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// Whether the mouse is over a button.
    pub fn hovered(&self, index: usize, mouse: (i32, i32)) -> bool {
        self.buttons[index].0.contains(&self.arena, mouse)
    }

    /// Each button, the highlighted one under the mouse or with the focus.
    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        for (index, (widget, _, _)) in self.buttons.iter().enumerate() {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == Some(index);
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
    }

    /// `extractDeferredElements`: the tooltip of the button asked about over the rest (the first
    /// asked for, but the focused one's replaces it).
    pub fn draw_tooltip(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
    ) {
        let mut pending: Option<Pending> = None;
        for (index, (widget, _, tooltip)) in self.buttons.iter().enumerate() {
            let Some(tooltip) = tooltip else {
                continue;
            };
            let focused = self.focus == Some(index);
            let hovered = widget.contains(&self.arena, mouse);
            if let Some(asked) = tooltip.refresh(
                frame.millis,
                (hovered, focused),
                self.input.get(),
                self.rectangle(widget.rect),
            ) && (pending.is_none() || focused)
            {
                pending = Some(asked);
            }
        }
        if let Some(pending) = pending {
            pending.draw(frame, atlas, font, mouse, self.size);
        }
    }

    fn rectangle(&self, rect: usize) -> Rectangle {
        let rect = self.arena.0[rect];
        Rectangle {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }

    /// The mouse moved or was pressed: the window's last input is the mouse.
    pub fn mouse_input(&self) {
        self.input.set(InputType::Mouse);
    }

    /// An arrow key: the focus goes to the button next to the focused one.
    pub fn arrow(&mut self, direction: Direction) {
        self.input.set(InputType::KeyboardArrow);
        let children: Vec<Candidate> = self
            .buttons
            .iter()
            .map(|(widget, _, _)| Candidate {
                rectangle: self.rectangle(widget.rect),
                focusable: widget.is_active(),
            })
            .collect();
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

    /// Tab (and Shift+Tab): the focus moves to the next active button.
    pub fn tab(&mut self, forward: bool) {
        self.input.set(InputType::KeyboardTab);
        let count = self.buttons.len();
        let mut next = self.focus;
        for _ in 0..count {
            let index = match (next, forward) {
                (None, true) => 0,
                (None, false) => count - 1,
                (Some(index), true) => (index + 1) % count,
                (Some(index), false) => (index + count - 1) % count,
            };
            next = Some(index);
            if self.buttons[index].0.is_active() {
                self.focus = next;
                return;
            }
        }
    }

    /// A press of the primary button at `mouse`: what the active button there asks for.
    pub fn clicked(&self, mouse: (i32, i32)) -> Option<A> {
        self.mouse_input();
        self.buttons
            .iter()
            .find(|(widget, _, _)| widget.is_active() && widget.contains(&self.arena, mouse))
            .map(|(_, action, _)| *action)
    }

    /// Enter or Space: what the focused button asks for.
    pub fn pressed(&self) -> Option<A> {
        self.focus
            .and_then(|index| self.buttons.get(index))
            .filter(|(widget, _, _)| widget.is_active())
            .map(|(_, action, _)| *action)
    }
}
