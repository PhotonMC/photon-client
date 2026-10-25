//! The numbers of a stack the `damage` and `count` range properties and the `damaged` condition
//! read.

/// The durability and the size of a stack as the item models see them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StackNumbers {
    /// `ItemStack.isDamaged`.
    pub damaged: bool,
    /// `ItemStack.getDamageValue`.
    pub damage: i32,
    /// `ItemStack.getMaxDamage`.
    pub max_damage: i32,
    /// `ItemStack.getCount`.
    pub count: i32,
    /// `ItemStack.getMaxStackSize`.
    pub max_count: i32,
}

/// The suffix of a `damage` or `count` property that is not normalized.
pub const RAW: &str = "/raw";

/// `Mth.clamp` on floats: a NaN passes through (`f32::min` would drop it).
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// `value / limit` clamped to a share, or the value clamped to the limit.
fn scaled(value: i32, limit: i32, normalize: bool) -> f32 {
    let (value, limit) = (value as f32, limit as f32);
    if normalize {
        clamp(value / limit, 0.0, 1.0)
    } else {
        clamp(value, 0.0, limit)
    }
}

impl StackNumbers {
    /// The `damage` property (`Damage.get`).
    pub fn damage_property(&self, normalize: bool) -> f32 {
        scaled(self.damage, self.max_damage, normalize)
    }

    /// The `count` property (`Count.get`).
    pub fn count_property(&self, normalize: bool) -> f32 {
        scaled(self.count, self.max_count, normalize)
    }
}
