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
    state: u32,
    model: String,
    rotation: (i32, i32, i32),
    uv_lock: bool,
    tints: Vec<u32>,
    /// The block's nudge off the grid, if it has one.
    offset: [f32; 3],
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

/// Every block's models and the one atlas they are drawn from, prepared
/// once, the way the reference bakes its block models at resource load.
pub struct BlockRenderer {
    models: BTreeMap<String, ResolvedModel>,
    /// Each item's client item model tree, by registry id.
    items: Vec<Option<crate::render::item_model::ClientItem>>,
    /// The alpha of every texture a flat item extrudes, with its size.
    layer_alpha: BTreeMap<String, crate::render::item_model::LayerAlpha>,
    /// What each block state draws, by state id.
    states: Vec<StateRender>,
    /// The block atlas, with the fluid and sky sprites too.
    pub atlas: BuiltAtlas,
    /// Blocks or models that could not be prepared, with why.
    pub skipped: Vec<(String, String)>,
}

/// What one block state draws, worked out once at load rather than for
/// every block.
enum StateRender {
    Air,
    /// Not air, but nothing to draw it with.
    Missing,
    Drawn {
        name: &'static str,
        properties: BTreeMap<String, String>,
        /// The weighted lists its variants are picked from.
        lists: Vec<Vec<crate::render::blockstate::Variant>>,
        /// Whether every face of every model it can draw hides behind a
        /// side, so a block closed in on all six sides draws nothing.
        closes_in: bool,
    },
}

/// One layer of an item as drawn in one display context.
pub struct ItemLayer {
    /// Quads in the model's unit cube, tinted, with their facings.
    pub mesh: Mesh,
    /// Whether the model lights like a block (`gui_light: side`) rather
    /// than flat from the front.
    pub side_lit: bool,
    /// The context's display transform; `None` is no transform.
    pub transform: Option<crate::render::model::ItemTransform>,
}

/// One meshed region.
pub struct RegionMesh {
    pub mesh: Mesh,
    /// How many positions held something other than air.
    pub solid_count: usize,
}

