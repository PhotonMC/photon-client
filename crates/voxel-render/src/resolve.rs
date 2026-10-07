//! Resolving a model by name: walking its parent chain and building its mesh.
//!
//! The pieces this composes are pinned individually - model file parsing,
//! texture slot resolution, face geometry, element rotation, UV rotation and
//! atlas packing. The three "walk up until something is set" rules here are
//! read straight from the reference and are not separately pinned, because
//! pinning them needs the reference's resource-manager machinery; they are
//! noted as such in the record.

use std::collections::BTreeMap;

use crate::atlas::{AtlasSprite, SpriteEntry, Stitcher};
use crate::geometry::{default_face_uv, face_vertex_uv, face_vertices};
use crate::mipmap::{compute_transparency, generate_mip_levels_with};
use crate::model::{
    CuboidModel, GuiLight, Material, ModelElement, TextureSlots, TextureSlotsResolver, parse_model,
};
use crate::png::{DecodedImage, decode_rgba};
use crate::rotation::rotate_vertex;
use crate::scene::{BlockVertex, Mesh};
use crate::texture_metadata::{AnimationFrames, TextureMetadata};
use voxel_core::identifier::Identifier;

/// Where model and texture files come from.
///
/// The client reads assets from a location the user names, so resolution is
/// written against this rather than against any bundled content.
pub trait AssetSource: Sync {
    /// The text of one model, by the name a model file uses for it.
    ///
    /// The name is what appears in a `parent` field, so the source owns the
    /// mapping to a file; resolution never builds paths itself.
    fn model(&self, name: &Identifier) -> Option<String>;
    /// The bytes of one texture, by the name a texture slot uses for it.
    fn texture(&self, name: &Identifier) -> Option<Vec<u8>>;
    /// The text of one block's blockstate file, by the block's name.
    ///
    /// A block that has none cannot be drawn, which is a caller's concern
    /// rather than an error here.
    fn block_state(&self, _name: &Identifier) -> Option<String> {
        None
    }

    /// The text of one item's client item file (`items/<name>.json`).
    fn item(&self, _name: &Identifier) -> Option<String> {
        None
    }

    /// The texture section of one sprite's metadata, if the sprite has one.
    ///
    /// Absent metadata is not an error: every field has a default, and the
    /// default mip strategy decides itself from the sprite's texels.
    fn texture_metadata(&self, _name: &Identifier) -> Option<TextureMetadata> {
        None
    }

    /// The animation section of one sprite's metadata, if it is animated.
    ///
    /// The atlas holds one frame of an animated sprite; the strip of frames
    /// in its image never goes into the atlas whole.
    fn texture_animation(&self, _name: &Identifier) -> Option<AnimationFrames> {
        None
    }
}

/// What resolution could not find.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// A model in the chain is not in the asset source.
    MissingModel(String),
    /// A model file did not parse.
    BadModel(String),
    /// The chain refers back to a model already in it.
    CyclicParent(String),
    /// A face names a texture slot nothing resolves.
    UnresolvedTexture(String),
    /// A texture file is missing or did not decode.
    BadTexture(String),
    /// The sprites would not fit an atlas of the given size.
    AtlasFull,
}

/// The point a variant's rotation turns about.
const BLOCK_CENTRE: [f32; 3] = [0.5, 0.5, 0.5];

/// How deep a parent chain may go before it is treated as broken.
const MAX_CHAIN: usize = 64;

/// One model and everything inherited from its ancestors.
#[derive(Debug, Clone)]
pub struct ResolvedModel {
    /// Texture slots resolved across the whole chain.
    pub textures: TextureSlots,
    /// The nearest ancestor's geometry, or none if no one declares any.
    pub elements: Vec<ModelElement>,
    /// Ambient occlusion, defaulting to on.
    pub ambient_occlusion: bool,
    /// GUI lighting, defaulting to side.
    pub gui_light: GuiLight,
    /// Each display context's transform, the nearest one set walking up
    /// from the child (`ResolvedModel.findTopTransform`).
    pub display: crate::model::ItemTransforms,
    /// Whether the chain ends at `builtin/generated`: the geometry is the
    /// item model generator's extruded layers.
    pub generated: bool,
}

