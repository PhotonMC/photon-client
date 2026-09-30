//! What every block state works out once (`BlockStateBase` and its
//! `Cache`): its fluid, its occlusion faces, what light and movement read of
//! it. Each value comes from the block's declaration, its shapes and the
//! nearest class in its lineage that overrides the reference's method.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use voxel_core::direction::{Axis, Direction};
use voxel_core::shape::{BooleanOp, Shapes, VoxelShape};
use voxel_data::block::blocks::id;
use voxel_data::block::classes::BlockClass;
use voxel_data::block::properties::{
    EAST, EXTENDED, HORIZONTAL_FACING, IN_WALL, LAYERS, NORTH, OPEN, SLAB_TYPE, SOUTH, WEST,
};
use voxel_data::block::registry::{BLOCKS, Block};

use super::fluid::{FluidState, fluid_state};
use super::shape_rules;
use super::shapes;

/// The directions in the reference's order, which the face arrays follow.
pub const DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// A face occlusion shape: nothing, the whole face, or one of the partial
/// faces numbered after them as they turn up.
pub type Face = u16;
pub const EMPTY_FACE: Face = 0;
pub const FULL_FACE: Face = 1;

/// One block state's cached values.
#[derive(Clone, Debug)]
pub struct StateInfo {
    /// The block's place in the registry.
    pub block: u16,
    /// How far into the block's states this one is.
    pub offset: u16,
    pub fluid: FluidState,
    /// `getFaceOcclusionShape` towards each direction.
    pub faces: [Face; 6],
    pub solid_render: bool,
    pub propagates_skylight_down: bool,
    pub light_dampening: u8,
    pub light_emission: u8,
    pub emissive_rendering: bool,
    pub shade_brightness: f32,
    pub collision_full_block: bool,
    pub use_shape_for_light_occlusion: bool,
    pub can_occlude: bool,
    pub blocks_motion: bool,
    /// `isFaceSturdy` with `SupportType.FULL`, bit `d` for each direction.
    pub sturdy_faces: u8,
    pub suffocating: bool,
    pub view_blocking: bool,
    pub redstone_conductor: bool,
    /// `getShape` and `getCollisionShape` with no level and no entity, as
    /// places in the shape pool ([`States::outline`], [`States::collision`]).
    outline: u16,
    collision: u16,
}

impl StateInfo {
    pub fn block(&self) -> &'static Block {
        &BLOCKS[usize::from(self.block)]
    }

    pub fn face(&self, direction: Direction) -> Face {
        self.faces[direction as usize]
    }

    pub fn is_face_sturdy(&self, direction: Direction) -> bool {
        self.sturdy_faces & (1 << direction as u8) != 0
    }
}

/// Every state's values, and the distinct partial faces with what the face
/// culling asks of each pair of them.
pub struct States {
    states: Vec<StateInfo>,
    /// The distinct outline and collision shapes the states use.
    shapes: Vec<VoxelShape>,
    faces: Vec<VoxelShape>,
    joins: PairCache,
    covers: PairCache,
}

/// A test on every pair of faces, each worked out the first time it is
/// asked, as the reference's occlusion cache does.
struct PairCache {
    test: fn(&VoxelShape, &VoxelShape) -> bool,
    /// Zero for not yet asked, then one plus the answer.
    answers: Vec<AtomicU8>,
}

impl PairCache {
    fn new(count: usize, test: fn(&VoxelShape, &VoxelShape) -> bool) -> Self {
        Self {
            test,
            answers: (0..count * count).map(|_| AtomicU8::new(0)).collect(),
        }
    }

    fn get(&self, faces: &[VoxelShape], shape: Face, occluder: Face) -> bool {
        let slot = &self.answers[usize::from(shape) * faces.len() + usize::from(occluder)];
        match slot.load(Ordering::Relaxed) {
            0 => {
                let answer = (self.test)(&faces[usize::from(shape)], &faces[usize::from(occluder)]);
                slot.store(1 + u8::from(answer), Ordering::Relaxed);
                answer
            }
            known => known == 2,
        }
    }
}

