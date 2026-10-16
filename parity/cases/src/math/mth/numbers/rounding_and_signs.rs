//! Cases for `mth/rounding-helpers`, `mth/positive-modulo`, `mth/abs-max`, `mth/equal` and more.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::mth::tables::*;
use crate::math::values::*;

fn rounding_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_rounding_helpers = mth_rounding_helpers_rows();
    for (label, width, function, input_value) in &mth_rounding_helpers {
        out.push(case(
            format!("mth/rounding-helpers/{label}"),
            "math",
            "mth_rounding_helpers",
            json!({"type": width, "fn": function, "input": input_value}),
        ));
    }
    for width in ["f32", "f64"] {
        let subnormal = if width == "f32" { 1.401298464324817e-45 } else { 5e-324 };
        let maximum_finite = if width == "f32" { 3.4028234663852886e+38 } else { 1.7976931348623157e+308 };
        let mth_positive_modulo = vec![
            ("basic-positive", 1.25, 3.0),
            ("negative-input", -1.25, 3.0),
            ("input-over-modulus", 5.5, 3.0),
            ("positive-exact-multiple", 6.0, 3.0),
            ("negative-exact-multiple", -6.0, 3.0),
            ("negative-zero-input", -0.0, 3.0),
            ("negative-modulus", 1.25, -3.0),
            ("both-negative", -1.25, -3.0),
            ("subnormal-input", subnormal, 1.0),
            ("maximum-finite-input", maximum_finite, 3.0),
            ("zero-modulus", 1.0, 0.0),
            ("negative-zero-modulus", 1.0, -0.0),
            ("positive-infinity-input", f64::INFINITY, 3.0),
            ("negative-infinity-input", f64::NEG_INFINITY, 3.0),
            ("infinite-modulus", 1.0, f64::INFINITY),
            ("negative-input-infinite-modulus", -1.0, f64::INFINITY),
            ("negative-infinite-modulus", 1.0, f64::NEG_INFINITY),
            ("negative-input-negative-infinite-modulus", -1.0, f64::NEG_INFINITY),
            ("nan-input", f64::NAN, 3.0),
            ("nan-input-negative-modulus", f64::NAN, -3.0),
            ("nan-modulus", 1.0, f64::NAN),
            ("negative-zero-input-zero-modulus", -0.0, 0.0),
        ];
        for (label, input_value, modulus) in mth_positive_modulo.iter().copied() {
            out.push(case(
                format!("mth/positive-modulo/{width}/{label}"),
                "math",
                "mth_positive_modulo",
                json!({"type": width, "input": float_bits(input_value, width), "mod": float_bits(modulus, width)}),
            ));
        }
    }
    out
}

