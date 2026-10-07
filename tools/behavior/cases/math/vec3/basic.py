"""The `math` family: the `vec3/basic` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    codec_decodes = {
        "basic": [float_bits(1.25, "f64"), float_bits(-2.5, "f64"), float_bits(3.75, "f64")],
        "zero": [float_bits(0.0, "f64"), float_bits(0.0, "f64"), float_bits(0.0, "f64")],
        "signed-zero": ["8000000000000000", "0000000000000000", "8000000000000000"],
        "subnormal-and-large": ["0000000000000001", "7fefffffffffffff", "8000000000000001"],
        "special": ["7ff8000000000002", "7ff0000000000000", "fff0000000000000"],
        "empty": [],
        "short": [float_bits(1.0, "f64"), float_bits(2.0, "f64")],
        "long": [float_bits(1.0, "f64"), float_bits(2.0, "f64"),
                  float_bits(3.0, "f64"), float_bits(4.0, "f64")],
    }

    for label, values in codec_decodes.items():
        yield {"case_id": f"vec3/codec/decode/{label}", "family": "math",
               "operation": "vec3_codec", "mode": "decode", "values": values}

    codec_encodes = {
        "basic": vector_bits(1.25, -2.5, 3.75),
        "zero": vector_bits(0.0, 0.0, 0.0),
        "signed-zero": {"x": "8000000000000000", "y": "0000000000000000",
                         "z": "8000000000000000"},
        "special": {"x": "7ff8000000000002", "y": "7ff0000000000000",
                     "z": "fff0000000000000"},
    }

    for label, values in codec_encodes.items():
        yield {"case_id": f"vec3/codec/encode/{label}", "family": "math",
               "operation": "vec3_codec", "mode": "encode", **values}

    vectors3 = {f"direction-{x}-{y}-{z}": (float_bits(x, "f64"), float_bits(y, "f64"), float_bits(z, "f64"))
                for x in (-1, 0, 1) for y in (-1, 0, 1) for z in (-1, 0, 1)}

    vectors3.update({"negative-zero": ("8000000000000000", "8000000000000000", "8000000000000000"),
                     "subnormal": ("0000000000000001", "0000000000000001", "0000000000000001"),
                     "max-finite": ("7fefffffffffffff", "7fefffffffffffff", "7fefffffffffffff"),
                     "unequal": (float_bits(3, "f64"), float_bits(-7, "f64"), float_bits(0.5, "f64"))})

    for index, bits in enumerate(neighbors(threshold, "f64")):
        vectors3[f"normalization-boundary-{index}"] = (bits, "0000000000000000", "0000000000000000")

    for label, (x, y, z) in vectors3.items():
        yield {"case_id": f"vec3/normalize/{label}", "family": "math", "operation": "vec3",
               "x": x, "y": y, "z": z, "steps": [{"action": "normalize"}]}

    for factor in (-2, -0.0, 0.0, 0.37, 0.98, 3):
        yield {"case_id": f"vec3/chain/{float_bits(factor, 'f64')}", "family": "math", "operation": "vec3",
               "x": float_bits(1, "f64"), "y": float_bits(-1, "f64"), "z": float_bits(0.5, "f64"),
               "steps": [{"action": "normalize"}, {"action": "scale", "factor": float_bits(factor, "f64")},
                         {"action": "normalize"}]}

    yield {"case_id": "vec3/reverse/basic", "family": "math", "operation": "vec3",
           "x": float_bits(1.25, "f64"), "y": float_bits(-2.5, "f64"), "z": float_bits(3.75, "f64"),
           "steps": [{"action": "reverse"}]}

    yield {"case_id": "vec3/reverse/chain", "family": "math", "operation": "vec3",
           "x": float_bits(-4.5, "f64"), "y": float_bits(6.25, "f64"), "z": float_bits(8.75, "f64"),
           "steps": [{"action": "reverse"},
                      {"action": "add_components", "x": float_bits(0.5, "f64"),
                       "y": float_bits(-1.5, "f64"), "z": float_bits(-0.75, "f64")}]}

    yield {"case_id": "vec3/reverse/signed-zero", "family": "math", "operation": "vec3",
           "x": float_bits(-0.0, "f64"), "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64"),
           "steps": [{"action": "reverse"}]}

    yield {"case_id": "vec3/reverse/subnormal-and-large", "family": "math", "operation": "vec3",
           "x": "0000000000000001", "y": "7fefffffffffffff", "z": "8000000000000001",
           "steps": [{"action": "reverse"}]}

    yield {"case_id": "vec3/reverse/infinities", "family": "math", "operation": "vec3",
           "x": float_bits(float("inf"), "f64"), "y": float_bits(-2.0, "f64"),
           "z": float_bits(float("-inf"), "f64"), "steps": [{"action": "reverse"}]}

    yield {"case_id": "vec3/horizontal/basic", "family": "math", "operation": "vec3",
           "x": float_bits(1.25, "f64"), "y": float_bits(-2.5, "f64"), "z": float_bits(3.75, "f64"),
           "steps": [{"action": "horizontal"}]}

    yield {"case_id": "vec3/horizontal/chain", "family": "math", "operation": "vec3",
           "x": float_bits(-4.5, "f64"), "y": float_bits(6.25, "f64"), "z": float_bits(8.75, "f64"),
           "steps": [{"action": "horizontal"},
                      {"action": "add_components", "x": float_bits(0.5, "f64"),
                       "y": float_bits(-1.5, "f64"), "z": float_bits(-0.75, "f64")}]}

    yield {"case_id": "vec3/horizontal/signed-zero", "family": "math", "operation": "vec3",
           "x": float_bits(-0.0, "f64"), "y": float_bits(-0.0, "f64"), "z": float_bits(-0.0, "f64"),
           "steps": [{"action": "horizontal"}]}

    yield {"case_id": "vec3/horizontal/subnormal-and-large", "family": "math", "operation": "vec3",
           "x": "0000000000000001", "y": "7fefffffffffffff", "z": "8000000000000001",
           "steps": [{"action": "horizontal"}]}

    yield {"case_id": "vec3/horizontal/infinities", "family": "math", "operation": "vec3",
           "x": float_bits(float("inf"), "f64"), "y": float_bits(float("-inf"), "f64"),
           "z": float_bits(float("-inf"), "f64"), "steps": [{"action": "horizontal"}]}

    yield {"case_id": "vec3/multiply/vector-basic", "family": "math", "operation": "vec3",
           "x": float_bits(1.25, "f64"), "y": float_bits(-2.5, "f64"), "z": float_bits(3.75, "f64"),
           "steps": [{"action": "multiply_vec3", "scale": vector_bits(-2.0, 0.5, 4.0)}]}

    yield {"case_id": "vec3/multiply/components-basic", "family": "math", "operation": "vec3",
           "x": float_bits(-1.5, "f64"), "y": float_bits(2.25, "f64"), "z": float_bits(0.75, "f64"),
           "steps": [{"action": "multiply_components", "x": float_bits(0.5, "f64"),
                      "y": float_bits(-4.0, "f64"), "z": float_bits(8.0, "f64")}]}

    yield {"case_id": "vec3/multiply/overload-chain", "family": "math", "operation": "vec3",
           "x": float_bits(2.0 ** 27, "f64"), "y": float_bits(-2.0 ** 27, "f64"), "z": float_bits(1.1, "f64"),
           "steps": [{"action": "multiply_components", "x": float_bits(0.5, "f64"),
                      "y": float_bits(-0.5, "f64"), "z": float_bits(2.0, "f64")},
                      {"action": "multiply_vec3", "scale": vector_bits(1.0, -2.0, 0.25)}]}

    yield {"case_id": "vec3/multiply/signed-zero", "family": "math", "operation": "vec3",
           "x": float_bits(-0.0, "f64"), "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64"),
           "steps": [{"action": "multiply_components", "x": float_bits(-0.0, "f64"),
                      "y": float_bits(-0.0, "f64"), "z": float_bits(0.0, "f64")},
                      {"action": "multiply_vec3", "scale": vector_bits(-1.0, 1.0, -1.0)}]}

    yield {"case_id": "vec3/multiply/overflow-and-special-values", "family": "math", "operation": "vec3",
           "x": "7fefffffffffffff", "y": "ffefffffffffffff", "z": float_bits(1.0, "f64"),
           "steps": [{"action": "multiply_vec3", "scale": {"x": float_bits(2.0, "f64"),
                                                                   "y": float_bits(2.0, "f64"),
                                                                   "z": "7ff8000000000002"}},
                      {"action": "multiply_components", "x": float_bits(1.0, "f64"),
                       "y": float_bits(-1.0, "f64"), "z": float_bits(float("inf"), "f64")}]}

    yield {"case_id": "vec3/add-subtract/vector-chain", "family": "math", "operation": "vec3",
           "x": float_bits(1.25, "f64"), "y": float_bits(-2.5, "f64"), "z": float_bits(3.75, "f64"),
           "steps": [{"action": "add_vec3", "delta": vector_bits(-0.25, 4.0, -1.5)},
                      {"action": "subtract_vec3", "delta": vector_bits(0.5, -1.0, 2.25)},
                      {"action": "normalize"}]}

    yield {"case_id": "vec3/add-subtract/scalar-chain", "family": "math", "operation": "vec3",
           "x": float_bits(-1.5, "f64"), "y": float_bits(2.25, "f64"), "z": float_bits(0.75, "f64"),
           "steps": [{"action": "add_scalar", "value": float_bits(0.5, "f64")},
                      {"action": "subtract_scalar", "value": float_bits(-1.25, "f64")},
                      {"action": "add_scalar", "value": float_bits(-0.0, "f64")}]}

    yield {"case_id": "vec3/add-subtract/component-chain", "family": "math", "operation": "vec3",
           "x": float_bits(10.0, "f64"), "y": float_bits(-20.0, "f64"), "z": float_bits(30.0, "f64"),
           "steps": [{"action": "add_components", "x": float_bits(-1.0, "f64"),
                      "y": float_bits(2.0, "f64"), "z": float_bits(-3.0, "f64")},
                      {"action": "subtract_components", "x": float_bits(4.0, "f64"),
                       "y": float_bits(-5.0, "f64"), "z": float_bits(6.0, "f64")},
                      {"action": "add_vec3", "delta": vector_bits(0.125, -0.25, 0.5)},
                      {"action": "subtract_scalar", "value": float_bits(0.125, "f64")}]}

    yield {"case_id": "vec3/add-subtract/signed-zero", "family": "math", "operation": "vec3",
           "x": float_bits(-0.0, "f64"), "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64"),
           "steps": [{"action": "add_scalar", "value": float_bits(-0.0, "f64")},
                      {"action": "subtract_components", "x": float_bits(0.0, "f64"),
                       "y": float_bits(-0.0, "f64"), "z": float_bits(0.0, "f64")},
                      {"action": "add_vec3", "delta": vector_bits(0.0, -0.0, 0.0)}]}

    yield {"case_id": "vec3/add-subtract/special-values", "family": "math", "operation": "vec3",
           "x": float_bits(1.0, "f64"), "y": float_bits(-2.0, "f64"),
           "z": float_bits(0.0, "f64"),
           "steps": [{"action": "add_vec3", "delta": {"x": float_bits(float("inf"), "f64"),
                                                            "y": float_bits(-float("inf"), "f64"),
                                                            "z": float_bits(1.0, "f64")}},
                      {"action": "subtract_components", "x": float_bits(1.0, "f64"),
                       "y": float_bits(-2.0, "f64"), "z": "7ff8000000000002"}]}

    yield {"case_id": "vec3/vector-to/basic", "family": "math", "operation": "vec3",
           "x": float_bits(1.25, "f64"), "y": float_bits(-2.5, "f64"), "z": float_bits(3.75, "f64"),
           "steps": [{"action": "vector_to", "target": vector_bits(4.5, -1.0, -0.25)}]}

    yield {"case_id": "vec3/vector-to/chain", "family": "math", "operation": "vec3",
           "x": float_bits(2.0, "f64"), "y": float_bits(-3.0, "f64"), "z": float_bits(4.0, "f64"),
           "steps": [{"action": "vector_to", "target": vector_bits(5.0, 1.0, -2.0)},
                      {"action": "vector_to", "target": vector_bits(-1.0, 1.0, 3.0)},
                      {"action": "add_components", "x": float_bits(0.25, "f64"),
                       "y": float_bits(-0.5, "f64"), "z": float_bits(0.75, "f64")},
                      {"action": "normalize"}]}

    yield {"case_id": "vec3/vector-to/signed-zero", "family": "math", "operation": "vec3",
           "x": float_bits(-0.0, "f64"), "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64"),
           "steps": [{"action": "vector_to", "target": vector_bits(0.0, -0.0, 0.0)},
                      {"action": "vector_to", "target": vector_bits(-0.0, 0.0, -0.0)}]}

    yield {"case_id": "vec3/vector-to/overflow", "family": "math", "operation": "vec3",
           "x": "7fefffffffffffff", "y": "8000000000000000", "z": "ffefffffffffffff",
           "steps": [{"action": "vector_to", "target": {"x": "ffefffffffffffff",
                                                                "y": "7fefffffffffffff",
                                                                "z": "7fefffffffffffff"}}]}

    yield {"case_id": "vec3/vector-to/special-values", "family": "math", "operation": "vec3",
           "x": float_bits(1.0, "f64"), "y": float_bits(-2.0, "f64"), "z": float_bits(0.0, "f64"),
           "steps": [{"action": "vector_to", "target": {"x": float_bits(float("inf"), "f64"),
                                                               "y": float_bits(-float("inf"), "f64"),
                                                               "z": "7ff8000000000002"}}]}

    yield {"case_id": "vec3/cross/basic", "family": "math", "operation": "vec3",
           "x": float_bits(1.0, "f64"), "y": float_bits(2.0, "f64"), "z": float_bits(3.0, "f64"),
           "steps": [{"action": "cross", "other": vector_bits(4.0, 5.0, 6.0)}]}

    yield {"case_id": "vec3/cross/chain", "family": "math", "operation": "vec3",
           "x": float_bits(1.0, "f64"), "y": float_bits(2.0, "f64"), "z": float_bits(3.0, "f64"),
           "steps": [{"action": "cross", "other": vector_bits(4.0, 5.0, 6.0)},
                      {"action": "cross", "other": vector_bits(-1.0, 0.5, 2.0)},
                      {"action": "normalize"}]}

    yield {"case_id": "vec3/cross/signed-zero", "family": "math", "operation": "vec3",
           "x": float_bits(0.0, "f64"), "y": float_bits(-0.0, "f64"), "z": float_bits(0.0, "f64"),
           "steps": [{"action": "cross", "other": vector_bits(0.0, 0.0, 0.0)}]}

    yield {"case_id": "vec3/cross/overflow", "family": "math", "operation": "vec3",
           "x": "7fefffffffffffff", "y": float_bits(0.0, "f64"), "z": float_bits(0.0, "f64"),
           "steps": [{"action": "cross", "other": vector_bits(0.0, 0.0, 2.0)}]}

    yield {"case_id": "vec3/cross/special-values", "family": "math", "operation": "vec3",
           "x": float_bits(float("inf"), "f64"), "y": float_bits(2.0, "f64"),
           "z": float_bits(3.0, "f64"),
           "steps": [{"action": "cross", "other": vector_bits(0.0, 0.0, 5.0)}]}
