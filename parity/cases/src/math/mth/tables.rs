//! The tables the `mth` cases share.

use crate::bits::{f32_bits, f64_bits};
use crate::math::values::*;

/// A rotation row: label, axis and quaternion.
type AxisRotation = (&'static str, (f64, f64, f64), (f64, f64, f64, f64));

pub fn mth_frac_helpers() -> Vec<(&'static str, &'static str, String)> {
    vec![
        ("f32/zero", "f32", f32_bits(0.0)),
        ("f32/negative-zero", "f32", "80000000".to_string()),
        ("f32/positive-fraction", "f32", f32_bits(1.25)),
        ("f32/negative-fraction", "f32", f32_bits(-1.25)),
        ("f32/positive-exact", "f32", f32_bits(7.0)),
        ("f32/negative-exact", "f32", f32_bits(-7.0)),
        ("f32/positive-subnormal", "f32", f32_bits(1.401298464324817e-45)),
        ("f32/negative-subnormal", "f32", f32_bits(-1.401298464324817e-45)),
        ("f32/int-upper-overflow", "f32", f32_bits(2147483648.0)),
        ("f32/infinity", "f32", f32_bits(f64::INFINITY)),
        ("f32/negative-infinity", "f32", f32_bits(f64::NEG_INFINITY)),
        ("f32/nan", "f32", f32_bits(f64::NAN)),
        ("f64/zero", "f64", f64_bits(0.0)),
        ("f64/negative-zero", "f64", "8000000000000000".to_string()),
        ("f64/positive-fraction", "f64", f64_bits(1.25)),
        ("f64/negative-fraction", "f64", f64_bits(-1.25)),
        ("f64/positive-exact", "f64", f64_bits(7.0)),
        ("f64/negative-exact", "f64", f64_bits(-7.0)),
        ("f64/positive-subnormal", "f64", f64_bits(5e-324)),
        ("f64/long-upper-overflow", "f64", f64_bits(9.223372036854776e+18)),
        ("f64/long-lower-bound", "f64", f64_bits(-9.223372036854776e+18)),
        ("f64/infinity", "f64", f64_bits(f64::INFINITY)),
        ("f64/negative-infinity", "f64", f64_bits(f64::NEG_INFINITY)),
        ("f64/nan", "f64", f64_bits(f64::NAN)),
    ]
}

pub fn mth_hsv_to_color() -> Vec<(&'static str, Bits, Bits, Bits, i64)> {
    vec![
        ("red-sector", num(0.0), num(1.0), num(1.0), 255),
        ("yellow-sector", num(0.16666666666666666), num(1.0), num(1.0), 128),
        ("green-sector", num(0.3333333333333333), num(1.0), num(1.0), 64),
        ("cyan-sector", num(0.5), num(1.0), num(1.0), 32),
        ("blue-sector", num(0.6666666666666666), num(1.0), num(1.0), 16),
        ("magenta-sector", num(0.8333333333333334), num(1.0), num(1.0), 8),
        ("wrapped-one", num(1.0), num(1.0), num(1.0), 255),
        ("boundary-below", num(0.16666666666666663), num(1.0), num(1.0), 255),
        ("zero-saturation", num(0.5), num(0.0), num(0.5), 17),
        ("fractional", num(0.23), num(0.61), num(0.73), 200),
        ("small-negative-hue", num(-0.1), num(1.0), num(1.0), -1),
        ("wrapped-positive-hue", num(1.2), num(1.0), num(1.0), 256),
        ("negative-sector", num(-0.2), num(1.0), num(1.0), 0),
        ("nan-hue", raw("7fc00000"), num(0.5), num(0.5), 42),
        ("positive-infinity-hue", raw("7f800000"), num(0.5), num(0.5), 42),
        ("negative-infinity-hue", raw("ff800000"), num(0.5), num(0.5), 42),
        ("nan-saturation", num(0.25), raw("7fc00000"), num(0.75), 255),
        ("nan-value", num(0.25), num(0.5), raw("7fc00000"), 255),
        ("outside-components", num(0.75), num(-0.5), num(1.5), 256),
        ("negative-value", num(0.75), num(0.5), num(-1.0), -128),
        ("signed-zero", raw("80000000"), raw("80000000"), raw("80000000"), 0),
    ]
}

pub fn mth_atan2() -> Vec<(&'static str, Bits, f64)> {
    vec![
        ("zero-zero", num(0.0), 0.0),
        ("negative-zero-y", num(-0.0), 0.0),
        ("negative-zero-x", num(0.0), -0.0),
        ("both-negative-zero", num(-0.0), -0.0),
        ("positive-x-axis", num(0.0), 1.0),
        ("negative-x-axis", num(0.0), -1.0),
        ("positive-y-axis", num(1.0), 0.0),
        ("negative-y-axis", num(-1.0), 0.0),
        ("first-quadrant", num(1.0), 1.0),
        ("second-quadrant", num(1.0), -1.0),
        ("third-quadrant", num(-1.0), -1.0),
        ("fourth-quadrant", num(-1.0), 1.0),
        ("shallow-positive", num(0.25), 1.0),
        ("steep-positive", num(1.0), 0.25),
        ("shallow-negative-y", num(-0.25), 1.0),
        ("steep-negative-y", num(-1.0), 0.25),
        ("table-boundary-low", num(0.0039062499999999996), 1.0),
        ("table-boundary", num(0.00390625), 1.0),
        ("table-boundary-high", num(0.003906250000000001), 1.0),
        ("steep-boundary-low", num(1.0), 0.9999999999999999),
        ("steep-boundary-high", num(1.0), 1.0000000000000002),
        ("tiny-y", num(5e-324), 1.0),
        ("both-min-subnormal", num(5e-324), 5e-324),
        ("mixed-exponents", num(3.054936363499605e-151), 3.273390607896142e+150),
        ("large-finite", num(1e+200), -1e+200),
        ("maximum-y", num(1e+308), 0.0),
        ("positive-infinity-x", num(1.0), f64::INFINITY),
        ("negative-infinity-x", num(1.0), f64::NEG_INFINITY),
        ("positive-infinity-y", num(f64::INFINITY), 1.0),
        ("negative-infinity-y", num(f64::NEG_INFINITY), 1.0),
        ("both-positive-infinity", num(f64::INFINITY), f64::INFINITY),
        ("nan-y", num(f64::NAN), 1.0),
        ("nan-x", num(1.0), f64::NAN),
        ("nan-y-payload", raw("7ff0000000000001"), 1.0),
    ]
}

pub fn mth_lengths() -> Vec<(&'static str, &'static str, &'static str, Vec<f64>)> {
    vec![
        ("f64-2/squared-basic", "f64_2", "length_squared", vec![3.0, 4.0]),
        ("f64-2/length-basic", "f64_2", "length", vec![3.0, 4.0]),
        ("f64-2/squared-signed-zero", "f64_2", "length_squared", vec![-0.0, 0.0]),
        ("f64-2/length-infinity", "f64_2", "length", vec![f64::INFINITY, 1.0]),
        ("f64-2/length-nan", "f64_2", "length", vec![f64::NAN, 1.0]),
        ("f32-2/length-basic", "f32_2", "length", vec![3.0, 4.0]),
        ("f32-2/length-signed-zero", "f32_2", "length", vec![-0.0, 0.0]),
        ("f32-2/length-overflow", "f32_2", "length", vec![3.4028234663852886e+38, 1.0]),
        ("f64-3/squared-basic", "f64_3", "length_squared", vec![1.0, 2.0, 2.0]),
        ("f64-3/length-basic", "f64_3", "length", vec![1.0, 2.0, 2.0]),
        ("f64-3/squared-overflow", "f64_3", "length_squared", vec![1e+308, 1e-308, 0.0]),
        ("f64-3/length-nan", "f64_3", "length", vec![f64::INFINITY, 0.0, f64::NAN]),
        ("f64-3/length-signed-zero", "f64_3", "length", vec![-0.0, 0.0, -0.0]),
        ("f32-3/squared-basic", "f32_3", "length_squared", vec![1.0, 2.0, 2.0]),
        (
            "f32-3/squared-overflow",
            "f32_3",
            "length_squared",
            vec![3.4028234663852886e+38, 3.4028234663852886e+38, 0.0],
        ),
        ("f32-3/squared-signed-zero", "f32_3", "length_squared", vec![-0.0, 0.0, -0.0]),
    ]
}

pub fn mth_quantize() -> Vec<(&'static str, f64, i64)> {
    vec![
        ("positive-fraction", 5.9, 2),
        ("exact-multiple", 6.0, 2),
        ("negative-fraction", -5.9, 2),
        ("negative-resolution", 5.9, -2),
        ("near-integer-rounding", 5.999999999999999, 2),
        ("zero", 0.0, 7),
        ("negative-zero", -0.0, 7),
        ("subnormal", 5e-324, 3),
        ("positive-infinity", f64::INFINITY, 2),
        ("negative-infinity", f64::NEG_INFINITY, 2),
        ("nan", f64::NAN, 2),
        ("above-i32-range", 2147483648.5, 1),
        ("below-i32-range", -2147483649.5, 1),
        ("multiplication-overflow", 4000000000.0, 2),
        ("zero-resolution-positive", 1.5, 0),
        ("zero-resolution-negative", -1.5, 0),
        ("zero-resolution-zero", 0.0, 0),
        ("minimum-resolution", 1.0, -2147483648),
        ("minimum-resolution-overflow", -1.0, -2147483648),
    ]
}

pub fn mth_rotation_around_axis() -> Vec<AxisRotation> {
    vec![
        ("axis-x", (1.0, 0.0, 0.0), (0.5, -0.25, 0.75, 1.0)),
        ("axis-y", (0.0, 1.0, 0.0), (-0.5, 0.25, -0.75, 1.0)),
        ("axis-z", (0.0, 0.0, 1.0), (0.5, 0.25, -0.75, -1.0)),
        ("non-unit-axis", (1.0, 2.0, 3.0), (0.25, -0.5, 0.75, 1.0)),
        ("negative-axis", (-1.0, 2.0, -3.0), (0.25, 0.5, -0.75, -0.5)),
        ("zero-axis", (0.0, 0.0, 0.0), (0.25, -0.5, 0.75, 1.0)),
        ("zero-vector", (1.0, -2.0, 3.0), (0.0, 0.0, 0.0, 1.0)),
        ("zero-scalar", (1.0, 2.0, 3.0), (0.25, -0.5, 0.75, 0.0)),
        ("projection-negative", (1.0, 2.0, 3.0), (-1.0, -2.0, -3.0, 0.5)),
        ("fma-cancellation", (1.0, 1.0, 1.0), (1.0, 1.1920928955078125e-07, -1.0, 0.25)),
        ("fma-order", (3.0, -2.0, 0.5), (0.125, 0.25, 0.75, -0.25)),
        ("fractional-axis", (0.125, -0.25, 0.5), (0.75, 0.5, -0.25, 0.125)),
        ("signed-zero", (-0.0, 0.0, -0.0), (-0.0, 0.0, -0.0, -0.0)),
        ("signed-zero-projection", (-0.0, 1.0, 0.0), (0.0, -0.0, 0.0, 1.0)),
        ("subnormal", (1.401298464324817e-45, 1.0, -1.0), (2.802596928649634e-45, -0.5, 0.25, 0.5)),
        ("tiny-quaternion", (1.0, -1.0, 0.5), (1.401298464324817e-45, -1.401298464324817e-45, 0.0, 1.0)),
        ("maximum-finite", (3.4028234663852886e+38, 1.0, -1.0), (1.0, 0.5, -0.25, 0.75)),
        ("infinite-axis", (f64::INFINITY, 1.0, 0.0), (0.0, 1.0, 0.0, 1.0)),
        ("infinite-rotation", (1.0, 0.0, 0.0), (f64::INFINITY, 1.0, 0.0, 1.0)),
        ("nan-axis", (f64::NAN, 1.0, 0.0), (1.0, 0.0, 0.0, 1.0)),
        ("nan-rotation", (1.0, 0.0, 0.0), (f64::NAN, 1.0, 0.0, 1.0)),
        ("zero-quaternion", (1.0, 2.0, 3.0), (0.0, 0.0, 0.0, 0.0)),
        ("large-components", (4096.0, -2048.0, 1024.0), (512.0, 256.0, -128.0, 64.0)),
        ("mixed-sign-finite", (-0.75, 1.25, -1.5), (2.5, -3.5, 4.5, -5.5)),
    ]
}

pub fn mth_integer_lerp() -> Vec<(&'static str, f64, i64, i64)> {
    vec![
        ("basic", 0.25, 0, 8),
        ("zero-alpha", 0.0, 5, 17),
        ("one-alpha", 1.0, -4, 12),
        ("over-one", 1.5, 2, 10),
        ("negative-extrapolation", -0.5, 2, 10),
        ("negative-range", 0.25, -12, -4),
        ("signed-zero-alpha", -0.0, 5, 17),
        ("subnormal-alpha", 1.401298464324817e-45, -7, 19),
        ("nan-alpha", f64::NAN, 7, 23),
        ("positive-infinity-alpha", f64::INFINITY, 1, 3),
        ("negative-infinity-alpha", f64::NEG_INFINITY, 1, 3),
        ("delta-overflow-half", 0.5, 2147483647, -2147483648),
        ("delta-min-half", 0.5, 0, -2147483648),
        ("start-overflow-one", 1.0, 2147483647, -2147483648),
    ]
}

pub fn mth_smoothstep() -> Vec<(&'static str, f64)> {
    vec![
        ("zero", 0.0),
        ("one", 1.0),
        ("midpoint", 0.5),
        ("quarter", 0.25),
        ("negative", -0.5),
        ("over-one", 1.5),
        ("signed-zero", -0.0),
        ("subnormal", 5e-324),
        ("maximum-finite", 1.7976931348623157e+308),
        ("positive-infinity", f64::INFINITY),
        ("negative-infinity", f64::NEG_INFINITY),
        ("nan", f64::NAN),
        ("below-one", 0.9999999999999999),
        ("above-one", 1.0000000000000002),
    ]
}

pub fn mth_sign() -> Vec<(&'static str, f64)> {
    vec![
        ("negative-one", -1.0),
        ("negative-quarter", -0.25),
        ("negative-zero", -0.0),
        ("positive-zero", 0.0),
        ("positive-quarter", 0.25),
        ("positive-one", 1.0),
        ("negative-subnormal", -5e-324),
        ("positive-subnormal", 5e-324),
        ("negative-maximum-finite", -1.7976931348623157e+308),
        ("positive-maximum-finite", 1.7976931348623157e+308),
        ("negative-infinity", f64::NEG_INFINITY),
        ("positive-infinity", f64::INFINITY),
        ("nan", f64::NAN),
        ("negative-small", -1e-300),
    ]
}

pub fn mth_rot_lerp_rad() -> Vec<(&'static str, f64, f64, f64)> {
    vec![
        ("basic", 0.25, 0.0, 1.0),
        ("zero-alpha", -0.0, 0.5, 1.0),
        ("one-alpha", 1.0, 0.5, 1.0),
        ("negative-alpha", -0.5, 0.5, 1.0),
        ("over-one-alpha", 1.5, 0.5, 1.0),
        ("positive-crossing", 0.5, 3.0, -3.0),
        ("negative-crossing", 0.5, -3.0, 3.0),
        ("positive-pi-boundary", 1.0, 0.0, 3.141592653589793),
        ("negative-pi-boundary", 1.0, 0.0, -3.141592653589793),
        ("full-turn", 0.5, 0.0, 6.283185307179586),
        ("signed-zero", 0.5, -0.0, 0.0),
        ("subnormal-difference", 1.0, 0.0, 1.401298464324817e-45),
        ("large-finite", 0.5, -100.0, 100.0),
        ("nan-alpha", f64::NAN, 0.0, 1.0),
        ("nan-to", 0.5, 0.0, f64::NAN),
    ]
}

pub fn mth_triangle_wave() -> Vec<(&'static str, f64, f64)> {
    vec![
        ("basic", 0.0, 4.0),
        ("quarter", 1.0, 4.0),
        ("midpoint", 2.0, 4.0),
        ("three-quarter", 3.0, 4.0),
        ("period-boundary", 4.0, 4.0),
        ("negative-index", -1.0, 4.0),
        ("negative-period", 2.0, -4.0),
        ("fractional-period", 0.75, 3.0),
        ("negative-fractional", -0.75, 3.0),
        ("signed-zero-index", -0.0, 4.0),
        ("subnormal-period", 1.0, 1.401298464324817e-45),
        ("maximum-finite-index", 3.4028234663852886e+38, 3.0),
        ("zero-period", 1.0, 0.0),
        ("infinite-index", f64::INFINITY, 4.0),
        ("nan-index", f64::NAN, 4.0),
    ]
}

pub fn mth_square_f32() -> Vec<(&'static str, f64)> {
    vec![
        ("basic", 3.0),
        ("zero", 0.0),
        ("signed-zero", -0.0),
        ("negative-one", -1.0),
        ("negative-fractional", -1.5),
        ("fractional", 1.5),
        ("subnormal-input", 1.401298464324817e-45),
        ("subnormal-result", 5.293955920339377e-23),
        ("large-finite", 1.8446744073709552e+19),
        ("maximum-finite", 3.4028234663852886e+38),
        ("negative-maximum-finite", -3.4028234663852886e+38),
        ("positive-infinity", f64::INFINITY),
        ("negative-infinity", f64::NEG_INFINITY),
        ("nan", f64::NAN),
        ("rounding-boundary", 1.0000001192092896),
    ]
}

pub fn mth_cube_f32() -> Vec<(&'static str, f64)> {
    vec![
        ("basic", 2.0),
        ("zero", 0.0),
        ("signed-zero", -0.0),
        ("negative-one", -1.0),
        ("fractional", 1.5),
        ("negative-fractional", -1.5),
        ("subnormal-input", 1.401298464324817e-45),
        ("subnormal-result", 1.7763568394002505e-15),
        ("large-finite", 4398046511104.0),
        ("overflow-finite", 8796093022208.0),
        ("maximum-finite", 3.4028234663852886e+38),
        ("negative-maximum-finite", -3.4028234663852886e+38),
        ("positive-infinity", f64::INFINITY),
        ("negative-infinity", f64::NEG_INFINITY),
        ("nan", f64::NAN),
    ]
}

pub fn mth_square_f64() -> Vec<(&'static str, f64)> {
    vec![
        ("basic", 3.0),
        ("zero", 0.0),
        ("signed-zero", -0.0),
        ("negative-fractional", -1.5),
        ("fractional", 1.5),
        ("minimum-positive-subnormal", 5e-324),
        ("subnormal-result", 2.2227587494850775e-162),
        ("large-finite", 6.703903964971299e+153),
        ("overflow-finite", 1.3407807929942597e+154),
        ("maximum-finite", 1.7976931348623157e+308),
        ("negative-maximum-finite", -1.7976931348623157e+308),
        ("positive-infinity", f64::INFINITY),
        ("negative-infinity", f64::NEG_INFINITY),
        ("nan", f64::NAN),
        ("rounding-boundary", 1.0000000000000002),
    ]
}
