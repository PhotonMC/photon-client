//! The video settings screen (`VideoSettingsScreen`): the display, quality
//! and preference options in a scrolling list. The GUI scale, the frame limits,
//! vsync, the attack indicator and the render distance do something; the others
//! hold a value on their widget and nothing follows.

use photon_game::options::values::{IntRange, MappedIntRange, SliderableEnum};
use photon_game::options::{AttackIndicator, FRAMERATE_LIMIT_DEFAULT, InactivityFpsLimit};
use photon_game::text::{Component, Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::options::list::{
    Build, Named, SubScreen, SubScreenInput, boolean, boolean_tip, constant_tip, counted, cycle,
    cycle_tip, named, named_tip, off_composed, slider, slider_tip,
};
use crate::gui::options::text::{generic_int, generic_value, on_off, percent_value};
use crate::gui::widgets::cycle::CycleButton;
use crate::gui::widgets::slider::{OptionSlider, SliderKey};
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
const ATTACK_INDICATOR: u8 = 5;
const FRAMERATE_LIMIT: u8 = 6;
const VSYNC: u8 = 7;
const INACTIVITY_FPS_LIMIT: u8 = 8;
const RENDER_DISTANCE: u8 = CUSTOMISING | 3;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoValues {
    pub attack_indicator: AttackIndicator,
    pub framerate_limit: i32,
    pub vsync: bool,
    pub inactivity_fps_limit: InactivityFpsLimit,
    /// `renderDistance`, in chunks.
    pub render_distance: i32,
}

impl Default for VideoValues {
    fn default() -> Self {
        Self {
            attack_indicator: AttackIndicator::Crosshair,
            framerate_limit: FRAMERATE_LIMIT_DEFAULT,
            vsync: true,
            inactivity_fps_limit: InactivityFpsLimit::Afk,
            render_distance: 12,
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
    crate::gui::options::list::toggle(value)
}

fn translated_value(value: &Named) -> Component {
    crate::gui::options::list::translated(value)
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
            PLAIN,
            slider(
                "options.gamma",
                photon_game::options::values::UnitDouble,
                gamma,
                0.5,
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

/// `qualityOptions`, the first nine.
fn quality_options_first(render_distance: i32) -> Vec<(u8, Build)> {
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
            RENDER_DISTANCE,
            slider(
                "options.renderDistance",
                IntRange::with_apply_value_immediately(2, 32, false),
                chunks,
                render_distance,
            ),
        ),
        (
            CUSTOMISING,
            named_tip(
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
            slider_tip(
                "options.accessibility.menu_background_blurriness",
                IntRange::new(0, 10),
                blurriness,
                5,
                constant_tip("options.accessibility.menu_background_blurriness.tooltip"),
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
        (
            CUSTOMISING,
            boolean_tip("options.cutoutLeaves", true, "options.cutoutLeaves.tooltip"),
        ),
        (
            CUSTOMISING,
            boolean_tip(
                "options.improvedTransparency",
                false,
                "options.improvedTransparency.tooltip",
            ),
        ),
        (
            TEXTURE_FILTERING,
            named_tip(
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
            slider_tip(
                "options.maxAnisotropy",
                IntRange::new(1, 3),
                anisotropy,
                2,
                constant_tip("options.maxAnisotropy.tooltip"),
            ),
        ),
        (
            CUSTOMISING,
            slider_tip(
                "options.weatherRadius",
                IntRange::new(3, 10),
                blocks,
                10,
                constant_tip("options.weatherRadius.tooltip"),
            ),
        ),
    ]
}

/// `qualityOptions`.
fn quality_options(render_distance: i32) -> Vec<(u8, Build)> {
    let mut options = quality_options_first(render_distance);
    options.extend(quality_options_rest());
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

mod size;

#[cfg(test)]
mod tests {
    use super::*;

    fn assets() -> Option<(Font, Lang)> {
        let root = std::path::PathBuf::from(std::env::var("PHOTON_ASSETS").ok()?);
        Some((Font::load(&root)?, Lang::load(&root)))
    }

    #[test]
    fn done_leaves_and_a_gui_scale_press_asks_the_window_for_a_size() {
        let Some((font, lang)) = assets() else { return };
        let mut screen =
            VideoSettingsScreen::new(&font, &lang, (427, 240), true, 2, 2, VideoValues::default());
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
