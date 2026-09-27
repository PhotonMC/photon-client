//! The `texture` section of a sprite's metadata file.
//!
//! Every field is optional, so what an absent field falls back to is most of
//! what this module is: the mip strategy defaults to deciding itself from the
//! sprite's pixels, and the cutout bias to nothing.

use crate::json::JsonObject;
use crate::render::mipmap::MipmapStrategy;

/// The parsed section.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TextureMetadata {
    /// Sample the sprite smoothly rather than at the nearest texel.
    pub blur: bool,
    /// Clamp at the sprite's edge rather than repeating.
    pub clamp: bool,
    /// Which mip strategy the sprite's levels are built with.
    pub mipmap_strategy: MipmapStrategy,
    /// How much the cutout strategies lift every mip level's alpha.
    pub alpha_cutoff_bias: f32,
}

impl Default for TextureMetadata {
    fn default() -> Self {
        Self {
            blur: false,
            clamp: false,
            mipmap_strategy: MipmapStrategy::Auto,
            alpha_cutoff_bias: 0.0,
        }
    }
}

impl TextureMetadata {
    /// Read the section from its object.
    ///
    /// A field of the wrong type is a rejection rather than a coercion, so a
    /// quoted number or a capitalised strategy name fails the whole section.
    /// A field the section does not know is ignored.
    pub fn parse(object: &JsonObject) -> Option<Self> {
        let mut parsed = Self::default();
        if object.has("blur") {
            parsed.blur = object.bool("blur")?;
        }
        if object.has("clamp") {
            parsed.clamp = object.bool("clamp")?;
        }
        if object.has("mipmap_strategy") {
            parsed.mipmap_strategy = MipmapStrategy::parse(&object.text("mipmap_strategy")?)?;
        }
        if object.has("alpha_cutoff_bias") {
            parsed.alpha_cutoff_bias = object.f32("alpha_cutoff_bias")?;
        }
        Some(parsed)
    }
}

/// The part of a sprite's `animation` section that decides what a still
/// atlas shows: the frame size and which frame comes first.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct AnimationFrames {
    pub frame_width: Option<i32>,
    pub frame_height: Option<i32>,
    /// The index the frame list starts on; zero when there is no list.
    pub first_frame: i32,
}

impl AnimationFrames {
    /// Read the section. A frame list entry is either an index or an object
    /// with one.
    pub fn parse(object: &JsonObject) -> Option<Self> {
        let mut parsed = Self::default();
        if object.has("width") {
            parsed.frame_width = Some(object.i32("width")?);
        }
        if object.has("height") {
            parsed.frame_height = Some(object.i32("height")?);
        }
        if let Some(first) = object.array("frames").and_then(|frames| frames.first()) {
            parsed.first_frame = match first {
                crate::json::JsonValue::Object(entry) => entry.i32("index")?,
                other => other.as_i32()?,
            };
        }
        Some(parsed)
    }

    /// The frame size for a sprite image: a missing side is the image's own,
    /// and with neither given the frame is the square of the shorter side.
    pub fn frame_size(&self, width: i32, height: i32) -> (i32, i32) {
        match (self.frame_width, self.frame_height) {
            (Some(frame_width), Some(frame_height)) => (frame_width, frame_height),
            (Some(frame_width), None) => (frame_width, height),
            (None, Some(frame_height)) => (width, frame_height),
            (None, None) => (width.min(height), width.min(height)),
        }
    }
}
