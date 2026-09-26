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
    generate_mip_levels_with(
        "block/",
        source,
        levels,
        MipmapStrategy::Mean,
        0.0,
        Transparency::NONE,
    )
}

/// Which mip strategy a sprite asks for, as the atlas metadata names them.
///
/// `Auto` is not a strategy of its own: it resolves to `Cutout` when the sprite
/// is known to have fully transparent texels, and to `Mean` otherwise.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MipmapStrategy {
    Auto,
    Mean,
    Cutout,
    StrictCutout,
    DarkCutout,
}

impl MipmapStrategy {
    /// The serialised name, which is what a sprite's metadata carries.
    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Mean => "mean",
            Self::Cutout => "cutout",
            Self::StrictCutout => "strict_cutout",
            Self::DarkCutout => "dark_cutout",
        }
    }

    /// Parse one serialised name.
    pub fn parse(name: &str) -> Option<Self> {
        [
            Self::Auto,
            Self::Mean,
            Self::Cutout,
            Self::StrictCutout,
            Self::DarkCutout,
        ]
        .into_iter()
        .find(|strategy| strategy.name() == name)
    }

    /// Whether the strategy measures alpha coverage and rescales to keep it.
    fn is_cutout(self) -> bool {
        matches!(self, Self::Cutout | Self::StrictCutout | Self::DarkCutout)
    }

    /// The alpha a texel has to pass to count as covered. The strict variant
    /// lowers the bar, so thin shapes keep more of themselves in far mips.
    fn cutout_reference_alpha(self) -> f32 {
        if self == Self::StrictCutout {
            STRICT_ALPHA_CUTOFF
        } else {
            ALPHA_CUTOFF
        }
    }
}

/// What the atlas knows about a sprite's alpha, which is what `Auto` reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Transparency {
    /// A texel is fully transparent somewhere.
    pub has_transparent: bool,
    /// A texel is partly transparent somewhere.
    pub has_translucent: bool,
}

impl Transparency {
    /// Fully opaque.
    pub const NONE: Self = Self {
        has_transparent: false,
        has_translucent: false,
    };

    /// Build from the two flags.
    pub fn of(has_transparent: bool, has_translucent: bool) -> Self {
        Self {
            has_transparent,
            has_translucent,
        }
    }
}

/// The alpha a texel needs to count as covered under the plain cutout.
const ALPHA_CUTOFF: f32 = 0.5;

/// The same bar under the strict cutout.
const STRICT_ALPHA_CUTOFF: f32 = 0.3;

/// The four neighbours the fill walks, in the order the reference lists them.
///
/// The order decides which colour wins when two opaque texels sit the same
/// distance from a transparent one, so it is part of the result.
const FILL_DIRECTIONS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Java `ARGB.alphaFloat`.
fn alpha_float(alpha: u8) -> f32 {
    f32::from(alpha) / 255.0
}

/// Java `ARGB.as8BitChannel`, which floors rather than rounds.
fn as_8_bit_channel(value: f32) -> u8 {
    crate::math::floor_f32(value * 255.0) as u8
}

/// Spread the nearest opaque colour into every fully transparent texel,
/// keeping the alpha at zero.
///
/// Without this a mip of a leaf blends its visible colour with whatever the
/// transparent texels happen to carry, which for most textures is black, and
/// the leaf grows a dark fringe as the camera pulls back.
pub fn solidify(image: &mut DecodedImage) {
    let width = image.width as usize;
    let height = image.height as usize;
    let mut nearest = vec![[0_u8; 3]; width * height];
    let mut distances = vec![u32::MAX; width * height];
    let mut queue = std::collections::VecDeque::new();
    // The seed order is x outer, y inner, which is what breaks ties between
    // two opaque texels the same distance away.
    for x in 0..width {
        for y in 0..height {
            let packed = x + y * width;
            if image.rgba[packed * 4 + 3] != 0 {
                distances[packed] = 0;
                nearest[packed] = [
                    image.rgba[packed * 4],
                    image.rgba[packed * 4 + 1],
                    image.rgba[packed * 4 + 2],
                ];
                queue.push_back(packed);
            }
        }
    }
    while let Some(packed) = queue.pop_front() {
        let x = (packed % width) as i32;
        let y = (packed / width) as i32;
        for (step_x, step_y) in FILL_DIRECTIONS {
            let neighbour_x = x + step_x;
            let neighbour_y = y + step_y;
            if neighbour_x < 0
                || neighbour_y < 0
                || neighbour_x >= width as i32
                || neighbour_y >= height as i32
            {
                continue;
            }
            let neighbour = neighbour_x as usize + neighbour_y as usize * width;
            if distances[neighbour] > distances[packed] + 1 {
                distances[neighbour] = distances[packed] + 1;
                nearest[neighbour] = nearest[packed];
                queue.push_back(neighbour);
            }
        }
    }
    for (texel, nearest) in image.rgba.as_chunks_mut::<4>().0.iter_mut().zip(&nearest) {
        if texel[3] == 0 {
            texel[..3].copy_from_slice(nearest);
        }
    }
}