/// The item model generator's own model, which item models reach as
/// `builtin/generated`.
const GENERATED: &str = "minecraft:builtin/generated";

/// Every model file read so far, parsed, so a parent many models share is
/// read once (`ModelDiscovery` loads each model once too).
#[derive(Default)]
pub struct ModelCache {
    parsed: std::collections::HashMap<String, Result<std::sync::Arc<CuboidModel>, ResolveError>>,
}

impl ModelCache {
    /// A model file, parsed, read the first time it is asked for.
    fn load(
        &mut self,
        source: &impl AssetSource,
        name: &Identifier,
    ) -> Result<std::sync::Arc<CuboidModel>, ResolveError> {
        self.parsed
            .entry(name.as_string())
            .or_insert_with(|| {
                let text = source
                    .model(name)
                    .ok_or_else(|| ResolveError::MissingModel(name.as_string()))?;
                parse_model(&text)
                    .map(std::sync::Arc::new)
                    .map_err(|_| ResolveError::BadModel(name.as_string()))
            })
            .clone()
    }

    /// Read and parse every named model and every parent they reach, the
    /// files of each generation in parallel, so resolving them afterwards
    /// reads nothing.
    pub fn prefetch(
        &mut self,
        source: &impl AssetSource,
        names: impl IntoIterator<Item = Identifier>,
    ) {
        use rayon::prelude::*;
        let mut frontier: Vec<Identifier> = Vec::new();
        let mut queued = std::collections::HashSet::new();
        for name in names {
            let key = name.as_string();
            if key != GENERATED && !self.parsed.contains_key(&key) && queued.insert(key) {
                frontier.push(name);
            }
        }
        while !frontier.is_empty() {
            let loaded: Vec<_> = frontier
                .par_iter()
                .map(|name| {
                    let parsed = source
                        .model(name)
                        .ok_or_else(|| ResolveError::MissingModel(name.as_string()))
                        .and_then(|text| {
                            parse_model(&text)
                                .map(std::sync::Arc::new)
                                .map_err(|_| ResolveError::BadModel(name.as_string()))
                        });
                    (name.as_string(), parsed)
                })
                .collect();
            let mut next = Vec::new();
            for (key, parsed) in loaded {
                if let Ok(model) = &parsed
                    && let Some(parent) = &model.parent
                {
                    let parent_key = parent.as_string();
                    if parent_key != GENERATED
                        && !self.parsed.contains_key(&parent_key)
                        && queued.insert(parent_key)
                    {
                        next.push(parent.clone());
                    }
                }
                self.parsed.insert(key, parsed);
            }
            frontier = next;
        }
    }

    /// Load a model and its ancestors, then apply the reference's
    /// inheritance.
    pub fn resolve(
        &mut self,
        source: &impl AssetSource,
        name: &Identifier,
    ) -> Result<ResolvedModel, ResolveError> {
        let mut chain: Vec<std::sync::Arc<CuboidModel>> = Vec::new();
        let mut seen: Vec<Identifier> = Vec::new();
        let mut next = Some(name.clone());
        let mut generated = false;
        while let Some(current) = next {
            if seen.contains(&current) || chain.len() >= MAX_CHAIN {
                return Err(ResolveError::CyclicParent(current.as_string()));
            }
            if current.as_string() == GENERATED {
                // `ItemModelGenerator`: no geometry of its own, front
                // lighting, and its particle is its first layer.
                generated = true;
                chain.push(std::sync::Arc::new(CuboidModel {
                    parent: None,
                    ambient_occlusion: None,
                    gui_light: Some(GuiLight::Front),
                    texture_slots: crate::model::TextureSlotsData::from_entries([(
                        "particle".to_owned(),
                        crate::model::SlotContents::parse_string("#layer0")
                            .expect("fixed reference"),
                    )]),
                    elements: None,
                    display: None,
                }));
                break;
            }
            let parsed = self.load(source, &current)?;
            next = parsed.parent.clone();
            seen.push(current);
            chain.push(parsed);
        }
        Ok(inherit(&chain, generated))
    }
}

