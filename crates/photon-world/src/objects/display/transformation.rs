use super::*;

/// Which display it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayKind {
    #[default]
    Block,
    Item,
    Text,
}

/// `Display.BillboardConstraints`, by id (an id out of range is the first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Billboard {
    Fixed,
    Vertical,
    Horizontal,
    Center,
}

/// `com.mojang.math.Transformation`, as its four parts (the display's data makes it from them).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformation {
    pub translation: [f32; 3],
    /// `x, y, z, w`.
    pub left_rotation: [f32; 4],
    pub scale: [f32; 3],
    pub right_rotation: [f32; 4],
}

// A transformation is made of finite numbers: never NaN.
impl Eq for Transformation {}

/// `org.joml.Math.fma` as the reference runs it (not fused).
pub(super) fn fma(a: f32, b: f32, c: f32) -> f32 {
    a * b + c
}

/// `Vector3f.lerp`.
pub(super) fn lerp3(from: [f32; 3], to: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|axis| fma(to[axis] - from[axis], t, from[axis]))
}

/// `Quaternionf.slerp`.
pub(super) fn slerp(from: [f32; 4], to: [f32; 4], alpha: f32) -> [f32; 4] {
    let [x, y, z, w] = from;
    let cosom = fma(x, to[0], fma(y, to[1], fma(z, to[2], w * to[3])));
    let abs_cosom = cosom.abs();
    let (scale0, mut scale1);
    if 1.0 - abs_cosom > 1.0e-6 {
        let sin_sqr = 1.0 - abs_cosom * abs_cosom;
        let sinom = 1.0 / sin_sqr.sqrt();
        let omega = f64::from(sin_sqr * sinom).atan2(f64::from(abs_cosom)) as f32;
        scale0 = (((1.0 - f64::from(alpha)) * f64::from(omega)).sin() * f64::from(sinom)) as f32;
        scale1 = f64::from(alpha * omega).sin() as f32 * sinom;
    } else {
        scale0 = 1.0 - alpha;
        scale1 = alpha;
    }
    if cosom < 0.0 {
        scale1 = -scale1;
    }
    [
        fma(scale0, x, scale1 * to[0]),
        fma(scale0, y, scale1 * to[1]),
        fma(scale0, z, scale1 * to[2]),
        fma(scale0, w, scale1 * to[3]),
    ]
}

/// `Display.GenericInterpolator<Transformation>`: a constant, or the last to the new.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformationInterpolator {
    Constant(Transformation),
    Slerp(Transformation, Transformation),
}

/// `Display.FloatInterpolator`: a constant, or the last to the new linearly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatInterpolator {
    Constant(f32),
    Linear(f32, f32),
}

// A float is made of finite numbers: never NaN.
impl Eq for FloatInterpolator {}

/// `Display.IntInterpolator`: a constant, the last to the new in whole numbers (`lerpInt`), or the
/// colours' channels one by one (`ARGB.srgbLerp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntInterpolator {
    Constant(i32),
    Linear(i32, i32),
    Color(i32, i32),
}

impl Billboard {
    pub(super) fn from_id(id: u8) -> Self {
        match id {
            1 => Self::Vertical,
            2 => Self::Horizontal,
            3 => Self::Center,
            _ => Self::Fixed,
        }
    }
}

impl Transformation {
    pub const IDENTITY: Self = Self {
        translation: [0.0; 3],
        left_rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0; 3],
        right_rotation: [0.0, 0.0, 0.0, 1.0],
    };

    /// `Transformation.slerp`: the translation and scale linearly, the turns spherically.
    pub fn slerp(&self, that: &Self, progress: f32) -> Self {
        Self {
            translation: lerp3(self.translation, that.translation, progress),
            left_rotation: slerp(self.left_rotation, that.left_rotation, progress),
            scale: lerp3(self.scale, that.scale, progress),
            right_rotation: slerp(self.right_rotation, that.right_rotation, progress),
        }
    }
}

impl TransformationInterpolator {
    pub fn get(&self, progress: f32) -> Transformation {
        match self {
            Self::Constant(value) => *value,
            Self::Slerp(previous, current) => {
                if f64::from(progress) >= 1.0 {
                    *current
                } else {
                    previous.slerp(current, progress)
                }
            }
        }
    }
}

impl FloatInterpolator {
    pub fn get(&self, progress: f32) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Linear(previous, current) => mth_lerp_f32(progress, *previous, *current),
        }
    }
}

impl IntInterpolator {
    pub fn get(&self, progress: f32) -> i32 {
        match *self {
            Self::Constant(value) => value,
            Self::Linear(previous, current) => mth_lerp_int(progress, previous, current),
            Self::Color(previous, current) => {
                let channel = |shift: u32| {
                    mth_lerp_int(
                        progress,
                        ((previous as u32 >> shift) & 0xFF) as i32,
                        ((current as u32 >> shift) & 0xFF) as i32,
                    ) as u32
                };
                // `ARGB.color(a, r, g, b)`.
                ((channel(24) << 24) | (channel(16) << 16) | (channel(8) << 8) | channel(0)) as i32
            }
        }
    }
}
