"""The `math` family: the `mth/angles` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.mth.shared import *


def cases():
    mth_wrap_degrees_90 = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("positive-boundary-below", "4233ffff"),
        ("positive-boundary", "42340000"),
        ("positive-boundary-above", "42340001"),
        ("negative-boundary-above", "c233ffff"),
        ("negative-boundary", "c2340000"),
        ("negative-boundary-below", "c2340001"),
        ("positive-quarter-turn", float_bits(90.0, "f32")),
        ("negative-quarter-turn", float_bits(-90.0, "f32")),
        ("positive-half-turn", float_bits(180.0, "f32")),
        ("negative-half-turn", float_bits(-180.0, "f32")),
        ("wrapped-positive", float_bits(360.25, "f32")),
        ("wrapped-negative", float_bits(-360.25, "f32")),
        ("positive-subnormal", "00000001"),
        ("negative-subnormal", "80000001"),
        ("positive-infinity", "7f800000"),
        ("nan-payload", "7fc12345"),
    ]

    for label, value in mth_wrap_degrees_90:
        yield {"case_id": f"mth/angle-wrap-difference/wrap-degrees-90/{label}", "family": "math",
               "operation": "mth_angle_wrap_difference", "type": "f32",
               "fn": "wrap_degrees_90", "value": value}

    mth_degrees_difference = [
        ("same", float_bits(10.0, "f32"), float_bits(10.0, "f32")),
        ("signed-zero", "80000000", "00000000"),
        ("simple-positive", float_bits(10.0, "f32"), float_bits(40.0, "f32")),
        ("simple-negative", float_bits(40.0, "f32"), float_bits(10.0, "f32")),
        ("upper-bound", float_bits(0.0, "f32"), float_bits(180.0, "f32")),
        ("lower-bound", float_bits(0.0, "f32"), float_bits(-180.0, "f32")),
        ("cross-positive", float_bits(170.0, "f32"), float_bits(-170.0, "f32")),
        ("cross-negative", float_bits(-170.0, "f32"), float_bits(170.0, "f32")),
        ("whole-turn", float_bits(10.0, "f32"), float_bits(370.0, "f32")),
        ("fractional", float_bits(-12.5, "f32"), float_bits(289.25, "f32")),
        ("large-finite", float_bits(1.0e10, "f32"), float_bits(-1.0e10, "f32")),
        ("positive-infinity", float_bits(0.0, "f32"), "7f800000"),
        ("negative-infinity", float_bits(0.0, "f32"), "ff800000"),
        ("nan-from", "7fc12345", float_bits(1.0, "f32")),
        ("nan-to", float_bits(1.0, "f32"), "7fc12345"),
        ("overflow", "ff7fffff", "7f7fffff"),
    ]

    for label, from_value, to_value in mth_degrees_difference:
        yield {"case_id": f"mth/angle-wrap-difference/degrees-difference/{label}", "family": "math",
               "operation": "mth_angle_wrap_difference", "type": "f32",
               "fn": "degrees_difference", "from": from_value, "to": to_value}

    mth_degrees_difference_abs = [
        ("zero", float_bits(0.0, "f32"), float_bits(0.0, "f32")),
        ("negative-zero", "80000000", "00000000"),
        ("simple-positive", float_bits(10.0, "f32"), float_bits(40.0, "f32")),
        ("simple-negative", float_bits(40.0, "f32"), float_bits(10.0, "f32")),
        ("upper-bound", float_bits(0.0, "f32"), float_bits(180.0, "f32")),
        ("lower-bound", float_bits(0.0, "f32"), float_bits(-180.0, "f32")),
        ("cross-positive", float_bits(170.0, "f32"), float_bits(-170.0, "f32")),
        ("cross-negative", float_bits(-170.0, "f32"), float_bits(170.0, "f32")),
        ("whole-turn", float_bits(10.0, "f32"), float_bits(370.0, "f32")),
        ("fractional", float_bits(-12.5, "f32"), float_bits(289.25, "f32")),
        ("negative-result-zero", "80000000", "80000000"),
        ("positive-infinity", float_bits(0.0, "f32"), "7f800000"),
        ("nan-from", "7fc12345", float_bits(1.0, "f32")),
        ("nan-to", float_bits(1.0, "f32"), "7fc12345"),
        ("overflow", "ff7fffff", "7f7fffff"),
        ("subnormal", "00000001", "80000001"),
    ]

    for label, from_value, to_value in mth_degrees_difference_abs:
        yield {"case_id": f"mth/angle-wrap-difference/degrees-difference-abs/{label}", "family": "math",
               "operation": "mth_angle_wrap_difference", "type": "f32",
               "fn": "degrees_difference_abs", "from": from_value, "to": to_value}

    mth_rotate_if_necessary = [
        ("same", 10.0, 10.0, 5.0),
        ("positive-under", 10.0, 12.0, 5.0),
        ("positive-boundary", 10.0, 15.0, 5.0),
        ("positive-over", 10.0, 20.0, 5.0),
        ("negative-under", 20.0, 18.0, 5.0),
        ("negative-boundary", 20.0, 15.0, 5.0),
        ("negative-over", 20.0, 10.0, 5.0),
        ("cross-positive", 170.0, -170.0, 30.0),
        ("cross-negative", -170.0, 170.0, 30.0),
        ("upper-bound", 0.0, 180.0, 45.0),
        ("lower-bound", 0.0, -180.0, 45.0),
        ("zero-limit", 10.0, 80.0, 0.0),
        ("large-limit", 10.0, 80.0, 100.0),
        ("negative-limit", 10.0, 20.0, -5.0),
        ("signed-zero", "80000000", "00000000", 1.0),
        ("positive-infinity-target", 0.0, "7f800000", 30.0),
        ("nan-base", "7fc12345", 0.0, 30.0),
        ("overflow", "7f7fffff", "ff7fffff", "7f7fffff"),
    ]

    for label, base, target, maximum in mth_rotate_if_necessary:
        base = base if isinstance(base, str) else float_bits(base, "f32")
        target = target if isinstance(target, str) else float_bits(target, "f32")
        maximum = maximum if isinstance(maximum, str) else float_bits(maximum, "f32")
        yield {"case_id": f"mth/angle-approach/rotate-if-necessary/{label}", "family": "math",
               "operation": "mth_angle_approach", "type": "f32", "fn": "rotate_if_necessary",
               "base": base, "target": target, "max": maximum}

    mth_approach = [
        ("same", 10.0, 10.0, 2.0),
        ("below", 0.0, 10.0, 2.0),
        ("exact", 0.0, 10.0, 10.0),
        ("over", 0.0, 10.0, 20.0),
        ("above", 10.0, 0.0, 2.0),
        ("negative-increment", 0.0, 10.0, -2.0),
        ("zero-increment", 0.0, 10.0, 0.0),
        ("signed-zero", "80000000", "00000000", 1.0),
        ("positive-subnormal", 0.0, "00000001", 1.0),
        ("negative-subnormal", "80000001", 0.0, 1.0),
        ("max-finite-current", "7f7fffff", 0.0, 1.0),
        ("infinity-increment", 0.0, 10.0, "7f800000"),
        ("infinity-current", "7f800000", 10.0, 1.0),
        ("infinity-target", 0.0, "7f800000", 1.0),
        ("nan-current", "7fc12345", 0.0, 1.0),
        ("nan-target", 0.0, "7fc12345", 1.0),
        ("nan-increment", 0.0, 10.0, "7fc12345"),
        ("rounding", "3f800001", "3f800101", "33800000"),
    ]

    for label, current, target, increment in mth_approach:
        current = current if isinstance(current, str) else float_bits(current, "f32")
        target = target if isinstance(target, str) else float_bits(target, "f32")
        increment = increment if isinstance(increment, str) else float_bits(increment, "f32")
        yield {"case_id": f"mth/angle-approach/approach/{label}", "family": "math",
               "operation": "mth_angle_approach", "type": "f32", "fn": "approach",
               "current": current, "target": target, "increment": increment}

    mth_approach_degrees = [
        ("same", 10.0, 10.0, 5.0),
        ("simple-positive", 10.0, 40.0, 5.0),
        ("simple-negative", 40.0, 10.0, 5.0),
        ("cross-positive", 170.0, -170.0, 5.0),
        ("cross-negative", -170.0, 170.0, 5.0),
        ("upper-bound", 0.0, 180.0, 10.0),
        ("lower-bound", 0.0, -180.0, 10.0),
        ("whole-turn", 10.0, 370.0, 5.0),
        ("fractional", -12.5, 289.25, 7.0),
        ("negative-increment", 0.0, 90.0, -10.0),
        ("zero-increment", 0.0, 90.0, 0.0),
        ("signed-zero", "80000000", "00000000", 1.0),
        ("positive-subnormal", 0.0, "00000001", "00000001"),
        ("large-finite", 1.0e10, -1.0e10, 1.0e5),
        ("infinity-increment", 0.0, 90.0, "7f800000"),
        ("infinity-target", 0.0, "7f800000", 1.0),
        ("nan-current", "7fc12345", 0.0, 1.0),
        ("nan-target", 0.0, "7fc12345", 1.0),
    ]

    for label, current, target, increment in mth_approach_degrees:
        current = current if isinstance(current, str) else float_bits(current, "f32")
        target = target if isinstance(target, str) else float_bits(target, "f32")
        increment = increment if isinstance(increment, str) else float_bits(increment, "f32")
        yield {"case_id": f"mth/angle-approach/approach-degrees/{label}", "family": "math",
               "operation": "mth_angle_approach", "type": "f32", "fn": "approach_degrees",
               "current": current, "target": target, "increment": increment}

    for label, y, x in mth_atan2:
        y = y if isinstance(y, str) else float_bits(y, "f64")
        x = x if isinstance(x, str) else float_bits(x, "f64")
        yield {"case_id": f"mth/atan2/{label}", "family": "math",
               "operation": "mth_atan2", "y": y, "x": x}

    mth_angle_helpers = [
        ("pack/zero", "f32", "pack_degrees", float_bits(0.0, "f32")),
        ("pack/negative-quarter", "f32", "pack_degrees", float_bits(-90.0, "f32")),
        ("pack/negative-half", "f32", "pack_degrees", float_bits(-180.0, "f32")),
        ("pack/quarter", "f32", "pack_degrees", float_bits(90.0, "f32")),
        ("pack/half-byte-wrap", "f32", "pack_degrees", float_bits(180.0, "f32")),
        ("pack/full-byte-wrap", "f32", "pack_degrees", float_bits(360.0, "f32")),
        ("pack/near-full-turn", "f32", "pack_degrees", float_bits(359.9, "f32")),
        ("pack/infinity", "f32", "pack_degrees", float_bits(float("inf"), "f32")),
        ("pack/nan", "f32", "pack_degrees", float_bits(float("nan"), "f32")),
        ("unpack/min", "i8", "unpack_degrees", "-128"),
        ("unpack/negative", "i8", "unpack_degrees", "-1"),
        ("unpack/zero", "i8", "unpack_degrees", "0"),
        ("unpack/positive", "i8", "unpack_degrees", "1"),
        ("unpack/max", "i8", "unpack_degrees", "127"),
        ("wrap-long/min", "i64", "wrap_degrees", "-9223372036854775808"),
        ("wrap-long/negative", "i64", "wrap_degrees", "-721"),
        ("wrap-long/exact-negative-turn", "i64", "wrap_degrees", "-360"),
        ("wrap-long/lower-bound", "i64", "wrap_degrees", "-180"),
        ("wrap-long/upper-bound", "i64", "wrap_degrees", "180"),
        ("wrap-long/exact-turn", "i64", "wrap_degrees", "360"),
        ("wrap-long/max", "i64", "wrap_degrees", "9223372036854775807"),
    ]

    for label, width, function, input_value in mth_angle_helpers:
        yield {"case_id": f"mth/angle-helpers/{label}", "family": "math",
               "operation": "mth_angle_helpers", "type": width, "fn": function,
               "input": input_value}

    for label, axis, rotation in mth_rotation_around_axis:
        yield {"case_id": f"mth/rotation-around-axis/{label}", "family": "math",
               "operation": "mth_rotation_around_axis",
               "axis_x": float_bits(axis[0]), "axis_y": float_bits(axis[1]),
               "axis_z": float_bits(axis[2]), "rotation_x": float_bits(rotation[0]),
               "rotation_y": float_bits(rotation[1]), "rotation_z": float_bits(rotation[2]),
               "rotation_w": float_bits(rotation[3])}
