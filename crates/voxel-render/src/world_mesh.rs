//! Turning loaded chunks into something the renderer can draw.
//!
//! The server sends a block state as one number. The registry turns that back
//! into a block and its properties, the block's blockstate file picks a
//! variant, and the variant names a model. This walks a box of the world and
//! does that for every position in it.

use std::collections::{BTreeMap, HashMap};

use crate::bake::{BakeKey, BakedModel};
use crate::resolve::{AssetSource, BuiltAtlas, ResolveError, ResolvedModel};
use crate::scene::Mesh;
use voxel_core::direction::Direction;
use voxel_core::identifier::Identifier;
use voxel_data::block::registry;

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

/// A quad's four corners in the model's unit cube, and the way it faces.
pub type FacedQuad = ([[f32; 3]; 4], Direction);

/// Every block's models and the one atlas they are drawn from, prepared
/// once, the way the reference bakes its block models at resource load.
pub struct BlockRenderer {
    models: BTreeMap<String, ResolvedModel>,
    /// Every variant's model, baked, by the index the states name.
    baked: Vec<BakedModel>,
    /// Each item's client item model tree, by registry id.
    items: Vec<Option<crate::item_model::ClientItem>>,
    /// The alpha of every texture a flat item extrudes, with its size.
    layer_alpha: BTreeMap<String, crate::item_model::LayerAlpha>,
    /// What each block state draws, by state id; after the registry's come the item frames'
    /// fake states ([`Self::item_frame_state`]).
    states: Vec<StateRender>,
    /// The id of the first fake state (the number of states the registry has).
    frame_base: u32,
    /// The block atlas, with the fluid and sky sprites too.
    pub atlas: BuiltAtlas,
    /// Blocks or models that could not be prepared, with why.
    pub skipped: Vec<(String, String)>,
}

/// The blockstate files that name no block: the item frames' (`BlockStateDefinitions`), each with
/// a `map` property.
const ITEM_FRAME_FILES: [&str; 2] = ["minecraft:item_frame", "minecraft:glow_item_frame"];

/// What one block state draws, worked out once at load rather than for
/// every block.
enum StateRender {
    Air,
    /// Not air, but nothing to draw it with.
    Missing,
    Drawn(DrawnState),
}

struct DrawnState {
    /// The weighted lists its variants are picked from.
    lists: Vec<Vec<BakedVariant>>,
    /// Whether every quad of every model it can draw hides behind a side,
    /// so a block closed in on all six sides draws nothing.
    closes_in: bool,
    /// Where each tint slot's colour comes from.
    tints: StateTints,
}

/// A variant as the random pick sees it, with its baked model.
struct BakedVariant {
    /// Index into [`BlockRenderer::baked`]; `None` when the model could not
    /// be baked, which still takes its draw.
    model: Option<u32>,
    weight: i32,
    /// Whether it was written inside a list, and so drawn for.
    listed: bool,
}

/// `BlockColors` for a state: fixed colours, or colours read from the
/// biome around the block.
enum StateTints {
    Fixed(Vec<u32>),
    Biome {
        layers: &'static [Option<crate::biome_tint::TintKind>],
        constant: &'static [u32],
        /// A tall plant's upper half takes the colour of the block below.
        below: bool,
    },
}

impl StateTints {
    /// The reference's colour providers for a block's state
    /// (`BlockColors.createDefault`).
    fn of(block: &str, properties: &BTreeMap<String, String>) -> Self {
        // The lily pad has a world colour of its own.
        if block == "minecraft:lily_pad" {
            return Self::Fixed(vec![crate::tint::LILY_PAD_IN_WORLD]);
        }
        if let Some(tint) = crate::tint::state_tint(block, properties) {
            return Self::Fixed(vec![tint]);
        }
        let constant = registry::tints(block);
        match crate::biome_tint::tint_layers(block) {
            None => Self::Fixed(constant.to_vec()),
            Some(layers) => Self::Biome {
                layers,
                constant,
                below: properties.get("half").map(String::as_str) == Some("upper")
                    && matches!(block, "minecraft:tall_grass" | "minecraft:large_fern"),
            },
        }
    }

