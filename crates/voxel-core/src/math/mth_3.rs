use super::*;

/// Return Java `Mth.nextInt(RandomSource, int, int)`, including its
/// min-greater-than-or-equal-to-max short circuit.
pub fn mth_next_int(
    random: &mut RandomSource,
    minimum: i32,
    maximum: i32,
) -> Result<i32, crate::random::RandomError> {
    if minimum >= maximum {
        Ok(minimum)
    } else {
        let bound = maximum.wrapping_sub(minimum).wrapping_add(1);
        random
            .next_int_bound(bound)
            .map(|value| value.wrapping_add(minimum))
    }
}

/// Return Java `Mth.nextFloat(RandomSource, float, float)` in source order.
pub fn mth_next_float(random: &mut RandomSource, minimum: f32, maximum: f32) -> f32 {
    if minimum >= maximum {
        minimum
    } else {
        random.next_float() * (maximum - minimum) + minimum
    }
}

/// Return Java `Mth.nextDouble(RandomSource, double, double)` in source order.
pub fn mth_next_double(random: &mut RandomSource, minimum: f64, maximum: f64) -> f64 {
    if minimum >= maximum {
        minimum
    } else {
        random.next_double() * (maximum - minimum) + minimum
    }
}

/// Return Java `Mth.randomBetweenInclusive(RandomSource, int, int)`.
pub fn mth_random_between_inclusive(
    random: &mut RandomSource,
    minimum: i32,
    maximum: i32,
) -> Result<i32, crate::random::RandomError> {
    let bound = maximum.wrapping_sub(minimum).wrapping_add(1);
    random
        .next_int_bound(bound)
        .map(|value| value.wrapping_add(minimum))
}

/// Return Java `Mth.randomBetween(RandomSource, float, float)` in source order.
pub fn mth_random_between(random: &mut RandomSource, minimum: f32, maximum: f32) -> f32 {
    random.next_float() * (maximum - minimum) + minimum
}

/// Return Java `Mth.normal(RandomSource, float, float)` in source order.
pub fn mth_normal(random: &mut RandomSource, mean: f32, deviation: f32) -> f32 {
    mean + random.next_gaussian() as f32 * deviation
}

/// Return Java `Mth.lengthSquared(double, double)` in source order.
pub fn mth_length_squared_f64_2(x: f64, y: f64) -> f64 {
    let x_squared = x * x;
    let y_squared = y * y;
    x_squared + y_squared
}

/// Return Java `Mth.length(double, double)` in source order.
pub fn mth_length_f64_2(x: f64, y: f64) -> f64 {
    mth_length_squared_f64_2(x, y).sqrt()
}

/// Return Java `Mth.length(float, float)`, including its double helper call.
pub fn mth_length_f32_2(x: f32, y: f32) -> f32 {
    mth_length_squared_f64_2(f64::from(x), f64::from(y)).sqrt() as f32
}

/// Return Java `Mth.lengthSquared(double, double, double)` in source order.
pub fn mth_length_squared_f64_3(x: f64, y: f64, z: f64) -> f64 {
    let x_squared = x * x;
    let y_squared = y * y;
    let z_squared = z * z;
    x_squared + y_squared + z_squared
}

/// Return Java `Mth.length(double, double, double)` in source order.
pub fn mth_length_f64_3(x: f64, y: f64, z: f64) -> f64 {
    mth_length_squared_f64_3(x, y, z).sqrt()
}

/// Return Java `Mth.lengthSquared(float, float, float)` with f32 rounding.
pub fn mth_length_squared_f32_3(x: f32, y: f32, z: f32) -> f32 {
    java_f32_add(java_f32_add(x * x, y * y), z * z)
}

/// Return Java `Mth.quantize(double, int)`, including Java i32 overflow.
pub fn mth_quantize(value: f64, resolution: i32) -> i32 {
    let floored = (value / f64::from(resolution)).floor() as i32;
    floored.wrapping_mul(resolution)
}