impl States {
    /// A state's values; `None` outside the registry.
    pub fn get(&self, state: u32) -> Option<&StateInfo> {
        self.states.get(state as usize)
    }

    /// `getShape` of a state, with no level and no entity and before the
    /// block's random offset.
    pub fn outline(&self, state: &StateInfo) -> &VoxelShape {
        &self.shapes[usize::from(state.outline)]
    }

    /// `getCollisionShape` of a state, with no level and no entity.
    pub fn collision(&self, state: &StateInfo) -> &VoxelShape {
        &self.shapes[usize::from(state.collision)]
    }

    /// The distinct face shapes, by face number.
    pub fn faces(&self) -> &[VoxelShape] {
        &self.faces
    }

    /// `Shapes.joinIsNotEmpty(shape, occluder, ONLY_FIRST)`: whether face
    /// `shape` has any part `occluder` does not cover.
    pub fn joins(&self, shape: Face, occluder: Face) -> bool {
        self.joins.get(&self.faces, shape, occluder)
    }

    /// `Shapes.faceShapeOccludes(shape, occluder)`: whether the two faces
    /// together cover the whole face.
    pub fn covers(&self, shape: Face, occluder: Face) -> bool {
        self.covers.get(&self.faces, shape, occluder)
    }
}

/// The cache, worked out on first use.
pub fn states() -> &'static States {
    static STATES: OnceLock<States> = OnceLock::new();
    STATES.get_or_init(build)
}

/// A state's values; `None` outside the registry.
pub fn state(state: u32) -> Option<&'static StateInfo> {
    states().get(state)
}

/// A state's values, a number outside the registry reading as air
/// (`Block.stateById`).
pub fn state_or_air(state: i64) -> &'static StateInfo {
    let states = states();
    u32::try_from(state)
        .ok()
        .and_then(|state| states.get(state))
        .unwrap_or(&states.states[0])
}

fn build() -> States {
    let mut faces = Faces::default();
    let mut facts = ShapeFacts::default();
    let mut pool = ShapePool::default();
    let states = BLOCKS
        .iter()
        .enumerate()
        .flat_map(|(index, block)| {
            (0..block.state_count()).map(move |offset| (index, block, offset))
        })
        .map(|(index, block, offset)| {
            compute(index, block, offset, &mut faces, &mut facts, &mut pool)
        })
        .collect();
    let faces = faces.shapes;
    let count = faces.len();
    States {
        states,
        shapes: pool.shapes,
        faces,
        joins: PairCache::new(count, |shape, occluder| {
            Shapes::join_is_not_empty(shape, occluder, BooleanOp::OnlyFirst)
        }),
        covers: PairCache::new(count, Shapes::face_shape_occludes),
    }
}

/// The distinct outline and collision shapes, each held once. The
/// reference's own whole-block and empty shapes are kept apart from shapes
/// that merely have the same boxes, because the reference tells them by
/// identity.
#[derive(Default)]
struct ShapePool {
    shapes: Vec<VoxelShape>,
    places: HashMap<(u8, Vec<u64>), u16>,
}

impl ShapePool {
    fn intern(&mut self, shape: VoxelShape) -> u16 {
        let kind = if shape.equals_java(&Shapes::empty()) {
            1
        } else if shape.equals_java(&Shapes::block()) {
            2
        } else {
            0
        };
        let boxes = shape
            .to_aabbs()
            .iter()
            .flat_map(|aabb| {
                [
                    aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
                ]
                .map(f64::to_bits)
            })
            .collect();
        *self.places.entry((kind, boxes)).or_insert_with(|| {
            self.shapes.push(shape);
            u16::try_from(self.shapes.len() - 1).expect("fewer than 65536 shapes")
        })
    }
}

