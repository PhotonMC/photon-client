//! The chat settings screen (`ChatOptionsScreen`): the options of the chat in
//! a list. The visibility and the sizes, spacing and opacities of the chat
//! are the options' and take effect; the rest hold a value on their widget.

use photon_game::options::values::{IntRange, UnitDouble};
use photon_game::options::{ChatVisibility, Options};
use photon_game::text::{Component, Lang};

use crate::chat::{ChatSettings, height_of, width_of};
use crate::options::list::{
    Build, Named, SubScreen, SubScreenInput, boolean, boolean_tip, cycle, off_composed, slider,
};
use crate::options::text::{chat_opacity, chat_visibility, percent_value};
use crate::widgets::cycle::CycleButton;
use crate::widgets::slider::{OptionSlider, SliderKey};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// What the screen finds its widgets by.
const PLAIN: u8 = 0;
const VISIBILITY: u8 = 1;
const OPACITY: u8 = 2;
const BACKGROUND_OPACITY: u8 = 3;
const SCALE: u8 = 4;
const LINE_SPACING: u8 = 5;
const WIDTH: u8 = 6;
const HEIGHT_FOCUSED: u8 = 7;
const HEIGHT_UNFOCUSED: u8 = 8;
const NARRATOR: u8 = 9;
const SAVE_DRAFTS: u8 = 10;

fn percent(caption: &Component, value: &f64) -> Component {
    percent_value(caption, *value)
}

/// `chatScale`'s label: nothing at all reads Off.
fn scale(caption: &Component, value: &f64) -> Component {
    if *value == 0.0 {
        off_composed(caption)
    } else {
        percent_value(caption, *value)
    }
}

/// `Options.pixelValueLabel`.
fn pixels(caption: &Component, value: i32) -> Component {
    Component::translatable_with(
        "options.pixel_value",
        vec![caption.clone(), Component::literal(value.to_string())],
    )
}

fn width(caption: &Component, value: &f64) -> Component {
    pixels(caption, width_of(*value))
}

fn height(caption: &Component, value: &f64) -> Component {
    pixels(caption, height_of(*value))
}

/// `chatDelay`'s label, which does not name the option when there is none.
fn delay(_caption: &Component, value: &f64) -> Component {
    if *value <= 0.0 {
        Component::translatable("options.chat.delay_none")
    } else {
        Component::translatable_with(
            "options.chat.delay",
            vec![Component::literal(format!("{value:.1}"))],
        )
    }
}

/// The narrator's value, there being no narrator.
fn narrator_unavailable(_value: &Named) -> Component {
    Component::translatable("options.narrator.notavailable")
}

fn unit(key: &'static str, label: fn(&Component, &f64) -> Component, initial: f64) -> Build {
    slider(key, UnitDouble, label, initial)
}

/// What the screen's options say, for the options and the chat to take.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChatChoices {
    pub visibility: ChatVisibility,
    pub settings: ChatSettings,
    pub save_drafts: bool,
}

pub struct ChatOptionsScreen {
    screen: SubScreen,
}

impl ChatOptionsScreen {
    /// `init`, with the options as they are.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        options: &Options,
    ) -> Self {
        let screen = SubScreen::new(font, lang, size, "options.chat.title", in_world, |list| {
            list.add_small(
                lang,
                vec![
                    (
                        VISIBILITY,
                        cycle(
                            "options.chat.visibility",
                            vec![
                                ChatVisibility::Full,
                                ChatVisibility::System,
                                ChatVisibility::Hidden,
                            ],
                            chat_visibility,
                            options.chat_visibility,
                        ),
                    ),
                    (PLAIN, boolean("options.chat.color", true)),
                    (PLAIN, boolean("options.chat.links", true)),
                    (PLAIN, boolean("options.chat.links.prompt", true)),
                    (
                        OPACITY,
                        unit("options.chat.opacity", chat_opacity, options.chat_opacity),
                    ),
                    (
                        BACKGROUND_OPACITY,
                        unit(
                            "options.accessibility.text_background_opacity",
                            percent,
                            options.text_background_opacity,
                        ),
                    ),
                    (SCALE, unit("options.chat.scale", scale, options.chat_scale)),
                    (
                        LINE_SPACING,
                        unit(
                            "options.chat.line_spacing",
                            percent,
                            options.chat_line_spacing,
                        ),
                    ),
                    (
                        PLAIN,
                        slider(
                            "options.chat.delay_instant",
                            IntRange::new(0, 60).xmap(
                                |value| f64::from(value) / 10.0,
                                |value: &f64| (*value * 10.0) as i32,
                                true,
                            ),
                            delay,
                            0.0,
                        ),
                    ),
                    (WIDTH, unit("options.chat.width", width, options.chat_width)),
                    (
                        HEIGHT_FOCUSED,
                        unit(
                            "options.chat.height.focused",
                            height,
                            options.chat_height_focused,
                        ),
                    ),
                    (
                        HEIGHT_UNFOCUSED,
                        unit(
                            "options.chat.height.unfocused",
                            height,
                            options.chat_height_unfocused,
                        ),
                    ),
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
                    (PLAIN, boolean("options.autoSuggestCommands", true)),
                    (
                        PLAIN,
                        boolean_tip(
                            "options.hideMatchedNames",
                            true,
                            "options.hideMatchedNames.tooltip",
                        ),
                    ),
                    (PLAIN, boolean("options.reducedDebugInfo", false)),
                    (
                        PLAIN,
                        boolean_tip(
                            "options.onlyShowSecureChat",
                            false,
                            "options.onlyShowSecureChat.tooltip",
                        ),
                    ),
                    (
                        SAVE_DRAFTS,
                        boolean_tip(
                            "options.chat.drafts",
                            options.save_chat_drafts,
                            "options.chat.drafts.tooltip",
                        ),
                    ),
                ],
            );
        });
        let mut chat = Self { screen };
        // `narratorButton.active = narrator.isActive()`: there is none.
        if let Some(control) = chat.screen.list.find_mut(NARRATOR) {
            control.set_active(false);
        }
        chat
    }

    /// What the options say now.
    pub fn choices(&self) -> ChatChoices {
        let list = &self.screen.list;
        let share = |tag: u8| {
            list.find(tag)
                .and_then(|control| control.as_any().downcast_ref::<OptionSlider<UnitDouble>>())
                .map_or(0.0, |slider| *slider.value())
        };
        let visibility = list
            .find(VISIBILITY)
            .and_then(|control| {
                control
                    .as_any()
                    .downcast_ref::<CycleButton<ChatVisibility>>()
            })
            .map_or(ChatVisibility::Full, |button| *button.value());
        let save_drafts = list
            .find(SAVE_DRAFTS)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
            .is_some_and(|button| *button.value());
        ChatChoices {
            visibility,
            save_drafts,
            settings: ChatSettings {
                width: share(WIDTH),
                scale: share(SCALE),
                opacity: share(OPACITY),
                background_opacity: share(BACKGROUND_OPACITY),
                line_spacing: share(LINE_SPACING),
                height_focused: share(HEIGHT_FOCUSED),
                height_unfocused: share(HEIGHT_UNFOCUSED),
            },
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
