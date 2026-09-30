//! UV lock: with it on, a face's texture keeps its world orientation while
//! the model turns (`BlockModelRotation`, `BlockMath.getFaceTransformation`).
//! The reference works each turn's face transforms out once at startup by
//! composing matrices and inverting the result, and so does this, with the
//! matrix library's own rounding: the entries hold values like `0.99999994`
//! that a direct computation would not.

use std::sync::OnceLock;

use voxel_core::direction::Direction;

use crate::joml::{Matrix4, Quaternion};
use crate::rotation::{FlatMatrix, Matrix4 as RotationMatrix, flatten, model_state_transform};

/// The directions in the order the reference lists them.
const DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// One turn's face transforms, by direction.
pub struct FaceTransforms {
    /// The whole-model transform they belong to.
    pub transform: FlatMatrix,
    /// What a face's own frame becomes under the turn.
    pub forward: [FlatMatrix; 6],
    /// The inverse of each, which is what baking applies to a texture
    /// coordinate.
    pub inverse: [FlatMatrix; 6],
}

fn flat(matrix: &Matrix4) -> FlatMatrix {
    flatten(&matrix.m)
}

/// `BlockMath.VANILLA_UV_TRANSFORM_LOCAL_TO_GLOBAL`: the turn from a face's
/// own frame to the world's, for a face looking in `side`.
fn local_to_global(side: Direction) -> Matrix4 {
    use std::f64::consts::PI;
    let quarter = (PI / 2.0) as f32;
    let rotation = match side {
        Direction::South => return Matrix4::identity(),
        Direction::East => Quaternion::identity().rotate_y(quarter),
        Direction::West => Quaternion::identity().rotate_y(-quarter),
        Direction::North => Quaternion::identity().rotate_y(PI as f32),
        Direction::Up => Quaternion::identity().rotate_x(-quarter),
        Direction::Down => Quaternion::identity().rotate_x(quarter),
    };
    let mut matrix = Matrix4::identity();
    matrix.rotate(&rotation);
    matrix
}

/// `Transformation.inverse`: the identity is its own inverse.
fn inverse(matrix: &Matrix4) -> Matrix4 {
    if matrix.is_identity() {
        *matrix
    } else {
        matrix.invert_affine()
    }
}

/// `Direction.getApproximateNearest(x, y, z)`: the first direction whose
/// normal has the highest dot product, north if none is above zero.
fn nearest(normal: [f32; 3]) -> Direction {
    let mut result = Direction::North;
    let mut highest = f32::from_bits(1);
    for direction in DIRECTIONS {
        let (x, y, z) = direction.step();
        let dot = normal[0] * x as f32 + normal[1] * y as f32 + normal[2] * z as f32;
        if dot > highest {
            highest = dot;
            result = direction;
        }
    }
    result
}

/// `BlockModelRotation`'s face mapping for a whole-model turn.
fn compute(transform: &RotationMatrix) -> FaceTransforms {
    let columns: [[f32; 3]; 3] =
        std::array::from_fn(|column| std::array::from_fn(|row| transform[column][row]));
    let group = Matrix4::from_affine(columns);
    let forward_matrices: [Matrix4; 6] = if group.is_identity() {
        // `Transformation.IDENTITY` stands for the turn and is returned as is.
        [Matrix4::identity(); 6]
    } else {
        DIRECTIONS.map(|side| {
            let action = group.mul(&local_to_global(side));
            let [x, y, z] = action.transform_direction(0.0, 0.0, 1.0);
            let new_side = nearest([x, y, z]);
            inverse(&local_to_global(new_side)).mul(&action)
        })
    };
    FaceTransforms {
        transform: flatten(transform),
        forward: forward_matrices.each_ref().map(flat),
        inverse: forward_matrices.map(|matrix| flat(&matrix.invert_affine())),
    }
}

/// The face transforms of every turn a variant can make, worked out once.
fn all() -> &'static [FaceTransforms] {
    static ALL: OnceLock<Vec<FaceTransforms>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut turns: Vec<FaceTransforms> = Vec::new();
        for x in [0, 90, 180, 270] {
            for y in [0, 90, 180, 270] {
                for z in [0, 90, 180, 270] {
                    let transform =
                        model_state_transform(x, y, z).expect("quarter turns are turns");
                    if !turns
                        .iter()
                        .any(|turn| turn.transform == flatten(&transform))
                    {
                        turns.push(compute(&transform));
                    }
                }
            }
        }
        turns
    })
}

/// The face transforms for one whole-model transform.
pub fn for_transform(transform: &FlatMatrix) -> Option<&'static FaceTransforms> {
    all().iter().find(|turn| turn.transform == *transform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cube_group_has_twenty_four_turns() {
        assert_eq!(all().len(), 24);
    }

    #[test]
    fn no_turn_leaves_the_faces_alone_but_the_identity() {
        let identity = for_transform(&flatten(&crate::rotation::IDENTITY)).expect("identity");
        assert!(
            identity
                .forward
                .iter()
                .all(|matrix| *matrix == flat(&Matrix4::identity()))
        );
        let quarter = model_state_transform(0, 90, 0).expect("a quarter turn");
        let turned = for_transform(&flatten(&quarter)).expect("turn");
        assert!(
            turned
                .forward
                .iter()
                .any(|matrix| *matrix != flat(&Matrix4::identity()))
        );
    }
}
