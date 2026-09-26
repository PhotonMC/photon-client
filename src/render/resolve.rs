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
use crate::render::atlas::{SpriteEntry, Stitcher};
use crate::render::geometry::{default_face_uv, face_vertex_uv, face_vertices};
use crate::render::model::{
    CuboidModel, GuiLight, Material, ModelElement, TextureSlots, TextureSlotsResolver, parse_model,
};
use crate::render::png::decode_rgba;
use crate::render::rotation::rotate_vertex;
use crate::render::scene::{Mesh, Vertex};

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
        let parsed =
            parse_model(&text).map_err(|_| ResolveError::BadModel(current.as_string()))?;
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
    /// Atlas width in texels.
    pub width: u32,
    /// Atlas height in texels.
    pub height: u32,
    /// Row-major RGBA8 texels.
    pub rgba: Vec<u8>,
    /// Where each sprite landed, in normalised coordinates.
    pub sprites: BTreeMap<String, [f32; 4]>,
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
    let mut wanted: BTreeMap<String, Material> = BTreeMap::new();
    for element in &model.elements {
        for (_, face) in &element.faces {
            let material = model
                .textures
                .material(&face.texture)
                .ok_or_else(|| ResolveError::UnresolvedTexture(face.texture.clone()))?;
            wanted.insert(material.sprite.as_string(), material.clone());
        }
    }

    let mut decoded = BTreeMap::new();
    let mut stitcher = Stitcher::new(max_size, max_size, mip_level, 0);
    for (key, material) in &wanted {
        let bytes = source
            .texture(&material.sprite)
            .ok_or_else(|| ResolveError::BadTexture(key.clone()))?;
        let image = decode_rgba(&bytes).map_err(|_| ResolveError::BadTexture(key.clone()))?;
        stitcher.register_sprite(SpriteEntry {
            width: image.width as i32,
            height: image.height as i32,
            name: material.sprite.clone(),
        });
        decoded.insert(key.clone(), image);
    }
    stitcher.stitch().map_err(|_| ResolveError::AtlasFull)?;

    let width = stitcher.width().max(1) as u32;
    let height = stitcher.height().max(1) as u32;
    let mut rgba = vec![0_u8; (width * height * 4) as usize];
    let mut sprites = BTreeMap::new();
    for placement in stitcher.placements() {
        let key = placement.name.as_string();
        let Some(image) = decoded.get(&key) else {
            continue;
        };
        for row in 0..image.height {
            let target = ((placement.y as u32 + row) * width + placement.x as u32) as usize * 4;
            let source_row = (row * image.width) as usize * 4;
            let span = (image.width * 4) as usize;
            rgba[target..target + span]
                .copy_from_slice(&image.rgba[source_row..source_row + span]);
        }
        sprites.insert(
            key,
            [
                placement.x as f32 / width as f32,
                placement.y as f32 / height as f32,
                (placement.x as u32 + image.width) as f32 / width as f32,
                (placement.y as u32 + image.height) as f32 / height as f32,
            ],
        );
    }

    Ok(BuiltAtlas {
        width,
        height,
        rgba,
        sprites,
    })
}

/// Build the mesh for a resolved model against a packed atlas.
///
/// Corner positions, default UVs, element rotation and UV rotation all come
/// from the pinned paths; the only thing added here is the per-face shade,
/// which is this module's own and matches nothing in the reference.
pub fn build_mesh(model: &ResolvedModel, atlas: &BuiltAtlas) -> Result<Mesh, ResolveError> {
    let mut mesh = Mesh::default();
    for element in &model.elements {
        for (facing, face) in &element.faces {
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
            let base = mesh.vertices.len() as u32;
            for (slot, position) in positions.into_iter().enumerate() {
                let rotated = match &element.rotation {
                    Some(rotation) => rotate_vertex(position, rotation.origin, &rotation.transform),
                    None => position,
                };
                let (u, v) = face_vertex_uv(uvs, face.rotation, slot as u32);
                mesh.vertices.push(Vertex {
                    position: rotated,
                    uv: [
                        rect[0] + (rect[2] - rect[0]) * u,
                        rect[1] + (rect[3] - rect[1]) * v,
                    ],
                    shade,
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
