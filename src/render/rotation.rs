//! Element rotation: the `rotation` block a model element may carry.
//!
//! Every block that is not an axis-aligned box goes through this - stairs,
//! torches, rails, fences. The matrices are built the way the reference's
//! maths library builds them, which is not the way the textbook formulas read:
//! the cosine is derived from the sine rather than computed directly, and that
//! derivation is what produces a negative zero at a quarter turn.

/// Column-major 4x4, indexed `[column][row]`, matching the reference's layout.
pub type Matrix4 = [[f32; 4]; 4];

/// The identity transform.
pub const IDENTITY: Matrix4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Which axis an element rotates about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationAxis {
    /// The X axis.
    X,
    /// The Y axis.
    Y,
    /// The Z axis.
    Z,
}

/// Degrees to radians, narrowed to `f32` first exactly as the reference does.
///
/// Converting in `f64` and narrowing afterwards gives a different last bit, so
/// the factor is narrowed before the multiply.
const DEGREES_TO_RADIANS: f32 = (std::f64::consts::PI / 180.0) as f32;
const TWO_PI: f32 = (std::f64::consts::PI * 2.0) as f32;
const HALF_PI: f32 = (std::f64::consts::PI * 0.5) as f32;

/// The reference maths library's sine: computed in double, returned as float.
fn sin(angle: f32) -> f32 {
    f64::from(angle).sin() as f32
}

/// The reference maths library's `cosFromSin`.
///
/// The magnitude comes from the Pythagorean identity and the sign from the
/// quadrant, which is why a quarter turn yields negative zero rather than zero.
fn cos_from_sin(sin: f32, angle: f32) -> f32 {
    let cos = (1.0_f32 - sin * sin).sqrt();
    let shifted = angle + HALF_PI;
    let mut wrapped = shifted - (shifted / TWO_PI) as i32 as f32 * TWO_PI;
    if wrapped < 0.0 {
        // IEEE addition is commutative, so the operand order here is free.
        wrapped += TWO_PI;
    }
    if wrapped >= std::f32::consts::PI {
        return -cos;
    }
    cos
}

/// A rotation of `degrees` about one axis, as the reference builds it.
pub fn axis_rotation(axis: RotationAxis, degrees: f32) -> Matrix4 {
    // The reference short-circuits a zero angle to the identity before any
    // trigonometry, so no negative zero appears there.
    if degrees == 0.0 {
        return IDENTITY;
    }
    let radians = degrees * DEGREES_TO_RADIANS;
    let sine = sin(radians);
    let cosine = cos_from_sin(sine, radians);
    let mut matrix = IDENTITY;
    match axis {
        RotationAxis::X => {
            matrix[1][1] = cosine;
            matrix[1][2] = sine;
            matrix[2][1] = -sine;
            matrix[2][2] = cosine;
        }
        RotationAxis::Y => {
            matrix[0][0] = cosine;
            matrix[0][2] = -sine;
            matrix[2][0] = sine;
            matrix[2][2] = cosine;
        }
        RotationAxis::Z => {
            matrix[0][0] = cosine;
            matrix[0][1] = sine;
            matrix[1][0] = -sine;
            matrix[1][1] = cosine;
        }
    }
    matrix
}

/// Whether a matrix is the identity, as the reference's check sees it.
fn is_identity(matrix: &Matrix4) -> bool {
    *matrix == IDENTITY
}

/// Transform a direction: the upper 3x3 only, ignoring translation.
fn transform_direction(matrix: &Matrix4, value: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|row| {
        matrix[0][row] * value[0] + matrix[1][row] * value[1] + matrix[2][row] * value[2]
    })
}

/// Transform a position, including translation.
pub fn transform_position(matrix: &Matrix4, value: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|row| {
        matrix[0][row] * value[0]
            + matrix[1][row] * value[1]
            + matrix[2][row] * value[2]
            + matrix[3][row]
    })
}

/// The scale `rescale` applies: each axis grown so its longest component
/// reaches one again, which is what keeps a rotated element touching the block
/// edge it started from.
fn rescale_factors(matrix: &Matrix4) -> [f32; 3] {
    let units = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    units.map(|unit| {
        let transformed = transform_direction(matrix, unit);
        let longest = transformed[0]
            .abs()
            .max(transformed[1].abs())
            .max(transformed[2].abs());
        1.0 / longest
    })
}

/// Scale the basis columns in place, as the reference's `scale` does.
fn scale_columns(matrix: &mut Matrix4, factors: [f32; 3]) {
    for (column, factor) in matrix.iter_mut().take(3).zip(factors) {
        for slot in column.iter_mut() {
            *slot *= factor;
        }
    }
}

/// Java `CuboidRotation`'s transform: the axis rotation, rescaled if asked.
///
/// The rescale is skipped when the rotation came out as the identity, so a zero
/// angle stays untouched.
pub fn element_transform(axis: RotationAxis, degrees: f32, rescale: bool) -> Matrix4 {
    let mut matrix = axis_rotation(axis, degrees);
    if rescale && !is_identity(&matrix) {
        let factors = rescale_factors(&matrix);
        scale_columns(&mut matrix, factors);
    }
    matrix
}

/// Java `FaceBakery.rotateVertexBy`: move to the origin, transform, move back.
pub fn rotate_vertex(vertex: [f32; 3], origin: [f32; 3], matrix: &Matrix4) -> [f32; 3] {
    let moved = [
        vertex[0] - origin[0],
        vertex[1] - origin[1],
        vertex[2] - origin[2],
    ];
    let transformed = transform_position(matrix, moved);
    [
        transformed[0] + origin[0],
        transformed[1] + origin[1],
        transformed[2] + origin[2],
    ]
}
