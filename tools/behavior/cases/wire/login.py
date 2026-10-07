"""The `wire` family: the `login` cases."""

from cases.wire.common import *
from cases.wire.shared import *


def cases():
    intention_writes = [
        ("basic", 776, "localhost", 25565, "status", "deadbeef"),
        ("unicode-host", 776, "例え.テスト", 25565, "login", ""),
        ("transfer", 776, "transfer.example", 0, "transfer", ""),
        ("empty-host", 0, "", 65535, "login", "aa"),
        ("negative-protocol", -1, "legacy", 25565, "status", ""),
        ("port-wrap", 776, "localhost", -1, "status", ""),
        ("host-over-decode-limit", 776, "a" * 256, 25565, "login", ""),
        ("host-encode-limit", 776, "a" * 32768, 25565, "login", ""),
    ]

    for label, protocol_version, host, port, intention, prefix_hex in intention_writes:
        yield {"case_id": f"handshake/intention/write/{label}", "family": "wire",
               "operation": "handshake_intention", "direction": "write",
               "protocol_version": protocol_version, "host_utf16_hex": utf16_hex(host),
               "port": port, "intention": intention, "prefix_hex": prefix_hex}

    intention_reads = [
        ("basic", intention_hex(776, b"localhost", 25565, 1)),
        ("unicode-host", intention_hex(776, "例え.テスト".encode("utf-8"), 25565, 2)),
        ("trailing", intention_hex(776, b"localhost", 25565, 1) + "aabb"),
        ("protocol-underflow", ""),
        ("host-prefix-underflow", varint_hex(776) + "80"),
        ("port-underflow", varint_hex(776) + "00" + "00"),
        ("unknown-intent", intention_hex(776, b"localhost", 25565, 4)),
        ("host-encoded-too-long", varint_hex(776) + varint_hex(766)),
        ("host-decoded-too-long", intention_hex(776, b"a" * 256, 25565, 2)),
        ("protocol-too-big", "808080808001"),
    ]

    for label, encoded in intention_reads:
        yield {"case_id": f"handshake/intention/read/{label}", "family": "wire",
               "operation": "handshake_intention", "direction": "read", "hex": encoded}

    hello_writes = [
        ("basic", "Player", "0123456789abcdef", "fedcba9876543210", ""),
        ("empty-name", "", "0000000000000000", "0000000000000000", "aa55"),
        ("unicode-name", "😀Alex", "8000000000000001", "7fffffffffffffff", ""),
        ("maximum-name", "a" * 16, "ffffffffffffffff", "0000000000000001", ""),
        ("too-long-name", "a" * 17, "0123456789abcdef", "fedcba9876543210", ""),
    ]

    for label, name, most_hex, least_hex, prefix_hex in hello_writes:
        yield {"case_id": f"login/hello/write/{label}", "family": "wire",
               "operation": "login_hello", "direction": "write",
               "name_utf16_hex": utf16_hex(name), "profile_most_hex": most_hex,
               "profile_least_hex": least_hex, "prefix_hex": prefix_hex}

    hello_reads = [
        ("basic", hello_hex(b"Player", "0123456789abcdef", "fedcba9876543210")),
        ("trailing", hello_hex(b"Player", "0123456789abcdef", "fedcba9876543210") + "aabb"),
        ("empty-name", hello_hex(b"", "0000000000000000", "0000000000000000")),
        ("name-underflow", "01"),
        ("name-encoded-too-long", "31"),
        ("name-decoded-too-long", hello_hex(b"a" * 17, "0123456789abcdef", "fedcba9876543210")),
        ("uuid-first-half-underflow", "00" + "00" * 7),
        ("uuid-second-half-underflow", "00" + "00" * 15),
        ("name-too-big", "808080808001"),
    ]

    for label, encoded in hello_reads:
        yield {"case_id": f"login/hello/read/{label}", "family": "wire",
               "operation": "login_hello", "direction": "read", "hex": encoded}

    compression_writes = [
        ("zero", 0, ""),
        ("one", 1, ""),
        ("single-byte-limit", 127, ""),
        ("two-byte-start", 128, ""),
        ("negative", -1, ""),
        ("maximum", (1 << 31) - 1, ""),
        ("with-prefix", 256, "deadbeef"),
    ]

    for label, threshold, prefix_hex in compression_writes:
        yield {"case_id": f"login/compression/write/{label}", "family": "wire",
               "operation": "login_compression", "direction": "write",
               "threshold": threshold, "prefix_hex": prefix_hex}

    compression_reads = [
        ("zero", varint_hex(0)),
        ("negative", varint_hex(-1)),
        ("trailing", varint_hex(128) + "aabb"),
        ("noncanonical-zero", "8000"),
        ("empty", ""),
        ("truncated", "80"),
        ("too-big", "808080808001"),
    ]

    for label, encoded in compression_reads:
        yield {"case_id": f"login/compression/read/{label}", "family": "wire",
               "operation": "login_compression", "direction": "read", "hex": encoded}

    server_hello_writes = [
        ("basic", "server", "000102", "aabb", True, "dead"),
        ("empty", "", "", "", False, ""),
        ("unicode-server-id", "服务器", "ff00", "01020304", True, ""),
        ("maximum-read-name", "a" * 20, "", "", False, ""),
        ("over-read-limit", "a" * 21, "00", "00", True, ""),
    ]

    for label, server_id, public_key_hex, challenge_hex, should_authenticate, prefix_hex in server_hello_writes:
        yield {"case_id": f"login/server-hello/write/{label}", "family": "wire",
               "operation": "login_server_hello", "direction": "write",
               "server_id_utf16_hex": utf16_hex(server_id), "public_key_hex": public_key_hex,
               "challenge_hex": challenge_hex, "should_authenticate": should_authenticate,
               "prefix_hex": prefix_hex}

    server_hello_reads = [
        ("basic", server_hello_hex(b"server", "000102", "aabb", True)),
        ("trailing", server_hello_hex(b"server", "000102", "aabb", True) + "aabb"),
        ("empty", server_hello_hex(b"", "", "", False)),
        ("server-id-underflow", "01"),
        ("server-id-encoded-too-long", varint_hex(61)),
        ("server-id-decoded-too-long", server_hello_hex(b"a" * 21, "00", "00", True)),
        ("public-key-underflow", "0003aabb"),
        ("challenge-underflow", "00010002aa"),
        ("boolean-underflow", "000000"),
        ("boolean-noncanonical", "00000002"),
        ("negative-array-size", "00" + varint_hex(-1)),
        ("server-id-too-big", "808080808001"),
    ]

    for label, encoded in server_hello_reads:
        yield {"case_id": f"login/server-hello/read/{label}", "family": "wire",
               "operation": "login_server_hello", "direction": "read", "hex": encoded}

    acknowledgement_writes = [
        ("empty", ""),
        ("with-prefix", "deadbeef"),
    ]

    for label, prefix_hex in acknowledgement_writes:
        yield {"case_id": f"login/acknowledged/write/{label}", "family": "wire",
               "operation": "login_acknowledged", "direction": "write", "prefix_hex": prefix_hex}

    acknowledgement_reads = [
        ("empty", ""),
        ("trailing", "aabbcc"),
        ("zero", "00"),
    ]

    for label, encoded in acknowledgement_reads:
        yield {"case_id": f"login/acknowledged/read/{label}", "family": "wire",
               "operation": "login_acknowledged", "direction": "read", "hex": encoded}

    profile_id = ("0123456789abcdef", "fedcba9876543210")

    session_id = ("0011223344556677", "8899aabbccddeeff")

    finished_writes = [
        ("basic", "Player", [], ""),
        ("empty", "", [], "aa55"),
        ("unicode-signed", "😀Alex", [profile_property("textures", "payload", "signature")], ""),
        ("maximum-name", "a" * 16,
         [profile_property("p", f"v{index}") for index in range(16)], ""),
        ("too-many-properties", "Player",
         [profile_property(f"p{index}", f"v{index}") for index in range(17)], ""),
        ("too-long-name", "a" * 17, [], ""),
        ("property-name-too-long", "Player", [profile_property("n" * 65, "v")], ""),
        ("property-value-too-long", "Player", [profile_property("n", "v" * 32768)], ""),
        ("signature-too-long", "Player", [profile_property("n", "v", "s" * 1025)], ""),
    ]

    for label, name, properties, prefix_hex in finished_writes:
        yield {"case_id": f"login/finished/write/{label}", "family": "wire",
               "operation": "login_finished", "direction": "write",
               "profile_most_hex": profile_id[0], "profile_least_hex": profile_id[1],
               "name_utf16_hex": utf16_hex(name), "properties": properties,
               "session_most_hex": session_id[0], "session_least_hex": session_id[1],
               "prefix_hex": prefix_hex}

    finished_reads = [
        ("basic", finished_hex(b"Player", [])),
        ("signed-property", finished_hex(b"Player", [property_hex(b"textures", b"payload", b"signature")])),
        ("trailing", finished_hex(b"Player", []) + "aabb"),
        ("profile-first-underflow", "00" * 7),
        ("profile-second-underflow", "00" * 15),
        ("name-underflow", profile_id[0] + profile_id[1] + "01"),
        ("name-too-long", profile_id[0] + profile_id[1] + varint_hex(49)),
        ("property-count-too-large", finished_hex(b"Player", [])[:-34] + varint_hex(17)),
        ("property-count-negative", finished_hex(b"", [])[:-34] + varint_hex(-1) + session_id[0] + session_id[1]),
        ("property-name-underflow", finished_hex(b"", [])[:-34] + varint_hex(1)),
        ("property-value-underflow", finished_hex(b"", [])[:-34] + varint_hex(1) + varint_hex(1) + b"n".hex() + "01"),
        ("signature-underflow", finished_hex(b"", [])[:-34] + varint_hex(1)
         + property_hex(b"n", b"v")[:-2] + "01" + "01"),
        ("session-first-underflow", finished_hex(b"", [])[:-32] + "00" * 7),
        ("session-second-underflow", finished_hex(b"", [])[:-32] + "00" * 15),
    ]

    for label, encoded in finished_reads:
        yield {"case_id": f"login/finished/read/{label}", "family": "wire",
               "operation": "login_finished", "direction": "read", "hex": encoded}