/// Fill every fully transparent texel with three quarters of the darkest
/// colour the image has, keeping the alpha at zero.
///
/// This is what the dark cutout uses instead of the fill above: a texture
/// whose visible part should fade towards its own shadow rather than towards
/// its nearest colour.
pub fn fill_empty_areas_with_dark_colour(image: &mut DecodedImage) {
    let width = image.width as usize;
    let height = image.height as usize;
    // No opaque texel leaves the reference holding an all-ones colour, whose
    // channels are 255, so the fill lands on 191 rather than on black.
    let mut darkest = [255_u8; 3];
    let mut min_brightness = i32::MAX;
    for x in 0..width {
        for y in 0..height {
            let packed = (x + y * width) * 4;
            if image.rgba[packed + 3] == 0 {
                continue;
            }
            let brightness = i32::from(image.rgba[packed])
                + i32::from(image.rgba[packed + 1])
                + i32::from(image.rgba[packed + 2]);
            if brightness < min_brightness {
                min_brightness = brightness;
                darkest = [
                    image.rgba[packed],
                    image.rgba[packed + 1],
                    image.rgba[packed + 2],
                ];
            }
        }
    }
    let darkened = darkest.map(|channel| (3 * u32::from(channel) / 4) as u8);
    for texel in image.rgba.as_chunks_mut::<4>().0 {
        if texel[3] == 0 {
            texel[..3].copy_from_slice(&darkened);
        }
    }
}

/// How much of the image passes the alpha reference, measured the way a
/// rasteriser would see it: four by four samples bilinearly interpolated
/// across every two by two texel corner.
///
/// A one texel wide image has no corner to sample, so the divisor is zero and
/// the result is not a number. The search below relies on that: every
/// comparison against it fails, so the scale it had stands.
fn alpha_test_coverage(image: &DecodedImage, alpha_reference: f32, alpha_scale: f32) -> f32 {
    let width = image.width as i64;
    let height = image.height as i64;
    let mut coverage = 0.0_f32;
    let sample = |x: i64, y: i64| -> f32 {
        let packed = (x + y * width) as usize * 4 + 3;
        (alpha_float(image.rgba[packed]) * alpha_scale).clamp(0.0, 1.0)
    };
    for y in 0..height.saturating_sub(1) {
        for x in 0..width.saturating_sub(1) {
            let alpha_00 = sample(x, y);
            let alpha_10 = sample(x + 1, y);
            let alpha_01 = sample(x, y + 1);
            let alpha_11 = sample(x + 1, y + 1);
            let mut texel_coverage = 0.0_f32;
            for subsample_y in 0..4 {
                let fraction_y = (subsample_y as f32 + 0.5) / 4.0;
                for subsample_x in 0..4 {
                    let fraction_x = (subsample_x as f32 + 0.5) / 4.0;
                    let alpha = alpha_00 * (1.0 - fraction_x) * (1.0 - fraction_y)
                        + alpha_10 * fraction_x * (1.0 - fraction_y)
                        + alpha_01 * (1.0 - fraction_x) * fraction_y
                        + alpha_11 * fraction_x * fraction_y;
                    if alpha > alpha_reference {
                        texel_coverage += 1.0;
                    }
                }
            }
            coverage += texel_coverage / 16.0;
        }
    }
    coverage / ((width - 1) * (height - 1)) as f32
}

