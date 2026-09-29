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
use crate::scene::{Mesh, Vertex};
use crate::texture_metadata::{AnimationFrames, TextureMetadata};
use voxel_core::identifier::Identifier;

/// Where model and texture files come from.
///
/// The client reads assets from a location the user names, so resolution is
/// written against this rather than against any bundled content.
pub trait AssetSource {
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

/// Load a model and its ancestors, then apply the reference's inheritance.
pub fn resolve_model(
    source: &impl AssetSource,
    name: &Identifier,
) -> Result<ResolvedModel, ResolveError> {
    let mut chain: Vec<CuboidModel> = Vec::new();
    let mut seen: Vec<Identifier> = Vec::new();
    let mut next = Some(name.clone());
    let mut generated = false;
    while let Some(current) = next {
        if seen.contains(&current) || chain.len() >= MAX_CHAIN {
            return Err(ResolveError::CyclicParent(current.as_string()));
        }
        if current.as_string() == GENERATED {
            // `ItemModelGenerator`: no geometry of its own, front lighting,
            // and its particle is its first layer.
            generated = true;
            chain.push(CuboidModel {
                parent: None,
                ambient_occlusion: None,
                gui_light: Some(GuiLight::Front),
                texture_slots: crate::model::TextureSlotsData::from_entries([(
                    "particle".to_owned(),
                    crate::model::SlotContents::parse_string("#layer0").expect("fixed reference"),
                )]),
                elements: None,
                display: None,
            });
            break;
        }
        let text = source
            .model(&current)
            .ok_or_else(|| ResolveError::MissingModel(current.as_string()))?;
        let parsed = parse_model(&text).map_err(|_| ResolveError::BadModel(current.as_string()))?;
        next = parsed.parent.clone();
        seen.push(current);
        chain.push(parsed);
    }

    // Texture maps are collected child first, which is the order the resolver
    // expects; it applies them root first so a child overrides its parent.
    let mut resolver = TextureSlotsResolver::new();
    for model in &chain {
        resolver.add_last(model.texture_slots.clone());
    }
    let textures = resolver.resolve();

    // Geometry, ambient occlusion and GUI light each take the first value set
    // walking up from the child.
    let elements = chain
        .iter()
        .find_map(|model| model.elements.clone())
        .unwrap_or_default();
    let ambient_occlusion = chain
        .iter()
        .find_map(|model| model.ambient_occlusion)
        .unwrap_or(true);
    let gui_light = chain
        .iter()
        .find_map(|model| model.gui_light)
        .unwrap_or(GuiLight::Side);

    let mut display = crate::model::ItemTransforms::default();
    for context in crate::model::DisplayContext::ALL {
        display.transforms[context as usize] = chain
            .iter()
            .find_map(|model| model.display.and_then(|transforms| transforms.get(context)));
    }

    Ok(ResolvedModel {
        textures,
        elements,
        ambient_occlusion,
        gui_light,
        display,
        generated,
    })
}

/// An atlas built from the sprites one model needs.
#[derive(Debug, Clone)]
pub struct BuiltAtlas {
    /// Atlas width in texels at level zero.
    pub width: u32,
    /// Atlas height in texels at level zero.
    pub height: u32,
    /// Row-major RGBA8 texels of level zero, kept for callers that take one
    /// level.
    pub rgba: Vec<u8>,
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

