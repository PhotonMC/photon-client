use super::*;

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

/// Return Java `Mth.ceilLong(double)` with Java's floating-point-to-long narrowing.
pub fn ceil_long_f64(value: f64) -> i64 {
    value.ceil() as i64
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

/// Return Java `Mth.abs(float)` while preserving raw NaN payload bits.
pub fn mth_abs_f32(value: f32) -> f32 {
    value.abs()
}

/// Return Java `Mth.smallestEncompassingPowerOfTwo(int)` with i32 wrapping.
pub fn smallest_encompassing_power_of_two(input: i32) -> i32 {
    let mut result = input.wrapping_sub(1);
    result |= result >> 1;
    result |= result >> 2;
    result |= result >> 4;
    result |= result >> 8;
    result |= result >> 16;
    result.wrapping_add(1)
}

/// Return Java `Mth.smallestSquareSide(int)` for a valid nonnegative count.
pub fn smallest_square_side(item_count: i32) -> Result<i32, String> {
    if item_count < 0 {
        Err("itemCount must be greater than or equal to zero".into())
    } else {
        Ok(f64::from(item_count).sqrt().ceil() as i32)
    }
}

/// Return Java `Mth.isPowerOfTwo(int)` using the input's wrapped bit pattern.
pub fn is_power_of_two_i32(input: i32) -> bool {
    input != 0 && (input & input.wrapping_sub(1)) == 0
}

/// Return Java `Mth.isPowerOfTwo(long)` using the input's wrapped bit pattern.
pub fn is_power_of_two_i64(input: i64) -> bool {
    input != 0 && (input & input.wrapping_sub(1)) == 0
}

/// Return Java `Mth.ceillog2(int)` with its De Bruijn lookup.
pub fn ceillog2(input: i32) -> i32 {
    let input = if is_power_of_two_i32(input) {
        input
    } else {
        smallest_encompassing_power_of_two(input)
    };
    let index = (((i64::from(input) * 125_613_361_i64) >> 27) as i32) & 31;
    MULTIPLY_DE_BRUIJN_BIT_POSITION[index as usize]
}

/// Return Java `Mth.log2(int)`, including its zero/non-power correction.
pub fn log2(input: i32) -> i32 {
    ceillog2(input) - i32::from(!is_power_of_two_i32(input))
}

/// Return Java `Mth.positiveModulo(float)` without changing operation order.
pub fn positive_modulo_f32(input: f32, modulus: f32) -> f32 {
    let result = (input % modulus + modulus) % modulus;
    if result.is_nan() {
        if input.is_finite() && modulus.is_infinite() {
            f32::from_bits(0xffc0_0000)
        } else {
            f32::from_bits(0x7fc0_0000)
        }
    } else {
        result
    }
}

/// Return Java `Mth.positiveModulo(double)` without changing operation order.
pub fn positive_modulo_f64(input: f64, modulus: f64) -> f64 {
    let result = (input % modulus + modulus) % modulus;
    if result.is_nan() {
        if input.is_finite() && modulus.is_infinite() {
            f64::from_bits(0xfff8_0000_0000_0000)
        } else {
            f64::from_bits(0x7ff8_0000_0000_0000)
        }
    } else {
        result
    }
}

/// Return Java `Mth.absMax(float)` with Java NaN and absolute-value bits.
pub fn abs_max_f32(first: f32, second: f32) -> f32 {
    let first = f32::from_bits(first.to_bits() & 0x7fff_ffff);
    let second = f32::from_bits(second.to_bits() & 0x7fff_ffff);
    if first.is_nan() {
        first
    } else if second.is_nan() {
        second
    } else {
        first.max(second)
    }
}

/// Return Java `Mth.absMax(double)` with Java NaN and absolute-value bits.
pub fn abs_max_f64(first: f64, second: f64) -> f64 {
    let first = f64::from_bits(first.to_bits() & 0x7fff_ffff_ffff_ffff);
    let second = f64::from_bits(second.to_bits() & 0x7fff_ffff_ffff_ffff);
    if first.is_nan() {
        first
    } else if second.is_nan() {
        second
    } else {
        first.max(second)
    }
}

/// Return Java `Mth.equal(float, float)` with its strict f32 threshold.
pub fn equal_f32(first: f32, second: f32) -> bool {
    (second - first).abs() < 1.0e-5_f32
}

/// Return Java `Mth.equal(double, double)` with its promoted f32 threshold.
pub fn equal_f64(first: f64, second: f64) -> bool {
    (second - first).abs() < f64::from(1.0e-5_f32)
}

/// Return Java `Mth.clampedLerp(float, float, float)` in source order.
pub fn clamped_lerp_f32(factor: f32, minimum: f32, maximum: f32) -> f32 {
    if factor < 0.0 {
        minimum
    } else if factor > 1.0 {
        maximum
    } else {
        minimum + factor * (maximum - minimum)
    }
}

/// Return Java `Mth.clampedLerp(double, double, double)` in source order.
pub fn clamped_lerp_f64(factor: f64, minimum: f64, maximum: f64) -> f64 {
    if factor < 0.0 {
        minimum
    } else if factor > 1.0 {
        maximum
    } else {
        minimum + factor * (maximum - minimum)
    }
}

/// Return Java `Mth.inverseLerp(float, float, float)` in source order.
pub fn mth_inverse_lerp_f32(value: f32, minimum: f32, maximum: f32) -> f32 {
    let numerator = value - minimum;
    let denominator = maximum - minimum;
    numerator / denominator
}

/// Return Java `Mth.inverseLerp(double, double, double)` in source order.
pub fn mth_inverse_lerp_f64(value: f64, minimum: f64, maximum: f64) -> f64 {
    let numerator = value - minimum;
    let denominator = maximum - minimum;
    numerator / denominator
}

/// Return Java `Mth.clampedMap(double, double, double, double, double)` in source order.
pub fn mth_clamped_map_f64(
    value: f64,
    from_minimum: f64,
    from_maximum: f64,
    to_minimum: f64,
    to_maximum: f64,
) -> f64 {
    let factor = mth_inverse_lerp_f64(value, from_minimum, from_maximum);
    clamped_lerp_f64(factor, to_minimum, to_maximum)
}

/// Return Java `Mth.clampedMap(float, float, float, float, float)` in source order.
pub fn mth_clamped_map_f32(
    value: f32,
    from_minimum: f32,
    from_maximum: f32,
    to_minimum: f32,
    to_maximum: f32,
) -> f32 {
    let factor = mth_inverse_lerp_f32(value, from_minimum, from_maximum);
    clamped_lerp_f32(factor, to_minimum, to_maximum)
}

/// Return Java `Mth.map(double, double, double, double, double)` in source order.
pub fn mth_map_f64(
    value: f64,
    from_minimum: f64,
    from_maximum: f64,
    to_minimum: f64,
    to_maximum: f64,
) -> f64 {
    let factor = mth_inverse_lerp_f64(value, from_minimum, from_maximum);
    mth_lerp_f64(factor, to_minimum, to_maximum)
}

/// Return Java `Mth.map(float, float, float, float, float)` in source order.
pub fn mth_map_f32(
    value: f32,
    from_minimum: f32,
    from_maximum: f32,
    to_minimum: f32,
    to_maximum: f32,
) -> f32 {
    let factor = mth_inverse_lerp_f32(value, from_minimum, from_maximum);
    mth_lerp_f32(factor, to_minimum, to_maximum)
}

/// Return Java `Mth.wobble(double)` with its deterministic floor-derived seed.
pub fn mth_wobble_f64(coord: f64) -> f64 {
    let seed = floor_f64(coord * 3000.0);
    let mut random = RandomSource::create(i64::from(seed));
    coord + (2.0 * random.next_double() - 1.0) * 1.0E-7 / 2.0
}

/// Return Java `Mth.lerp2(double, double, double, double, double, double)`.
pub fn mth_lerp2_f64(alpha1: f64, alpha2: f64, x00: f64, x10: f64, x01: f64, x11: f64) -> f64 {
    let lower = mth_lerp_f64(alpha1, x00, x10);
    let upper = mth_lerp_f64(alpha1, x01, x11);
    mth_lerp_f64(alpha2, lower, upper)
}

/// Return Java `Mth.lerp3(double, ...)` with its two plane calls in source order.
#[expect(
    clippy::too_many_arguments,
    reason = "the reference method's parameters, in its order"
)]
pub fn mth_lerp3_f64(
    alpha1: f64,
    alpha2: f64,
    alpha3: f64,
    x000: f64,
    x100: f64,
    x010: f64,
    x110: f64,
    x001: f64,
    x101: f64,
    x011: f64,
    x111: f64,
) -> f64 {
    let lower = mth_lerp2_f64(alpha1, alpha2, x000, x100, x010, x110);
    let upper = mth_lerp2_f64(alpha1, alpha2, x001, x101, x011, x111);
    mth_lerp_f64(alpha3, lower, upper)
}

