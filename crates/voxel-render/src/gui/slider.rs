//! The slider (`AbstractSliderButton`) and the option's slider
//! (`OptionInstanceSliderButton`): a handle that follows the mouse or the
//! arrow keys along a bar, and a label that says what the value is.

use voxel_game::option::{IntRange, MappedIntRange, SliderableEnum, UnitDouble};
use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::tooltip::{Holder, joined};
use crate::gui::widgets::draw_label;
use crate::gui::{GuiAtlas, GuiFrame};

/// `HANDLE_WIDTH`.
const HANDLE_WIDTH: i32 = 8;

/// What the window last took in (`Minecraft.getLastInputType`), which
/// decides whether a focused slider lets the arrow keys move it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    None,
    Mouse,
    KeyboardArrow,
    KeyboardTab,
}

/// The arrow keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderKey {
    Left,
    Right,
    /// The vertical arrows, which a slider does not take.
    Up,
    Down,
    /// Enter, Space or the keypad's Enter.
    Select,
}

/// A value set that a slider walks: where a value stands along the bar, and
/// the value at a place on it (`OptionInstance.SliderableValueSet`).
pub trait SliderValues {
    type Value: Clone + PartialEq;

    fn position_of(&self, value: &Self::Value) -> f64;
    fn value_at(&self, slider: f64) -> Self::Value;
    /// The value one step along, if the set has steps.
    fn next(&self, _current: &Self::Value) -> Option<Self::Value> {
        None
    }
    fn previous(&self, _current: &Self::Value) -> Option<Self::Value> {
        None
    }
}

impl SliderValues for IntRange {
    type Value = i32;

    fn position_of(&self, value: &i32) -> f64 {
        self.to_slider_value(*value)
    }

    fn value_at(&self, slider: f64) -> i32 {
        self.from_slider_value(slider)
    }

    fn next(&self, current: &i32) -> Option<i32> {
        Some(IntRange::next(*self, *current))
    }

    fn previous(&self, current: &i32) -> Option<i32> {
        Some(IntRange::previous(*self, *current))
    }
}

impl SliderValues for UnitDouble {
    type Value = f64;

    fn position_of(&self, value: &f64) -> f64 {
        Self::to_slider_value(*value)
    }

    fn value_at(&self, slider: f64) -> f64 {
        Self::from_slider_value(slider)
    }
}

impl<R, To, FromValue> SliderValues for MappedIntRange<R, To, FromValue>
where
    R: Clone + PartialEq,
    To: Fn(i32) -> R,
    FromValue: Fn(&R) -> i32,
{
    type Value = R;

    fn position_of(&self, value: &R) -> f64 {
        self.to_slider_value(value)
    }

    fn value_at(&self, slider: f64) -> R {
        self.from_slider_value(slider)
    }

    fn next(&self, current: &R) -> Option<R> {
        MappedIntRange::next(self, current)
    }

    fn previous(&self, current: &R) -> Option<R> {
        MappedIntRange::previous(self, current)
    }
}

impl<T: Clone + PartialEq> SliderValues for SliderableEnum<T> {
    type Value = T;

    fn position_of(&self, value: &T) -> f64 {
        self.to_slider_value(value)
    }

    fn value_at(&self, slider: f64) -> T {
        self.from_slider_value(slider)
    }

    fn next(&self, current: &T) -> Option<T> {
        Some(SliderableEnum::next(self, current))
    }

    fn previous(&self, current: &T) -> Option<T> {
        Some(SliderableEnum::previous(self, current))
    }
}

/// An option's slider: the bar, its value set and the label it makes of the
/// option's name and value.
pub struct OptionSlider<S: SliderValues> {
    rect: usize,
    active: bool,
    /// `value`: where the handle is, 0 to 1.
    position: f64,
    can_change: bool,
    dragging: bool,
    focused: bool,
    label: Vec<Run>,
    values: S,
    caption: Component,
    text: fn(&Component, &S::Value) -> Component,
    /// The option's own value, `instance.get()`.
    current: S::Value,
    /// What the tooltip says of a value, and the tooltip.
    tip: Option<SliderTip<S::Value>>,
    tooltip: Holder,
}

/// The lines of a tooltip for a value (`None` a blank line).
pub type SliderTip<V> = Box<dyn Fn(&V) -> Vec<Option<Component>>>;

impl<S: SliderValues> OptionSlider<S> {
    /// `OptionInstanceSliderButton`, for the bar at `rect` of the arena.
    pub fn new(
        lang: &Lang,
        rect: usize,
        values: S,
        caption: Component,
        text: fn(&Component, &S::Value) -> Component,
        current: S::Value,
    ) -> Self {
        let mut slider = Self {
            rect,
            active: true,
            position: values.position_of(&current),
            can_change: false,
            dragging: false,
            focused: false,
            label: Vec::new(),
            values,
            caption,
            text,
            current,
            tip: None,
            tooltip: Holder::default(),
        };
        slider.update_message(lang);
        slider
    }

    /// `OptionInstanceSliderButton`'s tooltip: it follows the handle.
    #[must_use]
    pub fn with_tip(mut self, lang: &Lang, tip: SliderTip<S::Value>) -> Self {
        self.tip = Some(tip);
        self.update_message(lang);
        self
    }

