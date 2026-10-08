//! The legacy random source used by the pinned Java reference.
//!
//! `RandomSource.create(seed)` selects the 48-bit linear congruential source
//! in this slice. The public methods keep the reference's bit widths and
//! integer overflow behavior explicit so callers can reproduce its sequence.

/// Errors raised by bounded random operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomError {
    InvalidBound,
}

/// Java-compatible 48-bit legacy random source.
#[derive(Debug, Clone)]
pub struct RandomSource {
    seed: u64,
    gaussian: Option<f64>,
}

impl RandomSource {
    const MASK: u64 = (1 << 48) - 1;
    const MULTIPLIER: u64 = 25_214_903_917;
    const INCREMENT: u64 = 11;

    /// Create a source with the same seed scrambling as Java's constructor.
    pub fn create(seed: i64) -> Self {
        let mut source = Self {
            seed: 0,
            gaussian: None,
        };
        source.set_seed(seed);
        source
    }

    /// Reset the source state and clear any cached Gaussian value.
    pub fn set_seed(&mut self, seed: i64) {
        self.seed = (seed as u64 ^ Self::MULTIPLIER) & Self::MASK;
        self.gaussian = None;
    }

    /// The 48-bit state the source holds (the reference's `seed` field).
    pub fn state(&self) -> u64 {
        self.seed
    }

    fn next_bits(&mut self, bits: u32) -> u32 {
        debug_assert!((1..=32).contains(&bits));
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT)
            & Self::MASK;
        (self.seed >> (48 - bits)) as u32
    }

    /// Draw a signed 32-bit value.
    pub fn next_int(&mut self) -> i32 {
        self.next_bits(32) as i32
    }

    /// Draw a value in `[0, bound)`, using Java's rejection rule.
    pub fn next_int_bound(&mut self, bound: i32) -> Result<i32, RandomError> {
        if bound <= 0 {
            return Err(RandomError::InvalidBound);
        }

        if bound & (bound - 1) == 0 {
            let sample = i64::from(self.next_bits(31));
            return Ok(((i64::from(bound) * sample) >> 31) as i32);
        }

        loop {
            let sample = self.next_bits(31) as i32;
            let modulo = sample % bound;
            // Java evaluates this condition as a signed i32 expression.
            let acceptance = sample.wrapping_sub(modulo).wrapping_add(bound - 1);
            if acceptance >= 0 {
                return Ok(modulo);
            }
        }
    }

    /// Draw an integer from the inclusive range using Java i32 arithmetic.
    pub fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> Result<i32, RandomError> {
        let bound = max.wrapping_sub(min).wrapping_add(1);
        self.next_int_bound(bound)
            .map(|value| value.wrapping_add(min))
    }

    /// Draw an integer from the half-open range `[origin, bound)`.
    pub fn next_int_range(&mut self, origin: i32, bound: i32) -> Result<i32, RandomError> {
        if origin >= bound {
            return Err(RandomError::InvalidBound);
        }
        let width = bound.wrapping_sub(origin);
        self.next_int_bound(width)
            .map(|value| value.wrapping_add(origin))
    }

    /// Draw a signed 64-bit value using two 32-bit draws.
    pub fn next_long(&mut self) -> i64 {
        let upper = self.next_bits(32) as i32;
        let lower = self.next_bits(32) as i32;
        i64::from(upper)
            .wrapping_shl(32)
            .wrapping_add(i64::from(lower))
    }

    /// Draw a boolean from one source bit.
    pub fn next_boolean(&mut self) -> bool {
        self.next_bits(1) != 0
    }

    /// Draw a Java-compatible 24-bit float fraction.
    pub fn next_float(&mut self) -> f32 {
        self.next_bits(24) as f32 * 5.9604645e-8_f32
    }

    /// Draw a Java-compatible 53-bit double fraction.
    pub fn next_double(&mut self) -> f64 {
        let upper = i64::from(self.next_bits(26));
        let lower = i64::from(self.next_bits(27));
        let combined = upper.wrapping_shl(27).wrapping_add(lower);
        combined as f64 * 1.1102230246251565e-16_f64
    }

    /// Draw a double triangle value from two consecutive doubles.
    pub fn triangle_f64(&mut self, mean: f64, spread: f64) -> f64 {
        let difference = self.next_double() - self.next_double();
        mean + spread * difference
    }

    /// Draw a float triangle value from two consecutive floats.
    pub fn triangle_f32(&mut self, mean: f32, spread: f32) -> f32 {
        let difference = self.next_float() - self.next_float();
        mean + spread * difference
    }

    /// Consume one integer draw for each nonnegative count below `rounds`.
    pub fn consume_count(&mut self, rounds: i32) {
        for _ in 0..rounds {
            self.next_int();
        }
    }

    /// Draw a Java-compatible Gaussian value, caching the paired result.
    pub fn next_gaussian(&mut self) -> f64 {
        if let Some(value) = self.gaussian.take() {
            return value;
        }

        loop {
            let x = 2.0 * self.next_double() - 1.0;
            let y = 2.0 * self.next_double() - 1.0;
            let radius_squared = x * x + y * y;
            if radius_squared >= 1.0 || radius_squared == 0.0 {
                continue;
            }

            let multiplier =
                (-2.0 * crate::math::strict_log(radius_squared) / radius_squared).sqrt();
            self.gaussian = Some(y * multiplier);
            return x * multiplier;
        }
    }

    /// Fork this source using one signed `next_long` seed.
    pub fn fork(&mut self) -> Self {
        Self::create(self.next_long())
    }

    /// Fork a factory using one signed `next_long` seed.
    pub fn fork_positional(&mut self) -> PositionalRandomFactory {
        PositionalRandomFactory {
            seed: self.next_long(),
        }
    }
}

/// Factory for deterministic random sources derived from positions or names.
#[derive(Debug, Clone, Copy)]
pub struct PositionalRandomFactory {
    seed: i64,
}

impl PositionalRandomFactory {
    /// Create a source from integer coordinates using Java's `Mth.getSeed`.
    pub fn at(&self, x: i32, y: i32, z: i32) -> RandomSource {
        RandomSource::create(crate::math::mth_get_seed(x, y, z) ^ self.seed)
    }

    /// Create a source from Java's UTF-16 `String.hashCode` result.
    pub fn from_hash_of(&self, name: &str) -> RandomSource {
        RandomSource::create(i64::from(java_string_hash(name)) ^ self.seed)
    }

    /// Create a source directly from a signed seed.
    pub fn from_seed(&self, seed: i64) -> RandomSource {
        RandomSource::create(seed)
    }
}

fn java_string_hash(value: &str) -> i32 {
    value.encode_utf16().fold(0_i32, |hash, code_unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(code_unit))
    })
}
