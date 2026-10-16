//! Cases for `mth/constants`, `mth`, `mth/sqrt`, `mth/abs-f32` and more.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::math::mth::tables::*;
use crate::math::values::*;

fn constants() -> Vec<Value> {
    let mut out = Vec::new();
    for name in vec![
        "pi",
        "half_pi",
        "two_pi",
        "deg_to_rad",
        "rad_to_deg",
        "epsilon",
        "sqrt_of_two",
        "y_axis",
        "x_axis",
        "z_axis",
    ] {
        out.push(case(format!("mth/constants/{name}"), "math", "mth_constants", json!({"name": name})));
    }
    let mth_trigonometric_values = vec![
        ("zero", f64_bits(0.0)),
        ("negative-zero", "8000000000000000".to_string()),
        ("quarter-turn", f64_bits(1.5707963267948966)),
        ("negative-quarter-turn", f64_bits(-1.5707963267948966)),
        ("pi", f64_bits(3.141592653589793)),
        ("negative-pi", f64_bits(-3.141592653589793)),
        ("large-finite", f64_bits(1e+300)),
        ("positive-infinity", "7ff0000000000000".to_string()),
        ("negative-infinity", "fff0000000000000".to_string()),
        ("nan-payload", "7ff8000000000001".to_string()),
    ];
    for function in ["sin", "cos"] {
        for (label, value) in &mth_trigonometric_values {
            out.push(case(
                format!("mth/{function}/{label}"),
                "math",
                "mth_floating_primitives",
                json!({"fn": function, "type": "f64", "value": value}),
            ));
        }
    }
    let mth_sqrt_values = vec![
        ("zero", "00000000".to_string()),
        ("negative-zero", "80000000".to_string()),
        ("one", f32_bits(1.0)),
        ("two", f32_bits(2.0)),
        ("min-subnormal", "00000001".to_string()),
        ("max-finite", "7f7fffff".to_string()),
        ("negative-one", f32_bits(-1.0)),
        ("positive-infinity", "7f800000".to_string()),
        ("negative-infinity", "ff800000".to_string()),
        ("nan-payload", "7fc00001".to_string()),
    ];
    for (label, value) in &mth_sqrt_values {
        out.push(case(
            format!("mth/sqrt/{label}"),
            "math",
            "mth_floating_primitives",
            json!({"fn": "sqrt", "type": "f32", "value": value}),
        ));
    }
    out
}

fn abs_f32() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_abs_values = vec![
        ("zero", "00000000".to_string()),
        ("negative-zero", "80000000".to_string()),
        ("one", f32_bits(1.0)),
        ("negative-one", f32_bits(-1.0)),
        ("min-subnormal", "00000001".to_string()),
        ("negative-min-subnormal", "80000001".to_string()),
        ("negative-max-finite", "ff7fffff".to_string()),
        ("positive-infinity", "7f800000".to_string()),
        ("negative-infinity", "ff800000".to_string()),
        ("negative-nan-payload", "ffc01234".to_string()),
    ];
    for (label, value) in &mth_abs_values {
        out.push(case(
            format!("mth/abs-f32/{label}"),
            "math",
            "mth_floating_primitives",
            json!({"fn": "abs", "type": "f32", "value": value}),
        ));
    }
    let mth_clamp_f32 = vec![
        ("inside", num(0.25), num(0.0), num(1.0)),
        ("below", num(-1.0), num(0.0), num(1.0)),
        ("above", num(2.0), num(0.0), num(1.0)),
        ("min-equal", num(0.0), num(0.0), num(1.0)),
        ("max-equal", num(1.0), num(0.0), num(1.0)),
        ("reversed-bounds", num(0.0), num(1.0), num(-1.0)),
        ("negative-zero-value", raw("80000000"), raw("00000000"), raw("3f800000")),
        ("negative-zero-max", raw("00000000"), raw("bf800000"), raw("80000000")),
        ("nan-value", raw("7fc01234"), raw("00000000"), raw("3f800000")),
        ("nan-max", raw("3f800000"), raw("00000000"), raw("7fc01234")),
    ];
    for (label, value, minimum, maximum) in &mth_clamp_f32 {
        out.push(case(
            format!("mth/clamp-f32/{label}"),
            "math",
            "mth_floating_primitives",
            json!({"fn": "clamp", "type": "f32", "value": value.f32(), "min": minimum.f32(), "max": maximum.f32()}),
        ));
    }
    out
}