/// An atlas built from the sprites one model needs.
#[derive(Debug, Clone)]
pub struct BuiltAtlas {
    /// Atlas width in texels at level zero.
    pub width: u32,
    /// Atlas height in texels at level zero.
    pub height: u32,
    /// Every mip level, level zero first, each half the previous one.
    pub levels: Vec<DecodedImage>,
    /// Where each sprite landed, in normalised coordinates.
    pub sprites: BTreeMap<String, [f32; 4]>,
    /// The animated sprites, each able to draw its current frame into the
    /// levels.
    pub animations: Vec<SpriteAnimation>,
}

/// One animated sprite in an atlas: `SpriteContents.AnimationState`.
///
/// Each frame's mip levels are cut from the levels built for the whole
/// strip. Every client tick advances the frame; a sprite is drawn into the
/// atlas when its frame changed, or every tick if it blends frames.
#[derive(Debug, Clone)]
pub struct SpriteAnimation {
    pub name: String,
    /// The sprite's corner in level zero, without its padding.
    x: u32,
    y: u32,
    padding: u32,
    /// Each strip frame's levels, by strip index.
    frame_levels: BTreeMap<i32, Vec<DecodedImage>>,
    /// The frames that play: a strip index and a time in ticks.
    playback: Vec<(i32, i32)>,
    interpolate: bool,
    frame: usize,
    sub_frame: i32,
    dirty: bool,
}

impl SpriteAnimation {
    /// `AnimationState.tick`.
    pub fn tick(&mut self) {
        self.sub_frame += 1;
        self.dirty = false;
        let (old_index, time) = self.playback[self.frame];
        if self.sub_frame >= time {
            self.frame = (self.frame + 1) % self.playback.len();
            self.sub_frame = 0;
            if self.playback[self.frame].0 != old_index {
                self.dirty = true;
            }
        }
    }

    /// `AnimationState.needsToDraw`.
    pub fn needs_to_draw(&self) -> bool {
        self.interpolate || self.dirty
    }

    /// Draw the current frame, or the blend towards the next one, for one
    /// atlas level of `level_size` texels, padding included. Returns the
    /// rectangle to write: its corner in that level's texels and its image.
    pub fn draw(&self, level: usize, level_size: (u32, u32)) -> (u32, u32, DecodedImage) {
        let (index, time) = self.playback[self.frame];
        let current = &self.frame_levels[&index][level];
        let blended;
        let image = if self.interpolate {
            let next =
                &self.frame_levels[&self.playback[(self.frame + 1) % self.playback.len()].0][level];
            // The progress goes through the reference's draw as thousandths.
            let progress = ((self.sub_frame as f32 / time as f32) * 1000.0) as i32 as f32 / 1000.0;
            blended = DecodedImage {
                width: current.width,
                height: current.height,
                rgba: current
                    .rgba
                    .iter()
                    .zip(&next.rgba)
                    .map(|(&a, &b)| {
                        let (a, b) = (f32::from(a) / 255.0, f32::from(b) / 255.0);
                        ((a * (1.0 - progress) + b * progress) * 255.0).round() as u8
                    })
                    .collect(),
            };
            &blended
        } else {
            current
        };
        let shift = level as u32;
        let padding = self.padding >> shift;
        let origin = (self.x >> shift, self.y >> shift);
        let x = origin.0.saturating_sub(padding);
        let y = origin.1.saturating_sub(padding);
        let width = (image.width + padding * 2).min(level_size.0 - x);
        let height = (image.height + padding * 2).min(level_size.1 - y);
        let mut region = DecodedImage {
            width,
            height,
            rgba: vec![0; width as usize * height as usize * 4],
        };
        blit_sprite(&mut region, image, origin.0 - x, origin.1 - y, padding);
        (x, y, region)
    }
}

