"""Original synthetic inputs only. Expected values always come from the Java JAR."""

import struct


def float_bits(value, width="f32"):
    return struct.pack(">f" if width == "f32" else ">d", value).hex()


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
    yield {"case_id": "aabb/contains/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [inside, outside]}
    yield {"case_id": "aabb/intersects/overlap", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersects", "box": dict(unit)},
                             {"op": "intersects", "box": {
                                 "min_x": float_bits(2.0, "f64"), "min_y": float_bits(2.0, "f64"),
                                 "min_z": float_bits(2.0, "f64"), "max_x": float_bits(3.0, "f64"),
                                 "max_y": float_bits(3.0, "f64"), "max_z": float_bits(3.0, "f64")}}]}
    yield {"case_id": "aabb/expand-deflate/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "expand", "x": float_bits(1.0, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(0.0, "f64")},
                             {"op": "deflate", "amount": float_bits(0.5, "f64")}]}
    def point(x, y, z):
        return {"x": float_bits(x, "f64"), "y": float_bits(y, "f64"), "z": float_bits(z, "f64")}

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
