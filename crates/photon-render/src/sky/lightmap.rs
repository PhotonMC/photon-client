use super::*;

/// The lightmap's inputs, as the lightmap extractor reads them.
#[derive(Debug, Clone, Copy)]
pub struct LightmapInputs {
    pub sky_colour: [f32; 3],
    pub sky_factor: f32,
    pub block_tint: [f32; 3],
    pub ambient: [f32; 3],
    /// `NIGHT_VISION_COLOR`.
    pub night_vision_colour: [f32; 3],
}

impl Attributes {
    /// The sky light colour and factor, the block light tint and the ambient
    /// colour, each colour as the lightmap takes it (0 to 1 per channel).
    pub fn lightmap(&self) -> LightmapInputs {
        let rgb = |attribute: Attribute| {
            let colour = self.get(attribute).color();
            [
                channel(colour, 16) as f32 / 255.0,
                channel(colour, 8) as f32 / 255.0,
                channel(colour, 0) as f32 / 255.0,
            ]
        };
        LightmapInputs {
            sky_colour: rgb(Attribute::SkyLightColor),
            sky_factor: self.get(Attribute::SkyLightFactor).float(),
            block_tint: rgb(Attribute::BlockLightTint),
            ambient: rgb(Attribute::AmbientLightColor),
            night_vision_colour: rgb(Attribute::NightVisionColor),
        }
    }
}

/// The brightness option's default (`Options.gamma`).
const GAMMA: f32 = 0.5;

/// What the local player gives the lightmap besides the attributes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightmapView {
    /// `getEffectBlendFactor(DARKNESS)`, zero without the effect.
    pub darkness_blend: f32,
    /// `Options.darknessEffectScale`.
    pub darkness_scale: f32,
    /// The player's `tickCount`.
    pub tick_count: i32,
    /// `nightVisionEffectIntensity`: the night vision scale, the water vision under conduit
    /// power, or zero.
    pub night_vision: f32,
    /// How far the boss bars have darkened the world, between two ticks.
    pub boss_darkening: f32,
    /// What the End's flash adds to the sky light's factor (see [`end_flash_boost`]).
    pub end_flash_boost: f32,
}

/// What `LightmapRenderStateExtractor` adds to the sky light's factor for the End's flash: its
/// intensity, a third of it under a boss bar's world fog, nothing in a dimension without flashes
/// or with the option that hides them on.
pub fn end_flash_boost(intensity: Option<f32>, hidden: bool, boss_fog: bool) -> f32 {
    match intensity {
        Some(intensity) if !hidden && boss_fog => intensity / 3.0,
        Some(intensity) if !hidden => intensity,
        _ => 0.0,
    }
}

/// The inputs the lightmap shader takes besides the attributes' own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightmapEffects {
    /// `max(ambient, night vision colour * intensity)`, what the sum starts from.
    pub base: [f32; 3],
    /// `renderState.darknessEffectScale`, taken from every channel.
    pub darkness: f32,
    /// `renderState.brightness`, the mix towards the lifted colour.
    pub brightness: f32,
    /// `renderState.bossOverlayWorldDarkening`: the sum's mix towards a redder dusk.
    pub boss_darkening: f32,
    /// `renderState.skyFactor`: the attribute's value and the End's flash.
    pub sky_factor: f32,
}

/// `nightVisionEffectIntensity`: the night vision's scale, else the water vision with conduit
/// power under water, else nothing.
pub fn night_vision_intensity(scale: Option<f32>, conduit_power: bool, water_vision: f32) -> f32 {
    match scale {
        Some(scale) => scale,
        None if water_vision > 0.0 && conduit_power => water_vision,
        None => 0.0,
    }
}

impl LightmapInputs {
    /// The darkness effect's and night vision's part of the extraction
    /// (`LightmapRenderStateExtractor.extract`).
    pub fn effects(&self, view: &LightmapView, partial: f32) -> LightmapEffects {
        let modifier = view.darkness_blend * view.darkness_scale;
        let strength = 0.45 * modifier;
        let angle = (view.tick_count as f32 - partial) * std::f32::consts::PI * 0.025;
        let pulse = photon_core::math::mth_cos_f64(f64::from(angle)) * strength;
        let night = self
            .night_vision_colour
            .map(|channel| channel * view.night_vision);
        LightmapEffects {
            base: std::array::from_fn(|i| self.ambient[i].max(night[i])),
            darkness: pulse.max(0.0) * view.darkness_scale,
            brightness: (GAMMA - modifier).max(0.0),
            boss_darkening: view.boss_darkening,
            sky_factor: self.sky_factor + view.end_flash_boost,
        }
    }
}
