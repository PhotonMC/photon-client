"""The `wire` family: the `framing` cases."""

from cases.wire.common import *
from cases.wire.shared import *


def cases():
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

    prepend_cases = [
        ("empty", 0, 0),
        ("one-with-prefix", 1, 3),
        ("single-byte-header-limit", 127, 2),
        ("two-byte-header-start", 128, 1),
        ("two-byte-header-limit", 16383, 0),
        ("three-byte-header-start", 16384, 4),
        ("maximum", (1 << 21) - 1, 0),
        ("too-large", 1 << 21, 0),
    ]

    for label, body_length, reader_skip in prepend_cases:
        yield {"case_id": f"frame/prepend/{label}", "family": "wire",
               "operation": "frame_prepend", "body_length": body_length,
               "reader_skip": reader_skip}

    decode_cases = [
        ("exact", ["03aabbcc"]),
        ("fragmented-header", ["80", "01" + "aa" * 128]),
        ("fragmented-body", ["02aa", "bb"]),
        ("zero-length", ["00"]),
        ("wide-length", ["808080"]),
        ("incomplete-header", ["80"]),
        ("incomplete-body", ["03aabb"]),
        ("trailing-frame", ["01aa01bb"]),
        ("successive-frames", ["01aa", "01bb"]),
        ("three-byte-length", ["808001" + "00" * 16384]),
    ]

    for label, chunks in decode_cases:
        yield {"case_id": f"frame/decode/{label}", "family": "wire",
               "operation": "frame_decode", "chunks": chunks}

    common_packets = [
        ("client-ping", 4, 32, 16909060),
        ("server-pong", 4, 32, 16909060),
        ("client-keepalive", 8, 64, 72623859790382856),
        ("server-keepalive", 8, 64, 72623859790382856),
    ]

    for packet, width, bits, pattern in common_packets:
        common_writes = [
            ("zero", "0", ""),
            ("negative-one", "-1", ""),
            ("minimum", str(-(1 << (bits - 1))), "deadbeef"),
            ("maximum", str((1 << (bits - 1)) - 1), ""),
            ("pattern", str(pattern), "aa55"),
        ]
        for label, value, prefix_hex in common_writes:
            yield {"case_id": f"common/{packet}/write/{label}", "family": "wire",
                   "operation": "common_packet", "direction": "write", "packet": packet,
                   "id": value, "prefix_hex": prefix_hex}

        common_reads = [
            ("zero", common_fixed_hex(0, width)),
            ("negative-one", common_fixed_hex(-1, width)),
            ("minimum", common_fixed_hex(-(1 << (bits - 1)), width)),
            ("maximum", common_fixed_hex((1 << (bits - 1)) - 1, width)),
            ("trailing", common_fixed_hex(pattern, width) + "aabb"),
            (f"underflow-{width - 1}", "00" * (width - 1)),
            ("empty", ""),
        ]
        for label, encoded in common_reads:
            yield {"case_id": f"common/{packet}/read/{label}", "family": "wire",
                   "operation": "common_packet", "direction": "read", "packet": packet,
                   "hex": encoded}