/// Return Java `Mth.catmullrom(float, float, float, float, float)`.
pub fn mth_catmullrom_f32(alpha: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let linear = (p2 - p0) * alpha;
    let quadratic_coefficient = 2.0_f32 * p0 - 5.0_f32 * p1 + 4.0_f32 * p2 - p3;
    let quadratic = quadratic_coefficient * alpha * alpha;
    let cubic_coefficient = 3.0_f32 * p1 - p0 - 3.0_f32 * p2 + p3;
    let cubic = cubic_coefficient * alpha * alpha * alpha;
    0.5_f32 * (2.0_f32 * p1 + linear + quadratic + cubic)
}

/// Return Java `Mth.lerp(float, float, float)` in source order.
pub fn mth_lerp_f32(alpha: f32, start: f32, end: f32) -> f32 {
    start + alpha * (end - start)
}

/// Return Java `Mth.lerp(double, double, double)` in source order.
pub fn mth_lerp_f64(alpha: f64, start: f64, end: f64) -> f64 {
    start + alpha * (end - start)
}

/// Return Java `Mth.lerpInt(float, int, int)` with i32 wrapping arithmetic.
pub fn mth_lerp_int(alpha: f32, start: i32, end: i32) -> i32 {
    let delta = end.wrapping_sub(start);
    start.wrapping_add(floor_f32(alpha * delta as f32))
}