impl BlockRenderer {
    /// Read every block's blockstate file and every model any of its
    /// variants names, and pack all their textures into one atlas.
    pub fn new(source: &impl AssetSource, mip_level: i32) -> Result<Self, ResolveError> {
        let mut blockstates = BTreeMap::new();
        let mut models: BTreeMap<String, ResolvedModel> = BTreeMap::new();
        let mut skipped: Vec<(String, String)> = Vec::new();
        for block in block_registry::BLOCKS.iter() {
            if matches!(
                block.name,
                "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
            ) {
                continue;
            }
            let Some(file) = Identifier::parse(block.name)
                .ok()
                .and_then(|name| source.block_state(&name))
                .and_then(|text| crate::render::blockstate::parse_block_state(&text))
            else {
                skipped.push((block.name.to_owned(), "no blockstate file".into()));
                continue;
            };
            let lists = file
                .variants
                .iter()
                .flat_map(|variants| variants.values())
                .chain(
                    file.multipart
                        .iter()
                        .flat_map(|parts| parts.iter().map(|part| &part.apply)),
                );
            for list in lists {
                for variant in list {
                    let name = variant.model.as_string();
                    if models.contains_key(&name) {
                        continue;
                    }
                    match crate::render::resolve::resolve_model(source, &variant.model) {
                        Ok(model) => {
                            models.insert(name, model);
                        }
                        Err(error) => {
                            if !skipped
                                .iter()
                                .any(|(block_name, _)| block_name == block.name)
                            {
                                skipped.push((block.name.to_owned(), format!("{error:?}")));
                            }
                        }
                    }
                }
            }
            blockstates.insert(block.name.to_owned(), file);
        }
        let mut states = Vec::new();
        for block in block_registry::BLOCKS.iter() {
            for offset in 0..u32::from(block.state_count) {
                let state_id = (block.first_state_id + offset) as usize;
                if states.len() <= state_id {
                    states.resize_with(state_id + 1, || StateRender::Missing);
                }
                if matches!(
                    block.name,
                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                ) {
                    states[state_id] = StateRender::Air;
                    continue;
                }
                let Some(file) = blockstates.get(block.name) else {
                    continue;
                };
                let properties: BTreeMap<String, String> = block
                    .properties_at(offset)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(key, value)| (key.to_owned(), value.to_owned()))
                    .collect();
                let lists = file.matched_lists(&properties);
                let closes_in = lists.iter().flatten().all(|variant| {
                    models.get(&variant.model.as_string()).is_none_or(|model| {
                        model.elements.iter().all(|element| {
                            element
                                .faces
                                .iter()
                                .all(|(_, face)| face.cull_for_direction.is_some())
                        })
                    })
                });
                states[state_id] = StateRender::Drawn {
                    name: block.name,
                    properties,
                    lists,
                    closes_in,
                };
            }
        }
        // Every item's model tree, and the models its layers name in any
        // display context, go in with the blocks'.
        let mut items = Vec::with_capacity(crate::item_table::ITEMS.len());
        let mut layer_alpha = BTreeMap::new();
        let mut layer_sprites = Vec::new();
        for item in crate::item_table::ITEMS.iter() {
            let def = Identifier::parse(item.name)
                .ok()
                .and_then(|name| source.item(&name))
                .and_then(|text| crate::render::item_model::parse_client_item(&text));
            if let Some(def) = &def {
                let mut layers = Vec::new();
                for context in crate::render::model::DisplayContext::ALL {
                    crate::render::item_model::layers_for(&def.model, context, &mut layers);
                }
                for layer in layers {
                    if let Some(crate::render::item_model::SpecialModel::Chest { texture }) =
                        &layer.special
                    {
                        layer_sprites.push(texture.clone());
                    }
                    let name = layer.model.as_string();
                    if models.contains_key(&name) {
                        continue;
                    }
                    let Ok(model) = crate::render::resolve::resolve_model(source, &layer.model)
                    else {
                        continue;
                    };
                    if model.generated {
                        for slot in ["layer0", "layer1", "layer2", "layer3", "layer4"] {
                            let Some(material) = model.textures.material(slot) else {
                                break;
                            };
                            let sprite = material.sprite.as_string();
                            if layer_alpha.contains_key(&sprite) {
                                continue;
                            }
                            // The geometry is made from the texture later, so
                            // no element names it for the atlas.
                            layer_sprites.push(material.sprite.clone());
                            if let Some(image) = source
                                .texture(&material.sprite)
                                .and_then(|bytes| crate::render::png::decode_rgba(&bytes).ok())
                            {
                                // An animated strip's first frame decides the sides.
                                let frame = image.width.min(image.height);
                                let alpha = image.rgba[..(frame * frame * 4) as usize]
                                    .chunks(4)
                                    .map(|pixel| pixel[3])
                                    .collect();
                                layer_alpha.insert(sprite, (alpha, frame as i32, frame as i32));
                            }
                        }
                    }
                    models.insert(name, model);
                }
            }
            items.push(def);
        }
        let ordered: Vec<&ResolvedModel> = models.values().collect();
        // The fluid renderer's sprites and the sky's sun and moon phases are
        // named by no block model, so they are asked for by name.
        let extra: Vec<Identifier> = FLUID_SPRITES
            .iter()
            .map(|name| (*name).to_owned())
            .chain(std::iter::once(crate::render::sky::SUN_SPRITE.to_owned()))
            .chain(crate::render::sky::moon_sprites())
            .map(|name| Identifier::parse(&name).expect("fixed sprite names are valid"))
            .chain(layer_sprites)
            .collect();
        let atlas =
            crate::render::resolve::build_atlas_with(source, &ordered, &extra, 8192, mip_level)?;
        Ok(Self {
            models,
            items,
            layer_alpha,
            states,
            atlas,
            skipped,
        })
    }

    /// The quads an item's default stack draws in a display context, layer
    /// by layer, each with its model's GUI light and display transforms.
    /// Corners are in the model's unit cube; tints are already applied to
    /// the vertices' colour. `grass` reads the grass colour map.
    pub fn item_layers(
        &self,
        item: i32,
        context: crate::render::model::DisplayContext,
        grass: &dyn Fn(f64, f64) -> u32,
    ) -> Vec<ItemLayer> {
        let Some(Some(def)) = usize::try_from(item)
            .ok()
            .and_then(|index| self.items.get(index))
        else {
            return Vec::new();
        };
        let mut layers = Vec::new();
        crate::render::item_model::layers_for(&def.model, context, &mut layers);
        let mut out = Vec::new();
        for layer in layers {
            let Some(model) = self.models.get(&layer.model.as_string()) else {
                continue;
            };
            let tints: Vec<u32> = layer
                .tints
                .iter()
                .map(|tint| crate::render::item_model::tint_colour(tint, grass))
                .collect();
            if let Some(crate::render::item_model::SpecialModel::Chest { texture }) = &layer.special
            {
                let Some(rect) = self.atlas.sprites.get(&texture.as_string()) else {
                    continue;
                };
                let mut mesh = chest_mesh(rect);
                apply_local(&mut mesh, &layer.local);
                out.push(ItemLayer {
                    mesh,
                    side_lit: model.gui_light == crate::render::model::GuiLight::Side,
                    transform: model.display.get(context),
                });
                continue;
            }
            let geometry;
            let model = if model.generated {
                let mut flat = model.clone();
                flat.elements =
                    crate::render::item_model::generated_elements(&model.textures, &|sprite| {
                        self.layer_alpha.get(&sprite.as_string()).cloned()
                    });
                geometry = flat;
                &geometry
            } else {
                model
            };
            let Ok(mut mesh) = crate::render::resolve::build_mesh(
                model,
                &self.atlas,
                None,
                &tints,
                |_| crate::render::rotation::FLAT_IDENTITY,
                |_| false,
            ) else {
                continue;
            };
            apply_local(&mut mesh, &layer.local);
            out.push(ItemLayer {
                mesh,
                side_lit: model.gui_light == crate::render::model::GuiLight::Side,
                transform: model.display.get(context),
            });
        }
        out
    }

    /// A client item's swap properties: whether a new stack lowers the
    /// hand, and how far. Items without a definition take the defaults.
    pub fn item_swap(&self, item: i32) -> (bool, f32) {
        usize::try_from(item)
            .ok()
            .and_then(|index| self.items.get(index))
            .and_then(Option::as_ref)
            .map_or((true, 1.0), |def| {
                (def.hand_animation_on_swap, def.swap_animation_scale)
            })
    }

    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Every quad a block's model draws at `position`, culled against
    /// nothing, as the breaking overlay draws them: corners in the model's
    /// own unit cube and each quad's facing, with the block's offset. The
    /// variants are the ones the world mesh picks there.
    pub fn block_quads(
        &self,
        state_id: u32,
        position: [i32; 3],
    ) -> (Vec<([[f32; 3]; 4], Direction)>, [f32; 3]) {
        let Some(StateRender::Drawn {
            name,
            properties,
            lists,
            ..
        }) = self.states.get(state_id as usize)
        else {
            return (Vec::new(), [0.0; 3]);
        };
        let mut random =
            crate::random::RandomSource::create(render_seed(name, properties, position));
        let variants = crate::render::blockstate::BlockStateFile::pick_variants(lists, &mut random);
        let mut quads = Vec::new();
        for variant in &variants {
            let Some(model) = self.models.get(&variant.model.as_string()) else {
                continue;
            };
            let (rotate_x, rotate_y, rotate_z) = (
                variant.x.degrees(),
                variant.y.degrees(),
                variant.z.degrees(),
            );
            let Some(state) =
                crate::render::rotation::model_state_transform(rotate_x, rotate_y, rotate_z)
            else {
                continue;
            };
            let identity = rotate_x % 360 == 0 && rotate_y % 360 == 0 && rotate_z % 360 == 0;
            let Ok(part) = crate::render::resolve::build_mesh(
                model,
                &self.atlas,
                (!identity).then_some(&state),
                &[],
                |_| crate::render::rotation::FLAT_IDENTITY,
                |_| false,
            ) else {
                continue;
            };
            for (quad, info) in part.quads.iter().enumerate() {
                let corners =
                    std::array::from_fn(|corner| part.vertices[quad * 4 + corner].position);
                quads.push((corners, info.direction));
            }
        }
        (quads, block_offset(name, position))
    }

    /// Mesh every block in a box of the world, positioned relative to
    /// `mesh_origin`.
    ///
    /// `read` answers the block state at a position, or nothing where no
    /// chunk is loaded, which reads as air. Neighbours outside the box are
    /// read from the world too, so a region's faces cull exactly as the
    /// whole world's would.
    pub fn mesh_region(
        &self,
        read: &impl Fn(i32, i32, i32) -> Option<i32>,
        light: &impl Fn(i32, i32, i32) -> (u8, u8),
        biome_tint: &impl Fn(crate::render::biome_tint::TintKind, [i32; 3]) -> u32,
        origin: [i32; 3],
        size: [i32; 3],
        mesh_origin: [i32; 3],
    ) -> RegionMesh {
        let mut placements = Vec::new();
        let mut solid_count = 0;
        for offset_x in 0..size[0] {
            for offset_y in 0..size[1] {
                for offset_z in 0..size[2] {
                    let position = [
                        origin[0] + offset_x,
                        origin[1] + offset_y,
                        origin[2] + offset_z,
                    ];
                    let Some(state_id) = read(position[0], position[1], position[2])
                        .and_then(|state| u32::try_from(state).ok())
                    else {
                        continue;
                    };
                    let StateRender::Drawn {
                        name,
                        properties,
                        lists,
                        closes_in,
                    } = (match self.states.get(state_id as usize) {
                        Some(StateRender::Air) => continue,
                        Some(render) => render,
                        None => &StateRender::Missing,
                    })
                    else {
                        solid_count += 1;
                        continue;
                    };
                    solid_count += 1;
                    // Closed in on every side, it would draw no face at all.
                    if *closes_in
                        && Direction::stream().all(|facing| {
                            let (step_x, step_y, step_z) = facing.step();
                            let neighbour = read(
                                position[0] + step_x,
                                position[1] + step_y,
                                position[2] + step_z,
                            )
                            .and_then(|state| u32::try_from(state).ok());
                            !crate::render::face_culling::should_render_face(
                                state_id, neighbour, facing,
                            )
                        })
                    {
                        continue;
                    }
                    let mut random = crate::random::RandomSource::create(render_seed(
                        name, properties, position,
                    ));
                    let variants = crate::render::blockstate::BlockStateFile::pick_variants(
                        lists,
                        &mut random,
                    );
                    if variants.is_empty() {
                        continue;
                    }
                    let tints = world_tints(name, properties, position, biome_tint);
                    let offset = block_offset(name, position);
                    for variant in &variants {
                        let model_name = variant.model.as_string();
                        if !self.models.contains_key(&model_name) {
                            continue;
                        }
                        placements.push(Placement {
                            position,
                            state: state_id,
                            model: model_name,
                            rotation: (
                                variant.x.degrees(),
                                variant.y.degrees(),
                                variant.z.degrees(),
                            ),
                            uv_lock: variant.uvlock,
                            tints: tints.clone(),
                            offset,
                        });
                    }
                }
            }
        }

        let atlas = &self.atlas;
        let lighter = crate::render::block_light::Lighter {
            state: |p: [i32; 3]| read(p[0], p[1], p[2]).and_then(|state| u32::try_from(state).ok()),
            light: |p: [i32; 3]| light(p[0], p[1], p[2]),
        };
        let mut mesh = Mesh::default();
        for placement in &placements {
            let model = &self.models[&placement.model];
            let (rotate_x, rotate_y, rotate_z) = placement.rotation;
            let Some(state) =
                crate::render::rotation::model_state_transform(rotate_x, rotate_y, rotate_z)
            else {
                continue;
            };
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
            // The reference's own culling decision against the real
            // neighbour; an unloaded one hides nothing.
            let occluded = |facing: Direction| {
                let (step_x, step_y, step_z) = facing.step();
                let neighbour_state = read(
                    placement.position[0] + step_x,
                    placement.position[1] + step_y,
                    placement.position[2] + step_z,
                )
                .and_then(|state| u32::try_from(state).ok());
                !crate::render::face_culling::should_render_face(
                    placement.state,
                    neighbour_state,
                    facing,
                )
            };
            let Ok(mut part) = crate::render::resolve::build_mesh(
                model,
                atlas,
                (!identity).then_some(&state),
                &placement.tints,
                uv_transform,
                occluded,
            ) else {
                continue;
            };
            // Smooth lighting, the reference's default, unless the block glows or
            // the model opts out of ambient occlusion.
            let emission = crate::render::block_light::light_traits(Some(placement.state)).emission;
            let smooth = emission == 0 && model.ambient_occlusion;
            for (quad, info) in part.quads.iter().enumerate() {
                let corners: [[f32; 3]; 4] =
                    std::array::from_fn(|corner| part.vertices[quad * 4 + corner].position);
                let lit = lighter.light_quad(
                    placement.state,
                    placement.position,
                    &corners,
                    info.direction,
                    info.cull,
                    smooth,
                );
                let side = if info.shade {
                    crate::render::block_light::cardinal(info.direction)
                } else {
                    1.0
                };
                for corner in 0..4 {
                    let vertex = &mut part.vertices[quad * 4 + corner];
                    // The colour is built in 8-bit steps as the reference builds
                    // it: the occlusion grey, then the side shade, then the tint.
                    let grey = if smooth {
                        let grey = (lit.shade[corner] * 255.0).floor() as i32;
                        ((grey as f32 * side) as i32).clamp(0, 255)
                    } else {
                        (side * 255.0).floor() as i32
                    };
                    let tint = vertex.tint.map(|channel| (channel * 255.0).round() as i32);
                    let white = tint == [255, 255, 255, 255];
                    let channel = |index: usize| {
                        if white {
                            grey
                        } else {
                            grey * tint[index] / 255
                        }
                    };
                    vertex.tint = [
                        channel(0) as f32 / 255.0,
                        channel(1) as f32 / 255.0,
                        channel(2) as f32 / 255.0,
                        1.0,
                    ];
                    vertex.shade = 1.0;
                    vertex.light = crate::render::block_light::light_uv(lit.light[corner]);
                }
            }
            // A model is built in its own block's unit cube, so it moves to
            // where the block sits, measured from the box's near corner.
            let offset = [
                (placement.position[0] - mesh_origin[0]) as f32 + placement.offset[0],
                (placement.position[1] - mesh_origin[1]) as f32 + placement.offset[1],
                (placement.position[2] - mesh_origin[2]) as f32 + placement.offset[2],
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

        let fluids = FluidMesher {
            biome_tint,
            read,
            lighter: &lighter,
            atlas,
            origin: mesh_origin,
        };
        for offset_x in 0..size[0] {
            for offset_y in 0..size[1] {
                for offset_z in 0..size[2] {
                    fluids.tesselate(
                        [
                            origin[0] + offset_x,
                            origin[1] + offset_y,
                            origin[2] + offset_z,
                        ],
                        &mut mesh,
                    );
                }
            }
        }
        RegionMesh { mesh, solid_count }
    }
}

/// Read a box of the world and build one mesh for everything in it, with
/// its own atlas.
pub fn build_world_mesh(
    source: &impl AssetSource,
    read: impl Fn(i32, i32, i32) -> Option<i32>,
    light: impl Fn(i32, i32, i32) -> (u8, u8),
    biome_tint: impl Fn(crate::render::biome_tint::TintKind, [i32; 3]) -> u32,
    origin: [i32; 3],
    size: [i32; 3],
    mip_level: i32,
) -> Result<WorldMesh, ResolveError> {
    let renderer = BlockRenderer::new(source, mip_level)?;
    let region = renderer.mesh_region(&read, &light, &biome_tint, origin, size, origin);
    let model_count = renderer.model_count();
    Ok(WorldMesh {
        mesh: region.mesh,
        atlas: renderer.atlas,
        solid_count: region.solid_count,
        model_count,
        skipped: renderer.skipped,
    })
}

/// The sprites the fluid renderer draws with, which no block model names.
const FLUID_SPRITES: [&str; 5] = [
    "minecraft:block/water_still",
    "minecraft:block/water_flow",
    "minecraft:block/water_overlay",
    "minecraft:block/lava_still",
    "minecraft:block/lava_flow",
];

/// One fluid block's surface and sides, built the way the reference's fluid
/// renderer builds them: corner heights averaged from the neighbours, the
/// still texture on a level surface and the flowing one turned along the
/// flow, and a face only where the neighbour is not the same fluid.
///
/// Where the reference asks a block for its face occlusion shape or whether
/// it is solid, this asks whether its collision shape is the whole block.
/// That is the same answer for ordinary full blocks and an approximation for
/// everything else.
struct FluidMesher<'a, R: Fn(i32, i32, i32) -> Option<i32>, S, L> {
    biome_tint: &'a dyn Fn(crate::render::biome_tint::TintKind, [i32; 3]) -> u32,
    read: &'a R,
    lighter: &'a crate::render::block_light::Lighter<S, L>,
    atlas: &'a BuiltAtlas,
    origin: [i32; 3],
}

