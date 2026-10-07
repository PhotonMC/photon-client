"""The `math` family: the `mth/numbers` cases."""

import math
import struct
from cases.math.common import *
from cases.math.shared import *
from cases.math.mth.shared import *


def cases():
    for name in ("pi", "half_pi", "two_pi", "deg_to_rad", "rad_to_deg", "epsilon",
                 "sqrt_of_two", "y_axis", "x_axis", "z_axis"):
        yield {"case_id": f"mth/constants/{name}", "family": "math",
               "operation": "mth_constants", "name": name}

    mth_trigonometric_values = [
        ("zero", float_bits(0.0, "f64")),
        ("negative-zero", "8000000000000000"),
        ("quarter-turn", float_bits(math.pi / 2.0, "f64")),
        ("negative-quarter-turn", float_bits(-math.pi / 2.0, "f64")),
        ("pi", float_bits(math.pi, "f64")),
        ("negative-pi", float_bits(-math.pi, "f64")),
        ("large-finite", float_bits(1.0e300, "f64")),
        ("positive-infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("nan-payload", "7ff8000000000001"),
    ]

    for function in ("sin", "cos"):
        for label, value in mth_trigonometric_values:
            yield {"case_id": f"mth/{function}/{label}", "family": "math",
                   "operation": "mth_floating_primitives", "fn": function,
                   "type": "f64", "value": value}

    mth_sqrt_values = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("one", float_bits(1.0, "f32")),
        ("two", float_bits(2.0, "f32")),
        ("min-subnormal", "00000001"),
        ("max-finite", "7f7fffff"),
        ("negative-one", float_bits(-1.0, "f32")),
        ("positive-infinity", "7f800000"),
        ("negative-infinity", "ff800000"),
        ("nan-payload", "7fc00001"),
    ]

    for label, value in mth_sqrt_values:
        yield {"case_id": f"mth/sqrt/{label}", "family": "math",
               "operation": "mth_floating_primitives", "fn": "sqrt",
               "type": "f32", "value": value}

    mth_abs_values = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("one", float_bits(1.0, "f32")),
        ("negative-one", float_bits(-1.0, "f32")),
        ("min-subnormal", "00000001"),
        ("negative-min-subnormal", "80000001"),
        ("negative-max-finite", "ff7fffff"),
        ("positive-infinity", "7f800000"),
        ("negative-infinity", "ff800000"),
        ("negative-nan-payload", "ffc01234"),
    ]

    for label, value in mth_abs_values:
        yield {"case_id": f"mth/abs-f32/{label}", "family": "math",
               "operation": "mth_floating_primitives", "fn": "abs",
               "type": "f32", "value": value}

    mth_clamp_f32 = [
        ("inside", 0.25, 0.0, 1.0),
        ("below", -1.0, 0.0, 1.0),
        ("above", 2.0, 0.0, 1.0),
        ("min-equal", 0.0, 0.0, 1.0),
        ("max-equal", 1.0, 0.0, 1.0),
        ("reversed-bounds", 0.0, 1.0, -1.0),
        ("negative-zero-value", "80000000", "00000000", "3f800000"),
        ("negative-zero-max", "00000000", "bf800000", "80000000"),
        ("nan-value", "7fc01234", "00000000", "3f800000"),
        ("nan-max", "3f800000", "00000000", "7fc01234"),
    ]

    for label, value, minimum, maximum in mth_clamp_f32:
        yield {"case_id": f"mth/clamp-f32/{label}", "family": "math",
               "operation": "mth_floating_primitives", "fn": "clamp", "type": "f32",
               "value": value if isinstance(value, str) else float_bits(value, "f32"),
               "min": minimum if isinstance(minimum, str) else float_bits(minimum, "f32"),
               "max": maximum if isinstance(maximum, str) else float_bits(maximum, "f32")}

    mth_clamp_f64 = [
        ("inside", 0.25, 0.0, 1.0),
        ("below", -1.0, 0.0, 1.0),
        ("above", 2.0, 0.0, 1.0),
        ("min-equal", 0.0, 0.0, 1.0),
        ("max-equal", 1.0, 0.0, 1.0),
        ("reversed-bounds", 0.0, 1.0, -1.0),
        ("negative-zero-value", "8000000000000000", "0000000000000000", "3ff0000000000000"),
        ("negative-zero-max", "0000000000000000", "bff0000000000000", "8000000000000000"),
        ("nan-value", "7ff8000000001234", "0000000000000000", "3ff0000000000000"),
        ("nan-max", "3ff0000000000000", "0000000000000000", "7ff8000000001234"),
    ]

    for label, value, minimum, maximum in mth_clamp_f64:
        yield {"case_id": f"mth/clamp-f64/{label}", "family": "math",
               "operation": "mth_floating_primitives", "fn": "clamp", "type": "f64",
               "value": value if isinstance(value, str) else float_bits(value, "f64"),
               "min": minimum if isinstance(minimum, str) else float_bits(minimum, "f64"),
               "max": maximum if isinstance(maximum, str) else float_bits(maximum, "f64")}

    mth_power_helpers = [
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

    for label, width, function, input_value in mth_power_helpers:
        yield {"case_id": f"mth/power-helpers/{label}", "family": "math",
               "operation": "mth_power_helpers", "type": width, "fn": function,
               "input": input_value}

    for label, width, input_value in mth_frac_helpers:
        yield {"case_id": f"mth/frac/{label}", "family": "math",
               "operation": "mth_frac", "type": width, "fn": "frac",
               "input": input_value}

    mth_get_int = [
        ("null-zero-default", None, "0"),
        ("null-negative-default", None, "-2147483648"),
        ("empty", "", "7"),
        ("decimal", "42", "7"),
        ("negative", "-42", "7"),
        ("positive-sign", "+42", "7"),
        ("leading-zeroes", "00042", "7"),
        ("negative-zero", "-0", "7"),
        ("minimum", "-2147483648", "7"),
        ("maximum", "2147483647", "-7"),
        ("underflow", "-2147483649", "11"),
        ("overflow", "2147483648", "11"),
        ("positive-overflow", "+2147483648", "11"),
        ("leading-space", " 42", "11"),
        ("trailing-space", "42 ", "11"),
        ("tab-prefix", "\t42", "11"),
        ("internal-space", "4 2", "11"),
        ("decimal-point", "42.0", "11"),
        ("hex-prefix", "0x2a", "11"),
        ("underscore", "1_000", "11"),
        ("plus-only", "+", "11"),
        ("minus-only", "-", "11"),
        ("arabic-indic", "٤٢", "11"),
        ("fullwidth", "４２", "11"),
        ("devanagari", "४२", "11"),
        ("unicode-minus", "−42", "11"),
        ("newline-suffix", "42\n", "11"),
        ("invalid-max-default", "invalid", "2147483647"),
        ("invalid-min-default", "invalid", "-2147483648"),
    ]

    for label, input_value, default in mth_get_int:
        yield {"case_id": f"mth/get-int/{label}", "family": "math",
               "operation": "mth_get_int", "input": input_value, "default": default}

    mth_integer_helpers = [
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
    ]

    for label, width, function, input_value, divisor in mth_integer_helpers:
        yield {"case_id": f"mth/integer-helpers/{label}", "family": "math",
               "operation": "mth_integer_helpers", "type": width, "fn": function,
               "input": input_value, "divisor": divisor}

    mth_integer_basics = [
        ("abs/min", "i32", "abs", "-2147483648", None, None, None),
        ("abs/negative", "i32", "abs", "-1", None, None, None),
        ("abs/zero", "i32", "abs", "0", None, None, None),
        ("abs/positive", "i32", "abs", "1", None, None, None),
        ("abs/max", "i32", "abs", "2147483647", None, None, None),
        ("clamp-i32/below", "i32", "clamp", "-5", "0", "10", None),
        ("clamp-i32/inside", "i32", "clamp", "5", "0", "10", None),
        ("clamp-i32/above", "i32", "clamp", "15", "0", "10", None),
        ("clamp-i32/full-range", "i32", "clamp", "-2147483648", "-2147483648", "2147483647", None),
        ("clamp-i32/reversed-bounds", "i32", "clamp", "5", "10", "-10", None),
        ("clamp-i64/min", "i64", "clamp", "-9223372036854775808", "-9223372036854775808", "9223372036854775807", None),
        ("clamp-i64/below", "i64", "clamp", "-20", "-10", "10", None),
        ("clamp-i64/above", "i64", "clamp", "20", "-10", "10", None),
        ("clamp-i64/reversed-bounds", "i64", "clamp", "5", "10", "-10", None),
        ("multiple/positive", "i32", "is_multiple", "12", None, None, "3"),
        ("multiple/remainder", "i32", "is_multiple", "13", None, None, "3"),
        ("multiple/negative", "i32", "is_multiple", "-12", None, None, "3"),
        ("multiple/min-overflow", "i32", "is_multiple", "-2147483648", None, None, "-1"),
        ("wrap/min", "i32", "wrap_degrees", "-2147483648", None, None, None),
        ("wrap/negative", "i32", "wrap_degrees", "-721", None, None, None),
        ("wrap/exact-negative-turn", "i32", "wrap_degrees", "-360", None, None, None),
        ("wrap/lower-bound", "i32", "wrap_degrees", "-180", None, None, None),
        ("wrap/just-above-lower-bound", "i32", "wrap_degrees", "-179", None, None, None),
        ("wrap/upper-bound", "i32", "wrap_degrees", "180", None, None, None),
        ("wrap/max", "i32", "wrap_degrees", "2147483647", None, None, None),
    ]

    for label, width, function, input_value, min_value, max_value, divisor in mth_integer_basics:
        case = {"case_id": f"mth/integer-basics/{label}", "family": "math",
                "operation": "mth_integer_basics", "type": width, "fn": function,
                "input": input_value}
        if min_value is not None:
            case.update({"min": min_value, "max": max_value})
        if divisor is not None:
            case["divisor"] = divisor
        yield case

    mth_rounding_helpers = [
        ("ceil-f32/zero", "f32", "ceil", float_bits(0.0, "f32")),
        ("ceil-f32/negative-zero", "f32", "ceil", float_bits(-0.0, "f32")),
        ("ceil-f32/positive-fraction", "f32", "ceil", float_bits(1.25, "f32")),
        ("ceil-f32/negative-fraction", "f32", "ceil", float_bits(-1.25, "f32")),
        ("ceil-f32/exact", "f32", "ceil", float_bits(7.0, "f32")),
        ("ceil-f32/lower-bound", "f32", "ceil", float_bits(-2147483648.0, "f32")),
        ("ceil-f32/upper-overflow", "f32", "ceil", float_bits(2147483648.0, "f32")),
        ("ceil-f32/infinity", "f32", "ceil", float_bits(float("inf"), "f32")),
        ("ceil-f32/negative-infinity", "f32", "ceil", float_bits(-float("inf"), "f32")),
        ("ceil-f32/nan", "f32", "ceil", float_bits(float("nan"), "f32")),
        ("ceil-f64/zero", "f64", "ceil", float_bits(0.0, "f64")),
        ("ceil-f64/positive-fraction", "f64", "ceil", float_bits(1.25, "f64")),
        ("ceil-f64/negative-fraction", "f64", "ceil", float_bits(-1.25, "f64")),
        ("ceil-f64/upper-bound", "f64", "ceil", float_bits(2147483647.0, "f64")),
        ("ceil-f64/upper-overflow", "f64", "ceil", float_bits(2147483647.0000001, "f64")),
        ("ceil-f64/lower-bound", "f64", "ceil", float_bits(-2147483648.0, "f64")),
        ("ceil-f64/lower-overflow", "f64", "ceil", float_bits(-2147483648.0000001, "f64")),
        ("ceil-f64/infinity", "f64", "ceil", float_bits(float("inf"), "f64")),
        ("ceil-f64/negative-infinity", "f64", "ceil", float_bits(-float("inf"), "f64")),
        ("ceil-f64/nan", "f64", "ceil", float_bits(float("nan"), "f64")),
        ("lfloor/zero", "f64", "lfloor", float_bits(0.0, "f64")),
        ("lfloor/positive-fraction", "f64", "lfloor", float_bits(1.25, "f64")),
        ("lfloor/negative-fraction", "f64", "lfloor", float_bits(-1.25, "f64")),
        ("lfloor/positive-long-overflow", "f64", "lfloor", float_bits(2.0 ** 63, "f64")),
        ("lfloor/negative-long-bound", "f64", "lfloor", float_bits(-(2.0 ** 63), "f64")),
        ("lfloor/infinity", "f64", "lfloor", float_bits(float("inf"), "f64")),
        ("lfloor/negative-infinity", "f64", "lfloor", float_bits(-float("inf"), "f64")),
        ("lfloor/nan", "f64", "lfloor", float_bits(float("nan"), "f64")),
        ("ceil-long/zero", "f64", "ceil_long", float_bits(0.0, "f64")),
        ("ceil-long/positive-fraction", "f64", "ceil_long", float_bits(1.25, "f64")),
        ("ceil-long/negative-fraction", "f64", "ceil_long", float_bits(-1.25, "f64")),
        ("ceil-long/positive-long-overflow", "f64", "ceil_long", float_bits(2.0 ** 63, "f64")),
        ("ceil-long/negative-long-bound", "f64", "ceil_long", float_bits(-(2.0 ** 63), "f64")),
        ("ceil-long/infinity", "f64", "ceil_long", float_bits(float("inf"), "f64")),
        ("ceil-long/negative-infinity", "f64", "ceil_long", float_bits(-float("inf"), "f64")),
        ("ceil-long/nan", "f64", "ceil_long", float_bits(float("nan"), "f64")),
    ]

    for label, width, function, input_value in mth_rounding_helpers:
        yield {"case_id": f"mth/rounding-helpers/{label}", "family": "math",
               "operation": "mth_rounding_helpers", "type": width, "fn": function,
               "input": input_value}

    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_positive_modulo = [
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
            ("positive-infinity-input", float("inf"), 3.0),
            ("negative-infinity-input", -float("inf"), 3.0),
            ("infinite-modulus", 1.0, float("inf")),
            ("negative-input-infinite-modulus", -1.0, float("inf")),
            ("negative-infinite-modulus", 1.0, -float("inf")),
            ("negative-input-negative-infinite-modulus", -1.0, -float("inf")),
            ("nan-input", float("nan"), 3.0),
            ("nan-input-negative-modulus", float("nan"), -3.0),
            ("nan-modulus", 1.0, float("nan")),
            ("negative-zero-input-zero-modulus", -0.0, 0.0),
        ]
        for label, input_value, modulus in mth_positive_modulo:
            yield {"case_id": f"mth/positive-modulo/{width}/{label}", "family": "math",
                   "operation": "mth_positive_modulo", "type": width,
                   "input": float_bits(input_value, width), "mod": float_bits(modulus, width)}

    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_abs_max = [
            ("basic", -3.0, 2.0),
            ("second-larger", -2.0, 4.5),
            ("both-negative", -7.25, -4.5),
            ("same-magnitude", -3.0, 3.0),
            ("zero", 0.0, 0.0),
            ("signed-zero", -0.0, 0.0),
            ("subnormal", subnormal, 0.0),
            ("maximum-finite", -maximum_finite, maximum_finite),
            ("positive-infinity", float("inf"), 3.0),
            ("negative-infinity", -float("inf"), -2.0),
            ("nan-first", float("nan"), 3.0),
            ("negative-nan-first", -float("nan"), 3.0),
            ("nan-second", 3.0, float("nan")),
            ("negative-nan-second", 3.0, -float("nan")),
        ]
        for label, first, second in mth_abs_max:
            yield {"case_id": f"mth/abs-max/{width}/{label}", "family": "math",
                   "operation": "mth_abs_max", "type": width,
                   "first": float_bits(first, width), "second": float_bits(second, width)}

    equal_threshold = struct.unpack(">f", struct.pack(">f", 1.0e-5))[0]

    for width in ("f32", "f64"):
        threshold = equal_threshold
        mth_equal = [
            ("zero", 0.0, 0.0),
            ("signed-zero", -0.0, 0.0),
            ("same-finite", 1.25, 1.25),
            ("within-threshold", 0.0, threshold * 0.5),
            ("negative-within-threshold", 0.0, -threshold * 0.5),
            ("threshold", 0.0, threshold),
            ("above-threshold", 0.0, threshold * 2.0),
            ("subnormal", 0.0, (float.fromhex("0x0.000002p-126") if width == "f32"
                                else float.fromhex("0x0.0000000000001p-1022"))),
            ("large-difference", 1.0e20, 1.0e20 + 1.0e6),
            ("positive-infinity", float("inf"), float("inf")),
            ("negative-infinity", -float("inf"), -float("inf")),
            ("opposite-infinity", float("inf"), -float("inf")),
            ("nan-first", float("nan"), 1.0),
            ("nan-second", 1.0, float("nan")),
        ]
        for label, first, second in mth_equal:
            yield {"case_id": f"mth/equal/{width}/{label}", "family": "math",
                   "operation": "mth_equal", "type": width,
                   "first": float_bits(first, width), "second": float_bits(second, width)}

    for label, width, function, values in mth_lengths:
        precision = "f32" if width.startswith("f32") else "f64"
        yield {"case_id": f"mth/length/{label}", "family": "math",
               "operation": "mth_length", "type": width, "fn": function,
               "x": float_bits(values[0], precision), "y": float_bits(values[1], precision),
               **({"z": float_bits(values[2], precision)} if len(values) == 3 else {})}

    mth_out_from_origin = [
        ("default-symmetric", {"mode": "default", "origin": "0", "lower": "-3", "upper": "3"}),
        ("default-negative-symmetric", {"mode": "default", "origin": "-1", "lower": "-4", "upper": "2"}),
        ("default-singleton", {"mode": "default", "origin": "10", "lower": "4", "upper": "4"}),
        ("default-origin-below", {"mode": "default", "origin": "-20", "lower": "2", "upper": "6"}),
        ("default-origin-above", {"mode": "default", "origin": "20", "lower": "-6", "upper": "-2"}),
        ("default-origin-at-lower", {"mode": "default", "origin": "-3", "lower": "-3", "upper": "4"}),
        ("default-origin-at-upper", {"mode": "default", "origin": "5", "lower": "-4", "upper": "5"}),
        ("default-left-asymmetric", {"mode": "default", "origin": "-1", "lower": "-5", "upper": "1"}),
        ("default-right-asymmetric", {"mode": "default", "origin": "1", "lower": "-1", "upper": "5"}),
        ("default-i32-origin-min", {"mode": "default", "origin": str(-(1 << 31)), "lower": "-2", "upper": "2"}),
        ("default-i32-origin-max", {"mode": "default", "origin": str((1 << 31) - 1), "lower": "-2", "upper": "2"}),
        ("step-two-symmetric", {"mode": "step", "origin": "0", "lower": "-6", "upper": "6", "step": "2"}),
        ("step-two-offset", {"mode": "step", "origin": "1", "lower": "-5", "upper": "7", "step": "2"}),
        ("step-three-asymmetric", {"mode": "step", "origin": "2", "lower": "-7", "upper": "8", "step": "3"}),
        ("step-exact-upper", {"mode": "step", "origin": "0", "lower": "-4", "upper": "4", "step": "4"}),
        ("step-exceeds-range", {"mode": "step", "origin": "0", "lower": "-2", "upper": "2", "step": "5"}),
        ("step-clamped-below", {"mode": "step", "origin": "-20", "lower": "2", "upper": "9", "step": "3"}),
        ("step-clamped-above", {"mode": "step", "origin": "20", "lower": "-9", "upper": "2", "step": "3"}),
        ("step-singleton", {"mode": "step", "origin": "0", "lower": "7", "upper": "7", "step": "1"}),
        ("step-i32-max", {"mode": "step", "origin": "0", "lower": "-1", "upper": "1", "step": str((1 << 31) - 1)}),
        ("invalid-reversed", {"mode": "default", "origin": "0", "lower": "4", "upper": "-4"}),
        ("invalid-zero-step", {"mode": "step", "origin": "0", "lower": "-1", "upper": "1", "step": "0"}),
        ("invalid-negative-step", {"mode": "step", "origin": "0", "lower": "-1", "upper": "1", "step": "-2"}),
        ("invalid-both", {"mode": "step", "origin": "0", "lower": "2", "upper": "-2", "step": "0"}),
    ]

    for label, values in mth_out_from_origin:
        yield {"case_id": f"mth/out-from-origin/{label}", "family": "math",
               "operation": "mth_out_from_origin", **values}

    mth_mul_and_truncate = [
        ("zero-numerator", "0", "7", "94"),
        ("unit", "1", "1", "12"),
        ("positive-truncate", "1", "3", "10"),
        ("negative-truncate", "-1", "3", "10"),
        ("negative-factor", "3", "2", "-5"),
        ("both-negative", "-3", "2", "-5"),
        ("non-reduced", "6", "4", "10"),
        ("zero-factor", "123", "7", "0"),
        ("denominator-one-max-numerator", str((1 << 31) - 1), "1", "1"),
        ("numerator-max-overflow", str((1 << 31) - 1), "1", "2"),
        ("numerator-min-overflow", str(-(1 << 31)), "1", "2"),
        ("min-times-negative-one", str(-(1 << 31)), "1", "-1"),
        ("factor-min", "1", "1", str(-(1 << 31))),
        ("product-to-min", str(1 << 30), "1", "2"),
        ("large-denominator", str((1 << 31) - 1), str((1 << 31) - 1), str((1 << 31) - 1)),
        ("negative-numerator-large", str(-(1 << 31)), str((1 << 31) - 1), str((1 << 31) - 1)),
        ("fractional-negative-result", "5", "3", str(-(1 << 31))),
        ("factor-maximum", "3", "7", str((1 << 31) - 1)),
        ("denominator-two", str((1 << 31) - 1), "2", "3"),
        ("mixed-overflow", "123456789", "97", "-987654321"),
    ]

    for label, numerator, denominator, factor in mth_mul_and_truncate:
        yield {"case_id": f"mth/mul-and-truncate/{label}", "family": "math",
               "operation": "mth_mul_and_truncate", "numerator": numerator,
               "denominator": denominator, "factor": factor}

    for label, value in mth_sign:
        yield {"case_id": f"mth/sign/{label}", "family": "math",
               "operation": "mth_sign", "x": float_bits(value, "f64")}

    for label, x in mth_square_f32:
        yield {"case_id": f"mth/square-f32/{label}", "family": "math",
               "operation": "mth_square_f32", "x": float_bits(x, "f32")}

    for label, x in mth_cube_f32:
        yield {"case_id": f"mth/cube-f32/{label}", "family": "math",
               "operation": "mth_cube_f32", "x": float_bits(x, "f32")}

    for label, x in mth_square_f64:
        yield {"case_id": f"mth/square-f64/{label}", "family": "math",
               "operation": "mth_square_f64", "x": float_bits(x, "f64")}

    mth_square_i32 = [
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
    ]

    for label, x in mth_square_i32:
        yield {"case_id": f"mth/square-i32/{label}", "family": "math",
               "operation": "mth_square_i32", "x": str(x)}

    mth_square_i64 = [
        ("basic", 3),
        ("zero", 0),
        ("positive-one", 1),
        ("negative-one", -1),
        ("negative", -3),
        ("exact-long-limit", 3037000499),
        ("positive-overflow", 3037000500),
        ("negative-overflow", -3037000500),
        ("power-of-two-wrap", 4294967296),
        ("negative-power-of-two-wrap", -4294967296),
        ("maximum", 9223372036854775807),
        ("minimum", -9223372036854775808),
        ("maximum-minus-one", 9223372036854775806),
        ("minimum-plus-one", -9223372036854775807),
        ("mixed", 1234567890123456789),
    ]

    for label, x in mth_square_i64:
        yield {"case_id": f"mth/square-i64/{label}", "family": "math",
               "operation": "mth_square_i64", "x": str(x)}
