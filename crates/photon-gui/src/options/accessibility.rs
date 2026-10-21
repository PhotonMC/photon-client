//! The accessibility settings screen (`AccessibilityOptionsScreen`): the options that make the
//! game easier to take in, in a list with the Controls link beside the narrator, and a link to the
//! guide beside Done. The options the game keeps (subtitles, the chat's text background and
//! opacities, the delay, the notification time, the screen, field of view, darkness and damage
//! effects, view bobbing, the glint and lightning flashes) are the options'; the others (the
//! menu blurriness, the Mojang background colour, the resource pack contrast, the minecart
//! rotation) hold a value on their widget.

use photon_game::options::Options;
use photon_game::options::values::{IntRange, UnitDouble};
use photon_game::text::{Component, Lang};

use crate::options::chat::{DelayValues, delay, delay_values, narrator_unavailable, percent, unit};
use crate::options::list::{
    Build, SubScreen, SubScreenInput, boolean, boolean_tip, button, constant_tip, cycle, slider,
    slider_tip,
};
use crate::options::text::{chat_opacity, generic_value, percent_value_or_off};
use crate::options::video::blurriness;
use crate::widgets::cycle::CycleButton;
use crate::widgets::slider::{OptionSlider, SliderKey};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// What the screen finds its widgets by.
const PLAIN: u8 = 0;
const NARRATOR: u8 = 13;
const CONTROLS: u8 = 1;
const SUBTITLES: u8 = 2;
const HIGH_CONTRAST: u8 = 3;
const TEXT_BACKGROUND_OPACITY: u8 = 4;
const BACKGROUND_FOR_CHAT_ONLY: u8 = 5;
const CHAT_OPACITY: u8 = 6;
const LINE_SPACING: u8 = 7;
const CHAT_DELAY: u8 = 8;
const NOTIFICATION_TIME: u8 = 9;
const SCREEN_EFFECTS: u8 = 10;
const LIGHTNING: u8 = 11;
const ROTATE_WITH_MINECART: u8 = 12;
const VIEW_BOBBING: u8 = 14;
const FOV_EFFECTS: u8 = 15;
const DARKNESS_EFFECTS: u8 = 16;
const DAMAGE_TILT: u8 = 17;
const GLINT_SPEED: u8 = 18;
const GLINT_STRENGTH: u8 = 19;
const MENU_BLURRINESS: u8 = 20;

/// `notificationDisplayTime`'s value set: tenths of a second from 0.5 to 10.
type TimeValues =
    photon_game::options::values::MappedIntRange<f64, fn(i32) -> f64, fn(&f64) -> i32>;

fn time_values() -> TimeValues {
    IntRange::new(5, 100).xmap(
        (|tenths| f64::from(tenths) / 10.0) as fn(i32) -> f64,
        (|seconds: &f64| (*seconds * 10.0) as i32) as fn(&f64) -> i32,
        true,
    )
}

/// `notificationDisplayTime`'s label, the value being a multiplier.
fn multiplier(caption: &Component, value: &f64) -> Component {
    generic_value(
        caption,
        Component::translatable_with(
            "options.multiplier",
            vec![Component::literal(format!("{value:?}"))],
        ),
    )
}

/// `backgroundForChatOnly`'s value.
fn text_background(value: &bool) -> Component {
    Component::translatable(if *value {
        "options.accessibility.text_background.chat"
    } else {
        "options.accessibility.text_background.everywhere"
    })
}

/// `genericValueLabel` of a share that reads Off at nothing.
fn percent_or_off(caption: &Component, value: &f64) -> Component {
    percent_value_or_off(caption, value)
}

/// What a value set of the square of a share reads as (`UnitDouble.xmap(Mth::square, Math::sqrt)`).
type Squared = photon_game::options::values::MappedUnitDouble<f64, fn(f64) -> f64, fn(&f64) -> f64>;

fn squared() -> Squared {
    UnitDouble::xmap(
        (|share| share * share) as fn(f64) -> f64,
        (|value: &f64| value.sqrt()) as fn(&f64) -> f64,
    )
}

fn unit_tip(key: &'static str, initial: f64, tip: &'static str) -> Build {
    slider_tip(key, UnitDouble, percent_or_off, initial, constant_tip(tip))
}

/// What the options the screen holds say, for the options to take.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccessibilityChoices {
    pub subtitles: bool,
    pub text_background_opacity: f64,
    pub background_for_chat_only: bool,
    pub chat_opacity: f64,
    pub chat_line_spacing: f64,
    /// `chatDelay`, in seconds.
    pub chat_delay: f64,
    pub notification_display_time: f64,
    pub screen_effect_scale: f64,
    pub hide_lightning_flashes: bool,
    pub view_bobbing: bool,
    pub fov_effect_scale: f64,
    pub darkness_effect_scale: f64,
    pub damage_tilt_strength: f64,
    pub glint_speed: f64,
    pub glint_strength: f64,
    pub menu_blurriness: i32,
}

