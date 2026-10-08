use super::*;

/// Four-component f32 value used by Java-compatible quaternion helpers.
#[derive(Debug, Clone, Copy)]
pub struct Quaternionf {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternionf {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Return the JOML `Quaternionf.rotationX(float)` construction used by
    /// Direction's fixed rotations.
    pub(crate) fn rotation_x(angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let sine = joml_sin_f32(half_angle);
        let cosine = joml_cos_from_sin(sine, half_angle);
        Self::new(sine, 0.0, 0.0, cosine)
    }

    /// Return the JOML `Quaternionf.rotationXYZ(float, float, float)`
    /// construction used by Direction's fixed rotations.
    pub(crate) fn rotation_xyz(x_angle: f32, y_angle: f32, z_angle: f32) -> Self {
        let x_half = x_angle * 0.5;
        let x_sin = joml_sin_f32(x_half);
        let x_cos = joml_cos_from_sin(x_sin, x_half);
        let y_half = y_angle * 0.5;
        let y_sin = joml_sin_f32(y_half);
        let y_cos = joml_cos_from_sin(y_sin, y_half);
        let z_half = z_angle * 0.5;
        let z_sin = joml_sin_f32(z_half);
        let z_cos = joml_cos_from_sin(z_sin, z_half);

        let cos_y_cos_z = y_cos * z_cos;
        let sin_y_sin_z = y_sin * z_sin;
        let sin_y_cos_z = y_sin * z_cos;
        let cos_y_sin_z = y_cos * z_sin;
        Self::new(
            x_sin * cos_y_cos_z + x_cos * sin_y_sin_z,
            x_cos * sin_y_cos_z - x_sin * cos_y_sin_z,
            x_cos * cos_y_sin_z + x_sin * sin_y_cos_z,
            x_cos * cos_y_cos_z - x_sin * sin_y_sin_z,
        )
    }

    pub(super) fn normalized(self) -> Self {
        // JOML accumulates the quaternion norm with a right-associated f32
        // FMA chain, then narrows Math.sqrt's f64 result before dividing.
        let length_squared = self.x.mul_add(
            self.x,
            self.y
                .mul_add(self.y, self.z.mul_add(self.z, self.w * self.w)),
        );
        let square_root = f64::from(length_squared).sqrt() as f32;
        let inverse = 1.0_f32 / square_root;
        Self::new(
            self.x * inverse,
            self.y * inverse,
            self.z * inverse,
            self.w * inverse,
        )
    }
}