/// Rescale the image's alpha so it covers as much as the source did.
///
/// Halving shrinks a cutout shape, because the blended alpha of a visible and
/// an invisible texel often falls below the cutoff. A five step bisection over
/// the scale between nothing and four brings the coverage back; the bias and
/// the fixed quarter tenth are added afterwards, so even a scale of one moves
/// the alpha.
fn scale_alpha_to_coverage(
    image: &mut DecodedImage,
    desired_coverage: f32,
    alpha_reference: f32,
    alpha_cutoff_bias: f32,
) {
    let mut min_alpha_scale = 0.0_f32;
    let mut max_alpha_scale = 4.0_f32;
    let mut alpha_scale = 1.0_f32;
    let mut best_alpha_scale = 1.0_f32;
    let mut best_error = f32::MAX;
    for _ in 0..5 {
        let current_coverage = alpha_test_coverage(image, alpha_reference, alpha_scale);
        let error = (current_coverage - desired_coverage).abs();
        if error < best_error {
            best_error = error;
            best_alpha_scale = alpha_scale;
        }
        if current_coverage < desired_coverage {
            min_alpha_scale = alpha_scale;
        } else if current_coverage > desired_coverage {
            max_alpha_scale = alpha_scale;
        } else {
            break;
        }
        alpha_scale = (min_alpha_scale + max_alpha_scale) * 0.5;
    }
    for texel in image.rgba.as_chunks_mut::<4>().0 {
        let alpha = alpha_float(texel[3]) * best_alpha_scale + alpha_cutoff_bias + 0.025;
        texel[3] = as_8_bit_channel(alpha.clamp(0.0, 1.0));
    }
}

impl SrgbTables {
    /// Blend four texels in linear light, ignoring the ones that are fully
    /// transparent and dividing by four all the same.
    ///
    /// Dividing by the full four rather than by how many took part is what
    /// darkens the result, which is the point: a texel next to a hole should
    /// read as shadowed rather than as its own colour.
    pub fn darkened_alpha_blend(&self, texels: [[u8; 4]; 4]) -> [u8; 4] {
        let mut totals = [0.0_f32; 4];
        for texel in texels {
            if texel[3] == 0 {
                continue;
            }
            for (total, channel) in totals
                .iter_mut()
                .zip([texel[3], texel[0], texel[1], texel[2]])
            {
                *total += f32::from(self.to_linear(channel)) / 1023.0;
            }
        }
        let srgb = totals.map(|total| self.linear_to_srgb_channel(total / 4.0));
        [srgb[1], srgb[2], srgb[3], srgb[0]]
    }

    /// Java `ARGB.linearToSrgbChannel`, which indexes the table by the floor
    /// of the linear value scaled to the table's depth.
    fn linear_to_srgb_channel(&self, linear: f32) -> u8 {
        self.to_srgb(crate::math::floor_f32(linear * 1023.0) as u16)
    }
}

/// Generate mip levels for one sprite under the strategy it asks for.
///
/// Level zero is the source, possibly filled in place first; each later level
/// halves both axes. The mean strategy blends in linear light and leaves alpha
/// alone, while the cutout strategies fill the transparent area beforehand and
/// rescale each level's alpha to keep the shape's size.
pub fn generate_mip_levels_with(
    path: &str,
    source: &DecodedImage,
    levels: u32,
    strategy: MipmapStrategy,
    alpha_cutoff_bias: f32,
    transparency: Transparency,
) -> Vec<DecodedImage> {
    let strategy = if strategy == MipmapStrategy::Auto {
        if transparency.has_transparent {
            MipmapStrategy::Cutout
        } else {
            MipmapStrategy::Mean
        }
    } else {
        strategy
    };
    let tables = SrgbTables::new();
    let mut level_zero = source.clone();
    // Item textures are drawn at a size where the fill would show, so they
    // keep their holes. The check is on the path, not on the sprite.
    if !path.starts_with("item/") {
        match strategy {
            MipmapStrategy::Cutout | MipmapStrategy::StrictCutout => solidify(&mut level_zero),
            MipmapStrategy::DarkCutout => fill_empty_areas_with_dark_colour(&mut level_zero),
            _ => {}
        }
    }
    let mut out = Vec::with_capacity(levels as usize + 1);
    out.push(level_zero);
    if levels == 0 {
        return out;
    }
    // The coverage every level aims at is the source's, measured after the
    // fill and before any halving.
    let cutout_reference = strategy.cutout_reference_alpha();
    let original_coverage = if strategy.is_cutout() {
        alpha_test_coverage(&out[0], cutout_reference, 1.0)
    } else {
        0.0
    };
    for level in 1..=levels as usize {
        let previous = &out[level - 1];
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
                let corners = [read(0, 0), read(1, 0), read(0, 1), read(1, 1)];
                let blended = if strategy == MipmapStrategy::DarkCutout {
                    tables.darkened_alpha_blend(corners)
                } else {
                    tables.mean_linear(corners)
                };
                let target = ((y * width) + x) as usize * 4;
                rgba[target..target + 4].copy_from_slice(&blended);
            }
        }
        let mut generated = DecodedImage {
            width,
            height,
            rgba,
        };
        if strategy.is_cutout() {
            scale_alpha_to_coverage(
                &mut generated,
                original_coverage,
                cutout_reference,
                alpha_cutoff_bias,
            );
        }
        out.push(generated);
    }
    out
}
