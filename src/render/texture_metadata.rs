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