fn clamp_f64() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_clamp_f64 = vec![
        ("inside", num(0.25), num(0.0), num(1.0)),
        ("below", num(-1.0), num(0.0), num(1.0)),
        ("above", num(2.0), num(0.0), num(1.0)),
        ("min-equal", num(0.0), num(0.0), num(1.0)),
        ("max-equal", num(1.0), num(0.0), num(1.0)),
        ("reversed-bounds", num(0.0), num(1.0), num(-1.0)),
        ("negative-zero-value", raw("8000000000000000"), raw("0000000000000000"), raw("3ff0000000000000")),
        ("negative-zero-max", raw("0000000000000000"), raw("bff0000000000000"), raw("8000000000000000")),
        ("nan-value", raw("7ff8000000001234"), raw("0000000000000000"), raw("3ff0000000000000")),
        ("nan-max", raw("3ff0000000000000"), raw("0000000000000000"), raw("7ff8000000001234")),
    ];
    for (label, value, minimum, maximum) in &mth_clamp_f64 {
        out.push(case(
            format!("mth/clamp-f64/{label}"),
            "math",
            "mth_floating_primitives",
            json!({"fn": "clamp", "type": "f64", "value": value.f64(), "min": minimum.f64(), "max": maximum.f64()}),
        ));
    }
    out
}

fn power_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_power_helpers = mth_power_helpers_rows();
    for (label, width, function, input_value) in &mth_power_helpers {
        out.push(case(
            format!("mth/power-helpers/{label}"),
            "math",
            "mth_power_helpers",
            json!({"type": width, "fn": function, "input": input_value}),
        ));
    }
    for (label, width, input_value) in &mth_frac_helpers() {
        out.push(case(
            format!("mth/frac/{label}"),
            "math",
            "mth_frac",
            json!({"type": width, "fn": "frac", "input": input_value}),
        ));
    }
    out
}

fn mth_power_helpers_rows() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    [mth_power_helpers_smallest_power_zero(), mth_power_helpers_ceillog2_five()].into_iter().flatten().collect()
}

fn mth_power_helpers_smallest_power_zero() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    vec![
        ("smallest-power/zero", "i32", "smallest_encompassing_power_of_two", "0"),
        ("smallest-power/one", "i32", "smallest_encompassing_power_of_two", "1"),
        ("smallest-power/two", "i32", "smallest_encompassing_power_of_two", "2"),
        ("smallest-power/three", "i32", "smallest_encompassing_power_of_two", "3"),
        ("smallest-power/four", "i32", "smallest_encompassing_power_of_two", "4"),
        ("smallest-power/fifteen", "i32", "smallest_encompassing_power_of_two", "15"),
        ("smallest-power/sixteen", "i32", "smallest_encompassing_power_of_two", "16"),
        ("smallest-power/seventeen", "i32", "smallest_encompassing_power_of_two", "17"),
        ("smallest-power/max", "i32", "smallest_encompassing_power_of_two", "2147483647"),
        ("smallest-power/min", "i32", "smallest_encompassing_power_of_two", "-2147483648"),
        ("smallest-square/zero", "i32", "smallest_square_side", "0"),
        ("smallest-square/one", "i32", "smallest_square_side", "1"),
        ("smallest-square/two", "i32", "smallest_square_side", "2"),
        ("smallest-square/three", "i32", "smallest_square_side", "3"),
        ("smallest-square/four", "i32", "smallest_square_side", "4"),
        ("smallest-square/eight", "i32", "smallest_square_side", "8"),
        ("smallest-square/fifteen", "i32", "smallest_square_side", "15"),
        ("smallest-square/sixteen", "i32", "smallest_square_side", "16"),
        ("smallest-square/seventeen", "i32", "smallest_square_side", "17"),
        ("smallest-square/max", "i32", "smallest_square_side", "2147483647"),
        ("is-power-i32/zero", "i32", "is_power_of_two", "0"),
        ("is-power-i32/one", "i32", "is_power_of_two", "1"),
        ("is-power-i32/two", "i32", "is_power_of_two", "2"),
        ("is-power-i32/three", "i32", "is_power_of_two", "3"),
        ("is-power-i32/four", "i32", "is_power_of_two", "4"),
        ("is-power-i32/five", "i32", "is_power_of_two", "5"),
        ("is-power-i32/high-power", "i32", "is_power_of_two", "1073741824"),
        ("is-power-i32/max", "i32", "is_power_of_two", "2147483647"),
        ("is-power-i32/min", "i32", "is_power_of_two", "-2147483648"),
        ("is-power-i32/negative-one", "i32", "is_power_of_two", "-1"),
        ("is-power-i64/zero", "i64", "is_power_of_two", "0"),
        ("is-power-i64/one", "i64", "is_power_of_two", "1"),
        ("is-power-i64/two", "i64", "is_power_of_two", "2"),
        ("is-power-i64/three", "i64", "is_power_of_two", "3"),
        ("is-power-i64/four", "i64", "is_power_of_two", "4"),
        ("is-power-i64/five", "i64", "is_power_of_two", "5"),
        ("is-power-i64/high-power", "i64", "is_power_of_two", "4611686018427387904"),
        ("is-power-i64/max", "i64", "is_power_of_two", "9223372036854775807"),
        ("is-power-i64/min", "i64", "is_power_of_two", "-9223372036854775808"),
        ("is-power-i64/negative-one", "i64", "is_power_of_two", "-1"),
        ("ceillog2/zero", "i32", "ceillog2", "0"),
        ("ceillog2/one", "i32", "ceillog2", "1"),
        ("ceillog2/two", "i32", "ceillog2", "2"),
        ("ceillog2/three", "i32", "ceillog2", "3"),
        ("ceillog2/four", "i32", "ceillog2", "4"),
    ]
}

