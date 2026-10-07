//! Axis-aligned bounding boxes for the first P01.d geometry slice.
//!
//! This covers box constructors/production helpers, the integer BoundingBox bridge,
//! AABB.Builder point accumulation, constructor normalization
//! and the contains/intersects,
//! expand-towards, equals/hashCode, toString, contract, inflate, intersect, minmax, move, sizes, has_nan,
//! positions, axis endpoint queries, `of_size`, endpoint setters, move overloads,
//! deflate, segment clipping (including raw-coordinate bounds and iterable hit
//! results) and center-segment collision operations used by the current behavior
//! corpus.

use crate::{
    binary::BlockPos,
    direction::{Axis, Direction},
    math::{Vec3, java_double_compare, java_double_hash, java_double_to_string, java_negate},
};

/// Integer-aligned bounds used by structure and world-generation geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    /// Construct bounds, ordering each axis as Java's BoundingBox does.
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            min_z: min_z.min(max_z),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
            max_z: min_z.max(max_z),
        }
    }
}

/// A block hit produced by AABB or VoxelShape clipping.
#[derive(Debug, Clone, Copy)]
pub struct BlockHitResult {
    pub location: Vec3,
    pub direction: Direction,
    pub block_pos: BlockPos,
    pub inside: bool,
    pub world_border_hit: bool,
}

impl BlockHitResult {
    pub(crate) fn new(
        location: Vec3,
        direction: Direction,
        block_pos: BlockPos,
        inside: bool,
    ) -> Self {
        Self {
            location,
            direction,
            block_pos,
            inside,
            world_border_hit: false,
        }
    }
}

/// An axis-aligned box with normalized inclusive lower and exclusive upper
/// query boundaries, matching the Java reference's stored endpoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

/// Error returned when an AABB builder has not received a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AabbBuilderError {
    Undefined,
}

/// Accumulate the bounds of f32 points like Java's `AABB.Builder`.
#[derive(Debug, Clone, Copy)]
pub struct AabbBuilder {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
    defined: bool,
}

impl AabbBuilder {
    /// Create an undefined builder with Java's infinite sentinels.
    pub fn new() -> Self {
        Self {
            min_x: f32::INFINITY,
            min_y: f32::INFINITY,
            min_z: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            max_y: f32::NEG_INFINITY,
            max_z: f32::NEG_INFINITY,
            defined: false,
        }
    }

    /// Include one f32 point from a Java `Vector3fc`-compatible value.
    pub fn include(&mut self, x: f32, y: f32, z: f32) {
        self.min_x = java_min_f32(self.min_x, x);
        self.min_y = java_min_f32(self.min_y, y);
        self.min_z = java_min_f32(self.min_z, z);
        self.max_x = java_max_f32(self.max_x, x);
        self.max_y = java_max_f32(self.max_y, y);
        self.max_z = java_max_f32(self.max_z, z);
        self.defined = true;
    }

    /// Return whether at least one point has been included.
    pub fn is_defined(&self) -> bool {
        self.defined
    }

    /// Build the accumulated double-precision AABB.
    pub fn build(&self) -> Result<Aabb, AabbBuilderError> {
        if !self.defined {
            return Err(AabbBuilderError::Undefined);
        }
        Ok(Aabb::new(
            f64::from(self.min_x),
            f64::from(self.min_y),
            f64::from(self.min_z),
            f64::from(self.max_x),
            f64::from(self.max_y),
            f64::from(self.max_z),
        ))
    }
}

impl Default for AabbBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct ClipState {
    from: Vec3,
    dx: f64,
    dy: f64,
    dz: f64,
    scale: f64,
    direction: Option<Direction>,
}

