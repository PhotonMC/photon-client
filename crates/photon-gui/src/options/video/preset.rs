//! `GraphicsPreset.apply`: a preset puts the options it controls to its values, and the widgets
//! of those options show them. The change is not the user's, so the preset stays as chosen
//! (`isApplyingGraphicsPreset`).

use photon_game::options::{
    CloudStatus, GraphicsPreset, PrioritizeChunkUpdates, QualityOptions, TextureFiltering,
};

use super::*;

/// A value a preset gives to the widget of one option.
enum Setting {
    Int(u8, i32),
    Flag(u8, bool),
    Name(u8, Named),
    Distance(u8, f64),
}

/// What a preset gives the options it controls (`GraphicsPreset.apply`).
struct Preset {
    biome_blend: i32,
    render_distance: i32,
    prioritize: Named,
    simulation_distance: i32,
    fancy: bool,
    clouds: Named,
    particles: Named,
    mipmaps: i32,
    entity_distance: f64,
    menu_blurriness: i32,
    cloud_range: i32,
    weather_radius: i32,
    improved_transparency: bool,
    anisotropy: i32,
    filtering: Named,
}

const FAST: Preset = Preset {
    biome_blend: 1,
    render_distance: 8,
    prioritize: "options.prioritizeChunkUpdates.none",
    simulation_distance: 6,
    fancy: false,
    clouds: "options.clouds.fast",
    particles: "options.particles.decreased",
    mipmaps: 2,
    entity_distance: 0.75,
    menu_blurriness: 2,
    cloud_range: 32,
    weather_radius: 5,
    improved_transparency: false,
    anisotropy: 1,
    filtering: "options.textureFiltering.none",
};

const FANCY: Preset = Preset {
    biome_blend: 2,
    render_distance: 16,
    prioritize: "options.prioritizeChunkUpdates.byPlayer",
    simulation_distance: 12,
    fancy: true,
    clouds: "options.clouds.fancy",
    particles: "options.particles.all",
    mipmaps: 4,
    entity_distance: 1.0,
    menu_blurriness: 5,
    cloud_range: 64,
    weather_radius: 10,
    improved_transparency: false,
    anisotropy: 1,
    filtering: "options.textureFiltering.rgss",
};

/// Improved transparency is off on macOS. The anisotropic filter is taken as the device allows it, as a device with known issues is
/// not modelled.
const FABULOUS: Preset = Preset {
    render_distance: 32,
    entity_distance: 1.25,
    cloud_range: 128,
    improved_transparency: !cfg!(target_os = "macos"),
    anisotropy: 2,
    filtering: "options.textureFiltering.anisotropic",
    ..FANCY
};

impl Preset {
    /// The preset a caption names, none for Custom.
    fn named(caption: &str) -> Option<&'static Self> {
        match caption {
            "options.graphics.fast" => Some(&FAST),
            "options.graphics.fancy" => Some(&FANCY),
            "options.graphics.fabulous" => Some(&FABULOUS),
            _ => None,
        }
    }

    /// The settings, in the order `GraphicsPreset.apply` makes them.
    fn settings(&self) -> [Setting; 17] {
        use Setting::{Distance, Flag, Int, Name};
        [
            Int(BIOME_BLEND, self.biome_blend),
            Int(RENDER_DISTANCE, self.render_distance),
            Name(PRIORITIZE_CHUNK_UPDATES, self.prioritize),
            Int(SIMULATION_DISTANCE, self.simulation_distance),
            Flag(AMBIENT_OCCLUSION, self.fancy),
            Name(CLOUDS, self.clouds),
            Name(PARTICLES, self.particles),
            Int(MIPMAPS, self.mipmaps),
            Flag(ENTITY_SHADOWS, self.fancy),
            Distance(ENTITY_DISTANCE, self.entity_distance),
            Int(MENU_BLURRINESS, self.menu_blurriness),
            Int(CLOUD_RANGE, self.cloud_range),
            Flag(CUTOUT_LEAVES, self.fancy),
            Flag(IMPROVED_TRANSPARENCY, self.improved_transparency),
            Int(WEATHER_RADIUS, self.weather_radius),
            Int(MAX_ANISOTROPY, self.anisotropy),
            Name(TEXTURE_FILTERING, self.filtering),
        ]
    }
}

impl VideoSettingsScreen {
    /// A change of the preset slider applies the preset it came to.
    pub(super) fn apply_preset(&mut self, lang: &Lang, changed: Option<u8>) {
        if changed != Some(GRAPHICS_PRESET) {
            return;
        }
        let Some(preset) = self
            .screen
            .list
            .find(GRAPHICS_PRESET)
            .and_then(|control| {
                control
                    .as_any()
                    .downcast_ref::<OptionSlider<SliderableEnum<Named>>>()
            })
            .map(|slider| *slider.value())
        else {
            return;
        };
        let Some(preset) = Preset::named(preset) else {
            return;
        };
        for setting in preset.settings() {
            self.show(lang, &setting);
        }
    }