/// What the reference caches per shape object (`SHAPE_FULL_BLOCK_CACHE` and
/// a shape's own face cache), kept with the shape so its identity stays
/// unique while it is a key.
#[derive(Default)]
struct ShapeFacts {
    known: HashMap<usize, (VoxelShape, Facts)>,
}

#[derive(Clone, Copy, Default)]
struct Facts {
    full: bool,
    /// `isFaceFull` towards each direction, bit `d`.
    full_faces: u8,
    /// The face numbers, once the shape has been an occlusion shape.
    faces: Option<[Face; 6]>,
}

impl ShapeFacts {
    fn of(&mut self, shape: &VoxelShape) -> Facts {
        self.known
            .entry(shape.identity())
            .or_insert_with(|| {
                let full_faces = DIRECTIONS
                    .iter()
                    .enumerate()
                    .filter(|(_, direction)| is_shape_full_block(&shape.face_shape(**direction)))
                    .fold(0, |bits, (slot, _)| bits | 1 << slot);
                let facts = Facts {
                    full: is_shape_full_block(shape),
                    full_faces,
                    faces: None,
                };
                (shape.clone(), facts)
            })
            .1
    }

    /// The face numbers of an occlusion shape (`occlusionShapesByFace`).
    fn faces(&mut self, shape: &VoxelShape, faces: &mut Faces) -> [Face; 6] {
        let facts = self.of(shape);
        if let Some(known) = facts.faces {
            return known;
        }
        let numbers = if shape.is_empty() {
            [EMPTY_FACE; 6]
        } else if facts.full {
            [FULL_FACE; 6]
        } else {
            DIRECTIONS.map(|direction| faces.number(shape.face_shape(direction)))
        };
        if let Some((_, facts)) = self.known.get_mut(&shape.identity()) {
            facts.faces = Some(numbers);
        }
        numbers
    }
}

/// The distinct face shapes seen so far. Nothing and the whole face are the
/// reference's own shapes, told apart by identity; a partial face is keyed
/// by its boxes.
struct Faces {
    shapes: Vec<VoxelShape>,
    numbers: HashMap<Vec<u64>, Face>,
}

impl Default for Faces {
    fn default() -> Self {
        Self {
            shapes: vec![Shapes::empty(), Shapes::block()],
            numbers: HashMap::new(),
        }
    }
}

impl Faces {
    fn number(&mut self, shape: VoxelShape) -> Face {
        if shape.equals_java(&Shapes::empty()) {
            return EMPTY_FACE;
        }
        if shape.equals_java(&Shapes::block()) {
            return FULL_FACE;
        }
        let key = shape
            .to_aabbs()
            .iter()
            .flat_map(|aabb| {
                [
                    aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
                ]
                .map(f64::to_bits)
            })
            .collect();
        *self.numbers.entry(key).or_insert_with(|| {
            self.shapes.push(shape);
            Face::try_from(self.shapes.len() - 1).expect("fewer than 65536 faces")
        })
    }
}

/// `Block.isShapeFullBlock`.
pub fn is_shape_full_block(shape: &VoxelShape) -> bool {
    !Shapes::join_is_not_empty(&Shapes::block(), shape, BooleanOp::NotSame)
}

fn horizontal_axis(block: &Block, offset: u32) -> Axis {
    match block.value(offset, &HORIZONTAL_FACING) {
        Some("east" | "west") => Axis::X,
        _ => Axis::Z,
    }
}