    /// Draw the current frame, or the blend towards the next one, into one
    /// atlas level, padding included. Returns the rectangle written: its
    /// corner and size in that level's texels.
    pub fn draw(&self, level: usize, target: &mut DecodedImage) -> (u32, u32, u32, u32) {
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
        let x = (self.x >> shift).saturating_sub(padding);
        let y = (self.y >> shift).saturating_sub(padding);
        blit_sprite(target, image, self.x >> shift, self.y >> shift, padding);
        (
            x,
            y,
            (image.width + padding * 2).min(target.width - x),
            (image.height + padding * 2).min(target.height - y),
        )
    }
}

/// How many mip levels a sprite set can actually carry.
///
/// A level halves both axes, so a sprite can only be halved as many times as
/// its size allows. The reference takes the smallest side across the whole set
/// and also the lowest set bit of every side, because a sprite whose side is
/// not a multiple of the level's texel size cannot be halved cleanly either.
/// Whichever is smaller decides, and the request is never exceeded.
pub fn effective_mip_level(sprites: &[(i32, i32)], max_mip_level: i32) -> i32 {
    let mut min_texel_size = i32::MAX;
    // Java masks a shift count to five bits, so an absurd request wraps here
    // rather than overflowing.
    let mut lowest_one_bit = 1_i32.wrapping_shl(max_mip_level as u32);
    for (width, height) in sprites {
        min_texel_size = min_texel_size.min(*width.min(height));
        let lowest = (width & width.wrapping_neg()).min(height & height.wrapping_neg());
        if lowest < lowest_one_bit {
            lowest_one_bit = lowest;
        }
    }
    let min_size = min_texel_size.min(lowest_one_bit);
    let min_power_of_two = voxel_core::math::log2(min_size);
    if min_power_of_two < max_mip_level {
        min_power_of_two
    } else {
        max_mip_level
    }
}

/// Pack every texture a resolved model uses into one atlas.
///
/// The placement comes from the pinned packer, so the layout is the one the
/// reference would produce for the same sprite set.
pub fn build_atlas(
    source: &impl AssetSource,
    model: &ResolvedModel,
    max_size: i32,
    mip_level: i32,
) -> Result<BuiltAtlas, ResolveError> {
    build_atlas_for(source, std::slice::from_ref(&model), max_size, mip_level)
}

/// Pack every texture a whole set of models uses into one atlas.
///
/// A world draws many models at once and they have to share one atlas, both
/// because the reference stitches one and because a draw call binds one
/// texture.
pub fn build_atlas_for(
    source: &impl AssetSource,
    models: &[&ResolvedModel],
    max_size: i32,
    mip_level: i32,
) -> Result<BuiltAtlas, ResolveError> {
    build_atlas_with(source, models, &[], max_size, mip_level)
}

/// Pack a set of models' textures together with sprites no model names,
/// such as the fluid sprites the fluid renderer draws with.
pub fn build_atlas_with(
    source: &impl AssetSource,
    models: &[&ResolvedModel],
    extra: &[Identifier],
    max_size: i32,
    mip_level: i32,
) -> Result<BuiltAtlas, ResolveError> {
    let mut wanted: BTreeMap<String, Material> = BTreeMap::new();
    for sprite in extra {
        wanted.insert(
            sprite.as_string(),
            Material {
                sprite: sprite.clone(),
                force_translucent: false,
            },
        );
    }
    for model in models {
        for element in &model.elements {
            for (_, face) in &element.faces {
                let material = model
                    .textures
                    .material(&face.texture)
                    .ok_or_else(|| ResolveError::UnresolvedTexture(face.texture.clone()))?;
                wanted.insert(material.sprite.as_string(), material.clone());
            }
        }
    }

    let mut decoded = BTreeMap::new();
    for (key, material) in &wanted {
        let bytes = source
            .texture(&material.sprite)
            .ok_or_else(|| ResolveError::BadTexture(key.clone()))?;
        let mut image = decode_rgba(&bytes).map_err(|_| ResolveError::BadTexture(key.clone()))?;
        let mut animated = None;
        if let Some(animation) = source.texture_animation(&material.sprite) {
            let strip = image;
            image = first_animation_frame(&strip, &animation)
                .ok_or_else(|| ResolveError::BadTexture(key.clone()))?;
            let per_row = strip.width / image.width;
            let count = per_row * (strip.height / image.height);
            if let Some(playback) = animation.playback(count as i32) {
                // Still sprites start on the list's first frame; animated
                // ones on the first frame that plays.
                image = first_animation_frame(
                    &strip,
                    &AnimationFrames {
                        first_frame: playback[0].0,
                        ..animation.clone()
                    },
                )
                .ok_or_else(|| ResolveError::BadTexture(key.clone()))?;
                animated = Some((strip, playback, animation.interpolate));
            }
        }
        let metadata = source
            .texture_metadata(&material.sprite)
            .unwrap_or_default();
        decoded.insert(key.clone(), (image, metadata, animated));
    }
    // The level is settled before packing, because the padding the packer
    // reserves is one texel of the deepest level on each side.
    let sizes: Vec<(i32, i32)> = decoded
        .values()
        .map(|(image, _, _)| (image.width as i32, image.height as i32))
        .collect();
    let mip_level = effective_mip_level(&sizes, mip_level.max(0));
    let mut stitcher = Stitcher::new(max_size, max_size, mip_level, 0);
    for material in wanted.values() {
        let Some((image, _, _)) = decoded.get(&material.sprite.as_string()) else {
            continue;
        };
        stitcher.register_sprite(SpriteEntry {
            width: image.width as i32,
            height: image.height as i32,
            name: material.sprite.clone(),
        });
    }
    stitcher.stitch().map_err(|_| ResolveError::AtlasFull)?;

    let width = stitcher.width().max(1) as u32;
    let height = stitcher.height().max(1) as u32;
    // Every sprite's levels are built first, because the level count is the
    // same for the whole atlas and each sprite's level N goes into atlas
    // level N.
    let level_count = mip_level.max(0) as u32;
    let mut sprites = BTreeMap::new();
    let mut animations = Vec::new();
    let mut levels: Vec<DecodedImage> = (0..=level_count)
        .map(|level| {
            let level_width = (width >> level).max(1);
            let level_height = (height >> level).max(1);
            DecodedImage {
                width: level_width,
                height: level_height,
                rgba: vec![0_u8; (level_width * level_height * 4) as usize],
            }
        })
        .collect();
    for placement in stitcher.placements() {
        let key = placement.name.as_string();
        let Some((image, metadata, animated)) = decoded.get(&key) else {
            continue;
        };
        let sprite = AtlasSprite::new(
            &placement,
            image.width as i32,
            image.height as i32,
            width as i32,
            height as i32,
        );
        let origin_x = placement.x as u32 + placement.padding as u32;
        let origin_y = placement.y as u32 + placement.padding as u32;
        let mips = if let Some((strip, playback, interpolate)) = animated {
            // The reference builds the levels of the whole strip, then cuts
            // each frame out of every level.
            let strip_mips = generate_mip_levels_with(
                placement.name.path(),
                strip,
                level_count,
                metadata.mipmap_strategy,
                metadata.alpha_cutoff_bias,
                compute_transparency(strip),
            );
            let per_row = strip.width / image.width;
            let mut frame_levels = BTreeMap::new();
            for &(index, _) in playback {
                frame_levels.entry(index).or_insert_with(|| {
                    let left = index as u32 % per_row * image.width;
                    let top = index as u32 / per_row * image.height;
                    strip_mips
                        .iter()
                        .enumerate()
                        .map(|(level, mip)| {
                            let shift = level as u32;
                            crop(
                                mip,
                                left >> shift,
                                top >> shift,
                                image.width >> shift,
                                image.height >> shift,
                            )
                        })
                        .collect::<Vec<_>>()
                });
            }
            let first = frame_levels[&playback[0].0].clone();
            animations.push(SpriteAnimation {
                name: key.clone(),
                x: origin_x,
                y: origin_y,
                padding: placement.padding.max(0) as u32,
                frame_levels,
                playback: playback.clone(),
                interpolate: *interpolate,
                frame: 0,
                sub_frame: 0,
                dirty: true,
            });
            first
        } else {
            generate_mip_levels_with(
                placement.name.path(),
                image,
                level_count,
                metadata.mipmap_strategy,
                metadata.alpha_cutoff_bias,
                compute_transparency(image),
            )
        };
        for (level, mip) in mips.iter().enumerate() {
            let shift = level as u32;
            blit_sprite(
                &mut levels[level],
                mip,
                (placement.x as u32 + placement.padding as u32) >> shift,
                (placement.y as u32 + placement.padding as u32) >> shift,
                placement.padding.max(0) as u32 >> shift,
            );
        }
        sprites.insert(key, [sprite.u0, sprite.v0, sprite.u1, sprite.v1]);
    }
    let rgba = levels[0].rgba.clone();

    Ok(BuiltAtlas {
        width,
        height,
        rgba,
        levels,
        sprites,
        animations,
    })
}

/// A rectangle of an image.
fn crop(image: &DecodedImage, left: u32, top: u32, width: u32, height: u32) -> DecodedImage {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for row in top..top + height {
        let start = ((row * image.width + left) * 4) as usize;
        rgba.extend_from_slice(&image.rgba[start..start + width as usize * 4]);
    }
    DecodedImage {
        width,
        height,
        rgba,
    }
}

/// Write one sprite level into one atlas level, extending its edge texels into
/// the padding around it.
///
/// The reference fills the atlas with a draw per sprite whose quad covers the
/// padded slot and whose texture coordinates run from `-padding/width` to
/// `1 + padding/width`, sampled at the nearest texel with clamping at the
/// sprite's edge. Working that through, atlas texel `i` of the slot reads
/// sprite texel `i - padding` clamped into range, so the padding carries the
/// edge texel repeated — which is the point, since it is what a sampler reads
/// when a face lands right on the sprite's border.
fn blit_sprite(
    target: &mut DecodedImage,
    sprite: &DecodedImage,
    origin_x: u32,
    origin_y: u32,
    padding: u32,
) {
    if sprite.width == 0 || sprite.height == 0 {
        return;
    }
    let start_x = origin_x.saturating_sub(padding);
    let start_y = origin_y.saturating_sub(padding);
    let span_x = sprite.width + padding * 2;
    let span_y = sprite.height + padding * 2;
    for row in 0..span_y {
        let target_y = start_y + row;
        if target_y >= target.height {
            break;
        }
        let source_y = row.saturating_sub(padding).min(sprite.height - 1);
        for column in 0..span_x {
            let target_x = start_x + column;
            if target_x >= target.width {
                break;
            }
            let source_x = column.saturating_sub(padding).min(sprite.width - 1);
            let from = ((source_y * sprite.width) + source_x) as usize * 4;
            let to = ((target_y * target.width) + target_x) as usize * 4;
            target.rgba[to..to + 4].copy_from_slice(&sprite.rgba[from..from + 4]);
        }
    }
}

/// Build the mesh for a resolved model against a packed atlas.
///
/// Corner positions, default UVs, element rotation, the variant's whole-model
/// rotation and UV rotation all come from the pinned paths; the only thing
/// added here is the per-face shade, which is this module's own and matches
/// nothing in the reference.
pub fn build_mesh(
    model: &ResolvedModel,
    atlas: &BuiltAtlas,
    model_state: Option<&crate::rotation::Matrix4>,
    tints: &[u32],
    uv_transform: impl Fn(voxel_core::direction::Direction) -> crate::rotation::FlatMatrix,
    occluded: impl Fn(voxel_core::direction::Direction) -> bool,
) -> Result<Mesh, ResolveError> {
    let mut mesh = Mesh::default();
    for element in &model.elements {
        for (facing, face) in &element.faces {
            // A face may name the side it hides behind. The reference sorts
            // quads into one bucket per such side plus one for the rest, and
            // skips a bucket when the neighbour on that side covers it.
            if let Some(cull) = face.cull_for_direction {
                // The side is named in the model's own frame, so the variant's
                // rotation has to be applied before the neighbour is asked.
                let cull = match model_state {
                    Some(state) => crate::rotation::rotate_direction(state, cull),
                    None => cull,
                };
                if occluded(cull) {
                    continue;
                }
            }
            let material = model
                .textures
                .material(&face.texture)
                .ok_or_else(|| ResolveError::UnresolvedTexture(face.texture.clone()))?;
            let key = material.sprite.as_string();
            let rect = atlas
                .sprites
                .get(&key)
                .ok_or_else(|| ResolveError::BadTexture(key.clone()))?;

            let mut positions = face_vertices(element.from, element.to, *facing);
            let uvs = face
                .uvs
                .unwrap_or_else(|| default_face_uv(element.from, element.to, *facing));

            // A face names a tint slot rather than a colour. A slot the
            // caller did not resolve leaves the face untinted, which is what
            // the reference does when a block has no source for that layer.
            let tint = usize::try_from(face.tint_index)
                .ok()
                .and_then(|slot| tints.get(slot).copied())
                .unwrap_or(0xFFFF_FFFF);
            let tint = [
                ((tint >> 16) & 0xFF) as f32 / 255.0,
                ((tint >> 8) & 0xFF) as f32 / 255.0,
                (tint & 0xFF) as f32 / 255.0,
                ((tint >> 24) & 0xFF) as f32 / 255.0,
            ];
            let face_uv = uv_transform(*facing);
            let base = mesh.vertices.len() as u32;
            let mut packed_uvs = [0_u64; 4];
            for (slot, position) in positions.iter_mut().enumerate() {
                let mut rotated = match &element.rotation {
                    Some(rotation) => {
                        rotate_vertex(*position, rotation.origin, &rotation.transform)
                    }
                    None => *position,
                };
                // The variant's rotation comes after the element's and turns
                // about the block's centre, as the reference orders them.
                if let Some(state) = model_state {
                    rotated = rotate_vertex(rotated, BLOCK_CENTRE, state);
                }
                *position = rotated;
                // UV lock turns the texture coordinate back by as much as the
                // model turned, so the texture keeps its world orientation.
                // Without it the transform is identity and the coordinate
                // passes through.
                let (u, v) = face_vertex_uv(uvs, face.rotation, slot as u32);
                let (u, v) = crate::rotation::transform_face_uv(&face_uv, u, v);
                packed_uvs[slot] = (u64::from(u.to_bits()) << 32) | u64::from(v.to_bits());
            }
            // FaceBakery: the quad faces whichever side its normal is closest
            // to, and without an element rotation its corners are put back in
            // that side's order.
            let direction = crate::geometry::closest_direction(crate::geometry::triangle_normal(
                positions[0],
                positions[1],
                positions[2],
            ));
            if element.rotation.is_none()
                && let Some(direction) = direction
            {
                let _ = crate::geometry::recalculate_winding(
                    &mut positions,
                    &mut packed_uvs,
                    direction,
                );
            }
            let direction = direction.unwrap_or(voxel_core::direction::Direction::Up);
            let shade = if element.shade {
                face_shade(direction)
            } else {
                1.0
            };
            for (position, packed) in positions.into_iter().zip(packed_uvs) {
                let u = f32::from_bits((packed >> 32) as u32);
                let v = f32::from_bits(packed as u32);
                mesh.vertices.push(Vertex {
                    position,
                    uv: [
                        rect[0] + (rect[2] - rect[0]) * u,
                        rect[1] + (rect[3] - rect[1]) * v,
                    ],
                    shade,
                    tint,
                    light: crate::scene::FULL_BRIGHT,
                });
            }
            mesh.quads.push(crate::scene::QuadInfo {
                direction,
                cull: face.cull_for_direction.map(|cull| match model_state {
                    Some(state) => crate::rotation::rotate_direction(state, cull),
                    None => cull,
                }),
                shade: element.shade,
            });
            mesh.indices
                .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    Ok(mesh)
}

/// Flat per-face shading. Not the reference's lighting; a placeholder that
/// keeps the faces of a block distinguishable.
fn face_shade(facing: voxel_core::direction::Direction) -> f32 {
    use voxel_core::direction::Direction;
    match facing {
        Direction::Up => 1.0,
        Direction::Down => 0.5,
        Direction::North | Direction::South => 0.8,
        Direction::West | Direction::East => 0.6,
    }
}

/// Cut the frame an animation starts on out of its strip. Frames run left to
/// right, then down, as many to a row as fit across the image.
fn first_animation_frame(
    image: &DecodedImage,
    animation: &AnimationFrames,
) -> Option<DecodedImage> {
    let (width, height) = (image.width as i32, image.height as i32);
    let (frame_width, frame_height) = animation.frame_size(width, height);
    if frame_width <= 0
        || frame_height <= 0
        || width % frame_width != 0
        || height % frame_height != 0
    {
        return None;
    }
    let per_row = width / frame_width;
    let count = per_row * (height / frame_height);
    if animation.first_frame < 0 || animation.first_frame >= count {
        return None;
    }
    let left = (animation.first_frame % per_row * frame_width) as usize;
    let top = (animation.first_frame / per_row * frame_height) as usize;
    let mut rgba = Vec::with_capacity((frame_width * frame_height * 4) as usize);
    for row in 0..frame_height as usize {
        let start = ((top + row) * image.width as usize + left) * 4;
        rgba.extend_from_slice(&image.rgba[start..start + frame_width as usize * 4]);
    }
    Some(DecodedImage {
        width: frame_width as u32,
        height: frame_height as u32,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