fn mth_rounding_helpers_rows() -> Vec<(&'static str, &'static str, &'static str, std::string::String)> {
    vec![
        ("ceil-f32/zero", "f32", "ceil", f32_bits(0.0)),
        ("ceil-f32/negative-zero", "f32", "ceil", f32_bits(-0.0)),
        ("ceil-f32/positive-fraction", "f32", "ceil", f32_bits(1.25)),
        ("ceil-f32/negative-fraction", "f32", "ceil", f32_bits(-1.25)),
        ("ceil-f32/exact", "f32", "ceil", f32_bits(7.0)),
        ("ceil-f32/lower-bound", "f32", "ceil", f32_bits(-2147483648.0)),
        ("ceil-f32/upper-overflow", "f32", "ceil", f32_bits(2147483648.0)),
        ("ceil-f32/infinity", "f32", "ceil", f32_bits(f64::INFINITY)),
        ("ceil-f32/negative-infinity", "f32", "ceil", f32_bits(f64::NEG_INFINITY)),
        ("ceil-f32/nan", "f32", "ceil", f32_bits(f64::NAN)),
        ("ceil-f64/zero", "f64", "ceil", f64_bits(0.0)),
        ("ceil-f64/positive-fraction", "f64", "ceil", f64_bits(1.25)),
        ("ceil-f64/negative-fraction", "f64", "ceil", f64_bits(-1.25)),
        ("ceil-f64/upper-bound", "f64", "ceil", f64_bits(2147483647.0)),
        ("ceil-f64/upper-overflow", "f64", "ceil", f64_bits(2147483647.0)),
        ("ceil-f64/lower-bound", "f64", "ceil", f64_bits(-2147483648.0)),
        ("ceil-f64/lower-overflow", "f64", "ceil", f64_bits(-2147483648.0)),
        ("ceil-f64/infinity", "f64", "ceil", f64_bits(f64::INFINITY)),
        ("ceil-f64/negative-infinity", "f64", "ceil", f64_bits(f64::NEG_INFINITY)),
        ("ceil-f64/nan", "f64", "ceil", f64_bits(f64::NAN)),
        ("lfloor/zero", "f64", "lfloor", f64_bits(0.0)),
        ("lfloor/positive-fraction", "f64", "lfloor", f64_bits(1.25)),
        ("lfloor/negative-fraction", "f64", "lfloor", f64_bits(-1.25)),
        ("lfloor/positive-long-overflow", "f64", "lfloor", f64_bits(9.223372036854776e+18)),
        ("lfloor/negative-long-bound", "f64", "lfloor", f64_bits(-9.223372036854776e+18)),
        ("lfloor/infinity", "f64", "lfloor", f64_bits(f64::INFINITY)),
        ("lfloor/negative-infinity", "f64", "lfloor", f64_bits(f64::NEG_INFINITY)),
        ("lfloor/nan", "f64", "lfloor", f64_bits(f64::NAN)),
        ("ceil-long/zero", "f64", "ceil_long", f64_bits(0.0)),
        ("ceil-long/positive-fraction", "f64", "ceil_long", f64_bits(1.25)),
        ("ceil-long/negative-fraction", "f64", "ceil_long", f64_bits(-1.25)),
        ("ceil-long/positive-long-overflow", "f64", "ceil_long", f64_bits(9.223372036854776e+18)),
        ("ceil-long/negative-long-bound", "f64", "ceil_long", f64_bits(-9.223372036854776e+18)),
        ("ceil-long/infinity", "f64", "ceil_long", f64_bits(f64::INFINITY)),
        ("ceil-long/negative-infinity", "f64", "ceil_long", f64_bits(f64::NEG_INFINITY)),
        ("ceil-long/nan", "f64", "ceil_long", f64_bits(f64::NAN)),
    ]
}

fn abs_max() -> Vec<Value> {
    let mut out = Vec::new();
    for width in ["f32", "f64"] {
        let subnormal = if width == "f32" { 1.401298464324817e-45 } else { 5e-324 };
        let maximum_finite = if width == "f32" { 3.4028234663852886e+38 } else { 1.7976931348623157e+308 };
        let mth_abs_max = vec![
            ("basic", -3.0, 2.0),
            ("second-larger", -2.0, 4.5),
            ("both-negative", -7.25, -4.5),
            ("same-magnitude", -3.0, 3.0),
            ("zero", 0.0, 0.0),
            ("signed-zero", -0.0, 0.0),
            ("subnormal", subnormal, 0.0),
            ("maximum-finite", -maximum_finite, maximum_finite),
            ("positive-infinity", f64::INFINITY, 3.0),
            ("negative-infinity", f64::NEG_INFINITY, -2.0),
            ("nan-first", f64::NAN, 3.0),
            ("negative-nan-first", -f64::NAN, 3.0),
            ("nan-second", 3.0, f64::NAN),
            ("negative-nan-second", 3.0, -f64::NAN),
        ];
        for (label, first, second) in mth_abs_max.iter().copied() {
            out.push(case(
                format!("mth/abs-max/{width}/{label}"),
                "math",
                "mth_abs_max",
                json!({"type": width, "first": float_bits(first, width), "second": float_bits(second, width)}),
            ));
        }
    }
    out
}

