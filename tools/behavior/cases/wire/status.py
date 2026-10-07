"""The `wire` family: the `status` cases."""

from cases.wire.common import *
from cases.wire.shared import *


def cases():
    status_request_writes = [
        ("empty", ""),
        ("with-prefix", "deadbeef"),
    ]

    for label, prefix_hex in status_request_writes:
        yield {"case_id": f"status/request/write/{label}", "family": "wire",
               "operation": "status_request", "direction": "write", "prefix_hex": prefix_hex}

    status_request_reads = [
        ("empty", ""),
        ("trailing", "aabbcc"),
        ("zero", "00"),
    ]

    for label, encoded in status_request_reads:
        yield {"case_id": f"status/request/read/{label}", "family": "wire",
               "operation": "status_request", "direction": "read", "hex": encoded}

    ping_writes = [
        ("zero", "0", "", "request"),
        ("negative-one", "-1", "", "request"),
        ("minimum", str(-(1 << 63)), "dead", "response"),
        ("maximum", str((1 << 63) - 1), "", "response"),
        ("pattern-request", "72623859790382856", "aa55", "request"),
        ("pattern-response", "-723685415333072913", "", "response"),
    ]

    for label, time, prefix_hex, packet in ping_writes:
        yield {"case_id": f"status/ping/write/{label}", "family": "wire",
               "operation": "ping", "direction": "write", "packet": packet,
               "time": time, "prefix_hex": prefix_hex}

    ping_reads = [
        ("zero", ping_hex(0)),
        ("negative-one", ping_hex(-1)),
        ("minimum", ping_hex(-(1 << 63))),
        ("maximum", ping_hex((1 << 63) - 1)),
        ("trailing", ping_hex(72623859790382856) + "aabb"),
        ("underflow-seven", "00" * 7),
        ("empty", ""),
    ]

    for packet in ("request", "response"):
        for label, encoded in ping_reads:
            yield {"case_id": f"status/ping/{packet}/read/{label}", "family": "wire",
                   "operation": "ping", "direction": "read", "packet": packet,
                   "hex": encoded}
