//! The sound settings screen (`SoundOptionsScreen`): a slider per sound
//! category, the device, the subtitles, the directional audio and the music.
//! The volumes, the subtitles, the directional audio and the music are the
//! options'; the device only offers the default one.

use photon_game::options::values::UnitDouble;
use photon_game::options::{MusicFrequency, MusicToast, Options};
use photon_game::sounds::SoundSource;
use photon_game::text::{Component, Lang};

use crate::gui::font::Font;
use crate::gui::options::list::{
    Build, SubScreen, SubScreenInput, boolean_tip, constant_tip, cycle, cycle_tip, slider,
};
use crate::gui::options::text::percent_value_or_off;
use crate::gui::widgets::cycle::CycleButton;
use crate::gui::widgets::slider::{OptionSlider, SliderKey};
use crate::gui::{GuiAtlas, GuiFrame};

const PLAIN: u8 = 0;
const SUBTITLES: u8 = 20;
const DIRECTIONAL: u8 = 21;
const FREQUENCY: u8 = 22;
const TOAST: u8 = 23;

/// A sound category's slider caption, in `SoundSource` order.
const CAPTIONS: [&str; 11] = [
    "soundCategory.master",
    "soundCategory.music",
    "soundCategory.record",
    "soundCategory.weather",
    "soundCategory.block",
    "soundCategory.hostile",
    "soundCategory.neutral",
    "soundCategory.player",
    "soundCategory.ambient",
    "soundCategory.voice",
    "soundCategory.ui",
];

/// A category's slider is tagged with its ordinal plus one.
const fn volume_tag(source: SoundSource) -> u8 {
    source as u8 + 1
}

/// What the sound screen's options say.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundChoices {
    pub volumes: [f64; 11],
    pub show_subtitles: bool,
    pub directional_audio: bool,
    pub music_frequency: MusicFrequency,
    pub music_toast: MusicToast,
}

pub struct SoundOptionsScreen {
    screen: SubScreen,
}

#[expect(
    clippy::ptr_arg,
    reason = "a cycle button's text takes `&V` of its value type"
)]
fn device_text(device: &String) -> Component {
    if device.is_empty() {
        Component::translatable("options.audioDevice.default")
    } else {
        Component::literal(device.clone())
    }
}

fn frequency_text(value: &MusicFrequency) -> Component {
    Component::translatable(value.caption_key())
}

fn toast_text(value: &MusicToast) -> Component {
    Component::translatable(value.text_key())
}

fn volume(source: SoundSource, options: &Options) -> (u8, Build) {
    (
        volume_tag(source),
        slider(
            CAPTIONS[source as usize],
            UnitDouble,
            percent_value_or_off,
            options.sound_category_volume(source),
        ),
    )
}

impl SoundOptionsScreen {
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        options: &Options,
    ) -> Self {
        let screen = SubScreen::new(font, lang, size, "options.sounds.title", in_world, |list| {
            let (tag, build) = volume(SoundSource::Master, options);
            list.add_big(lang, tag, build);
            list.add_small(
                lang,
                SoundSource::ALL
                    .into_iter()
                    .filter(|source| *source != SoundSource::Master)
                    .map(|source| volume(source, options))
                    .collect(),
            );
            // Only the default device is offered; choosing another is not built.
            list.add_big(
                lang,
                PLAIN,
                cycle(
                    "options.audioDevice",
                    vec![options.sound_device.clone()],
                    device_text,
                    options.sound_device.clone(),
                ),
            );
            list.add_small(
                lang,
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
                        DIRECTIONAL,
                        cycle_tip(
                            "options.directionalAudio",
                            vec![true, false],
                            crate::gui::options::list::toggle,
                            options.directional_audio,
                            Box::new(|on: &bool| {
                                vec![Some(Component::translatable(if *on {
                                    "options.directionalAudio.on.tooltip"
                                } else {
                                    "options.directionalAudio.off.tooltip"
                                }))]
                            }),
                        ),
                    ),
                ],
            );
            list.add_small(
                lang,
                vec![
                    (
                        FREQUENCY,
                        cycle_tip(
                            "options.music_frequency",
                            MusicFrequency::ALL.to_vec(),
                            frequency_text,
                            options.music_frequency,
                            constant_tip("options.music_frequency.tooltip"),
                        ),
                    ),
                    (
                        TOAST,
                        cycle_tip(
                            "options.musicToast",
                            MusicToast::ALL.to_vec(),
                            toast_text,
                            options.music_toast,
                            Box::new(|value: &MusicToast| {
                                vec![Some(Component::translatable(format!(
                                    "{}.tooltip",
                                    value.text_key()
                                )))]
                            }),
                        ),
                    ),
                ],
            );
        });
        Self { screen }
    }

    /// What the options say now.
    pub fn choices(&self) -> SoundChoices {
        let list = &self.screen.list;
        let mut volumes = [1.0; 11];
        for source in SoundSource::ALL {
            volumes[source as usize] = list
                .find(volume_tag(source))
                .and_then(|control| control.as_any().downcast_ref::<OptionSlider<UnitDouble>>())
                .map_or(1.0, |slider| *slider.value());
        }
        let cycled = |tag: u8| list.find(tag).map(|control| control.as_any());
        let flag = |tag: u8| {
            cycled(tag)
                .and_then(|any| any.downcast_ref::<CycleButton<bool>>())
                .is_some_and(|button| *button.value())
        };
        SoundChoices {
            volumes,
            show_subtitles: flag(SUBTITLES),
            directional_audio: flag(DIRECTIONAL),
            music_frequency: cycled(FREQUENCY)
                .and_then(|any| any.downcast_ref::<CycleButton<MusicFrequency>>())
                .map_or(MusicFrequency::Default, |button| *button.value()),
            music_toast: cycled(TOAST)
                .and_then(|any| any.downcast_ref::<CycleButton<MusicToast>>())
                .map_or(MusicToast::Never, |button| *button.value()),
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
