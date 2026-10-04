//! The few matrix operations entity models are posed with, rounded the way
//! the reference's maths library rounds them.
//!
//! Every product and sum is a single `f32` operation in the library's own
//! order, and its fused multiply-add is off by default, so a "fma" here is a
//! plain multiply then add. The matrix keeps the library's property flags,
//! because they choose which of its code paths an operation takes, and the
//! paths round differently.

// Matrix entries are addressed by column and row, as the library writes them.
#![expect(
    clippy::needless_range_loop,
    reason = "index loops that follow JOML's element-wise arithmetic"
)]

/// The matrix is a perspective projection.
const PERSPECTIVE: u8 = 1;
/// The last row is (0, 0, 0, 1).
const AFFINE: u8 = 2;
/// The matrix is the identity.
const IDENTITY: u8 = 4;
/// The matrix only translates.
const TRANSLATION: u8 = 8;
/// The upper 3x3 is orthonormal.
const ORTHONORMAL: u8 = 16;

fn fma(a: f32, b: f32, c: f32) -> f32 {
    a * b + c
}

/// `org.joml.Math.sin(float)`: the double sine of the widened angle.
pub fn sin(angle: f32) -> f32 {
    f64::from(angle).sin() as f32
}

/// `org.joml.Math.cosFromSin(float, float)`.
pub fn cos_from_sin(sin: f32, angle: f32) -> f32 {
    use std::f64::consts::PI;
    let cos = f64::from(1.0 - sin * sin).sqrt() as f32;
    let a = angle + (PI / 2.0) as f32;
    let turn = (PI * 2.0) as f32;
    let mut b = a - (a / turn) as i32 as f32 * turn;
    if f64::from(b) < 0.0 {
        b += turn;
    }
    if b >= PI as f32 { -cos } else { cos }
}

