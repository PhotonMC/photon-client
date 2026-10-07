"""The `binary` family: original synthetic inputs."""

from cases.helpers import *


def binary_cases():
    lp_vec3_values = {
        "zero": vector_bits(0.0, 0.0, 0.0),
        "below-min": vector_bits(0.00001, -0.00002, 0.00003),
        "unit-scale": vector_bits(1.0, -2.0, 3.0),
        "partial-scale": vector_bits(4.0, -8.0, 12.0),
        "sanitize": {"x": "7ff8000000000002", "y": "7ff0000000000000", "z": "fff0000000000000"},
        "large-clamp": vector_bits(3.4359738366e10, -3.4359738366e10, 0.0),
    }
    for label, values in lp_vec3_values.items():
        yield {"case_id": f"binary/lp-vec3-write/{label}", "family": "binary",
               "operation": "write", "codec": "lp_vec3", "prefix_hex": "a55a", **values}
    lp_vec3_reads = {
        "empty": "",
        "zero-early": "00aabb",
        "header-only": "01",
        "header-short": "0100",
        "unit-payload": "010000000000aabb",
        "continuation-no-scale": "040000000000",
        "continuation-payload": "04000000000001aabb",
        "continuation-varint-truncated": "04000000000080",
        "too-big-scale": "040000000000808080808080",
    }
    for label, framed in lp_vec3_reads.items():
        yield {"case_id": f"binary/lp-vec3-read/{label}", "family": "binary",
               "operation": "read", "codec": "lp_vec3", "prefix_hex": "a55a", "hex": framed}

    vec3_values = {
        "zero": vector_bits(0.0, 0.0, 0.0),
        "sample": vector_bits(1.25, -2.5, 3.75),
        "signed-zero": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"},
        "subnormal-and-large": {"x": "0000000000000001", "y": "7fefffffffffffff", "z": "8000000000000001"},
        "special": {"x": "7ff8000000000002", "y": "7ff0000000000000", "z": "fff0000000000000"},
    }
    for label, values in vec3_values.items():
        yield {"case_id": f"binary/vec3-write/{label}", "family": "binary",
               "operation": "write", "codec": "vec3", "prefix_hex": "a55a", **values}
    vec3_payload = "".join(vec3_values["sample"].values())
    for length in (0, 7, 8, 15, 16, 23, 24):
        suffix = "55aa" if length in (0, 8, 16, 24) else ""
        framed = bytes.fromhex(vec3_payload)[:length].hex() + suffix
        yield {"case_id": f"binary/vec3-read/truncated-{length}", "family": "binary",
               "operation": "read", "codec": "vec3", "prefix_hex": "a55a", "hex": framed}
    special_payload = "".join(vec3_values["special"].values())
    yield {"case_id": "binary/vec3-read/special", "family": "binary",
           "operation": "read", "codec": "vec3", "prefix_hex": "a55a",
           "hex": special_payload + "55aa"}

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

    chunk_positions = {
        "origin": (0, 0),
        "unit": (1, 2),
        "negative": (-1, -2),
        "alternating": (1437226410, -1437226411),
        "i32-max-min": (2147483647, -2147483648),
        "i32-min-max": (-2147483648, 2147483647),
    }
    for label, (x, z) in chunk_positions.items():
        yield {"case_id": f"binary/chunkpos-write/{label}", "family": "binary",
               "operation": "write", "codec": "chunkpos",
               "prefix_hex": "a55a", "x": str(x), "z": str(z)}
    chunk_payloads = {
        "zero": "0000000000000000",
        "unit": "0000000200000001",
        "negative": "fffffffeffffffff",
        "alternating": "aa55aa5555aa55aa",
        "i32-max-min": "800000007fffffff",
        "sample": "deadbeefcafebabe",
    }
    for label, payload in chunk_payloads.items():
        yield {"case_id": f"binary/chunkpos-read/{label}", "family": "binary",
               "operation": "read", "codec": "chunkpos",
               "prefix_hex": "a55a", "hex": payload + "55aa"}
    chunk_payload = chunk_payloads["sample"]
    for length in range(8):
        yield {"case_id": f"binary/chunkpos-read/truncated-{length}", "family": "binary",
               "operation": "read", "codec": "chunkpos",
               "prefix_hex": "a55a", "hex": chunk_payload[:length * 2] + "55aa"}

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