#[derive(Clone, Copy)]
struct FluidAt {
    water: bool,
    own_height: f32,
}

impl<R, S, L> FluidMesher<'_, R, S, L>
where
    R: Fn(i32, i32, i32) -> Option<i32>,
    S: Fn([i32; 3]) -> Option<u32>,
    L: Fn([i32; 3]) -> (u8, u8),
{
    /// `FluidRenderer.getLightCoords`: the brighter of here and above.
    fn fluid_light(&self, x: i32, y: i32, z: i32) -> [f32; 2] {
        let at = |p: [i32; 3]| self.lighter.light_coords((self.lighter.state)(p), p);
        let here = at([x, y, z]);
        let above = at([x, y + 1, z]);
        let block = ((here >> 4) & 15).max((above >> 4) & 15);
        let sky = ((here >> 20) & 15).max((above >> 20) & 15);
        crate::render::block_light::light_uv((block << 4) | (sky << 20))
    }

    fn fluid(&self, x: i32, y: i32, z: i32) -> Option<FluidAt> {
        let state = (self.read)(x, y, z)?;
        let (water, own_height, _) = crate::world::local_player::state_fluid(state)?;
        Some(FluidAt { water, own_height })
    }

    fn solid(&self, x: i32, y: i32, z: i32) -> bool {
        (self.read)(x, y, z)
            .and_then(|state| u32::try_from(state).ok())
            .is_some_and(|state| {
                crate::world::block_state::table_collision_shape(state)
                    .equals_java(&crate::shape::Shapes::block())
            })
    }

    fn same(&self, water: bool, x: i32, y: i32, z: i32) -> bool {
        self.fluid(x, y, z)
            .is_some_and(|other| other.water == water)
    }

    /// A neighbour's height for the corner average: the fluid's own height,
    /// a full block under the same fluid, nothing for an open neighbour and
    /// a negative mark for a solid one.
    fn height(&self, water: bool, x: i32, y: i32, z: i32) -> f32 {
        match self.fluid(x, y, z) {
            Some(fluid) if fluid.water == water => {
                if self.same(water, x, y + 1, z) {
                    1.0
                } else {
                    fluid.own_height
                }
            }
            _ if self.solid(x, y, z) => -1.0,
            _ => 0.0,
        }
    }

    fn average_height(
        &self,
        water: bool,
        own: f32,
        first: f32,
        second: f32,
        corner: [i32; 3],
    ) -> f32 {
        if second >= 1.0 || first >= 1.0 {
            return 1.0;
        }
        let mut weighted = [0.0_f32; 2];
        let mut add = |height: f32| {
            if height >= 0.8 {
                weighted[0] += height * 10.0;
                weighted[1] += 10.0;
            } else if height >= 0.0 {
                weighted[0] += height;
                weighted[1] += 1.0;
            }
        };
        if second > 0.0 || first > 0.0 {
            let corner_height = self.height(water, corner[0], corner[1], corner[2]);
            if corner_height >= 1.0 {
                return 1.0;
            }
            add(corner_height);
        }
        add(own);
        add(second);
        add(first);
        weighted[0] / weighted[1]
    }

    /// Whether the neighbour hides the fluid's face towards it, from the
    /// neighbour's face occlusion shape. A whole-block shape hides every face
    /// but a surface below full height; a partial one is treated as hiding
    /// nothing, which is where this still differs from the reference.
    fn occluded(&self, direction: Direction, height: f32, x: i32, y: i32, z: i32) -> bool {
        let kind = (self.read)(x, y, z)
            .and_then(|state| u32::try_from(state).ok())
            .and_then(|state| {
                crate::render::face_culling::occlusion_kind(state, direction.opposite())
            });
        kind == Some(1) && (direction != Direction::Up || height == 1.0)
    }

    /// Whether a water side against this block shows the overlay sprite:
    /// the reference's half-transparent blocks and leaves.
    fn shows_overlay(&self, x: i32, y: i32, z: i32) -> bool {
        (self.read)(x, y, z)
            .and_then(|state| u32::try_from(state).ok())
            .and_then(block_registry::block_for_state)
            .is_some_and(|(block, _)| {
                crate::render::fluid_overlay_table::OVERLAY_BLOCKS.contains(&block.name)
            })
    }

    /// The horizontal flow direction, which is all the surface texture uses.
    fn flow(&self, fluid: FluidAt, x: i32, y: i32, z: i32) -> (f64, f64) {
        let mut flow_x = 0.0_f64;
        let mut flow_z = 0.0_f64;
        for (step_x, step_z) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let (nx, nz) = (x + step_x, z + step_z);
            let neighbour = self.fluid(nx, y, nz);
            if neighbour.is_some_and(|other| other.water != fluid.water) {
                continue;
            }
            let mut distance = 0.0_f32;
            let neighbour_height = neighbour.map_or(0.0, |other| other.own_height);
            if neighbour_height == 0.0 {
                if !self.solid(nx, y, nz) {
                    let below = self.fluid(nx, y - 1, nz);
                    if below.is_none_or(|other| other.water == fluid.water) {
                        let below_height = below.map_or(0.0, |other| other.own_height);
                        if below_height > 0.0 {
                            distance = fluid.own_height - (below_height - 0.888_888_9);
                        }
                    }
                }
            } else {
                distance = fluid.own_height - neighbour_height;
            }
            if distance != 0.0 {
                flow_x += f64::from(step_x) * f64::from(distance);
                flow_z += f64::from(step_z) * f64::from(distance);
            }
        }
        (flow_x, flow_z)
    }

    fn sprite(&self, name: &str) -> [f32; 4] {
        self.atlas.sprites[name]
    }

    fn tesselate(&self, pos: [i32; 3], mesh: &mut Mesh) {
        let [x, y, z] = pos;
        let Some(fluid) = self.fluid(x, y, z) else {
            return;
        };
        let water = fluid.water;
        let render_up = !self.same(water, x, y + 1, z);
        let render_down = !self.same(water, x, y - 1, z)
            && !self.occluded(Direction::Down, 0.888_888_9, x, y - 1, z);
        let render_north = !self.same(water, x, y, z - 1);
        let render_south = !self.same(water, x, y, z + 1);
        let render_west = !self.same(water, x - 1, y, z);
        let render_east = !self.same(water, x + 1, y, z);
        if !(render_up || render_down || render_north || render_south || render_west || render_east)
        {
            return;
        }

        let own = self.height(water, x, y, z);
        let (mut north_east, mut north_west, mut south_east, mut south_west) = if own >= 1.0 {
            (1.0, 1.0, 1.0, 1.0)
        } else {
            let north = self.height(water, x, y, z - 1);
            let south = self.height(water, x, y, z + 1);
            let east = self.height(water, x + 1, y, z);
            let west = self.height(water, x - 1, y, z);
            (
                self.average_height(water, own, north, east, [x + 1, y, z - 1]),
                self.average_height(water, own, north, west, [x - 1, y, z - 1]),
                self.average_height(water, own, south, east, [x + 1, y, z + 1]),
                self.average_height(water, own, south, west, [x - 1, y, z + 1]),
            )
        };

        let (still, flowing) = if water {
            ("minecraft:block/water_still", "minecraft:block/water_flow")
        } else {
            ("minecraft:block/lava_still", "minecraft:block/lava_flow")
        };
        let tint = if water {
            (self.biome_tint)(crate::render::biome_tint::TintKind::Water, [x, y, z])
        } else {
            0xFFFFFF
        };
        let tint = [
            ((tint >> 16) & 0xff) as f32 / 255.0,
            ((tint >> 8) & 0xff) as f32 / 255.0,
            (tint & 0xff) as f32 / 255.0,
            1.0,
        ];
        let lx = (x - self.origin[0]) as f32;
        let ly = (y - self.origin[1]) as f32;
        let lz = (z - self.origin[2]) as f32;
        let bottom = if render_down { 0.001 } else { 0.0 };

        let lowest = north_west.min(south_west).min(south_east.min(north_east));
        if render_up && !self.occluded(Direction::Up, lowest, x, y + 1, z) {
            north_west -= 0.001;
            south_west -= 0.001;
            south_east -= 0.001;
            north_east -= 0.001;
            let (flow_x, flow_z) = self.flow(fluid, x, y, z);
            let uv = if flow_x == 0.0 && flow_z == 0.0 {
                let [u0, v0, u1, v1] = self.sprite(still);
                [[u0, v0], [u0, v1], [u1, v1], [u1, v0]]
            } else {
                let [u0, v0, u1, v1] = self.sprite(flowing);
                let angle =
                    crate::math::mth_atan2(flow_z, flow_x) as f32 - std::f32::consts::FRAC_PI_2;
                let s = crate::math::mth_sin_f64(f64::from(angle)) * 0.25;
                let c = crate::math::mth_cos_f64(f64::from(angle)) * 0.25;
                let u = |f: f32| u0 + (u1 - u0) * f;
                let v = |f: f32| v0 + (v1 - v0) * f;
                [
                    [u(0.5 + (-c - s)), v(0.5 + (-c + s))],
                    [u(0.5 + (-c + s)), v(0.5 + (c + s))],
                    [u(0.5 + (c + s)), v(0.5 + (c - s))],
                    [u(0.5 + (c - s)), v(0.5 + (-c - s))],
                ]
            };
            let corners = [
                [lx, ly + north_west, lz],
                [lx, ly + south_west, lz + 1.0],
                [lx + 1.0, ly + south_east, lz + 1.0],
                [lx + 1.0, ly + north_east, lz],
            ];
            // Seen from below, the surface is drawn again facing down, unless
            // the block above is solid enough to hide it.
            let light = self.fluid_light(x, y, z);
            add_face(
                mesh,
                corners,
                uv,
                1.0,
                tint,
                light,
                !self.solid(x, y + 1, z),
            );
        }

        if render_down {
            let [u0, v0, u1, v1] = self.sprite(still);
            add_face(
                mesh,
                [
                    [lx, ly + bottom, lz],
                    [lx + 1.0, ly + bottom, lz],
                    [lx + 1.0, ly + bottom, lz + 1.0],
                    [lx, ly + bottom, lz + 1.0],
                ],
                [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                0.5,
                tint,
                self.fluid_light(x, y - 1, z),
                false,
            );
        }

        let sides = [
            (
                Direction::North,
                render_north,
                [x, z - 1],
                north_west,
                north_east,
                [lx, lz + 0.001],
                [lx + 1.0, lz + 0.001],
                0.8,
            ),
            (
                Direction::South,
                render_south,
                [x, z + 1],
                south_east,
                south_west,
                [lx + 1.0, lz + 0.999],
                [lx, lz + 0.999],
                0.8,
            ),
            (
                Direction::West,
                render_west,
                [x - 1, z],
                south_west,
                north_west,
                [lx + 0.001, lz + 1.0],
                [lx + 0.001, lz],
                0.6,
            ),
            (
                Direction::East,
                render_east,
                [x + 1, z],
                north_east,
                south_east,
                [lx + 0.999, lz],
                [lx + 0.999, lz + 1.0],
                0.6,
            ),
        ];
        for (direction, render, [nx, nz], h0, h1, [x0, z0], [x1, z1], shade) in sides {
            if !render || self.occluded(direction, h0.max(h1), nx, y, nz) {
                continue;
            }
            // Water against glass, ice or leaves shows its overlay, drawn
            // from the front only.
            let overlay = water && self.shows_overlay(nx, y, nz);
            let [u0, v0, u1, v1] = self.sprite(if overlay {
                "minecraft:block/water_overlay"
            } else {
                flowing
            });
            let u = |f: f32| u0 + (u1 - u0) * f;
            let v = |f: f32| v0 + (v1 - v0) * f;
            add_face(
                mesh,
                [
                    [x0, ly + h0, z0],
                    [x1, ly + h1, z1],
                    [x1, ly + bottom, z1],
                    [x0, ly + bottom, z0],
                ],
                [
                    [u(0.0), v((1.0 - h0) * 0.5)],
                    [u(0.5), v((1.0 - h1) * 0.5)],
                    [u(0.5), v(0.5)],
                    [u(0.0), v(0.5)],
                ],
                shade,
                tint,
                self.fluid_light(x, y, z),
                !overlay,
            );
        }
    }
}