/// A rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    /// `Quaternionf.rotationZYX`: about z, then y, then x.
    pub fn rotation_zyx(angle_z: f32, angle_y: f32, angle_x: f32) -> Self {
        let sx = sin(angle_x * 0.5);
        let cx = cos_from_sin(sx, angle_x * 0.5);
        let sy = sin(angle_y * 0.5);
        let cy = cos_from_sin(sy, angle_y * 0.5);
        let sz = sin(angle_z * 0.5);
        let cz = cos_from_sin(sz, angle_z * 0.5);
        let cycz = cy * cz;
        let sysz = sy * sz;
        let sycz = sy * cz;
        let cysz = cy * sz;
        Self {
            w: cx * cycz + sx * sysz,
            x: sx * cycz - cx * sysz,
            y: cx * sycz + sx * cysz,
            z: cx * cysz - sx * sycz,
        }
    }

    /// `Quaternionf.rotationYXZ`: about y, then x, then z.
    pub fn rotation_yxz(angle_y: f32, angle_x: f32, angle_z: f32) -> Self {
        let sx = sin(angle_x * 0.5);
        let cx = cos_from_sin(sx, angle_x * 0.5);
        let sy = sin(angle_y * 0.5);
        let cy = cos_from_sin(sy, angle_y * 0.5);
        let sz = sin(angle_z * 0.5);
        let cz = cos_from_sin(sz, angle_z * 0.5);
        let x = cy * sx;
        let y = sy * cx;
        let z = sy * sx;
        let w = cy * cx;
        Self {
            x: x * cz + y * sz,
            y: y * cz - x * sz,
            z: w * sz - z * cz,
            w: w * cz + z * sz,
        }
    }

    /// `Quaternionf.rotationXYZ`: about x, then y, then z.
    pub fn rotation_xyz(angle_x: f32, angle_y: f32, angle_z: f32) -> Self {
        let sx = sin(angle_x * 0.5);
        let cx = cos_from_sin(sx, angle_x * 0.5);
        let sy = sin(angle_y * 0.5);
        let cy = cos_from_sin(sy, angle_y * 0.5);
        let sz = sin(angle_z * 0.5);
        let cz = cos_from_sin(sz, angle_z * 0.5);
        let cycz = cy * cz;
        let sysz = sy * sz;
        let sycz = sy * cz;
        let cysz = cy * sz;
        Self {
            w: cx * cycz - sx * sysz,
            x: sx * cycz + cx * sysz,
            y: cx * sycz - sx * cysz,
            z: cx * cysz + sx * sycz,
        }
    }

    /// `Quaternionf.rotationX`.
    pub fn rotation_x(angle: f32) -> Self {
        let sin = sin(angle * 0.5);
        let cos = cos_from_sin(sin, angle * 0.5);
        Self {
            x: sin,
            y: 0.0,
            z: 0.0,
            w: cos,
        }
    }

    /// `Quaternionf.rotationY`.
    pub fn rotation_y(angle: f32) -> Self {
        let sin = sin(angle * 0.5);
        let cos = cos_from_sin(sin, angle * 0.5);
        Self {
            x: 0.0,
            y: sin,
            z: 0.0,
            w: cos,
        }
    }

    /// `Quaternionf.rotationZ`.
    pub fn rotation_z(angle: f32) -> Self {
        let sin = sin(angle * 0.5);
        let cos = cos_from_sin(sin, angle * 0.5);
        Self {
            x: 0.0,
            y: 0.0,
            z: sin,
            w: cos,
        }
    }

    /// `Axis.rotationDegrees`: degrees turned to radians in `f32`.
    pub fn degrees_to_radians(degrees: f32) -> f32 {
        degrees * (std::f64::consts::PI / 180.0) as f32
    }

    /// `new Quaternionf()`.
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }

    /// `Quaternionf.setAngleAxis(angle, x, y, z)`: a turn of `angle` about the
    /// axis, which is used as given (the library does not normalise it).
    pub fn angle_axis(angle: f32, x: f32, y: f32, z: f32) -> Self {
        let half = angle * 0.5;
        let sin = sin(half);
        Self {
            x: x * sin,
            y: y * sin,
            z: z * sin,
            w: cos_from_sin(sin, half),
        }
    }

    /// `Quaternionf.rotateX(angle)`: this turned further about x.
    pub fn rotate_x(self, angle: f32) -> Self {
        let sin = sin(angle * 0.5);
        let cos = cos_from_sin(sin, angle * 0.5);
        Self {
            x: self.w * sin + self.x * cos,
            y: self.y * cos + self.z * sin,
            z: self.z * cos - self.y * sin,
            w: self.w * cos - self.x * sin,
        }
    }

    /// `Quaternionf.rotateY(angle)`: this turned further about y.
    pub fn rotate_y(self, angle: f32) -> Self {
        let sin = sin(angle * 0.5);
        let cos = cos_from_sin(sin, angle * 0.5);
        Self {
            x: self.x * cos - self.z * sin,
            y: self.w * sin + self.y * cos,
            z: self.x * sin + self.z * cos,
            w: self.w * cos - self.y * sin,
        }
    }

    /// The rotation's 3x3, in the library's shared expansion.
    fn expand(&self) -> [f32; 9] {
        let w2 = self.w * self.w;
        let x2 = self.x * self.x;
        let y2 = self.y * self.y;
        let z2 = self.z * self.z;
        let zw = self.z * self.w;
        let dzw = zw + zw;
        let xy = self.x * self.y;
        let dxy = xy + xy;
        let xz = self.x * self.z;
        let dxz = xz + xz;
        let yw = self.y * self.w;
        let dyw = yw + yw;
        let yz = self.y * self.z;
        let dyz = yz + yz;
        let xw = self.x * self.w;
        let dxw = xw + xw;
        [
            w2 + x2 - z2 - y2,
            dxy + dzw,
            dxz - dyw,
            -dzw + dxy,
            y2 - z2 + w2 - x2,
            dyz + dxw,
            dyw + dxz,
            dyz - dxw,
            z2 - y2 - x2 + w2,
        ]
    }
}

