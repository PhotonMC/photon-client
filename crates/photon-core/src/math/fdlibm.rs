//! `StrictMath.sin`, `cos`, `asin` and `pow` (fdlibm), which the reference's `Math.*` gives when the JVM
//! has no stub of its own (the oracle runs with them off). The platform's libm rounds a result
//! differently in rare cases, which the particle simulations carry into their positions.
//!
//! `sin` and `cos` reduce arguments up to 2^19 * pi / 2 (about 8e5) as fdlibm does; beyond that
//! the platform's answer is taken, which no simulation here reaches.

#![expect(
    clippy::excessive_precision,
    clippy::approx_constant,
    reason = "the constants are fdlibm's own decimal text, kept as published"
)]
#![expect(
    clippy::eq_op,
    reason = "fdlibm makes its NaNs and infinities by `x - x` and `0 / 0` style arithmetic"
)]

mod asin;
pub use asin::fdlibm_asin;

const S1: f64 = -1.66666666666666324348e-01;
const S2: f64 = 8.33333333332248946124e-03;
const S3: f64 = -1.98412698298579493134e-04;
const S4: f64 = 2.75573137070700676789e-06;
const S5: f64 = -2.50507602534068634195e-08;
const S6: f64 = 1.58969099521155010221e-10;
const C1: f64 = 4.16666666666666019037e-02;
const C2: f64 = -1.38888888888741095749e-03;
const C3: f64 = 2.48015872894767294178e-05;
const C4: f64 = -2.75573143513906633035e-07;
const C5: f64 = 2.08757232129817482790e-09;
const C6: f64 = -1.13596475577881948265e-11;
const INVPIO2: f64 = 6.36619772367581382433e-01;
const PIO2_1: f64 = 1.57079632673412561417e+00;
const PIO2_1T: f64 = 6.07710050650619224932e-11;
const PIO2_2: f64 = 6.07710050630396597660e-11;
const PIO2_2T: f64 = 2.02226624879595063154e-21;
const PIO2_3: f64 = 2.02226624871116645580e-21;
const PIO2_3T: f64 = 8.47842766036889956997e-32;

fn hi(x: f64) -> i32 {
    (x.to_bits() >> 32) as i32
}

fn lo(x: f64) -> u32 {
    x.to_bits() as u32
}

fn with_hi(x: f64, high: i32) -> f64 {
    f64::from_bits((u64::from(high as u32) << 32) | u64::from(lo(x)))
}

fn with_lo(x: f64, low: u32) -> f64 {
    f64::from_bits((x.to_bits() & 0xffff_ffff_0000_0000) | u64::from(low))
}

/// `__kernel_sin`; `tail` says whether `y` is used.
fn kernel_sin(x: f64, y: f64, tail: bool) -> f64 {
    let ix = hi(x) & 0x7fff_ffff;
    if ix < 0x3e40_0000 && x as i32 == 0 {
        return x;
    }
    let z = x * x;
    let v = z * x;
    let r = S2 + z * (S3 + z * (S4 + z * (S5 + z * S6)));
    if tail {
        x - ((z * (0.5 * y - v * r) - y) - v * S1)
    } else {
        x + v * (S1 + z * r)
    }
}

/// `__kernel_cos`.
fn kernel_cos(x: f64, y: f64) -> f64 {
    let ix = hi(x) & 0x7fff_ffff;
    if ix < 0x3e40_0000 && x as i32 == 0 {
        return 1.0;
    }
    let z = x * x;
    let r = z * (C1 + z * (C2 + z * (C3 + z * (C4 + z * (C5 + z * C6)))));
    if ix < 0x3fd3_3333 {
        1.0 - (0.5 * z - (z * r - x * y))
    } else {
        let qx = if ix > 0x3fe9_0000 {
            0.28125
        } else {
            f64::from_bits(u64::from((ix - 0x0020_0000) as u32) << 32)
        };
        let hz = 0.5 * z - qx;
        let a = 1.0 - qx;
        a - (hz - (z * r - x * y))
    }
}

