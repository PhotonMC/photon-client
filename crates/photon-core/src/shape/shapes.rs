use super::*;

impl Shapes {
    /// Return whether two block-local shapes occlude across a direction.
    pub fn block_occludes(shape: &VoxelShape, occluder: &VoxelShape, direction: Direction) -> bool {
        let axis = direction.axis();
        let (first, second, operation) =
            if matches!(direction.axis_direction(), AxisDirection::Positive) {
                (shape, occluder, BooleanOp::OnlyFirst)
            } else {
                (occluder, shape, BooleanOp::OnlySecond)
            };

        if !fuzzy_equals(first.max_coordinate(axis), 1.0)
            || !fuzzy_equals(second.min_coordinate(axis), 0.0)
        {
            return false;
        }

        let first_slice = first.slice_at(axis, first.axis_size(axis) as isize - 1);
        let second_slice = second.slice_at(axis, 0);
        !Self::join_is_not_empty(&first_slice, &second_slice, operation)
    }

    /// Return whether the merged boundary faces cover a unit block.
    pub fn merged_face_occludes(
        shape: &VoxelShape,
        occluder: &VoxelShape,
        direction: Direction,
    ) -> bool {
        if shape.is_block() || occluder.is_block() {
            return true;
        }

        let axis = direction.axis();
        let (first, second) = if matches!(direction.axis_direction(), AxisDirection::Positive) {
            (shape, occluder)
        } else {
            (occluder, shape)
        };
        let first = if fuzzy_equals(first.max_coordinate(axis), 1.0) {
            first.clone()
        } else {
            Self::empty()
        };
        let second = if fuzzy_equals(second.min_coordinate(axis), 0.0) {
            second.clone()
        } else {
            Self::empty()
        };
        let first_slice = first.slice_at(axis, first.axis_size(axis) as isize - 1);
        let second_slice = second.slice_at(axis, 0);
        let merged = Self::or(&first_slice, &second_slice);
        !Self::join_is_not_empty(&Self::block(), &merged, BooleanOp::OnlyFirst)
    }

    /// Return whether two face shapes cover a unit block together.
    pub fn face_shape_occludes(shape: &VoxelShape, occluder: &VoxelShape) -> bool {
        if shape.is_block() || occluder.is_block() {
            return true;
        }
        if shape.is_empty() && occluder.is_empty() {
            return false;
        }

        let merged = Self::or(shape, occluder);
        !Self::join_is_not_empty(&Self::block(), &merged, BooleanOp::OnlyFirst)
    }

    /// Limit a moving box against each shape in iteration order.
    pub fn collide(axis: Axis, moving: Aabb, shapes: &[VoxelShape], mut distance: f64) -> f64 {
        for shape in shapes {
            if distance.abs() < EPSILON {
                return 0.0;
            }
            distance = shape.collide(axis, moving, distance);
        }
        distance
    }

    /// Minimum shape extent used by Java's shape constructors.
    pub const EPSILON: f64 = 1.0e-7;

    /// Larger fuzzy comparison extent exposed by Java's shape helpers.
    pub const BIG_EPSILON: f64 = 1.0e-6;

    /// Return an empty shape.
    pub fn empty() -> VoxelShape {
        EMPTY_SHAPE
            .get_or_init(|| VoxelShape::new(vec![0.0], vec![0.0], vec![0.0], Vec::new()))
            .clone()
    }

