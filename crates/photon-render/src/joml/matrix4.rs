use super::*;

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

    /// `MatrixUtil.checkPropertyRaw(matrix, 16)`: whether the matrix is known to be orthonormal.
    pub fn is_orthonormal(&self) -> bool {
        self.properties & ORTHONORMAL != 0
    }

    /// `MatrixUtil.isPureTranslation`.
    pub fn is_pure_translation(&self) -> bool {
        self.properties & TRANSLATION != 0
    }

    /// Whether the matrix is exactly the identity
    /// (`MatrixUtil.isIdentity`, which compares values).
    pub fn is_identity(&self) -> bool {
        self.properties & IDENTITY != 0 || self.m == Self::identity().m
    }

    /// `Matrix4f.determineProperties` as `MatrixUtil.checkProperty` leaves it on a matrix a
    /// transformation keeps: one equal to the identity (zeros of either sign) is known to be.
    pub fn determine_properties(&mut self) {
        if self.m == Self::identity().m {
            self.properties |= IDENTITY | AFFINE | TRANSLATION | ORTHONORMAL;
        }
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
        if self.properties & TRANSLATION != 0 {
            // `mulTranslationAffine`: the right's rotation as it is, the translations added.
            let mut out = right.m;
            for column in out.iter_mut().take(3) {
                column[3] = l[0][3];
            }
            for row in 0..3 {
                out[3][row] = r[3][row] + l[3][row];
            }
            out[3][3] = l[3][3];
            return Self {
                m: out,
                properties: AFFINE | right.properties & ORTHONORMAL,
            };
        }
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

    /// `Matrix4f.rotationAround(quat, ox, oy, oz)`: the rotation about the point as a matrix of its
    /// own, affine and orthonormal.
    pub fn rotation_around(quat: &Quaternion, (ox, oy, oz): (f32, f32, f32)) -> Self {
        let r = quat.expand();
        let mut m = [[0.0_f32; 4]; 4];
        for (column, rm) in m.iter_mut().zip(r.chunks(3)) {
            column[..3].copy_from_slice(rm);
        }
        for row in 0..3 {
            m[3][row] = -m[0][row] * [ox, oy, oz][0]
                - m[1][row] * [ox, oy, oz][1]
                - m[2][row] * [ox, oy, oz][2]
                + [ox, oy, oz][row];
        }
        m[3][3] = 1.0;
        Self {
            m,
            properties: AFFINE | ORTHONORMAL,
        }
    }

    /// `Matrix4f.rotateAround(quat, ox, oy, oz)` for an affine matrix
    /// (`rotateAroundAffine`): the rotation about the point `(ox, oy, oz)`.
    pub fn rotate_around(&mut self, quat: &Quaternion, (ox, oy, oz): (f32, f32, f32)) {
        if self.properties & IDENTITY != 0 {
            // The identity turned about a point is that turn, as a matrix of its own.
            *self = Self::rotation_around(quat, (ox, oy, oz));
            return;
        }
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
            // The reference reads the matrix it has just written into: the turned third column.
            out[3][row] = -out[0][row] * ox - out[1][row] * oy - out[2][row] * oz + moved;
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

    pub(super) fn set_upper(&mut self, r: [f32; 9]) {
        for column in 0..3 {
            for row in 0..3 {
                self.m[column][row] = r[column * 3 + row];
            }
            self.m[column][3] = 0.0;
        }
    }

    /// `Matrix4f.rotateY(angle)`: turned about the y axis (a sine and a cosine, not a quaternion).
    pub fn rotate_y(&mut self, angle: f32) {
        let sin = sin(angle);
        let cos = cos_from_sin(sin, angle);
        let rotation = |translation: [f32; 3]| {
            let mut matrix = Self::identity();
            matrix.m[0][0] = cos;
            matrix.m[0][2] = -sin;
            matrix.m[2][0] = sin;
            matrix.m[2][2] = cos;
            matrix.m[3][..3].copy_from_slice(&translation);
            matrix.properties = AFFINE | ORTHONORMAL;
            matrix
        };
        if self.properties & IDENTITY != 0 {
            *self = rotation([0.0; 3]);
        } else if self.properties & TRANSLATION != 0 {
            *self = rotation([self.m[3][0], self.m[3][1], self.m[3][2]]);
        } else {
            for row in 0..4 {
                let (m0, m2) = (self.m[0][row], self.m[2][row]);
                self.m[0][row] = fma(m0, cos, m2 * -sin);
                self.m[2][row] = fma(m0, sin, m2 * cos);
            }
            self.properties &= !(PERSPECTIVE | IDENTITY | TRANSLATION);
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
