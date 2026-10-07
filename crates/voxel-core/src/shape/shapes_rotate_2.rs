use super::*;

impl Shapes {
    /// Return attach-face variants for a supported initial rotation.
    pub fn rotate_attach_face_with_initial(
        north: &VoxelShape,
        initial: ShapeRotation,
    ) -> [(AttachFace, [(Direction, VoxelShape); 4]); 3] {
        Self::rotate_attach_face_with_transform(north, initial.transform())
    }

    /// Return the horizontal direction variants of a shape in Java EnumMap order.
    pub fn rotate_horizontal(north: &VoxelShape) -> [(Direction, VoxelShape); 4] {
        [
            (Direction::North, north.clone()),
            (
                Direction::South,
                Self::rotate(north, ShapeRotation::Rot180FaceXz),
            ),
            (
                Direction::West,
                Self::rotate(north, ShapeRotation::Rot90YPos),
            ),
            (
                Direction::East,
                Self::rotate(north, ShapeRotation::Rot90YNeg),
            ),
        ]
    }

    /// Return horizontal direction variants for a supported initial rotation.
    pub fn rotate_horizontal_with_initial(
        north: &VoxelShape,
        initial: ShapeRotation,
    ) -> [(Direction, VoxelShape); 4] {
        Self::rotate_horizontal_with_transform(north, initial.transform())
    }

    /// Return all direction variants of a shape in Java EnumMap order.
    pub fn rotate_all(north: &VoxelShape) -> [(Direction, VoxelShape); 6] {
        Self::rotate_all_with_center(north, Vec3::new(0.5, 0.5, 0.5))
    }

    pub(super) fn rotate_all_with_transform(
        north: &VoxelShape,
        initial: ShapeTransform,
        rotation_center: Vec3,
    ) -> [(Direction, VoxelShape); 6] {
        let x_90 = ShapeRotation::Rot90XNeg.transform();
        let x_270 = ShapeRotation::Rot90XPos.transform();
        let y_90 = ShapeRotation::Rot90YNeg.transform();
        let y_180 = ShapeRotation::Rot180FaceXz.transform();
        let y_270 = ShapeRotation::Rot90YPos.transform();
        [
            (
                Direction::Down,
                Self::rotate_transform_with_center(north, x_90.compose(initial), rotation_center),
            ),
            (
                Direction::Up,
                Self::rotate_transform_with_center(north, x_270.compose(initial), rotation_center),
            ),
            (
                Direction::North,
                Self::rotate_transform_with_center(north, initial, Vec3::new(0.5, 0.5, 0.5)),
            ),
            (
                Direction::South,
                Self::rotate_transform_with_center(north, y_180.compose(initial), rotation_center),
            ),
            (
                Direction::West,
                Self::rotate_transform_with_center(north, y_270.compose(initial), rotation_center),
            ),
            (
                Direction::East,
                Self::rotate_transform_with_center(north, y_90.compose(initial), rotation_center),
            ),
        ]
    }

    /// Return all direction variants around an explicit point.
    pub fn rotate_all_with_initial(
        north: &VoxelShape,
        initial: ShapeRotation,
        rotation_center: Vec3,
    ) -> [(Direction, VoxelShape); 6] {
        Self::rotate_all_with_transform(north, initial.transform(), rotation_center)
    }

    /// Return all direction variants around an explicit point.
    pub fn rotate_all_with_center(
        north: &VoxelShape,
        rotation_center: Vec3,
    ) -> [(Direction, VoxelShape); 6] {
        Self::rotate_all_with_transform(north, ShapeRotation::Identity.transform(), rotation_center)
    }

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
}
