//! The settings screens' shell (`OptionsSubScreen`) and the scrolling list of
//! options in it (`OptionsList`): a title over the list, Done under it, and
//! in the list headers and rows of one wide or two narrow option widgets.

use std::any::Any;

use photon_game::text::{Component, Lang, Run, Style};

use crate::layout::{Arena, Grid, Item, Settings, horizontal, place_in_frame};
use crate::options::text::on_off;
use crate::tooltip::{Holder, Pending};
use crate::widgets::cycle::{CycleButton, Tip};
use crate::widgets::navigation::{Candidate, Direction, Rectangle, next_in_direction};
use crate::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::widgets::slider::{InputType, OptionSlider, SliderKey, SliderTip, SliderValues};
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `HeaderAndFooterLayout(this)`: the default header and footer heights.
const HEADER_AND_FOOTER: i32 = 33;
/// `OptionsList.DEFAULT_ITEM_HEIGHT`.
const ITEM_HEIGHT: i32 = 25;
/// `getRowWidth`, and the width of a wide widget.
const ROW_WIDTH: i32 = 310;
/// The width of each of the two narrow widgets, and the step between them.
const SMALL_WIDTH: i32 = 150;
const COLUMN_STEP: i32 = 160;
/// `Entry.CONTENT_PADDING`.
const PADDING: i32 = 2;
/// `ScrollbarSettings.scrollRate`: half of an item's height.
const SCROLL_RATE: f64 = (ITEM_HEIGHT / 2) as f64;

/// A widget of an option, whatever it is made of.
pub trait Control {
    /// `isMouseOver`: the widget is active and the mouse is on it.
    fn is_mouse_over(&self, arena: &Arena, mouse: (f64, f64)) -> bool;
    /// The press of the primary button that landed on the widget: whether
    /// the option's value changed.
    fn pressed(&mut self, lang: &Lang, arena: &Arena, mouse: (f64, f64), shift: bool) -> bool;
    /// The mouse moved with the primary button down: whether the option's
    /// value changed.
    fn dragged(&mut self, lang: &Lang, arena: &Arena, x: f64) -> bool;
    /// The primary button let go.
    fn released(&mut self, lang: &Lang);
    fn set_focused(&mut self, focused: bool, input: InputType);
    fn set_active(&mut self, active: bool);
    /// `active`: an inactive widget takes no focus.
    fn is_active(&self) -> bool;
    /// A key the focused widget is given (`keyPressed`): whether it took it,
    /// and whether the option's value changed.
    fn key_pressed(
        &mut self,
        lang: &Lang,
        arena: &Arena,
        key: SliderKey,
        shift: bool,
    ) -> (bool, bool);
    fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        mouse: (i32, i32),
    );
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// The widget's tooltip, if it has one.
    fn tooltip(&self) -> Option<&Holder> {
        None
    }
}