fn compute(
    index: usize,
    block: &'static Block,
    offset: u32,
    faces: &mut Faces,
    facts: &mut ShapeFacts,
    pool: &mut ShapePool,
) -> StateInfo {
    let fluid = fluid_state(block, offset);
    let outline_place = pool.intern(shape_rules::outline(block, offset));
    let collision_place = pool.intern(shape_rules::collision(block, offset));
    let outline = pool.shapes[usize::from(outline_place)].clone();
    let collision = pool.shapes[usize::from(collision_place)].clone();
    let occlusion = occlusion_shape(block, offset, &outline, &collision);
    let solid_render = facts.of(&occlusion).full;
    let face_numbers = facts.faces(&occlusion, faces);
    let propagates_skylight_down =
        propagates_skylight_down(block, offset, facts.of(&outline).full, fluid);
    let collision_full_block = collision_full_block(block, facts.of(&collision).full);
    let blocks_motion =
        index != id::COBWEB && index != id::BAMBOO_SAPLING && legacy_solid(block, &collision);
    let support = support_shape(block, offset, &outline, &collision);
    let sturdy_faces = facts.of(&support).full_faces;
    let fallback = blocks_motion && collision_full_block;
    StateInfo {
        block: u16::try_from(index).expect("fewer than 65536 blocks"),
        offset: u16::try_from(offset).expect("fewer than 65536 states a block"),
        fluid,
        faces: face_numbers,
        solid_render,
        propagates_skylight_down,
        light_dampening: light_dampening(block, solid_render, propagates_skylight_down),
        light_emission: block.light_emission.value(offset),
        emissive_rendering: block.emissive_rendering.value(offset).unwrap_or(false),
        shade_brightness: shade_brightness(block, offset, collision_full_block),
        collision_full_block,
        use_shape_for_light_occlusion: use_shape_for_light_occlusion(block, offset),
        can_occlude: block.can_occlude,
        blocks_motion,
        sturdy_faces,
        suffocating: block.suffocating.value(offset).unwrap_or(fallback),
        view_blocking: block.view_blocking.value(offset).unwrap_or(fallback),
        redstone_conductor: block
            .redstone_conductor
            .value(offset)
            .unwrap_or(collision_full_block),
        outline: outline_place,
        collision: collision_place,
    }
}

/// `canOcclude ? getOcclusionShape(state) : Shapes.empty()`.
fn occlusion_shape(
    block: &Block,
    offset: u32,
    outline: &VoxelShape,
    collision: &VoxelShape,
) -> VoxelShape {
    if !block.can_occlude {
        return Shapes::empty();
    }
    for class in block.class.lineage() {
        match class {
            BlockClass::FenceBlock => {
                return shapes::fence_occlusion(|direction| {
                    let property = match direction {
                        Direction::North => &NORTH,
                        Direction::East => &EAST,
                        Direction::South => &SOUTH,
                        _ => &WEST,
                    };
                    block.flag(offset, property)
                });
            }
            BlockClass::FenceGateBlock => {
                return shapes::fence_gate_occlusion(
                    horizontal_axis(block, offset),
                    block.flag(offset, &IN_WALL),
                );
            }
            // Their `SHAPE_COLLISION`, which is also their collision shape.
            BlockClass::LecternBlock | BlockClass::SculkShriekerBlock => return collision.clone(),
            _ => {}
        }
    }
    outline.clone()
}

/// `propagatesSkylightDown`.
fn propagates_skylight_down(
    block: &Block,
    offset: u32,
    full_outline: bool,
    fluid: FluidState,
) -> bool {
    for class in block.class.lineage() {
        match class {
            BlockClass::BambooStalkBlock
            | BlockClass::HangingMossBlock
            | BlockClass::MossyCarpetBlock
            | BlockClass::TransparentBlock
            | BlockClass::VineBlock => return true,
            BlockClass::BarrierBlock
            | BlockClass::GlowLichenBlock
            | BlockClass::LightBlock
            | BlockClass::VegetationBlock => return fluid.is_empty(),
            BlockClass::CrossCollisionBlock | BlockClass::WallBlock => {
                return !block.flag(offset, &voxel_data::block::properties::WATERLOGGED);
            }
            BlockClass::LiquidBlock
            | BlockClass::PipeBlock
            | BlockClass::ShulkerBoxBlock
            | BlockClass::TintedGlassBlock => return false,
            _ => {}
        }
    }
    !full_outline && fluid.is_empty()
}

