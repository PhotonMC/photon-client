//! `StrictMath.asin` (fdlibm `__ieee754_asin`). `Mth`'s atan2 tables are
//! built from it when the class loads.

use super::{hi, lo, with_lo};

const HUGE: f64 = 1.000e+300;
const PIO2_HI: f64 = 1.57079632679489655800e+00;
const PIO2_LO: f64 = 6.12323399573676603587e-17;
const PIO4_HI: f64 = 7.85398163397448278999e-01;
const PS0: f64 = 1.66666666666666657415e-01;
const PS1: f64 = -3.25565818622400915405e-01;
const PS2: f64 = 2.01212532134862925881e-01;
const PS3: f64 = -4.00555345006794114027e-02;
const PS4: f64 = 7.91534994289814532176e-04;
const PS5: f64 = 3.47933107596021167570e-05;
const QS1: f64 = -2.40339491173441421878e+00;
const QS2: f64 = 2.02094576023350569471e+00;
const QS3: f64 = -6.88283971605453293030e-01;
const QS4: f64 = 7.70381505559019352791e-02;

/// The numerator and denominator of the rational approximation in `t`.
fn rational(t: f64) -> (f64, f64) {
    let p = t * (PS0 + t * (PS1 + t * (PS2 + t * (PS3 + t * (PS4 + t * PS5)))));
    let q = 1.0 + t * (QS1 + t * (QS2 + t * (QS3 + t * QS4)));
    (p, q)
}

/// `asin(x)`, rounded exactly as fdlibm rounds it.
pub fn fdlibm_asin(x: f64) -> f64 {
    let hx = hi(x);
    let ix = hx & 0x7fff_ffff;
    if ix >= 0x3ff0_0000 {
        if ((ix - 0x3ff0_0000) as u32 | lo(x)) == 0 {
            return x * PIO2_HI + x * PIO2_LO;
        }
        return (x - x) / (x - x);
    }
    if ix < 0x3fe0_0000 {
        return small(x, ix);
    }
    let t = near_one(x, ix);
    if hx > 0 { t } else { -t }
}

/// `|x| < 0.5`.
fn small(x: f64, ix: i32) -> f64 {
    if ix < 0x3e40_0000 && HUGE + x > 1.0 {
        return x;
    }
    let t = x * x;
    let (p, q) = rational(t);
    x + x * (p / q)
}

/// `0.5 <= |x| < 1`, without the sign.
fn near_one(x: f64, ix: i32) -> f64 {
    let w = 1.0 - x.abs();
    let t = w * 0.5;
    let (p, q) = rational(t);
    let s = t.sqrt();
    if ix >= 0x3fef_3333 {
        let w = p / q;
        return PIO2_HI - (2.0 * (s + s * w) - PIO2_LO);
    }
    let w = with_lo(s, 0);
    let c = (t - w * w) / (s + w);
    let r = p / q;
    let p = 2.0 * s * r - (PIO2_LO - 2.0 * c);
    let q = PIO4_HI - 2.0 * w;
    PIO4_HI - (p - q)
}
