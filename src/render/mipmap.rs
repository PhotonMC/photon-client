//! Mip level generation for atlas sprites.
//!
//! Halving a texture is not an average of its bytes. The reference converts
//! each channel to linear light through a fixed table, averages there, and
//! converts back, so a mip of a dark and a bright texel is brighter than the
//! byte mean would give.

use crate::render::png::DecodedImage;

/// How many steps the linear side of the conversion has.
const LINEAR_CHANNEL_DEPTH: usize = 1024;

/// sRGB byte to linear step, as the reference builds it once at startup.
fn srgb_to_linear_table() -> [u16; 256] {
    std::array::from_fn(|index| {
        let channel = index as f32 / 255.0;
        java_round(compute_srgb_to_linear(channel) * 1023.0) as u16
    })
}

/// Linear step back to an sRGB byte.
fn linear_to_srgb_table() -> [u8; LINEAR_CHANNEL_DEPTH] {
    std::array::from_fn(|index| {
        let channel = index as f32 / 1023.0;
        java_round(compute_linear_to_srgb(channel) * 255.0) as u8
    })
}

/// The sRGB transfer function, with the linear segment near black.
fn compute_srgb_to_linear(value: f32) -> f32 {
    if value >= 0.04045 {
        ((f64::from(value) + 0.055) / 1.055).powf(2.4) as f32
    } else {
        value / 12.92
    }
}

/// The inverse transfer function.
fn compute_linear_to_srgb(value: f32) -> f32 {
    if value >= 0.003_130_8 {
        (1.055 * f64::from(value).powf(0.416_666_666_666_666_7) - 0.055) as f32
    } else {
        12.92 * value
    }
}

/// Java `Math.round(float)`, which is a floor of the value plus a half.
fn java_round(value: f32) -> i32 {
    crate::math::floor_f32(value + 0.5)
}

/// The two conversion tables, built once.
pub struct SrgbTables {
    forward: [u16; 256],
    backward: [u8; LINEAR_CHANNEL_DEPTH],
}

impl Default for SrgbTables {
    fn default() -> Self {
        Self::new()
    }
}

impl SrgbTables {
    /// Build both tables.
    pub fn new() -> Self {
        Self {
            forward: srgb_to_linear_table(),
            backward: linear_to_srgb_table(),
        }
    }

    /// The linear step one sRGB byte maps to.
    pub fn to_linear(&self, channel: u8) -> u16 {
        self.forward[usize::from(channel)]
    }

    /// The sRGB byte one linear step maps back to.
    pub fn to_srgb(&self, linear: u16) -> u8 {
        self.backward[usize::from(linear)]
    }

    /// Java `ARGB.meanLinear` for one channel: average in linear light, with
    /// the division taken on integers before the table lookup back.
    fn channel_mean(&self, values: [u8; 4]) -> u8 {
        let total: u32 = values
            .iter()
            .map(|value| u32::from(self.to_linear(*value)))
            .sum();
        self.to_srgb((total / 4) as u16)
    }

    /// Java `ARGB.meanLinear`: alpha averages as plain bytes, colour in linear.
    pub fn mean_linear(&self, texels: [[u8; 4]; 4]) -> [u8; 4] {
        let alpha: u32 = texels.iter().map(|texel| u32::from(texel[3])).sum();
        [
            self.channel_mean(texels.map(|texel| texel[0])),
            self.channel_mean(texels.map(|texel| texel[1])),
            self.channel_mean(texels.map(|texel| texel[2])),
            (alpha / 4) as u8,
        ]
    }
}

/// Generate mip levels by repeated halving, averaging in linear light.
///
/// Level zero is the source. Each later level halves both axes and blends the
/// four texels it covers; this is the reference's mean strategy, which is what
/// an opaque block texture uses.
pub fn generate_mip_levels(source: &DecodedImage, levels: u32) -> Vec<DecodedImage> {
    let tables = SrgbTables::new();
    let mut out = Vec::with_capacity(levels as usize + 1);
    out.push(source.clone());
    for level in 1..=levels {
        let previous = &out[level as usize - 1];
        let width = previous.width >> 1;
        let height = previous.height >> 1;
        let mut rgba = vec![0_u8; (width * height * 4) as usize];
        for x in 0..width {
            for y in 0..height {
                let read = |offset_x: u32, offset_y: u32| -> [u8; 4] {
                    let index =
                        (((y * 2 + offset_y) * previous.width) + (x * 2 + offset_x)) as usize * 4;
                    [
                        previous.rgba[index],
                        previous.rgba[index + 1],
                        previous.rgba[index + 2],
                        previous.rgba[index + 3],
                    ]
                };
                let blended = tables.mean_linear([read(0, 0), read(1, 0), read(0, 1), read(1, 1)]);
                let target = ((y * width) + x) as usize * 4;
                rgba[target..target + 4].copy_from_slice(&blended);
            }
        }
        out.push(DecodedImage {
            width,
            height,
            rgba,
        });
    }
    out
}