/// `rem_pio2` for `|x| < 3 pi / 4`: one subtraction of pi / 2 in the direction of the sign.
fn rem_pio2_one_step(x: f64, hx: i32, ix: i32) -> (i32, f64, f64) {
    if hx > 0 {
        let mut z = x - PIO2_1;
        let (y0, y1);
        if ix == 0x3ff9_21fb {
            z -= PIO2_2;
            y0 = z - PIO2_2T;
            y1 = (z - y0) - PIO2_2T;
        } else {
            y0 = z - PIO2_1T;
            y1 = (z - y0) - PIO2_1T;
        }
        return (1, y0, y1);
    }
    let mut z = x + PIO2_1;
    let (y0, y1);
    if ix == 0x3ff9_21fb {
        z += PIO2_2;
        y0 = z + PIO2_2T;
        y1 = (z - y0) + PIO2_2T;
    } else {
        y0 = z + PIO2_1T;
        y1 = (z - y0) + PIO2_1T;
    }
    (-1, y0, y1)
}

/// `__ieee754_rem_pio2` for arguments up to 2^19 * pi / 2: the quadrant and the remainder as a
/// head and a tail.
fn rem_pio2(x: f64) -> (i32, f64, f64) {
    let hx = hi(x);
    let ix = hx & 0x7fff_ffff;
    if ix < 0x4002_d97c {
        return rem_pio2_one_step(x, hx, ix);
    }
    let t = x.abs();
    let n = (t * INVPIO2 + 0.5) as i32;
    let fn_ = f64::from(n);
    let mut r = t - fn_ * PIO2_1;
    let mut w = fn_ * PIO2_1T;
    // The high word of n * pi / 2 for the first 32 multiples (`npio2_hw`).
    let near_multiple = n < 32 && ix == hi(f64::from(n) * std::f64::consts::FRAC_PI_2);
    let y0 = if n < 32 && !near_multiple {
        r - w
    } else {
        let j = ix >> 20;
        let mut y = r - w;
        let mut i = j - ((hi(y) >> 20) & 0x7ff);
        if i > 16 {
            let t = r;
            w = fn_ * PIO2_2;
            r = t - w;
            w = fn_ * PIO2_2T - ((t - r) - w);
            y = r - w;
            i = j - ((hi(y) >> 20) & 0x7ff);
            if i > 49 {
                let t = r;
                w = fn_ * PIO2_3;
                r = t - w;
                w = fn_ * PIO2_3T - ((t - r) - w);
                y = r - w;
            }
        }
        y
    };
    let y1 = (r - y0) - w;
    if hx < 0 { (-n, -y0, -y1) } else { (n, y0, y1) }
}

/// Whether an argument is beyond what `rem_pio2` here reduces.
fn too_large(x: f64) -> bool {
    (hi(x) & 0x7fff_ffff) > 0x4139_21fb
}

/// `StrictMath.sin`.
pub fn fdlibm_sin(x: f64) -> f64 {
    let ix = hi(x) & 0x7fff_ffff;
    if ix <= 0x3fe9_21fb {
        return kernel_sin(x, 0.0, false);
    }
    if ix >= 0x7ff0_0000 {
        return x - x;
    }
    if too_large(x) {
        return x.sin();
    }
    let (n, y0, y1) = rem_pio2(x);
    match n & 3 {
        0 => kernel_sin(y0, y1, true),
        1 => kernel_cos(y0, y1),
        2 => -kernel_sin(y0, y1, true),
        _ => -kernel_cos(y0, y1),
    }
}

/// `StrictMath.cos`.
pub fn fdlibm_cos(x: f64) -> f64 {
    let ix = hi(x) & 0x7fff_ffff;
    if ix <= 0x3fe9_21fb {
        return kernel_cos(x, 0.0);
    }
    if ix >= 0x7ff0_0000 {
        return x - x;
    }
    if too_large(x) {
        return x.cos();
    }
    let (n, y0, y1) = rem_pio2(x);
    match n & 3 {
        0 => kernel_cos(y0, y1),
        1 => -kernel_sin(y0, y1, true),
        2 => -kernel_cos(y0, y1),
        _ => kernel_sin(y0, y1, true),
    }
}

