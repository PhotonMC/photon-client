"""The `math` family: the `vec2` cases."""

import math
from cases.math.common import *
from cases.math.shared import *


def cases():
    vectors = {f"direction-{x}-{y}": (float_bits(x), float_bits(y))
               for x in (-1, 0, 1) for y in (-1, 0, 1)}

    vectors.update({"negative-zero": ("80000000", "80000000"),
                    "subnormal": ("00000001", "00000001"),
                    "max-finite": ("7f7fffff", "7f7fffff"),
                    "unequal": (float_bits(3), float_bits(-7))})

    for index, bits in enumerate(neighbors(0.0001, "f32")):
        vectors[f"normalization-boundary-{index}"] = (bits, "00000000")

    for label, (x, y) in vectors.items():
        yield {"case_id": f"vec2/normalize/{label}", "family": "math", "operation": "vec2",
               "x": x, "y": y, "steps": [{"action": "normalize"}]}

    for factor in (-2, -0.0, 0.0, 0.37, 0.98, 3):
        yield {"case_id": f"vec2/chain/{float_bits(factor)}", "family": "math", "operation": "vec2",
               "x": float_bits(1), "y": float_bits(-1),
               "steps": [{"action": "normalize"}, {"action": "scale", "factor": float_bits(factor)},
                         {"action": "normalize"}]}

    yield {"case_id": "vec2/rotate/basic", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(1.25, -2.5), "angle": float_bits(0.5, "f64")}

    yield {"case_id": "vec2/rotate/negative-components", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(-4.5, 6.25), "angle": float_bits(-0.75, "f64")}

    yield {"case_id": "vec2/rotate/zero-angle", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(2.0, -3.0), "angle": float_bits(0.0, "f64")}

    yield {"case_id": "vec2/rotate/quarter-turn", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(1.0, 0.0), "angle": float_bits(math.pi / 2.0, "f64")}

    yield {"case_id": "vec2/rotate/half-turn", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(0.25, -0.5), "angle": float_bits(math.pi, "f64")}

    yield {"case_id": "vec2/rotate/negative-quarter-turn", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(0.0, 1.0), "angle": float_bits(-math.pi / 2.0, "f64")}

    yield {"case_id": "vec2/rotate/small-quantized-angle", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(-0.125, 0.25), "angle": float_bits(0.0001, "f64")}

    yield {"case_id": "vec2/rotate/full-turn", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(1.0, -1.0), "angle": float_bits(2.0 * math.pi, "f64")}

    yield {"case_id": "vec2/rotate/combined", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(3.5, -7.25), "angle": float_bits(1.234567890123, "f64")}

    yield {"case_id": "vec2/rotate/signed-zero-angle", "family": "math", "operation": "vec2_rotate",
           "first": {"x": "80000000", "y": "00000000"}, "angle": "8000000000000000"}

    yield {"case_id": "vec2/rotate/signed-zero-vector", "family": "math", "operation": "vec2_rotate",
           "first": {"x": "00000000", "y": "80000000"}, "angle": "0000000000000000"}

    yield {"case_id": "vec2/rotate/subnormal-vector", "family": "math", "operation": "vec2_rotate",
           "first": {"x": "00000001", "y": "80000001"}, "angle": float_bits(math.pi / 4.0, "f64")}

    yield {"case_id": "vec2/rotate/max-finite-vector", "family": "math", "operation": "vec2_rotate",
           "first": {"x": "7f7fffff", "y": "ff7fffff"}, "angle": float_bits(0.25, "f64")}

    yield {"case_id": "vec2/rotate/special-vector", "family": "math", "operation": "vec2_rotate",
           "first": {"x": "7fc00001", "y": "ff800000"}, "angle": float_bits(0.5, "f64")}

    yield {"case_id": "vec2/rotate/positive-infinity-angle", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(1.0, -2.0), "angle": float_bits(float("inf"), "f64")}

    yield {"case_id": "vec2/rotate/negative-infinity-angle", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(-4.0, 6.0), "angle": float_bits(float("-inf"), "f64")}

    yield {"case_id": "vec2/rotate/nan-angle", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(3.5, -7.25), "angle": "7ff8000000000001"}

    yield {"case_id": "vec2/rotate/large-finite-angle", "family": "math", "operation": "vec2_rotate",
           "first": vector2_bits(0.5, -1.0), "angle": "7fefffffffffffff"}

    yield {"case_id": "vec2/dot/basic", "family": "math", "operation": "vec2_dot",
           "first": vector2_bits(1.25, -2.5), "second": vector2_bits(4.0, -1.0)}

    yield {"case_id": "vec2/dot/order-and-rounding", "family": "math", "operation": "vec2_dot",
           "first": vector2_bits(1.0, 2.0 ** -23), "second": vector2_bits(1.0, 1.0)}

    yield {"case_id": "vec2/dot/signed-zero", "family": "math", "operation": "vec2_dot",
           "first": {"x": "80000000", "y": "80000000"},
           "second": vector2_bits(1.0, 1.0)}

    yield {"case_id": "vec2/dot/overflow", "family": "math", "operation": "vec2_dot",
           "first": {"x": "7f7fffff", "y": float_bits(1.0)},
           "second": vector2_bits(2.0, 0.0)}

    yield {"case_id": "vec2/dot/special-values", "family": "math", "operation": "vec2_dot",
           "first": {"x": "7f800000", "y": float_bits(2.0)},
           "second": {"x": "00000000", "y": float_bits(-4.0)}}

    yield {"case_id": "vec2/add/vector-basic", "family": "math", "operation": "vec2_add_vec2",
           "first": vector2_bits(1.25, -2.5), "second": vector2_bits(-0.25, 4.0)}

    yield {"case_id": "vec2/add/vector-signed-zero", "family": "math", "operation": "vec2_add_vec2",
           "first": {"x": "80000000", "y": "80000000"},
           "second": {"x": "80000000", "y": "80000000"}}

    yield {"case_id": "vec2/add/vector-overflow", "family": "math", "operation": "vec2_add_vec2",
           "first": {"x": "7f7fffff", "y": "ff7fffff"},
           "second": {"x": "7f7fffff", "y": "ff7fffff"}}

    yield {"case_id": "vec2/add/scalar-basic", "family": "math", "operation": "vec2_add_scalar",
           "first": vector2_bits(-1.5, 2.25), "value": float_bits(0.5)}

    yield {"case_id": "vec2/add/scalar-special-values", "family": "math", "operation": "vec2_add_scalar",
           "first": {"x": "7f800000", "y": float_bits(2.0)}, "value": "ff800000"}

    yield {"case_id": "vec2/distance-to-sqr/vector-basic", "family": "math",
           "operation": "vec2_distance_to_sqr", "first": vector2_bits(1.25, -2.5),
           "second": vector2_bits(4.5, -1.0)}

    yield {"case_id": "vec2/distance-to-sqr/order-and-rounding", "family": "math",
           "operation": "vec2_distance_to_sqr", "first": vector2_bits(0.0, 0.0),
           "second": vector2_bits(2.0 ** 12, 1.1)}

    yield {"case_id": "vec2/distance-to-sqr/signed-zero", "family": "math",
           "operation": "vec2_distance_to_sqr", "first": vector2_bits(-0.0, -0.0),
           "second": vector2_bits(0.0, 0.0)}

    yield {"case_id": "vec2/distance-to-sqr/overflow", "family": "math",
           "operation": "vec2_distance_to_sqr", "first": vector2_bits(0.0, 0.0),
           "second": {"x": "7f7fffff", "y": "7f7fffff"}}

    yield {"case_id": "vec2/distance-to-sqr/special-values", "family": "math",
           "operation": "vec2_distance_to_sqr", "first": vector2_bits(0.0, 0.0),
           "second": {"x": "7f800000", "y": "7fc00002"}}

    yield {"case_id": "vec2/negated/basic", "family": "math", "operation": "vec2_negated",
           "first": vector2_bits(1.25, -2.5)}

    yield {"case_id": "vec2/negated/signed-zero", "family": "math", "operation": "vec2_negated",
           "first": {"x": "80000000", "y": "00000000"}}

    yield {"case_id": "vec2/negated/subnormal", "family": "math", "operation": "vec2_negated",
           "first": {"x": "00000001", "y": "80000001"}}

    yield {"case_id": "vec2/negated/infinity-and-boundary", "family": "math", "operation": "vec2_negated",
           "first": {"x": "7f7fffff", "y": "ff800000"}}

    yield {"case_id": "vec2/negated/nan-payload", "family": "math", "operation": "vec2_negated",
           "first": {"x": "7fc00001", "y": "ffc00002"}}

    yield {"case_id": "vec2/equality/basic", "family": "math", "operation": "vec2_equality",
           "first": vector2_bits(1.25, -2.5), "second": vector2_bits(1.25, -2.5)}

    yield {"case_id": "vec2/equality/different-component", "family": "math", "operation": "vec2_equality",
           "first": vector2_bits(1.25, -2.5), "second": vector2_bits(1.25, -2.25)}

    yield {"case_id": "vec2/equality/signed-zero", "family": "math", "operation": "vec2_equality",
           "first": {"x": "80000000", "y": "00000000"},
           "second": {"x": "00000000", "y": "00000000"}}

    yield {"case_id": "vec2/equality/nan-payloads", "family": "math", "operation": "vec2_equality",
           "first": {"x": "7fc00001", "y": "ffc00002"},
           "second": {"x": "7fc00002", "y": "ffc00001"}}

    yield {"case_id": "vec2/equality/infinities", "family": "math", "operation": "vec2_equality",
           "first": {"x": "7f800000", "y": "ff800000"},
           "second": {"x": "7f800000", "y": "ff800000"}}

    yield {"case_id": "vec2/equality/special-mismatch", "family": "math", "operation": "vec2_equality",
           "first": {"x": "7f800000", "y": "7fc00001"},
           "second": {"x": "ff800000", "y": "7fc00002"}}

    for name in ("zero", "one", "unit_x", "neg_unit_x", "unit_y", "neg_unit_y", "max", "min"):
        yield {"case_id": f"vec2/constants/{name}", "family": "math", "operation": "vec2_constant",
               "name": name}
