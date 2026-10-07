"""The `math` family: the `vec3/distance` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    yield {"case_id": "vec3/dot/basic", "family": "math", "operation": "vec3_dot",
           "first": vector_bits(1.25, -2.5, 3.75), "second": vector_bits(4.0, -1.0, 0.5)}

    yield {"case_id": "vec3/dot/order-and-cancellation", "family": "math", "operation": "vec3_dot",
           "first": vector_bits(1.0, 2.0 ** -53, -1.0),
           "second": vector_bits(2.0 ** 53, 1.0, 2.0 ** 53)}

    yield {"case_id": "vec3/dot/signed-zero", "family": "math", "operation": "vec3_dot",
           "first": vector_bits(-0.0, -0.0, -0.0), "second": vector_bits(1.0, 1.0, 1.0)}

    yield {"case_id": "vec3/dot/overflow", "family": "math", "operation": "vec3_dot",
           "first": {"x": "7fefffffffffffff", "y": float_bits(1.0, "f64"),
                     "z": float_bits(-1.0, "f64")},
           "second": {"x": float_bits(2.0, "f64"), "y": float_bits(0.0, "f64"),
                      "z": float_bits(0.0, "f64")}}

    yield {"case_id": "vec3/dot/special-values", "family": "math", "operation": "vec3_dot",
           "first": {"x": float_bits(float("inf"), "f64"), "y": float_bits(2.0, "f64"),
                     "z": float_bits(-3.0, "f64")},
           "second": {"x": float_bits(0.0, "f64"), "y": float_bits(-4.0, "f64"),
                      "z": float_bits(5.0, "f64")}}

    yield {"case_id": "vec3/distance-to-sqr/vector-basic", "family": "math",
           "operation": "vec3_distance_to_sqr", "first": vector_bits(1.25, -2.5, 3.75),
           "second": vector_bits(4.5, -1.0, -0.25)}

    yield {"case_id": "vec3/distance-to-sqr/components-basic", "family": "math",
           "operation": "vec3_distance_to_sqr_components", "first": vector_bits(1.25, -2.5, 3.75),
           "x": float_bits(4.5, "f64"), "y": float_bits(-1.0, "f64"), "z": float_bits(-0.25, "f64")}

    yield {"case_id": "vec3/distance-to-sqr/order-and-rounding", "family": "math",
           "operation": "vec3_distance_to_sqr", "first": vector_bits(0.0, 0.0, 0.0),
           "second": vector_bits(2.0 ** 27, 1.0, 1.1)}

    yield {"case_id": "vec3/distance-to-sqr/signed-zero", "family": "math",
           "operation": "vec3_distance_to_sqr_components", "first": vector_bits(-0.0, -0.0, -0.0),
           "x": float_bits(0.0, "f64"), "y": float_bits(0.0, "f64"), "z": float_bits(0.0, "f64")}

    yield {"case_id": "vec3/distance-to-sqr/overflow-and-special-values", "family": "math",
           "operation": "vec3_distance_to_sqr", "first": vector_bits(0.0, 0.0, 0.0),
           "second": {"x": "7fefffffffffffff", "y": float_bits(float("inf"), "f64"),
                      "z": "7ff8000000000002"}}

    yield {"case_id": "vec3/distance-to/basic", "family": "math",
           "operation": "vec3_distance_to", "first": vector_bits(1.25, -2.5, 3.75),
           "second": vector_bits(4.5, -1.0, -0.25)}

    yield {"case_id": "vec3/distance-to/order-and-rounding", "family": "math",
           "operation": "vec3_distance_to", "first": vector_bits(0.0, 0.0, 0.0),
           "second": vector_bits(2.0 ** 27, 1.0, 1.1)}

    yield {"case_id": "vec3/distance-to/signed-zero", "family": "math",
           "operation": "vec3_distance_to", "first": vector_bits(-0.0, -0.0, -0.0),
           "second": vector_bits(0.0, 0.0, 0.0)}

    yield {"case_id": "vec3/distance-to/subnormal", "family": "math",
           "operation": "vec3_distance_to", "first": {"x": "0000000000000001",
                                                            "y": "8000000000000001",
                                                            "z": "0000000000000000"},
           "second": vector_bits(0.0, 0.0, 0.0)}

    yield {"case_id": "vec3/distance-to/overflow-and-special-values", "family": "math",
           "operation": "vec3_distance_to", "first": vector_bits(0.0, 0.0, 0.0),
           "second": {"x": "7fefffffffffffff", "y": float_bits(float("inf"), "f64"),
                      "z": "7ff8000000000002"}}

    yield {"case_id": "vec3/horizontal-distances/basic", "family": "math",
           "operation": "vec3_horizontal_distances", "first": vector_bits(3.0, 4.0, -12.0)}

    yield {"case_id": "vec3/horizontal-distances/order-and-rounding", "family": "math",
           "operation": "vec3_horizontal_distances",
           "first": vector_bits(2.0 ** 27, -123.0, 1.1)}

    yield {"case_id": "vec3/horizontal-distances/signed-zero", "family": "math",
           "operation": "vec3_horizontal_distances", "first": vector_bits(-0.0, 5.0, 0.0)}

    yield {"case_id": "vec3/horizontal-distances/subnormal", "family": "math",
           "operation": "vec3_horizontal_distances",
           "first": {"x": "0000000000000001", "y": "7fefffffffffffff", "z": "8000000000000001"}}

    yield {"case_id": "vec3/horizontal-distances/infinities", "family": "math",
           "operation": "vec3_horizontal_distances",
           "first": {"x": float_bits(float("inf"), "f64"),
                     "y": "7ff8000000000001", "z": float_bits(float("-inf"), "f64")}}

    yield {"case_id": "vec3/closer-than/basic", "family": "math",
           "operation": "vec3_closer_than", "first": vector_bits(1.25, -2.5, 3.75),
           "second": vector_bits(4.5, -1.0, -0.25),
           "distance": float_bits(6.0, "f64"),
           "distance_xz": float_bits(5.25, "f64"), "distance_y": float_bits(2.0, "f64")}

    yield {"case_id": "vec3/closer-than/strict-boundary", "family": "math",
           "operation": "vec3_closer_than", "first": vector_bits(0.0, 0.0, 0.0),
           "second": vector_bits(3.0, 4.0, 0.0),
           "distance": float_bits(5.0, "f64"),
           "distance_xz": float_bits(3.0, "f64"), "distance_y": float_bits(4.0, "f64")}

    yield {"case_id": "vec3/closer-than/vertical-gate", "family": "math",
           "operation": "vec3_closer_than", "first": vector_bits(0.0, 0.0, 0.0),
           "second": vector_bits(0.0, 4.0, 0.0),
           "distance": float_bits(5.0, "f64"),
           "distance_xz": float_bits(1.0, "f64"), "distance_y": float_bits(4.0, "f64")}

    yield {"case_id": "vec3/closer-than/negative-zero-thresholds", "family": "math",
           "operation": "vec3_closer_than", "first": vector_bits(-0.0, -0.0, -0.0),
           "second": vector_bits(0.0, 0.0, 0.0),
           "distance": float_bits(-0.0, "f64"),
           "distance_xz": float_bits(-0.0, "f64"), "distance_y": float_bits(-0.0, "f64")}

    yield {"case_id": "vec3/closer-than/special-values", "family": "math",
           "operation": "vec3_closer_than",
           "first": {"x": float_bits(float("nan"), "f64"), "y": float_bits(1.0, "f64"),
                     "z": float_bits(float("inf"), "f64")},
           "second": vector_bits(0.0, 0.0, 0.0),
           "distance": float_bits(float("inf"), "f64"),
           "distance_xz": float_bits(float("inf"), "f64"), "distance_y": float_bits(float("inf"), "f64")}

    yield {"case_id": "vec3/offset-random/basic", "family": "math",
           "operation": "vec3_offset_random", "first": vector_bits(1.25, -2.5, 3.75),
           "seed": "42", "offset": float_bits(1.0)}

    yield {"case_id": "vec3/offset-random/negative-offset", "family": "math",
           "operation": "vec3_offset_random", "first": vector_bits(-4.5, 6.25, 8.75),
           "seed": "-1", "offset": float_bits(-2.5)}

    yield {"case_id": "vec3/offset-random/signed-zero", "family": "math",
           "operation": "vec3_offset_random", "first": vector_bits(-0.0, 0.0, -0.0),
           "seed": "0", "offset": float_bits(-0.0)}

    yield {"case_id": "vec3/offset-random/subnormal-offset", "family": "math",
           "operation": "vec3_offset_random", "first": {"x": "0000000000000001",
           "y": "8000000000000001", "z": "0000000000000000"},
           "seed": "12345", "offset": "00000001"}

    yield {"case_id": "vec3/offset-random/special-values", "family": "math",
           "operation": "vec3_offset_random", "first": {"x": float_bits(float("inf"), "f64"),
           "y": float_bits(float("nan"), "f64"), "z": float_bits(float("-inf"), "f64")},
           "seed": "-9223372036854775808", "offset": "7f7fffff"}

    yield {"case_id": "vec3/lerp/basic", "family": "math", "operation": "vec3_lerp",
           "first": vector_bits(1.25, -2.5, 3.75),
           "second": vector_bits(-4.5, 6.25, 8.75), "alpha": float_bits(0.25, "f64")}

    yield {"case_id": "vec3/lerp/negative-extrapolation", "family": "math", "operation": "vec3_lerp",
           "first": vector_bits(-4.5, 6.25, 8.75),
           "second": vector_bits(1.25, -2.5, 3.75), "alpha": float_bits(-0.5, "f64")}

    yield {"case_id": "vec3/lerp/over-one", "family": "math", "operation": "vec3_lerp",
           "first": vector_bits(1.0e200, -1.0e200, 1.0e-200),
           "second": vector_bits(-1.0e200, 1.0e200, -1.0e-200), "alpha": float_bits(1.5, "f64")}

    yield {"case_id": "vec3/lerp/signed-zero-and-subnormal", "family": "math", "operation": "vec3_lerp",
           "first": {"x": "8000000000000000", "y": "0000000000000001", "z": "8000000000000001"},
           "second": {"x": "0000000000000000", "y": "8000000000000000", "z": "0000000000000000"},
           "alpha": float_bits(-0.0, "f64")}

    yield {"case_id": "vec3/lerp/special-values", "family": "math", "operation": "vec3_lerp",
           "first": {"x": float_bits(float("inf"), "f64"), "y": float_bits(float("nan"), "f64"),
                     "z": float_bits(float("-inf"), "f64")},
           "second": {"x": float_bits(float("-inf"), "f64"), "y": float_bits(1.0, "f64"),
                      "z": float_bits(float("inf"), "f64")},
           "alpha": float_bits(0.5, "f64")}

    yield {"case_id": "vec3/is-finite/all-finite", "family": "math",
           "operation": "vec3_is_finite",
           "first": {"x": "7fefffffffffffff", "y": "8000000000000001", "z": "0000000000000000"}}

    yield {"case_id": "vec3/is-finite/positive-infinity", "family": "math",
           "operation": "vec3_is_finite",
           "first": {"x": float_bits(float("inf"), "f64"), "y": float_bits(-1.0, "f64"),
                     "z": float_bits(2.0, "f64")}}

    yield {"case_id": "vec3/is-finite/negative-infinity", "family": "math",
           "operation": "vec3_is_finite",
           "first": {"x": float_bits(1.0, "f64"), "y": float_bits(float("-inf"), "f64"),
                     "z": float_bits(2.0, "f64")}}

    yield {"case_id": "vec3/is-finite/nan", "family": "math",
           "operation": "vec3_is_finite",
           "first": {"x": float_bits(1.0, "f64"), "y": float_bits(-2.0, "f64"),
                     "z": "7ff8000000000001"}}

    yield {"case_id": "vec3/is-finite/mixed-special-values", "family": "math",
           "operation": "vec3_is_finite",
           "first": {"x": "fff0000000000000", "y": "7ff8000000000000", "z": "0000000000000001"}}

    yield {"case_id": "vec3/equality/basic", "family": "math", "operation": "vec3_equality",
           "first": vector_bits(1.25, -2.5, 3.75), "second": vector_bits(1.25, -2.5, 3.75)}

    yield {"case_id": "vec3/equality/different-component", "family": "math", "operation": "vec3_equality",
           "first": vector_bits(1.25, -2.5, 3.75), "second": vector_bits(1.25, -2.5, 4.0)}

    yield {"case_id": "vec3/equality/signed-zero", "family": "math", "operation": "vec3_equality",
           "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "0000000000000000"},
           "second": vector_bits(0.0, 0.0, 0.0)}

    yield {"case_id": "vec3/equality/nan-payloads", "family": "math", "operation": "vec3_equality",
           "first": {"x": "7ff8000000000001", "y": "8000000000000000", "z": "7ff0000000000000"},
           "second": {"x": "7ff0000000000001", "y": "8000000000000000", "z": "7ff0000000000000"}}

    yield {"case_id": "vec3/equality/infinities", "family": "math", "operation": "vec3_equality",
           "first": {"x": "7ff0000000000000", "y": "fff0000000000000", "z": "0000000000000000"},
           "second": {"x": "7ff0000000000000", "y": "fff0000000000000", "z": "0000000000000000"}}

    yield {"case_id": "vec3/equality/special-mismatch", "family": "math", "operation": "vec3_equality",
           "first": {"x": "7ff0000000000000", "y": "7ff8000000000001", "z": "8000000000000000"},
           "second": {"x": "fff0000000000000", "y": "7ff0000000000001", "z": "0000000000000000"}}

    yield {"case_id": "vec3/to-string/basic", "family": "math", "operation": "vec3_to_string",
           "first": vector_bits(0.125, -2.5, 3.75)}

    yield {"case_id": "vec3/to-string/scientific", "family": "math", "operation": "vec3_to_string",
           "first": vector_bits(0.0001, 10000000.0, -0.0)}

    yield {"case_id": "vec3/to-string/thresholds", "family": "math", "operation": "vec3_to_string",
           "first": vector_bits(0.001, 9999999.0, 100000000.0)}

    yield {"case_id": "vec3/to-string/signed-zero", "family": "math", "operation": "vec3_to_string",
           "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"}}

    yield {"case_id": "vec3/to-string/infinity", "family": "math", "operation": "vec3_to_string",
           "first": {"x": "fff0000000000000", "y": "7ff0000000000000", "z": "fff0000000000000"}}

    yield {"case_id": "vec3/to-string/nan", "family": "math", "operation": "vec3_to_string",
           "first": {"x": "7ff8000000000001", "y": "7ff0000000000001", "z": "fff8000000000001"}}