const BP: [f64; 2] = [1.0, 1.5];
const DP_H: [f64; 2] = [0.0, 5.84962487220764160156e-01];
const DP_L: [f64; 2] = [0.0, 1.35003920212974897128e-08];
const TWO53: f64 = 9007199254740992.0;
const HUGE: f64 = 1.0e300;
const TINY: f64 = 1.0e-300;
const L1: f64 = 5.99999999999994648725e-01;
const L2: f64 = 4.28571428578550184252e-01;
const L3: f64 = 3.33333329818377432918e-01;
const L4: f64 = 2.72728123808534006489e-01;
const L5: f64 = 2.30660745775561754067e-01;
const L6: f64 = 2.06975017800338417784e-01;
const P1: f64 = 1.66666666666666019037e-01;
const P2: f64 = -2.77777777770155933842e-03;
const P3: f64 = 6.61375632143793436117e-05;
const P4: f64 = -1.65339022054652515390e-06;
const P5: f64 = 4.13813679705723846039e-08;
const LG2: f64 = 6.93147180559945286227e-01;
const LG2_H: f64 = 6.93147182464599609375e-01;
const LG2_L: f64 = -1.90465429995776804525e-09;
const OVT: f64 = 8.0085662595372944372e-17;
const CP: f64 = 9.61796693925975554329e-01;
const CP_H: f64 = 9.61796700954437255859e-01;
const CP_L: f64 = -7.02846165095275826516e-09;
const IVLN2: f64 = 1.44269504088896338700e+00;
const IVLN2_H: f64 = 1.44269502162933349609e+00;
const IVLN2_L: f64 = 1.92596299112661746887e-08;

/// `scalbn(x, n)` for the one case `pow` needs: a result that is subnormal or near it.
fn scalbn(x: f64, n: i32) -> f64 {
    // Scaling by powers of two is exact except where the result is subnormal, where one rounding
    // happens; two steps keep the exponent in range.
    let half = n / 2;
    x * f64::from_bits(((0x3ff + half) as u64) << 52)
        * f64::from_bits(((0x3ff + n - half) as u64) << 52)
}

/// Whether `y` is an odd integer (1), an even one (2) or not an integer (0), for `x < 0`.
fn y_is_int(hx: i32, iy: i32, ly: u32) -> i32 {
    if hx >= 0 {
        return 0;
    }
    if iy >= 0x4340_0000 {
        return 2;
    }
    if iy >= 0x3ff0_0000 {
        let k = (iy >> 20) - 0x3ff;
        if k > 20 {
            let j = ly >> (52 - k);
            if (j << (52 - k)) == ly {
                return 2 - (j & 1) as i32;
            }
        } else if ly == 0 {
            let j = iy >> (20 - k);
            if (j << (20 - k)) == iy {
                return 2 - (j & 1);
            }
        }
    }
    0
}

/// Whether a high word and low word, with the sign cleared, are a NaN.
fn is_nan_words(high: i32, low: u32) -> bool {
    high > 0x7ff0_0000 || (high == 0x7ff0_0000 && low != 0)
}

/// `pow(x, y)` for an infinite `y`.
fn pow_infinite_exponent(x: f64, y: f64) -> f64 {
    let (hx, lx) = (hi(x), lo(x));
    let ix = hx & 0x7fff_ffff;
    let hy = hi(y);
    if ((ix - 0x3ff0_0000) as u32 | lx) == 0 {
        y - y
    } else if ix >= 0x3ff0_0000 {
        if hy >= 0 { y } else { 0.0 }
    } else if hy < 0 {
        -y
    } else {
        0.0
    }
}

