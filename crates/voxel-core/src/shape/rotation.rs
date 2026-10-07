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

    pub(crate) fn rotate_all_with_transform(
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

    pub(crate) fn rotate_transform_with_center(
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

    pub(crate) fn rotate_horizontal_with_transform(
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
    pub(crate) fn rotate_attach_face_with_transform(
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

/// Signed axis permutations supported by the current shape rotation slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeRotation {
    Identity,
    Rot90XNeg,
    Rot90XPos,
    Rot90YNeg,
    Rot90YPos,
    Rot90ZNeg,
    Rot90ZPos,
    Rot180FaceXy,
    Rot180FaceXz,
    Rot180FaceYz,
    /// `OctahedralGroup.INVERT_Y`: the mirror that turns a shape upside down.
    InvertY,
    /// `OctahedralGroup.INVERT_X`.
    InvertX,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShapeTransform {
    pub(crate) permutation: [usize; 3],
    pub(crate) inverted: [bool; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShapeAxisCycle {
    None,
    Forward,
    Backward,
}

/// The surface to which an attachable block shape is oriented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachFace {
    Floor,
    Wall,
    Ceiling,
}

pub(crate) fn rotate_axis_coords(
    coords: &[f64],
    invert: bool,
    source_center: f64,
    output_center: f64,
) -> Vec<f64> {
    if !invert && source_center == output_center {
        return coords.to_vec();
    }
    if invert {
        coords
            .iter()
            .rev()
            .map(|value| -(*value - source_center) + output_center)
            .collect()
    } else {
        coords
            .iter()
            .map(|value| *value - source_center + output_center)
            .collect()
    }
}

impl ShapeRotation {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "IDENTITY" => Some(Self::Identity),
            "ROT_90_X_NEG" => Some(Self::Rot90XNeg),
            "ROT_90_X_POS" => Some(Self::Rot90XPos),
            "ROT_90_Y_NEG" => Some(Self::Rot90YNeg),
            "ROT_90_Y_POS" => Some(Self::Rot90YPos),
            "ROT_90_Z_NEG" => Some(Self::Rot90ZNeg),
            "ROT_90_Z_POS" => Some(Self::Rot90ZPos),
            "ROT_180_FACE_XY" => Some(Self::Rot180FaceXy),
            "ROT_180_FACE_XZ" => Some(Self::Rot180FaceXz),
            "ROT_180_FACE_YZ" => Some(Self::Rot180FaceYz),
            "INVERT_Y" => Some(Self::InvertY),
            "INVERT_X" => Some(Self::InvertX),
            _ => None,
        }
    }

    pub(crate) fn axes(self) -> ([usize; 3], [bool; 3]) {
        match self {
            Self::Identity => ([0, 1, 2], [false, false, false]),
            Self::Rot90XNeg => ([0, 2, 1], [false, false, true]),
            Self::Rot90XPos => ([0, 2, 1], [false, true, false]),
            Self::Rot90YNeg => ([2, 1, 0], [true, false, false]),
            Self::Rot90YPos => ([2, 1, 0], [false, false, true]),
            Self::Rot90ZNeg => ([1, 0, 2], [false, true, false]),
            Self::Rot90ZPos => ([1, 0, 2], [true, false, false]),
            Self::Rot180FaceXy => ([0, 1, 2], [true, true, false]),
            Self::Rot180FaceXz => ([0, 1, 2], [true, false, true]),
            Self::Rot180FaceYz => ([0, 1, 2], [false, true, true]),
            Self::InvertY => ([0, 1, 2], [false, true, false]),
            Self::InvertX => ([0, 1, 2], [true, false, false]),
        }
    }

    pub(crate) fn transform(self) -> ShapeTransform {
        let (permutation, inverted) = self.axes();
        ShapeTransform {
            permutation,
            inverted,
        }
    }
}

impl ShapeAxisCycle {
    pub(crate) fn inverse(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }

    pub(crate) fn cycle_axis(self, axis: Axis) -> Axis {
        match (self, axis) {
            (Self::None, axis) => axis,
            (Self::Forward, Axis::X) => Axis::Y,
            (Self::Forward, Axis::Y) => Axis::Z,
            (Self::Forward, Axis::Z) => Axis::X,
            (Self::Backward, Axis::X) => Axis::Z,
            (Self::Backward, Axis::Y) => Axis::X,
            (Self::Backward, Axis::Z) => Axis::Y,
        }
    }

    pub(crate) fn cycle_index(self, point: [isize; 3], axis: Axis) -> isize {
        match (self, axis) {
            (Self::None, Axis::X) => point[0],
            (Self::None, Axis::Y) => point[1],
            (Self::None, Axis::Z) => point[2],
            (Self::Forward, Axis::X) => point[2],
            (Self::Forward, Axis::Y) => point[0],
            (Self::Forward, Axis::Z) => point[1],
            (Self::Backward, Axis::X) => point[1],
            (Self::Backward, Axis::Y) => point[2],
            (Self::Backward, Axis::Z) => point[0],
        }
    }
}

impl ShapeTransform {
    pub(crate) fn is_identity(self) -> bool {
        self.permutation == [0, 1, 2] && self.inverted == [false, false, false]
    }

    /// Compose in the same order as Java OctahedralGroup.compose.
    pub(crate) fn compose(self, that: Self) -> Self {
        Self {
            permutation: [
                that.permutation[self.permutation[0]],
                that.permutation[self.permutation[1]],
                that.permutation[self.permutation[2]],
            ],
            inverted: [
                self.inverted[0] ^ that.inverted[self.permutation[0]],
                self.inverted[1] ^ that.inverted[self.permutation[1]],
                self.inverted[2] ^ that.inverted[self.permutation[2]],
            ],
        }
    }
}

impl AttachFace {
    pub fn name(self) -> &'static str {
        match self {
            Self::Floor => "floor",
            Self::Wall => "wall",
            Self::Ceiling => "ceiling",
        }
    }
}