fn equal() -> Vec<Value> {
    let mut out = Vec::new();
    let equal_threshold = 9.999999747378752e-06;
    for width in ["f32", "f64"] {
        let threshold = equal_threshold;
        let mth_equal = vec![
            ("zero", 0.0, 0.0),
            ("signed-zero", -0.0, 0.0),
            ("same-finite", 1.25, 1.25),
            ("within-threshold", 0.0, (threshold * 0.5)),
            ("negative-within-threshold", 0.0, (-threshold * 0.5)),
            ("threshold", 0.0, threshold),
            ("above-threshold", 0.0, (threshold * 2.0)),
            ("subnormal", 0.0, if width == "f32" { 1.401298464324817e-45 } else { 5e-324 }),
            ("large-difference", 1e+20, 1.00000000000001e+20),
            ("positive-infinity", f64::INFINITY, f64::INFINITY),
            ("negative-infinity", f64::NEG_INFINITY, f64::NEG_INFINITY),
            ("opposite-infinity", f64::INFINITY, f64::NEG_INFINITY),
            ("nan-first", f64::NAN, 1.0),
            ("nan-second", 1.0, f64::NAN),
        ];
        for (label, first, second) in mth_equal.iter().copied() {
            out.push(case(
                format!("mth/equal/{width}/{label}"),
                "math",
                "mth_equal",
                json!({"type": width, "first": float_bits(first, width), "second": float_bits(second, width)}),
            ));
        }
    }
    for (label, width, function, values) in mth_lengths() {
        let precision = if width.starts_with("f32") { "f32" } else { "f64" };
        let mut fields = json!({
            "type": width, "fn": function,
            "x": float_bits(values[0], precision), "y": float_bits(values[1], precision),
        });
        if values.len() == 3 {
            fields["z"] = json!(float_bits(values[2], precision));
        }
        out.push(case(format!("mth/length/{label}"), "math", "mth_length", fields));
    }
    out
}

fn out_from_origin() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_out_from_origin = vec![
        ("default-symmetric", json!({"mode": "default", "origin": "0", "lower": "-3", "upper": "3"})),
        ("default-negative-symmetric", json!({"mode": "default", "origin": "-1", "lower": "-4", "upper": "2"})),
        ("default-singleton", json!({"mode": "default", "origin": "10", "lower": "4", "upper": "4"})),
        ("default-origin-below", json!({"mode": "default", "origin": "-20", "lower": "2", "upper": "6"})),
        ("default-origin-above", json!({"mode": "default", "origin": "20", "lower": "-6", "upper": "-2"})),
        ("default-origin-at-lower", json!({"mode": "default", "origin": "-3", "lower": "-3", "upper": "4"})),
        ("default-origin-at-upper", json!({"mode": "default", "origin": "5", "lower": "-4", "upper": "5"})),
        ("default-left-asymmetric", json!({"mode": "default", "origin": "-1", "lower": "-5", "upper": "1"})),
        ("default-right-asymmetric", json!({"mode": "default", "origin": "1", "lower": "-1", "upper": "5"})),
        ("default-i32-origin-min", json!({"mode": "default", "origin": "-2147483648", "lower": "-2", "upper": "2"})),
        ("default-i32-origin-max", json!({"mode": "default", "origin": "2147483647", "lower": "-2", "upper": "2"})),
        ("step-two-symmetric", json!({"mode": "step", "origin": "0", "lower": "-6", "upper": "6", "step": "2"})),
        ("step-two-offset", json!({"mode": "step", "origin": "1", "lower": "-5", "upper": "7", "step": "2"})),
        ("step-three-asymmetric", json!({"mode": "step", "origin": "2", "lower": "-7", "upper": "8", "step": "3"})),
        ("step-exact-upper", json!({"mode": "step", "origin": "0", "lower": "-4", "upper": "4", "step": "4"})),
        ("step-exceeds-range", json!({"mode": "step", "origin": "0", "lower": "-2", "upper": "2", "step": "5"})),
        ("step-clamped-below", json!({"mode": "step", "origin": "-20", "lower": "2", "upper": "9", "step": "3"})),
        ("step-clamped-above", json!({"mode": "step", "origin": "20", "lower": "-9", "upper": "2", "step": "3"})),
        ("step-singleton", json!({"mode": "step", "origin": "0", "lower": "7", "upper": "7", "step": "1"})),
        ("step-i32-max", json!({"mode": "step", "origin": "0", "lower": "-1", "upper": "1", "step": "2147483647"})),
        ("invalid-reversed", json!({"mode": "default", "origin": "0", "lower": "4", "upper": "-4"})),
        ("invalid-zero-step", json!({"mode": "step", "origin": "0", "lower": "-1", "upper": "1", "step": "0"})),
        ("invalid-negative-step", json!({"mode": "step", "origin": "0", "lower": "-1", "upper": "1", "step": "-2"})),
        ("invalid-both", json!({"mode": "step", "origin": "0", "lower": "2", "upper": "-2", "step": "0"})),
    ];
    for (label, values) in &mth_out_from_origin {
        out.push(case(format!("mth/out-from-origin/{label}"), "math", "mth_out_from_origin", values.clone()));
    }
    out
}