/// `pow` for an exponent with a zero low word that is infinite, one, two or one half.
fn pow_simple_exponent(x: f64, y: f64) -> Option<f64> {
    let hx = hi(x);
    let (hy, iy) = (hi(y), hi(y) & 0x7fff_ffff);
    if iy == 0x7ff0_0000 {
        return Some(pow_infinite_exponent(x, y));
    }
    if iy == 0x3ff0_0000 {
        return Some(if hy < 0 { 1.0 / x } else { x });
    }
    if hy == 0x4000_0000 {
        return Some(x * x);
    }
    if hy == 0x3fe0_0000 && hx >= 0 {
        return Some(x.sqrt());
    }
    None
}

/// `pow` for a base of zero, one or infinity (`yisint` as `y_is_int` gives it).
fn pow_unit_base(x: f64, y: f64, yisint: i32) -> Option<f64> {
    let (hx, lx) = (hi(x), lo(x));
    let ix = hx & 0x7fff_ffff;
    if lx != 0 || !(ix == 0x7ff0_0000 || ix == 0 || ix == 0x3ff0_0000) {
        return None;
    }
    let mut z = x.abs();
    if hi(y) < 0 {
        z = 1.0 / z;
    }
    if hx < 0 {
        if ((ix - 0x3ff0_0000) | yisint) == 0 {
            z = (z - z) / (z - z);
        } else if yisint == 1 {
            z = -z;
        }
    }
    Some(z)
}

/// The arguments `pow` answers without computing: the sign of the result to come, or the answer.
fn pow_special(x: f64, y: f64) -> Result<f64, f64> {
    let (hx, lx) = (hi(x), lo(x));
    let (hy, ly) = (hi(y), lo(y));
    let ix = hx & 0x7fff_ffff;
    let iy = hy & 0x7fff_ffff;
    if (iy as u32 | ly) == 0 {
        return Err(1.0);
    }
    if is_nan_words(ix, lx) || is_nan_words(iy, ly) {
        return Err(x + y);
    }
    let yisint = y_is_int(hx, iy, ly);
    if ly == 0
        && let Some(answer) = pow_simple_exponent(x, y)
    {
        return Err(answer);
    }
    if let Some(answer) = pow_unit_base(x, y, yisint) {
        return Err(answer);
    }
    let n_sign = (hx >> 31) + 1;
    if (n_sign | yisint) == 0 {
        return Err((x - x) / (x - x));
    }
    Ok(if (n_sign | (yisint - 1)) == 0 {
        -1.0
    } else {
        1.0
    })
}

/// `log2(|x|)` as a head and a tail, for `|y| > 2^31` with `x` close to one.
fn log2_near_one(ax: f64) -> (f64, f64) {
    let t = ax - 1.0;
    let w = (t * t) * (0.5 - t * (0.333_333_333_333_333_333_333_3 - t * 0.25));
    let u = IVLN2_H * t;
    let v = t * IVLN2_L - w * IVLN2;
    let high = with_lo(u + v, 0);
    (high, v - (high - u))
}

