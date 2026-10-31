//! The mouse settings screen (`MouseSettingsScreen`) and the controls screen
//! it is opened from (`ControlsScreen`). The mouse's sensitivities and
//! inversions are the options' and take effect; the rest hold a value on
//! their widget.

use photon_game::options::values::{IntRange, MappedIntRange, UnitDouble};
use photon_game::options::{Options, PlayOptions, log_mouse, unlog_mouse};
use photon_game::text::{Component, Lang};

use crate::options::list::{
    Build, OptionsList, SubScreen, SubScreenInput, boolean, boolean_tip, button, constant_tip,
    counted, cycle, slider, slider_tip,
};
use crate::options::text::{generic_value, on_off, sensitivity};
use crate::widgets::cycle::CycleButton;
use crate::widgets::slider::{OptionSlider, SliderKey};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

const SENSITIVITY: u8 = 1;
const WHEEL: u8 = 2;
const INVERT_X: u8 = 3;
const INVERT_Y: u8 = 4;
const MOUSE_SETTINGS: u8 = 5;
const KEY_BINDS: u8 = 6;
const DISCRETE: u8 = 7;
const CURSOR_CHANGES: u8 = 8;
const TOGGLE_CROUCH: u8 = 9;
const TOGGLE_SPRINT: u8 = 10;
const TOGGLE_ATTACK: u8 = 11;
const TOGGLE_USE: u8 = 12;
const AUTO_JUMP: u8 = 13;
const SPRINT_WINDOW: u8 = 14;
const OPERATOR_ITEMS: u8 = 15;

type Wheel = MappedIntRange<f64, fn(i32) -> f64, fn(&f64) -> i32>;

fn unlog(value: &f64) -> i32 {
    unlog_mouse(*value)
}

/// `mouseWheelSensitivity`'s label: two decimals.
fn wheel_label(caption: &Component, value: &f64) -> Component {
    generic_value(caption, Component::literal(format!("{value:.2}")))
}

/// The mouse screen's flags: the wheel, the inversions and the cursor.
fn flag_entries(options: &Options) -> Vec<(u8, Build)> {
    vec![
        (
            DISCRETE,
            boolean(
                "options.discrete_mouse_scroll",
                options.play.discrete_mouse_scroll,
            ),
        ),
        (
            INVERT_X,
            boolean("options.invertMouseX", options.invert_mouse_x),
        ),
        (
            INVERT_Y,
            boolean("options.invertMouseY", options.invert_mouse_y),
        ),
        (
            CURSOR_CHANGES,
            boolean_tip(
                "options.allowCursorChanges",
                options.play.allow_cursor_changes,
                "options.allowCursorChanges.tooltip",
            ),
        ),
    ]
}

/// What the mouse screen's options say.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseChoices {
    pub sensitivity: f64,
    pub wheel_sensitivity: f64,
    pub invert_x: bool,
    pub invert_y: bool,
    pub discrete_scroll: bool,
    pub cursor_changes: bool,
}

pub struct MouseSettingsScreen {
    screen: SubScreen,
}

impl MouseSettingsScreen {
    /// The screen with the cursor option still: no cursor shapes are set here.
    #[must_use]
    pub fn without_missing_effects(mut self, lang: &Lang) -> Self {
        let reason = Component::translatable_or(
            "photon.mouse.no_cursor_changes",
            "This client never changes the cursor's shape.",
        );
        self.screen.list.still(CURSOR_CHANGES, lang, &reason);
        self
    }

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
                let mut entries = vec![
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
                ];
                entries.extend(flag_entries(options));
                list.add_small(lang, entries);
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
            discrete_scroll: flag(DISCRETE),
            cursor_changes: flag(CURSOR_CHANGES),
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
    /// "Key Binds..." was pressed.
    pub key_binds: bool,
}

/// `ControlsScreen`: the mouse settings and the key binds, then how each
/// key is held.
pub struct ControlsScreen {
    screen: SubScreen,
}

impl ControlsScreen {
    /// The screen with the operator items tab still: the creative inventory has no such tab.
    #[must_use]
    pub fn without_missing_effects(mut self, lang: &Lang) -> Self {
        let reason = Component::translatable_or(
            "photon.controls.no_operator_tab",
            "The creative inventory has no operator items tab yet.",
        );
        self.screen.list.still(OPERATOR_ITEMS, lang, &reason);
        self
    }

    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        options: &Options,
    ) -> Self {
        let play = options.play;
        let toggle = |tag: u8, key: &'static str, value: bool| -> (u8, Build) {
            (tag, cycle(key, vec![true, false], hold_or_toggle, value))
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
                        (KEY_BINDS, button("controls.keybinds")),
                    ],
                );
                list.add_small(
                    lang,
                    vec![
                        toggle(TOGGLE_CROUCH, "key.sneak", play.toggle_crouch),
                        toggle(TOGGLE_SPRINT, "key.sprint", play.toggle_sprint),
                        toggle(TOGGLE_ATTACK, "key.attack", play.toggle_attack),
                        toggle(TOGGLE_USE, "key.use", play.toggle_use),
                        (AUTO_JUMP, boolean("options.autoJump", play.auto_jump)),
                        (
                            SPRINT_WINDOW,
                            slider_tip(
                                "options.sprintWindow",
                                IntRange::new(0, 10),
                                sprint_window,
                                play.sprint_window,
                                constant_tip("options.sprintWindow.tooltip"),
                            ),
                        ),
                        (
                            OPERATOR_ITEMS,
                            boolean("options.operatorItemsTab", play.operator_items_tab),
                        ),
                    ],
                );
            },
        );
        Self { screen }
    }

    /// What the play options on the screen say now, over `play`.
    pub fn choices(&self, play: PlayOptions) -> PlayOptions {
        let list = &self.screen.list;
        let flag = |tag: u8, own: bool| {
            list.find(tag)
                .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
                .map_or(own, |button| *button.value())
        };
        let window = list
            .find(SPRINT_WINDOW)
            .and_then(|control| control.as_any().downcast_ref::<OptionSlider<IntRange>>())
            .map_or(play.sprint_window, |slider| *slider.value());
        PlayOptions {
            toggle_crouch: flag(TOGGLE_CROUCH, play.toggle_crouch),
            toggle_sprint: flag(TOGGLE_SPRINT, play.toggle_sprint),
            toggle_attack: flag(TOGGLE_ATTACK, play.toggle_attack),
            toggle_use: flag(TOGGLE_USE, play.toggle_use),
            auto_jump: flag(AUTO_JUMP, play.auto_jump),
            sprint_window: window,
            operator_items_tab: flag(OPERATOR_ITEMS, play.operator_items_tab),
            ..play
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

    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> ControlsInput {
        let input = self.screen.clicked(lang, mouse, shift);
        ControlsInput {
            done: input.done,
            mouse_settings: input.changed == Some(MOUSE_SETTINGS),
            key_binds: input.changed == Some(KEY_BINDS),
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
            key_binds: input.changed == Some(KEY_BINDS),
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