/// A decoded sprite: its image (the first frame of an animated strip), its
/// metadata, and an animated strip with its playback.
type DecodedSprite = (
    DecodedImage,
    crate::texture_metadata::TextureMetadata,
    Option<(DecodedImage, Vec<(i32, i32)>, bool)>,
);

/// One sprite's levels, ready to write into the atlas.
struct BuiltSprite {
    placement: crate::atlas::SpritePlacement,
    key: String,
    rect: [f32; 4],
    mips: Vec<DecodedImage>,
    animation: Option<SpriteAnimation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A redrawn slot is the same texels whether it is drawn into a copy of
    /// the atlas and cut out, or built on its own, edge clipping included.
    #[test]
    fn an_animation_slot_is_the_same_cut_from_the_atlas_or_built_alone() {
        let frame = |shade: u8| DecodedImage {
            width: 3,
            height: 3,
            rgba: (0..9).flat_map(|i| [shade, i * 20, 7, 255]).collect(),
        };
        for (x, y, padding) in [(4, 4, 2), (0, 1, 2), (6, 6, 2)] {
            let animation = SpriteAnimation {
                name: String::new(),
                x,
                y,
                padding,
                frame_levels: BTreeMap::from([(0, vec![frame(40)]), (1, vec![frame(200)])]),
                playback: vec![(0, 2), (1, 2)],
                interpolate: true,
                frame: 0,
                sub_frame: 1,
                dirty: true,
            };
            let mut atlas = DecodedImage {
                width: 8,
                height: 8,
                rgba: vec![9; 8 * 8 * 4],
            };
            let image = &animation.frame_levels[&0][0];
            let (index, time) = animation.playback[0];
            assert_eq!((index, time), (0, 2));
            // What the whole-atlas way writes for a blended frame.
            let next = &animation.frame_levels[&1][0];
            let progress = ((1.0_f32 / 2.0) * 1000.0) as i32 as f32 / 1000.0;
            let blended = DecodedImage {
                width: 3,
                height: 3,
                rgba: image
                    .rgba
                    .iter()
                    .zip(&next.rgba)
                    .map(|(&a, &b)| {
                        let (a, b) = (f32::from(a) / 255.0, f32::from(b) / 255.0);
                        ((a * (1.0 - progress) + b * progress) * 255.0).round() as u8
                    })
                    .collect(),
            };
            blit_sprite(&mut atlas, &blended, x, y, padding);
            let (left, top, alone) = animation.draw(0, (8, 8));
            assert_eq!(
                (left, top),
                (x.saturating_sub(padding), y.saturating_sub(padding))
            );
            for row in 0..alone.height {
                let from = (((top + row) * 8 + left) * 4) as usize;
                let cut = &atlas.rgba[from..from + alone.width as usize * 4];
                let own =
                    &alone.rgba[(row * alone.width * 4) as usize..][..alone.width as usize * 4];
                assert_eq!(cut, own, "row {row} at {x},{y}");
            }
        }
    }

    #[test]
    fn an_animated_strip_contributes_its_first_frame() {
        // Three 2x2 frames stacked, each a solid shade, the second first.
        let mut rgba = Vec::new();
        for shade in [10_u8, 20, 30] {
            for _ in 0..4 {
                rgba.extend_from_slice(&[shade, shade, shade, 255]);
            }
        }
        let strip = DecodedImage {
            width: 2,
            height: 6,
            rgba,
        };
        let square = AnimationFrames::default();
        let first = first_animation_frame(&strip, &square).expect("frame zero");
        assert_eq!((first.width, first.height), (2, 2));
        assert!(first.rgba.chunks(4).all(|texel| texel[0] == 10));
        let listed = AnimationFrames {
            first_frame: 1,
            ..AnimationFrames::default()
        };
        let second = first_animation_frame(&strip, &listed).expect("frame one");
        assert!(second.rgba.chunks(4).all(|texel| texel[0] == 20));
        let outside = AnimationFrames {
            first_frame: 3,
            ..AnimationFrames::default()
        };
        assert!(first_animation_frame(&strip, &outside).is_none());
    }

