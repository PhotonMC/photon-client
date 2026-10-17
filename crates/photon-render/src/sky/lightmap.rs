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