/// Return Java `Mth.lerpDiscrete(float, int, int)` with i32 wrapping arithmetic.
pub fn mth_lerp_discrete(alpha: f32, start: i32, end: i32) -> i32 {
    let delta = end.wrapping_sub(start);
    let offset = floor_f32(alpha * delta.wrapping_sub(1) as f32);
    let positive_alpha = if alpha > 0.0 { 1 } else { 0 };
    start.wrapping_add(offset).wrapping_add(positive_alpha)
}

/// Return Java `Mth.smoothstep(double)` in source order.
pub fn mth_smoothstep(x: f64) -> f64 {
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

/// Return Java `Mth.smoothstepDerivative(double)` in source order.
pub fn mth_smoothstep_derivative(x: f64) -> f64 {
    30.0 * x * x * (x - 1.0) * (x - 1.0)
}

/// Return Java `Mth.sign(double)` with its NaN branch preserved.
pub fn mth_sign(number: f64) -> i32 {
    if number == 0.0 {
        0
    } else if number > 0.0 {
        1
    } else {
        -1
    }
}

/// Return Java `Mth.rotLerp(float, float, float)` in source order.
pub fn mth_rot_lerp_f32(alpha: f32, from: f32, to: f32) -> f32 {
    from + alpha * wrap_degrees_f32(to - from)
}

/// Return Java `Projectile.lerpRotation(float, float)` in source order.
pub fn projectile_lerp_rotation_f32(from: f32, to: f32) -> f32 {
    let mut from = from;
    while to - from < -180.0_f32 {
        from -= 360.0_f32;
    }
    while to - from >= 180.0_f32 {
        from += 360.0_f32;
    }
    mth_lerp_f32(0.2_f32, from, to)
}

/// Return Java `Mth.rotLerp(double, double, double)` in source order.
pub fn mth_rot_lerp_f64(alpha: f64, from: f64, to: f64) -> f64 {
    from + alpha * wrap_degrees_f64(to - from)
}

/// Return Java `Mth.rotLerpRad(float, float, float)` with float loop constants.
pub fn mth_rot_lerp_rad(alpha: f32, from: f32, to: f32) -> f32 {
    let mut diff = to - from;
    let pi = f32::from_bits(0x4049_0fdb);
    let two_pi = f32::from_bits(0x40c9_0fdb);

    while diff < -pi {
        diff += two_pi;
    }
    while diff >= pi {
        diff -= two_pi;
    }

    from + alpha * diff
}

/// Return Java `Mth.triangleWave(float, float)` in source order.
pub fn mth_triangle_wave(index: f32, period: f32) -> f32 {
    ((index % period - period * 0.5_f32).abs() - period * 0.25_f32) / (period * 0.25_f32)
}

/// Return Java `Mth.square(float)` in source order.
pub fn mth_square_f32(x: f32) -> f32 {
    x * x
}

/// Return Java `Mth.cube(float)` in source order.
pub fn mth_cube_f32(x: f32) -> f32 {
    x * x * x
}

/// Return Java `Mth.square(double)` in source order.
pub fn mth_square_f64(x: f64) -> f64 {
    x * x
}

/// Return Java `Mth.square(int)` with Java's wrapping i32 multiplication.
pub fn mth_square_i32(x: i32) -> i32 {
    x.wrapping_mul(x)
}

/// Return Java `Mth.square(long)` with Java's wrapping i64 multiplication.
pub fn mth_square_i64(x: i64) -> i64 {
    x.wrapping_mul(x)
}
