use super::*;
use photon_core::ids::BlockStateId;

/// The sprites the fluid renderer draws with, which no block model names.
pub(super) const FLUID_SPRITES: [&str; 5] = [
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
pub(super) struct FluidMesher<'a, R: Fn(i32, i32, i32) -> Option<BlockStateId>, S, L> {
    pub(super) biome_tint: &'a dyn Fn(crate::block::biome_tint::TintKind, [i32; 3]) -> u32,
    pub(super) read: &'a R,
    pub(super) lighter: &'a crate::block::light::Lighter<S, L>,
    pub(super) atlas: &'a BuiltAtlas,
    pub(super) origin: [i32; 3],
    pub(super) cardinal: photon_world::CardinalLightType,
}

#[derive(Clone, Copy)]
pub(super) struct FluidAt {
    pub(super) water: bool,
    pub(super) own_height: f32,
}

impl<R, S, L> FluidMesher<'_, R, S, L>
where
    R: Fn(i32, i32, i32) -> Option<BlockStateId>,
    S: Fn([i32; 3]) -> Option<photon_core::ids::BlockStateId>,
    L: Fn([i32; 3]) -> (u8, u8),
{
    /// `FluidRenderer.getLightCoords`: the brighter of here and above.
    pub(super) fn fluid_light(&self, x: i32, y: i32, z: i32) -> [f32; 2] {
        let at = |p: [i32; 3]| self.lighter.light_coords((self.lighter.state)(p), p);
        let here = at([x, y, z]);
        let above = at([x, y + 1, z]);
        let block = ((here >> 4) & 15).max((above >> 4) & 15);
        let sky = ((here >> 20) & 15).max((above >> 20) & 15);
        crate::block::light::light_uv((block << 4) | (sky << 20))
    }

    pub(super) fn fluid(&self, x: i32, y: i32, z: i32) -> Option<FluidAt> {
        let state = (self.read)(x, y, z)?;
        let (water, own_height, _) = photon_world::local_player::state_fluid(state)?;
        Some(FluidAt { water, own_height })
    }

    pub(super) fn solid(&self, x: i32, y: i32, z: i32) -> bool {
        (self.read)(x, y, z)
            .and_then(photon_world::block::state)
            .is_some_and(|state| {
                photon_world::block::states()
                    .collision(state)
                    .equals_java(&photon_core::shape::Shapes::block())
            })
    }

    pub(super) fn same(&self, water: bool, x: i32, y: i32, z: i32) -> bool {
        self.fluid(x, y, z)
            .is_some_and(|other| other.water == water)
    }

    /// A neighbour's height for the corner average: the fluid's own height,
    /// a full block under the same fluid, nothing for an open neighbour and
    /// a negative mark for a solid one.
    /// `BlockState.isSolidRender`: a block whose occlusion shape is the whole
    /// block, which glass and leaves are not despite their full collision.
    fn solid_render(&self, x: i32, y: i32, z: i32) -> bool {
        (self.read)(x, y, z)
            .and_then(photon_world::block::state)
            .is_some_and(|state| state.solid_render)
    }

    /// `FluidState.shouldRenderBackwardUpFace`: the surface is drawn again
    /// from below when any column around the block above it holds something
    /// that is neither this fluid nor solid.
    fn shows_backward_up_face(&self, water: bool, x: i32, y: i32, z: i32) -> bool {
        (-1..=1).any(|offset_x| {
            (-1..=1).any(|offset_z| {
                let (x, z) = (x + offset_x, z + offset_z);
                !self.same(water, x, y + 1, z) && !self.solid_render(x, y + 1, z)
            })
        })
    }

    pub(super) fn height(&self, water: bool, x: i32, y: i32, z: i32) -> f32 {
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

    pub(super) fn average_height(
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

    /// `FluidRenderer`'s shade of the top and bottom faces.
    fn shade(&self, direction: Direction) -> f32 {
        crate::block::light::cardinal(direction, self.cardinal)
    }

    /// The shade of a side face: the top's times its axis' own.
    fn side_shade(&self, direction: Direction) -> f32 {
        self.shade(Direction::Up) * self.shade(direction)
    }

    /// Whether the neighbour hides the fluid's face towards it, from the
    /// neighbour's face occlusion shape. A whole-block shape hides every face
    /// but a surface below full height; a partial one is treated as hiding
    /// nothing, which is where this still differs from the reference.
    pub(super) fn occluded(
        &self,
        direction: Direction,
        height: f32,
        x: i32,
        y: i32,
        z: i32,
    ) -> bool {
        let face = (self.read)(x, y, z)
            .and_then(photon_world::block::state)
            .map(|state| state.face(direction.opposite()));
        face == Some(photon_world::block::state::FULL_FACE)
            && (direction != Direction::Up || height == 1.0)
    }

    /// Whether a water side against this block shows the overlay sprite:
    /// the reference's half-transparent blocks and leaves.
    pub(super) fn shows_overlay(&self, x: i32, y: i32, z: i32) -> bool {
        (self.read)(x, y, z)
            .and_then(photon_world::block::state)
            .is_some_and(|state| photon_world::block::shows_fluid_overlay(state.block()))
    }

    /// The horizontal flow direction, which is all the surface texture uses.
    pub(super) fn flow(&self, fluid: FluidAt, x: i32, y: i32, z: i32) -> (f64, f64) {
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

    pub(super) fn sprite(&self, name: &str) -> [f32; 4] {
        self.atlas.sprites[name]
    }

    /// Whether a fluid's faces go to the translucent layer: any of its sprites
    /// (the overlay too, for water) has a texel that is partly transparent
    /// (`FluidModel.Unbaked.bake`). The other layers share the opaque pass.
    fn translucent(&self, water: bool) -> bool {
        let sprites: &[&str] = if water {
            &[FLUID_SPRITES[0], FLUID_SPRITES[1], FLUID_SPRITES[2]]
        } else {
            &[FLUID_SPRITES[3], FLUID_SPRITES[4]]
        };
        sprites.iter().any(|name| {
            self.atlas
                .alpha
                .get(*name)
                .is_some_and(|alpha| alpha.transparency.has_translucent)
        })
    }

    /// The block's faces, in the pass its sprites' alpha asks for.
    pub(super) fn tesselate_in_layer(&self, pos: [i32; 3], mesh: &mut Mesh) {
        let before = mesh.translucent_indices.len();
        self.tesselate(pos, mesh);
        let Some(fluid) = self.fluid(pos[0], pos[1], pos[2]) else {
            return;
        };
        if !self.translucent(fluid.water) {
            let opaque = mesh.translucent_indices.split_off(before);
            mesh.indices.extend(opaque);
        }
    }

    pub(super) fn tesselate(&self, pos: [i32; 3], mesh: &mut Mesh) {
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
            (self.biome_tint)(crate::block::biome_tint::TintKind::Water, [x, y, z])
        } else {
            0xFFFFFF
        };
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
                let angle = photon_core::math::mth_atan2(flow_z, flow_x) as f32
                    - std::f32::consts::FRAC_PI_2;
                let s = photon_core::math::mth_sin_f64(f64::from(angle)) * 0.25;
                let c = photon_core::math::mth_cos_f64(f64::from(angle)) * 0.25;
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
            // Seen from below, the surface is drawn again facing down.
            let light = self.fluid_light(x, y, z);
            add_face(
                mesh,
                corners,
                uv,
                (1.0, scaled_tint(tint, self.shade(Direction::Up))),
                light,
                self.shows_backward_up_face(water, x, y, z),
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
                (1.0, scaled_tint(tint, self.shade(Direction::Down))),
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
                self.side_shade(Direction::North),
            ),
            (
                Direction::South,
                render_south,
                [x, z + 1],
                south_east,
                south_west,
                [lx + 1.0, lz + 0.999],
                [lx, lz + 0.999],
                self.side_shade(Direction::North),
            ),
            (
                Direction::West,
                render_west,
                [x - 1, z],
                south_west,
                north_west,
                [lx + 0.001, lz + 1.0],
                [lx + 0.001, lz],
                self.side_shade(Direction::West),
            ),
            (
                Direction::East,
                render_east,
                [x + 1, z],
                north_east,
                south_east,
                [lx + 0.999, lz],
                [lx + 0.999, lz + 1.0],
                self.side_shade(Direction::West),
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
                (1.0, scaled_tint(tint, shade)),
                self.fluid_light(x, y, z),
                !overlay,
            );
        }
    }
}

/// The tint with the shade applied the way `ARGB.scaleRGB` applies it: each
/// channel scaled, then truncated to a whole number, where a face's own
/// shading would round it. Meant for `add_face` with a shade of one.
fn scaled_tint(tint: u32, shade: f32) -> [f32; 4] {
    let scaled = |shift: u32| {
        let channel = ((tint >> shift) & 0xff) as f32;
        (channel * shade).clamp(0.0, 255.0).trunc() / 255.0
    };
    [scaled(16), scaled(8), scaled(0), 1.0]
}

/// What a fluid block is meshed in: the level to read and light, the atlas,
/// the dimension's cardinal light and the water tint every position takes.
pub struct FluidView<'a, R, S, L> {
    pub read: &'a R,
    pub lighter: &'a crate::block::light::Lighter<S, L>,
    pub atlas: &'a BuiltAtlas,
    pub cardinal: photon_world::CardinalLightType,
    pub origin: [i32; 3],
    pub water_tint: u32,
}

/// The fluid block at `pos` meshed on its own, for a scene the caller lights
/// and reads: what `FluidRenderer` would add, placed relative to the view's
/// origin.
pub fn fluid_vertices<R, S, L>(view: &FluidView<'_, R, S, L>, pos: [i32; 3]) -> Mesh
where
    R: Fn(i32, i32, i32) -> Option<BlockStateId>,
    S: Fn([i32; 3]) -> Option<BlockStateId>,
    L: Fn([i32; 3]) -> (u8, u8),
{
    let tint = |kind: crate::block::biome_tint::TintKind, _: [i32; 3]| match kind {
        crate::block::biome_tint::TintKind::Water => view.water_tint,
        _ => 0xFF_FFFF,
    };
    let mesher = FluidMesher {
        biome_tint: &tint,
        read: view.read,
        lighter: view.lighter,
        atlas: view.atlas,
        origin: view.origin,
        cardinal: view.cardinal,
    };
    let mut mesh = Mesh::default();
    mesher.tesselate_in_layer(pos, &mut mesh);
    mesh
}
