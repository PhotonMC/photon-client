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

/// A three-axis rotation, as the reference's `rotationZYX` composes it.
///
/// The composition is fused rather than three matrix products: the library
/// writes some elements straight from a negated sine instead of deriving them
/// through a sum. That is observable - a zero rotation about Y puts a
/// **negative zero** where a product of three rotations would leave positive
/// zero - so the element order here follows the library rather than the algebra.
pub fn euler_rotation(x_degrees: f32, y_degrees: f32, z_degrees: f32) -> Matrix4 {
    // Each angle is converted separately with the narrowed factor.
    let angle_x = x_degrees * DEGREES_TO_RADIANS;
    let angle_y = y_degrees * DEGREES_TO_RADIANS;
    let angle_z = z_degrees * DEGREES_TO_RADIANS;
    let sin_x = sin(angle_x);
    let cos_x = cos_from_sin(sin_x, angle_x);
    let sin_y = sin(angle_y);
    let cos_y = cos_from_sin(sin_y, angle_y);
    let sin_z = sin(angle_z);
    let cos_z = cos_from_sin(sin_z, angle_z);
    let neg_sin_x = -sin_x;
    let neg_sin_y = -sin_y;
    let neg_sin_z = -sin_z;

    // The Z rotation, kept as scalars.
    let z00 = cos_z;
    let z01 = sin_z;
    let z10 = neg_sin_z;
    let z11 = cos_z;
    // Folded through the Y rotation.
    let y20 = z00 * sin_y;
    let y21 = z01 * sin_y;
    let y22 = cos_y;

    let mut matrix = [[0.0_f32; 4]; 4];
    matrix[0][0] = z00 * cos_y;
    matrix[0][1] = z01 * cos_y;
    matrix[0][2] = neg_sin_y;
    matrix[0][3] = 0.0;
    matrix[1][0] = z10 * cos_x + y20 * sin_x;
    matrix[1][1] = z11 * cos_x + y21 * sin_x;
    matrix[1][2] = y22 * sin_x;
    matrix[1][3] = 0.0;
    matrix[2][0] = z10 * neg_sin_x + y20 * cos_x;
    matrix[2][1] = z11 * neg_sin_x + y21 * cos_x;
    matrix[2][2] = y22 * cos_x;
    matrix[2][3] = 0.0;
    matrix[3] = [0.0, 0.0, 0.0, 1.0];
    matrix
}

