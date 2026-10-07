"""The `utf8` family: original synthetic inputs."""

from cases.helpers import *


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
