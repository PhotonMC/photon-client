use super::*;

impl Attributes {
    pub fn get(&self, attribute: Attribute) -> Value {
        self.values
            .get(&attribute)
            .copied()
            .unwrap_or_else(|| attribute.default_value())
    }

    /// Blend towards the next tick's values as the attribute probe does.
    pub fn lerp(&self, next: &Self, partial_tick: f32) -> Self {
        let values = ATTRIBUTES
            .into_iter()
            .map(|attribute| {
                (
                    attribute,
                    partial_tick_lerp(
                        attribute.kind(),
                        partial_tick,
                        self.get(attribute),
                        next.get(attribute),
                    ),
                )
            })
            .collect();
        Self { values }
    }
}

/// The lightmap's inputs, as the lightmap extractor reads them.
#[derive(Debug, Clone, Copy)]
pub struct LightmapInputs {
    pub sky_colour: [f32; 3],
    pub sky_factor: f32,
    pub block_tint: [f32; 3],
    pub ambient: [f32; 3],
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
        }
    }
}

/// A sky vertex, in view-rotated space around the camera.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SkyVertex {
    pub position: [f32; 3],
    /// Atlas coordinate, or negative for an untextured vertex.
    pub uv: [f32; 2],
    pub colour: [f32; 4],
    /// The spherical and cylindrical distances fog is measured over, or
    /// negative for no fog.
    pub fog_distance: [f32; 2],
}

/// How a run of sky vertices blends onto what is below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyBlend {
    /// Source alpha over the destination.
    Translucent,
    /// Source times its alpha added to the destination.
    Overlay,
}

/// Everything one frame of sky needs.
#[derive(Debug, Clone)]
pub struct SkyFrame {
    /// The colour the frame is cleared to, which is the fog colour.
    pub clear_colour: [f32; 4],
    pub fog_colour: [f32; 4],
    /// Where fog on the sky is complete, in blocks.
    pub sky_fog_end: f32,
    /// The terrain's fog, in blocks: where the environmental fog starts and
    /// ends over spherical distance, and where the render distance fog
    /// starts and ends over cylindrical distance.
    pub terrain_fog: [f32; 4],
    /// The cloud colour (red, green, blue, alpha, 0 to 1), the height of the
    /// clouds' bottom, and where cloud fog is complete, in blocks.
    pub cloud_colour: [f32; 4],
    pub cloud_height: f32,
    pub cloud_fog_end: f32,
    /// Triangles, three vertices each.
    pub vertices: Vec<SkyVertex>,
    /// Runs of triangles and how each blends.
    pub batches: Vec<(SkyBlend, u32, u32)>,
}

pub(super) type Matrix = [[f32; 4]; 4];

/// Where the sky's sprites sit on the atlas.
pub struct SkySprites {
    pub sun: [f32; 4],
    pub moon: [[f32; 4]; 8],
}

/// The inputs a frame of sky reads besides the attributes.
pub struct SkyView {
    /// The camera's forward direction.
    pub forward: [f32; 3],
    /// The camera's eye height above the horizon (sea level on a normal world).
    pub eye_above_horizon: f64,
    pub eye_in_water: bool,
    /// The effective render distance in chunks.
    pub render_distance: i32,
    /// The cloud range option, in chunks.
    pub cloud_range: i32,
    /// The level's rain and thunder at this frame.
    pub weather: Weather,
    /// How far rain has pulled the fog in, 0 to 1.
    pub rain_fog: f32,
}
