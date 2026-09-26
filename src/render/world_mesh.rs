//! Turning loaded chunks into something the renderer can draw.
//!
//! The server sends a block state as one number. The registry turns that back
//! into a block and its properties, the block's blockstate file picks a
//! variant, and the variant names a model. This walks a box of the world and
//! does that for every position in it.

use std::collections::BTreeMap;

use crate::direction::Direction;
use crate::identifier::Identifier;
use crate::render::resolve::{AssetSource, BuiltAtlas, ResolveError, ResolvedModel};
use crate::render::scene::Mesh;
use crate::world::block_registry;

/// What one position in the world draws.
struct Placement {
    position: [i32; 3],
    model: String,
    rotation: (i32, i32, i32),
    uv_lock: bool,
    tints: Vec<u32>,
}

/// How a world box turned out.
pub struct WorldMesh {
    /// The mesh, in block units with the box's near corner at the origin.
    pub mesh: Mesh,
    /// The atlas every sprite in it came from.
    pub atlas: BuiltAtlas,
    /// How many positions held something other than air.
    pub solid_count: usize,
    /// How many distinct models were used.
    pub model_count: usize,
    /// Block names the build could not resolve, with why, in the order first
    /// seen. Reported rather than swallowed: a world will always have blocks
    /// whose models need machinery that is not built yet.
    pub skipped: Vec<(String, String)>,
}

/// Read a box of the world and build one mesh for everything in it.
///
/// `read` answers the block state at a position, or nothing when the position
/// is outside what is loaded. A position that answers nothing is treated as
/// air, which is what an unloaded neighbour has to be for culling to stay
/// conservative: a face on the edge of the box is drawn rather than hidden.
pub fn build_world_mesh(
    source: &impl AssetSource,
    read: impl Fn(i32, i32, i32) -> Option<i32>,
    origin: [i32; 3],
    size: [i32; 3],
    mip_level: i32,
) -> Result<WorldMesh, ResolveError> {
    let mut placements = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();
    let mut models: BTreeMap<String, ResolvedModel> = BTreeMap::new();
    let mut blockstates: BTreeMap<String, Option<crate::render::blockstate::BlockStateFile>> =
        BTreeMap::new();
    let mut solid_count = 0;

    let note = |name: &str, reason: String, skipped: &mut Vec<(String, String)>| {
        if !skipped.iter().any(|(block, _)| block == name) {
            skipped.push((name.to_owned(), reason));
        }
    };

    for offset_x in 0..size[0] {
        for offset_y in 0..size[1] {
            for offset_z in 0..size[2] {
                let position = [
                    origin[0] + offset_x,
                    origin[1] + offset_y,
                    origin[2] + offset_z,
                ];
                let Some(state_id) = read(position[0], position[1], position[2]) else {
                    continue;
                };
                let Ok(state_id) = u32::try_from(state_id) else {
                    continue;
                };
                let Some((block, state_offset)) = block_registry::block_for_state(state_id) else {
                    continue;
                };
                if block.name == "minecraft:air"
                    || block.name == "minecraft:cave_air"
                    || block.name == "minecraft:void_air"
                {
                    continue;
                }
                solid_count += 1;

                // The blockstate file is read once per block, not per position.
                let file = blockstates.entry(block.name.to_owned()).or_insert_with(|| {
                    let name = Identifier::parse(block.name).ok()?;
                    let text = source.block_state(&name)?;
                    crate::render::blockstate::parse_block_state(&text)
                });
                let Some(file) = file else {
                    note(block.name, "no blockstate file".into(), &mut skipped);
                    continue;
                };

                let properties: BTreeMap<String, String> = block
                    .properties_at(state_offset)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(key, value)| (key.to_owned(), value.to_owned()))
                    .collect();
                let variants = file.variants_for(&properties);
                let Some(variant) = variants.first() else {
                    note(block.name, "no variant matches".into(), &mut skipped);
                    continue;
                };

                let model_name = variant.model.as_string();
                if !models.contains_key(&model_name) {
                    match crate::render::resolve::resolve_model(source, &variant.model) {
                        Ok(model) => {
                            models.insert(model_name.clone(), model);
                        }
                        Err(error) => {
                            note(block.name, format!("{error:?}"), &mut skipped);
                            continue;
                        }
                    }
                }
                placements.push(Placement {
                    position,
                    model: model_name,
                    rotation: (
                        variant.x.degrees(),
                        variant.y.degrees(),
                        variant.z.degrees(),
                    ),
                    uv_lock: variant.uvlock,
                    tints: block_registry::tints(block.name).to_vec(),
                });
            }
        }
    }

    let ordered: Vec<&ResolvedModel> = models.values().collect();
    let atlas = crate::render::resolve::build_atlas_for(source, &ordered, 2048, mip_level)?;

    // A neighbour is solid when it is inside the box and is not air. Anything
    // outside the box counts as open, so the box's own edges keep their faces.
    let solid_at = |x: i32, y: i32, z: i32| -> bool {
        read(x, y, z)
            .and_then(|state| u32::try_from(state).ok())
            .and_then(block_registry::block_for_state)
            .is_some_and(|(block, _)| {
                !matches!(
                    block.name,
                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                )
            })
    };

    let mut mesh = Mesh::default();
    for placement in &placements {
        let model = &models[&placement.model];
        let (rotate_x, rotate_y, rotate_z) = placement.rotation;
        let state = crate::render::rotation::model_state_transform(rotate_x, rotate_y, rotate_z)
            .ok_or(ResolveError::AtlasFull)?;
        let identity = rotate_x % 360 == 0 && rotate_y % 360 == 0 && rotate_z % 360 == 0;
        let uv_transform = |facing| {
            crate::render::rotation::inverse_face_transform(
                rotate_x,
                rotate_y,
                rotate_z,
                facing,
                placement.uv_lock,
            )
            .unwrap_or(crate::render::rotation::FLAT_IDENTITY)
        };
        let occluded = |facing: Direction| {
            let (step_x, step_y, step_z) = facing.step();
            solid_at(
                placement.position[0] + step_x,
                placement.position[1] + step_y,
                placement.position[2] + step_z,
            )
        };
        let part = crate::render::resolve::build_mesh(
            model,
            &atlas,
            (!identity).then_some(&state),
            &placement.tints,
            uv_transform,
            occluded,
        )?;
        // A model is built in its own block's unit cube, so it moves to
        // where the block sits, measured from the box's near corner.
        let offset = [
            (placement.position[0] - origin[0]) as f32,
            (placement.position[1] - origin[1]) as f32,
            (placement.position[2] - origin[2]) as f32,
        ];
        let base = mesh.vertices.len() as u32;
        for mut vertex in part.vertices {
            for (axis, shift) in vertex.position.iter_mut().zip(offset) {
                *axis += shift;
            }
            mesh.vertices.push(vertex);
        }
        mesh.indices
            .extend(part.indices.into_iter().map(|index| index + base));
    }

    Ok(WorldMesh {
        mesh,
        atlas,
        solid_count,
        model_count: models.len(),
        skipped,
    })
}
