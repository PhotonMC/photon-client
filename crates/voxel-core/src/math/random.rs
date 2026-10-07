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