/// A 4x4 matrix, column-major (`m[column][row]`), with the library's
/// property flags.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4 {
    pub m: [[f32; 4]; 4],
    properties: u8,
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Matrix4 {
    pub fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            properties: IDENTITY | AFFINE | TRANSLATION | ORTHONORMAL,
        }
    }

    /// `new Matrix4f(Matrix3fc)`: a 3x3 (as columns) in the upper left, the
    /// rest the identity's, known only to be affine.
    pub fn from_affine(columns: [[f32; 3]; 3]) -> Self {
        let mut matrix = Self::identity();
        for (column, values) in columns.iter().enumerate() {
            matrix.m[column][..3].copy_from_slice(values);
        }
        matrix.properties = AFFINE;
        matrix
    }

    /// Whether the matrix is exactly the identity
    /// (`MatrixUtil.isIdentity`, which compares values).
    pub fn is_identity(&self) -> bool {
        self.properties & IDENTITY != 0 || self.m == Self::identity().m
    }

    /// `Matrix4f.mul(right)`: this times `right`, for the affine cases the
    /// reference's dispatch sends to `mulAffine` and the identity shortcuts.
    pub fn mul(&self, right: &Self) -> Self {
        if self.properties & IDENTITY != 0 {
            return *right;
        }
        if right.properties & IDENTITY != 0 {
            return *self;
        }
        assert!(
            self.properties & AFFINE != 0 && right.properties & AFFINE != 0,
            "only affine products are ported"
        );
        let (l, r) = (&self.m, &right.m);
        let mut out = [[0.0_f32; 4]; 4];
        for column in 0..3 {
            for row in 0..3 {
                out[column][row] = fma(
                    l[0][row],
                    r[column][0],
                    fma(l[1][row], r[column][1], l[2][row] * r[column][2]),
                );
            }
            out[column][3] = l[column][3];
        }
        for row in 0..3 {
            out[3][row] = fma(
                l[0][row],
                r[3][0],
                fma(l[1][row], r[3][1], fma(l[2][row], r[3][2], l[3][row])),
            );
        }
        out[3][3] = l[3][3];
        Self {
            m: out,
            properties: AFFINE | self.properties & right.properties & ORTHONORMAL,
        }
    }

    /// `Matrix4f.translation(x, y, z)`.
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut matrix = Self::identity();
        matrix.m[3][0] = x;
        matrix.m[3][1] = y;
        matrix.m[3][2] = z;
        matrix.properties = AFFINE | TRANSLATION | ORTHONORMAL;
        matrix
    }

    /// `Matrix4f.rotateAround(quat, ox, oy, oz)` for an affine matrix
    /// (`rotateAroundAffine`): the rotation about the point `(ox, oy, oz)`.
    pub fn rotate_around(&mut self, quat: &Quaternion, (ox, oy, oz): (f32, f32, f32)) {
        assert!(
            self.properties & AFFINE != 0 && self.properties & IDENTITY == 0,
            "only affine matrices are rotated about a point"
        );
        // `rotateAroundAffine`: plain products and sums, no fused multiply-add.
        let r = quat.expand();
        let m = self.m;
        let mut out = [[0.0_f32; 4]; 4];
        let column_of =
            |rm: [f32; 3], row: usize| m[0][row] * rm[0] + m[1][row] * rm[1] + m[2][row] * rm[2];
        let columns = [[r[0], r[1], r[2]], [r[3], r[4], r[5]], [r[6], r[7], r[8]]];
        for row in 0..3 {
            for (index, rm) in columns.into_iter().enumerate() {
                out[index][row] = column_of(rm, row);
            }
        }
        for row in 0..3 {
            let moved = m[0][row] * ox + m[1][row] * oy + m[2][row] * oz + m[3][row];
            out[3][row] = -out[0][row] * ox - out[1][row] * oy - m[2][row] * oz + moved;
        }
        out[3][3] = 1.0;
        self.m = out;
        self.properties &= !(PERSPECTIVE | IDENTITY | TRANSLATION);
    }

    /// `Matrix4f.invertAffine()`.
    pub fn invert_affine(&self) -> Self {
        let m = &self.m;
        let m11m00 = m[0][0] * m[1][1];
        let m10m01 = m[0][1] * m[1][0];
        let m10m02 = m[0][2] * m[1][0];
        let m12m00 = m[0][0] * m[1][2];
        let m12m01 = m[0][1] * m[1][2];
        let m11m02 = m[0][2] * m[1][1];
        let det =
            (m11m00 - m10m01) * m[2][2] + (m10m02 - m12m00) * m[2][1] + (m12m01 - m11m02) * m[2][0];
        let s = 1.0 / det;
        let m10m22 = m[1][0] * m[2][2];
        let m10m21 = m[1][0] * m[2][1];
        let m11m22 = m[1][1] * m[2][2];
        let m11m20 = m[1][1] * m[2][0];
        let m12m21 = m[1][2] * m[2][1];
        let m12m20 = m[1][2] * m[2][0];
        let m20m02 = m[2][0] * m[0][2];
        let m20m01 = m[2][0] * m[0][1];
        let m21m02 = m[2][1] * m[0][2];
        let m21m00 = m[2][1] * m[0][0];
        let m22m01 = m[2][2] * m[0][1];
        let m22m00 = m[2][2] * m[0][0];
        let nm31 = (m20m02 * m[3][1] - m20m01 * m[3][2] + m21m00 * m[3][2] - m21m02 * m[3][0]
            + m22m01 * m[3][0]
            - m22m00 * m[3][1])
            * s;
        let nm32 = (m11m02 * m[3][0] - m12m01 * m[3][0] + m12m00 * m[3][1] - m10m02 * m[3][1]
            + m10m01 * m[3][2]
            - m11m00 * m[3][2])
            * s;
        Self {
            m: [
                [
                    (m11m22 - m12m21) * s,
                    (m21m02 - m22m01) * s,
                    (m12m01 - m11m02) * s,
                    0.0,
                ],
                [
                    (m12m20 - m10m22) * s,
                    (m22m00 - m20m02) * s,
                    (m10m02 - m12m00) * s,
                    0.0,
                ],
                [
                    (m10m21 - m11m20) * s,
                    (m20m01 - m21m00) * s,
                    (m11m00 - m10m01) * s,
                    0.0,
                ],
                [
                    (m10m22 * m[3][1] - m10m21 * m[3][2] + m11m20 * m[3][2] - m11m22 * m[3][0]
                        + m12m21 * m[3][0]
                        - m12m20 * m[3][1])
                        * s,
                    nm31,
                    nm32,
                    1.0,
                ],
            ],
            properties: AFFINE,
        }
    }

    /// `Matrix4f.transformDirection(x, y, z)`: the upper 3x3 applied.
    pub fn transform_direction(&self, x: f32, y: f32, z: f32) -> [f32; 3] {
        let m = &self.m;
        [
            fma(m[0][0], x, fma(m[1][0], y, m[2][0] * z)),
            fma(m[0][1], x, fma(m[1][1], y, m[2][1] * z)),
            fma(m[0][2], x, fma(m[1][2], y, m[2][2] * z)),
        ]
    }

    /// `Matrix4f.translate(x, y, z)`.
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        if self.properties & IDENTITY != 0 {
            *self = Self::identity();
            self.m[3][0] = x;
            self.m[3][1] = y;
            self.m[3][2] = z;
            self.properties = AFFINE | TRANSLATION | ORTHONORMAL;
            return;
        }
        let m = self.m;
        for row in 0..4 {
            self.m[3][row] = fma(
                m[0][row],
                x,
                fma(m[1][row], y, fma(m[2][row], z, m[3][row])),
            );
        }
        self.properties &= !(PERSPECTIVE | IDENTITY);
    }

    /// `Matrix4f.rotate(Quaternionfc)`.
    pub fn rotate(&mut self, quat: &Quaternion) {
        let r = quat.expand();
        let m = self.m;
        if self.properties & IDENTITY != 0 {
            *self = Self::identity();
            self.set_upper(r);
            self.properties = AFFINE | ORTHONORMAL;
        } else if self.properties & TRANSLATION != 0 {
            self.set_upper(r);
            self.properties &= !(PERSPECTIVE | IDENTITY | TRANSLATION);
        } else if self.properties & AFFINE != 0 {
            for (column, (a, b, c)) in [(r[0], r[1], r[2]), (r[3], r[4], r[5]), (r[6], r[7], r[8])]
                .into_iter()
                .enumerate()
            {
                for row in 0..3 {
                    self.m[column][row] = m[0][row] * a + m[1][row] * b + m[2][row] * c;
                }
                self.m[column][3] = 0.0;
            }
            self.properties &= !(PERSPECTIVE | IDENTITY | TRANSLATION);
        } else {
            unimplemented!("rotating a projective matrix");
        }
    }

    fn set_upper(&mut self, r: [f32; 9]) {
        for column in 0..3 {
            for row in 0..3 {
                self.m[column][row] = r[column * 3 + row];
            }
            self.m[column][3] = 0.0;
        }
    }

    /// `Matrix4f.scale(x, y, z)`.
    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        let one = [x, y, z].iter().all(|value| value.abs() == 1.0);
        if self.properties & IDENTITY != 0 {
            *self = Self::identity();
            self.m[0][0] = x;
            self.m[1][1] = y;
            self.m[2][2] = z;
            self.properties = AFFINE | if one { ORTHONORMAL } else { 0 };
            return;
        }
        for (column, factor) in [x, y, z].into_iter().enumerate() {
            for row in 0..4 {
                self.m[column][row] *= factor;
            }
        }
        self.properties &=
            !(PERSPECTIVE | IDENTITY | TRANSLATION | if one { 0 } else { ORTHONORMAL });
    }

    /// `Matrix4f.transformPosition(x, y, z)`.
    pub fn transform_position(&self, x: f32, y: f32, z: f32) -> [f32; 3] {
        if self.properties & IDENTITY != 0 {
            return [x, y, z];
        }
        if self.properties & TRANSLATION != 0 {
            return [x + self.m[3][0], y + self.m[3][1], z + self.m[3][2]];
        }
        let m = &self.m;
        std::array::from_fn(|row| {
            fma(
                m[0][row],
                x,
                fma(m[1][row], y, fma(m[2][row], z, m[3][row])),
            )
        })
    }
}