fn mth_power_helpers_ceillog2_five() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    vec![
        ("ceillog2/five", "i32", "ceillog2", "5"),
        ("ceillog2/seven", "i32", "ceillog2", "7"),
        ("ceillog2/eight", "i32", "ceillog2", "8"),
        ("ceillog2/high-power", "i32", "ceillog2", "1073741824"),
        ("ceillog2/max", "i32", "ceillog2", "2147483647"),
        ("log2/zero", "i32", "log2", "0"),
        ("log2/one", "i32", "log2", "1"),
        ("log2/two", "i32", "log2", "2"),
        ("log2/three", "i32", "log2", "3"),
        ("log2/four", "i32", "log2", "4"),
        ("log2/five", "i32", "log2", "5"),
        ("log2/seven", "i32", "log2", "7"),
        ("log2/eight", "i32", "log2", "8"),
        ("log2/high-power", "i32", "log2", "1073741824"),
        ("log2/max", "i32", "log2", "2147483647"),
    ]
}

fn get_int() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_get_int = vec![
        ("null-zero-default", None, "0"),
        ("null-negative-default", None, "-2147483648"),
        ("empty", Some(""), "7"),
        ("decimal", Some("42"), "7"),
        ("negative", Some("-42"), "7"),
        ("positive-sign", Some("+42"), "7"),
        ("leading-zeroes", Some("00042"), "7"),
        ("negative-zero", Some("-0"), "7"),
        ("minimum", Some("-2147483648"), "7"),
        ("maximum", Some("2147483647"), "-7"),
        ("underflow", Some("-2147483649"), "11"),
        ("overflow", Some("2147483648"), "11"),
        ("positive-overflow", Some("+2147483648"), "11"),
        ("leading-space", Some(" 42"), "11"),
        ("trailing-space", Some("42 "), "11"),
        ("tab-prefix", Some("\t42"), "11"),
        ("internal-space", Some("4 2"), "11"),
        ("decimal-point", Some("42.0"), "11"),
        ("hex-prefix", Some("0x2a"), "11"),
        ("underscore", Some("1_000"), "11"),
        ("plus-only", Some("+"), "11"),
        ("minus-only", Some("-"), "11"),
        ("arabic-indic", Some("\u{664}\u{662}"), "11"),
        ("fullwidth", Some("\u{ff14}\u{ff12}"), "11"),
        ("devanagari", Some("\u{96a}\u{968}"), "11"),
        ("unicode-minus", Some("\u{2212}42"), "11"),
        ("newline-suffix", Some("42\n"), "11"),
        ("invalid-max-default", Some("invalid"), "2147483647"),
        ("invalid-min-default", Some("invalid"), "-2147483648"),
    ];
    for (label, input_value, default) in &mth_get_int {
        out.push(case(
            format!("mth/get-int/{label}"),
            "math",
            "mth_get_int",
            json!({"input": input_value, "default": default}),
        ));
    }
    out
}

