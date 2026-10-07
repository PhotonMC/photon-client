"""The `math` family: the `mth/algorithms` cases."""

import math
from cases.math.common import *
from cases.math.shared import *
from cases.math.mth.shared import *


def cases():
    for seed in (0, 1, -1, 12345, 9223372036854775807):
        yield {"case_id": f"random/sequence/{seed}", "family": "math", "operation": "random",
               "seed": str(seed), "steps": [{"op": "nextInt"}, {"op": "nextFloat"},
                                            {"op": "nextDouble"}, {"op": "nextLong"},
                                            {"op": "nextBoolean"}]}

    for bound in (1, 2, 16, 100):
        yield {"case_id": f"random/bound-{bound}", "family": "math", "operation": "random",
               "seed": "42", "steps": [{"op": "nextIntBound", "bound": bound},
                                       {"op": "nextIntBound", "bound": bound}]}

    for label, seed in (("zero", 0), ("one", 1), ("negative-one", -1),
                        ("fixed", 12345), ("max", 9223372036854775807)):
        yield {"case_id": f"random/gaussian/sequence/{label}", "family": "math",
               "operation": "random", "seed": str(seed),
               "steps": [{"op": "nextGaussian"}, {"op": "nextGaussian"},
                         {"op": "nextGaussian"}]}

    yield {"case_id": "random/gaussian/interleaved", "family": "math", "operation": "random",
           "seed": "42", "steps": [{"op": "nextInt"}, {"op": "nextGaussian"},
                                      {"op": "nextGaussian"}, {"op": "nextDouble"},
                                      {"op": "nextGaussian"}]}

    yield {"case_id": "random/set-seed/gaussian-cache", "family": "math", "operation": "random",
           "seed": "12345", "steps": [{"op": "nextGaussian"},
                                           {"op": "setSeed", "seed": "12345"},
                                           {"op": "nextGaussian"}, {"op": "nextGaussian"}]}

    yield {"case_id": "random/set-seed/reseeded-state", "family": "math", "operation": "random",
           "seed": "-1", "steps": [{"op": "nextInt"}, {"op": "nextGaussian"},
                                        {"op": "setSeed", "seed": "0"},
                                        {"op": "nextDouble"}, {"op": "nextGaussian"},
                                        {"op": "nextGaussian"}]}

    yield {"case_id": "random/set-seed/boundaries", "family": "math", "operation": "random",
           "seed": "0", "steps": [{"op": "setSeed", "seed": "9223372036854775807"},
                                      {"op": "nextInt"},
                                      {"op": "setSeed", "seed": "-9223372036854775808"},
                                      {"op": "nextLong"}]}

    yield {"case_id": "random/overloads/integer-ranges", "family": "math", "operation": "random",
           "seed": "314159", "steps": [
               {"op": "nextIntBetweenInclusive", "min": -10, "max": 10},
               {"op": "nextIntBetweenInclusive", "min": 5, "max": 5},
               {"op": "nextIntRange", "origin": -20, "bound": 20},
               {"op": "nextIntRange", "origin": 0, "bound": 1}]}

    yield {"case_id": "random/overloads/triangle", "family": "math", "operation": "random",
           "seed": "-314159", "steps": [
               {"op": "triangleDouble", "mean": float_bits(-1.25, "f64"),
                "spread": float_bits(0.75, "f64")},
               {"op": "triangleFloat", "mean": float_bits(0.25),
                "spread": float_bits(1.5)}]}

    yield {"case_id": "random/overloads/consume-count", "family": "math", "operation": "random",
           "seed": "42", "steps": [{"op": "nextInt"}, {"op": "consumeCount", "rounds": 3},
                                      {"op": "nextInt"}, {"op": "consumeCount", "rounds": -2},
                                      {"op": "nextInt"}]}

    yield {"case_id": "random/fork/basic", "family": "math", "operation": "random_fork",
           "seed": "42", "prefix_steps": [],
           "child_steps": [{"op": "nextInt"}, {"op": "nextDouble"}, {"op": "nextLong"}],
           "parent_steps": [{"op": "nextInt"}, {"op": "nextLong"}]}

    yield {"case_id": "random/fork/gaussian-parent", "family": "math", "operation": "random_fork",
           "seed": "12345", "prefix_steps": [{"op": "nextGaussian"}],
           "child_steps": [{"op": "nextGaussian"}, {"op": "nextGaussian"}],
           "parent_steps": [{"op": "nextGaussian"}, {"op": "nextInt"}]}

    yield {"case_id": "random/positional/coordinates", "family": "math",
           "operation": "random_positional", "seed": "987654321",
           "factory_steps": [
               {"op": "at", "x": 0, "y": 0, "z": 0,
                "steps": [{"op": "nextInt"}, {"op": "nextDouble"}]},
               {"op": "at", "x": -2147483648, "y": 2147483647, "z": 2147483647,
                "steps": [{"op": "nextInt"}, {"op": "nextLong"}]},
               {"op": "at", "x": 123456, "y": -789, "z": 654321,
                "steps": [{"op": "nextGaussian"}, {"op": "nextGaussian"}]}],
           "parent_steps": [{"op": "nextInt"}]}

    yield {"case_id": "random/positional/hash", "family": "math",
           "operation": "random_positional", "seed": "-1",
           "factory_steps": [
               {"op": "fromHashOf", "name": "", "steps": [{"op": "nextInt"}]},
               {"op": "fromHashOf", "name": "minecraft:stone",
                "steps": [{"op": "nextInt"}, {"op": "nextDouble"}]},
               {"op": "fromHashOf", "name": "é😀",
                "steps": [{"op": "nextLong"}, {"op": "nextGaussian"}]}],
           "parent_steps": [{"op": "nextLong"}]}

    yield {"case_id": "random/positional/from-seed", "family": "math",
           "operation": "random_positional", "seed": "0",
           "factory_steps": [
               {"op": "fromSeed", "seed": "0", "steps": [{"op": "nextInt"}]},
               {"op": "fromSeed", "seed": "-1", "steps": [{"op": "nextLong"}]},
               {"op": "fromSeed", "seed": "9223372036854775807",
                "steps": [{"op": "nextGaussian"}, {"op": "nextGaussian"}]}],
           "parent_steps": [{"op": "nextInt"}]}

    for label, seed in (
        ("zero", 0),
        ("one", 1),
        ("negative-one", -1),
        ("positive", 42),
        ("negative", -42),
        ("large-positive", 987654321),
        ("large-negative", -987654321),
        ("long-max", 9223372036854775807),
        ("long-min", -9223372036854775808),
        ("alternating-positive", 305419896),
        ("alternating-negative", -6148914691236517206),
        ("high-bit", 4611686018427387904),
    ):
        yield {"case_id": f"mth/create-insecure-uuid/{label}", "family": "math",
               "operation": "mth_create_insecure_uuid", "seed": str(seed)}

    for label, input_value in (
        ("zero", 0),
        ("one", 1),
        ("negative-one", -1),
        ("i32-max", 2147483647),
        ("i32-min", -2147483648),
        ("high-bit-plus-one", -2147483647),
        ("low-31-bit", 1073741824),
        ("alternating-positive", 1431655765),
        ("alternating-negative", -1431655766),
        ("byte-alternating-positive", 16711935),
        ("byte-alternating-negative", -16711936),
        ("ascending-hex", 305419896),
        ("descending-hex", -2023406815),
        ("shift-16", 65536),
        ("shift-13", 8192),
        ("shift-1", 2),
        ("near-high-bit", 1073741823),
        ("near-low-bit", -1073741825),
        ("mixed-positive", 19088743),
        ("mixed-negative", -19088744),
        ("large-positive", 987654321),
        ("large-negative", -987654321),
        ("signed-pattern", -305419896),
        ("all-zero-high-byte", 16777215),
    ):
        yield {"case_id": f"mth/murmur-hash3-mixer/{label}", "family": "math",
               "operation": "mth_murmur_hash3_mixer", "input": str(input_value)}

    for label, from_value, to_value, condition, threshold in (
        ("empty", 0, 0, "ge", 0),
        ("single-true", 0, 1, "ge", 0),
        ("single-false", 0, 1, "ge", 1),
        ("single-gt-true", 0, 1, "gt", -1),
        ("basic-ge", 0, 16, "ge", 7),
        ("basic-gt", 0, 16, "gt", 7),
        ("threshold-at-to", -10, 10, "ge", 10),
        ("all-true", -10, 10, "ge", -20),
        ("negative-ge", -20, -5, "ge", -12),
        ("negative-gt", -20, -5, "gt", -12),
        ("cross-zero-ge", -10, 11, "ge", -1),
        ("cross-zero-gt", -10, 11, "gt", -1),
        ("offset-range", 5, 25, "ge", 13),
        ("descending-range", 25, 5, "ge", 13),
        ("max-empty", 2147483647, 2147483647, "ge", 2147483647),
        ("min-empty", -2147483648, -2147483648, "ge", -2147483648),
        ("full-range-overflow", -2147483648, 2147483647, "ge", 0),
        ("max-to-min-wrap", 2147483647, -2147483648, "gt", 2147483647),
        ("high-wrap-two", 2147483646, -2147483648, "gt", 2147483647),
        ("low-range-all-true", -2147483648, -2147483640, "ge", -2147483644),
        ("high-range-ge", 2147483630, 2147483647, "ge", 2147483640),
        ("high-range-gt", 2147483630, 2147483647, "gt", 2147483640),
        ("low-range-ge", -2147483648, -2147483630, "ge", -2147483640),
        ("low-range-gt", -2147483648, -2147483630, "gt", -2147483640),
    ):
        yield {"case_id": f"mth/binary-search/{label}", "family": "math",
               "operation": "mth_binary_search", "from": str(from_value),
               "to": str(to_value), "condition": condition, "threshold": str(threshold)}

    ray_intersects_aabb = [
        ("inside-positive-x", vector_bits(0.5, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("inside-negative-x", vector_bits(0.5, 0.5, 0.5), vector_bits(-1.0, -0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-left-toward", vector_bits(-1.0, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-left-away", vector_bits(-1.0, 0.5, 0.5), vector_bits(-1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-right-toward", vector_bits(2.0, 0.5, 0.5), vector_bits(-1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-right-away", vector_bits(2.0, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-below-toward", vector_bits(0.5, -1.0, 0.5), vector_bits(0.1, 1.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-above-away", vector_bits(0.5, 2.0, 0.5), vector_bits(0.1, 1.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-front-toward", vector_bits(0.5, 0.5, -1.0), vector_bits(0.0, 0.1, 1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-back-away", vector_bits(0.5, 0.5, 2.0), vector_bits(0.0, 0.1, 1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("parallel-x-miss", vector_bits(2.0, 0.5, 0.5), vector_bits(0.0, 1.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("parallel-negative-zero-x", vector_bits(2.0, 0.5, 0.5), vector_bits(-0.0, 1.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("diagonal-hit", vector_bits(-1.0, -1.0, -1.0), vector_bits(1.0, 1.0, 1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("diagonal-skew-miss", vector_bits(-1.0, -1.0, 2.0), vector_bits(1.0, 1.0, -1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("tangent-top-edge", vector_bits(-1.0, 1.0, 0.5), vector_bits(1.0, 0.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("tangent-corner", vector_bits(-1.0, 1.0, 1.0), vector_bits(1.0, 0.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("boundary-outward", vector_bits(0.5, 0.5, 0.0), vector_bits(0.0, 0.1, -1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("boundary-inward", vector_bits(0.5, 0.5, 0.0), vector_bits(0.0, 0.1, 1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("near-axis-hit", vector_bits(-1.0, 0.5, 0.5), vector_bits(1.0, 0.000001, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("zero-direction-inside", vector_bits(0.5, 0.5, 0.5), vector_bits(0.0, 0.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("zero-direction-outside", vector_bits(2.0, 0.5, 0.5), vector_bits(0.0, 0.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("large-coordinates", vector_bits(1000000000.0, 0.5, 0.5),
         vector_bits(-1.0, 0.1, 0.0), box_bits(999999999.0, 0.0, 0.0, 1000000001.0, 1.0, 1.0)),
        ("reversed-box", vector_bits(-1.0, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(1.0, 1.0, 1.0, 0.0, 0.0, 0.0)),
        ("point-box", vector_bits(-1.0, 0.0, 0.0), vector_bits(1.0, 0.1, 0.1),
         box_bits(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
        ("negative-box", vector_bits(-3.0, -2.5, -2.0), vector_bits(1.0, 0.2, 0.1),
         box_bits(-2.0, -2.0, -1.0, -1.0, -1.0, 0.0)),
        ("nan-start", vector_bits(float("nan"), 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("nan-direction", vector_bits(-1.0, 0.5, 0.5), vector_bits(float("nan"), 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("infinite-direction", vector_bits(-1.0, 0.5, 0.5), vector_bits(float("inf"), 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("infinite-box", vector_bits(-1.0, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(float("-inf"), 0.0, 0.0, float("inf"), 1.0, 1.0)),
    ]

    for label, ray_start, ray_dir, aabb in ray_intersects_aabb:
        yield {"case_id": f"mth/ray-intersects-aabb/{label}", "family": "math",
               "operation": "mth_ray_intersects_aabb", "ray_start": ray_start,
               "ray_dir": ray_dir, "aabb": aabb}

    mth_fast_inv_sqrt = [
        ("zero", float_bits(0.0, "f64")),
        ("negative-zero", "8000000000000000"),
        ("one", float_bits(1.0, "f64")),
        ("one-below", "3fefffffffffffff"),
        ("one-above", "3ff0000000000001"),
        ("two", float_bits(2.0, "f64")),
        ("quarter", float_bits(0.25, "f64")),
        ("four", float_bits(4.0, "f64")),
        ("pi", float_bits(math.pi, "f64")),
        ("large", float_bits(1.0e20, "f64")),
        ("max", "7fefffffffffffff"),
        ("min-normal", "0010000000000000"),
        ("positive-subnormal", "0000000000000001"),
        ("negative-one", float_bits(-1.0, "f64")),
        ("negative-two", float_bits(-2.0, "f64")),
        ("negative-max", "ffefffffffffffff"),
        ("negative-min-normal", "8010000000000000"),
        ("negative-subnormal", "8000000000000001"),
        ("infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("nan", "7ff8000000000000"),
        ("nan-payload", "7ff0000000000001"),
        ("negative-nan-payload", "fff0000000000001"),
        ("tiny-neighbor", "0000000000000002"),
    ]

    for label, input_value in mth_fast_inv_sqrt:
        yield {"case_id": f"mth/fast-inv-sqrt/{label}", "family": "math",
               "operation": "mth_fast_inv_sqrt", "input": input_value}

    mth_fast_inv_cube_root = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("one", "3f800000"),
        ("one-below", "3f7fffff"),
        ("one-above", "3f800001"),
        ("eight", "41000000"),
        ("eighth", "3e000000"),
        ("twenty-seven", "41d80000"),
        ("pi", float_bits(math.pi, "f32")),
        ("large", float_bits(1.0e20, "f32")),
        ("max", "7f7fffff"),
        ("min-normal", "00800000"),
        ("positive-subnormal", "00000001"),
        ("negative-one", "bf800000"),
        ("negative-eight", "c1000000"),
        ("negative-max", "ff7fffff"),
        ("negative-min-normal", "80800000"),
        ("negative-subnormal", "80000001"),
        ("infinity", "7f800000"),
        ("negative-infinity", "ff800000"),
        ("nan", "7fc00000"),
        ("nan-payload", "7fc12345"),
        ("negative-nan-payload", "ffc12345"),
        ("tiny-neighbor", "00000002"),
    ]

    for label, input_value in mth_fast_inv_cube_root:
        yield {"case_id": f"mth/fast-inv-cube-root/{label}", "family": "math",
               "operation": "mth_fast_inv_cube_root", "input": input_value}

    for label, hue, saturation, value, alpha in mth_hsv_to_color:
        hue = hue if isinstance(hue, str) else float_bits(hue, "f32")
        saturation = saturation if isinstance(saturation, str) else float_bits(saturation, "f32")
        value = value if isinstance(value, str) else float_bits(value, "f32")
        for mode in ("rgb", "argb"):
            yield {"case_id": f"mth/hsv-to-color/{label}/{mode}", "family": "math",
                   "operation": "mth_hsv_to_color", "mode": mode, "hue": hue,
                   "saturation": saturation, "value": value, "alpha": str(alpha)}

    mth_get_seed = [
        ("zero", 0, 0, 0),
        ("simple-positive", 1, 2, 3),
        ("simple-negative", -1, -2, -3),
        ("mixed-signs", -123, 456, -789),
        ("x-only", 123456, 0, 0),
        ("y-only", 0, -789, 0),
        ("z-only", 0, 0, 654321),
        ("x-min", -2147483648, 0, 0),
        ("x-max", 2147483647, 0, 0),
        ("y-min", 0, -2147483648, 0),
        ("y-max", 0, 2147483647, 0),
        ("z-min", 0, 0, -2147483648),
        ("z-max", 0, 0, 2147483647),
        ("all-min", -2147483648, -2147483648, -2147483648),
        ("all-max", 2147483647, 2147483647, 2147483647),
        ("alternating-boundaries", -2147483648, 2147483647, -2147483648),
        ("reversed-boundaries", 2147483647, -2147483648, 2147483647),
        ("x-product-within-i32", 686, 0, 0),
        ("x-product-wraps-positive", 687, 0, 0),
        ("x-product-wraps-negative", -687, 0, 0),
        ("large-mixed-positive", 1000000000, -1000000000, 1000000000),
        ("large-mixed-negative", -1000000000, 1000000000, -1000000000),
        ("hash-like", 123456789, -987654321, 135792468),
        ("section-like", -2097152, 319, 2097151),
    ]

    for label, x, y, z in mth_get_seed:
        yield {"case_id": f"mth/get-seed/{label}", "family": "math",
               "operation": "mth_get_seed", "x": x, "y": y, "z": z}

    mth_inv_sqrt_f32 = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("one", "3f800000"),
        ("two", "40000000"),
        ("quarter", "3e800000"),
        ("maximum-finite", "7f7fffff"),
        ("minimum-normal", "00800000"),
        ("positive-subnormal", "00000001"),
        ("negative-one", "bf800000"),
        ("negative-subnormal", "80000001"),
        ("infinity", "7f800000"),
        ("negative-infinity", "ff800000"),
        ("nan", "7fc00000"),
        ("nan-payload", "7fc12345"),
        ("negative-nan-payload", "ffc12345"),
    ]

    for label, input_value in mth_inv_sqrt_f32:
        yield {"case_id": f"mth/inv-sqrt/f32/{label}", "family": "math",
               "operation": "mth_inv_sqrt", "type": "f32", "input": input_value}

    mth_inv_sqrt_f64 = [
        ("zero", "0000000000000000"),
        ("negative-zero", "8000000000000000"),
        ("one", "3ff0000000000000"),
        ("two", "4000000000000000"),
        ("quarter", "3fd0000000000000"),
        ("maximum-finite", "7fefffffffffffff"),
        ("minimum-normal", "0010000000000000"),
        ("positive-subnormal", "0000000000000001"),
        ("negative-one", "bff0000000000000"),
        ("negative-subnormal", "8000000000000001"),
        ("infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("nan", "7ff8000000000000"),
        ("nan-payload", "7ff0000000000001"),
        ("negative-nan-payload", "fff0000000000001"),
    ]

    for label, input_value in mth_inv_sqrt_f64:
        yield {"case_id": f"mth/inv-sqrt/f64/{label}", "family": "math",
               "operation": "mth_inv_sqrt", "type": "f64", "input": input_value}

    mth_next_int = [
        ("normal", 42, -10, 10),
        ("same-bounds", 42, 5, 5),
        ("inverted-bounds", 42, 10, -10),
        ("positive-boundary", 123, 2147483640, 2147483647),
        ("negative-boundary", -123, -2147483648, -2147483641),
        ("single-bit-range", 0, 0, 1),
    ]

    for label, seed, minimum, maximum in mth_next_int:
        yield {"case_id": f"mth/next-random/i32/{label}", "family": "math",
               "operation": "mth_next_random", "type": "i32", "seed": str(seed),
               "min": str(minimum), "max": str(maximum)}

    for width in ("f32", "f64"):
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_next_float = [
            ("normal", 42, -1.0, 1.0),
            ("same-bounds", 42, 0.5, 0.5),
            ("inverted-bounds", 42, 1.0, -1.0),
            ("signed-zero", 42, -0.0, 0.0),
            ("wide-finite", 123, -maximum_finite, maximum_finite),
            ("infinite-max", 123, 1.0, float("inf")),
            ("nan-min", 123, float("nan"), 1.0),
        ]
        for label, seed, minimum, maximum in mth_next_float:
            yield {"case_id": f"mth/next-random/{width}/{label}", "family": "math",
                   "operation": "mth_next_random", "type": width, "seed": str(seed),
                   "min": float_bits(minimum, width), "max": float_bits(maximum, width)}

    mth_random_between_inclusive = [
        ("normal", 42, -10, 10),
        ("same-bounds", 42, 5, 5),
        ("positive-boundary", 123, 2147483640, 2147483647),
        ("negative-boundary", -123, -2147483648, -2147483641),
        ("wide-finite", 7, -100, 100),
        ("single-bit-range", 0, 0, 1),
    ]

    for label, seed, minimum, maximum in mth_random_between_inclusive:
        yield {"case_id": f"mth/random-helpers/inclusive/{label}", "family": "math",
               "operation": "mth_random_helpers", "type": "i32", "fn": "inclusive",
               "seed": str(seed), "min": str(minimum), "max": str(maximum)}

    mth_random_between = [
        ("normal", 42, -1.0, 1.0),
        ("same-bounds", 42, 0.5, 0.5),
        ("reversed-bounds", 42, 1.0, -1.0),
        ("signed-zero", 42, -0.0, 0.0),
        ("infinite-max", 123, 1.0, float("inf")),
        ("nan-min", 123, float("nan"), 1.0),
    ]

    for label, seed, minimum, maximum in mth_random_between:
        yield {"case_id": f"mth/random-helpers/between/{label}", "family": "math",
               "operation": "mth_random_helpers", "type": "f32", "fn": "between",
               "seed": str(seed), "min": float_bits(minimum, "f32"),
               "max": float_bits(maximum, "f32")}

    mth_normal = [
        ("normal", 42, 0.0, 1.0),
        ("shifted", 123, 2.5, 0.75),
        ("zero-deviation", 42, 4.0, 0.0),
        ("negative-deviation", -123, 1.0, -2.0),
        ("negative-zero-mean", 42, -0.0, 0.0),
        ("infinite-mean", 7, float("inf"), 1.0),
    ]

    for label, seed, mean, deviation in mth_normal:
        yield {"case_id": f"mth/random-helpers/normal/{label}", "family": "math",
               "operation": "mth_random_helpers", "type": "f32", "fn": "normal",
               "seed": str(seed), "mean": float_bits(mean, "f32"),
               "deviation": float_bits(deviation, "f32")}
