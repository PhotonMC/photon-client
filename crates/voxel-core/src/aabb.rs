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

impl Aabb {
    /// Construct a box, ordering each pair with Java `Math.min`/`Math.max`
    /// behavior for NaN and signed zero.
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            min_x: java_min(min_x, max_x),
            min_y: java_min(min_y, max_y),
            min_z: java_min(min_z, max_z),
            max_x: java_max(min_x, max_x),
            max_y: java_max(min_y, max_y),
            max_z: java_max(min_z, max_z),
        }
    }

    /// Construct the unit block represented by a BlockPos.
    pub fn from_block_pos(pos: BlockPos) -> Self {
        Self::new(
            f64::from(pos.x),
            f64::from(pos.y),
            f64::from(pos.z),
            f64::from(pos.x.wrapping_add(1)),
            f64::from(pos.y.wrapping_add(1)),
            f64::from(pos.z.wrapping_add(1)),
        )
    }

    /// Construct a box from two double-precision vector endpoints.
    pub fn from_vec3(begin: Vec3, end: Vec3) -> Self {
        Self::new(begin.x, begin.y, begin.z, end.x, end.y, end.z)
    }

    /// Construct a unit cube from a double-precision lower corner.
    pub fn unit_cube_from_lower_corner(pos: Vec3) -> Self {
        Self::new(pos.x, pos.y, pos.z, pos.x + 1.0, pos.y + 1.0, pos.z + 1.0)
    }

    /// Enclose the full integer blocks between two BlockPos endpoints.
    pub fn encapsulating_full_blocks(first: BlockPos, second: BlockPos) -> Self {
        Self::new(
            f64::from(first.x.min(second.x)),
            f64::from(first.y.min(second.y)),
            f64::from(first.z.min(second.z)),
            f64::from(first.x.max(second.x).wrapping_add(1)),
            f64::from(first.y.max(second.y).wrapping_add(1)),
            f64::from(first.z.max(second.z).wrapping_add(1)),
        )
    }

    /// Convert normalized integer bounds to an AABB with Java int upper-endpoint arithmetic.
    pub fn of_bounding_box(value: BoundingBox) -> Self {
        Self::new(
            f64::from(value.min_x),
            f64::from(value.min_y),
            f64::from(value.min_z),
            f64::from(value.max_x.wrapping_add(1)),
            f64::from(value.max_y.wrapping_add(1)),
            f64::from(value.max_z.wrapping_add(1)),
        )
    }

    /// Construct a box around a center using Java's per-axis half sizes.
    pub fn of_size(center: Vec3, size_x: f64, size_y: f64, size_z: f64) -> Self {
        Self::new(
            center.x - size_x / 2.0,
            center.y - size_y / 2.0,
            center.z - size_z / 2.0,
            center.x + size_x / 2.0,
            center.y + size_y / 2.0,
            center.z + size_z / 2.0,
        )
    }

    /// Replace the lower x endpoint and reapply constructor normalization.
    pub fn set_min_x(self, min_x: f64) -> Self {
        Self::new(
            min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z,
        )
    }

    /// Replace the lower y endpoint and reapply constructor normalization.
    pub fn set_min_y(self, min_y: f64) -> Self {
        Self::new(
            self.min_x, min_y, self.min_z, self.max_x, self.max_y, self.max_z,
        )
    }

    /// Replace the lower z endpoint and reapply constructor normalization.
    pub fn set_min_z(self, min_z: f64) -> Self {
        Self::new(
            self.min_x, self.min_y, min_z, self.max_x, self.max_y, self.max_z,
        )
    }

    /// Replace the upper x endpoint and reapply constructor normalization.
    pub fn set_max_x(self, max_x: f64) -> Self {
        Self::new(
            self.min_x, self.min_y, self.min_z, max_x, self.max_y, self.max_z,
        )
    }

    /// Replace the upper y endpoint and reapply constructor normalization.
    pub fn set_max_y(self, max_y: f64) -> Self {
        Self::new(
            self.min_x, self.min_y, self.min_z, self.max_x, max_y, self.max_z,
        )
    }

    /// Replace the upper z endpoint and reapply constructor normalization.
    pub fn set_max_z(self, max_z: f64) -> Self {
        Self::new(
            self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, max_z,
        )
    }

    /// Test an inclusive lower/exclusive upper point membership rule.
    pub fn contains(self, x: f64, y: f64, z: f64) -> bool {
        x >= self.min_x
            && x < self.max_x
            && y >= self.min_y
            && y < self.max_y
            && z >= self.min_z
            && z < self.max_z
    }

    /// Test point membership from a double-precision vector.
    pub fn contains_vec3(self, point: Vec3) -> bool {
        self.contains(point.x, point.y, point.z)
    }

    /// Test strict overlap on all three axes; touching faces do not overlap.
    pub fn intersects(self, other: Self) -> bool {
        self.intersects_coords(
            other.min_x,
            other.min_y,
            other.min_z,
            other.max_x,
            other.max_y,
            other.max_z,
        )
    }

    /// Test strict overlap against raw endpoints without normalizing them.
    pub fn intersects_coords(
        self,
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> bool {
        self.min_x < max_x
            && self.max_x > min_x
            && self.min_y < max_y
            && self.max_y > min_y
            && self.min_z < max_z
            && self.max_z > min_z
    }

    /// Test strict overlap after normalizing the two vector endpoints.
    pub fn intersects_vec3(self, min: Vec3, max: Vec3) -> bool {
        self.intersects_coords(
            java_min(min.x, max.x),
            java_min(min.y, max.y),
            java_min(min.z, max.z),
            java_max(min.x, max.x),
            java_max(min.y, max.y),
            java_max(min.z, max.z),
        )
    }

    /// Test strict overlap with the unit block whose origin is a BlockPos.
    pub fn intersects_block_pos(self, pos: BlockPos) -> bool {
        self.intersects_coords(
            f64::from(pos.x),
            f64::from(pos.y),
            f64::from(pos.z),
            f64::from(pos.x.wrapping_add(1)),
            f64::from(pos.y.wrapping_add(1)),
            f64::from(pos.z.wrapping_add(1)),
        )
    }

    /// Extend only the endpoint in the direction of each nonzero delta.
    pub fn expand_towards(self, x: f64, y: f64, z: f64) -> Self {
        let (min_x, max_x) = expand_axis(self.min_x, self.max_x, x);
        let (min_y, max_y) = expand_axis(self.min_y, self.max_y, y);
        let (min_z, max_z) = expand_axis(self.min_z, self.max_z, z);
        Self::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    /// Expand toward a double-precision vector like Java `AABB.expandTowards(Vec3)`.
    pub fn expand_towards_vec3(self, delta: Vec3) -> Self {
        self.expand_towards(delta.x, delta.y, delta.z)
    }

    /// Compare endpoints with Java `AABB.equals`, including signed zero and NaN rules.
    pub fn equals_java(self, other: Self) -> bool {
        java_double_compare(self.min_x, other.min_x) == 0
            && java_double_compare(self.min_y, other.min_y) == 0
            && java_double_compare(self.min_z, other.min_z) == 0
            && java_double_compare(self.max_x, other.max_x) == 0
            && java_double_compare(self.max_y, other.max_y) == 0
            && java_double_compare(self.max_z, other.max_z) == 0
    }

    /// Compute the Java `AABB.hashCode` value for the six endpoints.
    pub fn hash_code_java(self) -> i32 {
        let mut result = java_double_hash(self.min_x);
        result = result
            .wrapping_mul(31)
            .wrapping_add(java_double_hash(self.min_y));
        result = result
            .wrapping_mul(31)
            .wrapping_add(java_double_hash(self.min_z));
        result = result
            .wrapping_mul(31)
            .wrapping_add(java_double_hash(self.max_x));
        result = result
            .wrapping_mul(31)
            .wrapping_add(java_double_hash(self.max_y));
        result
            .wrapping_mul(31)
            .wrapping_add(java_double_hash(self.max_z))
    }

    /// Format the box like Java `AABB.toString()`.
    pub fn to_string_java(self) -> String {
        format!(
            "AABB[{}, {}, {}] -> [{}, {}, {}]",
            java_double_to_string(self.min_x),
            java_double_to_string(self.min_y),
            java_double_to_string(self.min_z),
            java_double_to_string(self.max_x),
            java_double_to_string(self.max_y),
            java_double_to_string(self.max_z),
        )
    }

    /// Move only the endpoint opposite each nonzero delta inward.
    pub fn contract(self, x: f64, y: f64, z: f64) -> Self {
        let (min_x, max_x) = contract_axis(self.min_x, self.max_x, x);
        let (min_y, max_y) = contract_axis(self.min_y, self.max_y, y);
        let (min_z, max_z) = contract_axis(self.min_z, self.max_z, z);
        Self::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    /// Intersect this box with another box, preserving Java endpoint rules.
    pub fn intersect(self, other: Self) -> Self {
        Self::new(
            java_max(self.min_x, other.min_x),
            java_max(self.min_y, other.min_y),
            java_max(self.min_z, other.min_z),
            java_min(self.max_x, other.max_x),
            java_min(self.max_y, other.max_y),
            java_min(self.max_z, other.max_z),
        )
    }

    /// Enclose this box and another box, preserving Java endpoint rules.
    pub fn minmax(self, other: Self) -> Self {
        Self::new(
            java_min(self.min_x, other.min_x),
            java_min(self.min_y, other.min_y),
            java_min(self.min_z, other.min_z),
            java_max(self.max_x, other.max_x),
            java_max(self.max_y, other.max_y),
            java_max(self.max_z, other.max_z),
        )
    }

    /// Translate every endpoint by the supplied per-axis amounts.
    pub fn move_by(self, x: f64, y: f64, z: f64) -> Self {
        Self::new(
            self.min_x + x,
            self.min_y + y,
            self.min_z + z,
            self.max_x + x,
            self.max_y + y,
            self.max_z + z,
        )
    }

    /// Translate by integer block coordinates, matching `AABB.move(BlockPos)`.
    pub fn move_by_block_pos(self, pos: BlockPos) -> Self {
        self.move_by(f64::from(pos.x), f64::from(pos.y), f64::from(pos.z))
    }

    /// Translate by a double-precision vector, matching `AABB.move(Vec3)`.
    pub fn move_by_vec3(self, pos: Vec3) -> Self {
        self.move_by(pos.x, pos.y, pos.z)
    }

    /// Translate by the float components of a `Vector3f` before widening them.
    pub fn move_by_vector3f(self, x: f32, y: f32, z: f32) -> Self {
        self.move_by(f64::from(x), f64::from(y), f64::from(z))
    }

    /// Return the edge lengths along the x, y and z axes.
    pub fn x_size(self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn y_size(self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn z_size(self) -> f64 {
        self.max_z - self.min_z
    }

    /// Return the arithmetic mean of the three edge lengths.
    pub fn size(self) -> f64 {
        (self.x_size() + self.y_size() + self.z_size()) / 3.0
    }

    /// Test whether any stored endpoint is NaN.
    pub fn has_nan(self) -> bool {
        self.min_x.is_nan()
            || self.min_y.is_nan()
            || self.min_z.is_nan()
            || self.max_x.is_nan()
            || self.max_y.is_nan()
            || self.max_z.is_nan()
    }

    /// Return the midpoint of all three box axes using Java's lerp order.
    pub fn center(self) -> Vec3 {
        Vec3::new(
            lerp_half(self.min_x, self.max_x),
            lerp_half(self.min_y, self.max_y),
            lerp_half(self.min_z, self.max_z),
        )
    }

    /// Return the midpoint of the bottom face.
    pub fn bottom_center(self) -> Vec3 {
        Vec3::new(
            lerp_half(self.min_x, self.max_x),
            self.min_y,
            lerp_half(self.min_z, self.max_z),
        )
    }

    /// Return the lower endpoint as a vector.
    pub fn min_position(self) -> Vec3 {
        Vec3::new(self.min_x, self.min_y, self.min_z)
    }

    /// Return the upper endpoint as a vector.
    pub fn max_position(self) -> Vec3 {
        Vec3::new(self.max_x, self.max_y, self.max_z)
    }

    /// Return the lower endpoint selected by an axis.
    pub fn min(self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.min_x,
            Axis::Y => self.min_y,
            Axis::Z => self.min_z,
        }
    }

    /// Return the upper endpoint selected by an axis.
    pub fn max(self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.max_x,
            Axis::Y => self.max_y,
            Axis::Z => self.max_z,
        }
    }

    /// Return the squared distance from a point to this box.
    pub fn distance_to_sqr_point(self, point: Vec3) -> f64 {
        let dx = java_max(java_max(self.min_x - point.x, point.x - self.max_x), 0.0);
        let dy = java_max(java_max(self.min_y - point.y, point.y - self.max_y), 0.0);
        let dz = java_max(java_max(self.min_z - point.z, point.z - self.max_z), 0.0);
        Vec3::new(dx, dy, dz).length_squared()
    }

    /// Return the squared distance between this box and another box.
    pub fn distance_to_sqr_box(self, other: Self) -> f64 {
        let dx = java_max(
            java_max(self.min_x - other.max_x, other.min_x - self.max_x),
            0.0,
        );
        let dy = java_max(
            java_max(self.min_y - other.max_y, other.min_y - self.max_y),
            0.0,
        );
        let dz = java_max(
            java_max(self.min_z - other.max_z, other.min_z - self.max_z),
            0.0,
        );
        Vec3::new(dx, dy, dz).length_squared()
    }

    /// Extend both endpoints by the supplied per-axis amounts.
    pub fn inflate(self, x: f64, y: f64, z: f64) -> Self {
        Self::new(
            self.min_x - x,
            self.min_y - y,
            self.min_z - z,
            self.max_x + x,
            self.max_y + y,
            self.max_z + z,
        )
    }

    /// Extend both endpoints by the same amount.
    pub fn inflate_all(self, amount: f64) -> Self {
        self.inflate(amount, amount, amount)
    }

    /// Move every lower endpoint inward and every upper endpoint inward.
    pub fn deflate_xyz(self, x: f64, y: f64, z: f64) -> Self {
        self.inflate(java_negate(x), java_negate(y), java_negate(z))
    }

    /// Move every lower endpoint inward and every upper endpoint inward by one amount.
    pub fn deflate(self, amount: f64) -> Self {
        let amount = java_negate(amount);
        self.inflate(amount, amount, amount)
    }

    /// Return the first segment point inside the box, matching Java `clip`.
    pub fn clip(self, from: Vec3, to: Vec3) -> Option<Vec3> {
        Self::clip_coords(
            Vec3::new(self.min_x, self.min_y, self.min_z),
            Vec3::new(self.max_x, self.max_y, self.max_z),
            from,
            to,
        )
    }

    /// Clip a segment against raw coordinates, without normalizing the bounds.
    pub fn clip_coords(min: Vec3, max: Vec3, from: Vec3, to: Vec3) -> Option<Vec3> {
        let mut state = ClipState::new(from, to);
        clip_bounds(min, max, &mut state);

        state.direction.map(|_| {
            Vec3::new(
                state.from.x + state.scale * state.dx,
                state.from.y + state.scale * state.dy,
                state.from.z + state.scale * state.dz,
            )
        })
    }

    /// Clip a segment against translated AABBs and return the nearest block hit.
    pub fn clip_many(
        boxes: &[Self],
        from: Vec3,
        to: Vec3,
        pos: BlockPos,
    ) -> Option<BlockHitResult> {
        let mut state = ClipState::new(from, to);

        for aabb in boxes {
            let translated = aabb.move_by_block_pos(pos);
            clip_bounds(
                translated.min_position(),
                translated.max_position(),
                &mut state,
            );
        }

        state.direction.map(|direction| {
            BlockHitResult::new(
                Vec3::new(
                    state.from.x + state.scale * state.dx,
                    state.from.y + state.scale * state.dy,
                    state.from.z + state.scale * state.dz,
                ),
                direction,
                pos,
                false,
            )
        })
    }

    /// Test whether the center's translated segment collides with any box.
    pub fn collided_along_vector(self, vector: Vec3, boxes: &[Self]) -> bool {
        let from = self.center();
        let to = Vec3::new(from.x + vector.x, from.y + vector.y, from.z + vector.z);
        let inflation = Vec3::new(
            self.x_size() * 0.5 - 1.0e-7,
            self.y_size() * 0.5 - 1.0e-7,
            self.z_size() * 0.5 - 1.0e-7,
        );

        boxes.iter().any(|shape_part| {
            let inflated = shape_part.inflate(inflation.x, inflation.y, inflation.z);
            inflated.contains_vec3(to)
                || inflated.contains_vec3(from)
                || inflated.clip(from, to).is_some()
        })
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
