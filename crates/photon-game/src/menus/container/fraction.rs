//! Apache Commons `Fraction`, which a bundle's weights are computed in.

/// A reduced fraction. The plain operations are exact; the `checked_` ones fail where the
/// reference throws an `ArithmeticException` (an `int` that overflows, a zero denominator).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Fraction {
    pub(super) numerator: i64,
    pub(super) denominator: i64,
}

/// The value if it fits an `int`.
fn fit(value: i64) -> Option<i64> {
    i32::try_from(value).ok().map(i64::from)
}

/// `Fraction.greatestCommonDivisor` of two values that are not `Integer.MIN_VALUE`.
fn gcd(first: i64, second: i64) -> i64 {
    let (mut a, mut b) = (first.abs(), second.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Fraction {
    pub(super) const ZERO: Self = Self::new(0, 1);
    pub(super) const ONE: Self = Self::new(1, 1);

    pub(super) const fn new(numerator: i64, denominator: i64) -> Self {
        let (mut a, mut b) = (numerator.abs(), denominator.abs());
        while b != 0 {
            (a, b) = (b, a % b);
        }
        let gcd = if a == 0 { 1 } else { a };
        let sign = if denominator < 0 { -1 } else { 1 };
        Self {
            numerator: sign * numerator / gcd,
            denominator: sign * denominator / gcd,
        }
    }

    /// `Fraction.getFraction(numerator, denominator)`: no zero denominator, and a negative one
    /// is moved to the numerator.
    pub(super) fn checked_new(numerator: i32, denominator: i32) -> Option<Self> {
        if denominator == 0
            || (denominator < 0 && (numerator == i32::MIN || denominator == i32::MIN))
        {
            return None;
        }
        Some(Self::new(i64::from(numerator), i64::from(denominator)))
    }

    pub(super) const fn add(self, other: Self) -> Self {
        Self::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }

    pub(super) const fn subtract(self, other: Self) -> Self {
        self.add(Self::new(-other.numerator, other.denominator))
    }

    pub(super) const fn times(self, count: i64) -> Self {
        Self::new(self.numerator * count, self.denominator)
    }

    /// `divideBy(...).intValue()`: truncated toward zero.
    pub(super) const fn whole_times_in(self, divisor: Self) -> i64 {
        (self.numerator * divisor.denominator) / (self.denominator * divisor.numerator)
    }

    /// `add` as `Fraction.addSub` does it, failing where it throws.
    pub(super) fn checked_add(self, other: Self) -> Option<Self> {
        if self.numerator == 0 {
            return Some(other);
        }
        if other.numerator == 0 {
            return Some(self);
        }
        let d1 = gcd(self.denominator, other.denominator);
        if d1 == 1 {
            let uvp = fit(self.numerator * other.denominator)?;
            let upv = fit(other.numerator * self.denominator)?;
            return Some(Self {
                numerator: fit(uvp + upv)?,
                denominator: fit(self.denominator * other.denominator)?,
            });
        }
        let uvp = i128::from(self.numerator) * i128::from(other.denominator / d1);
        let upv = i128::from(other.numerator) * i128::from(self.denominator / d1);
        let total = uvp + upv;
        let remainder = total.rem_euclid(i128::from(d1)) as i64;
        let d2 = if remainder == 0 {
            d1
        } else {
            gcd(remainder, d1)
        };
        let numerator = i32::try_from(total / i128::from(d2)).ok()?;
        Some(Self {
            numerator: i64::from(numerator),
            denominator: fit((self.denominator / d1) * (other.denominator / d2))?,
        })
    }

    /// `multiplyBy(Fraction.getFraction(count, 1))`, failing where it throws.
    pub(super) fn checked_times(self, count: i32) -> Option<Self> {
        let count = i64::from(count);
        if self.numerator == 0 || count == 0 {
            return Some(Self::ZERO);
        }
        let d2 = gcd(count, self.denominator);
        let numerator = fit(self.numerator * (count / d2))?;
        let denominator = fit(self.denominator / d2)?;
        Some(Self::new(numerator, denominator))
    }
}