    /// `resetOption`: the widget of the option takes the value.
    fn show(&mut self, lang: &Lang, setting: &Setting) {
        let list = &mut self.screen.list;
        match *setting {
            Setting::Int(tag, value) => {
                if let Some(slider) = list.find_mut(tag).and_then(|control| {
                    control
                        .as_any_mut()
                        .downcast_mut::<OptionSlider<IntRange>>()
                }) {
                    slider.set_current(lang, value);
                }
            }
            Setting::Distance(tag, value) => {
                if let Some(slider) = list.find_mut(tag).and_then(|control| {
                    control
                        .as_any_mut()
                        .downcast_mut::<OptionSlider<EntityDistanceValues>>()
                }) {
                    slider.set_current(lang, value);
                }
            }
            Setting::Flag(tag, value) => {
                if let Some(button) = list
                    .find_mut(tag)
                    .and_then(|control| control.as_any_mut().downcast_mut::<CycleButton<bool>>())
                {
                    button.set_value(lang, value);
                }
            }
            Setting::Name(tag, value) => {
                if let Some(button) = list
                    .find_mut(tag)
                    .and_then(|control| control.as_any_mut().downcast_mut::<CycleButton<Named>>())
                {
                    button.set_value(lang, value);
                }
            }
        }
    }
}

impl VideoSettingsScreen {
    /// What the sliders and buttons of the quality options say, over `fallback`.
    pub(super) fn quality(&self, fallback: QualityOptions) -> QualityOptions {
        QualityOptions {
            graphics_preset: self
                .name_of(GRAPHICS_PRESET, GraphicsPreset::from_caption_key)
                .unwrap_or(fallback.graphics_preset),
            biome_blend_radius: self.int_of(BIOME_BLEND, fallback.biome_blend_radius),
            prioritize_chunk_updates: self
                .name_of(
                    PRIORITIZE_CHUNK_UPDATES,
                    PrioritizeChunkUpdates::from_caption_key,
                )
                .unwrap_or(fallback.prioritize_chunk_updates),
            simulation_distance: self.int_of(SIMULATION_DISTANCE, fallback.simulation_distance),
            cloud_status: self
                .name_of(CLOUDS, CloudStatus::from_caption_key)
                .unwrap_or(fallback.cloud_status),
            mipmap_levels: self.int_of(MIPMAPS, fallback.mipmap_levels),
            entity_shadows: self.flag_of(ENTITY_SHADOWS, fallback.entity_shadows),
            entity_distance_scaling: self
                .screen
                .list
                .find(ENTITY_DISTANCE)
                .and_then(|control| {
                    control
                        .as_any()
                        .downcast_ref::<OptionSlider<EntityDistanceValues>>()
                })
                .map_or(fallback.entity_distance_scaling, |slider| *slider.value()),
            cloud_range: self.int_of(CLOUD_RANGE, fallback.cloud_range),
            cutout_leaves: self.flag_of(CUTOUT_LEAVES, fallback.cutout_leaves),
            improved_transparency: self
                .flag_of(IMPROVED_TRANSPARENCY, fallback.improved_transparency),
            weather_radius: self.int_of(WEATHER_RADIUS, fallback.weather_radius),
            max_anisotropy_bit: self.int_of(MAX_ANISOTROPY, fallback.max_anisotropy_bit),
            texture_filtering: self
                .name_of(TEXTURE_FILTERING, TextureFiltering::from_caption_key)
                .unwrap_or(fallback.texture_filtering),
        }
    }

    fn int_of(&self, tag: u8, fallback: i32) -> i32 {
        self.screen
            .list
            .find(tag)
            .and_then(|control| control.as_any().downcast_ref::<OptionSlider<IntRange>>())
            .map_or(fallback, |slider| *slider.value())
    }

    fn flag_of(&self, tag: u8, fallback: bool) -> bool {
        self.screen
            .list
            .find(tag)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
            .map_or(fallback, |button| *button.value())
    }

    /// The value a named button or the preset slider shows, by `parse` of its caption key.
    fn name_of<T>(&self, tag: u8, parse: fn(&str) -> Option<T>) -> Option<T> {
        let control = self.screen.list.find(tag)?;
        if let Some(slider) = control
            .as_any()
            .downcast_ref::<OptionSlider<SliderableEnum<Named>>>()
        {
            return parse(slider.value());
        }
        control
            .as_any()
            .downcast_ref::<CycleButton<Named>>()
            .and_then(|button| parse(button.value()))
    }
}