    /// Each slot's colour for the block at `position`, into `out`.
    fn colours(
        &self,
        position: [i32; 3],
        biome_tint: &impl Fn(crate::biome_tint::TintKind, [i32; 3]) -> u32,
        out: &mut Vec<u32>,
    ) {
        out.clear();
        match self {
            Self::Fixed(colours) => out.extend_from_slice(colours),
            Self::Biome {
                layers,
                constant,
                below,
            } => {
                let sample = if *below {
                    [position[0], position[1] - 1, position[2]]
                } else {
                    position
                };
                out.extend(layers.iter().enumerate().map(|(slot, layer)| match layer {
                    Some(kind) => biome_tint(*kind, sample),
                    None => constant.get(slot).copied().unwrap_or(0xFFFF_FFFF),
                }));
            }
        }
    }
}

/// A state's variant lists and colours, before they are baked.
type PendingState = Option<(Vec<Vec<crate::blockstate::Variant>>, StateTints)>;

/// One layer of an item as drawn in one display context.
pub struct ItemLayer {
    /// Quads in the model's unit cube, tinted, with their facings.
    pub mesh: Mesh,
    /// Whether the model lights like a block (`gui_light: side`) rather
    /// than flat from the front.
    pub side_lit: bool,
    /// The context's display transform; `None` is no transform.
    pub transform: Option<crate::model::ItemTransform>,
}

/// One meshed region.
pub struct RegionMesh {
    pub mesh: Mesh,
    /// How many positions held something other than air.
    pub solid_count: usize,
}

/// What meshing reads of the world around a region: block states, light,
/// biome colours and the server's block tags.
pub struct MeshWorld<'a, R, L, T> {
    pub read: &'a R,
    pub light: &'a L,
    pub biome_tint: &'a T,
    pub tags: &'a voxel_world::block::tags::BlockTags,
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
    biome_tint: &'a dyn Fn(crate::biome_tint::TintKind, [i32; 3]) -> u32,
    read: &'a R,
    lighter: &'a crate::block_light::Lighter<S, L>,
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
        crate::block_light::light_uv((block << 4) | (sky << 20))
    }

    fn fluid(&self, x: i32, y: i32, z: i32) -> Option<FluidAt> {
        let state = (self.read)(x, y, z)?;
        let (water, own_height, _) = voxel_world::local_player::state_fluid(state)?;
        Some(FluidAt { water, own_height })
    }

    fn solid(&self, x: i32, y: i32, z: i32) -> bool {
        (self.read)(x, y, z)
            .and_then(|state| u32::try_from(state).ok())
            .and_then(voxel_world::block::state)
            .is_some_and(|state| {
                voxel_world::block::states()
                    .collision(state)
                    .equals_java(&voxel_core::shape::Shapes::block())
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
        let face = (self.read)(x, y, z)
            .and_then(|state| u32::try_from(state).ok())
            .and_then(voxel_world::block::state)
            .map(|state| state.face(direction.opposite()));
        face == Some(voxel_world::block::state::FULL_FACE)
            && (direction != Direction::Up || height == 1.0)
    }

    /// Whether a water side against this block shows the overlay sprite:
    /// the reference's half-transparent blocks and leaves.
    fn shows_overlay(&self, x: i32, y: i32, z: i32) -> bool {
        (self.read)(x, y, z)
            .and_then(|state| u32::try_from(state).ok())
            .and_then(voxel_world::block::state)
            .is_some_and(|state| voxel_world::block::shows_fluid_overlay(state.block()))
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

    #[expect(
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
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
            (self.biome_tint)(crate::biome_tint::TintKind::Water, [x, y, z])
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
                let angle = voxel_core::math::mth_atan2(flow_z, flow_x) as f32
                    - std::f32::consts::FRAC_PI_2;
                let s = voxel_core::math::mth_sin_f64(f64::from(angle)) * 0.25;
                let c = voxel_core::math::mth_cos_f64(f64::from(angle)) * 0.25;
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

mod block_renderer_blocks;
mod block_renderer_item;

mod item;
pub use item::*;
mod model;
use model::*;