    #[test]
    fn frame_size_falls_back_side_by_side() {
        let neither = AnimationFrames::default();
        assert_eq!(neither.frame_size(16, 288), (16, 16));
        let wide = AnimationFrames {
            frame_width: Some(32),
            ..AnimationFrames::default()
        };
        assert_eq!(wide.frame_size(64, 32), (32, 32));
        let tall = AnimationFrames {
            frame_height: Some(8),
            ..AnimationFrames::default()
        };
        assert_eq!(tall.frame_size(16, 64), (16, 8));
    }

    /// A solid image with one distinguishable column, so a shift shows up.
    fn ramp(width: u32, height: u32) -> DecodedImage {
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                rgba.extend_from_slice(&[x as u8, y as u8, 7, 255]);
            }
        }
        DecodedImage {
            width,
            height,
            rgba,
        }
    }

    fn texel(image: &DecodedImage, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * image.width) + x) as usize * 4;
        image.rgba[index..index + 4].try_into().unwrap()
    }

    #[test]
    fn blit_places_the_sprite_past_its_padding() {
        let sprite = ramp(4, 4);
        let mut target = DecodedImage {
            width: 16,
            height: 16,
            rgba: vec![0; 16 * 16 * 4],
        };
        blit_sprite(&mut target, &sprite, 5, 5, 2);
        assert_eq!(texel(&target, 5, 5), [0, 0, 7, 255]);
        assert_eq!(texel(&target, 8, 8), [3, 3, 7, 255]);
    }

    #[test]
    fn blit_repeats_the_edge_texel_through_the_padding() {
        let sprite = ramp(4, 4);
        let mut target = DecodedImage {
            width: 16,
            height: 16,
            rgba: vec![0; 16 * 16 * 4],
        };
        blit_sprite(&mut target, &sprite, 5, 5, 2);
        // Left of the sprite the first column repeats, right of it the last.
        assert_eq!(texel(&target, 3, 5), [0, 0, 7, 255]);
        assert_eq!(texel(&target, 4, 5), [0, 0, 7, 255]);
        assert_eq!(texel(&target, 9, 5), [3, 0, 7, 255]);
        assert_eq!(texel(&target, 10, 5), [3, 0, 7, 255]);
        // The corner carries the corner texel, not a blend of two edges.
        assert_eq!(texel(&target, 3, 3), [0, 0, 7, 255]);
        assert_eq!(texel(&target, 10, 10), [3, 3, 7, 255]);
        // One texel further out is untouched.
        assert_eq!(texel(&target, 2, 5), [0, 0, 0, 0]);
    }

    #[test]
    fn mip_level_is_clamped_by_the_smallest_sprite() {
        // Four levels need a side of at least sixteen.
        assert_eq!(effective_mip_level(&[(16, 16)], 4), 4);
        assert_eq!(effective_mip_level(&[(8, 8)], 4), 3);
        assert_eq!(effective_mip_level(&[(16, 16), (4, 4)], 4), 2);
        // A side that is not a clean multiple limits it by its lowest set bit,
        // not by its size.
        assert_eq!(effective_mip_level(&[(48, 48)], 4), 4);
        assert_eq!(effective_mip_level(&[(24, 24)], 4), 3);
        assert_eq!(effective_mip_level(&[(12, 12)], 4), 2);
        // An empty set has nothing to limit it.
        assert_eq!(effective_mip_level(&[], 4), 4);
        // The request is a ceiling, never raised.
        assert_eq!(effective_mip_level(&[(256, 256)], 2), 2);
    }
}

mod sprite;
pub use sprite::*;
mod tint;
pub use tint::*;
