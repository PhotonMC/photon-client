use super::*;

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

/// Return Java `Mth.wrapDegrees(int)`.
pub fn wrap_degrees_i32(angle: i32) -> i32 {
    let mut normalized_angle = angle % 360;
    if normalized_angle >= 180 {
        normalized_angle -= 360;
    }
    if normalized_angle < -180 {
        normalized_angle += 360;
    }
    normalized_angle
}

/// Return Java `Mth.wrapDegrees(long)`, including the final f32 narrowing.
pub fn wrap_degrees_i64(angle: i64) -> f32 {
    let mut normalized_angle = (angle % 360) as f32;
    if normalized_angle >= 180.0 {
        normalized_angle -= 360.0;
    }
    if normalized_angle < -180.0 {
        normalized_angle += 360.0;
    }
    normalized_angle
}

/// Return Java `Mth.packDegrees(float)` with byte narrowing semantics.
pub fn pack_degrees_f32(angle: f32) -> i8 {
    floor_f32(angle * 256.0 / 360.0) as i8
}

/// Return Java `Mth.unpackDegrees(byte)`.
pub fn unpack_degrees_i8(rotation: i8) -> f32 {
    (i32::from(rotation) * 360) as f32 / 256.0
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

/// Return Java `Mth.createInsecureUUID(RandomSource)` from two long draws.
pub fn mth_create_insecure_uuid(random: &mut RandomSource) -> Uuid {
    let most = (random.next_long() as u64 & (-61_441_i64 as u64)) | 16_384_u64;
    let least = (random.next_long() as u64 & 4_611_686_018_427_387_903_u64) | (i64::MIN as u64);
    Uuid::new(most, least)
}

/// Return Java `Mth.murmurHash3Mixer(int)` with i32 wrapping arithmetic.
pub fn mth_murmur_hash3_mixer(mut hash: i32) -> i32 {
    hash ^= ((hash as u32) >> 16) as i32;
    hash = hash.wrapping_mul(-2_048_144_789);
    hash ^= ((hash as u32) >> 13) as i32;
    hash = hash.wrapping_mul(-1_028_477_387);
    hash ^ ((hash as u32) >> 16) as i32
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

/// Return Java `Mth.wrapDegrees90(float)` with its 90-degree remainder path.
pub fn mth_wrap_degrees_90_f32(angle: f32) -> f32 {
    let mut normalized_angle = angle % 90.0_f32;
    if normalized_angle >= 45.0_f32 {
        normalized_angle -= 90.0_f32;
    }
    if normalized_angle < -45.0_f32 {
        normalized_angle += 90.0_f32;
    }
    normalized_angle
}

/// Return Java `Mth.degreesDifference(float, float)` in source order.
pub fn mth_degrees_difference_f32(from_angle: f32, to_angle: f32) -> f32 {
    wrap_degrees_f32(to_angle - from_angle)
}

/// Return Java `Mth.degreesDifferenceAbs(float, float)` with Java abs semantics.
pub fn mth_degrees_difference_abs_f32(angle_a: f32, angle_b: f32) -> f32 {
    mth_abs_f32(mth_degrees_difference_f32(angle_a, angle_b))
}

/// Return Java `Mth.rotateIfNecessary(float, float, float)`.
pub fn mth_rotate_if_necessary_f32(base_angle: f32, target_angle: f32, max_angle_diff: f32) -> f32 {
    let delta_angle = mth_degrees_difference_f32(base_angle, target_angle);
    let delta_angle_clamped = clamp_f32(delta_angle, -max_angle_diff, max_angle_diff);
    target_angle - delta_angle_clamped
}

/// Return Java `Mth.approach(float, float, float)`.
pub fn mth_approach_f32(current: f32, target: f32, increment: f32) -> f32 {
    let increment = mth_abs_f32(increment);
    if current < target {
        clamp_f32(current + increment, current, target)
    } else {
        clamp_f32(current - increment, target, current)
    }
}

/// Return Java `Mth.approachDegrees(float, float, float)`.
pub fn mth_approach_degrees_f32(current: f32, target: f32, increment: f32) -> f32 {
    let difference = mth_degrees_difference_f32(current, target);
    mth_approach_f32(current, current + difference, increment)
}

pub(super) fn java_decimal_digit(character: char) -> Option<i64> {
    const DIGIT_RANGES: &[(u32, u32)] = &[
        (0x0030, 0x0039),
        (0x0660, 0x0669),
        (0x06f0, 0x06f9),
        (0x07c0, 0x07c9),
        (0x0966, 0x096f),
        (0x09e6, 0x09ef),
        (0x0a66, 0x0a6f),
        (0x0ae6, 0x0aef),
        (0x0b66, 0x0b6f),
        (0x0be6, 0x0bef),
        (0x0c66, 0x0c6f),
        (0x0ce6, 0x0cef),
        (0x0d66, 0x0d6f),
        (0x0de6, 0x0def),
        (0x0e50, 0x0e59),
        (0x0ed0, 0x0ed9),
        (0x0f20, 0x0f29),
        (0x1040, 0x1049),
        (0x1090, 0x1099),
        (0x17e0, 0x17e9),
        (0x1810, 0x1819),
        (0x1946, 0x194f),
        (0x19d0, 0x19d9),
        (0x1a80, 0x1a89),
        (0x1a90, 0x1a99),
        (0x1b50, 0x1b59),
        (0x1bb0, 0x1bb9),
        (0x1c40, 0x1c49),
        (0x1c50, 0x1c59),
        (0xa620, 0xa629),
        (0xa8d0, 0xa8d9),
        (0xa900, 0xa909),
        (0xa9d0, 0xa9d9),
        (0xa9f0, 0xa9f9),
        (0xaa50, 0xaa59),
        (0xabf0, 0xabf9),
        (0xff10, 0xff19),
    ];
    let codepoint = character as u32;
    DIGIT_RANGES.iter().find_map(|&(first, last)| {
        if (first..=last).contains(&codepoint) {
            Some(i64::from(codepoint - first))
        } else {
            None
        }
    })
}

pub(super) fn parse_java_i32(input: &str) -> Option<i32> {
    let mut characters = input.chars();
    let first = characters.next()?;
    let (negative, first_digit) = match first {
        '-' => (true, None),
        '+' => (false, None),
        character => (false, Some(java_decimal_digit(character)?)),
    };
    let limit = if negative {
        i64::from(i32::MIN)
    } else {
        -i64::from(i32::MAX)
    };
    let mut result = 0_i64;
    let mut digit_count = 0;
    if let Some(digit) = first_digit {
        result = -digit;
        digit_count = 1;
    }
    for character in characters {
        let digit = java_decimal_digit(character)?;
        result = result.checked_mul(10)?.checked_sub(digit)?;
        if result < limit {
            return None;
        }
        digit_count += 1;
    }
    if digit_count == 0 {
        return None;
    }
    if negative {
        Some(result as i32)
    } else {
        Some((-result) as i32)
    }
}

/// Return Java `Mth.getInt(String, int)`, including `Integer.parseInt` digit rules.
pub fn mth_get_int(input: Option<&str>, default: i32) -> i32 {
    input.and_then(parse_java_i32).unwrap_or(default)
}

/// Return Java `Mth.getSeed(int, int, int)` with Java-width overflow.
pub fn mth_get_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut seed = i64::from(x.wrapping_mul(3_129_871))
        ^ i64::from(z).wrapping_mul(116_129_781)
        ^ i64::from(y);
    seed = seed
        .wrapping_mul(seed)
        .wrapping_mul(42_317_861)
        .wrapping_add(seed.wrapping_mul(11));
    seed >> 16
}