fn mul_and_truncate() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_mul_and_truncate = vec![
        ("zero-numerator", "0", "7", "94"),
        ("unit", "1", "1", "12"),
        ("positive-truncate", "1", "3", "10"),
        ("negative-truncate", "-1", "3", "10"),
        ("negative-factor", "3", "2", "-5"),
        ("both-negative", "-3", "2", "-5"),
        ("non-reduced", "6", "4", "10"),
        ("zero-factor", "123", "7", "0"),
        ("denominator-one-max-numerator", "2147483647", "1", "1"),
        ("numerator-max-overflow", "2147483647", "1", "2"),
        ("numerator-min-overflow", "-2147483648", "1", "2"),
        ("min-times-negative-one", "-2147483648", "1", "-1"),
        ("factor-min", "1", "1", "-2147483648"),
        ("product-to-min", "1073741824", "1", "2"),
        ("large-denominator", "2147483647", "2147483647", "2147483647"),
        ("negative-numerator-large", "-2147483648", "2147483647", "2147483647"),
        ("fractional-negative-result", "5", "3", "-2147483648"),
        ("factor-maximum", "3", "7", "2147483647"),
        ("denominator-two", "2147483647", "2", "3"),
        ("mixed-overflow", "123456789", "97", "-987654321"),
    ];
    for (label, numerator, denominator, factor) in &mth_mul_and_truncate {
        out.push(case(
            format!("mth/mul-and-truncate/{label}"),
            "math",
            "mth_mul_and_truncate",
            json!({"numerator": numerator, "denominator": denominator, "factor": factor}),
        ));
    }
    for (label, value) in mth_sign().iter().copied() {
        out.push(case(format!("mth/sign/{label}"), "math", "mth_sign", json!({"x": f64_bits(value)})));
    }
    for (label, x) in mth_square_f32().iter().copied() {
        out.push(case(format!("mth/square-f32/{label}"), "math", "mth_square_f32", json!({"x": f32_bits(x)})));
    }
    for (label, x) in mth_cube_f32().iter().copied() {
        out.push(case(format!("mth/cube-f32/{label}"), "math", "mth_cube_f32", json!({"x": f32_bits(x)})));
    }
    for (label, x) in mth_square_f64().iter().copied() {
        out.push(case(format!("mth/square-f64/{label}"), "math", "mth_square_f64", json!({"x": f64_bits(x)})));
    }
    out
}

fn square_i32() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_square_i32 = vec![
        ("basic", 3),
        ("zero", 0),
        ("positive-one", 1),
        ("negative-one", -1),
        ("negative", -3),
        ("exact-int-limit", 46340),
        ("positive-overflow", 46341),
        ("negative-overflow", -46341),
        ("power-of-two-wrap", 65536),
        ("negative-power-of-two-wrap", -65536),
        ("maximum", 2147483647),
        ("minimum", -2147483648),
        ("maximum-minus-one", 2147483646),
        ("minimum-plus-one", -2147483647),
        ("mixed", 123456789),
    ];
    for (label, x) in &mth_square_i32 {
        out.push(case(format!("mth/square-i32/{label}"), "math", "mth_square_i32", json!({"x": (x).to_string()})));
    }
    let mth_square_i64 = vec![
        ("basic", 3),
        ("zero", 0),
        ("positive-one", 1),
        ("negative-one", -1),
        ("negative", -3),
        ("exact-long-limit", 3037000499_i64),
        ("positive-overflow", 3037000500_i64),
        ("negative-overflow", -3037000500_i64),
        ("power-of-two-wrap", 4294967296_i64),
        ("negative-power-of-two-wrap", -4294967296_i64),
        ("maximum", 9223372036854775807_i64),
        ("minimum", -9223372036854775808_i64),
        ("maximum-minus-one", 9223372036854775806_i64),
        ("minimum-plus-one", -9223372036854775807_i64),
        ("mixed", 1234567890123456789_i64),
    ];
    for (label, x) in &mth_square_i64 {
        out.push(case(format!("mth/square-i64/{label}"), "math", "mth_square_i64", json!({"x": (x).to_string()})));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [rounding_helpers(), abs_max(), equal(), out_from_origin(), mul_and_truncate(), square_i32()].concat()
}