/// `getLightDampening`.
fn light_dampening(block: &Block, solid_render: bool, propagates_skylight_down: bool) -> u8 {
    for class in block.class.lineage() {
        match class {
            BlockClass::LeavesBlock => return 1,
            BlockClass::TintedGlassBlock => return 15,
            _ => {}
        }
    }
    if solid_render {
        15
    } else if propagates_skylight_down {
        0
    } else {
        1
    }
}

/// `getShadeBrightness`.
fn shade_brightness(block: &Block, offset: u32, collision_full_block: bool) -> f32 {
    for class in block.class.lineage() {
        match class {
            BlockClass::BarrierBlock
            | BlockClass::LightBlock
            | BlockClass::StructureVoidBlock
            | BlockClass::TransparentBlock => return 1.0,
            BlockClass::MudBlock | BlockClass::SoulSandBlock => return 0.2,
            BlockClass::SnowLayerBlock => {
                return if block.value(offset, &LAYERS) == Some("8") {
                    0.2
                } else {
                    1.0
                };
            }
            _ => {}
        }
    }
    if collision_full_block { 0.2 } else { 1.0 }
}

/// `useShapeForLightOcclusion`.
fn use_shape_for_light_occlusion(block: &Block, offset: u32) -> bool {
    for class in block.class.lineage() {
        match class {
            BlockClass::DaylightDetectorBlock
            | BlockClass::DirtPathBlock
            | BlockClass::EnchantingTableBlock
            | BlockClass::EndPortalFrameBlock
            | BlockClass::FarmlandBlock
            | BlockClass::LecternBlock
            | BlockClass::SculkSensorBlock
            | BlockClass::SculkShriekerBlock
            | BlockClass::ShelfBlock
            | BlockClass::SnowLayerBlock
            | BlockClass::StairBlock
            | BlockClass::StonecutterBlock
            | BlockClass::PistonHeadBlock => return true,
            BlockClass::SlabBlock => return block.value(offset, &SLAB_TYPE) != Some("double"),
            BlockClass::PistonBaseBlock => return block.flag(offset, &EXTENDED),
            _ => {}
        }
    }
    false
}

/// `isCollisionShapeFullBlock`.
fn collision_full_block(block: &Block, full_collision: bool) -> bool {
    for class in block.class.lineage() {
        if matches!(
            class,
            BlockClass::BambooStalkBlock | BlockClass::SpeleothemBlock
        ) {
            return false;
        }
    }
    full_collision
}

/// `BlockStateBase.calculateSolid`: the deprecated "solid" that
/// `blocksMotion` reads. A block with a dynamic shape has no cache and so
/// never counts as solid unless forced.
fn legacy_solid(block: &Block, collision: &VoxelShape) -> bool {
    if block.force_solid_on {
        return true;
    }
    if block.force_solid_off || block.dynamic_shape {
        return false;
    }
    collision
        .bounds()
        .is_some_and(|bounds| bounds.size() >= 0.729_166_666_666_666_6 || bounds.y_size() >= 1.0)
}

/// `getBlockSupportShape` with no block entity at the origin.
fn support_shape(
    block: &Block,
    offset: u32,
    outline: &VoxelShape,
    collision: &VoxelShape,
) -> VoxelShape {
    for class in block.class.lineage() {
        match class {
            BlockClass::CeilingHangingSignBlock
            | BlockClass::WallHangingSignBlock
            | BlockClass::SnowLayerBlock => return outline.clone(),
            BlockClass::ChorusFlowerBlock => return shapes::chorus_flower_support(),
            BlockClass::FenceGateBlock => {
                return if block.flag(offset, &OPEN) {
                    Shapes::empty()
                } else {
                    shapes::fence_gate_support(horizontal_axis(block, offset))
                };
            }
            BlockClass::LeavesBlock => return Shapes::empty(),
            BlockClass::MudBlock | BlockClass::SoulSandBlock | BlockClass::ShulkerBoxBlock => {
                return Shapes::block();
            }
            _ => {}
        }
    }
    collision.clone()
}
