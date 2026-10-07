//! `Math.log`'s natural logarithm, as the reference's `StrictMath.log` computes it (fdlibm).
//!
//! The platform's libm rounds a result differently from fdlibm in rare cases (one unit in the
//! last place), and the reference's `Math.log` is fdlibm's wherever the JVM has no intrinsic of
//! its own, so the Gaussian source takes this one and gives the same bits on every platform.

const LN2_HI: f64 = f64::from_bits(0x3fe6_2e42_fee0_0000);
const LN2_LO: f64 = f64::from_bits(0x3dea_39ef_3579_3c76);
const TWO54: f64 = 1.801_439_850_948_198_4e16;
const LG1: f64 = f64::from_bits(0x3fe5_5555_5555_5593);
const LG2: f64 = f64::from_bits(0x3fd9_9999_9997_fa04);
const LG3: f64 = f64::from_bits(0x3fd2_4924_9422_9359);
const LG4: f64 = f64::from_bits(0x3fcc_71c5_1d8e_78af);
const LG5: f64 = f64::from_bits(0x3fc7_4664_96cb_03de);
const LG6: f64 = f64::from_bits(0x3fc3_9a09_d078_c69f);
const LG7: f64 = f64::from_bits(0x3fc2_f112_df3e_5244);

fn high_word(x: f64) -> i32 {
    (x.to_bits() >> 32) as i32
}

fn low_word(x: f64) -> u32 {
    x.to_bits() as u32
}

fn with_high_word(x: f64, high: i32) -> f64 {
    f64::from_bits((u64::from(high as u32) << 32) | u64::from(low_word(x)))
}

/// `StrictMath.log`.
pub fn strict_log(mut x: f64) -> f64 {
    let mut hx = high_word(x);
    let lx = low_word(x);
    let mut k: i32 = 0;
    if hx < 0x0010_0000 {
        // x < 2**-1022
        if ((hx & 0x7fff_ffff) as u32 | lx) == 0 {
            return f64::NEG_INFINITY;
        }
        if hx < 0 {
            return f64::NAN;
        }
        k -= 54;
        x *= TWO54;
        hx = high_word(x);
    }
    if hx >= 0x7ff0_0000 {
        return x + x;
    }
    k += (hx >> 20) - 1023;
    hx &= 0x000f_ffff;
    let mut i = (hx + 0x95f64) & 0x10_0000;
    x = with_high_word(x, hx | (i ^ 0x3ff0_0000));
    k += i >> 20;
    let f = x - 1.0;
    if (0x000f_ffff & (2 + hx)) < 3 {
        // |f| < 2**-20
        if f == 0.0 {
            if k == 0 {
                return 0.0;
            }
            let dk = f64::from(k);
            return dk * LN2_HI + dk * LN2_LO;
        }
        let r = f * f * (0.5 - 0.333_333_333_333_333_3 * f);
        if k == 0 {
            return f - r;
        }
        let dk = f64::from(k);
        return dk * LN2_HI - ((r - dk * LN2_LO) - f);
    }
    let s = f / (2.0 + f);
    let dk = f64::from(k);
    let z = s * s;
    i = hx - 0x6147a;
    let w = z * z;
    let j = 0x6b851 - hx;
    let t1 = w * (LG2 + w * (LG4 + w * LG6));
    let t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
    i |= j;
    let r = t2 + t1;
    if i > 0 {
        let hfsq = 0.5 * f * f;
        if k == 0 {
            f - (hfsq - s * (hfsq + r))
        } else {
            dk * LN2_HI - ((hfsq - (s * (hfsq + r) + dk * LN2_LO)) - f)
        }
    } else if k == 0 {
        f - s * (f - r)
    } else {
        dk * LN2_HI - ((s * (f - r) - dk * LN2_LO) - f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_references_strict_log() {
        // Taken from `StrictMath.log` of the reference's JVM: the argument and the result as bits.
        for (argument, result) in [
            (0x3fe7_61e2_f51b_b9a0_u64, 0xbfd4_148a_06bf_f7f0_u64),
            (0x3fc9_945a_0277_0b39, 0xbff9_c38a_0c83_7b88),
            (0x3ff0_0000_0000_0000, 0x0000_0000_0000_0000),
            (0x4000_0000_0000_0000, 0x3fe6_2e42_fefa_39ef),
        ] {
            assert_eq!(
                strict_log(f64::from_bits(argument)).to_bits(),
                result,
                "log of {argument:x}"
            );
        }
    }

    #[test]
    fn edge_arguments() {
        assert_eq!(strict_log(0.0), f64::NEG_INFINITY);
        assert!(strict_log(-1.0).is_nan());
        assert_eq!(strict_log(f64::INFINITY), f64::INFINITY);
        assert!(strict_log(f64::NAN).is_nan());
    }
}
