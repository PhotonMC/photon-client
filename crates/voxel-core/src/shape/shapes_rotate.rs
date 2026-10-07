use super::*;

impl Shapes {
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

    /// Rotate a shape around the unit block center.
    pub fn rotate(shape: &VoxelShape, rotation: ShapeRotation) -> VoxelShape {
        Self::rotate_with_center(shape, rotation, Vec3::new(0.5, 0.5, 0.5))
    }

    /// Rotate a shape around an explicit point.
    pub fn rotate_with_center(
        shape: &VoxelShape,
        rotation: ShapeRotation,
        rotation_center: Vec3,
    ) -> VoxelShape {
        Self::rotate_transform_with_center(shape, rotation.transform(), rotation_center)
    }

    pub(super) fn rotate_transform_with_center(
        shape: &VoxelShape,
        rotation: ShapeTransform,
        rotation_center: Vec3,
    ) -> VoxelShape {
        if rotation.is_identity() {
            return shape.clone();
        }

        let old_sizes = [shape.xs.len() - 1, shape.ys.len() - 1, shape.zs.len() - 1];
        let old_coords = [&shape.xs, &shape.ys, &shape.zs];
        let permutation = rotation.permutation;
        let inverted = rotation.inverted;
        let new_sizes = [
            old_sizes[permutation[0]],
            old_sizes[permutation[1]],
            old_sizes[permutation[2]],
        ];
        let shifts = [
            if inverted[0] && new_sizes[0] > 0 {
                new_sizes[0] as isize - 1
            } else {
                0
            },
            if inverted[1] && new_sizes[1] > 0 {
                new_sizes[1] as isize - 1
            } else {
                0
            },
            if inverted[2] && new_sizes[2] > 0 {
                new_sizes[2] as isize - 1
            } else {
                0
            },
        ];
        let grid = Grid {
            x_size: new_sizes[0],
            y_size: new_sizes[1],
            z_size: new_sizes[2],
        };
        let mut cells = vec![false; grid.cell_count()];

        for x in 0..old_sizes[0] {
            for y in 0..old_sizes[1] {
                for z in 0..old_sizes[2] {
                    if !shape.is_full(x, y, z) {
                        continue;
                    }
                    let old_position = [x as isize, y as isize, z as isize];
                    let new_position = [
                        shifts[0]
                            + if inverted[0] {
                                -old_position[permutation[0]]
                            } else {
                                old_position[permutation[0]]
                            },
                        shifts[1]
                            + if inverted[1] {
                                -old_position[permutation[1]]
                            } else {
                                old_position[permutation[1]]
                            },
                        shifts[2]
                            + if inverted[2] {
                                -old_position[permutation[2]]
                            } else {
                                old_position[permutation[2]]
                            },
                    ];
                    cells[index(
                        new_position[0] as usize,
                        new_position[1] as usize,
                        new_position[2] as usize,
                        grid,
                    )] = true;
                }
            }
        }

        let new_coords = [
            rotate_axis_coords(
                old_coords[permutation[0]],
                inverted[0],
                axis_component(rotation_center, permutation[0]),
                axis_component(rotation_center, 0),
            ),
            rotate_axis_coords(
                old_coords[permutation[1]],
                inverted[1],
                axis_component(rotation_center, permutation[1]),
                axis_component(rotation_center, 1),
            ),
            rotate_axis_coords(
                old_coords[permutation[2]],
                inverted[2],
                axis_component(rotation_center, permutation[2]),
                axis_component(rotation_center, 2),
            ),
        ];
        VoxelShape::new(
            new_coords[0].clone(),
            new_coords[1].clone(),
            new_coords[2].clone(),
            cells,
        )
    }

    /// Return the horizontal-axis variants of a shape in Java EnumMap order.
    pub fn rotate_horizontal_axis(z_axis: &VoxelShape) -> [(Axis, VoxelShape); 2] {
        Self::rotate_horizontal_axis_with_center(z_axis, Vec3::new(0.5, 0.5, 0.5))
    }

    /// Return horizontal-axis variants around an explicit point.
    pub fn rotate_horizontal_axis_with_center(
        z_axis: &VoxelShape,
        rotation_center: Vec3,
    ) -> [(Axis, VoxelShape); 2] {
        [
            (
                Axis::X,
                Self::rotate_with_center(z_axis, ShapeRotation::Rot90YNeg, rotation_center),
            ),
            (Axis::Z, z_axis.clone()),
        ]
    }

    /// Return the all-axis variants of a shape in Java EnumMap order.
    pub fn rotate_all_axis(north: &VoxelShape) -> [(Axis, VoxelShape); 3] {
        Self::rotate_all_axis_with_center(north, Vec3::new(0.5, 0.5, 0.5))
    }

    /// Return all-axis variants around an explicit point.
    pub fn rotate_all_axis_with_center(
        north: &VoxelShape,
        rotation_center: Vec3,
    ) -> [(Axis, VoxelShape); 3] {
        [
            (
                Axis::X,
                Self::rotate_with_center(north, ShapeRotation::Rot90YNeg, rotation_center),
            ),
            (
                Axis::Y,
                Self::rotate_with_center(north, ShapeRotation::Rot90XNeg, rotation_center),
            ),
            (Axis::Z, north.clone()),
        ]
    }

    pub(super) fn rotate_horizontal_with_transform(
        north: &VoxelShape,
        initial: ShapeTransform,
    ) -> [(Direction, VoxelShape); 4] {
        let y_90 = ShapeRotation::Rot90YNeg.transform();
        let y_180 = ShapeRotation::Rot180FaceXz.transform();
        let y_270 = ShapeRotation::Rot90YPos.transform();
        let center = Vec3::new(0.5, 0.5, 0.5);
        [
            (
                Direction::North,
                Self::rotate_transform_with_center(north, initial, center),
            ),
            (
                Direction::South,
                Self::rotate_transform_with_center(north, y_180.compose(initial), center),
            ),
            (
                Direction::West,
                Self::rotate_transform_with_center(north, y_270.compose(initial), center),
            ),
            (
                Direction::East,
                Self::rotate_transform_with_center(north, y_90.compose(initial), center),
            ),
        ]
    }

    /// Return the floor, wall, and ceiling direction variants in Java enum order.
    pub(super) fn rotate_attach_face_with_transform(
        north: &VoxelShape,
        initial: ShapeTransform,
    ) -> [(AttachFace, [(Direction, VoxelShape); 4]); 3] {
        let x_270 = ShapeRotation::Rot90XPos.transform();
        let x_90 = ShapeRotation::Rot90XNeg.transform();
        let y_180 = ShapeRotation::Rot180FaceXz.transform();
        [
            (
                AttachFace::Floor,
                Self::rotate_horizontal_with_transform(north, x_270.compose(initial)),
            ),
            (
                AttachFace::Wall,
                Self::rotate_horizontal_with_transform(north, initial),
            ),
            (
                AttachFace::Ceiling,
                Self::rotate_horizontal_with_transform(north, y_180.compose(x_90).compose(initial)),
            ),
        ]
    }

    /// Return the floor, wall, and ceiling direction variants in Java enum order.
    pub fn rotate_attach_face(
        north: &VoxelShape,
    ) -> [(AttachFace, [(Direction, VoxelShape); 4]); 3] {
        Self::rotate_attach_face_with_transform(north, ShapeRotation::Identity.transform())
    }
}