/// A 3x3 matrix, column-major, the normal matrix a pose carries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3 {
    pub m: [[f32; 3]; 3],
}

impl Default for Matrix3 {
    fn default() -> Self {
        Self {
            m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
}

impl Matrix3 {
    /// `new Matrix3f(Matrix4fc)`: the upper left of a 4x4.
    pub fn from_matrix4(matrix: &Matrix4) -> Self {
        Self {
            m: std::array::from_fn(|column| std::array::from_fn(|row| matrix.m[column][row])),
        }
    }

    /// `Matrix3f.mul(right)`.
    pub fn mul(&mut self, right: &Self) {
        let (l, r) = (self.m, &right.m);
        for column in 0..3 {
            for row in 0..3 {
                self.m[column][row] = fma(
                    l[0][row],
                    r[column][0],
                    fma(l[1][row], r[column][1], l[2][row] * r[column][2]),
                );
            }
        }
    }

    /// `Matrix3f.rotate(Quaternionfc)`.
    pub fn rotate(&mut self, quat: &Quaternion) {
        // The 3x3 writes the second column's first entry as `dxy - dzw`,
        // which rounds exactly as the 4x4's `-dzw + dxy`.
        let r = quat.expand();
        let m = self.m;
        for (column, (a, b, c)) in [(r[0], r[1], r[2]), (r[3], r[4], r[5]), (r[6], r[7], r[8])]
            .into_iter()
            .enumerate()
        {
            for row in 0..3 {
                self.m[column][row] = m[0][row] * a + m[1][row] * b + m[2][row] * c;
            }
        }
    }

    /// `Matrix3f.scale(x, y, z)`.
    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        for (column, factor) in [x, y, z].into_iter().enumerate() {
            for row in 0..3 {
                self.m[column][row] *= factor;
            }
        }
    }

    /// `Matrix3f.transform(x, y, z)`.
    pub fn transform(&self, x: f32, y: f32, z: f32) -> [f32; 3] {
        let m = &self.m;
        std::array::from_fn(|row| fma(m[0][row], x, fma(m[1][row], y, m[2][row] * z)))
    }
}

/// `Vector3f.normalize()`.
pub fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length_squared = fma(v[0], v[0], fma(v[1], v[1], v[2] * v[2]));
    let scalar = 1.0 / f64::from(length_squared).sqrt() as f32;
    v.map(|component| component * scalar)
}