/// Java `CuboidRotation`'s transform for a three-axis rotation block.
pub fn euler_element_transform(
    x_degrees: f32,
    y_degrees: f32,
    z_degrees: f32,
    rescale: bool,
) -> Matrix4 {
    let mut matrix = euler_rotation(x_degrees, y_degrees, z_degrees);
    if rescale && !is_identity(&matrix) {
        let factors = rescale_factors(&matrix);
        scale_columns(&mut matrix, factors);
    }
    matrix
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

/// A blockstate variant's whole-model rotation.
///
/// A variant may turn its model by quarter turns about all three axes. The
/// reference composes these through its cube symmetry group, whose matrices
/// hold only zero and plus or minus one, so the composition here is exact and
/// needs no trigonometry.
///
/// The order is not free: X is applied first, then Y wraps around it, then Z
/// wraps around both. The group is not commutative, so swapping two of them
/// gives a different orientation.
pub fn model_state_transform(x_degrees: i32, y_degrees: i32, z_degrees: i32) -> Option<Matrix4> {
    let mut result = IDENTITY;
    for (degrees, generator) in [
        (x_degrees, &ROTATE_X_QUARTER),
        (y_degrees, &ROTATE_Y_QUARTER),
        (z_degrees, &ROTATE_Z_QUARTER),
    ] {
        for _ in 0..quarter_turns(degrees)? {
            result = exact_multiply(generator, &result);
        }
    }
    Some(result)
}

/// How many quarter turns an angle names, or nothing if it is not a multiple.
fn quarter_turns(degrees: i32) -> Option<u32> {
    match photon_core::math::positive_modulo_i32(degrees, 360) {
        0 => Some(0),
        90 => Some(1),
        180 => Some(2),
        270 => Some(3),
        _ => None,
    }
}

/// One quarter turn about X, taking `(x, y, z)` to `(x, z, -y)`.
const ROTATE_X_QUARTER: Matrix4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, -1.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// One quarter turn about Z, taking `(x, y, z)` to `(-y, x, z)`.
const ROTATE_Z_QUARTER: Matrix4 = [
    [0.0, -1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// One quarter turn about Y, taking `(x, y, z)` to `(-z, y, x)`.
const ROTATE_Y_QUARTER: Matrix4 = [
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Column-major product over values that are only zero and plus or minus one.
fn exact_multiply(left: &Matrix4, right: &Matrix4) -> Matrix4 {
    std::array::from_fn(|column| {
        std::array::from_fn(|row| (0..4).map(|k| left[k][row] * right[column][k]).sum())
    })
}

/// One matrix of the UV lock face transforms, as sixteen floats flattened
/// column by column.
pub type FlatMatrix = [f32; 16];

/// The identity, in the same flattened form.
pub const FLAT_IDENTITY: FlatMatrix = [
    1.0, 0.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, 0.0, //
    0.0, 0.0, 1.0, 0.0, //
    0.0, 0.0, 0.0, 1.0,
];

/// Flatten a transform column by column, as the reference's matrix library
/// stores one.
pub fn flatten(matrix: &Matrix4) -> FlatMatrix {
    let mut out = [0.0_f32; 16];
    for (column, values) in matrix.iter().enumerate() {
        out[column * 4..column * 4 + 4].copy_from_slice(values);
    }
    out
}

/// The inverse face transform a UV-locked variant applies to one face.
///
/// Without UV lock every face is left at identity. With it on, the transform
/// comes from the face transforms of the rotation, which are looked up by the
/// whole-model transform the three angles compose to.
pub fn inverse_face_transform(
    x_degrees: i32,
    y_degrees: i32,
    z_degrees: i32,
    facing: photon_core::direction::Direction,
    uv_lock: bool,
) -> Option<FlatMatrix> {
    let transform = flatten(&model_state_transform(x_degrees, y_degrees, z_degrees)?);
    if !uv_lock {
        return Some(FLAT_IDENTITY);
    }
    Some(crate::block::uv_lock::for_transform(&transform)?.inverse[facing as usize])
}

/// The forward face transform, which the reference computes on the way to the
/// inverse above. Nothing in the baking path reads it; it is here so the
/// parity run compares both halves.
pub fn face_transform(
    x_degrees: i32,
    y_degrees: i32,
    z_degrees: i32,
    facing: photon_core::direction::Direction,
    uv_lock: bool,
) -> Option<FlatMatrix> {
    let transform = flatten(&model_state_transform(x_degrees, y_degrees, z_degrees)?);
    if !uv_lock {
        return Some(FLAT_IDENTITY);
    }
    Some(crate::block::uv_lock::for_transform(&transform)?.forward[facing as usize])
}

/// Whether a flattened transform is the identity.
///
/// The comparison is by value rather than by bits, and both halves of that
/// matter. A negative zero counts as zero, which several of the tables carry
/// and the reference still calls identity. A value a hair away from one does
/// *not* count, so those tables take the transforming path and shift the
/// texture coordinate slightly rather than passing it through.
pub fn is_flat_identity(matrix: &FlatMatrix) -> bool {
    matrix
        .iter()
        .zip(&FLAT_IDENTITY)
        .all(|(value, expected)| value == expected)
}

/// Apply a face's UV transform to one texture coordinate.
///
/// The coordinate is moved to the face's centre before the transform and back
/// afterwards, because the transform turns about the origin.
pub fn transform_face_uv(matrix: &FlatMatrix, u: f32, v: f32) -> (f32, f32) {
    if is_flat_identity(matrix) {
        return (u, v);
    }
    let x = u - 0.5;
    let y = v - 0.5;
    // Column major: the entry for column c and row r sits at c * 4 + r, and a
    // position is transformed with the translation column added in.
    let transformed_u = matrix[0] * x + matrix[4] * y + matrix[12];
    let transformed_v = matrix[1] * x + matrix[5] * y + matrix[13];
    (transformed_u + 0.5, transformed_v + 0.5)
}

/// Where a face's cullface direction lands once the variant is turned.
///
/// A face names the side it hides behind in the model's own frame. Turning the
/// model turns that side too, so the neighbour to test against is found by
/// pushing the direction's normal through the transform and taking the face it
/// ends up pointing at.
pub fn rotate_cull_direction(
    x_degrees: i32,
    y_degrees: i32,
    z_degrees: i32,
    facing: photon_core::direction::Direction,
) -> Option<photon_core::direction::Direction> {
    Some(rotate_direction(
        &model_state_transform(x_degrees, y_degrees, z_degrees)?,
        facing,
    ))
}

/// The same, given the transform directly.
pub fn rotate_direction(
    matrix: &Matrix4,
    facing: photon_core::direction::Direction,
) -> photon_core::direction::Direction {
    let unit = crate::block::geometry::direction_unit(facing);
    crate::block::geometry::approximate_nearest_direction(transform_direction(matrix, unit))
}
