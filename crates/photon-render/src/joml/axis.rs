//! `com.mojang.math.Axis` applied straight to a matrix: the version's `PoseStack.rotate` and
//! `rotateDegrees` turn the position and normal matrices with `rotateX`, `rotateY` or `rotateZ`
//! (a sine and a cosine each), not through a quaternion.

use super::*;

/// One of the six axes, `Axis.XP` to `Axis.ZN`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    XP,
    XN,
    YP,
    YN,
    ZP,
    ZN,
}

impl Axis {
    /// The rotation about the positive axis that has the same effect, as an axis index and the
    /// sign the angle takes.
    fn parts(self) -> (usize, f32) {
        match self {
            Self::XP => (0, 1.0),
            Self::XN => (0, -1.0),
            Self::YP => (1, 1.0),
            Self::YN => (1, -1.0),
            Self::ZP => (2, 1.0),
            Self::ZN => (2, -1.0),
        }
    }

    /// `Axis.rotateDegrees`'s conversion of degrees to radians.
    pub fn radians(degrees: f32) -> f32 {
        degrees * (std::f64::consts::PI / 180.0) as f32
    }
}

/// The two columns an axis mixes and the sign of the sine between them: x turns y into z, y turns
/// z into x, and z turns x into y, which the library writes for y with the sine negated.
fn columns(axis: usize) -> (usize, usize, f32) {
    match axis {
        0 => (1, 2, 1.0),
        1 => (0, 2, -1.0),
        _ => (0, 1, 1.0),
    }
}

impl Matrix4 {
    /// `Matrix4f.rotateX/Y/Z(angle)` for the axis index 0, 1 or 2.
    pub(super) fn rotate_about(&mut self, axis: usize, angle: f32) {
        let sin = sin(angle);
        let cos = cos_from_sin(sin, angle);
        let (p, q, sign) = columns(axis);
        let s = sign * sin;
        if self.properties & (IDENTITY | TRANSLATION) != 0 {
            let translation = [self.m[3][0], self.m[3][1], self.m[3][2]];
            *self = Self::identity();
            self.m[p][p] = cos;
            self.m[p][q] = s;
            self.m[q][p] = -s;
            self.m[q][q] = cos;
            self.m[3][..3].copy_from_slice(&translation);
            self.properties = AFFINE | ORTHONORMAL;
            return;
        }
        for row in 0..4 {
            let (a, b) = (self.m[p][row], self.m[q][row]);
            self.m[p][row] = fma(a, cos, b * s);
            self.m[q][row] = fma(a, -s, b * cos);
        }
        self.properties &= !(PERSPECTIVE | IDENTITY | TRANSLATION);
    }
}

impl Matrix3 {
    /// `Matrix3f.rotateX/Y/Z(angle)` for the axis index 0, 1 or 2.
    pub(super) fn rotate_about(&mut self, axis: usize, angle: f32) {
        let sin = sin(angle);
        let cos = cos_from_sin(sin, angle);
        let (p, q, sign) = columns(axis);
        let s = sign * sin;
        for row in 0..3 {
            let (a, b) = (self.m[p][row], self.m[q][row]);
            self.m[p][row] = a * cos + b * s;
            self.m[q][row] = a * -s + b * cos;
        }
    }
}

impl Matrix4 {
    /// `Matrix4f.rotateX/Y/Z(angle)` about an axis (negated for the negative ones).
    pub fn rotate_axis(&mut self, axis: Axis, angle: f32) {
        let (index, sign) = axis.parts();
        self.rotate_about(index, sign * angle);
    }
}

impl Pose {
    /// `PoseStack.Pose.rotate(axis, angle)`.
    pub fn rotate_axis(&mut self, axis: Axis, angle: f32) {
        let (index, sign) = axis.parts();
        self.pose.rotate_about(index, sign * angle);
        self.normal.rotate_about(index, sign * angle);
    }

    /// `PoseStack.Pose.rotateDegrees(axis, angle)`.
    pub fn rotate_degrees(&mut self, axis: Axis, degrees: f32) {
        self.rotate_axis(axis, Axis::radians(degrees));
    }
}
