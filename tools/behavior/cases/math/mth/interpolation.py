"""The `math` family: the `mth/interpolation` cases."""

import math
from cases.math.common import *
from cases.math.shared import *
from cases.math.mth.shared import *


def cases():
    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_clamped_lerp = [
            ("negative-factor", -0.5, -2.5, 4.5),
            ("negative-infinity-factor", -float("inf"), -2.5, 4.5),
            ("negative-zero-factor", -0.0, -0.0, 1.0),
            ("zero-factor", 0.0, -3.25, 7.5),
            ("within-factor", 0.25, -4.0, 12.0),
            ("half-factor", 0.5, -4.0, 12.0),
            ("one-factor", 1.0, -4.0, 12.0),
            ("above-factor", 1.5, -4.0, 12.0),
            ("positive-infinity-factor", float("inf"), -4.0, 12.0),
            ("nan-factor", float("nan"), 1.0, 2.0),
            ("reversed-range", 0.25, 10.0, -10.0),
            ("precision-order", 0.1, 1.0e20, 1.0e20 + 1.0e6),
            ("subnormal-range", 0.5, 0.0, subnormal),
            ("maximum-finite-range", 0.5, -maximum_finite, maximum_finite),
            ("infinite-range", 0.5, -float("inf"), float("inf")),
            ("nan-min-out-of-range", -0.5, float("nan"), 2.0),
        ]
        for label, factor, minimum, maximum in mth_clamped_lerp:
            yield {"case_id": f"mth/clamped-lerp/{width}/{label}", "family": "math",
                   "operation": "mth_clamped_lerp", "type": width,
                   "factor": float_bits(factor, width), "min": float_bits(minimum, width),
                   "max": float_bits(maximum, width)}

    for label, value, resolution in mth_quantize:
        yield {"case_id": f"mth/quantize/{label}", "family": "math",
               "operation": "mth_quantize", "value": float_bits(value, "f64"),
               "resolution": str(resolution)}

    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_inverse_lerp = [
            ("basic", 1.25, 0.0, 5.0),
            ("below", -1.0, 0.0, 5.0),
            ("above", 7.0, 0.0, 5.0),
            ("negative-range", -2.0, -5.0, 5.0),
            ("reverse-bounds", 2.0, 5.0, -5.0),
            ("exact-min", -2.0, -2.0, 6.0),
            ("exact-max", 6.0, -2.0, 6.0),
            ("signed-zero", -0.0, 0.0, 1.0),
            ("zero-range", 1.0, 1.0, 1.0),
            ("subnormal", 2.0 * subnormal, subnormal, 3.0 * subnormal),
            ("maximum-finite-overflow", maximum_finite, -maximum_finite, maximum_finite),
            ("infinite-value", float("inf"), 0.0, 1.0),
            ("nan-value", float("nan"), 0.0, 1.0),
            ("infinite-bound", 1.0, 0.0, float("inf")),
        ]
        for label, value, minimum, maximum in mth_inverse_lerp:
            yield {"case_id": f"mth/inverse-lerp/{width}/{label}", "family": "math",
                   "operation": "mth_inverse_lerp", "type": width,
                   "value": float_bits(value, width), "min": float_bits(minimum, width),
                   "max": float_bits(maximum, width)}

        mth_lerp = [
            ("basic", 0.25, 0.0, 8.0),
            ("negative-extrapolation", -0.5, 2.0, 10.0),
            ("over-one", 1.5, 2.0, 10.0),
            ("negative-range", 0.25, -4.0, 8.0),
            ("reverse-endpoints", 0.25, 8.0, -4.0),
            ("same-endpoints", 0.75, 3.0, 3.0),
            ("alpha-signed-zero", -0.0, 0.0, 1.0),
            ("negative-zero-start", 0.0, -0.0, 1.0),
            ("subnormal", 2.0 * subnormal, subnormal, 3.0 * subnormal),
            ("maximum-finite-difference", 0.5, -maximum_finite, maximum_finite),
            ("infinite-alpha", float("inf"), 0.0, 1.0),
            ("nan-alpha", float("nan"), 0.0, 1.0),
            ("infinite-endpoint", 0.5, 0.0, float("inf")),
            ("nan-endpoint", 0.5, float("nan"), 1.0),
        ]
        for label, alpha, p0, p1 in mth_lerp:
            yield {"case_id": f"mth/lerp/{width}/{label}", "family": "math",
                   "operation": "mth_lerp", "type": width,
                   "alpha": float_bits(alpha, width), "p0": float_bits(p0, width),
                   "p1": float_bits(p1, width)}

    for operation, path_name in (("mth_lerp_int", "lerp-int"),
                                 ("mth_lerp_discrete", "lerp-discrete")):
        for label, alpha, p0, p1 in mth_integer_lerp:
            yield {"case_id": f"mth/{path_name}/{label}", "family": "math",
                   "operation": operation, "alpha": float_bits(alpha, "f32"),
                   "p0": str(p0), "p1": str(p1)}

    for operation, path_name in (("mth_smoothstep", "smoothstep"),
                                 ("mth_smoothstep_derivative", "smoothstep-derivative")):
        for label, value in mth_smoothstep:
            yield {"case_id": f"mth/{path_name}/{label}", "family": "math",
                   "operation": operation, "x": float_bits(value, "f64")}

    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_rot_lerp = [
            ("basic", 0.25, 10.0, 50.0),
            ("zero-alpha", -0.0, 45.0, 90.0),
            ("one-alpha", 1.0, 45.0, 90.0),
            ("negative-alpha", -0.5, 10.0, 50.0),
            ("over-one-alpha", 1.5, 10.0, 50.0),
            ("positive-crossing", 0.5, 170.0, -170.0),
            ("negative-crossing", 0.5, -170.0, 170.0),
            ("positive-half-turn", 1.0, 0.0, 180.0),
            ("negative-half-turn", 1.0, 0.0, -180.0),
            ("full-turn", 0.5, 12.0, 372.0),
            ("signed-zero", 0.5, -0.0, 0.0),
            ("subnormal-difference", 1.0, 0.0, subnormal),
            ("maximum-finite", 0.5, -maximum_finite, maximum_finite),
            ("infinite-to", 0.5, 0.0, float("inf")),
            ("nan-to", 0.5, 0.0, float("nan")),
        ]
        for label, alpha, from_value, to_value in mth_rot_lerp:
            yield {"case_id": f"mth/rot-lerp/{width}/{label}", "family": "math",
                   "operation": "mth_rot_lerp", "type": width,
                   "alpha": float_bits(alpha, width),
                   "from": float_bits(from_value, width),
                   "to": float_bits(to_value, width)}

    for label, alpha, from_value, to_value in mth_rot_lerp_rad:
        yield {"case_id": f"mth/rot-lerp-rad/{label}", "family": "math",
               "operation": "mth_rot_lerp_rad",
               "alpha": float_bits(alpha, "f32"),
               "from": float_bits(from_value, "f32"),
               "to": float_bits(to_value, "f32")}

    for label, index, period in mth_triangle_wave:
        yield {"case_id": f"mth/triangle-wave/{label}", "family": "math",
               "operation": "mth_triangle_wave",
               "index": float_bits(index, "f32"),
               "period": float_bits(period, "f32")}

    min_subnormal_f64 = float.fromhex("0x0.0000000000001p-1022")

    maximum_finite_f64 = float.fromhex("0x1.fffffffffffffp1023")

    mth_clamped_map_f64 = [
        ("basic", 5.0, 0.0, 10.0, 0.0, 100.0),
        ("below-source", -5.0, 0.0, 10.0, 100.0, 200.0),
        ("above-source", 15.0, 0.0, 10.0, 100.0, 200.0),
        ("lower-endpoint", 0.0, 0.0, 10.0, -4.0, 12.0),
        ("upper-endpoint", 10.0, 0.0, 10.0, -4.0, 12.0),
        ("negative-range", -5.0, -10.0, 0.0, 10.0, 30.0),
        ("reversed-source", 5.0, 10.0, 0.0, 0.0, 100.0),
        ("reversed-target", 0.25, 0.0, 1.0, 10.0, -10.0),
        ("reversed-both", 5.0, 10.0, 0.0, 100.0, 0.0),
        ("negative-zero", -0.0, 0.0, 1.0, -0.0, 1.0),
        ("source-subnormal", min_subnormal_f64, 0.0, min_subnormal_f64, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, min_subnormal_f64),
        ("precision-order", 0.1, 1.0e20, 1.0e20 + 1.0e6, -1.0e20, 1.0e20),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, -float("inf"), float("inf"), 0.0, 1.0),
        ("positive-infinity-value", float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", -float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("nan-value", float("nan"), 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, float("nan"), 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -maximum_finite_f64, maximum_finite_f64),
    ]

    for label, value, from_min, from_max, to_min, to_max in mth_clamped_map_f64:
        yield {"case_id": f"mth/clamped-map-f64/{label}", "family": "math",
               "operation": "mth_clamped_map_f64",
               "value": float_bits(value, "f64"), "from_min": float_bits(from_min, "f64"),
               "from_max": float_bits(from_max, "f64"), "to_min": float_bits(to_min, "f64"),
               "to_max": float_bits(to_max, "f64")}

    min_subnormal_f32 = float.fromhex("0x0.000002p-126")

    maximum_finite_f32 = float.fromhex("0x1.fffffep127")

    mth_clamped_map_f32 = [
        ("basic", 5.0, 0.0, 10.0, 0.0, 100.0),
        ("below-source", -5.0, 0.0, 10.0, 100.0, 200.0),
        ("above-source", 15.0, 0.0, 10.0, 100.0, 200.0),
        ("lower-endpoint", 0.0, 0.0, 10.0, -4.0, 12.0),
        ("upper-endpoint", 10.0, 0.0, 10.0, -4.0, 12.0),
        ("negative-range", -5.0, -10.0, 0.0, 10.0, 30.0),
        ("reversed-source", 5.0, 10.0, 0.0, 0.0, 100.0),
        ("reversed-target", 0.25, 0.0, 1.0, 10.0, -10.0),
        ("reversed-both", 5.0, 10.0, 0.0, 100.0, 0.0),
        ("negative-zero", -0.0, 0.0, 1.0, -0.0, 1.0),
        ("source-subnormal", min_subnormal_f32, 0.0, min_subnormal_f32, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, min_subnormal_f32),
        ("precision-order", 0.1, 1.0e10, 1.0e10 + 1.0e6, -1.0e10, 1.0e10),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, -float("inf"), float("inf"), 0.0, 1.0),
        ("positive-infinity-value", float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", -float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("nan-value", float("nan"), 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, float("nan"), 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -maximum_finite_f32, maximum_finite_f32),
    ]

    for label, value, from_min, from_max, to_min, to_max in mth_clamped_map_f32:
        yield {"case_id": f"mth/clamped-map-f32/{label}", "family": "math",
               "operation": "mth_clamped_map_f32",
               "value": float_bits(value, "f32"), "from_min": float_bits(from_min, "f32"),
               "from_max": float_bits(from_max, "f32"), "to_min": float_bits(to_min, "f32"),
               "to_max": float_bits(to_max, "f32")}

    mth_map_f64 = [
        ("basic", 5.0, 0.0, 10.0, 0.0, 100.0),
        ("below-source", -5.0, 0.0, 10.0, 100.0, 200.0),
        ("above-source", 15.0, 0.0, 10.0, 100.0, 200.0),
        ("lower-endpoint", 0.0, 0.0, 10.0, -4.0, 12.0),
        ("upper-endpoint", 10.0, 0.0, 10.0, -4.0, 12.0),
        ("negative-range", -5.0, -10.0, 0.0, 10.0, 30.0),
        ("reversed-source", 5.0, 10.0, 0.0, 0.0, 100.0),
        ("reversed-target", 0.25, 0.0, 1.0, 10.0, -10.0),
        ("reversed-both", 5.0, 10.0, 0.0, 100.0, 0.0),
        ("negative-zero", -0.0, 0.0, 1.0, -0.0, 1.0),
        ("source-subnormal", min_subnormal_f64, 0.0, min_subnormal_f64, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, min_subnormal_f64),
        ("precision-order", 0.1, 1.0e20, 1.0e20 + 1.0e6, -1.0e20, 1.0e20),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, -float("inf"), float("inf"), 0.0, 1.0),
        ("positive-infinity-value", float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", -float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("nan-value", float("nan"), 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, float("nan"), 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -maximum_finite_f64, maximum_finite_f64),
    ]

    for label, value, from_min, from_max, to_min, to_max in mth_map_f64:
        yield {"case_id": f"mth/map-f64/{label}", "family": "math",
               "operation": "mth_map_f64",
               "value": float_bits(value, "f64"), "from_min": float_bits(from_min, "f64"),
               "from_max": float_bits(from_max, "f64"), "to_min": float_bits(to_min, "f64"),
               "to_max": float_bits(to_max, "f64")}

    mth_map_f32 = [
        ("basic", 5.0, 0.0, 10.0, 0.0, 100.0),
        ("below-source", -5.0, 0.0, 10.0, 100.0, 200.0),
        ("above-source", 15.0, 0.0, 10.0, 100.0, 200.0),
        ("lower-endpoint", 0.0, 0.0, 10.0, -4.0, 12.0),
        ("upper-endpoint", 10.0, 0.0, 10.0, -4.0, 12.0),
        ("negative-range", -5.0, -10.0, 0.0, 10.0, 30.0),
        ("reversed-source", 5.0, 10.0, 0.0, 0.0, 100.0),
        ("reversed-target", 0.25, 0.0, 1.0, 10.0, -10.0),
        ("reversed-both", 5.0, 10.0, 0.0, 100.0, 0.0),
        ("negative-zero", -0.0, 0.0, 1.0, -0.0, 1.0),
        ("source-subnormal", min_subnormal_f32, 0.0, min_subnormal_f32, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, min_subnormal_f32),
        ("precision-order", 0.1, 1.0e10, 1.0e10 + 1.0e6, -1.0e10, 1.0e10),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, -float("inf"), float("inf"), 0.0, 1.0),
        ("positive-infinity-value", float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", -float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("nan-value", float("nan"), 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, float("nan"), 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -maximum_finite_f32, maximum_finite_f32),
    ]

    for label, value, from_min, from_max, to_min, to_max in mth_map_f32:
        yield {"case_id": f"mth/map-f32/{label}", "family": "math",
               "operation": "mth_map_f32",
               "value": float_bits(value, "f32"), "from_min": float_bits(from_min, "f32"),
               "from_max": float_bits(from_max, "f32"), "to_min": float_bits(to_min, "f32"),
               "to_max": float_bits(to_max, "f32")}

    maximum_finite_f64 = float.fromhex("0x1.fffffffffffffp1023")

    wobble_coords = [
        ("zero", float_bits(0.0, "f64")),
        ("negative-zero", "8000000000000000"),
        ("positive-integer", float_bits(1.0, "f64")),
        ("negative-integer", float_bits(-1.0, "f64")),
        ("positive-fraction", float_bits(0.25, "f64")),
        ("negative-fraction", float_bits(-0.25, "f64")),
        ("positive-boundary", float_bits(1.0 / 3000.0, "f64")),
        ("negative-boundary", float_bits(-1.0 / 3000.0, "f64")),
        ("below-positive-boundary", float_bits(math.nextafter(1.0 / 3000.0, 0.0), "f64")),
        ("above-positive-boundary", float_bits(math.nextafter(1.0 / 3000.0, math.inf), "f64")),
        ("large-positive", float_bits(1_000_000.25, "f64")),
        ("large-negative", float_bits(-1_000_000.25, "f64")),
        ("positive-floor-overflow", float_bits(715_827.883, "f64")),
        ("negative-floor-overflow", float_bits(-715_827.883, "f64")),
        ("maximum-finite", float_bits(maximum_finite_f64, "f64")),
        ("positive-subnormal", "0000000000000001"),
        ("negative-subnormal", "8000000000000001"),
        ("positive-infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("negative-nan-payload", "fff8000000000001"),
    ]

    for label, coord in wobble_coords:
        yield {"case_id": f"mth/wobble-f64/{label}", "family": "math",
               "operation": "mth_wobble_f64", "coord": coord}

    def interpolation_bits(value, width):
        return value if isinstance(value, str) else float_bits(value, width)

    mth_lerp2 = [
        ("basic", 0.25, 0.5, 0.0, 10.0, 20.0, 30.0),
        ("zero-alpha", 0.0, 0.0, 1.0, 2.0, 3.0, 4.0),
        ("one-alpha", 1.0, 1.0, 1.0, 2.0, 3.0, 4.0),
        ("extrapolation", 1.5, -0.5, 0.0, 10.0, 20.0, 30.0),
        ("negative-alpha", -0.25, 0.75, -1.0, 3.0, -5.0, 7.0),
        ("reversed-grid", 0.75, 0.25, 10.0, 0.0, -10.0, -20.0),
        ("signed-zero-alpha", -0.0, 0.0, -0.0, 1.0, -0.0, 1.0),
        ("signed-zero-grid", 0.5, 0.5, "8000000000000000", "0000000000000000",
         "8000000000000000", "0000000000000000"),
        ("subnormal-grid", 0.5, 0.5, min_subnormal_f64, 2.0 * min_subnormal_f64,
         -min_subnormal_f64, -2.0 * min_subnormal_f64),
        ("tiny-alpha", float.fromhex("0x1.0000000000001p-1022"), 0.5,
         1.0, 2.0, 3.0, 4.0),
        ("large-finite", 0.25, 0.75, 1.0e300, -1.0e300, 1.0e300, -1.0e300),
        ("overflow-grid", 0.5, 0.5, maximum_finite_f64, -maximum_finite_f64,
         maximum_finite_f64, -maximum_finite_f64),
        ("positive-infinity-grid", 0.25, 0.75, float("inf"), 1.0, 2.0, float("inf")),
        ("negative-infinity-grid", 0.25, 0.75, float("-inf"), -1.0, -2.0, float("-inf")),
        ("nan-alpha", float("nan"), 0.5, 0.0, 1.0, 2.0, 3.0),
        ("nan-first-corner", 0.5, 0.5, float("nan"), 1.0, 2.0, 3.0),
        ("nan-last-corner", 0.5, 0.5, 0.0, 1.0, 2.0, float("nan")),
        ("precision-order", 0.1, 0.9, 1.0e20, 1.0e20 + 1.0e6,
         -1.0e20, -1.0e20 + 1.0e6),
        ("mixed-exponents", 1.0 / 3.0, 2.0 / 3.0, 2.0 ** -500, 2.0 ** 500,
         -2.0 ** -500, -2.0 ** 500),
        ("underflow", min_subnormal_f64, 0.5, 1.0, 2.0, -1.0, -2.0),
    ]

    for label, alpha1, alpha2, x00, x10, x01, x11 in mth_lerp2:
        yield {"case_id": f"mth/interpolation/lerp2/{label}", "family": "math",
               "operation": "mth_interpolation", "fn": "lerp2",
               "alpha1": interpolation_bits(alpha1, "f64"),
               "alpha2": interpolation_bits(alpha2, "f64"),
               "x00": interpolation_bits(x00, "f64"),
               "x10": interpolation_bits(x10, "f64"),
               "x01": interpolation_bits(x01, "f64"),
               "x11": interpolation_bits(x11, "f64")}

    mth_lerp3 = [
        ("basic", 0.25, 0.5, 0.75, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0),
        ("zero-alpha", 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0),
        ("one-alpha", 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0),
        ("extrapolation", 1.5, -0.5, 1.25, 0.0, 10.0, 20.0, 30.0,
         40.0, 50.0, 60.0, 70.0),
        ("negative-alpha", -0.25, 0.75, -0.5, -1.0, 3.0, -5.0, 7.0,
         -9.0, 11.0, -13.0, 15.0),
        ("reversed-grid", 0.75, 0.25, 0.5, 10.0, 0.0, -10.0, -20.0,
         20.0, 30.0, 40.0, 50.0),
        ("signed-zero-alpha", -0.0, 0.0, -0.0, -0.0, 1.0, -0.0, 1.0,
         -0.0, 1.0, -0.0, 1.0),
        ("signed-zero-cube", 0.5, 0.5, 0.5, "8000000000000000", "0000000000000000",
         "8000000000000000", "0000000000000000", "8000000000000000",
         "0000000000000000", "8000000000000000", "0000000000000000"),
        ("subnormal-cube", 0.5, 0.5, 0.5, min_subnormal_f64, 2.0 * min_subnormal_f64,
         3.0 * min_subnormal_f64, 4.0 * min_subnormal_f64, -min_subnormal_f64,
         -2.0 * min_subnormal_f64, -3.0 * min_subnormal_f64, -4.0 * min_subnormal_f64),
        ("tiny-alpha", float.fromhex("0x1.0000000000001p-1022"), 0.5, 0.75,
         1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0),
        ("large-finite", 0.25, 0.75, 0.5, 1.0e300, -1.0e300, 1.0e300, -1.0e300,
         1.0e300, -1.0e300, 1.0e300, -1.0e300),
        ("overflow-cube", 0.5, 0.5, 0.5, maximum_finite_f64, -maximum_finite_f64,
         maximum_finite_f64, -maximum_finite_f64, maximum_finite_f64,
         -maximum_finite_f64, maximum_finite_f64, -maximum_finite_f64),
        ("positive-infinity-cube", 0.25, 0.5, 0.75, float("inf"), 1.0, 2.0, 3.0,
         4.0, 5.0, 6.0, float("inf")),
        ("negative-infinity-cube", 0.25, 0.5, 0.75, float("-inf"), -1.0, -2.0, -3.0,
         -4.0, -5.0, -6.0, float("-inf")),
        ("nan-alpha", float("nan"), 0.5, 0.75, 0.0, 1.0, 2.0, 3.0,
         4.0, 5.0, 6.0, 7.0),
        ("nan-first-corner", 0.5, 0.5, 0.5, float("nan"), 1.0, 2.0, 3.0,
         4.0, 5.0, 6.0, 7.0),
        ("nan-last-corner", 0.5, 0.5, 0.5, 0.0, 1.0, 2.0, 3.0,
         4.0, 5.0, 6.0, float("nan")),
        ("precision-order", 0.1, 0.9, 0.3, 1.0e20, 1.0e20 + 1.0e6,
         -1.0e20, -1.0e20 + 1.0e6, 2.0e20, 2.0e20 + 1.0e6,
         -2.0e20, -2.0e20 + 1.0e6),
        ("mixed-exponents", 1.0 / 3.0, 2.0 / 3.0, 0.125, 2.0 ** -500, 2.0 ** 500,
         -2.0 ** -500, -2.0 ** 500, 2.0 ** -400, 2.0 ** 400,
         -2.0 ** -400, -2.0 ** 400),
        ("underflow", min_subnormal_f64, 0.5, 0.75, 1.0, 2.0, 3.0, 4.0,
         5.0, 6.0, 7.0, 8.0),
    ]

    for label, alpha1, alpha2, alpha3, x000, x100, x010, x110, x001, x101, x011, x111 in mth_lerp3:
        yield {"case_id": f"mth/interpolation/lerp3/{label}", "family": "math",
               "operation": "mth_interpolation", "fn": "lerp3",
               "alpha1": interpolation_bits(alpha1, "f64"),
               "alpha2": interpolation_bits(alpha2, "f64"),
               "alpha3": interpolation_bits(alpha3, "f64"),
               "x000": interpolation_bits(x000, "f64"),
               "x100": interpolation_bits(x100, "f64"),
               "x010": interpolation_bits(x010, "f64"),
               "x110": interpolation_bits(x110, "f64"),
               "x001": interpolation_bits(x001, "f64"),
               "x101": interpolation_bits(x101, "f64"),
               "x011": interpolation_bits(x011, "f64"),
               "x111": interpolation_bits(x111, "f64")}

    maximum_finite_f32 = float.fromhex("0x1.fffffep127")

    min_subnormal_f32 = float.fromhex("0x0.000002p-126")

    mth_catmullrom = [
        ("basic", 0.5, 0.0, 1.0, 2.0, 3.0),
        ("zero-alpha", 0.0, -1.0, 0.0, 1.0, 2.0),
        ("one-alpha", 1.0, -1.0, 0.0, 1.0, 2.0),
        ("extrapolation-negative", -0.5, 0.0, 1.0, 2.0, 3.0),
        ("extrapolation-positive", 1.5, 0.0, 1.0, 2.0, 3.0),
        ("constant", 0.25, 4.0, 4.0, 4.0, 4.0),
        ("peak", 0.5, 0.0, 0.0, 1.0, 0.0),
        ("symmetric", 0.5, -2.0, -1.0, 1.0, 2.0),
        ("negative", 0.25, -1.0, -3.0, -5.0, -7.0),
        ("signed-zero-alpha", "80000000", -0.0, 0.0, -0.0, 0.0),
        ("signed-zero-points", 0.5, "80000000", "00000000", "80000000", "00000000"),
        ("subnormal-points", 0.5, min_subnormal_f32, 2.0 * min_subnormal_f32,
         3.0 * min_subnormal_f32, 4.0 * min_subnormal_f32),
        ("subnormal-alpha", min_subnormal_f32, 1.0, 2.0, 3.0, 4.0),
        ("large-finite", 0.25, 1.0e10, -1.0e10, 1.0e10, -1.0e10),
        ("maximum-finite", 0.5, maximum_finite_f32, -maximum_finite_f32,
         maximum_finite_f32, -maximum_finite_f32),
        ("overflow", maximum_finite_f32, 1.0, -1.0, 1.0, -1.0),
        ("positive-infinity", 0.5, float("inf"), 1.0, 2.0, 3.0),
        ("negative-infinity", 0.5, float("-inf"), -1.0, -2.0, -3.0),
        ("nan-alpha", float("nan"), 0.0, 1.0, 2.0, 3.0),
        ("nan-point", 0.5, "7fc00001", 1.0, 2.0, 3.0),
    ]

    for label, alpha, p0, p1, p2, p3 in mth_catmullrom:
        yield {"case_id": f"mth/interpolation/catmullrom/{label}", "family": "math",
               "operation": "mth_interpolation", "fn": "catmullrom",
               "alpha": interpolation_bits(alpha, "f32"),
               "p0": interpolation_bits(p0, "f32"),
               "p1": interpolation_bits(p1, "f32"),
               "p2": interpolation_bits(p2, "f32"),
               "p3": interpolation_bits(p3, "f32")}
