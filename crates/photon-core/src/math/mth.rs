use super::*;

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

/// Return Java `Mth.ceilLong(double)` with Java's floating-point-to-long narrowing.
pub fn ceil_long_f64(value: f64) -> i64 {
    value.ceil() as i64
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

/// Block-to-section coordinate conversion, flooring toward negative infinity.
pub fn block_to_section_coord(block_coord: i32) -> i32 {
    block_coord >> 4
}

/// Section-to-block coordinate conversion with Java int wrapping.
pub fn section_to_block_coord(section_coord: i32) -> i32 {
    section_coord << 4
}

/// Position inside its section, always in `0..16`.
pub fn section_relative(block_coord: i32) -> i32 {
    block_coord & 15
}

/// Return Java `Math.max(Math.abs(a), Math.abs(b))` for signed i32 values.
pub fn abs_max(a: i32, b: i32) -> i32 {
    a.wrapping_abs().max(b.wrapping_abs())
}

/// Return the Java `Mth.chessboardDistance` result with i32 wrapping.
pub fn chessboard_distance(x0: i32, z0: i32, x1: i32, z1: i32) -> i32 {
    abs_max(x1.wrapping_sub(x0), z1.wrapping_sub(z0))
}

/// Return Java `Mth.abs(int)`, including `Integer.MIN_VALUE` wrapping.
pub fn abs_i32(value: i32) -> i32 {
    value.wrapping_abs()
}

/// Return Java `Mth.clamp(int, int, int)`.
pub fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

/// Return Java `Mth.clamp(long, long, long)`.
pub fn clamp_i64(value: i64, min: i64, max: i64) -> i64 {
    value.max(min).min(max)
}

/// Return Java `Mth.isMultipleOf(int, int)`.
pub fn is_multiple_of_i32(dividend: i32, divisor: i32) -> bool {
    if divisor == 0 {
        panic!("division by zero");
    }
    if dividend == i32::MIN && divisor == -1 {
        return true;
    }
    dividend % divisor == 0
}

/// Return Java `Mth.ceil(float)` with Java's floating-point-to-int narrowing.
pub fn ceil_f32(value: f32) -> i32 {
    f64::from(value).ceil() as i32
}

/// Return Java `Mth.ceil(double)` with Java's floating-point-to-int narrowing.
pub fn ceil_f64(value: f64) -> i32 {
    value.ceil() as i32
}

/// Return Java `Mth.lfloor(double)` with Java's floating-point-to-long narrowing.
pub fn lfloor_f64(value: f64) -> i64 {
    value.floor() as i64
}

/// Return Java `Mth.frac(float)` with its floor result narrowed to f32.
pub fn frac_f32(value: f32) -> f32 {
    value - floor_f32(value) as f32
}

/// Return Java `Mth.frac(double)` with its lfloor result widened to f64.
pub fn frac_f64(value: f64) -> f64 {
    value - lfloor_f64(value) as f64
}

/// Return Java `Mth.binarySearch(int, int, IntPredicate)` with i32 wrapping steps.
pub fn mth_binary_search<F>(mut from: i32, to: i32, condition: F) -> i32
where
    F: Fn(i32) -> bool,
{
    let mut len = to.wrapping_sub(from);
    while len > 0 {
        let half = len / 2;
        let middle = from.wrapping_add(half);
        if condition(middle) {
            len = half;
        } else {
            from = middle.wrapping_add(1);
            len = len.wrapping_sub(half.wrapping_add(1));
        }
    }
    from
}

/// Return Java `Mth.rayIntersectsAABB(Vec3, Vec3, AABB)` in source order.
pub fn mth_ray_intersects_aabb(ray_start: Vec3, ray_dir: Vec3, aabb: Aabb) -> bool {
    let center_x = (aabb.min_x + aabb.max_x) * 0.5;
    let box_extent_x = (aabb.max_x - aabb.min_x) * 0.5;
    let diff_x = ray_start.x - center_x;
    if diff_x.abs() > box_extent_x && diff_x * ray_dir.x >= 0.0 {
        return false;
    }

    let center_y = (aabb.min_y + aabb.max_y) * 0.5;
    let box_extent_y = (aabb.max_y - aabb.min_y) * 0.5;
    let diff_y = ray_start.y - center_y;
    if diff_y.abs() > box_extent_y && diff_y * ray_dir.y >= 0.0 {
        return false;
    }

    let center_z = (aabb.min_z + aabb.max_z) * 0.5;
    let box_extent_z = (aabb.max_z - aabb.min_z) * 0.5;
    let diff_z = ray_start.z - center_z;
    if diff_z.abs() > box_extent_z && diff_z * ray_dir.z >= 0.0 {
        return false;
    }

    let andrew_woo_diff_x = ray_dir.x.abs();
    let andrew_woo_diff_y = ray_dir.y.abs();
    let andrew_woo_diff_z = ray_dir.z.abs();
    let mut f = ray_dir.y * diff_z - ray_dir.z * diff_y;
    if f.abs() > box_extent_y * andrew_woo_diff_z + box_extent_z * andrew_woo_diff_y {
        return false;
    }

    f = ray_dir.z * diff_x - ray_dir.x * diff_z;
    if f.abs() > box_extent_x * andrew_woo_diff_z + box_extent_z * andrew_woo_diff_x {
        return false;
    }

    f = ray_dir.x * diff_y - ray_dir.y * diff_x;
    f.abs() < box_extent_x * andrew_woo_diff_y + box_extent_y * andrew_woo_diff_x
}

/// Return Java `Mth.hsvToRgb(float, float, float)` with alpha zero.
pub fn mth_hsv_to_rgb_f32(hue: f32, saturation: f32, value: f32) -> Result<i32, &'static str> {
    mth_hsv_to_argb_f32(hue, saturation, value, 0)
}