/// Add one fluid quad to the translucent list, and its reverse when the
/// reference draws the face from behind too.
fn add_face(
    mesh: &mut Mesh,
    corners: [[f32; 3]; 4],
    uv: [[f32; 2]; 4],
    shade: f32,
    tint: [f32; 4],
    light: [f32; 2],
    back_face: bool,
) {
    let base = mesh.vertices.len() as u32;
    for (position, uv) in corners.into_iter().zip(uv) {
        mesh.vertices.push(crate::render::scene::Vertex {
            position,
            uv,
            shade,
            tint,
            light,
        });
    }
    mesh.translucent_indices
        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    if back_face {
        mesh.translucent_indices
            .extend([base, base + 3, base + 2, base, base + 2, base + 1]);
    }
}

/// The colours a block's tint layers take at a position in a world: the
/// biome's for the layers that read one, the fixed colour for the rest.
fn world_tints(
    block: &str,
    properties: &BTreeMap<String, String>,
    position: [i32; 3],
    biome_tint: &impl Fn(crate::render::biome_tint::TintKind, [i32; 3]) -> u32,
) -> Vec<u32> {
    // The lily pad has a world colour of its own.
    if block == "minecraft:lily_pad" {
        return vec![crate::render::tint::LILY_PAD_IN_WORLD];
    }
    if let Some(tint) = crate::render::tint::state_tint(block, properties) {
        return vec![tint];
    }
    let constant = block_registry::tints(block);
    let Some(layers) = crate::render::biome_tint::tint_layers(block) else {
        return constant.to_vec();
    };
    // A tall plant's upper half takes the colour of the block below it.
    let sample = if properties.get("half").map(String::as_str) == Some("upper")
        && matches!(block, "minecraft:tall_grass" | "minecraft:large_fern")
    {
        [position[0], position[1] - 1, position[2]]
    } else {
        position
    };
    layers
        .iter()
        .enumerate()
        .map(|(slot, layer)| match layer {
            Some(kind) => biome_tint(*kind, sample),
            None => constant.get(slot).copied().unwrap_or(0xFFFF_FFFF),
        })
        .collect()
}