/// `log2(|x|)` as a head and a tail.
fn log2_parts(mut ax: f64) -> (f64, f64) {
    let mut n = 0_i32;
    let mut ix = hi(ax);
    if ix < 0x0010_0000 {
        ax *= TWO53;
        n -= 53;
        ix = hi(ax);
    }
    n += (ix >> 20) - 0x3ff;
    let j = ix & 0x000f_ffff;
    ix = j | 0x3ff0_0000;
    let k;
    if j <= 0x3988e {
        k = 0;
    } else if j < 0xbb67a {
        k = 1;
    } else {
        k = 0;
        n += 1;
        ix -= 0x0010_0000;
    }
    ax = with_hi(ax, ix);
    // ss = s_h + s_l = (x - 1) / (x + 1), or (x - 1.5) / (x + 1.5).
    let u = ax - BP[k];
    let v = 1.0 / (ax + BP[k]);
    let ss = u * v;
    let s_h = with_lo(ss, 0);
    let t_h = f64::from_bits(
        u64::from((((ix >> 1) | 0x2000_0000) + 0x0008_0000 + ((k as i32) << 18)) as u32) << 32,
    );
    let t_l = ax - (t_h - BP[k]);
    let s_l = v * ((u - s_h * t_h) - s_h * t_l);
    let mut s2 = ss * ss;
    let mut r = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))));
    r += s_l * (s_h + ss);
    s2 = s_h * s_h;
    let t_h = with_lo(3.0 + s2 + r, 0);
    let t_l = r - ((t_h - 3.0) - s2);
    let u = s_h * t_h;
    let v = s_l * t_h + t_l * ss;
    let p_h = with_lo(u + v, 0);
    let p_l = v - (p_h - u);
    let z_h = CP_H * p_h;
    let z_l = CP_L * p_h + p_l * CP + DP_L[k];
    let t = f64::from(n);
    let high = with_lo(((z_h + z_l) + DP_H[k]) + t, 0);
    (high, z_l - (((high - t) - DP_H[k]) - z_h))
}

/// `StrictMath.pow`.
pub fn fdlibm_pow(x: f64, y: f64) -> f64 {
    let s = match pow_special(x, y) {
        Ok(sign) => sign,
        Err(answer) => return answer,
    };
    let ix = hi(x) & 0x7fff_ffff;
    let iy = hi(y) & 0x7fff_ffff;
    let hy = hi(y);
    let ax = x.abs();
    let (t1, t2);
    if iy > 0x41e0_0000 {
        // |y| > 2^31.
        if iy > 0x43f0_0000 {
            if ix <= 0x3fef_ffff {
                return if hy < 0 { HUGE * HUGE } else { TINY * TINY };
            }
            if ix >= 0x3ff0_0000 {
                return if hy > 0 { HUGE * HUGE } else { TINY * TINY };
            }
        }
        if ix < 0x3fef_ffff {
            return if hy < 0 {
                s * HUGE * HUGE
            } else {
                s * TINY * TINY
            };
        }
        if ix > 0x3ff0_0000 {
            return if hy > 0 {
                s * HUGE * HUGE
            } else {
                s * TINY * TINY
            };
        }
        (t1, t2) = log2_near_one(ax);
    } else {
        (t1, t2) = log2_parts(ax);
    }
    // Split y into y1 + y2 and compute (y1 + y2) * (t1 + t2).
    let y1 = with_lo(y, 0);
    let p_l = (y - y1) * t1 + y * t2;
    let p_h = y1 * t1;
    let z = p_l + p_h;
    let j = hi(z);
    let i = lo(z);
    if j >= 0x4090_0000 {
        if ((j - 0x4090_0000) as u32 | i) != 0 || p_l + OVT > z - p_h {
            return s * HUGE * HUGE;
        }
    } else if (j & 0x7fff_ffff) >= 0x4090_cc00
        && (((j as u32).wrapping_sub(0xc090_cc00)) | i != 0 || p_l <= z - p_h)
    {
        return s * TINY * TINY;
    }
    s * exp2_parts(p_h, p_l, j)
}

