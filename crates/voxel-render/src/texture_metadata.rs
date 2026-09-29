//! The `texture` section of a sprite's metadata file.
//!
//! Every field is optional, so what an absent field falls back to is most of
//! what this module is: the mip strategy defaults to deciding itself from the
//! sprite's pixels, and the cutout bias to nothing.

use crate::mipmap::MipmapStrategy;
use voxel_core::json::JsonObject;

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

/// A sprite's `animation` section: the frame size, the frame list with each
/// frame's time, and whether frames blend into the next.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AnimationFrames {
    pub frame_width: Option<i32>,
    pub frame_height: Option<i32>,
    /// The index the frame list starts on; zero when there is no list.
    pub first_frame: i32,
    /// Ticks a frame shows for when it gives no time of its own.
    pub frame_time: i32,
    pub interpolate: bool,
    /// The listed frames, each an index and an optional time, or none when
    /// the section lists none and every frame of the strip plays in order.
    pub frames: Option<Vec<(i32, Option<i32>)>>,
}

impl Default for AnimationFrames {
    fn default() -> Self {
        Self {
            frame_width: None,
            frame_height: None,
            first_frame: 0,
            frame_time: 1,
            interpolate: false,
            frames: None,
        }
    }
}

impl AnimationFrames {
    /// Read the section. A frame list entry is either an index or an object
    /// with one and, optionally, a time.
    pub fn parse(object: &JsonObject) -> Option<Self> {
        let mut parsed = Self::default();
        if object.has("width") {
            parsed.frame_width = Some(object.i32("width")?);
        }
        if object.has("height") {
            parsed.frame_height = Some(object.i32("height")?);
        }
        if object.has("frametime") {
            parsed.frame_time = object.i32("frametime")?;
        }
        if object.has("interpolate") {
            parsed.interpolate = object.bool("interpolate")?;
        }
        if let Some(frames) = object.array("frames") {
            let mut listed = Vec::with_capacity(frames.len());
            for frame in frames {
                listed.push(match frame {
                    voxel_core::json::JsonValue::Object(entry) => (
                        entry.i32("index")?,
                        if entry.has("time") {
                            Some(entry.i32("time")?)
                        } else {
                            None
                        },
                    ),
                    other => (other.as_i32()?, None),
                });
            }
            if let Some(&(first, _)) = listed.first() {
                parsed.first_frame = first;
            }
            parsed.frames = Some(listed);
        }
        Some(parsed)
    }

    /// `SpriteContents.createAnimatedTexture`: the frames that play, each
    /// an index into the strip and a time in ticks. Without a list every
    /// frame of the strip plays for the default time; a listed frame with a
    /// time that is not positive or an index outside the strip is dropped.
    /// Fewer than two frames is no animation.
    pub fn playback(&self, frame_count: i32) -> Option<Vec<(i32, i32)>> {
        let frames: Vec<(i32, i32)> = match &self.frames {
            None => (0..frame_count)
                .map(|index| (index, self.frame_time))
                .collect(),
            Some(listed) => listed
                .iter()
                .map(|&(index, time)| (index, time.unwrap_or(self.frame_time)))
                .filter(|&(index, time)| time > 0 && index >= 0 && index < frame_count)
                .collect(),
        };
        (frames.len() > 1).then_some(frames)
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