/// `BlockState.getSeed`: the position a block's model randomness is seeded
/// from. Two-block plants and doors share their lower half's seed, and a bed
/// its head's column.
fn render_seed(block: &str, properties: &BTreeMap<String, String>, position: [i32; 3]) -> i64 {
    let [x, y, z] = position;
    let double_plant = matches!(
        block,
        "minecraft:tall_grass"
            | "minecraft:large_fern"
            | "minecraft:sunflower"
            | "minecraft:lilac"
            | "minecraft:rose_bush"
            | "minecraft:peony"
            | "minecraft:tall_seagrass"
            | "minecraft:small_dripleaf"
            | "minecraft:pitcher_plant"
            | "minecraft:pitcher_crop"
    );
    let door = block.ends_with("_door") && !block.ends_with("trapdoor");
    if double_plant || door {
        let upper = properties.get("half").map(String::as_str) == Some("upper");
        return crate::math::mth_get_seed(x, if upper { y - 1 } else { y }, z);
    }
    if block.ends_with("_bed") {
        let step = if properties.get("part").map(String::as_str) == Some("head") {
            0
        } else {
            1
        };
        let (dx, dz) = match properties.get("facing").map(String::as_str) {
            Some("north") => (0, -1),
            Some("south") => (0, 1),
            Some("west") => (-1, 0),
            _ => (1, 0),
        };
        return crate::math::mth_get_seed(x + dx * step, y, z + dz * step);
    }
    crate::math::mth_get_seed(x, y, z)
}

