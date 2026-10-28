//! One step of a minecart along its track, as the server sends it.

/// `NewMinecartBehavior.MinecartStep`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinecartStep {
    pub position: (f64, f64, f64),
    pub movement: (f64, f64, f64),
    pub y_rot: f32,
    pub x_rot: f32,
    pub weight: f32,
}

// A step is made of finite numbers: never NaN.
impl Eq for MinecartStep {}

impl MinecartStep {
    /// `MinecartStep.ZERO`.
    pub const ZERO: Self = Self {
        position: (0.0, 0.0, 0.0),
        movement: (0.0, 0.0, 0.0),
        y_rot: 0.0,
        x_rot: 0.0,
        weight: 0.0,
    };
}
