//! Cases for `mth/interpolation`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::values::*;

/// A `lerp3` row: label, three alphas and the eight corner values.
type Lerp3Row = (&'static str, f64, f64, f64, Bits, Bits, Bits, Bits, Bits, Bits, Bits, Bits);

const MIN_SUBNORMAL_F64: f64 = 5e-324;
const MAXIMUM_FINITE_F64: f64 = 1.7976931348623157e+308;

fn lerp3() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_lerp3 = mth_lerp3_rows();
    for (label, alpha1, alpha2, alpha3, x000, x100, x010, x110, x001, x101, x011, x111) in mth_lerp3.iter().cloned() {
        out.push(case(
            format!("mth/interpolation/lerp3/{}", label),
            "math",
            "mth_interpolation",
            json!({
                "fn": "lerp3",
                "alpha1": interpolation_bits(alpha1, "f64"),
                "alpha2": interpolation_bits(alpha2, "f64"),
                "alpha3": interpolation_bits(alpha3, "f64"),
                "x000": interpolation_bits(x000.clone(), "f64"),
                "x100": interpolation_bits(x100.clone(), "f64"),
                "x010": interpolation_bits(x010.clone(), "f64"),
                "x110": interpolation_bits(x110.clone(), "f64"),
                "x001": interpolation_bits(x001.clone(), "f64"),
                "x101": interpolation_bits(x101.clone(), "f64"),
                "x011": interpolation_bits(x011.clone(), "f64"),
                "x111": interpolation_bits(x111.clone(), "f64"),
            }),
        ));
    }
    out
}

