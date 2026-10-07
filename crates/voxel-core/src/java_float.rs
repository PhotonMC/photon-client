//! Floating-point values kept as the reference keeps them, with bit equality.
//!
//! A state compared or hashed as a whole (for a test, or to tell a change) must tell `0.0` from
//! `-0.0` and one `NaN` from another, which `==` on a float does not. These types make the bits
//! the value's identity, so a struct holding them can be `Eq` and `Hash` without storing bits.
//! Arithmetic is done on [`get`](JavaF64::get), in the reference's order, and the result stored
//! again with `new`.

use std::hash::{Hash, Hasher};

macro_rules! java_float {
    ($(#[$meta:meta])* $name:ident, $float:ty) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Default)]
        pub struct $name($float);

        impl $name {
            pub const ZERO: Self = Self(0.0);

            pub const fn new(value: $float) -> Self {
                Self(value)
            }

            pub const fn get(self) -> $float {
                self.0
            }

            pub const fn to_bits(self) -> <$float as FloatBits>::Bits {
                self.0.to_bits()
            }

            pub const fn from_bits(bits: <$float as FloatBits>::Bits) -> Self {
                Self(<$float>::from_bits(bits))
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0.to_bits() == other.0.to_bits()
            }
        }

        impl Eq for $name {}

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.0.to_bits().hash(state);
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl From<$float> for $name {
            fn from(value: $float) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $float {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

/// The unsigned integer that holds a float's bits.
pub trait FloatBits {
    type Bits;
}

impl FloatBits for f64 {
    type Bits = u64;
}

impl FloatBits for f32 {
    type Bits = u32;
}

java_float!(
    /// A Java `double`, equal to another when its bits are.
    JavaF64, f64
);
java_float!(
    /// A Java `float`, equal to another when its bits are.
    JavaF32, f32
);

/// Three Java doubles: a position, a motion, an offset.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct DVec3 {
    pub x: JavaF64,
    pub y: JavaF64,
    pub z: JavaF64,
}

impl DVec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: JavaF64::new(x),
            y: JavaF64::new(y),
            z: JavaF64::new(z),
        }
    }

    /// The components as plain doubles, for arithmetic.
    pub const fn get(self) -> (f64, f64, f64) {
        (self.x.get(), self.y.get(), self.z.get())
    }
}

impl From<(f64, f64, f64)> for DVec3 {
    fn from((x, y, z): (f64, f64, f64)) -> Self {
        Self::new(x, y, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn equality_is_by_bits() {
        assert_ne!(JavaF64::new(0.0), JavaF64::new(-0.0));
        assert_eq!(JavaF64::new(f64::NAN), JavaF64::new(f64::NAN));
        assert_ne!(JavaF32::new(1.0), JavaF32::new(1.000_000_1));
        assert_eq!(JavaF32::new(0.1), JavaF32::new(0.1));
    }

    #[test]
    fn the_value_comes_back_unchanged_in_every_bit() {
        for bits in [
            0_u64,
            1,
            0x8000_0000_0000_0000,
            0x7ff8_0000_0000_0001,
            u64::MAX,
        ] {
            assert_eq!(JavaF64::from_bits(bits).to_bits(), bits);
        }
        assert_eq!(JavaF32::from_bits(0x7fc0_0001).to_bits(), 0x7fc0_0001);
    }

    #[test]
    fn equal_values_hash_alike_and_zero_signs_differ() {
        let mut seen = HashSet::new();
        seen.insert(DVec3::new(1.0, 2.0, 3.0));
        assert!(seen.contains(&DVec3::new(1.0, 2.0, 3.0)));
        assert!(!seen.contains(&DVec3::new(1.0, 2.0, -0.0)));
        assert_eq!(DVec3::ZERO, DVec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn debug_shows_the_number() {
        assert_eq!(format!("{:?}", JavaF64::new(1.5)), "1.5");
        assert!(format!("{:?}", DVec3::new(1.0, 2.0, 3.0)).contains("x: 1.0"));
    }
}
