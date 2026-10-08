//! Java's `Math` rules for floats where they differ from Rust's: NaN and
//! signed zeros.

/// `Math.signum`.
pub(crate) fn signum(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        value
    }
}

/// `Mth.clamp`: below the range is its minimum, otherwise `Math.min`.
pub(crate) fn clamp(value: f64, low: f64, high: f64) -> f64 {
    if value < low { low } else { min(value, high) }
}

/// `Math.min` for doubles; `-0.0` is less than `0.0`.
pub(crate) fn min(left: f64, right: f64) -> f64 {
    if left.is_nan() || right.is_nan() {
        f64::NAN
    } else if left == 0.0 && right == 0.0 {
        f64::from_bits(left.to_bits() | right.to_bits())
    } else if left <= right {
        left
    } else {
        right
    }
}

/// `Math.max` for doubles; `0.0` is greater than `-0.0`.
pub(crate) fn max(left: f64, right: f64) -> f64 {
    if left.is_nan() || right.is_nan() {
        f64::NAN
    } else if left == 0.0 && right == 0.0 {
        f64::from_bits(left.to_bits() & right.to_bits())
    } else if left >= right {
        left
    } else {
        right
    }
}

/// `Math.min` for floats.
pub(crate) fn min_f32(left: f32, right: f32) -> f32 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == 0.0 && right == 0.0 {
        f32::from_bits(left.to_bits() | right.to_bits())
    } else if left <= right {
        left
    } else {
        right
    }
}
