//! The mouse settings screen (`MouseSettingsScreen`) and the controls screen
//! it is opened from (`ControlsScreen`). The mouse's sensitivities and
//! inversions are the options' and take effect; the rest hold a value on
//! their widget.

use voxel_game::option::{IntRange, MappedIntRange, UnitDouble};
use voxel_game::options::{Options, log_mouse, unlog_mouse};
use voxel_game::text::{Component, Lang};

use crate::gui::cycle::CycleButton;
use crate::gui::font::Font;
use crate::gui::option_text::{generic_value, on_off, sensitivity};
use crate::gui::options_list::{
    Build, OptionsList, SubScreen, SubScreenInput, boolean, boolean_tip, button, constant_tip,
    counted, cycle, slider, slider_tip,
};
use crate::gui::slider::{OptionSlider, SliderKey};
use crate::gui::{GuiAtlas, GuiFrame};

const PLAIN: u8 = 0;
const SENSITIVITY: u8 = 1;
const WHEEL: u8 = 2;
const INVERT_X: u8 = 3;
const INVERT_Y: u8 = 4;
const MOUSE_SETTINGS: u8 = 5;

type Wheel = MappedIntRange<f64, fn(i32) -> f64, fn(&f64) -> i32>;

fn unlog(value: &f64) -> i32 {
    unlog_mouse(*value)
}

/// `mouseWheelSensitivity`'s label: two decimals.
fn wheel_label(caption: &Component, value: &f64) -> Component {
    generic_value(caption, Component::literal(format!("{value:.2}")))
}

/// What the mouse screen's options say.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseChoices {
    pub sensitivity: f64,
    pub wheel_sensitivity: f64,
    pub invert_x: bool,
    pub invert_y: bool,
}

pub struct MouseSettingsScreen {
    screen: SubScreen,
}

impl MouseSettingsScreen {
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        options: &Options,
    ) -> Self {
        let wheel: Wheel = IntRange::new(-200, 100).xmap(
            log_mouse as fn(i32) -> f64,
            unlog as fn(&f64) -> i32,
            false,
        );
        let screen = SubScreen::new(
            font,
            lang,
            size,
            "options.mouse_settings.title",
            in_world,
            |list| {
                list.add_small(
                    lang,
                    vec![
                        (
                            SENSITIVITY,
                            slider(
                                "options.sensitivity",
                                UnitDouble,
                                sensitivity,
                                options.sensitivity,
                            ),
                        ),
                        (
                            WHEEL,
                            slider(
                                "options.mouseWheelSensitivity",
                                wheel,
                                wheel_label,
                                options.mouse_wheel_sensitivity,
                            ),
                        ),
                        (PLAIN, boolean("options.discrete_mouse_scroll", false)),
                        (
                            INVERT_X,
                            boolean("options.invertMouseX", options.invert_mouse_x),
                        ),
                        (
                            INVERT_Y,
                            boolean("options.invertMouseY", options.invert_mouse_y),
                        ),
                        (
                            PLAIN,
                            boolean_tip(
                                "options.allowCursorChanges",
                                true,
                                "options.allowCursorChanges.tooltip",
                            ),
                        ),
                    ],
                );
            },
        );
        Self { screen }
    }

    /// What the options say now.
    pub fn choices(&self) -> MouseChoices {
        let list = &self.screen.list;
        let unit = list
            .find(SENSITIVITY)
            .and_then(|control| control.as_any().downcast_ref::<OptionSlider<UnitDouble>>())
            .map_or(0.0, |slider| *slider.value());
        let wheel_sensitivity = list
            .find(WHEEL)
            .and_then(|control| control.as_any().downcast_ref::<OptionSlider<Wheel>>())
            .map_or(1.0, |slider| *slider.value());
        let flag = |tag: u8| {
            list.find(tag)
                .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
                .is_some_and(|button| *button.value())
        };
        MouseChoices {
            sensitivity: unit,
            wheel_sensitivity,
            invert_x: flag(INVERT_X),
            invert_y: flag(INVERT_Y),
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.screen.size()
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.screen.resize(size);
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        self.screen.draw(frame, atlas, font, mouse);
    }

    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> SubScreenInput {
        self.screen.clicked(lang, mouse, shift)
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) {
        self.screen.dragged(lang, mouse, dy);
    }

    /// Tab (and Shift+Tab).
    pub fn tab(&mut self, forward: bool) {
        self.screen.tab(forward);
    }

    /// An arrow, Enter or Space.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> SubScreenInput {
        self.screen.key_pressed(lang, key, shift)
    }

    pub fn released(&mut self, lang: &Lang) {
        self.screen.released(lang);
    }

    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) {
        self.screen.list.scrolled(mouse, scroll_y);
    }

    pub fn scroll_amount(&self) -> f64 {
        self.screen.list.scroll_amount()
    }
}