    /// Return the unit block shape.
    pub fn block() -> VoxelShape {
        BLOCK_SHAPE
            .get_or_init(|| cube_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0, 0, 0]))
            .clone()
    }

    /// Return the shape spanning negative to positive infinity on every axis.
    pub fn infinity() -> VoxelShape {
        INFINITY_SHAPE
            .get_or_init(|| {
                Self::box_shape(
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                    f64::INFINITY,
                    f64::INFINITY,
                    f64::INFINITY,
                )
                .expect("infinity bounds are ordered")
            })
            .clone()
    }

    /// Construct a shape from raw coordinates like Java `Shapes.create(double...)`.
    pub fn create_from_coords(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> VoxelShape {
        if max_x - min_x < EPSILON || max_y - min_y < EPSILON || max_z - min_z < EPSILON {
            return Self::empty();
        }

        match (
            find_bits(min_x, max_x),
            find_bits(min_y, max_y),
            find_bits(min_z, max_z),
        ) {
            (Some(0), Some(0), Some(0)) => Self::block(),
            (Some(x_bits), Some(y_bits), Some(z_bits)) => cube_box(
                [min_x, min_y, min_z],
                [max_x, max_y, max_z],
                [x_bits, y_bits, z_bits],
            ),
            _ => VoxelShape::new(
                vec![min_x, max_x],
                vec![min_y, max_y],
                vec![min_z, max_z],
                vec![true],
            ),
        }
    }

    /// Construct a box, applying Java's minimum extent threshold.
    pub fn box_shape(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> Result<VoxelShape, ShapeError> {
        if min_x > max_x || min_y > max_y || min_z > max_z {
            return Err(ShapeError::ReversedBounds);
        }
        Ok(Self::create_from_coords(
            min_x, min_y, min_z, max_x, max_y, max_z,
        ))
    }

    /// Construct a shape from normalized AABB endpoints like Java `Shapes.create(AABB)`.
    pub fn create_from_aabb(aabb: Aabb) -> Result<VoxelShape, ShapeError> {
        Self::box_shape(
            aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
        )
    }

    /// Join two shapes by filling every cell occupied by either input.
    pub fn or(first: &VoxelShape, second: &VoxelShape) -> VoxelShape {
        Self::join(first, second, BooleanOp::Or)
    }

    /// Join a first shape with Java's left-to-right varargs OR tail.
    pub fn or_many(first: &VoxelShape, tail: &[VoxelShape]) -> VoxelShape {
        tail.iter()
            .fold(first.clone(), |joined, shape| Self::or(&joined, shape))
    }

    /// Join two shapes and normalize the result through Java's optimize path.
    pub fn join(first: &VoxelShape, second: &VoxelShape, operation: BooleanOp) -> VoxelShape {
        Self::join_unoptimized(first, second, operation).optimize()
    }

    /// Join two shapes while preserving the merged coordinate grid.
    pub fn join_unoptimized(
        first: &VoxelShape,
        second: &VoxelShape,
        operation: BooleanOp,
    ) -> VoxelShape {
        if first.equals_java(second) {
            return if operation.apply(true, true) {
                first.clone()
            } else {
                Self::empty()
            };
        }

        let first_only_matters = operation.apply(true, false);
        let second_only_matters = operation.apply(false, true);
        if first.is_empty() {
            return if second_only_matters {
                second.clone()
            } else {
                Self::empty()
            };
        }
        if second.is_empty() {
            return if first_only_matters {
                first.clone()
            } else {
                Self::empty()
            };
        }

        let xs = merged_coords(&first.xs, &second.xs);
        let ys = merged_coords(&first.ys, &second.ys);
        let zs = merged_coords(&first.zs, &second.zs);
        let grid = Grid::from_coords(&xs, &ys, &zs);
        let mut cells = vec![false; grid.cell_count()];

        for x in 0..grid.x_size {
            for y in 0..grid.y_size {
                for z in 0..grid.z_size {
                    let center_x = midpoint(xs[x], xs[x + 1]);
                    let center_y = midpoint(ys[y], ys[y + 1]);
                    let center_z = midpoint(zs[z], zs[z + 1]);
                    cells[index(x, y, z, grid)] = operation.apply(
                        first.full_at(center_x, center_y, center_z),
                        second.full_at(center_x, center_y, center_z),
                    );
                }
            }
        }

        VoxelShape::new(xs, ys, zs, cells)
    }

    /// Return whether a BooleanOp join contains any occupied cell, like Java
    /// `Shapes.joinIsNotEmpty`: without building the joined shape, and
    /// stopping at the first occupied cell.
    pub fn join_is_not_empty(
        first: &VoxelShape,
        second: &VoxelShape,
        operation: BooleanOp,
    ) -> bool {
        let (first_empty, second_empty) = (first.is_empty(), second.is_empty());
        if first_empty || second_empty {
            return operation.apply(!first_empty, !second_empty);
        }
        if first.equals_java(second) {
            return operation.apply(true, true);
        }
        let first_only_matters = operation.apply(true, false);
        let second_only_matters = operation.apply(false, true);
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            if first.max(axis) < second.min(axis) - 1.0e-7
                || second.max(axis) < first.min(axis) - 1.0e-7
            {
                return first_only_matters || second_only_matters;
            }
        }
        // Each merged interval's cell in either shape, looked up once per axis.
        let cells = |merged: &[f64], own: &[f64]| -> Vec<Option<usize>> {
            merged
                .windows(2)
                .map(|pair| interval(own, midpoint(pair[0], pair[1])))
                .collect()
        };
        let xs = merged_coords(&first.xs, &second.xs);
        let ys = merged_coords(&first.ys, &second.ys);
        let zs = merged_coords(&first.zs, &second.zs);
        let [first_x, first_y, first_z] = [
            cells(&xs, &first.xs),
            cells(&ys, &first.ys),
            cells(&zs, &first.zs),
        ];
        let [second_x, second_y, second_z] = [
            cells(&xs, &second.xs),
            cells(&ys, &second.ys),
            cells(&zs, &second.zs),
        ];
        let full = |shape: &VoxelShape, x: Option<usize>, y: Option<usize>, z: Option<usize>| matches!((x, y, z), (Some(x), Some(y), Some(z)) if shape.is_full(x, y, z));
        for x in 0..xs.len() - 1 {
            for y in 0..ys.len() - 1 {
                for z in 0..zs.len() - 1 {
                    if operation.apply(
                        full(first, first_x[x], first_y[y], first_z[z]),
                        full(second, second_x[x], second_y[y], second_z[z]),
                    ) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Return whether two shapes occupy the same cells.
    pub fn equal(first: &VoxelShape, second: &VoxelShape) -> bool {
        !Self::join_is_not_empty(first, second, BooleanOp::NotSame)
    }
}

pub(super) const EPSILON: f64 = 1.0e-7;

pub(super) static EMPTY_SHAPE: OnceLock<VoxelShape> = OnceLock::new();

pub(super) static BLOCK_SHAPE: OnceLock<VoxelShape> = OnceLock::new();

pub(super) static INFINITY_SHAPE: OnceLock<VoxelShape> = OnceLock::new();

/// Constructors and operations for voxel shapes.
pub struct Shapes;