impl<S: SliderValues + 'static> Control for OptionSlider<S>
where
    S::Value: 'static,
{
    fn tooltip(&self) -> Option<&Holder> {
        Some(OptionSlider::tooltip(self))
    }

    fn is_mouse_over(&self, arena: &Arena, mouse: (f64, f64)) -> bool {
        OptionSlider::is_mouse_over(self, arena, mouse)
    }

    fn pressed(&mut self, lang: &Lang, arena: &Arena, mouse: (f64, f64), _shift: bool) -> bool {
        self.clicked(lang, arena, mouse).is_some()
    }

    fn dragged(&mut self, lang: &Lang, arena: &Arena, x: f64) -> bool {
        OptionSlider::dragged(self, lang, arena, x).is_some()
    }

    fn released(&mut self, lang: &Lang) {
        // `AbstractSliderButton.onRelease`: the click comes when the button lets go.
        crate::ui_sound::play_down_sound();
        OptionSlider::released(self, lang);
    }

    fn set_focused(&mut self, focused: bool, input: InputType) {
        OptionSlider::set_focused(self, focused, input);
    }

    fn set_active(&mut self, active: bool) {
        OptionSlider::set_active(self, active);
    }

    fn is_active(&self) -> bool {
        OptionSlider::is_active(self)
    }

    fn key_pressed(
        &mut self,
        lang: &Lang,
        arena: &Arena,
        key: SliderKey,
        _shift: bool,
    ) -> (bool, bool) {
        let (taken, value) = OptionSlider::key_pressed(self, lang, arena, key);
        (taken, value.is_some())
    }

    fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        mouse: (i32, i32),
    ) {
        OptionSlider::draw(self, frame, atlas, font, arena, mouse);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<V: Clone + PartialEq + 'static> Control for CycleButton<V> {
    fn tooltip(&self) -> Option<&Holder> {
        Some(CycleButton::tooltip(self))
    }

    fn is_mouse_over(&self, arena: &Arena, mouse: (f64, f64)) -> bool {
        CycleButton::is_mouse_over(self, arena, mouse)
    }

    fn pressed(&mut self, lang: &Lang, _arena: &Arena, _mouse: (f64, f64), shift: bool) -> bool {
        crate::ui_sound::play_down_sound();
        self.press(lang, shift);
        true
    }

    fn dragged(&mut self, _lang: &Lang, _arena: &Arena, _x: f64) -> bool {
        false
    }

    fn released(&mut self, _lang: &Lang) {}

    fn set_focused(&mut self, focused: bool, _input: InputType) {
        CycleButton::set_focused(self, focused);
    }

    fn set_active(&mut self, active: bool) {
        CycleButton::set_active(self, active);
    }

    fn is_active(&self) -> bool {
        CycleButton::is_active(self)
    }

    fn key_pressed(
        &mut self,
        lang: &Lang,
        _arena: &Arena,
        key: SliderKey,
        shift: bool,
    ) -> (bool, bool) {
        // `AbstractButton.keyPressed`: Enter and Space press it.
        let pressed = key == SliderKey::Select && self.selected(lang, shift).is_some();
        if pressed {
            crate::ui_sound::play_down_sound();
        }
        (pressed, pressed)
    }

    fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        mouse: (i32, i32),
    ) {
        CycleButton::draw(self, frame, atlas, font, arena, mouse);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Makes the widget for the place `rect` of the list's arena.
pub type Build = Box<dyn FnOnce(&Lang, usize) -> Box<dyn Control>>;

/// A widget of an option and what a screen calls it by, to find it again
/// (`findOption`).
struct Child {
    tag: u8,
    rect: usize,
    control: Box<dyn Control>,
}

enum Content {
    /// `HeaderEntry`: a line of text and the space above it.
    Header { text: Widget, padding_top: i32 },
    /// `Entry`: one or two option widgets.
    Row(Vec<Child>),
}

struct Entry {
    y: i32,
    height: i32,
    content: Content,
}

/// The widget that has the focus: its entry, then its place in it.
type Focus = (usize, usize);

/// `FocusNavigationEvent`: Tab (forward or back) or an arrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    Tab(bool),
    Arrow(Direction),
}

/// `OptionsList`.
pub struct OptionsList {
    arena: Arena,
    place: Place,
    entries: Vec<Entry>,
    scroll: f64,
    /// Whether the mouse holds the scroll bar.
    scrolling: bool,
    focus: Option<Focus>,
    /// Whether the screen's focus is on the list, which a click on an entry
    /// gives it without any widget of it taking the focus.
    screen_focused: bool,
    /// Whether the mouse holds the focused widget (`isDragging`).
    dragging: bool,
    /// What the window last took in.
    input: InputType,
}

/// Builders of the widgets the settings lists are made of.
/// The value of a cycle button of a few named values: the key of its name.
pub type Named = &'static str;

pub fn translated(key: &Named) -> Component {
    Component::translatable(*key)
}

pub fn toggle(value: &bool) -> Component {
    on_off(*value)
}

pub fn counted(key: &str, value: i32) -> Component {
    Component::translatable_with(key, vec![Component::literal(value.to_string())])
}

/// `CommonComponents.optionStatus(caption, false)`.
pub fn off_composed(caption: &Component) -> Component {
    Component::translatable_with("options.off.composed", vec![caption.clone()])
}

/// A cycle button of an option.
pub fn cycle<V: Clone + PartialEq + 'static>(
    key: &'static str,
    values: Vec<V>,
    text: fn(&V) -> Component,
    initial: V,
) -> Build {
    Box::new(move |lang, rect| {
        Box::new(CycleButton::new(
            lang,
            rect,
            Component::translatable(key),
            values,
            text,
            &initial,
        ))
    })
}

pub fn boolean(key: &'static str, initial: bool) -> Build {
    cycle(key, vec![true, false], toggle, initial)
}

pub fn named(key: &'static str, values: &[Named], initial: Named) -> Build {
    cycle(key, values.to_vec(), translated, initial)
}

/// A slider of an option.
pub fn slider<S>(
    key: &'static str,
    values: S,
    text: fn(&Component, &S::Value) -> Component,
    initial: S::Value,
) -> Build
where
    S: SliderValues + 'static,
    S::Value: 'static,
{
    Box::new(move |lang, rect| {
        Box::new(OptionSlider::new(
            lang,
            rect,
            values,
            Component::translatable(key),
            text,
            initial,
        ))
    })
}

/// The lines of a tooltip that says one thing whatever the value
/// (`cachedConstantTooltip`).
pub fn constant_tip<V>(key: &'static str) -> Tip<V> {
    Box::new(move |_| vec![Some(Component::translatable(key))])
}

