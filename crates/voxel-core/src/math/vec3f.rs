/// Three-component f32 value returned by the Java-compatible Vec3 conversion.
#[derive(Debug, Clone, Copy)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Java `Mth` public unit-axis constants (`Vector3fc` values).
pub const MTH_Y_AXIS: Vec3f = Vec3f::new(0.0, 1.0, 0.0);

pub const MTH_X_AXIS: Vec3f = Vec3f::new(1.0, 0.0, 0.0);

pub const MTH_Z_AXIS: Vec3f = Vec3f::new(0.0, 0.0, 1.0);

impl Vec3f {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}
