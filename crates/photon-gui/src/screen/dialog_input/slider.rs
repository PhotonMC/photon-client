//! `NumberRangeHandler.SliderImpl`: a slider over a range, which says its value in its label.

use photon_game::dialog::{NumberRangeInput, range_text};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::widgets::draw_label;
use crate::widgets::navigation::Rectangle;

/// `AbstractSliderButton.HANDLE_WIDTH`.
const HANDLE: i32 = 8;

pub(super) struct Slider {
    spec: NumberRangeInput,
    lang: Lang,
    /// Where the handle is, 0 to 1.
    value: f64,
    can_change: bool,
    dragging: bool,
    label: Vec<Run>,
}

impl Slider {
    pub(super) fn new(lang: &Lang, spec: &NumberRangeInput) -> Self {
        let mut slider = Self {
            spec: spec.clone(),
            lang: lang.clone(),
            value: f64::from(spec.range.initial_slider()),
            can_change: false,
            dragging: false,
            label: Vec::new(),
        };
        slider.update_message();
        slider
    }

    /// `floatValueToSend`.
    pub(super) fn scaled(&self) -> f32 {
        self.spec.range.scaled(self.value as f32)
    }

    pub(super) fn text(&self) -> String {
        range_text(self.scaled())
    }

    /// `computeMessage`: the label in its format, with the value as text.
    fn update_message(&mut self) {
        let shown = Component::translatable_with(
            self.spec.label_format.as_str(),
            vec![
                self.spec.label.clone(),
                Component::literal(self.text().as_str()),
            ],
        );
        self.label = self.lang.runs(&shown, &Style::default());
    }

    fn set_value(&mut self, value: f64) {
        self.value = value.clamp(0.0, 1.0);
        self.update_message();
    }

    fn set_from_mouse(&mut self, x: i32, rect: Rectangle) {
        let left = f64::from(rect.x + HANDLE / 2);
        self.set_value((f64::from(x) - left) / f64::from(rect.width - HANDLE));
    }

    pub(super) fn clicked(&mut self, x: i32, rect: Rectangle) {
        self.dragging = true;
        self.set_from_mouse(x, rect);
    }

    pub(super) fn dragged(&mut self, x: i32, rect: Rectangle) {
        if self.dragging {
            self.set_from_mouse(x, rect);
        }
    }

    pub(super) fn released(&mut self) {
        // `AbstractSliderButton.onRelease`: the click comes when the button lets go.
        if self.dragging {
            crate::ui_sound::play_down_sound();
        }
        self.dragging = false;
    }

    /// Enter or Space: the keys move the handle or not.
    pub(super) fn select(&mut self) {
        self.can_change = !self.can_change;
    }

    /// The focus came by the mouse or Tab, which lets the keys move the handle, or went.
    pub(super) fn focus(&mut self, gained: bool) {
        self.can_change = gained;
    }

    /// An arrow: one pixel of the bar (`float direction / (width - 8)`). Whether it was taken.
    pub(super) fn arrow(&mut self, right: bool, width: i32) -> bool {
        if !self.can_change {
            return false;
        }
        let direction = if right { 1.0_f32 } else { -1.0_f32 };
        self.set_value(self.value + f64::from(direction / (width - HANDLE) as f32));
        true
    }

    pub(super) fn copy_from(&mut self, old: &Self) {
        self.value = old.value;
        self.update_message();
    }

    pub(super) fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        rect: Rectangle,
        (focused, hovered): (bool, bool),
    ) {
        let bar = if focused && !self.can_change {
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
            (rect.x, rect.y, rect.width, rect.height),
        );
        let handle = if !hovered && !self.can_change {
            "widget/slider_handle"
        } else {
            "widget/slider_handle_highlighted"
        };
        let x = rect.x + (self.value * f64::from(rect.width - HANDLE)) as i32;
        frame.nine_slice_edges(
            atlas,
            handle,
            (HANDLE, 20),
            (2, 2, 2, 3),
            false,
            (x, rect.y, HANDLE, rect.height),
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