/// A cycle button of an option with a tooltip.
pub fn cycle_tip<V: Clone + PartialEq + 'static>(
    key: &'static str,
    values: Vec<V>,
    text: fn(&V) -> Component,
    initial: V,
    tip: Tip<V>,
) -> Build {
    Box::new(move |lang, rect| {
        Box::new(
            CycleButton::new(
                lang,
                rect,
                Component::translatable(key),
                values,
                text,
                &initial,
            )
            .with_tip(lang, tip),
        )
    })
}

/// A cycle button of on and off with a tooltip that says one thing.
pub fn boolean_tip(key: &'static str, initial: bool, tip: &'static str) -> Build {
    cycle_tip(key, vec![true, false], toggle, initial, constant_tip(tip))
}

/// A cycle button of named values whose tooltip is the value's own
/// (`<value>.tooltip`).
pub fn named_tip(key: &'static str, values: &[Named], initial: Named) -> Build {
    cycle_tip(
        key,
        values.to_vec(),
        translated,
        initial,
        Box::new(|value: &Named| vec![Some(Component::translatable(format!("{value}.tooltip")))]),
    )
}

/// A slider of an option with a tooltip.
pub fn slider_tip<S>(
    key: &'static str,
    values: S,
    text: fn(&Component, &S::Value) -> Component,
    initial: S::Value,
    tip: SliderTip<S::Value>,
) -> Build
where
    S: SliderValues + 'static,
    S::Value: 'static,
{
    Box::new(move |lang, rect| {
        Box::new(
            OptionSlider::new(
                lang,
                rect,
                values,
                Component::translatable(key),
                text,
                initial,
            )
            .with_tip(lang, tip),
        )
    })
}

/// A button of the list that opens another screen (`Button` in an
/// `OptionsList`): a press is its "change".
pub struct ButtonControl {
    widget: Widget,
    focused: bool,
}

impl Control for ButtonControl {
    fn is_mouse_over(&self, arena: &Arena, mouse: (f64, f64)) -> bool {
        self.widget.is_active()
            && self
                .widget
                .contains(arena, (mouse.0 as i32, mouse.1 as i32))
    }

    fn pressed(&mut self, _lang: &Lang, _arena: &Arena, _mouse: (f64, f64), _shift: bool) -> bool {
        crate::ui_sound::play_down_sound();
        true
    }

    fn dragged(&mut self, _lang: &Lang, _arena: &Arena, _x: f64) -> bool {
        false
    }

    fn released(&mut self, _lang: &Lang) {}

    fn set_focused(&mut self, focused: bool, _input: InputType) {
        self.focused = focused;
    }

    fn set_active(&mut self, active: bool) {
        self.widget.active = active;
    }

    fn is_active(&self) -> bool {
        self.widget.is_active()
    }

    fn key_pressed(
        &mut self,
        _lang: &Lang,
        _arena: &Arena,
        key: SliderKey,
        _shift: bool,
    ) -> (bool, bool) {
        let pressed = key == SliderKey::Select && self.widget.is_active();
        if pressed {
            crate::ui_sound::play_down_sound();
        }
        (pressed, pressed)
    }

    fn draw(
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A button with a label that a press of is reported to the screen.
pub fn button(key: &'static str) -> Build {
    Box::new(move |lang, rect| {
        Box::new(ButtonControl {
            widget: Widget {
                rect,
                active: true,
                kind: Kind::Button {
                    label: lang.runs(&Component::translatable(key), &Style::default()),
                },
            },
            focused: false,
        })
    })
}

/// What an input to a settings screen asks for.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SubScreenInput {
    /// "Done", or Escape: back to the screen before.
    pub done: bool,
    /// The tag of the widget whose option changed.
    pub changed: Option<u8>,
    /// The extra footer button, if the screen has one.
    pub help: bool,
}

/// `OptionsSubScreen`: the title, the list and Done.
pub struct SubScreen {
    arena: Arena,
    title: Widget,
    /// A second line in the header, and whether it is shown: once it has
    /// been added it keeps its place.
    warning: Option<(Widget, bool)>,
    done: Widget,
    /// A button before Done in the footer (`addFooter` of the accessibility screen), and
    /// whether it has the focus.
    help: Option<Widget>,
    help_focused: bool,
    pub list: OptionsList,
    /// Whether Done has the focus (the list keeps its own).
    done_focused: bool,
    size: (i32, i32),
    in_world: bool,
}

/// Where the focus goes on the screen: a widget of the list, or Done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    List(Focus),
    Help,
    Done,
}

mod focus;
mod scrolled;

mod sub_screen;
