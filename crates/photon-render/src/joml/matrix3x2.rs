//! `Matrix3x2f`, the pose of the GUI: translate, scale and rotate in the library's own order of
//! single-precision operations (its fused multiply-add is off, so a "fma" is a multiply and an
//! add).

use super::sin;

/// A 2D pose `[m00, m01, m10, m11, m20, m21]`; a point `(x, y)` goes to
/// `(m00 x + m10 y + m20, m01 x + m11 y + m21)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x2 {
    pub m: [f32; 6],
}

impl Default for Matrix3x2 {
    fn default() -> Self {
        Self {
            m: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}

impl Matrix3x2 {
    /// `translate(x, y)`.
    pub fn translate(&mut self, x: f32, y: f32) {
        let [m00, m01, m10, m11, m20, m21] = self.m;
        self.m[4] = m00 * x + m10 * y + m20;
        self.m[5] = m01 * x + m11 * y + m21;
    }

    /// `scale(x, y)`.
    pub fn scale(&mut self, x: f32, y: f32) {
        self.m[0] *= x;
        self.m[1] *= x;
        self.m[2] *= y;
        self.m[3] *= y;
    }

    /// `rotate(angle)`, in radians: the cosine is the double cosine of the widened angle, not
    /// derived from the sine.
    pub fn rotate(&mut self, angle: f32) {
        let s = sin(angle);
        let c = f64::from(angle).cos() as f32;
        let [m00, m01, m10, m11, ..] = self.m;
        self.m[0] = m00 * c + m10 * s;
        self.m[1] = m01 * c + m11 * s;
        self.m[2] = m00 * -s + m10 * c;
        self.m[3] = m01 * -s + m11 * c;
    }
}
