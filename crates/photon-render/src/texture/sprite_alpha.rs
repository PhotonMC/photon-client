//! What a sprite's pixels hold in alpha, kept to pick the layer a face is
//! drawn in (`FaceBakery.computeMaterialTransparency`).

use super::mipmap::{Transparency, compute_transparency};
use super::png::DecodedImage;

/// The pass a face is drawn in (`ChunkSectionLayer.byTransparency`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Solid,
    Cutout,
    Translucent,
}

impl Layer {
    /// A partly transparent texel makes a layer translucent, a fully
    /// transparent one cutout, and neither solid.
    pub fn of(transparency: Transparency) -> Self {
        if transparency.has_translucent {
            Self::Translucent
        } else if transparency.has_transparent {
            Self::Cutout
        } else {
            Self::Solid
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Cutout => "cutout",
            Self::Translucent => "translucent",
        }
    }
}

/// A sprite's alpha: its whole-image answer, and for one that is not opaque
/// the pixels a part of it is measured from.
#[derive(Debug, Clone, Default)]
pub struct SpriteAlpha {
    pub transparency: Transparency,
    pixels: Option<Pixels>,
}

#[derive(Debug, Clone)]
struct Pixels {
    /// The whole image: the strip for an animated sprite.
    image: DecodedImage,
    /// One frame's size.
    frame: (u32, u32),
    /// Each frame the animation shows once, empty for a still sprite.
    frames: Vec<u32>,
}

impl SpriteAlpha {
    /// A sprite whose parts all answer as the whole does.
    pub fn uniform(transparency: Transparency) -> Self {
        Self {
            transparency,
            pixels: None,
        }
    }

    /// Measure a sprite: its image (the whole strip when animated), the size
    /// of one frame and the distinct frames its animation plays.
    pub fn measure(image: &DecodedImage, frame: (u32, u32), frames: Vec<u32>) -> Self {
        let transparency = compute_transparency(image);
        Self {
            transparency,
            pixels: (!transparency.is_opaque()).then(|| Pixels {
                image: image.clone(),
                frame,
                frames,
            }),
        }
    }

    /// `SpriteContents.computeTransparency(u0, v0, u1, v1)`: the part of the
    /// sprite between the fractions, over every distinct frame. `None` where
    /// the part reaches outside the image, which the reference rejects.
    pub fn part(&self, u0: f32, v0: f32, u1: f32, v1: f32) -> Option<Transparency> {
        let Some(pixels) = &self.pixels else {
            return Some(self.transparency);
        };
        if u0 == 0.0 && v0 == 0.0 && u1 == 1.0 && v1 == 1.0 {
            return Some(self.transparency);
        }
        let (width, height) = pixels.frame;
        let x0 = floor(u0 * width as f32);
        let y0 = floor(v0 * height as f32);
        let x1 = ceil(u1 * width as f32);
        let y1 = ceil(v1 * height as f32);
        if pixels.frames.is_empty() {
            return pixels.measure(x0, y0, x1, y1);
        }
        let per_row = (pixels.image.width / width.max(1)).max(1) as i64;
        let mut transparency = Transparency::NONE;
        for &frame in &pixels.frames {
            let (frame_x, frame_y) = (i64::from(frame) % per_row, i64::from(frame) / per_row);
            let part = pixels.measure(
                frame_x * i64::from(width) + x0,
                frame_y * i64::from(height) + y0,
                frame_x * i64::from(width) + x1,
                frame_y * i64::from(height) + y1,
            )?;
            transparency = Transparency::of(
                transparency.has_transparent || part.has_transparent,
                transparency.has_translucent || part.has_translucent,
            );
        }
        Some(transparency)
    }

    /// `FaceBakery.computeMaterialTransparency` for a face's texture
    /// rectangle `[min_u, min_v, max_u, max_v]` in sixteenths of the sprite.
    pub fn face(&self, force_translucent: bool, uvs: [f32; 4]) -> Option<Transparency> {
        if force_translucent {
            return Some(Transparency::of(false, true));
        }
        self.part(
            uvs[0].min(uvs[2]) / 16.0,
            uvs[1].min(uvs[3]) / 16.0,
            uvs[0].max(uvs[2]) / 16.0,
            uvs[1].max(uvs[3]) / 16.0,
        )
    }
}

impl Pixels {
    /// `NativeImage.computeTransparency(x0, y0, x1, y1)` over the whole image.
    fn measure(&self, x0: i64, y0: i64, x1: i64, y1: i64) -> Option<Transparency> {
        let (width, height) = (i64::from(self.image.width), i64::from(self.image.height));
        if x0 < 0 || y0 < 0 || x1 > width || y1 > height {
            return None;
        }
        let (mut transparent, mut translucent) = (false, false);
        for y in y0..y1 {
            for x in x0..x1 {
                match self.image.rgba[((y * width + x) * 4 + 3) as usize] {
                    0 => transparent = true,
                    255 => {}
                    _ => translucent = true,
                }
            }
        }
        Some(Transparency::of(transparent, translucent))
    }
}

/// `Mth.floor`.
fn floor(value: f32) -> i64 {
    let truncated = value as i64;
    if value < truncated as f32 {
        truncated - 1
    } else {
        truncated
    }
}

/// `Mth.ceil`.
fn ceil(value: f32) -> i64 {
    let truncated = value as i64;
    if value > truncated as f32 {
        truncated + 1
    } else {
        truncated
    }
}
