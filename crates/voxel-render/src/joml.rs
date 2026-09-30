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

    pub fn rotate(&mut self, quat: &Quaternion) {
        self.pose.rotate(quat);
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
