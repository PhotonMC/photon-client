"""Original synthetic inputs only. Expected values always come from the Java JAR."""

import struct


def float_bits(value, width="f32"):
    return struct.pack(">f" if width == "f32" else ">d", value).hex()


def box_bits(min_x, min_y, min_z, max_x, max_y, max_z):
    return {"min_x": float_bits(min_x, "f64"), "min_y": float_bits(min_y, "f64"),
            "min_z": float_bits(min_z, "f64"), "max_x": float_bits(max_x, "f64"),
            "max_y": float_bits(max_y, "f64"), "max_z": float_bits(max_z, "f64")}


def vector_bits(x, y, z, width="f64"):
    return {"x": float_bits(x, width), "y": float_bits(y, width),
            "z": float_bits(z, width)}


def raw_box(min_x, min_y, min_z, max_x, max_y, max_z):
    return {"min_x": min_x, "min_y": min_y, "min_z": min_z,
            "max_x": max_x, "max_y": max_y, "max_z": max_z}


def neighbors(value, width):
    bits = int(float_bits(value, width), 16)
    digits = 8 if width == "f32" else 16
    return [f"{bits + offset:0{digits}x}" for offset in (-1, 0, 1)]


def wire_cases():
    for codec, width, limit in [("varint", 32, 5), ("varlong", 64, 10)]:
        values = {0, 1, -1, -(1 << (width - 1)), (1 << (width - 1)) - 1}
        for shift in range(7, width - 1, 7):
            for delta in (-1, 0, 1):
                values.add((1 << shift) + delta)
                values.add(-((1 << shift) + delta))
        for value in sorted(values):
            yield {"case_id": f"{codec}/write/{value}", "family": "wire",
                   "operation": "write", "codec": codec, "value": str(value)}
        # These byte sequences are authored independently of both implementations.
        reads = {"empty": "", "zero": "00", "one": "01", "boundary": "8001",
                 "trailing": "012abb", "noncanonical-zero": "8000",
                 "noncanonical-one": "8100"}
        for count in range(1, limit + 2):
            reads[f"truncated-{count}"] = "80" * count
            reads[f"terminated-{count}"] = "80" * count + "00aa"
        for last in (0, 1, 15, 16, 127, 128, 255):
            reads[f"last-byte-{last:02x}"] = "ff" * (limit - 1) + f"{last:02x}" + "55"
        for label, value in reads.items():
            yield {"case_id": f"{codec}/read/{label}", "family": "wire",
                   "operation": "read", "codec": codec, "hex": value}