fn mth_lerp3_rows() -> Vec<Lerp3Row> {
    [
        mth_lerp3_basic(),
        mth_lerp3_signed_zero_alpha(),
        mth_lerp3_tiny_alpha(),
        mth_lerp3_positive_infinity_cube(),
        mth_lerp3_nan_first_corner(),
        mth_lerp3_mixed_exponents(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn mth_lerp3_basic() -> Vec<Lerp3Row> {
    vec![
        ("basic", 0.25, 0.5, 0.75, num(0.0), num(1.0), num(2.0), num(3.0), num(4.0), num(5.0), num(6.0), num(7.0)),
        ("zero-alpha", 0.0, 0.0, 0.0, num(1.0), num(2.0), num(3.0), num(4.0), num(5.0), num(6.0), num(7.0), num(8.0)),
        ("one-alpha", 1.0, 1.0, 1.0, num(1.0), num(2.0), num(3.0), num(4.0), num(5.0), num(6.0), num(7.0), num(8.0)),
        (
            "extrapolation",
            1.5,
            -0.5,
            1.25,
            num(0.0),
            num(10.0),
            num(20.0),
            num(30.0),
            num(40.0),
            num(50.0),
            num(60.0),
            num(70.0),
        ),
        (
            "negative-alpha",
            -0.25,
            0.75,
            -0.5,
            num(-1.0),
            num(3.0),
            num(-5.0),
            num(7.0),
            num(-9.0),
            num(11.0),
            num(-13.0),
            num(15.0),
        ),
        (
            "reversed-grid",
            0.75,
            0.25,
            0.5,
            num(10.0),
            num(0.0),
            num(-10.0),
            num(-20.0),
            num(20.0),
            num(30.0),
            num(40.0),
            num(50.0),
        ),
    ]
}

fn mth_lerp3_signed_zero_alpha() -> Vec<Lerp3Row> {
    vec![
        (
            "signed-zero-alpha",
            -0.0,
            0.0,
            -0.0,
            num(-0.0),
            num(1.0),
            num(-0.0),
            num(1.0),
            num(-0.0),
            num(1.0),
            num(-0.0),
            num(1.0),
        ),
        (
            "signed-zero-cube",
            0.5,
            0.5,
            0.5,
            raw("8000000000000000"),
            raw("0000000000000000"),
            raw("8000000000000000"),
            raw("0000000000000000"),
            raw("8000000000000000"),
            raw("0000000000000000"),
            raw("8000000000000000"),
            raw("0000000000000000"),
        ),
        (
            "subnormal-cube",
            0.5,
            0.5,
            0.5,
            num(MIN_SUBNORMAL_F64),
            num(2.0 * MIN_SUBNORMAL_F64),
            num(3.0 * MIN_SUBNORMAL_F64),
            num(4.0 * MIN_SUBNORMAL_F64),
            num(-MIN_SUBNORMAL_F64),
            num(-2.0 * MIN_SUBNORMAL_F64),
            num(-3.0 * MIN_SUBNORMAL_F64),
            num(-4.0 * MIN_SUBNORMAL_F64),
        ),
    ]
}

fn mth_lerp3_tiny_alpha() -> Vec<Lerp3Row> {
    vec![
        (
            "tiny-alpha",
            2.225073858507202e-308,
            0.5,
            0.75,
            num(1.0),
            num(2.0),
            num(3.0),
            num(4.0),
            num(5.0),
            num(6.0),
            num(7.0),
            num(8.0),
        ),
        (
            "large-finite",
            0.25,
            0.75,
            0.5,
            num(1e+300),
            num(-1e+300),
            num(1e+300),
            num(-1e+300),
            num(1e+300),
            num(-1e+300),
            num(1e+300),
            num(-1e+300),
        ),
        (
            "overflow-cube",
            0.5,
            0.5,
            0.5,
            num(MAXIMUM_FINITE_F64),
            num(-MAXIMUM_FINITE_F64),
            num(MAXIMUM_FINITE_F64),
            num(-MAXIMUM_FINITE_F64),
            num(MAXIMUM_FINITE_F64),
            num(-MAXIMUM_FINITE_F64),
            num(MAXIMUM_FINITE_F64),
            num(-MAXIMUM_FINITE_F64),
        ),
    ]
}

fn mth_lerp3_positive_infinity_cube() -> Vec<Lerp3Row> {
    vec![
        (
            "positive-infinity-cube",
            0.25,
            0.5,
            0.75,
            num(f64::INFINITY),
            num(1.0),
            num(2.0),
            num(3.0),
            num(4.0),
            num(5.0),
            num(6.0),
            num(f64::INFINITY),
        ),
        (
            "negative-infinity-cube",
            0.25,
            0.5,
            0.75,
            num(f64::NEG_INFINITY),
            num(-1.0),
            num(-2.0),
            num(-3.0),
            num(-4.0),
            num(-5.0),
            num(-6.0),
            num(f64::NEG_INFINITY),
        ),
        (
            "nan-alpha",
            f64::NAN,
            0.5,
            0.75,
            num(0.0),
            num(1.0),
            num(2.0),
            num(3.0),
            num(4.0),
            num(5.0),
            num(6.0),
            num(7.0),
        ),
    ]
}

fn mth_lerp3_nan_first_corner() -> Vec<Lerp3Row> {
    vec![
        (
            "nan-first-corner",
            0.5,
            0.5,
            0.5,
            num(f64::NAN),
            num(1.0),
            num(2.0),
            num(3.0),
            num(4.0),
            num(5.0),
            num(6.0),
            num(7.0),
        ),
        (
            "nan-last-corner",
            0.5,
            0.5,
            0.5,
            num(0.0),
            num(1.0),
            num(2.0),
            num(3.0),
            num(4.0),
            num(5.0),
            num(6.0),
            num(f64::NAN),
        ),
        (
            "precision-order",
            0.1,
            0.9,
            0.3,
            num(1e+20),
            num(1.00000000000001e+20),
            num(-1e+20),
            num(-9.9999999999999e+19),
            num(2e+20),
            num(2.00000000000001e+20),
            num(-2e+20),
            num(-1.99999999999999e+20),
        ),
    ]
}

fn mth_lerp3_mixed_exponents() -> Vec<Lerp3Row> {
    vec![
        (
            "mixed-exponents",
            0.3333333333333333,
            0.6666666666666666,
            0.125,
            num(3.054936363499605e-151),
            num(3.273390607896142e+150),
            num(-3.054936363499605e-151),
            num(-3.273390607896142e+150),
            num(3.8725919148493183e-121),
            num(2.5822498780869086e+120),
            num(-3.8725919148493183e-121),
            num(-2.5822498780869086e+120),
        ),
        (
            "underflow",
            MIN_SUBNORMAL_F64,
            0.5,
            0.75,
            num(1.0),
            num(2.0),
            num(3.0),
            num(4.0),
            num(5.0),
            num(6.0),
            num(7.0),
            num(8.0),
        ),
    ]
}

fn catmullrom() -> Vec<Value> {
    let mut out = Vec::new();
    let maximum_finite_f32 = 3.4028234663852886e+38;
    let min_subnormal_f32 = 1.401298464324817e-45;
    let mth_catmullrom = vec![
        ("basic", num(0.5), num(0.0), num(1.0), num(2.0), num(3.0)),
        ("zero-alpha", num(0.0), num(-1.0), num(0.0), num(1.0), num(2.0)),
        ("one-alpha", num(1.0), num(-1.0), num(0.0), num(1.0), num(2.0)),
        ("extrapolation-negative", num(-0.5), num(0.0), num(1.0), num(2.0), num(3.0)),
        ("extrapolation-positive", num(1.5), num(0.0), num(1.0), num(2.0), num(3.0)),
        ("constant", num(0.25), num(4.0), num(4.0), num(4.0), num(4.0)),
        ("peak", num(0.5), num(0.0), num(0.0), num(1.0), num(0.0)),
        ("symmetric", num(0.5), num(-2.0), num(-1.0), num(1.0), num(2.0)),
        ("negative", num(0.25), num(-1.0), num(-3.0), num(-5.0), num(-7.0)),
        ("signed-zero-alpha", raw("80000000"), num(-0.0), num(0.0), num(-0.0), num(0.0)),
        ("signed-zero-points", num(0.5), raw("80000000"), raw("00000000"), raw("80000000"), raw("00000000")),
        (
            "subnormal-points",
            num(0.5),
            num(min_subnormal_f32),
            num(2.0 * min_subnormal_f32),
            num(3.0 * min_subnormal_f32),
            num(4.0 * min_subnormal_f32),
        ),
        ("subnormal-alpha", num(min_subnormal_f32), num(1.0), num(2.0), num(3.0), num(4.0)),
        ("large-finite", num(0.25), num(10000000000.0), num(-10000000000.0), num(10000000000.0), num(-10000000000.0)),
        (
            "maximum-finite",
            num(0.5),
            num(maximum_finite_f32),
            num(-maximum_finite_f32),
            num(maximum_finite_f32),
            num(-maximum_finite_f32),
        ),
        ("overflow", num(maximum_finite_f32), num(1.0), num(-1.0), num(1.0), num(-1.0)),
        ("positive-infinity", num(0.5), num(f64::INFINITY), num(1.0), num(2.0), num(3.0)),
        ("negative-infinity", num(0.5), num(f64::NEG_INFINITY), num(-1.0), num(-2.0), num(-3.0)),
        ("nan-alpha", num(f64::NAN), num(0.0), num(1.0), num(2.0), num(3.0)),
        ("nan-point", num(0.5), raw("7fc00001"), num(1.0), num(2.0), num(3.0)),
    ];
    for (label, alpha, p0, p1, p2, p3) in mth_catmullrom.iter().cloned() {
        out.push(case(
            format!("mth/interpolation/catmullrom/{}", label),
            "math",
            "mth_interpolation",
            json!({
                "fn": "catmullrom",
                "alpha": interpolation_bits(alpha, "f32"),
                "p0": interpolation_bits(p0, "f32"),
                "p1": interpolation_bits(p1, "f32"),
                "p2": interpolation_bits(p2.clone(), "f32"),
                "p3": interpolation_bits(p3.clone(), "f32"),
            }),
        ));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [lerp3(), catmullrom()].concat()
}
