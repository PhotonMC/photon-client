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

    /// `Quaternionf.rotateXYZ`: this rotation, then about x, then y, then z.
    pub fn rotate_xyz(&mut self, angle_x: f32, angle_y: f32, angle_z: f32) {
        let turn = Self::rotation_xyz(angle_x, angle_y, angle_z);
        let (w, x, y, z) = (turn.w, turn.x, turn.y, turn.z);
        let this = *self;
        *self = Self {
            x: fma(this.w, x, fma(this.x, w, fma(this.y, z, -this.z * y))),
            y: fma(this.w, y, fma(-this.x, z, fma(this.y, w, this.z * x))),
            z: fma(this.w, z, fma(this.x, y, fma(-this.y, x, this.z * w))),
            w: fma(this.w, w, fma(-this.x, x, fma(-this.y, y, -this.z * z))),
        };
    }

    /// `Quaternionf.setAngleAxis(angle, x, y, z)` on a new quaternion.
    pub fn set_angle_axis(angle: f32, x: f32, y: f32, z: f32) -> Self {
        let s = sin(angle * 0.5);
        Self {
            x: x * s,
            y: y * s,
            z: z * s,
            w: cos_from_sin(s, angle * 0.5),
        }
    }

    /// `Quaternionf.rotateAxis(angle, x, y, z)`: this turned further about an axis.
    pub fn rotate_axis(self, angle: f32, axis_x: f32, axis_y: f32, axis_z: f32) -> Self {
        let half = angle / 2.0;
        let sin_angle = sin(half);
        let inverse_length = 1.0
            / f64::from(fma(axis_x, axis_x, fma(axis_y, axis_y, axis_z * axis_z))).sqrt() as f32;
        let (rx, ry, rz) = (
            axis_x * inverse_length * sin_angle,
            axis_y * inverse_length * sin_angle,
            axis_z * inverse_length * sin_angle,
        );
        let rw = cos_from_sin(sin_angle, half);
        Self {
            x: fma(self.w, rx, fma(self.x, rw, fma(self.y, rz, -self.z * ry))),
            y: fma(self.w, ry, fma(-self.x, rz, fma(self.y, rw, self.z * rx))),
            z: fma(self.w, rz, fma(self.x, ry, fma(-self.y, rx, self.z * rw))),
            w: fma(self.w, rw, fma(-self.x, rx, fma(-self.y, ry, -self.z * rz))),
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

    /// `Quaternionf.rotateZ(angle)`: this turned further about z.
    pub fn rotate_z(self, angle: f32) -> Self {
        let sin = sin(angle * 0.5);
        let cos = cos_from_sin(sin, angle * 0.5);
        Self {
            x: self.x * cos + self.y * sin,
            y: self.y * cos - self.x * sin,
            z: self.w * sin + self.z * cos,
            w: self.w * cos - self.z * sin,
        }
    }

    /// `Quaternionf.transform(Vector3f)`: a vector turned by the rotation.
    pub fn transform(&self, [vx, vy, vz]: [f32; 3]) -> [f32; 3] {
        let w2 = self.w * self.w;
        let x2 = self.x * self.x;
        let y2 = self.y * self.y;
        let z2 = self.z * self.z;
        let zw = self.z * self.w;
        let xy = self.x * self.y;
        let xz = self.x * self.z;
        let yw = self.y * self.w;
        let yz = self.y * self.z;
        let xw = self.x * self.w;
        let k = 1.0 / (x2 + y2 + z2 + w2);
        [
            ((x2 - y2 - z2 + w2) * k).mul_add(
                vx,
                (2.0 * (xy - zw) * k).mul_add(vy, 2.0 * (xz + yw) * k * vz),
            ),
            (2.0 * (xy + zw) * k).mul_add(
                vx,
                ((y2 - x2 - z2 + w2) * k).mul_add(vy, 2.0 * (yz - xw) * k * vz),
            ),
            (2.0 * (xz - yw) * k).mul_add(
                vx,
                (2.0 * (yz + xw) * k).mul_add(vy, (z2 - x2 - y2 + w2) * k * vz),
            ),
        ]
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

    /// `Matrix3f.invert().transpose()`: the normal matrix of a pose whose matrix does not keep
    /// normals unit length (`Pose.computeNormalMatrix`).
    pub fn inverse_transpose(&self) -> Self {
        let m = |column: usize, row: usize| self.m[column][row];
        let (m00, m01, m02) = (m(0, 0), m(0, 1), m(0, 2));
        let (m10, m11, m12) = (m(1, 0), m(1, 1), m(1, 2));
        let (m20, m21, m22) = (m(2, 0), m(2, 1), m(2, 2));
        let a = fma(m00, m11, -m01 * m10);
        let b = fma(m02, m10, -m00 * m12);
        let c = fma(m01, m12, -m02 * m11);
        let d = fma(a, m22, fma(b, m21, c * m20));
        let s = 1.0 / d;
        let inverse = [
            [
                fma(m11, m22, -m21 * m12) * s,
                fma(m21, m02, -m01 * m22) * s,
                c * s,
            ],
            [
                fma(m20, m12, -m10 * m22) * s,
                fma(m00, m22, -m20 * m02) * s,
                b * s,
            ],
            [
                fma(m10, m21, -m20 * m11) * s,
                fma(m20, m01, -m00 * m21) * s,
                a * s,
            ],
        ];
        // Transposed: the row of the inverse is the column of the result.
        Self {
            m: std::array::from_fn(|column| std::array::from_fn(|row| inverse[row][column])),
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

    /// `Matrix3f.rotationZYX(angleZ, angleY, angleX)`.
    pub fn rotation_zyx(angle_z: f32, angle_y: f32, angle_x: f32) -> Self {
        let sin_x = sin(angle_x);
        let cos_x = cos_from_sin(sin_x, angle_x);
        let sin_y = sin(angle_y);
        let cos_y = cos_from_sin(sin_y, angle_y);
        let sin_z = sin(angle_z);
        let cos_z = cos_from_sin(sin_z, angle_z);
        let (m_sin_z, m_sin_y, m_sin_x) = (-sin_z, -sin_y, -sin_x);
        let (nm00, nm01, nm10, nm11) = (cos_z, sin_z, m_sin_z, cos_z);
        let (nm20, nm21, nm22) = (nm00 * sin_y, nm01 * sin_y, cos_y);
        Self {
            m: [
                [nm00 * cos_y, nm01 * cos_y, m_sin_y],
                [
                    nm10 * cos_x + nm20 * sin_x,
                    nm11 * cos_x + nm21 * sin_x,
                    nm22 * sin_x,
                ],
                [
                    nm10 * m_sin_x + nm20 * cos_x,
                    nm11 * m_sin_x + nm21 * cos_x,
                    nm22 * cos_x,
                ],
            ],
        }
    }

    /// `Matrix3f.getEulerAnglesZYX`: the angles about x, y and z.
    pub fn euler_angles_zyx(&self) -> [f32; 3] {
        let m = &self.m;
        [
            f64::from(m[1][2]).atan2(f64::from(m[2][2])) as f32,
            f64::from(-m[0][2]).atan2(f64::from(f64::from(1.0 - m[0][2] * m[0][2]).sqrt() as f32))
                as f32,
            f64::from(m[0][1]).atan2(f64::from(m[0][0])) as f32,
        ]
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

    /// `PoseStack.Pose.mulPose(Matrix4fc)` for an affine matrix: the normals follow it as it is,
    /// or are worked out again when it does not keep them unit length.
    pub fn mul_matrix(&mut self, matrix: &Matrix4) {
        self.pose = self.pose.mul(matrix);
        if !matrix.is_pure_translation() {
            if matrix.is_orthonormal() {
                self.normal.mul(&Matrix3::from_matrix4(matrix));
            } else {
                // `computeNormalMatrix`.
                self.normal = Matrix3::from_matrix4(&self.pose).inverse_transpose();
                self.untrusted_normals = true;
            }
        }
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

mod matrix4;

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
