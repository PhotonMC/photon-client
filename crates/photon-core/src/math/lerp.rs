use super::*;

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
    let positive_alpha = i32::from(alpha > 0.0);
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