    pub fn tooltip(&self) -> &Holder {
        &self.tooltip
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// The bar's place in the arena.
    pub fn rect(&self) -> usize {
        self.rect
    }

    /// `setFocused`: losing the focus stops the keys moving the handle; a
    /// focus the mouse or Tab gave lets them.
    pub fn set_focused(&mut self, focused: bool, last_input: InputType) {
        self.focused = focused;
        if !focused {
            self.can_change = false;
        } else if matches!(last_input, InputType::Mouse | InputType::KeyboardTab) {
            self.can_change = true;
        }
    }

    pub fn value(&self) -> &S::Value {
        &self.current
    }

    /// `instance.set` and `resetValue`: the option takes a value from
    /// elsewhere and the handle goes to it.
    pub fn set_current(&mut self, lang: &Lang, value: S::Value) {
        self.current = value;
        self.reset(lang);
    }

    /// The words on the bar.
    pub fn label(&self) -> &[Run] {
        &self.label
    }

    fn update_message(&mut self, lang: &Lang) {
        let at = self.values.value_at(self.position);
        let text = (self.text)(&self.caption, &at);
        self.label = lang.runs(&text, &Style::default());
        if let Some(tip) = &self.tip {
            self.tooltip.set(Some(joined(lang, &tip(&at))));
        }
    }

    fn width(&self, arena: &Arena) -> i32 {
        arena.0[self.rect].width
    }

    /// `setValue`: the handle moves (kept on the bar); the option takes the
    /// value it stands for, and the label follows.
    fn set_position(&mut self, lang: &Lang, new: f64) -> Option<S::Value> {
        let old = self.position;
        self.position = new.clamp(0.0, 1.0);
        let applied = (old != self.position).then(|| self.apply()).flatten();
        self.update_message(lang);
        applied
    }

    /// `applyUnsavedValue`: the option takes the value, if it is another.
    fn apply(&mut self) -> Option<S::Value> {
        let value = self.values.value_at(self.position);
        if value == self.current {
            return None;
        }
        self.current = value.clone();
        Some(value)
    }

    /// `resetValue`: the handle goes to where the option's value is.
    fn reset(&mut self, lang: &Lang) {
        let at = self.values.position_of(&self.current);
        if self.position != at {
            self.position = at;
            self.update_message(lang);
        }
    }

    fn set_from_mouse(&mut self, lang: &Lang, arena: &Arena, x: f64) -> Option<S::Value> {
        let left = f64::from(arena.0[self.rect].x + HANDLE_WIDTH / 2);
        let span = f64::from(self.width(arena) - HANDLE_WIDTH);
        self.set_position(lang, (x - left) / span)
    }

    /// `isMouseOver`: the mouse is on the bar, which is active.
    pub fn is_mouse_over(&self, arena: &Arena, (x, y): (f64, f64)) -> bool {
        let rect = arena.0[self.rect];
        self.active
            && x >= f64::from(rect.x)
            && y >= f64::from(rect.y)
            && x < f64::from(rect.x + rect.width)
            && y < f64::from(rect.y + rect.height)
    }

    /// A press of the primary button at `mouse`: the handle goes there and
    /// follows the mouse from now on. The option's new value, if it changed.
    pub fn clicked(&mut self, lang: &Lang, arena: &Arena, mouse: (f64, f64)) -> Option<S::Value> {
        if !self.is_mouse_over(arena, mouse) {
            return None;
        }
        self.dragging = self.active;
        self.set_from_mouse(lang, arena, mouse.0)
    }

    /// The mouse moved with the primary button down.
    pub fn dragged(&mut self, lang: &Lang, arena: &Arena, x: f64) -> Option<S::Value> {
        self.set_from_mouse(lang, arena, x)
    }

    /// The primary button let go: the handle settles on the option's value.
    pub fn released(&mut self, lang: &Lang) {
        self.dragging = false;
        self.reset(lang);
    }

    /// Whether the mouse holds the handle.
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// `keyPressed`: Enter lets the keys move the handle or not; the arrows
    /// step through the values or, with none, by a pixel. Whether the key
    /// was taken, and the option's new value if it changed.
    pub fn key_pressed(
        &mut self,
        lang: &Lang,
        arena: &Arena,
        key: SliderKey,
    ) -> (bool, Option<S::Value>) {
        if key == SliderKey::Select {
            self.can_change = !self.can_change;
            return (true, None);
        }
        if !self.can_change || !matches!(key, SliderKey::Left | SliderKey::Right) {
            return (false, None);
        }
        let left = key == SliderKey::Left;
        let current = self.values.value_at(self.position);
        let stepped = if left {
            self.values.previous(&current)
        } else {
            self.values.next(&current)
        };
        if let Some(stepped) = stepped {
            let at = self.values.position_of(&stepped);
            return (true, self.set_position(lang, at));
        }
        // `float direction / (width - 8)`: a float division.
        let direction = if left { -1.0_f32 } else { 1.0_f32 };
        let step = f64::from(direction / (self.width(arena) - HANDLE_WIDTH) as f32);
        (true, self.set_position(lang, self.position + step))
    }

    /// The bar, the handle, then the label.
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
        let bar = if self.active && self.focused && !self.can_change {
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
        let handle = if !self.active || (!hovered && !self.can_change) {
            "widget/slider_handle"
        } else {
            "widget/slider_handle_highlighted"
        };
        let handle_x = rect.x + (self.position * f64::from(rect.width - HANDLE_WIDTH)) as i32;
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
            (&self.label, self.active),
            (rect.x + 2, rect.x + rect.width - 2),
            (rect.y, rect.y + rect.height),
        );
    }
}