/// Return Java `Mth.outFromOrigin(int, int, int, int)` as an ordered list.
pub fn mth_out_from_origin(
    origin: i32,
    lower_bound: i32,
    upper_bound: i32,
    step_size: i32,
) -> Result<Vec<i32>, String> {
    if lower_bound > upper_bound {
        return Err(format!(
            "upperBound {upper_bound} expected to be > lowerBound {lower_bound}"
        ));
    }
    if step_size < 1 {
        return Err(format!("step size expected to be >= 1, was {step_size}"));
    }

    let clamped_origin = clamp_i32(origin, lower_bound, upper_bound);
    let mut cursor = clamped_origin;
    let mut values = Vec::new();
    loop {
        let current_distance = clamped_origin.wrapping_sub(cursor).wrapping_abs();
        let has_next = clamped_origin.wrapping_sub(current_distance) >= lower_bound
            || clamped_origin.wrapping_add(current_distance) <= upper_bound;
        if !has_next {
            break;
        }
        values.push(cursor);

        let previous_was_negative = cursor <= clamped_origin;
        let can_move_positive = clamped_origin
            .wrapping_add(current_distance)
            .wrapping_add(step_size)
            <= upper_bound;
        if !previous_was_negative || !can_move_positive {
            let attempted_step = clamped_origin
                .wrapping_sub(current_distance)
                .wrapping_sub(if previous_was_negative { step_size } else { 0 });
            if attempted_step >= lower_bound {
                cursor = attempted_step;
                continue;
            }
        }

        cursor = clamped_origin
            .wrapping_add(current_distance)
            .wrapping_add(step_size);
    }
    Ok(values)
}

/// Return Java `Mth.mulAndTruncate(Fraction, int)` for a Fraction's fields.
pub fn mth_mul_and_truncate(numerator: i32, denominator: i32, factor: i32) -> i32 {
    let product = numerator.wrapping_mul(factor);
    if product == i32::MIN && denominator == -1 {
        i32::MIN
    } else {
        product / denominator
    }
}

/// Return Java `Mth.rotationAroundAxis(Vector3fc, Quaternionf, Quaternionf)`.
pub fn mth_rotation_around_axis(axis: Vec3f, rotation: Quaternionf) -> Quaternionf {
    let projected_length = axis
        .x
        .mul_add(rotation.x, axis.y.mul_add(rotation.y, axis.z * rotation.z));
    Quaternionf::new(
        axis.x * projected_length,
        axis.y * projected_length,
        axis.z * projected_length,
        rotation.w,
    )
    .normalized()
}

/// Return Java `Math.floorDiv(int, int)`, including Java's signed overflow.
pub fn floor_div_i32(dividend: i32, divisor: i32) -> i32 {
    if divisor == 0 {
        panic!("division by zero");
    }
    // Java's integer division defines MIN_VALUE / -1 as MIN_VALUE; Rust's
    // checked debug division would otherwise panic here.
    if dividend == i32::MIN && divisor == -1 {
        return i32::MIN;
    }
    let quotient = dividend / divisor;
    let remainder = dividend % divisor;
    if remainder != 0 && (dividend < 0) != (divisor < 0) {
        quotient.wrapping_sub(1)
    } else {
        quotient
    }
}

/// Return Java `Math.floorMod(int, int)` as used by `Mth.positiveModulo`.
pub fn positive_modulo_i32(input: i32, modulus: i32) -> i32 {
    let quotient = floor_div_i32(input, modulus);
    input.wrapping_sub(quotient.wrapping_mul(modulus))
}

/// Return `Mth.positiveCeilDiv(int, int)` with Java negation semantics.
pub fn positive_ceil_div_i32(input: i32, divisor: i32) -> i32 {
    floor_div_i32(input.wrapping_neg(), divisor).wrapping_neg()
}

/// Return `Mth.positiveCeilDiv(long, long)` with Java negation semantics.
pub fn positive_ceil_div_i64(input: i64, divisor: i64) -> i64 {
    floor_div_i64(input.wrapping_neg(), divisor).wrapping_neg()
}