/// `2^(p_h + p_l)`, where `j` is the high word of their sum.
fn exp2_parts(mut p_h: f64, p_l: f64, j: i32) -> f64 {
    let i = j & 0x7fff_ffff;
    let mut k = (i >> 20) - 0x3ff;
    let mut n = 0_i32;
    if i > 0x3fe0_0000 {
        n = j.wrapping_add(0x0010_0000 >> (k + 1));
        k = ((n & 0x7fff_ffff) >> 20) - 0x3ff;
        let t = f64::from_bits(u64::from((n & !(0x000f_ffff >> k)) as u32) << 32);
        n = ((n & 0x000f_ffff) | 0x0010_0000) >> (20 - k);
        if j < 0 {
            n = -n;
        }
        p_h -= t;
    }
    let t = with_lo(p_l + p_h, 0);
    let u = t * LG2_H;
    let v = (p_l - (t - p_h)) * LG2 + t * LG2_L;
    let mut z = u + v;
    let w = v - (z - u);
    let t = z * z;
    let t1 = z - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    let r = (z * t1) / (t1 - 2.0) - (w + z * w);
    z = 1.0 - (r - z);
    let high = hi(z).wrapping_add(n << 20);
    if (high >> 20) <= 0 {
        scalbn(z, n)
    } else {
        with_hi(z, high)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `StrictMath` of the reference's JVM: the argument bits and the result bits.
    const SIN: [(u64, u64); 6] = [
        (0xbffc37b160ea59fc_u64, 0xbfef6836dc81346b_u64),
        (0x4054bf05af82d8b1_u64, 0x3feedcaf7186ac44_u64),
        (0x41141c146e5aa363_u64, 0xbfe116d8329676f3_u64),
        (0x3fe5450652598d00_u64, 0x3fe3bcdbd02866d2_u64),
        (0x403f1506c5f159e0_u64, 0xbfd4f7db84ef1d86_u64),
        (0x40fb64a7b9e0ad3b_u64, 0xbfdec0fae259fe81_u64),
    ];
    const COS: [(u64, u64); 6] = [
        (0xbffc37b160ea59fc_u64, 0xbfc8869f18e91a71_u64),
        (0x4054bf05af82d8b1_u64, 0x3fd0ea5fd1a9424c_u64),
        (0x41141c146e5aa363_u64, 0x3feb0e0247de2342_u64),
        (0x3fe5450652598d00_u64, 0x3fe9300f7961ab9a_u64),
        (0x403f1506c5f159e0_u64, 0x3fee3bdb75ad32e9_u64),
        (0x40fb64a7b9e0ad3b_u64, 0xbfec10366bbcfd1d_u64),
    ];
    const POW: [(u64, u64, u64); 6] = [
        (
            0x408a8c9d1ef99648_u64,
            0xbff67e3eefb1e04a_u64,
            0x3f13faf89d54d624_u64,
        ),
        (
            0x3fcde8c567d4f308_u64,
            0xc025c5f5b59e706f_u64,
            0x415c882cd4d5b33e_u64,
        ),
        (
            0x401820c41f219e39_u64,
            0x3ff4000000000000_u64,
            0x4022e80322408ba0_u64,
        ),
        (
            0x3ef1859f13c5f439_u64,
            0x4029dc02293b4a4f_u64,
            0x331c3db18b053614_u64,
        ),
        (
            0xc08cf04859110a7c_u64,
            0x4010000000000000_u64,
            0x426566fce150dff4_u64,
        ),
        (
            0x3fe92508f5055893_u64,
            0x3fe0f546ea702958_u64,
            0x3fec2976b30ac55b_u64,
        ),
    ];

    #[test]
    fn matches_the_references_strict_math() {
        for (x, want) in SIN {
            assert_eq!(fdlibm_sin(f64::from_bits(x)).to_bits(), want, "sin {x:x}");
        }
        for (x, want) in COS {
            assert_eq!(fdlibm_cos(f64::from_bits(x)).to_bits(), want, "cos {x:x}");
        }
        for (x, y, want) in POW {
            let got = fdlibm_pow(f64::from_bits(x), f64::from_bits(y));
            assert_eq!(got.to_bits(), want, "pow {x:x} {y:x}");
        }
    }

    #[test]
    fn edge_arguments() {
        assert_eq!(fdlibm_pow(2.0, 0.0), 1.0);
        assert_eq!(fdlibm_pow(0.0, -1.0), f64::INFINITY);
        assert!(fdlibm_pow(-8.0, 0.5).is_nan());
        assert_eq!(fdlibm_pow(-2.0, 3.0), -8.0);
        assert!(fdlibm_sin(f64::INFINITY).is_nan());
        assert_eq!(fdlibm_cos(0.0), 1.0);
    }
}