/// Return Java `Mth.hsvToArgb(float, float, float, int)` in source order.
pub fn mth_hsv_to_argb_f32(
    hue: f32,
    saturation: f32,
    value: f32,
    alpha: i32,
) -> Result<i32, &'static str> {
    let h = (hue * 6.0_f32) as i32 % 6;
    let f = hue * 6.0_f32 - h as f32;
    let p = value * (1.0_f32 - saturation);
    let q = value * (1.0_f32 - f * saturation);
    let t = value * (1.0_f32 - (1.0_f32 - f) * saturation);
    let (red, green, blue) = match h {
        0 => (value, t, p),
        1 => (q, value, p),
        2 => (p, value, t),
        3 => (p, q, value),
        4 => (t, p, value),
        5 => (value, p, q),
        _ => return Err("hue_out_of_range"),
    };

    let red = clamp_i32((red * 255.0_f32) as i32, 0, 255) as u32;
    let green = clamp_i32((green * 255.0_f32) as i32, 0, 255) as u32;
    let blue = clamp_i32((blue * 255.0_f32) as i32, 0, 255) as u32;
    Ok((((alpha as u32 & 0xff) << 24) | (red << 16) | (green << 8) | blue) as i32)
}

// The same contract at two precisions, without a public numeric trait hierarchy.
macro_rules! scalar_operations {
    ($ty:ty, $floor:ident, $wrap:ident, $clamp:ident) => {
        /// Round downward and convert to i32, saturating at the signed bounds.
        /// NaN converts to zero, as with the reference's Java narrowing cast.
        pub fn $floor(value: $ty) -> i32 {
            f64::from(value).floor() as i32
        }

        /// Wrap degrees to [-180, 180), retaining the sign of zero.
        pub fn $wrap(value: $ty) -> $ty {
            let remainder = value % 360.0;
            if remainder >= 180.0 {
                remainder - 360.0
            } else if remainder < -180.0 {
                remainder + 360.0
            } else {
                remainder
            }
        }

        /// Reference clamp semantics, including NaN and signed-zero behavior.
        /// Unlike a general clamp API, this does not assert ordered bounds.
        pub fn $clamp(value: $ty, min: $ty, max: $ty) -> $ty {
            if value < min {
                min
            } else if value.is_nan() {
                value
            } else if max.is_nan() {
                max
            } else if value == 0.0 && max == 0.0 {
                // Java Math.min chooses negative zero when either zero is negative.
                <$ty>::from_bits(value.to_bits() | max.to_bits())
            } else if value <= max {
                value
            } else {
                max
            }
        }
    };
}

scalar_operations!(f32, floor_f32, wrap_degrees_f32, clamp_f32);

scalar_operations!(f64, floor_f64, wrap_degrees_f64, clamp_f64);
