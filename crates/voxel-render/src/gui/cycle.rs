//! The button that cycles through a list of values (`CycleButton`): a click
//! or Enter takes the next, Shift the one before, the wheel one either way.

use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::option_text::generic_value;
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

pub struct CycleButton<V> {
    widget: Widget,
    name: Component,
    values: Vec<V>,
    value: V,
    /// `-1` for a value that is not listed, as `indexOf` says.
    index: i32,
    value_text: fn(&V) -> Component,
    focused: bool,
}

impl<V: Clone + PartialEq> CycleButton<V> {
    /// `CycleButton.Builder.create`, with the button at `rect` of the arena
    /// and `initial` among `values`: "name: value".
    ///
    /// # Panics
    ///
    /// When `values` is empty, as the reference throws.
    pub fn new(
        lang: &Lang,
        rect: usize,
        name: Component,
        values: Vec<V>,
        value_text: fn(&V) -> Component,
        initial: &V,
    ) -> Self {
        assert!(!values.is_empty(), "No values for cycle button");
        let index = values
            .iter()
            .position(|value| value == initial)
            .map_or(-1, |index| index as i32);
        let mut button = Self {
            widget: Widget {
                rect,
                active: true,
                kind: Kind::Button { label: Vec::new() },
            },
            name,
            values,
            value: initial.clone(),
            index,
            value_text,
            focused: false,
        };
        button.update_label(lang);
        button
    }

    pub fn set_active(&mut self, active: bool) {
        self.widget.active = active;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    /// `setValue`: the button takes a value, which moves its place in the
    /// list if the list has it.
    pub fn set_value(&mut self, lang: &Lang, value: V) {
        if let Some(index) = self.values.iter().position(|candidate| *candidate == value) {
            self.index = index as i32;
        }
        self.value = value;
        self.update_label(lang);
    }

    /// `isMouseOver`: the button is active and the mouse is on it.
    pub fn is_mouse_over(&self, arena: &Arena, mouse: (f64, f64)) -> bool {
        self.widget.is_active()
            && self
                .widget
                .contains(arena, (mouse.0 as i32, mouse.1 as i32))
    }

    fn update_label(&mut self, lang: &Lang) {
        let text = generic_value(&self.name, (self.value_text)(self.value()));
        self.widget.kind = Kind::Button {
            label: lang.runs(&text, &Style::default()),
        };
    }

    /// The words on the button.
    pub fn label(&self) -> &[Run] {
        match &self.widget.kind {
            Kind::Button { label } => label,
            _ => &[],
        }
    }

    /// `cycleValue`: the value `delta` along the list, wrapping round.
    fn cycle(&mut self, lang: &Lang, delta: i32) -> V {
        let count = self.values.len() as i32;
        self.index = (self.index + delta).rem_euclid(count);
        self.value = self.values[self.index as usize].clone();
        self.update_label(lang);
        self.value.clone()
    }

    /// `onPress`: Shift takes the value before.
    pub fn press(&mut self, lang: &Lang, shift: bool) -> V {
        self.cycle(lang, if shift { -1 } else { 1 })
    }

    /// A press of the primary button at `mouse`.
    pub fn clicked(
        &mut self,
        lang: &Lang,
        arena: &Arena,
        mouse: (i32, i32),
        shift: bool,
    ) -> Option<V> {
        (self.widget.is_active() && self.widget.contains(arena, mouse))
            .then(|| self.press(lang, shift))
    }

    /// Enter, Space or the keypad's Enter, which the screen gives to the
    /// focused button.
    pub fn selected(&mut self, lang: &Lang, shift: bool) -> Option<V> {
        self.widget.is_active().then(|| self.press(lang, shift))
    }

    /// `mouseScrolled`: the wheel up takes the value before, down the next.
    pub fn scrolled(&mut self, lang: &Lang, scroll_y: f64) -> Option<V> {
        if scroll_y > 0.0 {
            Some(self.cycle(lang, -1))
        } else if scroll_y < 0.0 {
            Some(self.cycle(lang, 1))
        } else {
            None
        }
    }

    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        mouse: (i32, i32),
    ) {
        let highlighted = self.widget.contains(arena, mouse) || self.focused;
        self.widget.draw(frame, atlas, font, arena, highlighted);
    }
}
