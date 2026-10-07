use super::*;

pub(super) fn java_asin(value: f64) -> f64 {
    if value.abs() > 1.0 {
        // Java Math.asin returns the negative canonical NaN for every
        // out-of-domain finite or infinite argument. NaN inputs themselves
        // retain their payload and sign through the platform implementation.
        f64::from_bits(0xfff8_0000_0000_0000)
    } else {
        value.asin()
    }
}

pub(super) fn mth_sin(value: f64) -> f32 {
    mth_sin_table_value(mth_lookup_index(value * MTH_SIN_SCALE))
}

pub(super) fn mth_cos(value: f64) -> f32 {
    mth_sin_table_value(mth_lookup_index(value * MTH_SIN_SCALE + 16384.0))
}

pub(super) fn mth_lookup_index(scaled: f64) -> usize {
    // Java's double-to-long narrowing conversion maps NaN to zero and clamps
    // out-of-range values; Rust's float-to-int cast has the same contract.
    ((scaled as i64) as u64 & 0xffff) as usize
}

pub(super) fn mth_sin_table_value(index: usize) -> f32 {
    (index as f64 / MTH_SIN_SCALE).sin() as f32
}

/// Return Java `Mth.atan2(double, double)` with its table approximation.
pub fn mth_atan2(mut y: f64, mut x: f64) -> f64 {
    let squared_length = x * x + y * y;
    if squared_length.is_nan() {
        return f64::NAN;
    }

    let negative_y = y < 0.0;
    if negative_y {
        y = -y;
    }

    let negative_x = x < 0.0;
    if negative_x {
        x = -x;
    }

    let steep = y > x;
    if steep {
        std::mem::swap(&mut x, &mut y);
    }

    let inverse_length = mth_fast_inv_sqrt_f64(squared_length);
    x *= inverse_length;
    y *= inverse_length;

    let fractional_bias = f64::from_bits(0x42b0_0000_0000_0000);
    let y_with_bias = fractional_bias + y;
    let index = (y_with_bias.to_bits() as u32) as usize;
    let (phi, cosine_phi) = mth_atan2_table_entry(index);
    let sine_phi = y_with_bias - fractional_bias;
    let sine_difference = y * cosine_phi - x * sine_phi;
    let correction =
        (6.0 + sine_difference * sine_difference) * sine_difference * 0.16666666666666666;
    let mut theta = phi + correction;
    if steep {
        theta = (std::f64::consts::PI / 2.0) - theta;
    }
    if negative_x {
        theta = std::f64::consts::PI - theta;
    }
    if negative_y {
        theta = -theta;
    }
    theta
}

pub(super) fn mth_atan2_table_entry(index: usize) -> (f64, f64) {
    let value = index as f64 / 256.0;
    let asin = value.asin();
    (asin, asin.cos())
}

/// Return JOML-backed Java `Mth.invSqrt(float)` with its f64 sqrt and f32 divide.
pub fn mth_inv_sqrt_f32(value: f32) -> f32 {
    let square_root = f64::from(value).sqrt() as f32;
    1.0_f32 / square_root
}

/// Return JOML-backed Java `Mth.invSqrt(double)`.
pub fn mth_inv_sqrt_f64(value: f64) -> f64 {
    1.0 / value.sqrt()
}

/// Return Java `Mth.sin(double)` from its quantized f32 lookup table.
pub fn mth_sin_f64(value: f64) -> f32 {
    mth_sin(value)
}

/// Return Java `Mth.cos(double)` from its quantized f32 lookup table.
pub fn mth_cos_f64(value: f64) -> f32 {
    mth_cos(value)
}

/// Return Java `Mth.sqrt(float)`, including the f64 math and f32 narrowing.
pub fn mth_sqrt_f32(value: f32) -> f32 {
    f64::from(value).sqrt() as f32
}

pub(super) fn joml_sin_f32(value: f32) -> f32 {
    f64::from(value).sin() as f32
}

pub(super) fn joml_cos_from_sin(sine: f32, angle: f32) -> f32 {
    const JOML_HALF_PI: f32 = f32::from_bits(0x3fc9_0fdb);
    const JOML_TWO_PI: f32 = f32::from_bits(0x40c9_0fdb);
    const JOML_PI: f32 = f32::from_bits(0x4049_0fdb);

    let magnitude = f64::from(1.0_f32 - sine * sine).sqrt() as f32;
    let mut quadrant = angle + JOML_HALF_PI;
    let turns = (quadrant / JOML_TWO_PI) as i32;
    quadrant -= turns as f32 * JOML_TWO_PI;
    if quadrant < 0.0 {
        quadrant += JOML_TWO_PI;
    }
    if quadrant >= JOML_PI {
        -magnitude
    } else {
        magnitude
    }
}

/// Return Java `Mth.fastInvSqrt(double)` with the raw-bit seed and one correction.
pub fn mth_fast_inv_sqrt_f64(mut value: f64) -> f64 {
    let half = 0.5 * value;
    let bits = (value.to_bits() as i64) >> 1;
    let bits = 6_910_469_410_427_058_090_i64.wrapping_sub(bits);
    value = f64::from_bits(bits as u64);
    value * (1.5 - half * value * value)
}

/// Return Java `Mth.fastInvCubeRoot(float)` with two f32 corrections.
pub fn mth_fast_inv_cube_root_f32(value: f32) -> f32 {
    let bits = if value.is_nan() {
        0x7fc0_0000
    } else {
        value.to_bits()
    };
    let seed = 1_419_967_116_i32.wrapping_sub((bits as i32) / 3);
    let mut estimate = f32::from_bits(seed as u32);
    estimate = 0.6666667_f32 * estimate + 1.0_f32 / (3.0_f32 * estimate * estimate * value);
    0.6666667_f32 * estimate + 1.0_f32 / (3.0_f32 * estimate * estimate * value)
}
