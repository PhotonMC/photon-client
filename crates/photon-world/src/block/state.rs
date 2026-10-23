//! What every block state works out once (`BlockStateBase` and its
//! `Cache`): its fluid, its occlusion faces, what light and movement read of
//! it. Each value comes from the block's declaration, its shapes and the
//! nearest class in its lineage that overrides the reference's method.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use rayon::prelude::*;

use photon_core::direction::{Axis, Direction};
use photon_core::shape::{BooleanOp, Shapes, VoxelShape};
use photon_data::block::blocks::id;
use photon_data::block::classes::BlockClass;
use photon_data::block::properties::{
    EAST, EXTENDED, HORIZONTAL_FACING, IN_WALL, LAYERS, NORTH, OPEN, SLAB_TYPE, SOUTH, WEST,
};
use photon_data::block::registry::{BLOCKS, Block};

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

/// `SupportType`: how much of a face must carry a block placed against it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportType {
    Full,
    Center,
    Rigid,
}

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
    /// `isFaceSturdy` with `SupportType.CENTER` and `SupportType.RIGID`, the same bits.
    pub center_faces: u8,
    pub rigid_faces: u8,
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

    /// `isFaceSturdy(direction, supportType)`.
    pub fn is_face_sturdy_for(&self, direction: Direction, support: SupportType) -> bool {
        let bits = match support {
            SupportType::Full => self.sturdy_faces,
            SupportType::Center => self.center_faces,
            SupportType::Rigid => self.rigid_faces,
        };
        bits & (1 << direction as u8) != 0
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
    pub fn get(&self, state: photon_core::ids::BlockStateId) -> Option<&StateInfo> {
        self.states.get(usize::try_from(state.0).ok()?)
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
    /// `SupportType.CENTER` and `SupportType.RIGID` towards each direction, bit `d`.
    center_faces: u8,
    rigid_faces: u8,
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
                let supporting = |wanted: &VoxelShape| {
                    DIRECTIONS
                        .iter()
                        .enumerate()
                        .filter(|(_, direction)| {
                            !Shapes::join_is_not_empty(
                                &shape.face_shape(**direction),
                                wanted,
                                BooleanOp::OnlySecond,
                            )
                        })
                        .fold(0, |bits, (slot, _)| bits | 1 << slot)
                };
                let facts = Facts {
                    full: is_shape_full_block(shape),
                    full_faces,
                    center_faces: supporting(&shapes::center_support()),
                    rigid_faces: supporting(&shapes::rigid_support()),
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

mod shape;
pub use shape::*;