impl ClipState {
    fn new(from: Vec3, to: Vec3) -> Self {
        Self {
            from,
            dx: to.x - from.x,
            dy: to.y - from.y,
            dz: to.z - from.z,
            scale: 1.0,
            direction: None,
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "kept whole to read against the reference method line by line"
)]
fn clip_bounds(min: Vec3, max: Vec3, state: &mut ClipState) {
    if state.dx > 1.0e-7 {
        if clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dx,
                db: state.dy,
                dc: state.dz,
                point: min.x,
                min_b: min.y,
                max_b: max.y,
                min_c: min.z,
                max_c: max.z,
                from_a: state.from.x,
                from_b: state.from.y,
                from_c: state.from.z,
            },
        ) {
            state.direction = Some(Direction::West);
        }
    } else if state.dx < -1.0e-7
        && clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dx,
                db: state.dy,
                dc: state.dz,
                point: max.x,
                min_b: min.y,
                max_b: max.y,
                min_c: min.z,
                max_c: max.z,
                from_a: state.from.x,
                from_b: state.from.y,
                from_c: state.from.z,
            },
        )
    {
        state.direction = Some(Direction::East);
    }

    if state.dy > 1.0e-7 {
        if clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dy,
                db: state.dz,
                dc: state.dx,
                point: min.y,
                min_b: min.z,
                max_b: max.z,
                min_c: min.x,
                max_c: max.x,
                from_a: state.from.y,
                from_b: state.from.z,
                from_c: state.from.x,
            },
        ) {
            state.direction = Some(Direction::Down);
        }
    } else if state.dy < -1.0e-7
        && clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dy,
                db: state.dz,
                dc: state.dx,
                point: max.y,
                min_b: min.z,
                max_b: max.z,
                min_c: min.x,
                max_c: max.x,
                from_a: state.from.y,
                from_b: state.from.z,
                from_c: state.from.x,
            },
        )
    {
        state.direction = Some(Direction::Up);
    }

    if state.dz > 1.0e-7 {
        if clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dz,
                db: state.dx,
                dc: state.dy,
                point: min.z,
                min_b: min.x,
                max_b: max.x,
                min_c: min.y,
                max_c: max.y,
                from_a: state.from.z,
                from_b: state.from.x,
                from_c: state.from.y,
            },
        ) {
            state.direction = Some(Direction::North);
        }
    } else if state.dz < -1.0e-7
        && clip_point(
            &mut state.scale,
            ClipPoint {
                da: state.dz,
                db: state.dx,
                dc: state.dy,
                point: max.z,
                min_b: min.x,
                max_b: max.x,
                min_c: min.y,
                max_c: max.y,
                from_a: state.from.z,
                from_b: state.from.x,
                from_c: state.from.y,
            },
        )
    {
        state.direction = Some(Direction::South);
    }
}

fn lerp_half(start: f64, end: f64) -> f64 {
    start + 0.5 * (end - start)
}

struct ClipPoint {
    da: f64,
    db: f64,
    dc: f64,
    point: f64,
    min_b: f64,
    max_b: f64,
    min_c: f64,
    max_c: f64,
    from_a: f64,
    from_b: f64,
    from_c: f64,
}

fn clip_point(scale: &mut f64, candidate: ClipPoint) -> bool {
    let candidate_scale = (candidate.point - candidate.from_a) / candidate.da;
    let point_b = candidate.from_b + candidate_scale * candidate.db;
    let point_c = candidate.from_c + candidate_scale * candidate.dc;
    if 0.0 < candidate_scale
        && candidate_scale < *scale
        && candidate.min_b - 1.0e-7 < point_b
        && point_b < candidate.max_b + 1.0e-7
        && candidate.min_c - 1.0e-7 < point_c
        && point_c < candidate.max_c + 1.0e-7
    {
        *scale = candidate_scale;
        true
    } else {
        false
    }
}

fn expand_axis(min: f64, max: f64, delta: f64) -> (f64, f64) {
    if delta < 0.0 {
        (min + delta, max)
    } else if delta > 0.0 {
        (min, max + delta)
    } else {
        (min, max)
    }
}

fn contract_axis(min: f64, max: f64, delta: f64) -> (f64, f64) {
    if delta < 0.0 {
        (min - delta, max)
    } else if delta > 0.0 {
        (min, max - delta)
    } else {
        (min, max)
    }
}

fn java_min(left: f64, right: f64) -> f64 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == right {
        if left == 0.0 && left.to_bits() & (1 << 63) == 0 {
            right
        } else {
            left
        }
    } else if left < right {
        left
    } else {
        right
    }
}

fn java_min_f32(left: f32, right: f32) -> f32 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == right {
        if left == 0.0 && left.to_bits() & (1 << 31) == 0 {
            right
        } else {
            left
        }
    } else if left < right {
        left
    } else {
        right
    }
}

fn java_max_f32(left: f32, right: f32) -> f32 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == right {
        if left == 0.0 && left.to_bits() & (1 << 31) != 0 {
            right
        } else {
            left
        }
    } else if left > right {
        left
    } else {
        right
    }
}

fn java_max(left: f64, right: f64) -> f64 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == right {
        if left == 0.0 && left.to_bits() & (1 << 63) != 0 {
            right
        } else {
            left
        }
    } else if left > right {
        left
    } else {
        right
    }
}

mod aabb_clip;
mod aabb_vec3;
