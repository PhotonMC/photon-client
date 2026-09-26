//! Resolving a model by name: walking its parent chain and building its mesh.
//!
//! The pieces this composes are pinned individually - model file parsing,
//! texture slot resolution, face geometry, element rotation, UV rotation and
//! atlas packing. The three "walk up until something is set" rules here are
//! read straight from the reference and are not separately pinned, because
//! pinning them needs the reference's resource-manager machinery; they are
//! noted as such in the record.

use std::collections::BTreeMap;

use crate::identifier::Identifier;
use crate::render::atlas::{AtlasSprite, SpriteEntry, Stitcher};
use crate::render::geometry::{default_face_uv, face_vertex_uv, face_vertices};
use crate::render::mipmap::{compute_transparency, generate_mip_levels_with};
use crate::render::model::{
    CuboidModel, GuiLight, Material, ModelElement, TextureSlots, TextureSlotsResolver, parse_model,
};
use crate::render::png::{DecodedImage, decode_rgba};
use crate::render::rotation::rotate_vertex;
use crate::render::scene::{Mesh, Vertex};
use crate::render::texture_metadata::TextureMetadata;

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

    /// The texture section of one sprite's metadata, if the sprite has one.
    ///
    /// Absent metadata is not an error: every field has a default, and the
    /// default mip strategy decides itself from the sprite's texels.
    fn texture_metadata(&self, _name: &Identifier) -> Option<TextureMetadata> {
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
}

/// Load a model and its ancestors, then apply the reference's inheritance.
pub fn resolve_model(
    source: &impl AssetSource,
    name: &Identifier,
) -> Result<ResolvedModel, ResolveError> {
    let mut chain: Vec<CuboidModel> = Vec::new();
    let mut seen: Vec<Identifier> = Vec::new();
    let mut next = Some(name.clone());
    while let Some(current) = next {
        if seen.contains(&current) || chain.len() >= MAX_CHAIN {
            return Err(ResolveError::CyclicParent(current.as_string()));
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

    Ok(ResolvedModel {
        textures,
        elements,
        ambient_occlusion,
        gui_light,
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
    let min_power_of_two = crate::math::log2(min_size);
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
    let mut wanted: BTreeMap<String, Material> = BTreeMap::new();
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
        let image = decode_rgba(&bytes).map_err(|_| ResolveError::BadTexture(key.clone()))?;
        let metadata = source
            .texture_metadata(&material.sprite)
            .unwrap_or_default();
        decoded.insert(key.clone(), (image, metadata));
    }
    // The level is settled before packing, because the padding the packer
    // reserves is one texel of the deepest level on each side.
    let sizes: Vec<(i32, i32)> = decoded
        .values()
        .map(|(image, _)| (image.width as i32, image.height as i32))
        .collect();
    let mip_level = effective_mip_level(&sizes, mip_level.max(0));
    let mut stitcher = Stitcher::new(max_size, max_size, mip_level, 0);
    for material in wanted.values() {
        let Some((image, _)) = decoded.get(&material.sprite.as_string()) else {
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
        let Some((image, metadata)) = decoded.get(&key) else {
            continue;
        };
        let sprite = AtlasSprite::new(
            &placement,
            image.width as i32,
            image.height as i32,
            width as i32,
            height as i32,
        );
        let mips = generate_mip_levels_with(
            placement.name.path(),
            image,
            level_count,
            metadata.mipmap_strategy,
            metadata.alpha_cutoff_bias,
            compute_transparency(image),
        );
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
    })
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
    model_state: Option<&crate::render::rotation::Matrix4>,
    tints: &[u32],
    uv_transform: impl Fn(crate::direction::Direction) -> crate::render::rotation::FlatMatrix,
    occluded: impl Fn(crate::direction::Direction) -> bool,
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
                    Some(state) => crate::render::rotation::rotate_direction(state, cull),
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

            let positions = face_vertices(element.from, element.to, *facing);
            let uvs = face
                .uvs
                .unwrap_or_else(|| default_face_uv(element.from, element.to, *facing));
            let shade = if element.shade {
                face_shade(*facing)
            } else {
                1.0
            };
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
            for (slot, position) in positions.into_iter().enumerate() {
                let mut rotated = match &element.rotation {
                    Some(rotation) => rotate_vertex(position, rotation.origin, &rotation.transform),
                    None => position,
                };
                // The variant's rotation comes after the element's and turns
                // about the block's centre, as the reference orders them.
                if let Some(state) = model_state {
                    rotated = rotate_vertex(rotated, BLOCK_CENTRE, state);
                }
                // UV lock turns the texture coordinate back by as much as the
                // model turned, so the texture keeps its world orientation.
                // Without it the transform is identity and the coordinate
                // passes through.
                let (u, v) = face_vertex_uv(uvs, face.rotation, slot as u32);
                let (u, v) = crate::render::rotation::transform_face_uv(&face_uv, u, v);
                mesh.vertices.push(Vertex {
                    position: rotated,
                    uv: [
                        rect[0] + (rect[2] - rect[0]) * u,
                        rect[1] + (rect[3] - rect[1]) * v,
                    ],
                    shade,
                    tint,
                });
            }
            mesh.indices
                .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    Ok(mesh)
}

/// Flat per-face shading. Not the reference's lighting; a placeholder that
/// keeps the faces of a block distinguishable.
fn face_shade(facing: crate::direction::Direction) -> f32 {
    use crate::direction::Direction;
    match facing {
        Direction::Up => 1.0,
        Direction::Down => 0.5,
        Direction::North | Direction::South => 0.8,
        Direction::West | Direction::East => 0.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
