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
                    state: state_id,
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
    let fluid_sprites: Vec<Identifier> = FLUID_SPRITES
        .iter()
        .map(|name| Identifier::parse(name).expect("fluid sprite names are valid"))
        .collect();
    let atlas = crate::render::resolve::build_atlas_with(
        source,
        &ordered,
        &fluid_sprites,
        2048,
        mip_level,
    )?;

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
        // The reference's own culling decision. A neighbour outside the box
        // hides nothing, so the box's edges keep their faces.
        let occluded = |facing: Direction| {
            let (step_x, step_y, step_z) = facing.step();
            let neighbour = [
                placement.position[0] + step_x,
                placement.position[1] + step_y,
                placement.position[2] + step_z,
            ];
            let inside = (0..3).all(|axis| {
                neighbour[axis] >= origin[axis] && neighbour[axis] < origin[axis] + size[axis]
            });
            let neighbour_state = inside
                .then(|| read(neighbour[0], neighbour[1], neighbour[2]))
                .flatten()
                .and_then(|state| u32::try_from(state).ok());
            !crate::render::face_culling::should_render_face(
                placement.state,
                neighbour_state,
                facing,
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

    let fluids = FluidMesher {
        read: &read,
        atlas: &atlas,
        origin,
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

    Ok(WorldMesh {
        mesh,
        atlas,
        solid_count,
        model_count: models.len(),
        skipped,
    })
}

/// The sprites the fluid renderer draws with, which no block model names.
const FLUID_SPRITES: [&str; 4] = [
    "minecraft:block/water_still",
    "minecraft:block/water_flow",
    "minecraft:block/lava_still",
    "minecraft:block/lava_flow",
];

/// The water colour of the default biome. Biome colours are not read from
/// the world yet, so every water block takes this one.
const WATER_TINT: u32 = 0x3F76E4;

/// One fluid block's surface and sides, built the way the reference's fluid
/// renderer builds them: corner heights averaged from the neighbours, the
/// still texture on a level surface and the flowing one turned along the
/// flow, and a face only where the neighbour is not the same fluid.
///
/// Where the reference asks a block for its face occlusion shape or whether
/// it is solid, this asks whether its collision shape is the whole block.
/// That is the same answer for ordinary full blocks and an approximation for
/// everything else.
struct FluidMesher<'a, R: Fn(i32, i32, i32) -> Option<i32>> {
    read: &'a R,
    atlas: &'a BuiltAtlas,
    origin: [i32; 3],
}

#[derive(Clone, Copy)]
struct FluidAt {
    water: bool,
    own_height: f32,
}

impl<R: Fn(i32, i32, i32) -> Option<i32>> FluidMesher<'_, R> {
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
        let tint = if water { WATER_TINT } else { 0xFFFFFF };
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
            add_face(mesh, corners, uv, 1.0, tint, !self.solid(x, y + 1, z));
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
        let [u0, v0, u1, v1] = self.sprite(flowing);
        let u = |f: f32| u0 + (u1 - u0) * f;
        let v = |f: f32| v0 + (v1 - v0) * f;
        for (direction, render, [nx, nz], h0, h1, [x0, z0], [x1, z1], shade) in sides {
            if !render || self.occluded(direction, h0.max(h1), nx, y, nz) {
                continue;
            }
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
                true,
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
    back_face: bool,
) {
    let base = mesh.vertices.len() as u32;
    for (position, uv) in corners.into_iter().zip(uv) {
        mesh.vertices.push(crate::render::scene::Vertex {
            position,
            uv,
            shade,
            tint,
        });
    }
    mesh.translucent_indices
        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    if back_face {
        mesh.translucent_indices
            .extend([base, base + 3, base + 2, base, base + 2, base + 1]);
    }
}