/// Return `Mth.roundToward(int, int)` with Java multiplication semantics.
pub fn round_toward_i32(input: i32, multiple: i32) -> i32 {
    positive_ceil_div_i32(input, multiple).wrapping_mul(multiple)
}

/// Return `Mth.roundToward(long, long)` with Java multiplication semantics.
pub fn round_toward_i64(input: i64, multiple: i64) -> i64 {
    positive_ceil_div_i64(input, multiple).wrapping_mul(multiple)
}

pub(super) fn floor_div_i64(dividend: i64, divisor: i64) -> i64 {
    if divisor == 0 {
        panic!("division by zero");
    }
    if dividend == i64::MIN && divisor == -1 {
        return i64::MIN;
    }
    let quotient = dividend / divisor;
    let remainder = dividend % divisor;
    if remainder != 0 && (dividend < 0) != (divisor < 0) {
        quotient.wrapping_sub(1)
    } else {
        quotient
    }
}

#[inline(never)]
pub(crate) fn java_negate(value: f64) -> f64 {
    f64::from_bits(value.to_bits() ^ (1_u64 << 63))
}

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

pub(super) fn java_f32_add(left: f32, right: f32) -> f32 {
    if right.is_nan() {
        right
    } else if left.is_nan() {
        left
    } else {
        left + right
    }
}

pub(crate) fn java_double_compare(left: f64, right: f64) -> i32 {
    if left < right {
        -1
    } else if left > right {
        1
    } else {
        let left_bits = java_double_to_long_bits(left) as i64;
        let right_bits = java_double_to_long_bits(right) as i64;
        if left_bits < right_bits {
            -1
        } else if left_bits > right_bits {
            1
        } else {
            0
        }
    }
}

pub(super) fn java_double_to_long_bits(value: f64) -> u64 {
    if value.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        value.to_bits()
    }
}

pub(crate) fn java_double_hash(value: f64) -> i32 {
    let bits = java_double_to_long_bits(value);
    (bits ^ (bits >> 32)) as i32
}

pub(super) fn java_float_hash(value: f32) -> i32 {
    let bits = if value.is_nan() {
        0x7fc0_0000_u32
    } else {
        value.to_bits()
    };
    bits as i32
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

pub(crate) fn java_double_to_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f64::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_owned();
    }

    let raw = format!("{value:?}");
    if value == 0.0 || (value.abs() >= 1.0e-3 && value.abs() < 1.0e7) {
        return raw;
    }

    if let Some((mantissa, exponent)) = raw.split_once('e') {
        return java_scientific_string(
            mantissa,
            exponent.parse().expect("Rust exponent is numeric"),
        );
    }
    java_decimal_to_scientific(&raw)
}

pub(super) fn java_scientific_string(mantissa: &str, exponent: i32) -> String {
    let mantissa = if mantissa.contains('.') {
        mantissa.to_owned()
    } else {
        format!("{mantissa}.0")
    };
    format!("{mantissa}E{exponent}")
}

pub(super) fn java_decimal_to_scientific(raw: &str) -> String {
    let (sign, magnitude) = match raw.strip_prefix('-') {
        Some(magnitude) => ("-", magnitude),
        None => ("", raw),
    };
    let decimal = magnitude.find('.').unwrap_or(magnitude.len());
    let digits = magnitude.replace('.', "");
    let digits = digits.trim_end_matches('0');
    let first = digits
        .bytes()
        .position(|digit| digit != b'0')
        .expect("nonzero finite value has a significant digit");
    let exponent = decimal as i32 - first as i32 - 1;
    let mut mantissa = String::new();
    mantissa.push(digits.as_bytes()[first] as char);
    mantissa.push('.');
    if first + 1 < digits.len() {
        mantissa.push_str(&digits[first + 1..]);
    } else {
        mantissa.push('0');
    }
    format!("{sign}{mantissa}E{exponent}")
}