/// `PoseStack.Pose`: a position matrix and its normal matrix.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Pose {
    pub pose: Matrix4,
    pub normal: Matrix3,
    /// Whether the normal matrix keeps normals unit length.
    untrusted_normals: bool,
}

impl Pose {
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.pose.translate(x, y, z);
    }

    /// `PoseStack.mulPose(Matrix4fc)` for an orthonormal, affine matrix.
    pub fn mul_matrix(&mut self, matrix: &Matrix4) {
        self.pose = self.pose.mul(matrix);
        self.normal.mul(&Matrix3::from_matrix4(matrix));
    }

    pub fn rotate(&mut self, quat: &Quaternion) {
        self.pose.rotate(quat);
        self.normal.rotate(quat);
    }

    /// `PoseStack.rotateAround`.
    pub fn rotate_around(&mut self, quat: &Quaternion, point: (f32, f32, f32)) {
        self.pose.rotate_around(quat, point);
        self.normal.rotate(quat);
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.pose.scale(x, y, z);
        if x.abs() == y.abs() && y.abs() == z.abs() {
            if x < 0.0 || y < 0.0 || z < 0.0 {
                self.normal
                    .scale(java_signum(x), java_signum(y), java_signum(z));
            }
        } else {
            self.normal.scale(1.0 / x, 1.0 / y, 1.0 / z);
            self.untrusted_normals = true;
        }
    }

    /// `Pose.transformNormal`.
    pub fn transform_normal(&self, normal: [f32; 3]) -> [f32; 3] {
        let result = self.normal.transform(normal[0], normal[1], normal[2]);
        if self.untrusted_normals {
            normalize(result)
        } else {
            result
        }
    }
}

fn java_signum(value: f32) -> f32 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quarter_turn_about_y_takes_x_to_minus_z() {
        let mut pose = Pose::default();
        pose.rotate(&Quaternion::rotation_zyx(
            0.0,
            std::f32::consts::FRAC_PI_2,
            0.0,
        ));
        let [x, y, z] = pose.pose.transform_position(1.0, 0.0, 0.0);
        assert!(x.abs() < 1e-6 && y.abs() < 1e-6 && (z + 1.0).abs() < 1e-6);
    }
}