/// `BlockState.getOffset`: how far a plant is nudged off the centre of its
/// block, from its column's seed and the block's limits.
fn block_offset(block: &str, position: [i32; 3]) -> [f32; 3] {
    crate::world::block_offset(block, position).map(|value| value as f32)
}

/// The chest model's quads as an item layer's mesh: corners in the unit
/// block, texture coordinates on the chest's sprite in the atlas, white.
fn chest_mesh(rect: &[f32; 4]) -> Mesh {
    let mut mesh = Mesh::default();
    for quad in crate::render::player_model::chest_quads() {
        let base = mesh.vertices.len() as u32;
        let normal = quad[0].normal;
        for vertex in quad {
            mesh.vertices.push(crate::render::Vertex {
                position: vertex.position,
                uv: [
                    rect[0] + (rect[2] - rect[0]) * vertex.uv[0],
                    rect[1] + (rect[3] - rect[1]) * vertex.uv[1],
                ],
                shade: 1.0,
                tint: [1.0; 4],
                light: crate::render::scene::FULL_BRIGHT,
            });
        }
        mesh.quads.push(crate::render::scene::QuadInfo {
            direction: crate::render::geometry::closest_direction(normal).unwrap_or(Direction::Up),
            cull: None,
            shade: true,
        });
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    mesh
}

/// A layer's `transformation` applied to its quads, in the unit block. The
/// quads keep facing outwards: a mirroring matrix turns each quad's corners
/// round, and the facing each quad is shaded by follows the matrix.
fn apply_local(mesh: &mut Mesh, matrix: &crate::render::item_model::Matrix4) {
    if *matrix == crate::render::item_model::IDENTITY {
        return;
    }
    let determinant = {
        let m = matrix;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    for vertex in &mut mesh.vertices {
        let [x, y, z] = vertex.position;
        vertex.position = std::array::from_fn(|row| {
            matrix[row][0] * x + matrix[row][1] * y + matrix[row][2] * z + matrix[row][3]
        });
    }
    for (quad, info) in mesh.quads.iter_mut().enumerate() {
        let first = quad * 4;
        if determinant < 0.0 {
            mesh.vertices[first + 1..first + 4].reverse();
        }
        let normal = crate::render::geometry::triangle_normal(
            mesh.vertices[first].position,
            mesh.vertices[first + 1].position,
            mesh.vertices[first + 2].position,
        );
        if let Some(direction) = crate::render::geometry::closest_direction(normal) {
            info.direction = direction;
        }
    }
}
