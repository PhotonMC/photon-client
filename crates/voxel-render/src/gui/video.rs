//! The video settings screen (`VideoSettingsScreen`): the display, quality
//! and preference options in a scrolling list. Only the GUI scale does
//! anything; the others hold a value on their widget and nothing follows.

use voxel_game::option::{IntRange, SliderableEnum};
use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::cycle::CycleButton;
use crate::gui::font::Font;
use crate::gui::option_text::{generic_int, generic_value, on_off, percent_value};
use crate::gui::options_list::{Build, SubScreen, SubScreenInput};
use crate::gui::slider::{OptionSlider, SliderValues};
use crate::gui::{GuiAtlas, GuiFrame};

/// What the screen finds its widgets by; an option with `CUSTOMISING` in its
/// tag turns the graphics preset to Custom when it changes
/// (`setGraphicsPresetToCustom`).
const PLAIN: u8 = 0;
const GUI_SCALE: u8 = 1;
const GRAPHICS_PRESET: u8 = 2;
const CUSTOMISING: u8 = 0x80;
const TEXTURE_FILTERING: u8 = CUSTOMISING | 1;
const MAX_ANISOTROPY: u8 = CUSTOMISING | 2;
const EXCLUSIVE_FULLSCREEN: u8 = 3;
const GRAPHICS_API: u8 = 4;
/// `RESTART_REQUIRED`'s colour, `withColor(-2142128)`.
const RESTART_COLOUR: u32 = 0xDF_5050;

/// A cycle button of a few named values: the keys of their names.
type Named = &'static str;

fn translated(key: &Named) -> Component {
    Component::translatable(*key)
}

fn toggle(value: &bool) -> Component {
    on_off(*value)
}

fn gui_scale_text(value: &i32) -> Component {
    if *value == 0 {
        Component::translatable("options.guiScale.auto")
    } else {
        Component::literal(value.to_string())
    }
}

fn counted(key: &str, value: i32) -> Component {
    Component::translatable_with(key, vec![Component::literal(value.to_string())])
}

/// `genericValueLabel(caption, translatable("options.chunks", value))`.
fn chunks(caption: &Component, value: &i32) -> Component {
    generic_value(caption, counted("options.chunks", *value))
}

fn blocks(caption: &Component, value: &i32) -> Component {
    generic_value(caption, counted("options.blocks", *value))
}

/// `CommonComponents.optionStatus(caption, false)`.
fn off_composed(caption: &Component) -> Component {
    Component::translatable_with("options.off.composed", vec![caption.clone()])
}

fn biome_blend(caption: &Component, value: &i32) -> Component {
    generic_value(
        caption,
        Component::translatable(format!("options.biomeBlendRadius.{}", value * 2 + 1)),
    )
}

fn entity_distance(caption: &Component, value: &f64) -> Component {
    percent_value(caption, *value)
}

/// `genericValueOrOffLabel`.
fn blurriness(caption: &Component, value: &i32) -> Component {
    if *value == 0 {
        generic_value(caption, on_off(false))
    } else {
        generic_int(caption, *value)
    }
}

fn mipmaps(caption: &Component, value: &i32) -> Component {
    if *value == 0 {
        off_composed(caption)
    } else {
        generic_int(caption, *value)
    }
}

fn anisotropy(caption: &Component, value: &i32) -> Component {
    if *value == 0 {
        off_composed(caption)
    } else {
        generic_value(caption, counted("options.multiplier", 1 << value))
    }
}

/// `UNLIMITED_FRAMERATE_CUTOFF` is 260.
fn framerate(caption: &Component, value: &i32) -> Component {
    if *value == 260 {
        generic_value(
            caption,
            Component::translatable("options.framerateLimit.max"),
        )
    } else {
        generic_value(caption, counted("options.framerate", *value))
    }
}

fn chunk_fade(_caption: &Component, value: &f64) -> Component {
    if *value <= 0.0 {
        Component::translatable("options.chunkFade.none")
    } else {
        Component::translatable_with(
            "options.chunkFade.seconds",
            vec![Component::literal(format!("{value:.2}"))],
        )
    }
}

fn gamma(caption: &Component, value: &f64) -> Component {
    let shown = (*value * 100.0) as i32;
    match shown {
        0 => generic_value(caption, Component::translatable("options.gamma.min")),
        50 => generic_value(caption, Component::translatable("options.gamma.default")),
        100 => generic_value(caption, Component::translatable("options.gamma.max")),
        _ => generic_int(caption, shown),
    }
}

