//! The jigsaw screen's levels slider: an `AbstractSliderButton` whose value is a unit double that
//! the screen turns into a whole number of levels.

use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::layout::Arena;
use crate::widgets::draw_label;
use crate::widgets::slider::InputType;

/// `HANDLE_WIDTH`.
const HANDLE_WIDTH: i32 = 8;

pub struct LevelsSlider {
    rect: usize,
    /// `value`: where the handle is, 0 to 1.
    value: f64,
    can_change: bool,
    dragging: bool,
    focused: bool,
    /// The screen's `levels`, which the bar's words say.
    levels: i32,
    label: Vec<Run>,
}

impl LevelsSlider {
    /// The slider the screen makes in `init`: the handle at the left, whatever `levels` says.
    pub fn new(lang: &Lang, rect: usize, levels: i32) -> Self {
        let mut slider = Self {
            rect,
            value: 0.0,
            can_change: false,
            dragging: false,
            focused: false,
            levels,
            label: Vec::new(),
        };
        slider.update_message(lang);
        slider
    }

    pub fn levels(&self) -> i32 {
        self.levels
    }

    fn update_message(&mut self, lang: &Lang) {
        let text = Component::translatable_with(
            "jigsaw_block.levels",
            vec![Component::literal(self.levels.to_string())],
        );
        self.label = lang.runs(&text, &Style::default());
    }

    /// `setValue`: the handle moves (kept on the bar), and `applyValue` takes the levels it stands
    /// for if it moved.
    fn set_value(&mut self, lang: &Lang, new: f64) {
        let old = self.value;
        self.value = new.clamp(0.0, 1.0);
        if old != self.value {
            self.levels = (self.value * 20.0).floor() as i32;
        }
        self.update_message(lang);
    }

    pub fn set_focused(&mut self, focused: bool, input: InputType) {
        self.focused = focused;
        if !focused {
            self.can_change = false;
        } else if matches!(input, InputType::Mouse | InputType::KeyboardTab) {
            self.can_change = true;
        }
    }

    /// A press of the primary button: the handle goes to the mouse and follows it.
    pub fn clicked(&mut self, lang: &Lang, arena: &Arena, x: i32) {
        self.dragging = true;
        self.set_by_mouse(lang, arena, x);
    }

    /// The mouse moved with the primary button down.
    pub fn dragged(&mut self, lang: &Lang, arena: &Arena, x: i32) {
        if self.dragging {
            self.set_by_mouse(lang, arena, x);
        }
    }

    pub fn released(&mut self) {
        self.dragging = false;
    }

    fn set_by_mouse(&mut self, lang: &Lang, arena: &Arena, x: i32) {
        let rect = arena.0[self.rect];
        let value = f64::from(x - (rect.x + 4)) / f64::from(rect.width - HANDLE_WIDTH);
        self.set_value(lang, value);
    }

    /// `keyPressed`: a selection key lets the arrows move the handle or not; the arrows then move
    /// it a pixel. Whether the key was taken.
    pub fn key_pressed(&mut self, lang: &Lang, arena: &Arena, key: glfw::Key) -> bool {
        if matches!(
            key,
            glfw::Key::Enter | glfw::Key::KpEnter | glfw::Key::Space
        ) {
            self.can_change = !self.can_change;
            return true;
        }
        let direction = match key {
            glfw::Key::Left => -1.0_f32,
            glfw::Key::Right => 1.0_f32,
            _ => return false,
        };
        if !self.can_change {
            return false;
        }
        let width = arena.0[self.rect].width - HANDLE_WIDTH;
        self.set_value(lang, self.value + f64::from(direction / width as f32));
        true
    }

    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        mouse: (i32, i32),
    ) {
        let rect = arena.0[self.rect];
        let hovered = mouse.0 >= rect.x
            && mouse.1 >= rect.y
            && mouse.0 < rect.x + rect.width
            && mouse.1 < rect.y + rect.height;
        let bar = if self.focused && !self.can_change {
            "widget/slider_highlighted"
        } else {
            "widget/slider"
        };
        frame.nine_slice(
            atlas,
            bar,
            (200, 20),
            1,
            false,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        );
        let handle = if hovered || self.can_change {
            "widget/slider_handle_highlighted"
        } else {
            "widget/slider_handle"
        };
        let handle_x = rect.x + (self.value * f64::from(rect.width - HANDLE_WIDTH)) as i32;
        frame.nine_slice_edges(
            atlas,
            handle,
            (HANDLE_WIDTH, 20),
            (2, 2, 2, 3),
            false,
            (handle_x, rect.y, HANDLE_WIDTH, rect.height),
        );
        draw_label(
            frame,
            atlas,
            font,
            (&self.label, true),
            (rect.x + 2, rect.x + rect.width - 2),
            (rect.y, rect.y + rect.height),
        );
    }
}
