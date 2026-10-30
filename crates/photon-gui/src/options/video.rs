//! The video settings screen (`VideoSettingsScreen`): the display, quality
//! and preference options in a scrolling list. The GUI scale, the frame limits,
//! vsync, the attack indicator, the render distance and ambient occlusion do something; the others
//! hold a value on their widget and nothing follows.

use photon_game::options::values::{IntRange, MappedIntRange, SliderableEnum};
use photon_game::options::{AttackIndicator, FRAMERATE_LIMIT_DEFAULT, InactivityFpsLimit};
use photon_game::text::{Component, Lang, Run, Style};

use crate::options::list::{
    Build, Named, SubScreen, SubScreenInput, boolean, boolean_tip, constant_tip, counted, cycle,
    cycle_tip, named, named_tip, off_composed, slider, slider_tip,
};
use crate::options::text::{generic_int, generic_value, on_off, percent_value};
use crate::widgets::cycle::CycleButton;
use crate::widgets::slider::{OptionSlider, SliderKey};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

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
const ATTACK_INDICATOR: u8 = 5;
const FRAMERATE_LIMIT: u8 = 6;
const VSYNC: u8 = 7;
const INACTIVITY_FPS_LIMIT: u8 = 8;
const RENDER_DISTANCE: u8 = CUSTOMISING | 3;
const AMBIENT_OCCLUSION: u8 = CUSTOMISING | 4;
const MENU_BLURRINESS: u8 = CUSTOMISING | 5;
const GAMMA: u8 = 9;
const PARTICLES: u8 = CUSTOMISING | 6;
const BIOME_BLEND: u8 = CUSTOMISING | 7;
const PRIORITIZE_CHUNK_UPDATES: u8 = CUSTOMISING | 8;
const SIMULATION_DISTANCE: u8 = CUSTOMISING | 9;
const CLOUDS: u8 = CUSTOMISING | 10;
const MIPMAPS: u8 = CUSTOMISING | 11;
const ENTITY_SHADOWS: u8 = CUSTOMISING | 12;
const ENTITY_DISTANCE: u8 = CUSTOMISING | 13;
const CLOUD_RANGE: u8 = CUSTOMISING | 14;
const CUTOUT_LEAVES: u8 = CUSTOMISING | 15;
const IMPROVED_TRANSPARENCY: u8 = CUSTOMISING | 16;
const WEATHER_RADIUS: u8 = CUSTOMISING | 17;
/// `RESTART_REQUIRED`'s colour, `withColor(-2142128)`.
const RESTART_COLOUR: u32 = 0xDF_5050;

fn gui_scale_text(value: &i32) -> Component {
    if *value == 0 {
        Component::translatable("options.guiScale.auto")
    } else {
        Component::literal(value.to_string())
    }
}

/// `genericValueLabel(caption, translatable("options.chunks", value))`.
fn chunks(caption: &Component, value: &i32) -> Component {
    generic_value(caption, counted("options.chunks", *value))
}

fn blocks(caption: &Component, value: &i32) -> Component {
    generic_value(caption, counted("options.blocks", *value))
}

fn biome_blend(caption: &Component, value: &i32) -> Component {
    generic_value(
        caption,
        Component::translatable(format!("options.biomeBlendRadius.{}", value * 2 + 1)),
    )
}

/// `entityDistanceScaling`'s slider steps: quarters, from a half to five.
type EntityDistanceValues = MappedIntRange<f64, fn(i32) -> f64, fn(&f64) -> i32>;

fn entity_distance_values() -> EntityDistanceValues {
    IntRange::new(2, 20).xmap(
        (|value| f64::from(value) / 4.0) as fn(i32) -> f64,
        (|value: &f64| (*value * 4.0) as i32) as fn(&f64) -> i32,
        true,
    )
}

fn entity_distance(caption: &Component, value: &f64) -> Component {
    percent_value(caption, *value)
}