/// `KEY_TOGGLE` and `KEY_HOLD`.
fn hold_or_toggle(value: &bool) -> Component {
    Component::translatable(if *value {
        "options.key.toggle"
    } else {
        "options.key.hold"
    })
}

/// `sprintWindow`'s label.
fn sprint_window(caption: &Component, value: &i32) -> Component {
    if *value == 0 {
        generic_value(caption, on_off(false))
    } else {
        generic_value(caption, counted("options.value", *value))
    }
}

/// What an input to the controls screen asks for.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ControlsInput {
    pub done: bool,
    /// "Mouse Settings..." was pressed.
    pub mouse_settings: bool,
}

/// `ControlsScreen`: the mouse settings and the key binds, then how each
/// key is held. Only the mouse settings open anything.
pub struct ControlsScreen {
    screen: SubScreen,
}

impl ControlsScreen {
    pub fn new(font: &Font, lang: &Lang, size: (i32, i32), in_world: bool) -> Self {
        let toggle = |key: &'static str| -> (u8, Build) {
            (PLAIN, cycle(key, vec![true, false], hold_or_toggle, false))
        };
        let screen = SubScreen::new(
            font,
            lang,
            size,
            "controls.title",
            in_world,
            |list: &mut OptionsList| {
                list.add_small(
                    lang,
                    vec![
                        (MOUSE_SETTINGS, button("options.mouse_settings")),
                        (PLAIN, button("controls.keybinds")),
                    ],
                );
                list.add_small(
                    lang,
                    vec![
                        toggle("key.sneak"),
                        toggle("key.sprint"),
                        toggle("key.attack"),
                        toggle("key.use"),
                        (PLAIN, boolean("options.autoJump", false)),
                        (
                            PLAIN,
                            slider_tip(
                                "options.sprintWindow",
                                IntRange::new(0, 10),
                                sprint_window,
                                7,
                                constant_tip("options.sprintWindow.tooltip"),
                            ),
                        ),
                        (PLAIN, boolean("options.operatorItemsTab", false)),
                    ],
                );
            },
        );
        Self { screen }
    }

    pub fn size(&self) -> (i32, i32) {
        self.screen.size()
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.screen.resize(size);
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        self.screen.draw(frame, atlas, font, mouse);
    }

    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> ControlsInput {
        let input = self.screen.clicked(lang, mouse, shift);
        ControlsInput {
            done: input.done,
            mouse_settings: input.changed == Some(MOUSE_SETTINGS),
        }
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) {
        self.screen.dragged(lang, mouse, dy);
    }

    /// Tab (and Shift+Tab).
    pub fn tab(&mut self, forward: bool) {
        self.screen.tab(forward);
    }

    /// An arrow, Enter or Space.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> ControlsInput {
        let input = self.screen.key_pressed(lang, key, shift);
        ControlsInput {
            done: input.done,
            mouse_settings: input.changed == Some(MOUSE_SETTINGS),
        }
    }

    pub fn released(&mut self, lang: &Lang) {
        self.screen.released(lang);
    }

    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) {
        self.screen.list.scrolled(mouse, scroll_y);
    }

    pub fn scroll_amount(&self) -> f64 {
        self.screen.list.scroll_amount()
    }
}