def math_cases():
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
    vectors3 = {f"direction-{x}-{y}-{z}": (float_bits(x, "f64"), float_bits(y, "f64"), float_bits(z, "f64"))
                for x in (-1, 0, 1) for y in (-1, 0, 1) for z in (-1, 0, 1)}
    vectors3.update({"negative-zero": ("8000000000000000", "8000000000000000", "8000000000000000"),
                     "subnormal": ("0000000000000001", "0000000000000001", "0000000000000001"),
                     "max-finite": ("7fefffffffffffff", "7fefffffffffffff", "7fefffffffffffff"),
                     "unequal": (float_bits(3, "f64"), float_bits(-7, "f64"), float_bits(0.5, "f64"))})
    threshold = struct.unpack(">d", struct.pack(">d", struct.unpack(">f", struct.pack(">f", 1e-5))[0]))[0]
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
    for name in ("down", "up", "north", "south", "west", "east"):
        yield {"case_id": f"direction/properties/{name}", "family": "math", "operation": "direction",
               "name": name, "steps": []}
    for name in ("down", "up", "north", "south", "west", "east"):
        yield {"case_id": f"direction/rotate-y/{name}", "family": "math", "operation": "direction",
               "name": name, "steps": [{"axis": "y", "clockwise": True},
                                       {"axis": "y", "clockwise": True}]}
    for name, axis in (("north", "x"), ("north", "z"), ("down", "x"),
                       ("up", "z"), ("west", "x"), ("east", "z")):
        yield {"case_id": f"direction/rotate-{axis}/{name}", "family": "math", "operation": "direction",
               "name": name, "steps": [{"axis": axis, "clockwise": True}]}
    coords = [0, 1, 15, 16, 17, -1, -15, -16, -17, 31, 32, -32, -33,
              2147483647, -2147483648, 33554431, -33554432, 2097151, -2097152]
    for fn in ("block_to_section", "section_to_block", "section_relative"):
        for value in coords:
            yield {"case_id": f"coord/{fn}/{value}", "family": "math", "operation": "coord",
                   "fn": fn, "value": str(value)}
    blockpos_steps = {
        "basic": [
            {"action": "offset", "x": "0", "y": "0", "z": "0"},
            {"action": "offset", "x": "1", "y": "-2", "z": "3"},
            {"action": "min", "x": "-4", "y": "5", "z": "2"},
            {"action": "max", "x": "8", "y": "-9", "z": "10"},
        ],
        "negative-boundaries": [
            {"action": "offset", "x": "16", "y": "16", "z": "16"},
            {"action": "min", "x": "-32", "y": "-15", "z": "-1"},
            {"action": "max", "x": "-16", "y": "17", "z": "31"},
        ],
        "overflow": [
            {"action": "offset", "x": "1", "y": "-1", "z": "1"},
            {"action": "offset", "x": "-2", "y": "2", "z": "-2"},
        ],
        "reversed-min-max": [
            {"action": "max", "x": "-20", "y": "30", "z": "-40"},
            {"action": "min", "x": "20", "y": "-30", "z": "40"},
        ],
    }
    blockpos_origins = {
        "basic": (0, 0, 0),
        "negative-boundaries": (-17, -16, -1),
        "overflow": (2147483647, -2147483648, 2147483647),
        "reversed-min-max": (9, -4, 20),
    }
    for label, steps in blockpos_steps.items():
        x, y, z = blockpos_origins[label]
        yield {"case_id": f"blockpos/chain/{label}", "family": "math", "operation": "blockpos",
               "x": str(x), "y": str(y), "z": str(z), "steps": steps}
    for label, (x, y, z) in {
        "origin": (0, 0, 0),
        "positive-boundary": (15, 15, 15),
        "section-next": (16, 16, 16),
        "negative-one": (-1, -1, -1),
        "negative-section": (-17, -16, -1),
        "integer-boundaries": (2147483647, -2147483648, 31),
    }.items():
        yield {"case_id": f"section-relative/{label}", "family": "math", "operation": "section_relative",
               "x": str(x), "y": str(y), "z": str(z)}
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
    yield {"case_id": "shape/basic/block", "family": "math", "operation": "shape",
           "kind": "single", "box": box_bits(0, 0, 0, 1, 1, 1), "steps": []}
    yield {"case_id": "shape/basic/move", "family": "math", "operation": "shape",
           "kind": "single", "box": box_bits(-0.25, 0.125, 0.0, 0.75, 0.875, 1.25),
           "steps": [{"op": "move", "dx": float_bits(0.5, "f64"),
                      "dy": float_bits(-0.125, "f64"), "dz": float_bits(2, "f64")}]}
    yield {"case_id": "shape/move-overload/vec3", "family": "math", "operation": "shape",
           "kind": "single", "box": box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875),
           "steps": [{"op": "move_vec3", "delta": {"x": float_bits(-0.5, "f64"),
                                                        "y": float_bits(1.25, "f64"),
                                                        "z": float_bits(-1.75, "f64")}}]}
    yield {"case_id": "shape/move-overload/block-pos", "family": "math", "operation": "shape",
           "kind": "single", "box": box_bits(-3.5, -2.25, 4.75, -1.0, 0.5, 6.25),
           "steps": [{"op": "move_block_pos", "delta": {"x": "-2", "y": "3", "z": "4"}}]}
    yield {"case_id": "shape/move-overload/empty", "family": "math", "operation": "shape",
           "kind": "single", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "steps": [{"op": "move_vec3", "delta": {"x": float_bits(float("inf"), "f64"),
                                                        "y": float_bits(-float("inf"), "f64"),
                                                        "z": float_bits(float("nan"), "f64")}},
                      {"op": "move_block_pos", "delta": {"x": "2147483647", "y": "-2147483648", "z": "0"}}]}
    yield {"case_id": "shape/optimize/empty", "family": "math", "operation": "shape",
           "kind": "optimize", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/optimize/touching-row", "family": "math", "operation": "shape",
           "kind": "optimize", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.25, 1.0, 1.0),
               box_bits(0.25, 0.0, 0.0, 0.5, 1.0, 1.0),
               box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/optimize/disjoint", "family": "math", "operation": "shape",
           "kind": "optimize", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.25, 0.5, 0.75),
               box_bits(0.5, 0.25, 0.25, 0.75, 1.0, 0.5),
               box_bits(0.75, 0.0, 0.75, 1.0, 0.25, 1.0),
           ]}
    yield {"case_id": "shape/edges/empty", "family": "math", "operation": "shape",
           "kind": "edges", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/edges/box", "family": "math", "operation": "shape",
           "kind": "edges", "shapes": [
               box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875),
           ]}
    yield {"case_id": "shape/edges/join", "family": "math", "operation": "shape",
           "kind": "edges", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.5, 0.5, 0.5),
               box_bits(0.5, 0.25, 0.25, 1.0, 0.75, 0.75),
               box_bits(0.25, 0.5, 0.5, 0.75, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/join-coords/touching", "family": "math", "operation": "shape",
           "kind": "join_coords", "op": "or",
           "first": box_bits(0.0, 0.0, 0.0, 0.25, 1.0, 1.0),
           "second": box_bits(0.25, 0.0, 0.0, 0.5, 1.0, 1.0)}
    yield {"case_id": "shape/join-coords/hole", "family": "math", "operation": "shape",
           "kind": "join_coords", "op": "only_first",
           "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "second": box_bits(0.25, 0.0, 0.0, 0.75, 1.0, 1.0)}
    yield {"case_id": "shape/join-coords/empty-first", "family": "math", "operation": "shape",
           "kind": "join_coords", "op": "or",
           "first": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "second": box_bits(2.0, -1.0, 3.0, 2.5, 0.5, 4.0)}
    yield {"case_id": "shape/boxes/empty", "family": "math", "operation": "shape",
           "kind": "boxes", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/boxes/box", "family": "math", "operation": "shape",
           "kind": "boxes", "shapes": [
               box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875),
           ]}
    yield {"case_id": "shape/boxes/join", "family": "math", "operation": "shape",
           "kind": "boxes", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.25, 0.5, 0.75),
               box_bits(0.5, 0.25, 0.25, 0.75, 1.0, 0.5),
               box_bits(0.75, 0.0, 0.75, 1.0, 0.25, 1.0),
           ]}
    yield {"case_id": "shape/create-aabb/reversed", "family": "math", "operation": "shape",
           "kind": "create_aabb", "aabb": box_bits(
               0.75, 0.5, 0.875, 0.125, 0.25, 0.375)}
    yield {"case_id": "shape/create-aabb/empty", "family": "math", "operation": "shape",
           "kind": "create_aabb", "aabb": box_bits(
               0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}
    yield {"case_id": "shape/create-aabb/array", "family": "math", "operation": "shape",
           "kind": "create_aabb", "aabb": box_bits(
               -1.25, 0.125, 2.0, 1.5, 0.875, 3.25)}
    yield {"case_id": "shape/create-coords/cube", "family": "math", "operation": "shape",
           "kind": "create_coords", "box": box_bits(
               0.125, 0.25, 0.375, 0.625, 0.75, 0.875)}
    yield {"case_id": "shape/create-coords/reversed", "family": "math", "operation": "shape",
           "kind": "create_coords", "box": box_bits(
               0.75, 0.5, 0.875, 0.125, 0.25, 0.375)}
    yield {"case_id": "shape/create-coords/array", "family": "math", "operation": "shape",
           "kind": "create_coords", "box": box_bits(
               -1.25, 0.125, 2.0, 1.5, 0.875, 3.25)}
    yield {"case_id": "shape/infinity/snapshot", "family": "math", "operation": "shape",
           "kind": "infinity", "view": "snapshot"}
    yield {"case_id": "shape/infinity/boxes", "family": "math", "operation": "shape",
           "kind": "infinity", "view": "boxes"}
    yield {"case_id": "shape/infinity/not-same-self", "family": "math", "operation": "shape",
           "kind": "infinity", "view": "not_same_self"}
    yield {"case_id": "shape/constants/epsilon", "family": "math", "operation": "shape",
           "kind": "constants", "name": "epsilon"}
    yield {"case_id": "shape/constants/big-epsilon", "family": "math", "operation": "shape",
           "kind": "constants", "name": "big_epsilon"}
    yield {"case_id": "shape/constants/both", "family": "math", "operation": "shape",
           "kind": "constants", "name": "both"}
    yield {"case_id": "shape/empty/sliver", "family": "math", "operation": "shape",
           "kind": "single", "box": box_bits(0, 0, 0, 0.00000005, 1, 1), "steps": []}
    yield {"case_id": "shape/or/disjoint", "family": "math", "operation": "shape", "kind": "join",
           "op": "or", "first": box_bits(0, 0, 0, 0.25, 1, 1),
           "second": box_bits(0.75, 0, 0, 1, 1, 1)}
    yield {"case_id": "shape/or/touching", "family": "math", "operation": "shape", "kind": "join",
           "op": "or", "first": box_bits(0, 0, 0, 0.5, 1, 1),
           "second": box_bits(0.5, 0, 0, 1, 1, 1)}
    yield {"case_id": "shape/and/overlap", "family": "math", "operation": "shape", "kind": "join",
           "op": "and", "first": box_bits(0, 0, 0, 0.75, 1, 1),
           "second": box_bits(0.25, 0, 0, 1, 1, 1)}
    yield {"case_id": "shape/and/disjoint", "family": "math", "operation": "shape", "kind": "join",
           "op": "and", "first": box_bits(0, 0, 0, 0.25, 1, 1),
           "second": box_bits(0.75, 0, 0, 1, 1, 1)}
    yield {"case_id": "shape/only-first/hole", "family": "math", "operation": "shape", "kind": "join",
           "op": "only_first", "first": box_bits(0, 0, 0, 0.75, 1, 1),
           "second": box_bits(0.25, 0, 0, 0.5, 1, 1)}
    yield {"case_id": "shape/only-second/disjoint", "family": "math", "operation": "shape", "kind": "join",
           "op": "only_second", "first": box_bits(0, 0, 0, 0.25, 1, 1),
           "second": box_bits(0.75, 0, 0, 1, 1, 1)}
    yield {"case_id": "shape/false/overlap", "family": "math", "operation": "shape", "kind": "join",
           "op": "false", "first": box_bits(0, 0, 0, 0.75, 1, 1),
           "second": box_bits(0.25, 0, 0, 1, 1, 1)}
    yield {"case_id": "shape/not-same/overlap", "family": "math", "operation": "shape", "kind": "join",
           "op": "not_same", "first": box_bits(0, 0, 0, 0.75, 1, 1),
           "second": box_bits(0.25, 0, 0, 1, 1, 1)}
    yield {"case_id": "shape/second/displaced", "family": "math", "operation": "shape", "kind": "join",
           "op": "second", "first": box_bits(0, 0, 0, 0.25, 1, 1),
           "second": box_bits(0.75, 0, 0, 1, 1, 1)}
    yield {"case_id": "shape/first/displaced", "family": "math", "operation": "shape", "kind": "join",
           "op": "first", "first": box_bits(0.75, 0, 0, 1, 1, 1),
           "second": box_bits(0, 0, 0, 0.25, 1, 1)}
    yield {"case_id": "shape/or-many/touching-row", "family": "math", "operation": "shape",
           "kind": "join_many", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.25, 1.0, 1.0),
               box_bits(0.25, 0.0, 0.0, 0.5, 1.0, 1.0),
               box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/or-many/disjoint", "family": "math", "operation": "shape",
           "kind": "join_many", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.25, 0.25, 0.25),
               box_bits(0.5, 0.5, 0.5, 0.75, 0.75, 0.75),
               box_bits(0.75, 0.0, 0.75, 1.0, 0.25, 1.0),
           ]}
    yield {"case_id": "shape/or-many/empty-first", "family": "math", "operation": "shape",
           "kind": "join_many", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
               box_bits(0.25, 0.0, 0.0, 0.5, 1.0, 1.0),
               box_bits(0.5, 0.0, 0.0, 0.75, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/single-encompassing/empty", "family": "math", "operation": "shape",
           "kind": "single_encompassing", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/single-encompassing/touching", "family": "math", "operation": "shape",
           "kind": "single_encompassing", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.5, 1.0, 1.0),
               box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/single-encompassing/disjoint", "family": "math", "operation": "shape",
           "kind": "single_encompassing", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.25, 0.5, 0.75),
               box_bits(0.75, 0.25, 0.5, 1.0, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/axis-limits/empty", "family": "math", "operation": "shape",
           "kind": "axis_limits", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           ]}
    yield {"case_id": "shape/axis-limits/disjoint", "family": "math", "operation": "shape",
           "kind": "axis_limits", "shapes": [
               box_bits(0.25, 0.0, 0.5, 0.5, 0.25, 0.75),
               box_bits(0.75, 0.5, -0.5, 1.0, 1.0, 0.0),
           ]}
    yield {"case_id": "shape/axis-limits/negative", "family": "math", "operation": "shape",
           "kind": "axis_limits", "shapes": [
               box_bits(-2.0, -0.75, 1.5, -1.25, -0.25, 2.0),
               box_bits(-0.5, 0.5, -2.0, 0.25, 1.25, -1.5),
           ]}
    yield {"case_id": "shape/axis-limits-at/empty", "family": "math", "operation": "shape",
           "kind": "axis_limits_at", "shapes": [
               box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           ], "queries": [
               {"op": "min", "axis": "x", "b": float_bits(0.5, "f64"),
                "c": float_bits(0.5, "f64")},
               {"op": "max", "axis": "z", "b": float_bits(0.5, "f64"),
                "c": float_bits(0.5, "f64")},
           ]}
    yield {"case_id": "shape/axis-limits-at/disjoint", "family": "math", "operation": "shape",
           "kind": "axis_limits_at", "shapes": [
               box_bits(0.25, 0.0, 0.5, 0.5, 0.25, 0.75),
               box_bits(0.75, 0.5, -0.5, 1.0, 1.0, 0.0),
           ], "queries": [
               {"op": "min", "axis": "x", "b": float_bits(0.125, "f64"),
                "c": float_bits(0.625, "f64")},
               {"op": "max", "axis": "x", "b": float_bits(0.125, "f64"),
                "c": float_bits(0.625, "f64")},
               {"op": "min", "axis": "y", "b": float_bits(-0.25, "f64"),
                "c": float_bits(0.875, "f64")},
               {"op": "max", "axis": "y", "b": float_bits(-0.25, "f64"),
                "c": float_bits(0.875, "f64")},
               {"op": "min", "axis": "z", "b": float_bits(0.375, "f64"),
                "c": float_bits(0.125, "f64")},
               {"op": "max", "axis": "z", "b": float_bits(0.375, "f64"),
                "c": float_bits(0.125, "f64")},
               {"op": "min", "axis": "x", "b": float_bits(0.75, "f64"),
                "c": float_bits(0.25, "f64")},
           ]}
    yield {"case_id": "shape/axis-limits-at/negative", "family": "math", "operation": "shape",
           "kind": "axis_limits_at", "shapes": [
               box_bits(-2.0, -0.75, 1.5, -1.25, -0.25, 2.0),
               box_bits(-0.5, 0.5, -2.0, 0.25, 1.25, -1.5),
           ], "queries": [
               {"op": "min", "axis": "x", "b": float_bits(-0.5, "f64"),
                "c": float_bits(1.75, "f64")},
               {"op": "max", "axis": "x", "b": float_bits(-0.5, "f64"),
                "c": float_bits(1.75, "f64")},
               {"op": "min", "axis": "y", "b": float_bits(1.75, "f64"),
                "c": float_bits(-1.5, "f64")},
               {"op": "max", "axis": "y", "b": float_bits(1.75, "f64"),
                "c": float_bits(-1.5, "f64")},
               {"op": "min", "axis": "z", "b": float_bits(-1.5, "f64"),
                "c": float_bits(-0.5, "f64")},
               {"op": "max", "axis": "z", "b": float_bits(-1.5, "f64"),
                "c": float_bits(-0.5, "f64")},
           ]}
    yield {"case_id": "shape/equal/same-box", "family": "math", "operation": "shape", "kind": "equal",
           "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "second": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)}
    yield {"case_id": "shape/equal/same-translated-box", "family": "math", "operation": "shape", "kind": "equal",
           "first": box_bits(-2.0, 1.0, 3.0, -1.0, 2.0, 4.0),
           "second": box_bits(-2.0, 1.0, 3.0, -1.0, 2.0, 4.0)}
    yield {"case_id": "shape/equal/different-size", "family": "math", "operation": "shape", "kind": "equal",
           "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "second": box_bits(0.0, 0.0, 0.0, 0.5, 1.0, 1.0)}
    yield {"case_id": "shape/equal/disjoint", "family": "math", "operation": "shape", "kind": "equal",
           "first": box_bits(0.0, 0.0, 0.0, 0.5, 1.0, 1.0),
           "second": box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0)}
    yield {"case_id": "shape/equal/partial-overlap", "family": "math", "operation": "shape", "kind": "equal",
           "first": box_bits(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
           "second": box_bits(0.25, 0.0, 0.0, 1.0, 1.0, 1.0)}
    yield {"case_id": "shape/equal/both-empty", "family": "math", "operation": "shape", "kind": "equal",
           "first": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "second": box_bits(-2.0, -1.0, 3.0, -1.99999995, 0.0, 4.0)}
    object_equals_cube = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875)
    yield {"case_id": "shape/object-equals/cube", "family": "math", "operation": "shape",
           "kind": "object_equals", "shape": object_equals_cube, "same": object_equals_cube,
           "different": box_bits(0.125, 0.25, 0.375, 0.75, 0.75, 0.875)}
    object_equals_array = box_bits(-2.0, -0.75, 1.5, -1.25, -0.25, 2.0)
    yield {"case_id": "shape/object-equals/array", "family": "math", "operation": "shape",
           "kind": "object_equals", "shape": object_equals_array, "same": object_equals_array,
           "different": box_bits(-2.0, -0.75, 1.5, -1.25, -0.25, 2.25)}
    yield {"case_id": "shape/object-equals/constants", "family": "math", "operation": "shape",
           "kind": "object_equals_constants"}
    yield {"case_id": "shape/to-string/empty", "family": "math", "operation": "shape",
           "kind": "to_string", "shape": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}
    yield {"case_id": "shape/to-string/cube", "family": "math", "operation": "shape",
           "kind": "to_string", "shape": box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875)}
    yield {"case_id": "shape/to-string/scientific", "family": "math", "operation": "shape",
           "kind": "to_string", "shape": box_bits(0.0001, 10000000.0, -0.0,
                                                        0.0002, 10000001.0, 2.0)}
    yield {"case_id": "shape/to-string/infinity", "family": "math", "operation": "shape",
           "kind": "to_string", "shape": box_bits(float("-inf"), float("-inf"), float("-inf"),
                                                      float("inf"), float("inf"), float("inf"))}
    yield {"case_id": "shape/to-string/nan", "family": "math", "operation": "shape",
           "kind": "to_string", "shape": box_bits(float("nan"), 0.0, 0.0, 1.0, 1.0, 1.0)}
    rotate_box = box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875)
    yield {"case_id": "shape/rotate/identity", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "IDENTITY"}
    yield {"case_id": "shape/rotate/x-90", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_X_NEG"}
    yield {"case_id": "shape/rotate/y-90", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_Y_NEG"}
    yield {"case_id": "shape/rotate/z-90", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_Z_NEG"}
    yield {"case_id": "shape/rotate/x-180", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_180_FACE_YZ"}
    yield {"case_id": "shape/rotate/y-180", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_180_FACE_XZ"}
    yield {"case_id": "shape/rotate/z-180", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_180_FACE_XY"}
    yield {"case_id": "shape/rotate/x-90-positive", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_X_POS"}
    yield {"case_id": "shape/rotate/y-90-positive", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_Y_POS"}
    yield {"case_id": "shape/rotate/z-90-positive", "family": "math", "operation": "shape", "kind": "rotate",
           "box": rotate_box, "rotation": "ROT_90_Z_POS"}
    yield {"case_id": "shape/rotate/cube-grid", "family": "math", "operation": "shape", "kind": "rotate",
           "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0), "rotation": "ROT_90_Y_NEG"}
    yield {"case_id": "shape/rotate/joined-cells", "family": "math", "operation": "shape", "kind": "rotate_join",
           "first": box_bits(0.0, 0.0, 0.0, 0.5, 0.5, 1.0),
           "second": box_bits(0.5, 0.5, 0.0, 1.0, 1.0, 1.0), "rotation": "ROT_90_Z_NEG"}
    def center_bits(x, y, z):
        return {"x": float_bits(x, "f64"), "y": float_bits(y, "f64"), "z": float_bits(z, "f64")}

    yield {"case_id": "shape/rotate-center/y-asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_center", "box": rotate_box, "rotation": "ROT_90_Y_NEG",
           "center": center_bits(0.25, 0.5, 0.75)}
    yield {"case_id": "shape/rotate-center/x-cube", "family": "math", "operation": "shape",
           "kind": "rotate_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "rotation": "ROT_90_X_NEG", "center": center_bits(0.25, 0.5, 0.75)}
    yield {"case_id": "shape/rotate-center/z-grid", "family": "math", "operation": "shape",
           "kind": "rotate_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "rotation": "ROT_90_Z_POS", "center": center_bits(0.75, 0.25, 0.5)}
    yield {"case_id": "shape/rotate-center/identity", "family": "math", "operation": "shape",
           "kind": "rotate_center", "box": rotate_box, "rotation": "IDENTITY",
           "center": center_bits(2.0, -1.0, 3.0)}
    yield {"case_id": "shape/rotate-all-center/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_all_center", "box": rotate_box,
           "center": center_bits(0.25, 0.5, 0.75)}
    yield {"case_id": "shape/rotate-all-center/grid", "family": "math", "operation": "shape",
           "kind": "rotate_all_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "center": center_bits(0.75, 0.25, 0.5)}
    yield {"case_id": "shape/rotate-all-center/empty", "family": "math", "operation": "shape",
           "kind": "rotate_all_center", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "center": center_bits(2.0, -1.0, 3.0)}
    yield {"case_id": "shape/rotate-horizontal-axis/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis", "box": rotate_box}
    yield {"case_id": "shape/rotate-horizontal-axis/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}
    yield {"case_id": "shape/rotate-horizontal-axis/empty", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}
    yield {"case_id": "shape/rotate-horizontal-axis-center/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis_center", "box": rotate_box,
           "center": center_bits(0.25, 0.5, 0.75)}
    yield {"case_id": "shape/rotate-horizontal-axis-center/grid", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "center": center_bits(0.75, 0.25, 0.5)}
    yield {"case_id": "shape/rotate-horizontal-axis-center/empty", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_axis_center", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "center": center_bits(2.0, -1.0, 3.0)}
    yield {"case_id": "shape/rotate-all-axis/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis", "box": rotate_box}
    yield {"case_id": "shape/rotate-all-axis/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}
    yield {"case_id": "shape/rotate-all-axis/empty", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}
    yield {"case_id": "shape/rotate-all-axis-center/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis_center", "box": rotate_box,
           "center": center_bits(0.25, 0.5, 0.75)}
    yield {"case_id": "shape/rotate-all-axis-center/grid", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis_center", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "center": center_bits(0.75, 0.25, 0.5)}
    yield {"case_id": "shape/rotate-all-axis-center/empty", "family": "math", "operation": "shape",
           "kind": "rotate_all_axis_center", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "center": center_bits(2.0, -1.0, 3.0)}
    yield {"case_id": "shape/rotate-horizontal/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal", "box": rotate_box}
    yield {"case_id": "shape/rotate-horizontal/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}
    yield {"case_id": "shape/rotate-horizontal/empty", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}
    yield {"case_id": "shape/rotate-horizontal-initial/identity", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box, "initial": "IDENTITY"}
    yield {"case_id": "shape/rotate-horizontal-initial/x-90", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box, "initial": "ROT_90_X_NEG"}
    yield {"case_id": "shape/rotate-horizontal-initial/y-180", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "initial": "ROT_180_FACE_XZ"}
    yield {"case_id": "shape/rotate-horizontal-initial/y-90", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box,
           "initial": "ROT_90_Y_NEG"}
    yield {"case_id": "shape/rotate-horizontal-initial/z-90", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box,
           "initial": "ROT_90_Z_NEG"}
    yield {"case_id": "shape/rotate-horizontal-initial/xy-180", "family": "math", "operation": "shape",
           "kind": "rotate_horizontal_initial", "box": rotate_box,
           "initial": "ROT_180_FACE_XY"}
    yield {"case_id": "shape/rotate-all-initial/identity", "family": "math", "operation": "shape",
           "kind": "rotate_all_initial", "box": rotate_box, "initial": "IDENTITY",
           "center": center_bits(0.25, 0.5, 0.75)}
    yield {"case_id": "shape/rotate-all-initial/x-90", "family": "math", "operation": "shape",
           "kind": "rotate_all_initial", "box": rotate_box, "initial": "ROT_90_X_NEG",
           "center": center_bits(0.25, 0.5, 0.75)}
    yield {"case_id": "shape/rotate-all-initial/y-180", "family": "math", "operation": "shape",
           "kind": "rotate_all_initial", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "initial": "ROT_180_FACE_XZ", "center": center_bits(0.75, 0.25, 0.5)}
    yield {"case_id": "shape/rotate-all/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_all", "box": rotate_box}
    yield {"case_id": "shape/rotate-all/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_all", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}
    yield {"case_id": "shape/rotate-all/empty", "family": "math", "operation": "shape",
           "kind": "rotate_all", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}
    yield {"case_id": "shape/rotate-attach-face/asymmetric", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face", "box": rotate_box}
    yield {"case_id": "shape/rotate-attach-face/cube-grid", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}
    yield {"case_id": "shape/rotate-attach-face/empty", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face", "box": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)}
    yield {"case_id": "shape/rotate-attach-face-initial/identity", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face_initial", "box": rotate_box, "initial": "IDENTITY"}
    yield {"case_id": "shape/rotate-attach-face-initial/x-90", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face_initial", "box": rotate_box, "initial": "ROT_90_X_NEG"}
    yield {"case_id": "shape/rotate-attach-face-initial/y-180", "family": "math", "operation": "shape",
           "kind": "rotate_attach_face_initial", "box": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "initial": "ROT_180_FACE_XZ"}
    yield {"case_id": "shape/collide/positive-x", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(4, "f64")}
    yield {"case_id": "shape/collide/negative-x", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(2, 0.25, 0.25, 3, 0.75, 0.75), "distance": float_bits(-4, "f64")}
    yield {"case_id": "shape/collide/positive-y", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "y", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(0.25, -2, 0.25, 0.75, -1, 0.75), "distance": float_bits(4, "f64")}
    yield {"case_id": "shape/collide/negative-z", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "z", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(0.25, 0.25, 2, 0.75, 0.75, 3), "distance": float_bits(-4, "f64")}
    yield {"case_id": "shape/collide/orthogonal-gap", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(-2, 2, 0.25, -1, 3, 0.75), "distance": float_bits(4, "f64")}
    yield {"case_id": "shape/collide/zero-distance", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(0, "f64")}
    yield {"case_id": "shape/collide/empty-shape", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 0.00000005, 1, 1),
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(4, "f64")}
    yield {"case_id": "shape/collide/limited-distance", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(0.5, "f64")}
    yield {"case_id": "shape/collide-many/positive-nearest", "family": "math", "operation": "shape", "kind": "collide_many",
           "axis": "x", "shapes": [box_bits(2, 0, 0, 3, 1, 1), box_bits(0, 0, 0, 1, 1, 1)],
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(4, "f64")}
    yield {"case_id": "shape/collide-many/negative-nearest", "family": "math", "operation": "shape", "kind": "collide_many",
           "axis": "x", "shapes": [box_bits(-2, 0, 0, -1, 1, 1), box_bits(0, 0, 0, 1, 1, 1)],
           "moving": box_bits(2, 0.25, 0.25, 3, 0.75, 0.75), "distance": float_bits(-4, "f64")}
    yield {"case_id": "shape/collide-many/empty-first", "family": "math", "operation": "shape", "kind": "collide_many",
           "axis": "x", "shapes": [box_bits(0, 0, 0, 0.00000005, 1, 1), box_bits(0, 0, 0, 1, 1, 1)],
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(4, "f64")}
    yield {"case_id": "shape/collide-many/zero-distance", "family": "math", "kind": "collide_many", "operation": "shape",
           "axis": "x", "shapes": [box_bits(0, 0, 0, 1, 1, 1)],
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(0, "f64")}
    yield {"case_id": "shape/collide-many/empty-list", "family": "math", "operation": "shape", "kind": "collide_many",
           "axis": "z", "shapes": [], "moving": box_bits(0, 0, 2, 1, 1, 3), "distance": float_bits(-4, "f64")}
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
    unit = {"min_x": float_bits(0.0, "f64"), "min_y": float_bits(0.0, "f64"),
            "min_z": float_bits(0.0, "f64"), "max_x": float_bits(1.0, "f64"),
            "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64")}
    inside = {"op": "contains", "x": float_bits(0.5, "f64"),
              "y": float_bits(0.5, "f64"), "z": float_bits(0.5, "f64")}
    outside = {"op": "contains", "x": float_bits(2.0, "f64"),
               "y": float_bits(0.0, "f64"), "z": float_bits(0.0, "f64")}
    def block_position(x, y, z):
        return {"x": str(x), "y": str(y), "z": str(z)}

    yield {"case_id": "aabb-builder/empty", "family": "math", "operation": "aabb_builder",
           "points": []}
    yield {"case_id": "aabb-builder/single", "family": "math", "operation": "aabb_builder",
           "points": [vector_bits(1.25, -2.5, 3.75, "f32")]}
    yield {"case_id": "aabb-builder/multiple", "family": "math", "operation": "aabb_builder",
           "points": [vector_bits(3.5, -1.25, 8.0, "f32"),
                      vector_bits(-4.0, 2.5, -6.75, "f32"),
                      vector_bits(1.0, -3.0, 4.25, "f32")]}
    yield {"case_id": "aabb-builder/float-precision", "family": "math", "operation": "aabb_builder",
           "points": [vector_bits(0.2, -0.3, 123456.78, "f32"),
                      vector_bits(-1.0 / 3.0, 1.0 / 7.0, -0.0, "f32")]}
    yield {"case_id": "aabb-builder/special-values", "family": "math", "operation": "aabb_builder",
           "points": [vector_bits(-0.0, 0.0, -0.0, "f32"),
                      vector_bits(0.0, -0.0, 0.0, "f32"),
                      vector_bits(float("nan"), float("inf"), float("-inf"), "f32")]}

    yield {"case_id": "aabb/contains/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [inside, outside]}
    yield {"case_id": "aabb/intersects/overlap", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersects", "box": dict(unit)},
                             {"op": "intersects", "box": {
                                 "min_x": float_bits(2.0, "f64"), "min_y": float_bits(2.0, "f64"),
                                 "min_z": float_bits(2.0, "f64"), "max_x": float_bits(3.0, "f64"),
                                 "max_y": float_bits(3.0, "f64"), "max_z": float_bits(3.0, "f64")}}]}
    yield {"case_id": "aabb/predicate-overload/basic", "family": "math", "operation": "aabb",
           **box_bits(-2.0, -2.0, -2.0, 2.0, 2.0, 2.0),
           "steps": [{"op": "contains_vec3", "position": vector_bits(0.5, -0.5, 1.5)},
                      {"op": "intersects_coords", **box_bits(-1.0, -1.0, -1.0, 0.0, 0.0, 0.0)},
                      {"op": "intersects_vec3", "min": vector_bits(1.0, 1.0, 1.0),
                       "max": vector_bits(-1.0, -1.0, -1.0)},
                      {"op": "intersects_block_pos", "position": block_position(-1, -1, -1)}]}
    yield {"case_id": "aabb/predicate-overload/contains-boundaries", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contains_vec3", "position": vector_bits(0.0, 0.0, 0.0)},
                              {"op": "contains_vec3", "position": vector_bits(1.0, 0.5, 0.5)},
                              {"op": "contains_vec3", "position": vector_bits(float("nan"), 0.5, 0.5)},
                              {"op": "contains_vec3", "position": vector_bits(float("inf"), 0.5, 0.5)}]}
    yield {"case_id": "aabb/predicate-overload/raw-coordinates", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersects_coords", **box_bits(-0.5, -0.5, -0.5, 0.5, 0.5, 0.5)},
                              {"op": "intersects_coords", **box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)},
                              {"op": "intersects_coords", **box_bits(0.5, 0.5, 0.5, -0.5, -0.5, -0.5)},
                              {"op": "intersects_coords", **box_bits(float("nan"), 0.0, 0.0, 1.0, 1.0, 1.0)}]}
    yield {"case_id": "aabb/predicate-overload/vec3-normalizes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersects_vec3", "min": vector_bits(1.0, 1.0, 1.0),
                               "max": vector_bits(-1.0, -1.0, -1.0)},
                              {"op": "intersects_vec3", "min": vector_bits(0.0, -0.0, 0.0),
                               "max": vector_bits(-0.0, 0.0, -0.0)},
                              {"op": "intersects_vec3", "min": vector_bits(float("nan"), 0.0, 0.0),
                               "max": vector_bits(1.0, 1.0, 1.0)}]}
    yield {"case_id": "aabb/predicate-overload/block-pos-boundaries", "family": "math", "operation": "aabb",
           "min_x": float_bits(-2147483648.0, "f64"), "min_y": float_bits(0.0, "f64"),
           "min_z": float_bits(0.0, "f64"), "max_x": float_bits(-2147483647.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "intersects_block_pos", "position": block_position(-2147483648, 0, 0)},
                      {"op": "intersects_block_pos", "position": block_position(2147483647, 0, 0)}]}
    yield {"case_id": "aabb/predicate-overload/special-chain", "family": "math", "operation": "aabb",
           **box_bits(-1.0, -1.0, -1.0, 1.0, 1.0, 1.0),
           "steps": [{"op": "intersects_coords", **box_bits(float("inf"), -1.0, -1.0, float("inf"), 1.0, 1.0)},
                      {"op": "intersects_vec3", "min": vector_bits(-2.0, -2.0, -2.0),
                       "max": vector_bits(2.0, 2.0, 2.0)},
                      {"op": "contains_vec3", "position": vector_bits(-1.0, 0.0, 0.0)},
                      {"op": "intersects_block_pos", "position": block_position(0, 0, 0)}]}
    yield {"case_id": "aabb/collided-along-vector/empty", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(1.0, 0.0, 0.0),
                              "boxes": []}]}
    yield {"case_id": "aabb/collided-along-vector/from-contained", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(0.0, 0.0, 0.0),
                              "boxes": [box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75)]}]}
    yield {"case_id": "aabb/collided-along-vector/to-contained", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(2.0, 0.0, 0.0),
                              "boxes": [box_bits(2.5, 0.0, 0.0, 3.5, 1.0, 1.0)]}]}
    yield {"case_id": "aabb/collided-along-vector/crossing", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(3.0, 0.0, 0.0),
                              "boxes": [box_bits(1.5, 0.0, 0.0, 2.0, 1.0, 1.0)]}]}
    yield {"case_id": "aabb/collided-along-vector/miss", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(3.0, 0.0, 0.0),
                              "boxes": [box_bits(1.5, 2.0, 0.0, 2.0, 3.0, 1.0)]}]}
    yield {"case_id": "aabb/collided-along-vector/many", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(3.0, 0.0, 0.0),
                              "boxes": [box_bits(1.5, 2.0, 0.0, 2.0, 3.0, 1.0),
                                        box_bits(1.5, 0.0, 0.0, 2.0, 1.0, 1.0)]}]}
    yield {"case_id": "aabb/collided-along-vector/epsilon-special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(0.0, 0.0, 0.0),
                              "boxes": [box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)]},
                             {"op": "collided_along_vector", "vector": vector_bits(0.0000002, 0.0, 0.0),
                              "boxes": [box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)]},
                             {"op": "collided_along_vector", "vector": vector_bits(float("nan"), 0.0, 0.0),
                              "boxes": [box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)]}]}
    yield {"case_id": "aabb/collided-along-vector/tiny-moving-box", "family": "math", "operation": "aabb",
           **box_bits(0.0, 0.0, 0.0, 0.00000001, 0.00000001, 0.00000001),
           "steps": [{"op": "collided_along_vector", "vector": vector_bits(1.0, 0.0, 0.0),
                      "boxes": [box_bits(0.5, 0.0, 0.0, 0.6, 1.0, 1.0)]}]}
    yield {"case_id": "aabb/constructor/block-pos", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "from_block_pos", "position": block_position(-2, 3, 4)},
                              {"op": "from_block_pos", "position": block_position(2147483647, -2147483648, 0)}]}
    yield {"case_id": "aabb/constructor/vec3", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "from_vec3", "begin": vector_bits(3.0, -2.0, 1.0),
                               "end": vector_bits(-4.0, 5.0, -6.0)},
                              {"op": "from_vec3", "begin": vector_bits(-0.0, 0.0, -0.0),
                               "end": vector_bits(0.0, -0.0, 0.0)},
                              {"op": "from_vec3", "begin": vector_bits(float("inf"), -float("inf"), float("nan")),
                               "end": vector_bits(0.0, 1.0, 2.0)}]}
    yield {"case_id": "aabb/constructor/unit-cube", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "unit_cube_from_lower_corner", "position": vector_bits(1.25, -2.5, 0.125)},
                              {"op": "unit_cube_from_lower_corner", "position": vector_bits(-0.0, 0.0, -0.0)},
                              {"op": "unit_cube_from_lower_corner", "position": vector_bits(float("inf"), -float("inf"), float("nan"))}]}
    yield {"case_id": "aabb/constructor/full-blocks", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "encapsulating_full_blocks", "first": block_position(-3, 4, 8),
                               "second": block_position(2, -1, 5)},
                              {"op": "encapsulating_full_blocks", "first": block_position(10, 20, 30),
                               "second": block_position(-10, -20, -30)}]}
    yield {"case_id": "aabb/constructor/full-blocks-overflow", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "encapsulating_full_blocks",
                               "first": block_position(2147483647, 2147483647, 2147483647),
                               "second": block_position(2147483647, 2147483647, 2147483647)},
                              {"op": "encapsulating_full_blocks",
                               "first": block_position(-2147483648, -2147483648, -2147483648),
                               "second": block_position(-2147483648, -2147483648, -2147483648)}]}
    yield {"case_id": "aabb/constructor/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "from_block_pos", "position": block_position(2, 3, 4)},
                              {"op": "unit_cube_from_lower_corner", "position": vector_bits(-1.5, 0.25, 2.75)},
                              {"op": "from_vec3", "begin": vector_bits(5.0, 6.0, 7.0),
                               "end": vector_bits(1.0, 2.0, 3.0)},
                              {"op": "encapsulating_full_blocks", "first": block_position(-2, -1, 0),
                               "second": block_position(4, 5, 6)}]}
    yield {"case_id": "aabb/expand-deflate/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "expand", "x": float_bits(1.0, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(0.0, "f64")},
                             {"op": "deflate", "amount": float_bits(0.5, "f64")}]}
    yield {"case_id": "aabb/expand-overload/vec3", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "expand_vec3", "delta": vector_bits(1.0, -0.5, 0.0)}]}
    yield {"case_id": "aabb/expand-overload/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "expand_vec3", "delta": vector_bits(-0.0, 0.0, -0.0)}]}
    yield {"case_id": "aabb/expand-overload/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "expand_vec3", "delta": vector_bits(-0.25, 0.5, 1.0)},
                              {"op": "expand_vec3", "delta": vector_bits(0.125, -0.75, -0.5)}]}
    yield {"case_id": "aabb/equality/identical", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "equals", "box": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)},
                              {"op": "hash_code"}]}
    yield {"case_id": "aabb/equality/different-endpoint", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "equals", "box": box_bits(0.0, 0.0, 0.0, 2.0, 1.0, 1.0)},
                              {"op": "hash_code"}]}
    yield {"case_id": "aabb/equality/signed-zero", "family": "math", "operation": "aabb",
           **raw_box(float_bits(-0.0, "f64"), float_bits(0.0, "f64"), float_bits(0.0, "f64"),
                     float_bits(1.0, "f64"), float_bits(1.0, "f64"), float_bits(1.0, "f64")),
           "steps": [{"op": "equals", "box": raw_box(
               float_bits(0.0, "f64"), float_bits(0.0, "f64"), float_bits(0.0, "f64"),
               float_bits(1.0, "f64"), float_bits(1.0, "f64"), float_bits(1.0, "f64"))},
                     {"op": "hash_code"}]}
    yield {"case_id": "aabb/equality/nan-payload", "family": "math", "operation": "aabb",
           **raw_box("7ff8000000000000", float_bits(0.0, "f64"), float_bits(0.0, "f64"),
                     float_bits(float("inf"), "f64"), float_bits(1.0, "f64"), float_bits(1.0, "f64")),
           "steps": [{"op": "equals", "box": raw_box(
               "7ff8000000000001", float_bits(0.0, "f64"), float_bits(0.0, "f64"),
               float_bits(float("inf"), "f64"), float_bits(1.0, "f64"), float_bits(1.0, "f64"))},
                     {"op": "hash_code"}]}
    yield {"case_id": "aabb/equality/infinities", "family": "math", "operation": "aabb",
           **box_bits(float("-inf"), float("-inf"), float("-inf"),
                      float("inf"), float("inf"), float("inf")),
           "steps": [{"op": "equals", "box": box_bits(
               float("-inf"), float("-inf"), float("-inf"),
               float("inf"), float("inf"), float("inf"))},
                     {"op": "equals", "box": box_bits(-1.0, -1.0, -1.0, 1.0, 1.0, 1.0)},
                     {"op": "hash_code"}]}
    yield {"case_id": "aabb/to-string/basic", "family": "math", "operation": "aabb",
           **box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875),
           "steps": [{"op": "to_string"}]}
    yield {"case_id": "aabb/to-string/scientific", "family": "math", "operation": "aabb",
           **box_bits(0.0001, 10000000.0, -0.0, 0.0002, 10000001.0, 2.0),
           "steps": [{"op": "to_string"}]}
    yield {"case_id": "aabb/to-string/signed-zero", "family": "math", "operation": "aabb",
           **box_bits(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0),
           "steps": [{"op": "to_string"}]}
    yield {"case_id": "aabb/to-string/infinity", "family": "math", "operation": "aabb",
           **box_bits(float("-inf"), float("-inf"), float("-inf"),
                      float("inf"), float("inf"), float("inf")),
           "steps": [{"op": "to_string"}]}
    yield {"case_id": "aabb/to-string/nan", "family": "math", "operation": "aabb",
           **box_bits(float("nan"), 0.0, 0.0, 1.0, 1.0, 1.0),
           "steps": [{"op": "to_string"}]}
    yield {"case_id": "aabb/contract/positive", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(0.25, "f64"),
                              "y": float_bits(0.5, "f64"), "z": float_bits(1.0, "f64")}]}
    yield {"case_id": "aabb/contract/negative", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(-0.25, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(-1.0, "f64")}]}
    yield {"case_id": "aabb/contract/mixed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(-0.25, "f64"),
                              "y": float_bits(0.0, "f64"), "z": float_bits(0.5, "f64")}]}
    yield {"case_id": "aabb/contract/signed-zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(-0.0, "f64"),
                              "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64")}]}
    yield {"case_id": "aabb/contract/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(0.25, "f64"),
                              "y": float_bits(-0.25, "f64"), "z": float_bits(0.0, "f64")},
                             {"op": "contract", "x": float_bits(-0.125, "f64"),
                              "y": float_bits(0.125, "f64"), "z": float_bits(-0.5, "f64")}]}
    yield {"case_id": "aabb/inflate/scalar-positive", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate", "amount": float_bits(0.25, "f64")}]}
    yield {"case_id": "aabb/inflate/vector-mixed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate_xyz", "x": float_bits(0.25, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(0.75, "f64")}]}
    yield {"case_id": "aabb/inflate/scalar-negative", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate", "amount": float_bits(-0.75, "f64")}]}
    yield {"case_id": "aabb/inflate/signed-zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate_xyz", "x": float_bits(-0.0, "f64"),
                              "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64")}]}
    yield {"case_id": "aabb/inflate/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate_xyz", "x": float_bits(0.25, "f64"),
                              "y": float_bits(0.5, "f64"), "z": float_bits(0.0, "f64")},
                             {"op": "inflate", "amount": float_bits(-0.125, "f64")}]}
    yield {"case_id": "aabb/deflate/vector-mixed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(0.25, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(0.75, "f64")}]}
    yield {"case_id": "aabb/deflate/vector-reversed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(2.0, "f64"),
                              "y": float_bits(2.0, "f64"), "z": float_bits(2.0, "f64")}]}
    yield {"case_id": "aabb/deflate/signed-zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(-0.0, "f64"),
                              "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64")}]}
    yield {"case_id": "aabb/deflate/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(0.25, "f64"),
                              "y": float_bits(0.5, "f64"), "z": float_bits(0.0, "f64")},
                             {"op": "deflate", "amount": float_bits(-0.125, "f64")}]}
    yield {"case_id": "aabb/deflate/special-values", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(float("nan"), "f64"),
                              "y": float_bits(float("inf"), "f64"),
                              "z": float_bits(float("-inf"), "f64")}]}
    yield {"case_id": "aabb/intersect/contained", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(0.25, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(0.75, "f64"),
               "max_y": float_bits(0.75, "f64"), "max_z": float_bits(0.75, "f64")}}]}
    yield {"case_id": "aabb/intersect/partial", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(-0.5, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(-1.0, "f64"), "max_x": float_bits(0.5, "f64"),
               "max_y": float_bits(1.5, "f64"), "max_z": float_bits(0.5, "f64")}}]}
    yield {"case_id": "aabb/intersect/disjoint-normalizes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(2.0, "f64"), "min_y": float_bits(2.0, "f64"),
               "min_z": float_bits(2.0, "f64"), "max_x": float_bits(3.0, "f64"),
               "max_y": float_bits(3.0, "f64"), "max_z": float_bits(3.0, "f64")}}]}
    yield {"case_id": "aabb/intersect/touching-face", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(1.0, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(2.0, "f64"),
               "max_y": float_bits(0.75, "f64"), "max_z": float_bits(0.75, "f64")}}]}
    yield {"case_id": "aabb/intersect/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(0.0, "f64"), "min_y": float_bits(0.0, "f64"),
               "min_z": float_bits(0.0, "f64"), "max_x": float_bits(0.0, "f64"),
               "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64")}}]}
    yield {"case_id": "aabb/intersect/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(-0.5, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(0.75, "f64"),
               "max_y": float_bits(1.5, "f64"), "max_z": float_bits(0.75, "f64")}},
                             {"op": "intersect", "box": {
                                 "min_x": float_bits(0.5, "f64"), "min_y": float_bits(-1.0, "f64"),
                                 "min_z": float_bits(0.5, "f64"), "max_x": float_bits(2.0, "f64"),
                                 "max_y": float_bits(0.5, "f64"), "max_z": float_bits(2.0, "f64")}}]}
    yield {"case_id": "aabb/minmax/contained", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(0.25, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(0.75, "f64"),
               "max_y": float_bits(0.75, "f64"), "max_z": float_bits(0.75, "f64")}}]}
    yield {"case_id": "aabb/minmax/partial", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(-0.5, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(-1.0, "f64"), "max_x": float_bits(0.5, "f64"),
               "max_y": float_bits(1.5, "f64"), "max_z": float_bits(0.5, "f64")}}]}
    yield {"case_id": "aabb/minmax/disjoint", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(2.0, "f64"), "min_y": float_bits(2.0, "f64"),
               "min_z": float_bits(2.0, "f64"), "max_x": float_bits(3.0, "f64"),
               "max_y": float_bits(3.0, "f64"), "max_z": float_bits(3.0, "f64")}}]}
    yield {"case_id": "aabb/minmax/touching-face", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(1.0, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(2.0, "f64"),
               "max_y": float_bits(0.75, "f64"), "max_z": float_bits(0.75, "f64")}}]}
    yield {"case_id": "aabb/minmax/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(0.0, "f64"), "min_y": float_bits(0.0, "f64"),
               "min_z": float_bits(0.0, "f64"), "max_x": float_bits(0.0, "f64"),
               "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64")}}]}
    yield {"case_id": "aabb/minmax/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(-0.5, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(0.75, "f64"),
               "max_y": float_bits(1.5, "f64"), "max_z": float_bits(0.75, "f64")}},
                             {"op": "minmax", "box": {
                                 "min_x": float_bits(-1.0, "f64"), "min_y": float_bits(-1.0, "f64"),
                                 "min_z": float_bits(-1.0, "f64"), "max_x": float_bits(0.5, "f64"),
                                 "max_y": float_bits(0.5, "f64"), "max_z": float_bits(2.0, "f64")}}]}
    yield {"case_id": "aabb/move/positive", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(1.0, "f64"),
                              "y": float_bits(2.0, "f64"), "z": float_bits(3.0, "f64")}]}
    yield {"case_id": "aabb/move/negative", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(-1.0, "f64"),
                              "y": float_bits(-2.0, "f64"), "z": float_bits(-3.0, "f64")}]}
    yield {"case_id": "aabb/move/mixed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(-0.25, "f64"),
                              "y": float_bits(0.5, "f64"), "z": float_bits(1.25, "f64")}]}
    yield {"case_id": "aabb/move/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "move", "x": float_bits(-0.0, "f64"),
                      "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64")}]}
    yield {"case_id": "aabb/move/large-delta", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(9007199254740992.0, "f64"),
                              "y": float_bits(-9007199254740992.0, "f64"),
                              "z": float_bits(0.5, "f64")}]}
    yield {"case_id": "aabb/move/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(0.25, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(1.0, "f64")},
                             {"op": "move", "x": float_bits(-0.125, "f64"),
                              "y": float_bits(0.25, "f64"), "z": float_bits(-0.5, "f64")}]}
    yield {"case_id": "aabb/move-overload/basic", "family": "math", "operation": "aabb",
           **box_bits(-1.25, 2.5, -3.75, 4.5, 5.75, 6.25),
           "steps": [{"op": "move_block_pos", "position": block_position(3, -2, 5)},
                      {"op": "move_vec3", "position": vector_bits(0.125, -0.25, 1.5)},
                      {"op": "move_vector3f", "position": vector_bits(0.2, -0.3, 0.4, "f32")}]}
    yield {"case_id": "aabb/move-overload/block-pos-extremes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move_block_pos", "position": block_position(2147483647, -2147483648, -2147483647)},
                              {"op": "move_block_pos", "position": block_position(-2147483648, 2147483647, 2147483647)}]}
    yield {"case_id": "aabb/move-overload/vec3-special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move_vec3", "position": vector_bits(float("inf"), float("-inf"), float("nan"))},
                              {"op": "move_vec3", "position": vector_bits(float("nan"), 0.5, float("inf"))}]}
    yield {"case_id": "aabb/move-overload/vector3f-precision", "family": "math", "operation": "aabb",
           **box_bits(-100.0, -200.0, -300.0, 400.0, 500.0, 600.0),
           "steps": [{"op": "move_vector3f", "position": vector_bits(0.2, -0.3, 123456.78, "f32")},
                      {"op": "move_vector3f", "position": vector_bits(-1.0 / 3.0, 1.0 / 7.0, -0.0, "f32")}]}
    yield {"case_id": "aabb/move-overload/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "move_block_pos", "position": block_position(0, 0, 0)},
                      {"op": "move_vec3", "position": vector_bits(-0.0, 0.0, -0.0)},
                      {"op": "move_vector3f", "position": vector_bits(-0.0, 0.0, -0.0, "f32")}]}
    yield {"case_id": "aabb/move-overload/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move_vec3", "position": vector_bits(0.25, -0.5, 1.0)},
                              {"op": "move_block_pos", "position": block_position(-2, 3, -4)},
                              {"op": "move_vector3f", "position": vector_bits(0.75, 0.125, -0.25, "f32")}]}
    yield {"case_id": "aabb/sizes/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "sizes"}]}
    yield {"case_id": "aabb/sizes/rectangular", "family": "math", "operation": "aabb",
           "min_x": float_bits(-2.0, "f64"), "min_y": float_bits(0.5, "f64"),
           "min_z": float_bits(10.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(4.5, "f64"), "max_z": float_bits(12.5, "f64"),
           "steps": [{"op": "sizes"}]}
    yield {"case_id": "aabb/sizes/reversed-normalizes", "family": "math", "operation": "aabb",
           "min_x": float_bits(3.0, "f64"), "min_y": float_bits(2.0, "f64"),
           "min_z": float_bits(1.0, "f64"), "max_x": float_bits(-1.0, "f64"),
           "max_y": float_bits(-2.0, "f64"), "max_z": float_bits(-3.0, "f64"),
           "steps": [{"op": "sizes"}]}
    yield {"case_id": "aabb/sizes/zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(1.5, "f64"), "min_y": float_bits(-2.0, "f64"),
           "min_z": float_bits(4.0, "f64"), "max_x": float_bits(1.5, "f64"),
           "max_y": float_bits(-2.0, "f64"), "max_z": float_bits(4.0, "f64"),
           "steps": [{"op": "sizes"}]}
    yield {"case_id": "aabb/sizes/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(0.0, "f64"),
           "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64"),
           "steps": [{"op": "sizes"}]}
    yield {"case_id": "aabb/sizes/large-sum-rounding", "family": "math", "operation": "aabb",
           **unit, "max_z": float_bits(9007199254740992.0, "f64"),
           "steps": [{"op": "sizes"}]}
    yield {"case_id": "aabb/has-nan/finite", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "has_nan"}]}
    yield {"case_id": "aabb/has-nan/x-min", "family": "math", "operation": "aabb",
           **unit, "min_x": float_bits(float("nan"), "f64"),
           "steps": [{"op": "has_nan"}]}
    yield {"case_id": "aabb/has-nan/y-max", "family": "math", "operation": "aabb",
           **unit, "max_y": float_bits(float("nan"), "f64"),
           "steps": [{"op": "has_nan"}]}
    yield {"case_id": "aabb/has-nan/z-both", "family": "math", "operation": "aabb",
           **unit, "min_z": float_bits(float("nan"), "f64"),
           "max_z": float_bits(float("nan"), "f64"),
           "steps": [{"op": "has_nan"}]}
    yield {"case_id": "aabb/has-nan/positive-infinity", "family": "math", "operation": "aabb",
           **unit, "min_x": float_bits(float("inf"), "f64"),
           "max_x": float_bits(float("inf"), "f64"),
           "steps": [{"op": "has_nan"}]}
    yield {"case_id": "aabb/has-nan/negative-infinity", "family": "math", "operation": "aabb",
           **unit, "min_y": float_bits(float("-inf"), "f64"),
           "max_y": float_bits(float("-inf"), "f64"),
           "steps": [{"op": "has_nan"}]}
    yield {"case_id": "aabb/positions/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "positions"}]}
    yield {"case_id": "aabb/positions/reversed-normalizes", "family": "math", "operation": "aabb",
           "min_x": float_bits(3.0, "f64"), "min_y": float_bits(2.0, "f64"),
           "min_z": float_bits(1.0, "f64"), "max_x": float_bits(-1.0, "f64"),
           "max_y": float_bits(-2.0, "f64"), "max_z": float_bits(-3.0, "f64"),
           "steps": [{"op": "positions"}]}
    yield {"case_id": "aabb/positions/asymmetric", "family": "math", "operation": "aabb",
           "min_x": float_bits(-7.25, "f64"), "min_y": float_bits(0.125, "f64"),
           "min_z": float_bits(10.5, "f64"), "max_x": float_bits(2.75, "f64"),
           "max_y": float_bits(8.875, "f64"), "max_z": float_bits(11.25, "f64"),
           "steps": [{"op": "positions"}]}
    yield {"case_id": "aabb/positions/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(0.0, "f64"),
           "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64"),
           "steps": [{"op": "positions"}]}
    yield {"case_id": "aabb/positions/infinite", "family": "math", "operation": "aabb",
           "min_x": float_bits(float("-inf"), "f64"), "min_y": float_bits(float("-inf"), "f64"),
           "min_z": float_bits(float("-inf"), "f64"), "max_x": float_bits(float("inf"), "f64"),
           "max_y": float_bits(float("inf"), "f64"), "max_z": float_bits(float("inf"), "f64"),
           "steps": [{"op": "positions"}]}
    yield {"case_id": "aabb/positions/after-move", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "positions"},
                              {"op": "move", "x": float_bits(1.25, "f64"),
                               "y": float_bits(-0.5, "f64"), "z": float_bits(2.0, "f64")},
                              {"op": "positions"}]}
    axis_box = box_bits(-4.0, -3.0, -2.0, 4.0, 3.0, 2.0)
    yield {"case_id": "aabb/axis-limit/min-all", "family": "math", "operation": "aabb",
           **axis_box, "steps": [{"op": "min", "axis": "x"},
                                  {"op": "min", "axis": "y"},
                                  {"op": "min", "axis": "z"}]}
    yield {"case_id": "aabb/axis-limit/max-all", "family": "math", "operation": "aabb",
           **axis_box, "steps": [{"op": "max", "axis": "x"},
                                  {"op": "max", "axis": "y"},
                                  {"op": "max", "axis": "z"}]}
    yield {"case_id": "aabb/axis-limit/mixed", "family": "math", "operation": "aabb",
           **axis_box, "steps": [{"op": "min", "axis": "z"},
                                  {"op": "max", "axis": "x"},
                                  {"op": "min", "axis": "y"},
                                  {"op": "max", "axis": "z"}]}
    yield {"case_id": "aabb/axis-limit/reversed-normalizes", "family": "math", "operation": "aabb",
           "min_x": float_bits(4.0, "f64"), "min_y": float_bits(3.0, "f64"),
           "min_z": float_bits(2.0, "f64"), "max_x": float_bits(-4.0, "f64"),
           "max_y": float_bits(-3.0, "f64"), "max_z": float_bits(-2.0, "f64"),
           "steps": [{"op": "min", "axis": "x"}, {"op": "max", "axis": "y"}]}
    yield {"case_id": "aabb/axis-limit/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(0.0, "f64"),
           "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64"),
           "steps": [{"op": "min", "axis": "x"}, {"op": "max", "axis": "y"},
                      {"op": "min", "axis": "z"}, {"op": "max", "axis": "x"}]}
    yield {"case_id": "aabb/axis-limit/nan", "family": "math", "operation": "aabb",
           **unit, "min_x": float_bits(float("nan"), "f64"),
           "steps": [{"op": "min", "axis": "x"}, {"op": "max", "axis": "x"},
                      {"op": "min", "axis": "y"}]}
    def point(x, y, z):
        return {"x": float_bits(x, "f64"), "y": float_bits(y, "f64"), "z": float_bits(z, "f64")}

    yield {"case_id": "aabb/distance/point-inside-boundary", "family": "math", "operation": "aabb",
           **box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "steps": [{"op": "distance_point", "point": point(0.5, 0.5, 0.5)},
                      {"op": "distance_point", "point": point(0.0, 0.0, 0.0)},
                      {"op": "distance_point", "point": point(1.0, 1.0, 1.0)}]}
    yield {"case_id": "aabb/distance/point-outside", "family": "math", "operation": "aabb",
           **box_bits(-2.0, -1.0, 0.0, 3.0, 4.0, 5.0),
           "steps": [{"op": "distance_point", "point": point(5.0, -3.0, 7.0)},
                      {"op": "distance_point", "point": point(-4.0, 2.0, 1.0)}]}
    yield {"case_id": "aabb/distance/point-special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "distance_point", "point": point(float("inf"), 0.5, 0.5)},
                              {"op": "distance_point", "point": point(float("-inf"), 0.5, 0.5)},
                              {"op": "distance_point", "point": point(float("nan"), 0.5, 0.5)}]}
    yield {"case_id": "aabb/distance/box-overlap-touch", "family": "math", "operation": "aabb",
           **box_bits(0.0, 0.0, 0.0, 2.0, 2.0, 2.0),
           "steps": [{"op": "distance_box", "box": box_bits(1.0, 1.0, 1.0, 3.0, 3.0, 3.0)},
                      {"op": "distance_box", "box": box_bits(2.0, 2.0, 2.0, 4.0, 4.0, 4.0)}]}
    yield {"case_id": "aabb/distance/box-separated-diagonal", "family": "math", "operation": "aabb",
           **box_bits(-2.0, -1.0, 0.0, -1.0, 1.0, 2.0),
           "steps": [{"op": "distance_box", "box": box_bits(2.0, 3.0, 5.0, 4.0, 6.0, 7.0)},
                      {"op": "distance_box", "box": box_bits(-5.0, -4.0, -3.0, -3.0, -2.0, -1.0)}]}
    yield {"case_id": "aabb/distance/box-reversed-and-special", "family": "math", "operation": "aabb",
           "min_x": float_bits(4.0, "f64"), "min_y": float_bits(3.0, "f64"),
           "min_z": float_bits(2.0, "f64"), "max_x": float_bits(-4.0, "f64"),
           "max_y": float_bits(-3.0, "f64"), "max_z": float_bits(-2.0, "f64"),
           "steps": [{"op": "distance_box", "box": box_bits(10.0, 10.0, 10.0, 11.0, 11.0, 11.0)},
                      {"op": "distance_box", "box": box_bits(float("nan"), 0.0, 0.0, 1.0, 1.0, 1.0)}]}

    yield {"case_id": "aabb/of-size/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(0.5, 0.5, 0.5),
                               "size_x": float_bits(1.0, "f64"),
                               "size_y": float_bits(1.0, "f64"),
                               "size_z": float_bits(1.0, "f64")}]}
    yield {"case_id": "aabb/of-size/asymmetric", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(-3.25, 4.5, 0.125),
                               "size_x": float_bits(2.5, "f64"),
                               "size_y": float_bits(7.25, "f64"),
                               "size_z": float_bits(0.75, "f64")}]}
    yield {"case_id": "aabb/of-size/negative-sizes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(1.0, -2.0, 3.0),
                               "size_x": float_bits(-4.0, "f64"),
                               "size_y": float_bits(-6.0, "f64"),
                               "size_z": float_bits(-8.0, "f64")}]}
    yield {"case_id": "aabb/of-size/zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(1.5, -2.25, 3.75),
                               "size_x": float_bits(0.0, "f64"),
                               "size_y": float_bits(0.0, "f64"),
                               "size_z": float_bits(0.0, "f64")}]}
    yield {"case_id": "aabb/of-size/signed-zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(-0.0, 0.0, -0.0),
                               "size_x": float_bits(-0.0, "f64"),
                               "size_y": float_bits(0.0, "f64"),
                               "size_z": float_bits(-0.0, "f64")}]}
    yield {"case_id": "aabb/of-size/special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(float("inf"), float("-inf"), float("nan")),
                               "size_x": float_bits(2.0, "f64"),
                               "size_y": float_bits(float("inf"), "f64"),
                               "size_z": float_bits(float("nan"), "f64")}]}

    yield {"case_id": "aabb/set-endpoint/basic", "family": "math", "operation": "aabb",
           **box_bits(-2.0, -3.0, -4.0, 5.0, 6.0, 7.0),
           "steps": [{"op": "set_min_x", "value": float_bits(-1.5, "f64")},
                      {"op": "set_min_y", "value": float_bits(-2.5, "f64")},
                      {"op": "set_min_z", "value": float_bits(-3.5, "f64")},
                      {"op": "set_max_x", "value": float_bits(4.5, "f64")},
                      {"op": "set_max_y", "value": float_bits(5.5, "f64")},
                      {"op": "set_max_z", "value": float_bits(6.5, "f64")}]}
    yield {"case_id": "aabb/set-endpoint/min-chain", "family": "math", "operation": "aabb",
           **box_bits(0.0, 10.0, 20.0, 30.0, 40.0, 50.0),
           "steps": [{"op": "set_min_x", "value": float_bits(1.0, "f64")},
                      {"op": "set_min_y", "value": float_bits(11.0, "f64")},
                      {"op": "set_min_z", "value": float_bits(21.0, "f64")}]}
    yield {"case_id": "aabb/set-endpoint/max-chain", "family": "math", "operation": "aabb",
           **box_bits(-30.0, -40.0, -50.0, 0.0, 10.0, 20.0),
           "steps": [{"op": "set_max_x", "value": float_bits(-1.0, "f64")},
                      {"op": "set_max_y", "value": float_bits(9.0, "f64")},
                      {"op": "set_max_z", "value": float_bits(19.0, "f64")}]}
    yield {"case_id": "aabb/set-endpoint/reversed-normalizes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "set_min_x", "value": float_bits(4.0, "f64")},
                              {"op": "set_min_y", "value": float_bits(5.0, "f64")},
                              {"op": "set_min_z", "value": float_bits(6.0, "f64")},
                              {"op": "set_max_x", "value": float_bits(-4.0, "f64")},
                              {"op": "set_max_y", "value": float_bits(-5.0, "f64")},
                              {"op": "set_max_z", "value": float_bits(-6.0, "f64")}]}
    yield {"case_id": "aabb/set-endpoint/signed-zero", "family": "math", "operation": "aabb",
           **box_bits(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0),
           "steps": [{"op": "set_min_x", "value": float_bits(-0.0, "f64")},
                      {"op": "set_max_x", "value": float_bits(0.0, "f64")},
                      {"op": "set_min_y", "value": float_bits(0.0, "f64")},
                      {"op": "set_max_y", "value": float_bits(-0.0, "f64")},
                      {"op": "set_min_z", "value": float_bits(-0.0, "f64")},
                      {"op": "set_max_z", "value": float_bits(0.0, "f64")}]}
    yield {"case_id": "aabb/set-endpoint/special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "set_min_x", "value": float_bits(float("nan"), "f64")},
                              {"op": "set_max_y", "value": float_bits(float("inf"), "f64")},
                              {"op": "set_min_z", "value": float_bits(float("-inf"), "f64")},
                              {"op": "set_max_x", "value": float_bits(float("nan"), "f64")},
                              {"op": "set_min_y", "value": float_bits(float("-inf"), "f64")},
                              {"op": "set_max_z", "value": float_bits(float("inf"), "f64")}]}

    clip_rays = {
        "west-to-east": ((-1.0, 0.5, 0.5), (2.0, 0.5, 0.5)),
        "east-to-west": ((2.0, 0.5, 0.5), (-1.0, 0.5, 0.5)),
        "inside-to-east": ((0.5, 0.5, 0.5), (2.0, 0.5, 0.5)),
        "diagonal": ((-1.0, -1.0, -1.0), (2.0, 2.0, 2.0)),
        "parallel-miss": ((-1.0, 2.0, 0.5), (2.0, 2.0, 0.5)),
        "below-threshold": ((-1.0, 0.5, 0.5), (-1.0 + 1.0e-8, 0.5, 0.5)),
    }
    for label, (source, target) in clip_rays.items():
        yield {"case_id": f"aabb/clip/{label}", "family": "math", "operation": "aabb",
               **unit, "steps": [{"op": "clip", "from": point(*source), "to": point(*target)}]}
    clip_coordinate_cases = {
        "west-to-east": {
            "bounds": (0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "source": (-1.0, 0.5, 0.5),
            "target": (2.0, 0.5, 0.5),
        },
        "east-to-west": {
            "bounds": (0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "source": (2.0, 0.5, 0.5),
            "target": (-1.0, 0.5, 0.5),
        },
        "diagonal": {
            "bounds": (-2.0, -1.0, 0.0, 3.0, 2.0, 4.0),
            "source": (-3.0, -2.0, -1.0),
            "target": (4.0, 3.0, 5.0),
        },
        "parallel-miss": {
            "bounds": (0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "source": (-1.0, 2.0, 0.5),
            "target": (2.0, 2.0, 0.5),
        },
        "reversed-bounds": {
            "bounds": (1.0, 1.0, 1.0, 0.0, 0.0, 0.0),
            "source": (-1.0, 0.5, 0.5),
            "target": (2.0, 0.5, 0.5),
        },
        "threshold-special": {
            "bounds": (0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "source": (-1.0, 0.5, 0.5),
            "target": (-1.0 + 1.0e-8, 0.5, 0.5),
            "extra": [
                {"from": point(0.5, 0.5, 0.5), "to": point(0.5, 0.5, 0.5)},
                {"from": point(float("nan"), 0.5, 0.5), "to": point(2.0, 0.5, 0.5)},
                {"from": point(float("-inf"), 0.5, 0.5), "to": point(float("inf"), 0.5, 0.5)},
            ],
        },
    }
    for label, details in clip_coordinate_cases.items():
        min_x, min_y, min_z, max_x, max_y, max_z = details["bounds"]
        steps = [{"op": "clip_coords", "min_x": float_bits(min_x, "f64"),
                  "min_y": float_bits(min_y, "f64"), "min_z": float_bits(min_z, "f64"),
                  "max_x": float_bits(max_x, "f64"), "max_y": float_bits(max_y, "f64"),
                  "max_z": float_bits(max_z, "f64"), "from": point(*details["source"]),
                  "to": point(*details["target"])}]
        for extra in details.get("extra", []):
            steps.append({"op": "clip_coords", "min_x": float_bits(min_x, "f64"),
                          "min_y": float_bits(min_y, "f64"), "min_z": float_bits(min_z, "f64"),
                          "max_x": float_bits(max_x, "f64"), "max_y": float_bits(max_y, "f64"),
                          "max_z": float_bits(max_z, "f64"), **extra})
        yield {"case_id": f"aabb/clip-coords/{label}", "family": "math", "operation": "aabb",
               **unit, "steps": steps}
    def clip_many_step(boxes, source, target, position):
        return {"op": "clip_many", "boxes": boxes, "from": point(*source),
                "to": point(*target), "pos": block_position(*position)}

    clip_many_unit = [box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)]
    yield {"case_id": "aabb/clip-many/west-hit", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step(clip_many_unit, (-1.0, 0.5, 0.5),
                                              (2.0, 0.5, 0.5), (0, 0, 0))]}
    yield {"case_id": "aabb/clip-many/east-hit-offset", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step(clip_many_unit, (5.0, -1.5, 5.5),
                                              (2.0, -1.5, 5.5), (3, -2, 5))]}
    yield {"case_id": "aabb/clip-many/vertical-and-depth", "family": "math", "operation": "aabb",
           **unit, "steps": [
               clip_many_step(clip_many_unit, (0.5, -1.0, 0.5), (0.5, 2.0, 0.5), (0, 0, 0)),
               clip_many_step(clip_many_unit, (0.5, 2.0, 0.5), (0.5, -1.0, 0.5), (0, 0, 0)),
               clip_many_step(clip_many_unit, (0.5, 0.5, -1.0), (0.5, 0.5, 2.0), (0, 0, 0)),
               clip_many_step(clip_many_unit, (0.5, 0.5, 2.0), (0.5, 0.5, -1.0), (0, 0, 0)),
           ]}
    yield {"case_id": "aabb/clip-many/diagonal-priority", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step(clip_many_unit, (-1.0, -1.0, -1.0),
                                              (2.0, 2.0, 2.0), (0, 0, 0))]}
    yield {"case_id": "aabb/clip-many/nearest-far-first", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step([
               box_bits(3.0, 0.0, 0.0, 4.0, 1.0, 1.0),
               box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           ], (9.0, 0.5, 0.5), (16.0, 0.5, 0.5), (10, 0, 0))]}
    yield {"case_id": "aabb/clip-many/nearest-near-first", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step([
               box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
               box_bits(3.0, 0.0, 0.0, 4.0, 1.0, 1.0),
           ], (-1.0, 0.5, 0.5), (6.0, 0.5, 0.5), (0, 0, 0))]}
    yield {"case_id": "aabb/clip-many/empty", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step([], (-1.0, 0.5, 0.5),
                                              (2.0, 0.5, 0.5), (0, 0, 0))]}
    yield {"case_id": "aabb/clip-many/translated-miss", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step(clip_many_unit, (-1.0, 0.5, 0.5),
                                              (2.0, 0.5, 0.5), (0, 3, 0))]}
    yield {"case_id": "aabb/clip-many/interior-and-threshold", "family": "math", "operation": "aabb",
           **unit, "steps": [
               clip_many_step(clip_many_unit, (0.5, 0.5, 0.5), (2.0, 0.5, 0.5), (0, 0, 0)),
               clip_many_step(clip_many_unit, (-1.0, 0.5, 0.5), (-1.0 + 1.0e-8, 0.5, 0.5), (0, 0, 0)),
           ]}
    def shape_clip_query(source, target, position=(0, 0, 0)):
        return {"from": point(*source), "to": point(*target), "pos": block_position(*position)}

    yield {"case_id": "shape/clip/empty", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "queries": [shape_clip_query((-1.0, 0.5, 0.5), (2.0, 0.5, 0.5))]}
    yield {"case_id": "shape/clip/short-segment", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "queries": [shape_clip_query((0.5, 0.5, 0.5), (0.5001, 0.5, 0.5))]}
    yield {"case_id": "shape/clip/inside-directions", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "queries": [
               shape_clip_query((0.5, 0.5, 0.5), (2.0, 0.5, 0.5)),
               shape_clip_query((0.5, 0.5, 0.5), (-1.0, 0.5, 0.5)),
               shape_clip_query((0.5, 0.5, 0.5), (0.5, 2.0, 0.5)),
               shape_clip_query((0.5, 0.5, 0.5), (0.5, -1.0, 0.5)),
               shape_clip_query((0.5, 0.5, 0.5), (0.5, 0.5, 2.0)),
               shape_clip_query((0.5, 0.5, 0.5), (0.5, 0.5, -1.0)),
               shape_clip_query((0.5, 0.5, 0.5), (2.0, 2.0, 2.0)),
           ]}
    yield {"case_id": "shape/clip/outside-offset", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "queries": [shape_clip_query((5.0, -1.5, 5.5), (2.0, -1.5, 5.5), (3, -2, 5))]}
    yield {"case_id": "shape/clip/partial-box", "family": "math", "operation": "shape", "kind": "clip",
           "shape": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
           "queries": [shape_clip_query((-1.0, 0.5, 0.5), (2.0, 0.5, 0.5))]}
    yield {"case_id": "shape/clip/joined-fallback", "family": "math", "operation": "shape", "kind": "clip_join",
           "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "second": box_bits(2.0, 0.0, 0.0, 3.0, 1.0, 1.0),
           "queries": [shape_clip_query((-1.0, 0.5, 0.5), (4.0, 0.5, 0.5))]}
    yield {"case_id": "shape/closest/empty", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "point": point(0.5, 0.5, 0.5)}
    yield {"case_id": "shape/closest/inside", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "point": point(0.5, 0.5, 0.5)}
    yield {"case_id": "shape/closest/axis-clamp", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
           "point": point(-1.0, 0.5, 0.5)}
    yield {"case_id": "shape/closest/diagonal-clamp", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "point": point(2.0, -1.0, 3.0)}
    yield {"case_id": "shape/closest/join-tie", "family": "math", "operation": "shape", "kind": "closest_join",
           "first": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "second": box_bits(2.0, 0.0, 0.0, 3.0, 1.0, 1.0),
           "point": point(1.5, 0.5, 0.5)}
    yield {"case_id": "shape/closest/translated-box", "family": "math", "operation": "shape", "kind": "closest",
           "shape": box_bits(-2.0, -3.0, 4.0, -1.0, -1.0, 6.0),
           "point": point(-4.0, -5.0, 8.0)}
    yield {"case_id": "shape/face/empty", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/face/block", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "north"}
    yield {"case_id": "shape/face/axis-full", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.0, 0.25, 0.0, 1.0, 0.75, 1.0), "direction": "east"}
    yield {"case_id": "shape/face/west-slice", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.0, 0.25, 0.0, 0.75, 0.75, 1.0), "direction": "west"}
    yield {"case_id": "shape/face/east-slice", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.25, 0.25, 0.0, 1.0, 0.75, 1.0), "direction": "east"}
    yield {"case_id": "shape/face/missing-boundary", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.25, 0.25, 0.0, 0.75, 0.75, 1.0), "direction": "east"}
    yield {"case_id": "shape/face/up-slice", "family": "math", "operation": "shape", "kind": "face",
           "shape": box_bits(0.25, 0.25, 0.25, 0.75, 1.0, 0.75), "direction": "up"}
    yield {"case_id": "shape/face/joined", "family": "math", "operation": "shape", "kind": "face_join",
           "first": box_bits(0.25, 0.0, 0.0, 1.0, 0.5, 1.0),
           "second": box_bits(0.25, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/block-block-east", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/gap", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/partial-hole", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/partial-covered", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/negative-direction", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "west"}
    yield {"case_id": "shape/occlusion/empty-occluder", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0), "direction": "up"}
    yield {"case_id": "shape/occlusion/vertical-contact", "family": "math", "operation": "shape",
           "kind": "block_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 0.5),
           "occluder": box_bits(0.0, 0.0, 0.5, 1.0, 1.0, 1.0), "direction": "south"}
    yield {"case_id": "shape/occlusion/joined-shape", "family": "math", "operation": "shape",
           "kind": "block_occludes_join", "first": box_bits(0.0, 0.0, 0.0, 0.5, 1.0, 1.0),
           "second": box_bits(0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/face-empty", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 0.00000005, 1.0)}
    yield {"case_id": "shape/occlusion/face-block-first", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)}
    yield {"case_id": "shape/occlusion/face-block-second", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)}
    yield {"case_id": "shape/occlusion/face-complementary", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0)}
    yield {"case_id": "shape/occlusion/face-gap", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.4, 1.0),
           "occluder": box_bits(0.0, 0.6, 0.0, 1.0, 1.0, 1.0)}
    yield {"case_id": "shape/occlusion/face-depth-complementary", "family": "math", "operation": "shape",
           "kind": "face_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 0.5),
           "occluder": box_bits(0.0, 0.0, 0.5, 1.0, 1.0, 1.0)}
    yield {"case_id": "shape/occlusion/merged-complementary", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/merged-block-first", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/merged-first-gap", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
           "occluder": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/merged-second-gap", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.25, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "east"}
    yield {"case_id": "shape/occlusion/merged-west", "family": "math", "operation": "shape",
           "kind": "merged_occludes", "shape": box_bits(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
           "occluder": box_bits(0.0, 0.5, 0.0, 1.0, 1.0, 1.0), "direction": "west"}
    def bounding_box(min_x, min_y, min_z, max_x, max_y, max_z):
        return {"min_x": str(min_x), "min_y": str(min_y), "min_z": str(min_z),
                "max_x": str(max_x), "max_y": str(max_y), "max_z": str(max_z)}

    bounding_box_cases = {
        "normal": [bounding_box(-4, -2, 3, 5, 6, 9)],
        "inverted-all": [bounding_box(4, 5, 6, -1, -2, -3)],
        "inverted-axes": [bounding_box(4, -2, 8, -1, 6, 3)],
        "integer-maximum-overflow": [bounding_box(-2, -3, -4, 2147483647, 2147483647, 2147483647)],
        "integer-limits": [bounding_box(-2147483648, -2147483648, -2147483648,
                                            2147483647, 2147483647, 2147483647)],
        "chain": [bounding_box(10, 20, 30, 12, 24, 36),
                   bounding_box(-8, -6, -4, -2, 0, 2)],
    }
    for label, boxes in bounding_box_cases.items():
        steps = [{"op": "of_bounding_box", "box": box} for box in boxes]
        yield {"case_id": f"aabb/of-bounding-box/{label}", "family": "math", "operation": "aabb",
               **unit, "steps": steps}
    for width in ("f32", "f64"):
        specials = {float_bits(x, width) for x in (0.0, -0.0, float("inf"), -float("inf"), float("nan"))}
        specials.update({"00000001", "80000001"} if width == "f32"
                        else {"0000000000000001", "8000000000000001"})
        for operation, centers in [("floor", (-2, -1, 1, 2, -(2**31), 2**31)),
                                   ("wrap_degrees", (-720, -360, -180, 180, 360, 720, 100000))]:
            values = specials | {bits for value in centers for bits in neighbors(value, width)}
            for bits in sorted(values):
                yield {"case_id": f"math/{operation}/{width}/{bits}", "family": "math",
                       "operation": operation, "type": width, "value": bits}
        for low, high in [(-1.0, 1.0), (0.0, 1.0)]:
            values = specials | {bits for value in (1.0, -1.0) for bits in neighbors(value, width)}
            for bits in sorted(values):
                yield {"case_id": f"math/clamp/{width}/{low}/{bits}", "family": "math",
                       "operation": "clamp", "type": width, "value": bits,
                       "min": float_bits(low, width), "max": float_bits(high, width)}


def timer_cases():
    def trace(label, steps, start=0, tps=20, target=50):
        return {"case_id": f"timer/{label}", "family": "timer", "operation": "trace",
                "start_ms": str(start), "tps": float_bits(tps), "target_mspt": float_bits(target),
                "steps": steps}

    def time(action, ms):
        return {"action": action, "ms": str(ms)}

    for delta in (0, 1, 49, 50, 51, 1000):
        yield trace(f"boundary-{delta}", [time("game", delta), time("real", delta)])
    yield trace("residual", [time("game", ms) for ms in (17, 34, 49, 50, 51, 99, 100, 100, 101)])
    for tps, target in [(10, 100), (20, 25), (30, 1000 / 30), (60, 1000 / 60)]:
        yield trace(f"rate-{tps}-{target}", [step for ms in (1, 17, 33, 50, 100, 1000)
                                            for step in (time("game", ms), time("real", ms))],
                    tps=tps, target=target)
    yield trace("rate-change", [time("game", 25), {"action": "target", "value": float_bits(100)},
                               time("game", 100), {"action": "target", "value": float_bits(25)},
                               time("game", 125), time("real", 125)])
    yield trace("pause-residual", [time("game", 25), {"action": "pause", "value": True},
                                  time("game", 50), {"action": "pause", "value": True},
                                  time("real", 75), time("game", 75),
                                  {"action": "pause", "value": False}, time("game", 100),
                                  {"action": "pause", "value": False}])
    for pause in (False, True):
        for freeze in (False, True):
            yield trace(f"pause-{pause}-freeze-{freeze}", [time("game", 25),
                        {"action": "pause", "value": pause}, {"action": "freeze", "value": freeze},
                        time("game", 75), time("real", 100), {"action": "freeze", "value": False},
                        {"action": "pause", "value": False}, time("game", 100)])
    for delta in (349, 350, 351, 5000):
        yield trace(f"real-limit-{delta}", [time("real", delta), time("real", delta + 1)])
    start = 2**53 + 1
    yield trace("large-clock", [time("game", start + 49), time("real", start + 50),
                                time("game", start + 50), time("game", start + 51)], start=start)


def utf8_cases():
    # UTF-16 units keep unpaired surrogates intact without relying on JSON text decoding.
    values = {
        "empty": "", "ascii": "0041007a", "nul": "0000",
        "turkish": "01300131015f", "combining": "00650301",
        "bmp-widths": "007f008007ff0800ffff", "supplementary": "d83dde80",
        "high-alone": "d800", "low-alone": "dc00", "high-ascii": "d8000041",
        "high-high": "d800d801", "low-high": "dc00d800",
        "high-high-low": "d800d801dc00", "high-bmp": "d8000800",
        "pair-low": "d83dde80dc00",
    }
    for label, units in values.items():
        length = len(units) // 4
        for limit in sorted({max(0, length - 1), length, length + 1}):
            yield {"case_id": f"utf8/write/{label}/limit-{limit}", "family": "utf8",
                   "operation": "write", "max_length": limit,
                   "prefix_hex": "a55a", "utf16_hex": units}
    # Large values straddle the real default limit and VarInt prefix boundaries.
    for length in (127, 128, 32766, 32767, 32768):
        yield {"case_id": f"utf8/write/ascii-length-{length}", "family": "utf8",
               "operation": "write", "max_length": 32767,
               "prefix_hex": "a55a", "utf16_hex": "0061" * length}
    yield {"case_id": "utf8/write/three-byte-default-limit", "family": "utf8",
           "operation": "write", "max_length": 32767,
           "prefix_hex": "a55a", "utf16_hex": "0800" * 32767}

    # Independent byte inputs, not encoder output or calculated expectations.
    payloads = {
        "empty": "", "nul": "00", "ascii": "417a", "two-byte": "c480",
        "three-byte": "e0a080", "supplementary": "f09f9a80",
        "unicode-max": "f48fbfbf", "combining": "65cc81",
        "continuation": "80bf", "overlong-two": "c080", "overlong-three": "e08080",
        "overlong-four": "f0808080", "surrogate": "eda080",
        "out-of-range": "f4908080", "invalid-leaders": "f580ff",
        "truncated-two": "c2", "truncated-three": "e282",
        "truncated-four": "f09f9a", "invalid-middle": "e241a1",
        "invalid-fourth": "f09f9a41", "invalid-second": "c241",
        "replacement": "efbfbd",
    }
    for label, payload in payloads.items():
        # All of these payload sizes fit in a single literal length byte.
        framed = bytes([len(payload) // 2]).hex() + payload + "55aa"
        for limit in (1, 2, 8):
            yield {"case_id": f"utf8/read/{label}/limit-{limit}", "family": "utf8",
                   "operation": "read", "max_length": limit,
                   "prefix_hex": "a55a", "hex": framed}
    reads = {
        "zero-limit-empty": (0, "0055aa"),
        "zero-limit-nonempty": (0, "014155aa"),
        "negative-length": (8, "ffffffff0f55aa"),
        "encoded-limit-before-availability": (1, "04"),
        "encoded-limit-with-payload": (1, "044142434455aa"),
        "missing-payload": (8, "01"),
        "short-payload": (8, "034142"),
        "decoded-limit-after-payload": (1, "0341424355aa"),
        "noncanonical-zero": (8, "800055aa"),
        "noncanonical-one": (8, "81004155aa"),
        "prefix-overflow-to-zero": (8, "808080801055aa"),
        "ascii-127": (32767, "7f" + "61" * 127 + "55aa"),
        "ascii-128": (32767, "8001" + "61" * 128 + "55aa"),
        "default-limit": (32767, "ffff01" + "61" * 32767 + "55aa"),
        "default-limit-exceeded": (32767, "808002" + "61" * 32768 + "55aa"),
        "encoded-default-limit": (32767, "fdff05" + "e0a080" * 32767 + "55aa"),
        "encoded-default-limit-exceeded": (32767, "feff05"),
    }
    for count in range(7):
        reads[f"prefix-truncated-{count}"] = (8, "80" * count)
    reads["prefix-too-big-terminated"] = (8, "80808080800055aa")
    for label, (limit, framed) in reads.items():
        yield {"case_id": f"utf8/read/{label}", "family": "utf8", "operation": "read",
               "max_length": limit, "prefix_hex": "a55a", "hex": framed}


def binary_cases():
    uuids = {
        "zero": ("0000000000000000", "0000000000000000"),
        "max": ("ffffffffffffffff", "ffffffffffffffff"),
        "sample": ("0011223344556677", "8899aabbccddeeff"),
        "alternating": ("f0f0f0f0f0f0f0f0", "0f0f0f0f0f0f0f0f"),
        "version4": ("123e4567e89b12d3", "a456426655440000"),
        "high-bit": ("8000000000000000", "0000000000000001"),
        "low-bit": ("0000000000000001", "8000000000000000"),
        "deadbeef": ("deadbeefdeadbeef", "cafebabecafebabe"),
    }
    for label, (most, least) in uuids.items():
        yield {"case_id": f"binary/uuid-write/{label}", "family": "binary",
               "operation": "write", "codec": "uuid",
               "prefix_hex": "a55a", "most_hex": most, "least_hex": least}
    sample = bytes.fromhex("00112233445566778899aabbccddeeff").hex()
    for length in (0, 1, 7, 8, 9, 15, 16):
        framed = sample[:length * 2] + "55aa"
        yield {"case_id": f"binary/uuid-read/truncated-{length}", "family": "binary",
               "operation": "read", "codec": "uuid",
               "prefix_hex": "a55a", "hex": framed}
    yield {"case_id": "binary/uuid-read/exact", "family": "binary",
           "operation": "read", "codec": "uuid",
           "prefix_hex": "a55a", "hex": sample}
    yield {"case_id": "binary/uuid-read/all-ones", "family": "binary",
           "operation": "read", "codec": "uuid",
           "prefix_hex": "a55a", "hex": "ffffffffffffffffffffffffffffffff55aa"}

    positions = {
        "origin": (0, 0, 0),
        "unit": (1, 2, 3),
        "negative": (-1, -1, -1),
        "max-corner": (33554431, 2047, 33554431),
        "min-corner": (-33554432, -2048, -33554432),
        "wrap-x": (33554432, 0, 0),
        "wrap-y-high": (0, 2048, 0),
        "wrap-y-low": (0, -2049, 0),
        "border": (30000000, 100, -30000000),
        "i32-max": (2147483647, 2147483647, 2147483647),
        "i32-min": (-2147483648, -2147483648, -2147483648),
        "mixed": (123456, -789, 654321),
    }
    for label, (x, y, z) in positions.items():
        yield {"case_id": f"binary/blockpos-write/{label}", "family": "binary",
               "operation": "write", "codec": "blockpos",
               "prefix_hex": "a55a", "x": str(x), "y": str(y), "z": str(z)}
    payloads = {
        "zero": "0000000000000000",
        "ones": "ffffffffffffffff",
        "sample": "0040000030020000",
        "max-corner": "7fffffdffffff7ff",
        "high-bit": "8000000000000000",
        "deadbeef": "deadbeefcafebabe",
    }
    for label, payload in payloads.items():
        yield {"case_id": f"binary/blockpos-read/{label}", "family": "binary",
               "operation": "read", "codec": "blockpos",
               "prefix_hex": "a55a", "hex": payload + "55aa"}
    full = "0011223344556677"
    for length in range(8):
        yield {"case_id": f"binary/blockpos-read/truncated-{length}", "family": "binary",
               "operation": "read", "codec": "blockpos",
               "prefix_hex": "a55a", "hex": full[:length * 2] + "55aa"}

    sections = {
        "origin": (0, 0, 0),
        "unit": (1, 2, 3),
        "negative": (-1, -1, -1),
        "max-corner": (2097151, 524287, 2097151),
        "min-corner": (-2097152, -524288, -2097152),
        "wrap-x": (2097152, 0, 0),
        "wrap-y-high": (0, 524288, 0),
        "wrap-y-low": (0, -524289, 0),
        "wrap-z": (0, 0, 2097152),
        "i32-max": (2147483647, 2147483647, 2147483647),
        "i32-min": (-2147483648, -2147483648, -2147483648),
        "mixed": (123456, -789, 654321),
    }
    for label, (x, y, z) in sections.items():
        yield {"case_id": f"binary/sectionpos-write/{label}", "family": "binary",
               "operation": "write", "codec": "sectionpos",
               "prefix_hex": "a55a", "x": str(x), "y": str(y), "z": str(z)}
    section_payloads = {
        "zero": "0000000000000000",
        "ones": "ffffffffffffffff",
        "sample": "0008420000c00030",
        "high-bit": "8000000000000000",
    }
    for label, payload in section_payloads.items():
        yield {"case_id": f"binary/sectionpos-read/{label}", "family": "binary",
               "operation": "read", "codec": "sectionpos",
               "prefix_hex": "a55a", "hex": payload + "55aa"}
    for length in range(8):
        yield {"case_id": f"binary/sectionpos-read/truncated-{length}", "family": "binary",
               "operation": "read", "codec": "sectionpos",
               "prefix_hex": "a55a", "hex": full[:length * 2] + "55aa"}

    arrays = {
        "empty": [],
        "zero": ["0"],
        "sample": ["0", "1", "-1"],
        "extremes": [str(-(1 << 63)), str((1 << 63) - 1)],
        "powers": [str(1 << 7), str(-(1 << 7)), str(1 << 35), str(-(1 << 35))],
        "deadbeef": ["-2401053089206453570", "2309737966"],
    }
    for label, values in arrays.items():
        yield {"case_id": f"binary/longarray-write/{label}", "family": "binary",
               "operation": "write", "codec": "longarray",
               "prefix_hex": "a55a", "values": values}
    reads = {
        "empty": "0055aa",
        "one": "01010000000000000055aa",
        "two": "0200112233445566778899aabbccddeeff55aa",
        "short-payload": "02001122334455667755aa",
        "missing-payload": "01",
        "encoded-limit": "04",
        "negative-length": "ffffffff0f55aa",
        "noncanonical-zero": "800055aa",
        "prefix-overflow-to-zero": "808080801055aa",
        "ascii-127": "7f" + "00" * 127 + "55aa",
    }
    for label, framed in reads.items():
        yield {"case_id": f"binary/longarray-read/{label}", "family": "binary",
               "operation": "read", "codec": "longarray",
               "prefix_hex": "a55a", "hex": framed}
    for count in range(7):
        yield {"case_id": f"binary/longarray-read/prefix-truncated-{count}", "family": "binary",
               "operation": "read", "codec": "longarray",
               "prefix_hex": "a55a", "hex": "80" * count}
    yield {"case_id": "binary/longarray-read/prefix-too-big", "family": "binary",
           "operation": "read", "codec": "longarray",
           "prefix_hex": "a55a", "hex": "80808080800055aa"}

    bitsets = {
        "empty": [],
        "bit-zero": [0],
        "byte-boundary": [7, 8],
        "word-boundary": [63, 64, 65],
        "sample": [1, 65, 127],
        "trailing-zero": [0, 64],
    }
    for label, bits in bitsets.items():
        yield {"case_id": f"binary/bitset-write/{label}", "family": "binary",
               "operation": "write", "codec": "bitset",
               "prefix_hex": "a55a", "bits": bits}
    bit_reads = {
        "empty": "0055aa",
        "bit-zero": "01010000000000000055aa",
        "two-words": "020000000000000001010000000000000055aa",
        "short-payload": "02001122334455667755aa",
        "negative-length": "ffffffff0f55aa",
    }
    for label, framed in bit_reads.items():
        yield {"case_id": f"binary/bitset-read/{label}", "family": "binary",
               "operation": "read", "codec": "bitset",
               "prefix_hex": "a55a", "hex": framed}

    fixed_reads = {
        "size-zero-empty": (0, "55aa"),
        "size-zero-ignored": (0, "ff55aa"),
        "size-one-empty": (1, "0055aa"),
        "size-eight": (8, "ff55aa"),
        "size-nine": (9, "ffff55aa"),
        "size-sixteen": (16, "010255aa"),
        "short-payload": (9, "ff"),
        "missing-payload": (8, ""),
    }
    for label, (size, framed) in fixed_reads.items():
        yield {"case_id": f"binary/fixedbitset-read/{label}", "family": "binary",
               "operation": "read", "codec": "fixedbitset",
               "prefix_hex": "a55a", "size": size, "hex": framed}
    fixed_writes = {
        "size-zero-empty": (0, []),
        "size-zero-overflow": (0, [0]),
        "size-eight": (8, [0, 7]),
        "size-eight-overflow": (8, [8]),
        "size-nine": (9, [8]),
        "size-sixteen": (16, [0, 15]),
    }
    for label, (size, bits) in fixed_writes.items():
        yield {"case_id": f"binary/fixedbitset-write/{label}", "family": "binary",
               "operation": "write", "codec": "fixedbitset",
               "prefix_hex": "a55a", "size": size, "bits": bits}


def nbt_cases():
    prims = {
        "byte": ("byte", "127"),
        "short": ("short", "-12345"),
        "int": ("int", "2147483647"),
        "long": ("long", "-9223372036854775808"),
        "float": ("float", "3f800000"),
        "double": ("double", "bff0000000000000"),
        "string": ("string", "hi"),
    }
    for label, (kind, value) in prims.items():
        yield {"case_id": f"nbt/write/{label}", "family": "nbt",
               "operation": "write", "prefix_hex": "a55a",
               "fields": [{"key": "k", "type": kind, "value": value}]}
    yield {"case_id": "nbt/write/empty", "family": "nbt",
           "operation": "write", "prefix_hex": "a55a", "fields": []}
    yield {"case_id": "nbt/write/null", "family": "nbt",
           "operation": "write", "prefix_hex": "a55a",
           "null_root": True, "fields": []}
    trees = {
        "nested": {"type": "compound", "value": {
            "c": {"type": "compound", "value": {
                "b": {"type": "byte", "value": "2"}}}}},
        "list-int": {"type": "compound", "value": {
            "l": {"type": "list", "value": [
                {"type": "int", "value": "1"},
                {"type": "int", "value": "-2"},
                {"type": "int", "value": "300"}]}}},
        "list-string": {"type": "compound", "value": {
            "l": {"type": "list", "value": [
                {"type": "string", "value": "a"},
                {"type": "string", "value": "bc"}]}}},
        "empty-list": {"type": "compound", "value": {
            "e": {"type": "list", "value": []}}},
        "byte-array": {"type": "compound", "value": {
            "a": {"type": "byte-array", "value": ["0", "127", "-128"]}}},
        "int-array": {"type": "compound", "value": {
            "a": {"type": "int-array", "value": ["0", "-1", "2147483647"]}}},
        "long-array": {"type": "compound", "value": {
            "a": {"type": "long-array", "value": ["0", "-9223372036854775808"]}}},
        "list-compound": {"type": "compound", "value": {
            "l": {"type": "list", "value": [
                {"type": "compound", "value": {"x": {"type": "int", "value": "1"}}},
                {"type": "compound", "value": {"x": {"type": "int", "value": "2"}}}]}}},
    }
    for label, tree in trees.items():
        yield {"case_id": f"nbt/write-tree/{label}", "family": "nbt",
               "operation": "write", "prefix_hex": "a55a", "tree": tree}
    reads = {
        "empty": "0a0055aa",
        "null": "0055aa",
        "byte": "0a0100016b010055aa",
        "string": "0a08000161000268690055aa",
        "nested": "0a0a0001630100016202000055aa",
        "list-int": "0a0900016c03000000030000000100000002000000030055aa",
        "empty-list": "0a0900016500000000000055aa",
        "non-compound": "030102030455aa",
        "truncated-tag": "0a01",
        "truncated-body": "0a0100016b",
        "truncated-nested": "0a0a000163",
        "short-nested": "0a0a000163010001620200",
        "missing-payload": "0a",
        "empty-input": "",
    }
    for label, framed in reads.items():
        yield {"case_id": f"nbt/read/{label}", "family": "nbt",
               "operation": "read", "prefix_hex": "a55a", "hex": framed}


def build_cases(families):
    generators = {"wire": wire_cases, "math": math_cases, "timer": timer_cases, "utf8": utf8_cases,
                  "binary": binary_cases, "nbt": nbt_cases}
    cases = [dict(schema_version=1, **case)
             for family in sorted(families) for case in generators[family]()]
    ids = [case["case_id"] for case in cases]
    if not cases or len(ids) != len(set(ids)):
        raise ValueError("The corpus must be nonempty with unique case IDs")
    return cases