fn integer_helpers() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_integer_helpers = vec![
        ("floor-div/int/negative-remainder", "i32", "floor_div", "-7", "3"),
        ("floor-div/int/negative-divisor", "i32", "floor_div", "7", "-3"),
        ("floor-div/int/min-overflow", "i32", "floor_div", "-2147483648", "-1"),
        ("positive-modulo/int/negative-remainder", "i32", "positive_modulo", "-7", "3"),
        ("positive-modulo/int/negative-divisor", "i32", "positive_modulo", "7", "-3"),
        ("positive-modulo/int/min-overflow", "i32", "positive_modulo", "-2147483648", "-1"),
        ("positive-ceil-div/int/positive-remainder", "i32", "positive_ceil_div", "7", "3"),
        ("positive-ceil-div/int/negative-input", "i32", "positive_ceil_div", "-7", "3"),
        ("positive-ceil-div/int/min-input", "i32", "positive_ceil_div", "-2147483648", "3"),
        ("positive-ceil-div/long/min-input", "i64", "positive_ceil_div", "-9223372036854775808", "3"),
        ("round-toward/int/positive-remainder", "i32", "round_toward", "7", "3"),
        ("round-toward/int/negative-input", "i32", "round_toward", "-7", "3"),
        ("round-toward/int/multiplication-overflow", "i32", "round_toward", "2147483647", "2"),
        ("round-toward/long/negative-input", "i64", "round_toward", "-7", "3"),
        ("round-toward/long/multiplication-overflow", "i64", "round_toward", "9223372036854775807", "2"),
    ];
    for (label, width, function, input_value, divisor) in &mth_integer_helpers {
        out.push(case(
            format!("mth/integer-helpers/{label}"),
            "math",
            "mth_integer_helpers",
            json!({"type": width, "fn": function, "input": input_value, "divisor": divisor}),
        ));
    }
    out
}

fn integer_basics() -> Vec<Value> {
    let mut out = Vec::new();
    let mth_integer_basics = vec![
        ("abs/min", "i32", "abs", "-2147483648", None, None, None),
        ("abs/negative", "i32", "abs", "-1", None, None, None),
        ("abs/zero", "i32", "abs", "0", None, None, None),
        ("abs/positive", "i32", "abs", "1", None, None, None),
        ("abs/max", "i32", "abs", "2147483647", None, None, None),
        ("clamp-i32/below", "i32", "clamp", "-5", Some("0"), Some("10"), None),
        ("clamp-i32/inside", "i32", "clamp", "5", Some("0"), Some("10"), None),
        ("clamp-i32/above", "i32", "clamp", "15", Some("0"), Some("10"), None),
        ("clamp-i32/full-range", "i32", "clamp", "-2147483648", Some("-2147483648"), Some("2147483647"), None),
        ("clamp-i32/reversed-bounds", "i32", "clamp", "5", Some("10"), Some("-10"), None),
        (
            "clamp-i64/min",
            "i64",
            "clamp",
            "-9223372036854775808",
            Some("-9223372036854775808"),
            Some("9223372036854775807"),
            None,
        ),
        ("clamp-i64/below", "i64", "clamp", "-20", Some("-10"), Some("10"), None),
        ("clamp-i64/above", "i64", "clamp", "20", Some("-10"), Some("10"), None),
        ("clamp-i64/reversed-bounds", "i64", "clamp", "5", Some("10"), Some("-10"), None),
        ("multiple/positive", "i32", "is_multiple", "12", None, None, Some("3")),
        ("multiple/remainder", "i32", "is_multiple", "13", None, None, Some("3")),
        ("multiple/negative", "i32", "is_multiple", "-12", None, None, Some("3")),
        ("multiple/min-overflow", "i32", "is_multiple", "-2147483648", None, None, Some("-1")),
        ("wrap/min", "i32", "wrap_degrees", "-2147483648", None, None, None),
        ("wrap/negative", "i32", "wrap_degrees", "-721", None, None, None),
        ("wrap/exact-negative-turn", "i32", "wrap_degrees", "-360", None, None, None),
        ("wrap/lower-bound", "i32", "wrap_degrees", "-180", None, None, None),
        ("wrap/just-above-lower-bound", "i32", "wrap_degrees", "-179", None, None, None),
        ("wrap/upper-bound", "i32", "wrap_degrees", "180", None, None, None),
        ("wrap/max", "i32", "wrap_degrees", "2147483647", None, None, None),
    ];
    for (label, width, function, input_value, min_value, max_value, divisor) in mth_integer_basics {
        let mut fields = json!({"type": width, "fn": function, "input": input_value});
        if min_value.is_some() {
            fields["min"] = json!(min_value);
            fields["max"] = json!(max_value);
        }
        if divisor.is_some() {
            fields["divisor"] = json!(divisor);
        }
        out.push(case(format!("mth/integer-basics/{label}"), "math", "mth_integer_basics", fields));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [constants(), abs_f32(), clamp_f64(), power_helpers(), get_int(), integer_helpers(), integer_basics()].concat()
}