/// `genericValueOrOffLabel`.
pub(super) fn blurriness(caption: &Component, value: &i32) -> Component {
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

/// The frame rate limit's slider steps: tens, from 10 to 260.
type FramerateValues = MappedIntRange<i32, fn(i32) -> i32, fn(&i32) -> i32>;

fn framerate_values() -> FramerateValues {
    IntRange::new(1, 26).xmap(
        (|value| value * 10) as fn(i32) -> i32,
        (|value: &i32| value / 10) as fn(&i32) -> i32,
        true,
    )
}

/// The options the screen shows and hands back; the default is the game's own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VideoValues {
    pub attack_indicator: AttackIndicator,
    pub framerate_limit: i32,
    pub vsync: bool,
    /// `ao`.
    pub ambient_occlusion: bool,
    pub inactivity_fps_limit: InactivityFpsLimit,
    /// `renderDistance`, in chunks.
    pub render_distance: i32,
    /// `menuBackgroundBlurriness`.
    pub menu_blurriness: i32,
    /// `gamma`, the brightness.
    pub gamma: f64,
    /// `particles`.
    pub particles: photon_game::options::ParticleStatus,
    /// The other options a graphics preset controls, and the preset.
    pub quality: photon_game::options::QualityOptions,
}

impl Default for VideoValues {
    fn default() -> Self {
        Self {
            attack_indicator: AttackIndicator::Crosshair,
            framerate_limit: FRAMERATE_LIMIT_DEFAULT,
            vsync: true,
            ambient_occlusion: true,
            inactivity_fps_limit: InactivityFpsLimit::Afk,
            render_distance: 12,
            menu_blurriness: 5,
            gamma: 0.5,
            particles: photon_game::options::ParticleStatus::All,
            quality: photon_game::options::QualityOptions::default(),
        }
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

fn toggle_text(value: &bool) -> Component {
    crate::options::list::toggle(value)
}

fn translated_value(value: &Named) -> Component {
    crate::options::list::translated(value)
}

/// `displayOptions`, with the GUI scale a cycle button of `0..=max`.
fn display_options(gui_scale: i32, max_gui_scale: i32, display: &VideoValues) -> Vec<(u8, Build)> {
    vec![
        (
            FRAMERATE_LIMIT,
            slider(
                "options.framerateLimit",
                framerate_values(),
                framerate,
                display.framerate_limit,
            ),
        ),
        (VSYNC, boolean("options.vsync", display.vsync)),
        (
            INACTIVITY_FPS_LIMIT,
            named_tip(
                "options.inactivityFpsLimit",
                &[
                    InactivityFpsLimit::Minimized.caption_key(),
                    InactivityFpsLimit::Afk.caption_key(),
                ],
                display.inactivity_fps_limit.caption_key(),
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
            cycle_tip(
                "options.exclusiveFullscreen",
                vec![true, false],
                toggle_text,
                false,
                Box::new(|value: &bool| {
                    // A value other than the one the game started with
                    // needs a restart (`TOOLTIP_NEEDS_RESTART`).
                    let mut lines = Vec::new();
                    if *value {
                        lines.push(Some(Component::translatable("options.needsRestart")));
                        lines.push(None);
                    }
                    lines.push(Some(Component::translatable(if *value {
                        "options.exclusiveFullscreen.on.tooltip"
                    } else {
                        "options.exclusiveFullscreen.off.tooltip"
                    })));
                    lines
                }),
            ),
        ),
        (
            GAMMA,
            slider(
                "options.gamma",
                photon_game::options::values::UnitDouble,
                gamma,
                display.gamma,
            ),
        ),
        (
            GRAPHICS_API,
            cycle_tip(
                "options.graphicsApi",
                vec![
                    "options.graphicsApi.default",
                    "options.graphicsApi.opengl",
                    "options.graphicsApi.vulkan",
                ],
                translated_value,
                "options.graphicsApi.default",
                Box::new(|value: &Named| {
                    let mut lines = Vec::new();
                    if *value != "options.graphicsApi.default" {
                        lines.push(Some(Component::translatable("options.needsRestart")));
                        lines.push(None);
                    }
                    lines.push(Some(Component::translatable("options.graphicsApi.tooltip")));
                    if *value == "options.graphicsApi.vulkan" {
                        lines.push(None);
                        lines.push(Some(Component::translatable(
                            "options.graphicsApi.tooltip.vulkan",
                        )));
                    }
                    lines
                }),
            ),
        ),
    ]
}

/// `ao`, which builds every section again when it changes.
fn ambient_occlusion(display: &VideoValues) -> (u8, Build) {
    let initial = display.ambient_occlusion;
    (AMBIENT_OCCLUSION, boolean("options.ao", initial))
}

/// `qualityOptions`, the distances and smooth lighting.
fn quality_options_first(display: &VideoValues) -> Vec<(u8, Build)> {
    vec![
        (
            BIOME_BLEND,
            slider(
                "options.biomeBlendRadius",
                IntRange::with_apply_value_immediately(0, 7, false),
                biome_blend,
                display.quality.biome_blend_radius,
            ),
        ),
        (
            RENDER_DISTANCE,
            slider(
                "options.renderDistance",
                IntRange::with_apply_value_immediately(2, 32, false),
                chunks,
                display.render_distance,
            ),
        ),
        (
            PRIORITIZE_CHUNK_UPDATES,
            named_tip(
                "options.prioritizeChunkUpdates",
                &[
                    "options.prioritizeChunkUpdates.none",
                    "options.prioritizeChunkUpdates.byPlayer",
                    "options.prioritizeChunkUpdates.nearby",
                ],
                display.quality.prioritize_chunk_updates.caption_key(),
            ),
        ),
        (
            SIMULATION_DISTANCE,
            slider(
                "options.simulationDistance",
                IntRange::with_apply_value_immediately(5, 32, false),
                chunks,
                display.quality.simulation_distance,
            ),
        ),
        ambient_occlusion(display),
    ]
}

/// `qualityOptions`, the clouds, particles, mipmaps and shadows.
fn quality_options_visuals(display: &VideoValues) -> Vec<(u8, Build)> {
    vec![
        (
            CLOUDS,
            named(
                "options.renderClouds",
                &["options.off", "options.clouds.fast", "options.clouds.fancy"],
                display.quality.cloud_status.caption_key(),
            ),
        ),
        (
            PARTICLES,
            named(
                "options.particles",
                &[
                    "options.particles.all",
                    "options.particles.decreased",
                    "options.particles.minimal",
                ],
                display.particles.caption_key(),
            ),
        ),
        (
            MIPMAPS,
            slider(
                "options.mipmapLevels",
                IntRange::new(0, 4),
                mipmaps,
                display.quality.mipmap_levels,
            ),
        ),
        (
            ENTITY_SHADOWS,
            boolean("options.entityShadows", display.quality.entity_shadows),
        ),
    ]
}

/// `qualityOptions`, the rest.
fn quality_options_rest(display: &VideoValues) -> Vec<(u8, Build)> {
    vec![
        (
            ENTITY_DISTANCE,
            slider(
                "options.entityDistanceScaling",
                entity_distance_values(),
                entity_distance,
                display.quality.entity_distance_scaling,
            ),
        ),
        (
            MENU_BLURRINESS,
            slider_tip(
                "options.accessibility.menu_background_blurriness",
                IntRange::new(0, 10),
                blurriness,
                display.menu_blurriness,
                constant_tip("options.accessibility.menu_background_blurriness.tooltip"),
            ),
        ),
        (
            CLOUD_RANGE,
            slider(
                "options.renderCloudsDistance",
                IntRange::with_apply_value_immediately(2, 128, true),
                chunks,
                display.quality.cloud_range,
            ),
        ),
        (
            CUTOUT_LEAVES,
            boolean_tip(
                "options.cutoutLeaves",
                display.quality.cutout_leaves,
                "options.cutoutLeaves.tooltip",
            ),
        ),
        (
            IMPROVED_TRANSPARENCY,
            boolean_tip(
                "options.improvedTransparency",
                display.quality.improved_transparency,
                "options.improvedTransparency.tooltip",
            ),
        ),
    ]
}

/// `qualityOptions`, the filtering and the weather.
fn quality_options_filtering(display: &VideoValues) -> Vec<(u8, Build)> {
    vec![
        (
            TEXTURE_FILTERING,
            named_tip(
                "options.textureFiltering",
                &[
                    "options.textureFiltering.none",
                    "options.textureFiltering.rgss",
                    "options.textureFiltering.anisotropic",
                ],
                display.quality.texture_filtering.caption_key(),
            ),
        ),
        (
            MAX_ANISOTROPY,
            slider_tip(
                "options.maxAnisotropy",
                IntRange::new(1, 3),
                anisotropy,
                display.quality.max_anisotropy_bit,
                constant_tip("options.maxAnisotropy.tooltip"),
            ),
        ),
        (
            WEATHER_RADIUS,
            slider_tip(
                "options.weatherRadius",
                IntRange::new(3, 10),
                blocks,
                display.quality.weather_radius,
                constant_tip("options.weatherRadius.tooltip"),
            ),
        ),
    ]
}

/// `qualityOptions`.
fn quality_options(display: &VideoValues) -> Vec<(u8, Build)> {
    let mut options = quality_options_first(display);
    options.extend(quality_options_visuals(display));
    options.extend(quality_options_rest(display));
    options.extend(quality_options_filtering(display));
    options
}

/// `preferenceOptions`.
fn preference_options(attack_indicator: AttackIndicator) -> Vec<(u8, Build)> {
    vec![
        (PLAIN, boolean("options.autosaveIndicator", true)),
        (
            PLAIN,
            boolean_tip("options.vignette", true, "options.vignette.tooltip"),
        ),
        (
            ATTACK_INDICATOR,
            named(
                "options.attackIndicator",
                &[
                    "options.off",
                    "options.attack.crosshair",
                    "options.attack.hotbar",
                ],
                attack_indicator.caption_key(),
            ),
        ),
        (
            PLAIN,
            slider_tip(
                "options.chunkFade",
                IntRange::new(0, 40).xmap(
                    |value| f64::from(value) / 20.0,
                    |value: &f64| (*value * 20.0) as i32,
                    true,
                ),
                chunk_fade,
                0.75,
                constant_tip("options.chunkFade.tooltip"),
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

mod preset;
mod size;

#[cfg(test)]
mod tests {
    use super::*;

    fn assets() -> Option<(Font, Lang)> {
        let root = std::path::PathBuf::from(photon_core::content_root::assets().ok()?);
        Some((Font::load(&root)?, Lang::load(&root)))
    }

    #[test]
    fn done_leaves_and_a_gui_scale_press_asks_the_window_for_a_size() {
        let Some((font, lang)) = assets() else { return };
        let mut screen = VideoSettingsScreen::new(
            &font,
            &lang,
            (427, 240),
            true,
            (2, 2),
            VideoValues::default(),
        );
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

    #[test]
    fn the_brightness_slider_shows_and_hands_back_the_option() {
        let Some((font, lang)) = assets() else { return };
        let shown = VideoValues {
            gamma: 0.8,
            particles: photon_game::options::ParticleStatus::Minimal,
            ..VideoValues::default()
        };
        let screen = VideoSettingsScreen::new(&font, &lang, (427, 240), true, (2, 2), shown);
        let back = screen.values(VideoValues::default());
        assert_eq!(back.gamma, 0.8);
        assert_eq!(
            back.particles,
            photon_game::options::ParticleStatus::Minimal
        );
    }

    #[test]
    fn a_preset_puts_the_options_it_controls_to_its_values() {
        let Some((font, lang)) = assets() else { return };
        let mut screen = VideoSettingsScreen::new(
            &font,
            &lang,
            (427, 240),
            true,
            (2, 2),
            VideoValues::default(),
        );
        screen
            .screen
            .list
            .find_mut(GRAPHICS_PRESET)
            .and_then(|control| {
                control
                    .as_any_mut()
                    .downcast_mut::<OptionSlider<SliderableEnum<Named>>>()
            })
            .expect("the preset slider")
            .set_current(&lang, "options.graphics.fast");
        screen.apply_preset(&lang, Some(GRAPHICS_PRESET));
        let values = screen.values(VideoValues::default());
        assert_eq!(values.render_distance, 8);
        assert!(!values.ambient_occlusion);
        assert_eq!(
            values.particles,
            photon_game::options::ParticleStatus::Decreased
        );
        assert_eq!(values.menu_blurriness, 2);
        assert_eq!(
            values.quality.graphics_preset,
            photon_game::options::GraphicsPreset::Fast
        );
        assert_eq!(values.quality.simulation_distance, 6);
        assert_eq!(
            values.quality.cloud_status,
            photon_game::options::CloudStatus::Fast
        );
        assert_eq!(values.quality.entity_distance_scaling, 0.75);
        assert!(!values.quality.entity_shadows);
        // The screen shows what is stored, and hands it back.
        let stored = VideoValues {
            quality: values.quality,
            ..VideoValues::default()
        };
        let reopened = VideoSettingsScreen::new(&font, &lang, (427, 240), true, (2, 2), stored);
        assert_eq!(
            reopened.values(VideoValues::default()).quality,
            values.quality
        );
        // The preset stays as chosen: only a change the user made turns it to Custom.
        let preset = screen
            .screen
            .list
            .find(GRAPHICS_PRESET)
            .and_then(|control| {
                control
                    .as_any()
                    .downcast_ref::<OptionSlider<SliderableEnum<Named>>>()
            });
        assert_eq!(
            preset.map(|slider| *slider.value()),
            Some("options.graphics.fast")
        );
    }
}