/// What an input to the screen asks for.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AccessibilityInput {
    pub done: bool,
    /// "Controls..." was pressed.
    pub controls: bool,
    /// "Accessibility Guide" was pressed.
    pub guide: bool,
}

pub struct AccessibilityScreen {
    screen: SubScreen,
}

impl AccessibilityScreen {
    /// `init`, with the options as they are; `minecart_improvements` says the level has the
    /// feature `rotateWithMinecart` needs.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        (options, minecart_improvements): (&Options, bool),
    ) -> Self {
        let mut screen = SubScreen::new(
            font,
            lang,
            size,
            "options.accessibility.title",
            in_world,
            |list| list.add_small(lang, rows(options)),
        );
        screen.add_help_button(lang, "options.accessibility.link");
        let mut accessibility = Self { screen };
        // `narratorButton.active = narrator.isActive()`: there is none.
        if let Some(control) = accessibility.screen.list.find_mut(NARRATOR) {
            control.set_active(false);
        }
        // `highContrast.active = false` with the error tooltip: there is no resource pack.
        if let Some(control) = accessibility.screen.list.find_mut(HIGH_CONTRAST) {
            control.set_active(false);
        }
        if let Some(control) = accessibility.screen.list.find_mut(ROTATE_WITH_MINECART) {
            control.set_active(in_world && minecart_improvements);
        }
        accessibility
    }

    /// What the options say now.
    pub fn choices(&self) -> AccessibilityChoices {
        let list = &self.screen.list;
        let share = |tag: u8| {
            list.find(tag)
                .and_then(|control| control.as_any().downcast_ref::<OptionSlider<UnitDouble>>())
                .map_or(0.0, |slider| *slider.value())
        };
        let flag = |tag: u8| {
            list.find(tag)
                .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
                .is_some_and(|button| *button.value())
        };
        let seconds = |tag: u8| {
            list.find(tag)
                .and_then(|control| control.as_any().downcast_ref::<OptionSlider<DelayValues>>())
                .map_or(0.0, |slider| *slider.value())
        };
        let squares = |tag: u8| {
            list.find(tag)
                .and_then(|control| control.as_any().downcast_ref::<OptionSlider<Squared>>())
                .map_or(1.0, |slider| *slider.value())
        };
        let time = list
            .find(NOTIFICATION_TIME)
            .and_then(|control| control.as_any().downcast_ref::<OptionSlider<TimeValues>>())
            .map_or(1.0, |slider| *slider.value());
        AccessibilityChoices {
            subtitles: flag(SUBTITLES),
            text_background_opacity: share(TEXT_BACKGROUND_OPACITY),
            background_for_chat_only: flag(BACKGROUND_FOR_CHAT_ONLY),
            chat_opacity: share(CHAT_OPACITY),
            chat_line_spacing: share(LINE_SPACING),
            chat_delay: seconds(CHAT_DELAY),
            notification_display_time: time,
            screen_effect_scale: share(SCREEN_EFFECTS),
            hide_lightning_flashes: flag(LIGHTNING),
            view_bobbing: flag(VIEW_BOBBING),
            fov_effect_scale: squares(FOV_EFFECTS),
            darkness_effect_scale: squares(DARKNESS_EFFECTS),
            damage_tilt_strength: share(DAMAGE_TILT),
            glint_speed: share(GLINT_SPEED),
            glint_strength: share(GLINT_STRENGTH),
            menu_blurriness: list
                .find(MENU_BLURRINESS)
                .and_then(|control| control.as_any().downcast_ref::<OptionSlider<IntRange>>())
                .map_or(5, |slider| *slider.value()),
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

    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> AccessibilityInput {
        let input = self.screen.clicked(lang, mouse, shift);
        Self::input(input)
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) {
        self.screen.dragged(lang, mouse, dy);
    }

    /// Tab (and Shift+Tab).
    pub fn tab(&mut self, forward: bool) {
        self.screen.tab(forward);
    }

    /// An arrow, Enter or Space.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> AccessibilityInput {
        let input = self.screen.key_pressed(lang, key, shift);
        Self::input(input)
    }

    fn input(input: SubScreenInput) -> AccessibilityInput {
        AccessibilityInput {
            done: input.done,
            controls: input.changed == Some(CONTROLS),
            guide: input.help,
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

/// The widgets of the list, in `addOptions`'s order: the narrator and the Controls link, then the
/// rest two to a row.
fn rows(options: &Options) -> Vec<(u8, Build)> {
    let mut rows = vec![
        (
            NARRATOR,
            cycle(
                "options.narrator",
                vec![
                    "options.narrator.off",
                    "options.narrator.all",
                    "options.narrator.chat",
                    "options.narrator.system",
                ],
                narrator_unavailable,
                "options.narrator.off",
            ),
        ),
        (CONTROLS, button("options.controls")),
    ];
    rows.extend(first_options(options));
    rows.extend(chat_options(options));
    rows.extend(effect_options(options));
    rows.extend(other_options(options));
    rows.extend(last_options());
    rows
}

fn first_options(options: &Options) -> Vec<(u8, Build)> {
    vec![
        (
            SUBTITLES,
            boolean_tip(
                "options.showSubtitles",
                options.show_subtitles,
                "options.showSubtitles.tooltip",
            ),
        ),
        (
            HIGH_CONTRAST,
            boolean_tip(
                "options.accessibility.high_contrast",
                false,
                "options.accessibility.high_contrast.error.tooltip",
            ),
        ),
        (
            MENU_BLURRINESS,
            slider_tip(
                "options.accessibility.menu_background_blurriness",
                IntRange::new(0, 10),
                blurriness,
                options.menu_background_blurriness,
                constant_tip("options.accessibility.menu_background_blurriness.tooltip"),
            ),
        ),
    ]
}

fn chat_options(options: &Options) -> Vec<(u8, Build)> {
    vec![
        (
            TEXT_BACKGROUND_OPACITY,
            unit(
                "options.accessibility.text_background_opacity",
                percent,
                options.text_background_opacity,
            ),
        ),
        (
            BACKGROUND_FOR_CHAT_ONLY,
            cycle(
                "options.accessibility.text_background",
                vec![true, false],
                text_background,
                options.background_for_chat_only,
            ),
        ),
        (
            CHAT_OPACITY,
            unit("options.chat.opacity", chat_opacity, options.chat_opacity),
        ),
        (
            LINE_SPACING,
            unit(
                "options.chat.line_spacing",
                percent,
                options.chat_line_spacing,
            ),
        ),
        (
            CHAT_DELAY,
            slider(
                "options.chat.delay_instant",
                delay_values(),
                delay,
                options.chat_delay,
            ),
        ),
        (
            NOTIFICATION_TIME,
            slider_tip(
                "options.notifications.display_time",
                time_values(),
                multiplier,
                options.notification_display_time,
                constant_tip("options.notifications.display_time.tooltip"),
            ),
        ),
    ]
}

fn effect_options(options: &Options) -> Vec<(u8, Build)> {
    vec![
        (
            VIEW_BOBBING,
            boolean("options.viewBobbing", options.view_bobbing),
        ),
        (
            SCREEN_EFFECTS,
            unit_tip(
                "options.screenEffectScale",
                options.screen_effect_scale,
                "options.screenEffectScale.tooltip",
            ),
        ),
        (
            FOV_EFFECTS,
            slider_tip(
                "options.fovEffectScale",
                squared(),
                percent_or_off,
                options.fov_effect_scale,
                constant_tip("options.fovEffectScale.tooltip"),
            ),
        ),
        (
            DARKNESS_EFFECTS,
            slider_tip(
                "options.darknessEffectScale",
                squared(),
                percent_or_off,
                options.darkness_effect_scale,
                constant_tip("options.darknessEffectScale.tooltip"),
            ),
        ),
        (
            DAMAGE_TILT,
            unit_tip(
                "options.damageTiltStrength",
                options.damage_tilt_strength,
                "options.damageTiltStrength.tooltip",
            ),
        ),
        (
            GLINT_SPEED,
            unit_tip(
                "options.glintSpeed",
                options.glint_speed,
                "options.glintSpeed.tooltip",
            ),
        ),
        (
            GLINT_STRENGTH,
            unit_tip(
                "options.glintStrength",
                options.glint_strength,
                "options.glintStrength.tooltip",
            ),
        ),
    ]
}

fn other_options(options: &Options) -> Vec<(u8, Build)> {
    vec![
        (
            LIGHTNING,
            boolean_tip(
                "options.hideLightningFlashes",
                options.hide_lightning_flashes,
                "options.hideLightningFlashes.tooltip",
            ),
        ),
        (
            PLAIN,
            boolean_tip(
                "options.darkMojangStudiosBackgroundColor",
                false,
                "options.darkMojangStudiosBackgroundColor.tooltip",
            ),
        ),
        (
            PLAIN,
            slider(
                "options.accessibility.panorama_speed",
                UnitDouble,
                percent,
                1.0,
            ),
        ),
        (
            PLAIN,
            boolean_tip(
                "options.hideSplashTexts",
                false,
                "options.hideSplashTexts.tooltip",
            ),
        ),
    ]
}

fn last_options() -> Vec<(u8, Build)> {
    vec![
        (
            PLAIN,
            boolean_tip(
                "options.accessibility.narrator_hotkey",
                true,
                "options.accessibility.narrator_hotkey.tooltip",
            ),
        ),
        (
            ROTATE_WITH_MINECART,
            boolean_tip(
                "options.rotateWithMinecart",
                false,
                "options.rotateWithMinecart.tooltip",
            ),
        ),
        (
            PLAIN,
            boolean_tip(
                "options.accessibility.high_contrast_block_outline",
                false,
                "options.accessibility.high_contrast_block_outline.tooltip",
            ),
        ),
    ]
}

#[cfg(test)]
mod tests;