fn graphics_preset(caption: &Component, value: &Named) -> Component {
    generic_value(caption, Component::translatable(*value))
}

/// The fullscreen resolutions of a monitor, of which there is none.
fn fullscreen_resolution(_caption: &Component, _value: &i32) -> Component {
    Component::translatable("options.fullscreen.unavailable")
}

/// A cycle button of an option.
fn cycle<V: Clone + PartialEq + 'static>(
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

fn boolean(key: &'static str, initial: bool) -> Build {
    cycle(key, vec![true, false], toggle, initial)
}

fn named(key: &'static str, values: &[Named], initial: Named) -> Build {
    cycle(key, values.to_vec(), translated, initial)
}

/// A slider of an option.
fn slider<S>(
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

/// `displayOptions`, with the GUI scale a cycle button of `0..=max`.
fn display_options(gui_scale: i32, max_gui_scale: i32) -> Vec<(u8, Build)> {
    vec![
        (
            PLAIN,
            slider(
                "options.framerateLimit",
                IntRange::new(1, 26).xmap(|value| value * 10, |value: &i32| value / 10, true),
                framerate,
                120,
            ),
        ),
        (PLAIN, boolean("options.vsync", true)),
        (
            PLAIN,
            named(
                "options.inactivityFpsLimit",
                &[
                    "options.inactivityFpsLimit.minimized",
                    "options.inactivityFpsLimit.afk",
                ],
                "options.inactivityFpsLimit.afk",
            ),
        ),
        (
            GUI_SCALE,
            cycle(
                "options.guiScale",
                (0..=max_gui_scale).collect(),
                gui_scale_text,
                gui_scale,
            ),
        ),
        (PLAIN, boolean("options.fullscreen", false)),
        (
            EXCLUSIVE_FULLSCREEN,
            boolean("options.exclusiveFullscreen", false),
        ),
        (
            PLAIN,
            slider("options.gamma", voxel_game::option::UnitDouble, gamma, 0.5),
        ),
        (
            GRAPHICS_API,
            named(
                "options.graphicsApi",
                &[
                    "options.graphicsApi.default",
                    "options.graphicsApi.opengl",
                    "options.graphicsApi.vulkan",
                ],
                "options.graphicsApi.default",
            ),
        ),
    ]
}

/// `qualityOptions`, the first nine.
fn quality_options_first() -> Vec<(u8, Build)> {
    vec![
        (
            CUSTOMISING,
            slider(
                "options.biomeBlendRadius",
                IntRange::with_apply_value_immediately(0, 7, false),
                biome_blend,
                2,
            ),
        ),
        (
            CUSTOMISING,
            slider(
                "options.renderDistance",
                IntRange::with_apply_value_immediately(2, 32, false),
                chunks,
                12,
            ),
        ),
        (
            CUSTOMISING,
            named(
                "options.prioritizeChunkUpdates",
                &[
                    "options.prioritizeChunkUpdates.none",
                    "options.prioritizeChunkUpdates.byPlayer",
                    "options.prioritizeChunkUpdates.nearby",
                ],
                "options.prioritizeChunkUpdates.none",
            ),
        ),
        (
            CUSTOMISING,
            slider(
                "options.simulationDistance",
                IntRange::with_apply_value_immediately(5, 32, false),
                chunks,
                12,
            ),
        ),
        (CUSTOMISING, boolean("options.ao", true)),
        (
            CUSTOMISING,
            named(
                "options.renderClouds",
                &["options.off", "options.clouds.fast", "options.clouds.fancy"],
                "options.clouds.fancy",
            ),
        ),
        (
            CUSTOMISING,
            named(
                "options.particles",
                &[
                    "options.particles.all",
                    "options.particles.decreased",
                    "options.particles.minimal",
                ],
                "options.particles.all",
            ),
        ),
        (
            CUSTOMISING,
            slider("options.mipmapLevels", IntRange::new(0, 4), mipmaps, 4),
        ),
        (CUSTOMISING, boolean("options.entityShadows", true)),
    ]
}

/// `qualityOptions`, the rest.
fn quality_options_rest() -> Vec<(u8, Build)> {
    vec![
        (
            CUSTOMISING,
            slider(
                "options.entityDistanceScaling",
                IntRange::new(2, 20).xmap(
                    |value| f64::from(value) / 4.0,
                    |value: &f64| (*value * 4.0) as i32,
                    true,
                ),
                entity_distance,
                1.0,
            ),
        ),
        (
            CUSTOMISING,
            slider(
                "options.accessibility.menu_background_blurriness",
                IntRange::new(0, 10),
                blurriness,
                5,
            ),
        ),
        (
            CUSTOMISING,
            slider(
                "options.renderCloudsDistance",
                IntRange::with_apply_value_immediately(2, 128, true),
                chunks,
                128,
            ),
        ),
        (CUSTOMISING, boolean("options.cutoutLeaves", true)),
        (CUSTOMISING, boolean("options.improvedTransparency", false)),
        (
            TEXTURE_FILTERING,
            named(
                "options.textureFiltering",
                &[
                    "options.textureFiltering.none",
                    "options.textureFiltering.rgss",
                    "options.textureFiltering.anisotropic",
                ],
                "options.textureFiltering.none",
            ),
        ),
        (
            MAX_ANISOTROPY,
            slider("options.maxAnisotropy", IntRange::new(1, 3), anisotropy, 2),
        ),
        (
            CUSTOMISING,
            slider("options.weatherRadius", IntRange::new(3, 10), blocks, 10),
        ),
    ]
}

/// `qualityOptions`.
fn quality_options() -> Vec<(u8, Build)> {
    let mut options = quality_options_first();
    options.extend(quality_options_rest());
    options
}

/// `preferenceOptions`.
fn preference_options() -> Vec<(u8, Build)> {
    vec![
        (PLAIN, boolean("options.autosaveIndicator", true)),
        (PLAIN, boolean("options.vignette", true)),
        (
            PLAIN,
            named(
                "options.attackIndicator",
                &[
                    "options.off",
                    "options.attack.crosshair",
                    "options.attack.hotbar",
                ],
                "options.attack.crosshair",
            ),
        ),
        (
            PLAIN,
            slider(
                "options.chunkFade",
                IntRange::new(0, 40).xmap(
                    |value| f64::from(value) / 20.0,
                    |value: &f64| (*value * 20.0) as i32,
                    true,
                ),
                chunk_fade,
                0.75,
            ),
        ),
    ]
}

pub struct VideoSettingsScreen {
    screen: SubScreen,
    /// The GUI scale the screen has taken, 0 for the automatic one.
    gui_scale: i32,
    /// The scale the window can show most (`ClampingLazyMaxIntRange`).
    max_gui_scale: i32,
    /// The words that say a restart is needed, and how wide they are.
    restart: (Vec<Run>, i32),
}

impl VideoSettingsScreen {
    /// `init`, with the GUI scale as it is set, and the most the window
    /// allows.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        gui_scale: i32,
        max_gui_scale: i32,
    ) -> Self {
        let screen = SubScreen::new(font, lang, size, "options.videoTitle", in_world, |list| {
            list.add_header(font, lang, "options.video.display.header");
            list.add_big(
                lang,
                PLAIN,
                slider(
                    "options.fullscreen.resolution",
                    IntRange::new(-1, -1),
                    fullscreen_resolution,
                    -1,
                ),
            );
            list.add_small(lang, display_options(gui_scale, max_gui_scale));
            list.add_header(font, lang, "options.video.quality.header");
            list.add_big(
                lang,
                GRAPHICS_PRESET,
                slider(
                    "options.graphics.preset",
                    SliderableEnum::new(vec![
                        "options.graphics.fast",
                        "options.graphics.fancy",
                        "options.graphics.fabulous",
                        "options.graphics.custom",
                    ]),
                    graphics_preset,
                    "options.graphics.fancy",
                ),
            );
            list.add_small(lang, quality_options());
            list.add_header(font, lang, "options.video.preferences.header");
            list.add_small(lang, preference_options());
        });
        let style = Style {
            colour: Some(RESTART_COLOUR),
            ..Style::default()
        };
        let restart = lang.runs(&Component::translatable("options.restartRequired"), &style);
        let restart_width = font.width_of_runs(&restart);
        let mut video = Self {
            screen,
            gui_scale,
            max_gui_scale,
            restart: (restart, restart_width),
        };
        video.tick();
        video
    }

    /// `tick`: the anisotropy slider is for the filtering that uses it, and
    /// the header says when a restart is needed
    /// (`isRestartRequiredToApplyVideoSettings`).
    pub fn tick(&mut self) {
        let list = &self.screen.list;
        let exclusive = list
            .find(EXCLUSIVE_FULLSCREEN)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
            .is_some_and(|button| *button.value());
        let api_changed = list
            .find(GRAPHICS_API)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<Named>>())
            .is_some_and(|button| *button.value() != "options.graphicsApi.default");
        self.screen
            .set_header_line((&self.restart.0, self.restart.1), exclusive || api_changed);
        let anisotropic = self
            .screen
            .list
            .find(TEXTURE_FILTERING)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<Named>>())
            .is_some_and(|button| *button.value() == "options.textureFiltering.anisotropic");
        if let Some(control) = self.screen.list.find_mut(MAX_ANISOTROPY) {
            control.set_active(anisotropic);
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.screen.size()
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.screen.resize(size);
    }

    pub fn scroll_amount(&self) -> f64 {
        self.screen.list.scroll_amount()
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        self.screen.draw(frame, atlas, font, mouse);
    }

    /// `setGraphicsPresetToCustom`: the preset says Custom.
    fn customise(&mut self, lang: &Lang, changed: Option<u8>) {
        if changed.is_none_or(|tag| tag & CUSTOMISING == 0) {
            return;
        }
        if let Some(preset) = self
            .screen
            .list
            .find_mut(GRAPHICS_PRESET)
            .and_then(|control| {
                control
                    .as_any_mut()
                    .downcast_mut::<OptionSlider<SliderableEnum<Named>>>()
            })
        {
            preset.set_current(lang, "options.graphics.custom");
        }
    }

    /// The GUI scale the button says, if it is another than the screen
    /// last took: the window follows, and the screen is laid out again.
    fn follow_gui_scale(&mut self, resize: &mut dyn FnMut(i32) -> (i32, i32)) {
        let shown = self
            .screen
            .list
            .find(GUI_SCALE)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<i32>>())
            .map(|button| *button.value());
        if let Some(shown) = shown
            && shown != self.gui_scale
        {
            self.gui_scale = shown;
            let size = resize(shown);
            self.screen.resize(size);
        }
    }

    /// A press of the primary button. A new GUI scale is given to `resize`,
    /// which answers with the GUI size the window now has.
    pub fn clicked(
        &mut self,
        lang: &Lang,
        mouse: (f64, f64),
        shift: bool,
        resize: &mut dyn FnMut(i32) -> (i32, i32),
    ) -> SubScreenInput {
        let input = self.screen.clicked(lang, mouse, shift);
        self.customise(lang, input.changed);
        self.follow_gui_scale(resize);
        self.tick();
        input
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) {
        let changed = self.screen.dragged(lang, mouse, dy);
        self.customise(lang, changed);
        self.tick();
    }

    pub fn released(&mut self, lang: &Lang) {
        self.screen.released(lang);
        self.tick();
    }

    /// `mouseScrolled`: the wheel scrolls the list; with Control it steps
    /// the GUI scale, from the automatic one down from the largest.
    pub fn scrolled(
        &mut self,
        lang: &Lang,
        mouse: (f64, f64),
        scroll_y: f64,
        control: bool,
        resize: &mut dyn FnMut(i32) -> (i32, i32),
    ) {
        if !control {
            self.screen.list.scrolled(mouse, scroll_y);
            return;
        }
        let old = self.gui_scale;
        let adjusted = if old == 0 {
            self.max_gui_scale + 1
        } else {
            old
        };
        let new = adjusted + scroll_y.signum() as i32 * i32::from(scroll_y != 0.0);
        if new != 0 && new <= self.max_gui_scale && new >= 0 {
            self.gui_scale = new;
            let size = resize(new);
            self.screen.resize(size);
            if let Some(button) = self
                .screen
                .list
                .find_mut(GUI_SCALE)
                .and_then(|control| control.as_any_mut().downcast_mut::<CycleButton<i32>>())
            {
                button.set_value(lang, new);
            }
            self.screen.list.set_scroll_amount(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets() -> Option<(Font, Lang)> {
        let root = std::path::PathBuf::from(std::env::var("VOXEL_ASSETS").ok()?);
        Some((Font::load(&root)?, Lang::load(&root)))
    }

    #[test]
    fn done_leaves_and_a_gui_scale_press_asks_the_window_for_a_size() {
        let Some((font, lang)) = assets() else { return };
        let mut screen = VideoSettingsScreen::new(&font, &lang, (427, 240), true, 2, 2);
        let mut asked = Vec::new();
        let mut resize = |scale: i32| {
            asked.push(scale);
            (854, 480)
        };
        // The GUI scale is the second of the second row; a press takes the
        // next scale, which wraps round to the automatic one.
        let input = screen.clicked(&lang, (250.0, 110.0), false, &mut resize);
        assert!(!input.done);
        assert_eq!(screen.size(), (854, 480));
        // Done, in the middle of the footer.
        let input = screen.clicked(&lang, (427.0, 463.0), false, &mut resize);
        assert!(input.done);
        assert_eq!(asked, [0]);
    }
}
