"""Original synthetic inputs only. Expected values always come from the Java JAR."""

import math
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


def vector2_bits(x, y, width="f32"):
    return {"x": float_bits(x, width), "y": float_bits(y, width)}


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

    def varint_hex(value):
        value &= 0xFFFFFFFF
        encoded = []
        while value & ~0x7F:
            encoded.append((value & 0x7F) | 0x80)
            value >>= 7
        encoded.append(value)
        return bytes(encoded).hex()

    def utf16_hex(value):
        return value.encode("utf-16-be", "surrogatepass").hex()

    def intention_hex(protocol_version, host_bytes, port, intention_id):
        return (varint_hex(protocol_version) + varint_hex(len(host_bytes)) + host_bytes.hex()
                + (port & 0xFFFF).to_bytes(2, "big").hex() + varint_hex(intention_id))

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

    def hello_hex(name_bytes, most_hex, least_hex):
        return (varint_hex(len(name_bytes)) + name_bytes.hex()
                + most_hex + least_hex)

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

    def server_hello_hex(server_bytes, public_key_hex, challenge_hex, should_authenticate):
        return (varint_hex(len(server_bytes)) + server_bytes.hex()
                + varint_hex(len(bytes.fromhex(public_key_hex))) + public_key_hex
                + varint_hex(len(bytes.fromhex(challenge_hex))) + challenge_hex
                + ("01" if should_authenticate else "00"))

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

    def profile_property(name, value, signature=None):
        return {"name_utf16_hex": utf16_hex(name), "value_utf16_hex": utf16_hex(value),
                "signature_utf16_hex": None if signature is None else utf16_hex(signature)}

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

    def property_hex(name_bytes, value_bytes, signature_bytes=None):
        result = varint_hex(len(name_bytes)) + name_bytes.hex()
        result += varint_hex(len(value_bytes)) + value_bytes.hex()
        if signature_bytes is None:
            return result + "00"
        return result + "01" + varint_hex(len(signature_bytes)) + signature_bytes.hex()

    def finished_hex(name_bytes, properties, profile=("0123456789abcdef", "fedcba9876543210"),
                     session=("0011223344556677", "8899aabbccddeeff")):
        return (profile[0] + profile[1] + varint_hex(len(name_bytes)) + name_bytes.hex()
                + varint_hex(len(properties)) + "".join(properties)
                + session[0] + session[1])

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

    configuration_units = ["server-finish", "client-finish", "accept-code-of-conduct"]
    for packet in configuration_units:
        for label, prefix_hex in [("empty", ""), ("with-prefix", "deadbeef")]:
            yield {"case_id": f"configuration/unit/{packet}/write/{label}", "family": "wire",
                   "operation": "configuration_unit", "direction": "write",
                   "packet": packet, "prefix_hex": prefix_hex}
        for label, encoded in [("empty", ""), ("trailing", "aabbcc"), ("zero", "00")]:
            yield {"case_id": f"configuration/unit/{packet}/read/{label}", "family": "wire",
                   "operation": "configuration_unit", "direction": "read",
                   "packet": packet, "hex": encoded}

    def code_of_conduct_frame(value):
        encoded = value.encode("utf-8")
        return varint_hex(len(encoded)) + encoded.hex()

    code_of_conduct_writes = [
        ("empty", "", ""),
        ("url", "https://example.invalid/rules", "deadbeef"),
        ("unicode", "Kurallar: 😀 / Türkçe", ""),
        ("encode-limit", "a" * 32767, ""),
        ("encode-too-long", "a" * 32768, ""),
    ]
    for label, value, prefix_hex in code_of_conduct_writes:
        yield {"case_id": f"configuration/code-of-conduct/write/{label}", "family": "wire",
               "operation": "configuration_misc", "direction": "write",
               "packet": "code-of-conduct", "code_of_conduct_utf16_hex": utf16_hex(value),
               "prefix_hex": prefix_hex}

    code_of_conduct_reads = [
        ("empty", code_of_conduct_frame("")),
        ("url", code_of_conduct_frame("https://example.invalid/rules")),
        ("unicode", code_of_conduct_frame("Kurallar: 😀 / Türkçe")),
        ("trailing", code_of_conduct_frame("ok") + "aabb"),
        ("body-underflow", "01"),
        ("malformed-utf8", "01ff"),
        ("encoded-too-big", "808080808000"),
        ("decoded-too-long", varint_hex(32768) + "61" * 32768),
    ]
    for label, encoded in code_of_conduct_reads:
        yield {"case_id": f"configuration/code-of-conduct/read/{label}", "family": "wire",
               "operation": "configuration_misc", "direction": "read",
               "packet": "code-of-conduct", "hex": encoded}

    for label, prefix_hex in [("empty", ""), ("with-prefix", "deadbeef")]:
        yield {"case_id": f"configuration/reset-chat/write/{label}", "family": "wire",
               "operation": "configuration_misc", "direction": "write",
               "packet": "reset-chat", "prefix_hex": prefix_hex}
    for label, encoded in [("empty", ""), ("trailing", "aabbcc"), ("zero", "00")]:
        yield {"case_id": f"configuration/reset-chat/read/{label}", "family": "wire",
               "operation": "configuration_misc", "direction": "read",
               "packet": "reset-chat", "hex": encoded}

    for name in ["HANDSHAKING", "PLAY", "STATUS", "LOGIN", "CONFIGURATION"]:
        yield {"case_id": f"protocol/connection/{name.lower()}", "family": "wire",
               "operation": "protocol_metadata", "kind": "connection", "name": name}
    for name in ["SERVERBOUND", "CLIENTBOUND"]:
        yield {"case_id": f"protocol/flow/{name.lower()}", "family": "wire",
               "operation": "protocol_metadata", "kind": "flow", "name": name}

    def client_information_hex(language_bytes, view_distance, chat_visibility, chat_colors,
                               model_customisation, main_hand, text_filtering, allows_listing,
                               particle_status):
        return (varint_hex(len(language_bytes)) + language_bytes.hex()
                + (view_distance & 0xff).to_bytes(1, "big").hex()
                + varint_hex(chat_visibility) + ("01" if chat_colors else "00")
                + (model_customisation & 0xff).to_bytes(1, "big").hex()
                + varint_hex(main_hand) + ("01" if text_filtering else "00")
                + ("01" if allows_listing else "00") + varint_hex(particle_status))

    client_information_writes = [
        ("basic", "en_us", 2, "full", True, 0, "right", False, False, "all", ""),
        ("empty", "", 0, "hidden", False, 255, "left", True, True, "minimal", "aa55"),
        ("unicode", "tr_TR😀", -1, "system", True, 3, "right", True, False, "decreased", ""),
        ("view-overflow", "en_us", 256, "full", False, 0, "left", False, False, "all", ""),
        ("model-overflow", "en_us", 2, "full", False, 511, "right", False, False, "all", ""),
        ("language-encode-limit", "a" * 32768, 2, "full", True, 0, "right", False, False, "all", ""),
    ]
    chat_ids = {"full": 0, "system": 1, "hidden": 2}
    hand_ids = {"left": 0, "right": 1}
    particle_ids = {"all": 0, "decreased": 1, "minimal": 2}
    for label, language, view_distance, chat_visibility, chat_colors, model_customisation, main_hand, text_filtering, allows_listing, particle_status, prefix_hex in client_information_writes:
        yield {"case_id": f"configuration/client-information/write/{label}", "family": "wire",
               "operation": "client_information", "direction": "write",
               "language_utf16_hex": utf16_hex(language), "view_distance": view_distance,
               "chat_visibility": chat_visibility, "chat_colors": chat_colors,
               "model_customisation": model_customisation, "main_hand": main_hand,
               "text_filtering_enabled": text_filtering, "allows_listing": allows_listing,
               "particle_status": particle_status, "prefix_hex": prefix_hex}

    information_values = {
        "basic": client_information_hex(b"en_us", 2, 0, True, 0, 1, False, False, 0),
        "trailing": client_information_hex(b"en_us", 2, 0, True, 0, 1, False, False, 0) + "aabb",
        "unicode": client_information_hex("tr_TR😀".encode("utf-8"), -1, 1, True, 3, 1, True, False, 1),
        "language-underflow": "01",
        "language-encoded-too-long": varint_hex(49),
        "language-decoded-too-long": varint_hex(17) + "61" * 17,
        "view-underflow": "00",
        "chat-prefix-underflow": "000080",
        "chat-invalid": "000003",
        "chat-colors-underflow": "000000",
        "model-underflow": "00000000",
        "arm-invalid": "000000000003",
        "text-filter-underflow": "000000000001",
        "allows-underflow": "00000000000100",
        "particle-invalid": "000000000001000003",
        "particle-underflow": "000000000001000080",
        "boolean-noncanonical": "000000020001020300",
    }
    for label, encoded in information_values.items():
        yield {"case_id": f"configuration/client-information/read/{label}", "family": "wire",
               "operation": "client_information", "direction": "read", "hex": encoded}

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

    def ping_hex(value):
        return int(value).to_bytes(8, "big", signed=True).hex()

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

    def common_fixed_hex(value, width):
        return int(value).to_bytes(width, "big", signed=True).hex()

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

    def feature_string_frame(value):
        encoded = value.encode("utf-8")
        return varint_hex(len(encoded)) + encoded.hex()

    def feature_frame(values):
        return varint_hex(len(values)) + "".join(feature_string_frame(value) for value in values)

    feature_writes = [
        ("empty", [], "deadbeef"),
        ("single", ["minecraft:vanilla"], ""),
        ("default-namespace", ["vanilla"], ""),
        ("multiple", ["minecraft:vanilla", "minecraft:trade_rebalance", "custom:feature/path"], ""),
        ("reordered", ["custom:feature/path", "minecraft:trade_rebalance", "minecraft:vanilla"], ""),
        ("duplicate", ["minecraft:vanilla", "minecraft:vanilla", "custom:feature/path"], "aa55"),
        ("feature-encode-too-long", ["minecraft:" + "a" * 32768], ""),
    ]
    for label, features, prefix_hex in feature_writes:
        yield {"case_id": f"configuration/enabled-features/write/{label}", "family": "wire",
               "operation": "enabled_features", "direction": "write", "features": features,
               "prefix_hex": prefix_hex}

    oversized_feature = "a" * 32768
    feature_reads = [
        ("empty", feature_frame([])),
        ("single", feature_frame(["minecraft:vanilla"])),
        ("default-namespace", feature_frame(["vanilla"])),
        ("multiple", feature_frame(["minecraft:vanilla", "minecraft:trade_rebalance", "custom:feature/path"])),
        ("reordered", feature_frame(["custom:feature/path", "minecraft:trade_rebalance", "minecraft:vanilla"])),
        ("duplicate", feature_frame(["minecraft:vanilla", "minecraft:vanilla", "custom:feature/path"])),
        ("trailing", feature_frame(["minecraft:vanilla"]) + "aabb"),
        ("feature-prefix-underflow", varint_hex(1)),
        ("feature-body-underflow", varint_hex(1) + varint_hex(4) + "6162"),
        ("invalid-namespace", feature_frame(["Bad:feature"])),
        ("invalid-path", feature_frame(["minecraft:bad?feature"])),
        ("malformed-utf8", varint_hex(1) + varint_hex(1) + "ff"),
        ("encoded-too-long", varint_hex(1) + varint_hex(32767 * 3 + 1)),
        ("decoded-too-long", feature_frame([oversized_feature])),
        ("negative-count", varint_hex(-1)),
        ("count-too-big", "808080808001"),
    ]
    for label, encoded in feature_reads:
        yield {"case_id": f"configuration/enabled-features/read/{label}", "family": "wire",
               "operation": "enabled_features", "direction": "read", "hex": encoded}

    def tag_values_frame(values):
        return varint_hex(len(values)) + "".join(varint_hex(value) for value in values)

    def tags_frame(registry, entries):
        return (varint_hex(1) + feature_string_frame(registry) + varint_hex(len(entries))
                + "".join(feature_string_frame(tag) + tag_values_frame(values)
                           for tag, values in entries))

    tag_writes = [
        ("empty", [] , "deadbeef"),
        ("registry-empty", [{"registry": "minecraft:block", "tags": []}], ""),
        ("multiple", [{"registry": "minecraft:block", "tags": [
            {"id": "minecraft:stone", "values": [1, 2]},
            {"id": "custom:ore/rare", "values": [-1, 300]},
            {"id": "minecraft:logs", "values": []},
        ]}], ""),
        ("reordered", [{"registry": "minecraft:block", "tags": [
            {"id": "custom:ore/rare", "values": [-1, 300]},
            {"id": "minecraft:logs", "values": []},
            {"id": "minecraft:stone", "values": [1, 2]},
        ]}], ""),
        ("duplicate-tag", [{"registry": "minecraft:block", "tags": [
            {"id": "minecraft:stone", "values": [1]},
            {"id": "minecraft:stone", "values": [2, 3]},
        ]}], "aa55"),
    ]
    for label, registries, prefix_hex in tag_writes:
        yield {"case_id": f"common/update-tags/write/{label}", "family": "wire",
               "operation": "update_tags", "direction": "write", "registries": registries,
               "prefix_hex": prefix_hex}

    tag_read_values = [
        ("empty", varint_hex(0)),
        ("registry-empty", tags_frame("minecraft:block", [])),
        ("multiple", tags_frame("minecraft:block", [
            ("minecraft:stone", [1, 2]),
            ("custom:ore/rare", [-1, 300]),
            ("minecraft:logs", []),
        ])),
        ("reordered", tags_frame("minecraft:block", [
            ("custom:ore/rare", [-1, 300]),
            ("minecraft:logs", []),
            ("minecraft:stone", [1, 2]),
        ])),
        ("duplicate-tag", tags_frame("minecraft:block", [
            ("minecraft:stone", [1]),
            ("minecraft:stone", [2, 3]),
        ])),
        ("trailing", tags_frame("minecraft:block", []) + "aabb"),
        ("registry-prefix-underflow", varint_hex(1)),
        ("registry-body-underflow", varint_hex(1) + varint_hex(14) + "6d696e656372616674"),
        ("tag-count-underflow", varint_hex(1) + feature_string_frame("minecraft:block")),
        ("tag-prefix-underflow", tags_frame("minecraft:block", [])[:-2] + "01"),
        ("values-prefix-underflow", varint_hex(1) + feature_string_frame("minecraft:block")
         + varint_hex(1) + feature_string_frame("minecraft:stone")),
        ("negative-values-count", varint_hex(1) + feature_string_frame("minecraft:block")
         + varint_hex(1) + feature_string_frame("minecraft:stone") + varint_hex(-1)),
        ("invalid-tag-identifier", tags_frame("minecraft:block", [("Bad:tag", [])])),
        ("negative-count", varint_hex(-1)),
        ("count-too-big", "808080808001"),
    ]
    for label, encoded in tag_read_values:
        yield {"case_id": f"common/update-tags/read/{label}", "family": "wire",
               "operation": "update_tags", "direction": "read", "hex": encoded}

    def nbt_tree_hex(node):
        type_name = node["type"]
        type_ids = {
            "byte": 1, "short": 2, "int": 3, "long": 4, "float": 5,
            "double": 6, "byte-array": 7, "string": 8, "list": 9,
            "compound": 10, "int-array": 11, "long-array": 12,
        }
        type_id = type_ids[type_name]
        if type_name == "byte":
            return f"{type_id:02x}" + int(node["value"]).to_bytes(1, "big", signed=True).hex()
        if type_name == "short":
            return f"{type_id:02x}" + int(node["value"]).to_bytes(2, "big", signed=True).hex()
        if type_name == "int":
            return f"{type_id:02x}" + int(node["value"]).to_bytes(4, "big", signed=True).hex()
        if type_name == "long":
            return f"{type_id:02x}" + int(node["value"]).to_bytes(8, "big", signed=True).hex()
        if type_name == "float":
            return f"{type_id:02x}" + node["value"]
        if type_name == "double":
            return f"{type_id:02x}" + node["value"]
        if type_name == "byte-array":
            values = node["value"]
            return (f"{type_id:02x}" + len(values).to_bytes(4, "big", signed=True).hex()
                    + bytes(int(value) & 0xff for value in values).hex())
        if type_name == "string":
            encoded = node["value"].encode("utf-8")
            return f"{type_id:02x}" + len(encoded).to_bytes(2, "big").hex() + encoded.hex()
        if type_name == "list":
            values = node["value"]
            element_id = 0 if not values else type_ids[values[0]["type"]]
            assert all(type_ids[value["type"]] == element_id for value in values)
            payload = "".join(nbt_tree_hex(value)[2:] for value in values)
            return (f"{type_id:02x}{element_id:02x}"
                    + len(values).to_bytes(4, "big", signed=True).hex() + payload)
        if type_name == "compound":
            payload = ""
            for key, value in node["value"].items():
                encoded_key = key.encode("utf-8")
                value_hex = nbt_tree_hex(value)
                payload += value_hex[:2] + len(encoded_key).to_bytes(2, "big").hex() + encoded_key.hex()
                payload += value_hex[2:]
            return f"{type_id:02x}" + payload + "00"
        if type_name == "int-array":
            values = node["value"]
            return (f"{type_id:02x}" + len(values).to_bytes(4, "big", signed=True).hex()
                    + "".join(int(value).to_bytes(4, "big", signed=True).hex() for value in values))
        if type_name == "long-array":
            values = node["value"]
            return (f"{type_id:02x}" + len(values).to_bytes(4, "big", signed=True).hex()
                    + "".join(int(value).to_bytes(8, "big", signed=True).hex() for value in values))
        raise AssertionError(type_name)

    def registry_data_frame(registry, entries):
        encoded = feature_string_frame(registry) + varint_hex(len(entries))
        for entry in entries:
            encoded += feature_string_frame(entry["id"])
            data = entry.get("data")
            if data is None:
                encoded += "00"
            else:
                encoded += "01" + nbt_tree_hex(data)
        return encoded

    registry = "minecraft:dimension_type"
    compound_tag = {"type": "compound", "value": {
        "name": {"type": "string", "value": "overworld"},
    }}
    list_tag = {"type": "list", "value": [
        {"type": "int", "value": "1"},
        {"type": "int", "value": "-2"},
    ]}
    registry_writes = [
        ("empty", [], "deadbeef"),
        ("absent-data", [{"id": "minecraft:overworld", "data": None}], ""),
        ("scalar-data", [{"id": "minecraft:overworld",
                           "data": {"type": "int", "value": "42"}}], ""),
        ("compound-data", [{"id": "minecraft:overworld", "data": compound_tag}], ""),
        ("list-data", [{"id": "minecraft:overworld", "data": list_tag}], ""),
        ("multiple", [
            {"id": "minecraft:overworld", "data": None},
            {"id": "minecraft:the_nether", "data": compound_tag},
        ], "aa55"),
        ("mixed-tags", [
            {"id": "custom:one", "data": {"type": "byte", "value": "-128"}},
            {"id": "custom:two", "data": {"type": "string", "value": "hi"}},
            {"id": "custom:three", "data": None},
        ], ""),
    ]
    for label, entries, prefix_hex in registry_writes:
        yield {"case_id": f"configuration/registry-data/write/{label}", "family": "wire",
               "operation": "registry_data", "direction": "write", "registry": registry,
               "entries": entries, "prefix_hex": prefix_hex}

    registry_read_values = [
        ("empty", registry_data_frame(registry, [])),
        ("absent-data", registry_data_frame(registry, [{"id": "minecraft:overworld", "data": None}])),
        ("scalar-data", registry_data_frame(registry, [{
            "id": "minecraft:overworld", "data": {"type": "int", "value": "42"},
        }])),
        ("compound-data", registry_data_frame(registry, [{
            "id": "minecraft:overworld", "data": compound_tag,
        }])),
        ("list-data", registry_data_frame(registry, [{
            "id": "minecraft:overworld", "data": list_tag,
        }])),
        ("multiple", registry_data_frame(registry, [
            {"id": "minecraft:overworld", "data": None},
            {"id": "minecraft:the_nether", "data": compound_tag},
        ])),
        ("trailing", registry_data_frame(registry, []) + "aabb"),
        ("noncanonical-present", feature_string_frame(registry) + varint_hex(1)
         + feature_string_frame("minecraft:overworld")
         + "02" + nbt_tree_hex({"type": "int", "value": "42"})),
        ("registry-prefix-underflow", "80"),
        ("registry-body-underflow", "01"),
        ("entry-count-underflow", feature_string_frame(registry)),
        ("entry-id-prefix-underflow", feature_string_frame(registry) + varint_hex(1) + "80"),
        ("optional-underflow", feature_string_frame(registry) + varint_hex(1)
         + feature_string_frame("minecraft:overworld")),
        ("tag-empty", feature_string_frame(registry) + varint_hex(1)
         + feature_string_frame("minecraft:overworld") + "01"),
        ("tag-end", feature_string_frame(registry) + varint_hex(1)
         + feature_string_frame("minecraft:overworld") + "0100"),
        ("tag-body-underflow", feature_string_frame(registry) + varint_hex(1)
         + feature_string_frame("minecraft:overworld") + "0103"),
        ("invalid-registry-identifier", feature_string_frame("Bad:registry") + varint_hex(0)),
        ("invalid-entry-identifier", feature_string_frame(registry) + varint_hex(1)
         + feature_string_frame("Bad:entry") + "00"),
        ("negative-entry-count", feature_string_frame(registry) + varint_hex(-1)),
        ("entry-count-too-big", feature_string_frame(registry) + "808080808001"),
    ]
    for label, encoded in registry_read_values:
        yield {"case_id": f"configuration/registry-data/read/{label}", "family": "wire",
               "operation": "registry_data", "direction": "read", "hex": encoded}


def math_cases():
    def varint(value):
        value &= 0xFFFFFFFF
        encoded = bytearray()
        while value & ~0x7F:
            encoded.append((value & 0x7F) | 0x80)
            value >>= 7
        encoded.append(value)
        return bytes(encoded)

    def sparse_values(entry_count, assignments):
        values = [0] * entry_count
        for index, value in assignments.items():
            if not 0 <= index < entry_count:
                raise ValueError("palette assignment is outside its container")
            values[index] = value
        return values

    def palette_container(bits, palette, values, entry_count, block_states):
        if bits == 0:
            if len(palette) != 1:
                raise ValueError("single-value palette must have one entry")
            return bytes([bits]) + varint(palette[0])

        is_local = (block_states and bits <= 8) or (not block_states and bits <= 3)
        storage_bits = 4 if block_states and bits <= 4 else bits
        encoded = bytearray([bits])
        if is_local:
            encoded.extend(varint(len(palette)))
            for value in palette:
                encoded.extend(varint(value))
        elif palette:
            raise ValueError("global palette must not carry palette entries")

        values_per_long = 64 // storage_bits
        word_count = (entry_count + values_per_long - 1) // values_per_long
        mask = (1 << storage_bits) - 1
        words = [0] * word_count
        for index, value in enumerate(values):
            word_index = index // values_per_long
            offset = (index % values_per_long) * storage_bits
            words[word_index] |= (value & mask) << offset
        for word in words:
            encoded.extend(word.to_bytes(8, "big"))
        return bytes(encoded)

    def section_hex(
        non_empty_block_count,
        fluid_count,
        block_bits,
        block_palette,
        block_values,
        biome_bits,
        biome_palette,
        biome_values,
    ):
        encoded = bytearray()
        encoded.extend(non_empty_block_count.to_bytes(2, "big", signed=True))
        encoded.extend(fluid_count.to_bytes(2, "big", signed=True))
        encoded.extend(palette_container(
            block_bits, block_palette, block_values, 4096, True))
        encoded.extend(palette_container(
            biome_bits, biome_palette, biome_values, 64, False))
        return encoded.hex()

    section_air = section_hex(
        0, 0, 0, [0], [0] * 4096, 0, [1], [0] * 64)
    section_linear = section_hex(
        4,
        0,
        4,
        [0, 1, 2, 3],
        sparse_values(4096, {0: 1, 1: 2, 17: 3, 255: 1}),
        2,
        [0, 1, 2, 3],
        sparse_values(64, {0: 1, 3: 2, 15: 3, 63: 1}),
    )
    section_hash_map = section_hex(
        4,
        0,
        5,
        [0, 1, 2, 3, 4],
        sparse_values(4096, {10: 1, 511: 2, 1023: 3, 4095: 4}),
        3,
        [0, 1, 2, 3, 4],
        sparse_values(64, {0: 1, 15: 2, 16: 3, 63: 4}),
    )
    section_global = section_hex(
        6,
        0,
        15,
        [],
        sparse_values(4096, {6: 1, 7: 2, 8: 3, 63: 1, 64: 2, 4095: 4}),
        7,
        [],
        sparse_values(64, {3: 1, 4: 2, 15: 3, 16: 4, 63: 4}),
    )
    yield {
        "case_id": "client-world/section-read-and-coordinate-lookup",
        "family": "math",
        "operation": "client_world_section_lookup",
        "dimension_min_y": -64,
        "dimension_height": 384,
        "tracks": [{
            "label": "negative-chunk-and-overworld-floor",
            "chunk_x": -2,
            "chunk_z": 3,
            "sections": [section_air, section_linear, section_hash_map, section_global],
            "queries": [
                {"label": "air-at-world-floor", "x": -32, "y": -64, "z": 48},
                {"label": "linear-first-entry", "x": -32, "y": -48, "z": 48},
                {"label": "linear-local-boundary", "x": -17, "y": -47, "z": 63},
                {"label": "hash-map-entry", "x": -22, "y": -32, "z": 48},
                {"label": "hash-map-last-row", "x": -17, "y": -29, "z": 63},
                {"label": "global-cross-word-start", "x": -17, "y": -16, "z": 51},
                {"label": "global-cross-word-next", "x": -32, "y": -16, "z": 52},
                {"label": "global-last-entry", "x": -17, "y": -1, "z": 63},
                {"label": "outside-chunk", "x": -16, "y": -48, "z": 48},
                {"label": "outside-loaded-sections", "x": -32, "y": 0, "z": 48},
            ],
        }],
    }

    cache_air = section_hex(
        0, 0, 0, [0], [0] * 4096, 0, [1], [0] * 64)
    cache_non_empty = [0] * 4096
    cache_non_empty[0] = 1
    cache_non_empty = section_hex(
        1, 0, 1, [0, 1], cache_non_empty, 0, [1], [0] * 64)
    cache_fluid = [0] * 4096
    cache_fluid[4095] = 1
    cache_fluid = section_hex(
        1, 1, 1, [0, 1], cache_fluid, 0, [1], [0] * 64)
    yield {
        "case_id": "client-world/chunk-cache-lifecycle",
        "family": "math",
        "operation": "client_world_chunk_cache",
        "dimension_min_y": -64,
        "server_view_radius": 0,
        "operations": [
            {
                "action": "replace",
                "chunk_x": 0,
                "chunk_z": 0,
                "sections": [cache_air, cache_non_empty],
                "checks": [
                    {"x": -5, "z": -5},
                    {"x": -6, "z": 0},
                    {"x": 5, "z": 5},
                ],
            },
            {
                "action": "replace",
                "chunk_x": 0,
                "chunk_z": 0,
                "sections": [cache_non_empty, cache_air],
                "checks": [{"x": 0, "z": 0}],
            },
            {"action": "flip", "checks": []},
            {"action": "center", "chunk_x": 8, "chunk_z": 0, "checks": [
                {"x": 8, "z": 0},
                {"x": 2, "z": 0},
                {"x": 14, "z": 0},
            ]},
            {
                "action": "replace",
                "chunk_x": 11,
                "chunk_z": 0,
                "sections": [cache_air, cache_fluid],
                "checks": [{"x": 11, "z": 0}, {"x": 0, "z": 0}],
            },
            {"action": "drop", "chunk_x": 0, "chunk_z": 0, "checks": []},
            {"action": "drop", "chunk_x": 11, "chunk_z": 0, "checks": []},
            {
                "action": "replace",
                "chunk_x": 8,
                "chunk_z": 1,
                "sections": [cache_air],
                "checks": [{"x": 8, "z": 1}],
            },
            {"action": "radius", "server_view_radius": 1, "checks": []},
            {"action": "radius", "server_view_radius": 6, "checks": [
                {"x": -1, "z": 0},
                {"x": 17, "z": 1},
                {"x": 18, "z": 1},
            ]},
            {"action": "section_emptiness", "section_x": 8, "section_y": -4,
             "section_z": 1, "empty": False, "checks": []},
            {"action": "section_emptiness", "section_x": 99, "section_y": 0,
             "section_z": 99, "empty": True, "checks": []},
            {"action": "flip", "checks": []},
            {"action": "drop", "chunk_x": 18, "chunk_z": 1, "checks": []},
        ],
    }

    block_update_values = [0] * 4096
    block_update_section = section_hex(
        0, 0, 1, [0, 1], block_update_values, 0, [1], [0] * 64)
    yield {
        "case_id": "client-world/block-update-section-counters",
        "family": "math",
        "operation": "client_world_block_update",
        "dimension_min_y": -64,
        "dimension_height": 384,
        "chunk_x": -2,
        "chunk_z": 3,
        "sections": [block_update_section],
        "updates": [
            {
                "label": "same-air-empty-rejected",
                "x": -32, "y": -64, "z": 48, "state_id": 0,
                "queries": [
                    {"label": "unchanged-cell", "x": -32, "y": -64, "z": 48},
                ],
            },
            {
                "label": "air-to-stone",
                "x": -32, "y": -64, "z": 48, "state_id": 1,
                "queries": [
                    {"label": "updated-cell", "x": -32, "y": -64, "z": 48},
                    {"label": "section-edge", "x": -17, "y": -49, "z": 63},
                ],
            },
            {
                "label": "stone-to-water",
                "x": -32, "y": -64, "z": 48, "state_id": 86,
                "queries": [
                    {"label": "updated-cell", "x": -32, "y": -64, "z": 48},
                ],
            },
            {
                "label": "water-to-air",
                "x": -32, "y": -64, "z": 48, "state_id": 0,
                "queries": [
                    {"label": "updated-cell", "x": -32, "y": -64, "z": 48},
                ],
            },
            {
                "label": "air-to-stone-at-negative-edge",
                "x": -17, "y": -49, "z": 63, "state_id": 1,
                "queries": [
                    {"label": "updated-cell", "x": -17, "y": -49, "z": 63},
                ],
            },
            {
                "label": "same-state-stone",
                "x": -17, "y": -49, "z": 63, "state_id": 1,
                "queries": [
                    {"label": "updated-cell", "x": -17, "y": -49, "z": 63},
                ],
            },
            {
                "label": "missing-section-rejected",
                "x": -32, "y": -48, "z": 48, "state_id": 1,
                "queries": [
                    {"label": "missing-section", "x": -32, "y": -48, "z": 48},
                ],
            },
            {
                "label": "outside-chunk-rejected",
                "x": -16, "y": -64, "z": 48, "state_id": 1,
                "queries": [
                    {"label": "outside-chunk", "x": -16, "y": -64, "z": 48},
                ],
            },
        ],
    }

    def light_layer_hex(value):
        layer = bytearray(2048)
        layer[0] = value
        return layer.hex()

    yield {
        "case_id": "client-world/light-update-queue-and-publication",
        "family": "math",
        "operation": "client_world_light",
        "dimension_min_y": -64,
        "dimension_height": 384,
        "operations": [
            {
                "action": "apply",
                "label": "queue-first-unloaded",
                "chunk_x": -2,
                "chunk_z": 3,
                "sky_mask": [1],
                "sky_empty_mask": [2],
                "sky_updates": [light_layer_hex(0xA5)],
                "block_mask": [1],
                "block_empty_mask": [2],
                "block_updates": [light_layer_hex(0xB6)],
                "queries": [],
            },
            {
                "action": "apply",
                "label": "queue-second-replaces-first",
                "chunk_x": -2,
                "chunk_z": 3,
                "sky_mask": [1],
                "sky_empty_mask": [2],
                "sky_updates": [light_layer_hex(0x3C)],
                "block_mask": [1],
                "block_empty_mask": [2],
                "block_updates": [light_layer_hex(0x4D)],
                "queries": [],
            },
            {
                "action": "load",
                "label": "load-and-publish",
                "chunk_x": -2,
                "chunk_z": 3,
                "section_count": 1,
                "non_empty_sections": [0],
                "queries": [
                    {"label": "sky-low", "layer": "sky", "x": -32, "y": -64, "z": 48},
                    {"label": "sky-high-nibble", "layer": "sky", "x": -31, "y": -64, "z": 48},
                    {"label": "block-low", "layer": "block", "x": -32, "y": -64, "z": 48},
                    {"label": "block-high-nibble", "layer": "block", "x": -31, "y": -64, "z": 48},
                    {"label": "empty-sky-section", "layer": "sky", "x": -32, "y": -48, "z": 48},
                    {"label": "empty-block-section", "layer": "block", "x": -32, "y": -48, "z": 48},
                ],
            },
            {
                "action": "apply",
                "label": "replace-loaded-layer",
                "chunk_x": -2,
                "chunk_z": 3,
                "sky_mask": [1],
                "sky_empty_mask": [],
                "sky_updates": [light_layer_hex(0x2E)],
                "block_mask": [1],
                "block_empty_mask": [],
                "block_updates": [light_layer_hex(0x7F)],
                "queries": [
                    {"label": "updated-sky-low", "layer": "sky", "x": -32, "y": -64, "z": 48},
                    {"label": "updated-sky-high-nibble", "layer": "sky", "x": -31, "y": -64, "z": 48},
                    {"label": "updated-block-low", "layer": "block", "x": -32, "y": -64, "z": 48},
                    {"label": "updated-block-high-nibble", "layer": "block", "x": -31, "y": -64, "z": 48},
                ],
            },
        ],
    }

    yield {
        "case_id": "client-world/light-unload-removal",
        "family": "math",
        "operation": "client_world_light",
        "dimension_min_y": -64,
        "dimension_height": 384,
        "operations": [
            {
                "action": "apply",
                "label": "queue-before-unload",
                "chunk_x": -2,
                "chunk_z": 3,
                "sky_mask": [1],
                "sky_empty_mask": [],
                "sky_updates": [light_layer_hex(0x0F)],
                "block_mask": [1],
                "block_empty_mask": [],
                "block_updates": [light_layer_hex(0xA6)],
                "queries": [],
            },
            {
                "action": "unload",
                "label": "forget-clears-queued-light",
                "chunk_x": -2,
                "chunk_z": 3,
                "queries": [
                    {"label": "sky-after-forget", "layer": "sky", "x": -32, "y": -64, "z": 48},
                    {"label": "block-after-forget", "layer": "block", "x": -32, "y": -64, "z": 48},
                ],
            },
            {
                "action": "load",
                "label": "load-after-queued-cleanup",
                "chunk_x": -2,
                "chunk_z": 3,
                "section_count": 1,
                "non_empty_sections": [0],
                "queries": [
                    {"label": "sky-not-republished", "layer": "sky", "x": -32, "y": -64, "z": 48},
                    {"label": "block-not-republished", "layer": "block", "x": -32, "y": -64, "z": 48},
                ],
            },
            {
                "action": "apply",
                "label": "loaded-light-before-unload",
                "chunk_x": -2,
                "chunk_z": 3,
                "sky_mask": [1],
                "sky_empty_mask": [],
                "sky_updates": [light_layer_hex(0x26)],
                "block_mask": [1],
                "block_empty_mask": [],
                "block_updates": [light_layer_hex(0x37)],
                "queries": [
                    {"label": "sky-loaded", "layer": "sky", "x": -32, "y": -64, "z": 48},
                    {"label": "block-loaded", "layer": "block", "x": -32, "y": -64, "z": 48},
                ],
            },
            {
                "action": "unload",
                "label": "forget-clears-loaded-light",
                "chunk_x": -2,
                "chunk_z": 3,
                "queries": [
                    {"label": "sky-after-loaded-forget", "layer": "sky", "x": -32, "y": -64, "z": 48},
                    {"label": "block-after-loaded-forget", "layer": "block", "x": -32, "y": -64, "z": 48},
                ],
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-shared-flags",
        "family": "math",
        "operation": "client_world_entity_shared_flags",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "all-defined-flags",
                "metadata_hex": "0000fbff",
            },
            {
                "label": "air-supply-only-preserves-flags",
                "metadata_hex": "010107ff",
            },
            {
                "label": "clear-shared-flags",
                "metadata_hex": "000000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-air-supply",
        "family": "math",
        "operation": "client_world_entity_air_supply",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "set-positive-air-supply",
                "metadata_hex": "0101fa01ff",
            },
            {
                "label": "combined-signed-air-supply",
                "metadata_hex": "0000400101ffffffff0fff",
            },
            {
                "label": "flags-only-preserves-air-supply",
                "metadata_hex": "000000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-pose",
        "family": "math",
        "operation": "client_world_entity_pose",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "set-crouching",
                "metadata_hex": "061405ff",
            },
            {
                "label": "set-sitting-with-glowing",
                "metadata_hex": "00004006140aff",
            },
            {
                "label": "unknown-pose-falls-back",
                "metadata_hex": "061463ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-ticks-frozen",
        "family": "math",
        "operation": "client_world_entity_ticks_frozen",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "set-positive-freeze",
                "metadata_hex": "070146ff",
            },
            {
                "label": "set-full-freeze",
                "metadata_hex": "07018c01ff",
            },
            {
                "label": "combined-shared-flags-and-negative-freeze",
                "metadata_hex": "0000400701ffffffff0fff",
            },
            {
                "label": "clear-freeze",
                "metadata_hex": "070100ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-custom-name",
        "family": "math",
        "operation": "client_world_entity_custom_name",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "set-custom-name",
                "metadata_hex": "02060108000c446973706c6179206e616d65ff",
            },
            {
                "label": "set-visible-only",
                "metadata_hex": "030801ff",
            },
            {
                "label": "clear-custom-name",
                "metadata_hex": "020600ff",
            },
            {
                "label": "restore-hidden-custom-name",
                "metadata_hex": "02060108000c446973706c6179206e616d65030800ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-living-flags",
        "family": "math",
        "operation": "client_world_entity_living_flags",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-all-living-flags",
                "metadata_hex": "080007ff",
            },
            {
                "label": "combined-shared-air-and-living",
                "metadata_hex": "000040010107080001ff",
            },
            {
                "label": "clear-living-flags",
                "metadata_hex": "080000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-health",
        "family": "math",
        "operation": "client_world_entity_health",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-health-after-living-flags",
                "metadata_hex": "080000090340f00000ff",
            },
            {
                "label": "set-zero-health",
                "metadata_hex": "090300000000ff",
            },
            {
                "label": "no-gravity-preserves-health",
                "metadata_hex": "050801ff",
            },
            {
                "label": "set-negative-health",
                "metadata_hex": "0903bf800000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-mob-flags",
        "family": "math",
        "operation": "client_world_entity_mob_flags",
        "entity_id": 37,
        "entity_type_id": 70,
        "operations": [
            {
                "label": "set-all-mob-flags",
                "metadata_hex": "0f0007ff",
            },
            {
                "label": "combined-shared-air-and-mob",
                "metadata_hex": "0000400101070f0002ff",
            },
            {
                "label": "no-gravity-preserves-mob",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-mob-flags",
                "metadata_hex": "0f0000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-arrow-count",
        "family": "math",
        "operation": "client_world_entity_arrow_count",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-positive-arrow-count",
                "metadata_hex": "0c0107ff",
            },
            {
                "label": "combined-living-and-negative-arrow-count",
                "metadata_hex": "0800010c01ffffffff0fff",
            },
            {
                "label": "no-gravity-preserves-arrow-count",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-arrow-count",
                "metadata_hex": "0c0100ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-effect-ambience",
        "family": "math",
        "operation": "client_world_entity_effect_ambience",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-effect-ambience",
                "metadata_hex": "0b0801ff",
            },
            {
                "label": "combined-living-and-effect-ambience",
                "metadata_hex": "0800010b0801ff",
            },
            {
                "label": "empty-particles-preserves-effect-ambience",
                "metadata_hex": "0a11000b0801ff",
            },
            {
                "label": "no-gravity-preserves-effect-ambience",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-effect-ambience",
                "metadata_hex": "0b0800ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-effect-particles",
        "family": "math",
        "operation": "client_world_entity_effect_particles",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-one-effect-particle",
                "metadata_hex": "0a11011c11223344ff",
            },
            {
                "label": "set-two-effect-particles-with-living-flag",
                "metadata_hex": "0800010a11021c112233441c55667788ff",
            },
            {
                "label": "no-gravity-preserves-effect-particles",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-effect-particles",
                "metadata_hex": "0a1100ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-sleeping-position",
        "family": "math",
        "operation": "client_world_entity_sleeping_position",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-positive-sleeping-position",
                "metadata_hex": "0e0b01000003bffffe905aff",
            },
            {
                "label": "combined-living-and-negative-sleeping-position",
                "metadata_hex": "0800010e0b01ffff0000007fffc0ff",
            },
            {
                "label": "no-gravity-preserves-sleeping-position",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-sleeping-position",
                "metadata_hex": "0e0b00ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-stinger-count",
        "family": "math",
        "operation": "client_world_entity_stinger_count",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-positive-stinger-count",
                "metadata_hex": "0d0102ff",
            },
            {
                "label": "combined-living-and-negative-stinger-count",
                "metadata_hex": "0800010d01ffffffff0fff",
            },
            {
                "label": "no-gravity-preserves-stinger-count",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-stinger-count",
                "metadata_hex": "0d0100ff",
            },
        ],
    }

    def attribute_modifier(identifier, amount, operation):
        return {
            "id": identifier,
            "amount": float_bits(amount, "f64"),
            "operation": operation,
        }

    yield {
        "case_id": "client-world/player-attribute-application-and-effective-value",
        "family": "math",
        "operation": "client_world_player_attributes",
        "player_id": 42,
        "operations": [
            {
                "action": "apply",
                "label": "initial-snapshot",
                "player_id": 42,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(0.25, "f64"),
                        "modifiers": [
                            attribute_modifier("p08:oracle_add", 0.1, 0),
                            attribute_modifier("p08:oracle_base", 0.2, 1),
                            attribute_modifier("p08:oracle_total", 0.5, 2),
                        ],
                    },
                    {
                        "attribute_id": 23,
                        "base": float_bits(40.0, "f64"),
                        "modifiers": [
                            attribute_modifier("p08:health_add", 5.0, 0),
                        ],
                    },
                ],
                "queries": [
                    {
                        "label": "movement-speed-effective",
                        "attribute_id": 26,
                        "min": float_bits(0.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                    {
                        "label": "max-health-effective",
                        "attribute_id": 23,
                        "min": float_bits(1.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                ],
            },
            {
                "action": "apply",
                "label": "movement-speed-replacement",
                "player_id": 42,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(0.5, "f64"),
                        "modifiers": [
                            attribute_modifier("p08:replacement", -0.1, 0),
                        ],
                    },
                ],
                "queries": [
                    {
                        "label": "replacement-clears-old-modifiers",
                        "attribute_id": 26,
                        "min": float_bits(0.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                    {
                        "label": "unmentioned-attribute-preserved",
                        "attribute_id": 23,
                        "min": float_bits(1.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                ],
            },
            {
                "action": "apply",
                "label": "wrong-player-ignored",
                "player_id": 99,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(2.0, "f64"),
                        "modifiers": [
                            attribute_modifier("p08:ignored", 1.0, 0),
                        ],
                    },
                ],
                "queries": [
                    {
                        "label": "state-after-wrong-player",
                        "attribute_id": 26,
                        "min": float_bits(0.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                ],
            },
        ],
    }

    yield {
        "case_id": "client-world/player-attribute-movement-speed-binding",
        "family": "math",
        "operation": "client_world_player_movement_speed",
        "player_id": 42,
        "operations": [
            {
                "label": "default-player-speed",
                "player_id": 42,
                "attributes": [],
                "on_ground": True,
                "block_friction": float_bits(0.6, "f32"),
            },
            {
                "label": "modified-attribute-ground-speed",
                "player_id": 42,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(0.25, "f64"),
                        "modifiers": [
                            attribute_modifier("p10:movement_add", 0.1, 0),
                            attribute_modifier("p10:movement_base", 0.2, 1),
                            attribute_modifier("p10:movement_total", 0.5, 2),
                        ],
                    },
                ],
                "on_ground": True,
                "block_friction": float_bits(0.8, "f32"),
            },
            {
                "label": "replacement-attribute-ground-speed",
                "player_id": 42,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(0.5, "f64"),
                        "modifiers": [
                            attribute_modifier("p10:movement_replacement", -0.1, 0),
                        ],
                    },
                ],
                "on_ground": True,
                "block_friction": float_bits(0.6, "f32"),
            },
            {
                "label": "wrong-player-preserves-speed",
                "player_id": 99,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(2.0, "f64"),
                        "modifiers": [],
                    },
                ],
                "on_ground": True,
                "block_friction": float_bits(0.6, "f32"),
            },
        ],
    }

    player_attribute_water_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "sprinting": False,
        "water_slow_down": float_bits(0.8, "f32"),
        "water_efficiency": float_bits(0.6, "f32"),
        "on_ground": True,
        "dolphins_grace": False,
        "horizontal_collision": False,
        "on_climbable": False,
        "free_after_jump": False,
        "base_gravity": float_bits(0.08, "f64"),
        "is_falling": False,
        "old_y": float_bits(10.0, "f64"),
        "current_y": float_bits(10.0, "f64"),
    }
    player_attribute_water_operations = [
        {
            "label": "default-speed-enters-water-efficiency",
            "player_id": 42,
            "attributes": [],
        },
        {
            "label": "modified-speed-enters-water-efficiency",
            "player_id": 42,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(0.25, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:water_add", 0.1, 0),
                        attribute_modifier("p10:water_base", 0.2, 1),
                        attribute_modifier("p10:water_total", 0.5, 2),
                    ],
                },
            ],
        },
        {
            "label": "replacement-speed-enters-water-efficiency",
            "player_id": 42,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(0.5, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:water_replacement", -0.1, 0),
                    ],
                },
            ],
        },
        {
            "label": "wrong-player-preserves-water-speed",
            "player_id": 99,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(2.0, "f64"),
                    "modifiers": [],
                },
            ],
        },
    ]
    yield {
        "case_id": "client-world/player-attribute-water-travel-binding",
        "family": "math",
        "operation": "client_world_player_attribute_water",
        "player_id": 42,
        "operations": [
            {**player_attribute_water_defaults, **operation}
            for operation in player_attribute_water_operations
        ],
    }

    player_sprint_attribute_operations = [
        {
            "label": "default-start-sprinting",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": True,
            "attributes": [],
        },
        {
            "label": "repeated-start-sprinting",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": True,
            "attributes": [],
        },
        {
            "label": "custom-speed-start-sprinting",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": True,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(0.25, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:sprint_custom", 0.2, 2),
                    ],
                },
            ],
        },
        {
            "label": "stop-sprinting-removes-modifier",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": False,
            "attributes": [],
        },
        {
            "label": "wrong-player-preserves-stopped-state",
            "player_id": 99,
            "action": "set_sprinting",
            "sprinting": True,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(2.0, "f64"),
                    "modifiers": [],
                },
            ],
        },
    ]
    yield {
        "case_id": "client-world/player-sprint-transition-attribute-binding",
        "family": "math",
        "operation": "client_world_player_sprint_attribute",
        "player_id": 42,
        "operations": player_sprint_attribute_operations,
    }

    player_sneaking_input_defaults = {
        "action": "apply_input",
        "player_id": 42,
        "move_x": float_bits(0.0),
        "move_y": float_bits(1.0),
        "jump": False,
        "xxa": float_bits(0.0),
        "zza": float_bits(0.0),
        "jumping": False,
        "x_bob": float_bits(0.0),
        "y_bob": float_bits(0.0),
        "x_bob_o": float_bits(0.0),
        "y_bob_o": float_bits(0.0),
        "x_rot": float_bits(0.0),
        "y_rot": float_bits(0.0),
        "controlled_camera": True,
        "using_item": False,
        "passenger": False,
        "moving_slowly": True,
        "item_use_speed_multiplier": float_bits(0.2),
    }
    player_sneaking_input_operations = [
        {
            "label": "default-sneaking-speed",
            "attributes": [],
        },
        {
            "label": "custom-sneaking-speed",
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.5, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:sneaking_total", 0.5, 2),
                    ],
                },
            ],
        },
        {
            "label": "item-and-sneaking-attribute-order",
            "move_x": float_bits(1.0),
            "move_y": float_bits(1.0),
            "using_item": True,
            "item_use_speed_multiplier": float_bits(0.5),
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.4, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:sneaking_add", 0.1, 0),
                    ],
                },
            ],
        },
        {
            "label": "non-slow-input-ignores-sneaking-speed",
            "moving_slowly": False,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.9, "f64"),
                    "modifiers": [],
                },
            ],
        },
        {
            "label": "wrong-player-preserves-sneaking-speed",
            "player_id": 99,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.8, "f64"),
                    "modifiers": [],
                },
            ],
        },
    ]
    yield {
        "case_id": "client-world/player-sneaking-speed-input-binding",
        "family": "math",
        "operation": "client_world_player_sneaking_input",
        "player_id": 42,
        "operations": [
            {**player_sneaking_input_defaults, **operation}
            for operation in player_sneaking_input_operations
        ],
    }

    player_keyboard_input_defaults = {
        "action": "apply_keyboard_input",
        "player_id": 42,
        "up": False,
        "down": False,
        "left": False,
        "right": False,
        "jump": False,
        "shift": False,
        "sprint": False,
        "controlled_camera": True,
        "using_item": False,
        "passenger": False,
        "moving_slowly": False,
        "item_use_speed_multiplier": float_bits(0.2),
        "attributes": [],
    }
    player_keyboard_input_operations = [
        {
            "label": "forward-key-reaches-local-input",
            "up": True,
        },
        {
            "label": "diagonal-keys-use-sneaking-attribute",
            "up": True,
            "left": True,
            "jump": True,
            "moving_slowly": True,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.4, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:keyboard_sneaking_total", 0.5, 2),
                    ],
                },
            ],
        },
        {
            "label": "opposing-keys-zero-movement",
            "up": True,
            "down": True,
            "left": True,
            "right": True,
            "jump": True,
            "shift": True,
            "sprint": True,
            "moving_slowly": True,
        },
        {
            "label": "item-and-sneaking-key-order",
            "up": True,
            "right": True,
            "using_item": True,
            "item_use_speed_multiplier": float_bits(0.5),
            "moving_slowly": True,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.4, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:keyboard_sneaking_add", 0.1, 0),
                    ],
                },
            ],
        },
        {
            "label": "wrong-player-preserves-key-attribute",
            "player_id": 99,
            "up": True,
            "moving_slowly": True,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.9, "f64"),
                    "modifiers": [],
                },
            ],
        },
    ]
    yield {
        "case_id": "client-world/player-keyboard-input-binding",
        "family": "math",
        "operation": "client_world_player_keyboard_input",
        "player_id": 42,
        "operations": [
            {**player_keyboard_input_defaults, **operation}
            for operation in player_keyboard_input_operations
        ],
    }

    player_tick_defaults = {
        "action": "tick",
        "position": vector_bits(10.0, 64.0, -2.0),
        "rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "ticks": 1,
        "head_target": None,
        "effects": [],
    }
    player_tick_operations = [
        {
            "label": "head-target-and-effect-start",
            "head_target": float_bits(135.0),
        },
        {
            "label": "position-and-rotation-update-before-tick",
            "position": vector_bits(14.0, 68.0, 6.0),
            "rotation": {
                "y": float_bits(10.0),
                "x": float_bits(20.0),
            },
            "ticks": 0,
        },
        {
            "label": "head-target-reached-after-position-tick",
            "position": vector_bits(14.0, 68.0, 6.0),
            "rotation": {
                "y": float_bits(10.0),
                "x": float_bits(20.0),
            },
            "ticks": 2,
        },
        {
            "label": "effect-replacement-before-tick",
            "effects": [
                {
                    "effect_id": 18,
                    "amplifier": 1,
                    "duration_ticks": 2,
                    "flags": 15,
                },
            ],
        },
    ]
    yield {
        "case_id": "client-world/player-tick-state",
        "family": "math",
        "operation": "client_world_player_tick",
        "player_id": 42,
        "initial_position": vector_bits(10.0, 64.0, -2.0),
        "initial_rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(350.0),
        "initial_effects": [
            {
                "effect_id": 18,
                "amplifier": 0,
                "duration_ticks": 3,
                "flags": 8,
            },
            {
                "effect_id": 19,
                "amplifier": 1,
                "duration_ticks": -1,
                "flags": 0,
            },
        ],
        "operations": [
            {**player_tick_defaults, **operation}
            for operation in player_tick_operations
        ],
    }

    remote_player_head_defaults = {
        "action": "rotate_head",
        "head_target": None,
        "ticks": 0,
    }
    remote_player_head_operations = [
        {
            "label": "packet-target-wraps-across-zero",
            "head_target": float_bits(10.0),
        },
        {
            "label": "first-tick-uses-shortest-path",
            "ticks": 1,
        },
        {
            "label": "second-tick-before-replacement",
            "ticks": 1,
        },
        {
            "label": "new-packet-restarts-three-step-target",
            "head_target": float_bits(-180.0),
        },
        {
            "label": "replacement-first-tick",
            "ticks": 1,
        },
        {
            "label": "replacement-reaches-after-two-more-ticks",
            "ticks": 2,
        },
        {
            "label": "negative-packet-target-starts-new-interpolation",
            "head_target": float_bits(-90.0),
        },
        {
            "label": "negative-target-reaches-after-three-ticks",
            "ticks": 3,
        },
    ]
    yield {
        "case_id": "client-world/entity-player-head-rotation",
        "family": "math",
        "operation": "client_world_entity_player_head_rotation",
        "entity_id": 77,
        "entity_type_id": 156,
        "initial_position": vector_bits(10.0, 64.0, -2.0),
        "initial_rotation": {
            "y": float_bits(15.0),
            "x": float_bits(-10.0),
        },
        "initial_head": float_bits(350.0),
        "operations": [
            {**remote_player_head_defaults, **operation}
            for operation in remote_player_head_operations
        ],
    }

    yield {
        "case_id": "client-world/entity-snowball-tick",
        "family": "math",
        "operation": "client_world_entity_snowball_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 1.0, 2.0),
                "movement": vector_bits(0.25, 0.5, 0.25),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(-20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 1.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(355.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-arrow-tick",
        "family": "math",
        "operation": "client_world_entity_arrow_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 1.0, 2.0),
                "movement": vector_bits(0.25, 0.5, 0.25),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(-20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 1.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(355.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-experience-orb-tick",
        "family": "math",
        "operation": "client_world_entity_experience_orb_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 10.0, 2.0),
                "movement": vector_bits(0.125, 0.25, -0.125),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 10.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(45.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-client-tick", "ticks": 1},
            {"action": "tick", "label": "second-client-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-primed-tnt-tick",
        "family": "math",
        "operation": "client_world_entity_primed_tnt_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 10.0, 2.0),
                "movement": vector_bits(0.125, 0.25, -0.125),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 10.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(45.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-falling-block-tick",
        "family": "math",
        "operation": "client_world_entity_falling_block_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 10.0, 2.0),
                "movement": vector_bits(0.125, 0.25, -0.125),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 10.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(45.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-firework-tick",
        "family": "math",
        "operation": "client_world_entity_firework_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 10.0, 2.0),
                "movement": vector_bits(0.25, 0.5, 0.125),
                "rotation": {
                    "y": float_bits(355.0),
                    "x": float_bits(-20.0),
                },
                "no_gravity": False,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-boat-air-tick",
        "family": "math",
        "operation": "client_world_entity_boat_air_tick",
        "player_id": 38,
        "boat": {
            "entity_id": 37,
            "position": vector_bits(2.0, 10.0, 2.0),
            "movement": vector_bits(0.25, 0.5, -0.75),
            "rotation": {
                "y": float_bits(10.0),
                "x": float_bits(-20.0),
            },
            "no_gravity": False,
        },
        "input": {
            "left": False,
            "right": True,
            "forward": True,
            "backward": False,
        },
        "operations": [
            {"action": "tick", "label": "air-gravity-control", "ticks": 1},
            {"action": "set_no_gravity", "label": "enable-no-gravity", "value": True},
            {"action": "tick", "label": "air-no-gravity-control", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-boat-remote-tick",
        "family": "math",
        "operation": "client_world_entity_boat_remote_tick",
        "passenger": {
            "entity_id": 38,
            "type_id": 151,
        },
        "boat": {
            "entity_id": 37,
            "position": vector_bits(2.0, 12.0, 2.0),
            "movement": vector_bits(0.25, 0.5, -0.75),
            "rotation": {
                "y": float_bits(10.0),
                "x": float_bits(-20.0),
            },
            "no_gravity": False,
        },
        "input": {
            "left": False,
            "right": True,
            "forward": True,
            "backward": False,
        },
        "operations": [
            {"action": "tick", "label": "remote-first-tick", "ticks": 1},
            {"action": "tick", "label": "remote-second-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-ticking-eligibility",
        "family": "math",
        "operation": "client_world_entity_ticking_eligibility",
        "player": {
            "entity_id": 42,
            "position": vector_bits(10.0, 64.0, -2.0),
        },
        "remote": {
            "entity_id": 37,
            "type_id": 71,
            "position": vector_bits(1.25, 80.0, -35.5),
        },
        "operations": [
            {"action": "tick", "label": "loaded-first-tick", "ticks": 1},
            {
                "action": "unload_chunk",
                "label": "remote-chunk-unloaded",
                "chunk_x": 0,
                "chunk_z": -3,
            },
            {"action": "tick", "label": "unloaded-tick", "ticks": 1},
            {
                "action": "load_chunk",
                "label": "remote-chunk-reloaded",
                "chunk_x": 0,
                "chunk_z": -3,
            },
            {"action": "tick", "label": "reloaded-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-transform-history",
        "family": "math",
        "operation": "client_world_entity_transform_history",
        "player": {
            "entity_id": 42,
            "position": vector_bits(10.0, 64.0, -2.0),
            "rotation": {
                "y": float_bits(350.0),
                "x": float_bits(-20.0),
            },
        },
        "entity": {
            "entity_id": 37,
            "type_id": 151,
            "position": vector_bits(1.25, 80.0, -35.5),
            "rotation": {
                "y": float_bits(350.0),
                "x": float_bits(-20.0),
                "head": float_bits(350.0),
            },
        },
        "operations": [
            {"action": "tick", "label": "snapshot-initial", "ticks": 1},
            {
                "action": "update",
                "label": "updated-transform",
                "player": {
                    "position": vector_bits(14.0, 68.0, 6.0),
                    "rotation": {
                        "y": float_bits(10.0),
                        "x": float_bits(20.0),
                    },
                },
                "entity": {
                    "position": vector_bits(3.0, 82.0, -34.5),
                    "head": float_bits(10.0),
                },
                "partial_tick": float_bits(0.5),
            },
            {
                "action": "sample",
                "label": "partial-one",
                "partial_tick": float_bits(1.0),
            },
            {
                "action": "tick",
                "label": "snapshot-updated",
                "ticks": 1,
                "partial_tick": float_bits(0.5),
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-player-interpolation",
        "family": "math",
        "operation": "client_world_entity_player_interpolation",
        "entity_id": 37,
        "initial_position": vector_bits(1.0, 80.0, -35.5),
        "initial_rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(350.0),
        "operations": [
            {
                "action": "move",
                "label": "packet-target-starts-interpolation",
                "delta": {"x": 12288, "y": 8192, "z": 4096},
                "target_position": vector_bits(4.0, 82.0, -34.5),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
            },
            {"action": "tick", "label": "first-interpolation-tick", "ticks": 1},
            {
                "action": "move",
                "label": "same-target-keeps-remaining-steps",
                "delta": {"x": 0, "y": 0, "z": 0},
                "target_position": vector_bits(4.0, 82.0, -34.5),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
            },
            {"action": "tick", "label": "second-interpolation-tick", "ticks": 1},
            {
                "action": "move",
                "label": "changed-target-restarts-interpolation",
                "delta": {"x": 4096, "y": -4096, "z": 4096},
                "target_position": vector_bits(5.0, 81.0, -33.5),
                "rotation": {
                    "y": float_bits(190.0),
                    "x": float_bits(-10.0),
                },
            },
            {"action": "tick", "label": "replacement-first-tick", "ticks": 1},
            {"action": "tick", "label": "replacement-final-two-ticks", "ticks": 2},
        ],
    }

    yield {
        "case_id": "client-world/entity-player-snap",
        "family": "math",
        "operation": "client_world_entity_player_snap",
        "entity_id": 37,
        "initial_position": vector_bits(1.0, 80.0, -35.5),
        "initial_rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(350.0),
        "operations": [
            {
                "action": "position_sync",
                "label": "exact-threshold-interpolates",
                "target_position": vector_bits(65.0, 80.0, -35.5),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
            },
            {"action": "tick", "label": "exact-threshold-first-tick", "ticks": 1},
            {
                "action": "position_sync",
                "label": "over-threshold-snaps",
                "target_position": vector_bits(90.0, 80.0, -35.5),
                "rotation": {
                    "y": float_bits(20.0),
                    "x": float_bits(10.0),
                },
            },
            {
                "action": "position_sync",
                "label": "exact-threshold-after-snap",
                "target_position": vector_bits(154.0, 80.0, -35.5),
                "rotation": {
                    "y": float_bits(30.0),
                    "x": float_bits(5.0),
                },
            },
            {"action": "tick", "label": "boundary-first-tick", "ticks": 1},
            {
                "action": "position_sync",
                "label": "second-over-threshold-snaps",
                "target_position": vector_bits(219.0, 80.0, -35.5),
                "rotation": {
                    "y": float_bits(40.0),
                    "x": float_bits(-5.0),
                },
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-player-motion",
        "family": "math",
        "operation": "client_world_entity_player_motion",
        "entity_id": 37,
        "initial_position": vector_bits(1.0, 80.0, -35.5),
        "initial_rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(350.0),
        "initial_motion": vector_bits(0.125, 0.25, -0.375),
        "operations": [
            {
                "action": "motion",
                "label": "packet-target-keeps-current-motion",
                "target_motion": vector_bits(0.875, -0.5, 0.625),
            },
            {"action": "tick", "label": "first-motion-tick", "ticks": 1},
            {"action": "tick", "label": "second-motion-tick", "ticks": 1},
            {
                "action": "motion",
                "label": "replacement-target-resets-three-steps",
                "target_motion": vector_bits(-0.375, 0.75, 1.125),
            },
            {"action": "tick", "label": "replacement-first-tick", "ticks": 1},
            {"action": "tick", "label": "replacement-final-two-ticks", "ticks": 2},
        ],
    }

    yield {
        "case_id": "client-world/entity-teleport",
        "family": "math",
        "operation": "client_world_entity_teleport",
        "entity_id": 37,
        "initial_position": vector_bits(1.0, 80.0, -35.5),
        "initial_rotation": {
            "y": float_bits(30.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(40.0),
        "initial_motion": vector_bits(1.0, 2.0, 3.0),
        "operations": [
            {
                "action": "teleport",
                "label": "relative-all-interpolates",
                "position": vector_bits(2.0, 3.0, 4.0),
                "delta_motion": vector_bits(0.25, -0.5, 0.75),
                "rotation": {
                    "y": float_bits(15.0),
                    "x": float_bits(10.0),
                },
                "relative_flags": 511,
            },
            {"action": "tick", "label": "relative-first-tick", "ticks": 1},
            {
                "action": "teleport",
                "label": "absolute-over-threshold-snaps",
                "position": vector_bits(100.0, 80.0, -35.5),
                "delta_motion": vector_bits(0.5, 0.25, -0.75),
                "rotation": {
                    "y": float_bits(-170.0),
                    "x": float_bits(80.0),
                },
                "relative_flags": 0,
            },
            {
                "action": "teleport",
                "label": "relative-after-snap",
                "position": vector_bits(1.0, 1.0, 1.0),
                "delta_motion": vector_bits(-0.125, 0.25, 0.5),
                "rotation": {
                    "y": float_bits(5.0),
                    "x": float_bits(-5.0),
                },
                "relative_flags": 255,
            },
            {"action": "tick", "label": "relative-after-snap-first-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-passengers",
        "family": "math",
        "operation": "client_world_entity_passengers",
        "entities": [
            {"entity_id": 37, "type_id": 89},
            {"entity_id": 38, "type_id": 120},
            {"entity_id": 39, "type_id": 89},
            {"entity_id": 40, "type_id": 71},
            {"entity_id": 41, "type_id": 120},
            {"entity_id": 42, "type_id": 156},
        ],
        "initial_links": [
            {"vehicle_id": 39, "passenger_ids": [38]},
            {"vehicle_id": 37, "passenger_ids": [40]},
            {"vehicle_id": 40, "passenger_ids": [41]},
        ],
        "operations": [
            {
                "label": "replacement-reparent-duplicate-unknown",
                "vehicle_id": 37,
                "passenger_ids": [38, 39, 999, 38, 42],
            },
            {
                "label": "cycle-rejected-after-reparent",
                "vehicle_id": 39,
                "passenger_ids": [37],
            },
            {"label": "empty-replacement", "vehicle_id": 37, "passenger_ids": []},
            {
                "label": "unknown-vehicle-ignored",
                "vehicle_id": 999,
                "passenger_ids": [38],
            },
            {
                "label": "nested-relation-order",
                "vehicle_id": 40,
                "passenger_ids": [37, 42],
            },
            {
                "label": "cycle-keeps-existing-links",
                "vehicle_id": 37,
                "passenger_ids": [38, 40],
            },
            {
                "label": "nested-replacement",
                "vehicle_id": 40,
                "passenger_ids": [41],
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-passenger-tick",
        "family": "math",
        "operation": "client_world_entity_passenger_tick",
        "vehicle": {
            "entity_id": 37,
            "type_id": 71,
            "position": vector_bits(2.0, 2.0, 2.0),
        },
        "passenger": {
            "entity_id": 38,
            "type_id": 120,
            "position": vector_bits(2.0, 2.0, 2.0),
            "movement": vector_bits(0.125, 0.0, 0.0),
            "rotation": {"y": float_bits(10.0), "x": float_bits(-15.0)},
            "no_gravity": False,
        },
        "root_entity": {
            "entity_id": 39,
            "type_id": 120,
            "position": vector_bits(5.0, 2.0, 2.0),
            "movement": vector_bits(0.125, 0.0, 0.0),
            "rotation": {"y": float_bits(10.0), "x": float_bits(-15.0)},
            "no_gravity": False,
        },
        "operations": [
            {"action": "tick", "label": "passenger-reset-first-tick", "ticks": 1},
            {"action": "tick", "label": "passenger-reset-second-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-passenger-position",
        "family": "math",
        "operation": "client_world_entity_passenger_position",
        "vehicle": {
            "entity_id": 37,
            "type_id": 89,
            "position": vector_bits(2.0, 2.0, 2.0),
            "rotation": {"y": float_bits(90.0), "x": float_bits(0.0)},
            "no_gravity": False,
        },
        "passengers": [
            {
                "entity_id": 38,
                "type_id": 120,
                "position": vector_bits(2.0, 12.0, 2.0),
                "movement": vector_bits(0.125, 0.0, 0.0),
                "rotation": {"y": float_bits(10.0), "x": float_bits(-15.0)},
                "no_gravity": False,
            },
            {
                "entity_id": 39,
                "type_id": 120,
                "position": vector_bits(2.0, 12.0, 2.0),
                "movement": vector_bits(-0.125, 0.0, 0.0),
                "rotation": {"y": float_bits(-20.0), "x": float_bits(25.0)},
                "no_gravity": False,
            },
        ],
        "initial_passenger_ids": [38],
        "operations": [
            {"action": "tick", "label": "single-seat", "ticks": 1},
            {
                "action": "set-passengers-and-tick",
                "label": "multi-seat",
                "passenger_ids": [38, 39],
                "ticks": 1,
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-passenger-player-position",
        "family": "math",
        "operation": "client_world_entity_passenger_player_position",
        "vehicle": {
            "entity_id": 37,
            "type_id": 89,
            "position": vector_bits(2.0, 2.0, 2.0),
            "rotation": {"y": float_bits(90.0), "x": float_bits(0.0)},
            "no_gravity": False,
        },
        "passenger": {
            "entity_id": 38,
            "type_id": 156,
            "position": vector_bits(2.0, 12.0, 2.0),
            "movement": vector_bits(0.125, 0.0, 0.0),
            "rotation": {"y": float_bits(10.0), "x": float_bits(-15.0)},
            "no_gravity": False,
        },
        "operations": [
            {"action": "tick", "label": "player-seat", "ticks": 1},
        ],
    }

    boat_family_rows = [
        ("oak-boat", 37, 89, 38),
        ("oak-chest-boat", 39, 90, 40),
        ("bamboo-raft", 41, 9, 42),
        ("bamboo-chest-raft", 43, 8, 44),
    ]
    yield {
        "case_id": "client-world/entity-passenger-boat-family-position",
        "family": "math",
        "operation": "client_world_entity_passenger_boat_family_position",
        "rows": [
            {
                "label": label,
                "vehicle": {
                    "entity_id": vehicle_id,
                    "type_id": vehicle_type_id,
                    "position": vector_bits(2.0, 10.0, 2.0),
                    "rotation": {"y": float_bits(90.0), "x": float_bits(0.0)},
                    "no_gravity": False,
                },
                "passenger": {
                    "entity_id": passenger_id,
                    "type_id": 120,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(0.125, 0.0, 0.0),
                    "rotation": {"y": float_bits(10.0), "x": float_bits(-15.0)},
                    "no_gravity": False,
                },
            }
            for label, vehicle_id, vehicle_type_id, passenger_id in boat_family_rows
        ],
        "operations": [
            {"action": "tick", "label": "family-seat", "ticks": 1},
        ],
    }

    rotation_rows = [
        ("snowball-clamp", 45, 120, 46, 170.0),
        ("player-clamp", 47, 156, 48, 140.0),
    ]
    yield {
        "case_id": "client-world/entity-passenger-rotation",
        "family": "math",
        "operation": "client_world_entity_passenger_rotation",
        "rows": [
            {
                "label": label,
                "vehicle": {
                    "entity_id": vehicle_id,
                    "type_id": 89,
                    "position": vector_bits(2.0, 2.0, 2.0),
                    "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
                "passenger": {
                    "entity_id": passenger_id,
                    "type_id": passenger_type_id,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 0.0),
                    "rotation": {"y": float_bits(passenger_yaw), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
            }
            for label, vehicle_id, passenger_type_id, passenger_id, passenger_yaw
            in rotation_rows
        ],
        "operations": [
            {"action": "tick", "label": "clamp", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-passenger-boat-turn-exemption",
        "family": "math",
        "operation": "client_world_entity_passenger_boat_turn_exemption",
        "vehicle": {
            "entity_id": 49,
            "type_id": 89,
            "position": vector_bits(2.0, 2.0, 2.0),
            "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
            "no_gravity": True,
        },
        "passenger": {
            "entity_id": 50,
            "type_id": 17,
            "position": vector_bits(2.0, 12.0, 2.0),
            "movement": vector_bits(0.0, 0.0, 0.0),
            "rotation": {"y": float_bits(170.0), "x": float_bits(0.0)},
            "no_gravity": True,
        },
        "operations": [
            {"action": "tick", "label": "breeze-exemption", "ticks": 1},
        ],
    }

    animal_rotation_rows = [
        ("even-animal", 51, 52, 53),
        ("odd-animal", 54, 55, 56),
    ]
    yield {
        "case_id": "client-world/entity-passenger-animal-rotation",
        "family": "math",
        "operation": "client_world_entity_passenger_animal_rotation",
        "rows": [
            {
                "label": label,
                "vehicle": {
                    "entity_id": vehicle_id,
                    "type_id": 89,
                    "position": vector_bits(2.0, 2.0, 2.0),
                    "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
                "animal": {
                    "entity_id": animal_id,
                    "type_id": 30,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 0.0),
                    "rotation": {"y": float_bits(170.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
                "filler": {
                    "entity_id": filler_id,
                    "type_id": 120,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 0.0),
                    "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
            }
            for label, vehicle_id, animal_id, filler_id in animal_rotation_rows
        ],
        "operations": [
            {"action": "tick", "label": "animal-capacity-offset", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-passenger-animal-position",
        "family": "math",
        "operation": "client_world_entity_passenger_animal_position",
        "vehicle": {
            "entity_id": 57,
            "type_id": 89,
            "position": vector_bits(2.0, 2.0, 2.0),
            "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
            "no_gravity": True,
        },
        "animal": {
            "entity_id": 58,
            "type_id": 30,
            "position": vector_bits(2.0, 12.0, 2.0),
            "movement": vector_bits(0.0, 0.0, 0.0),
            "rotation": {"y": float_bits(170.0), "x": float_bits(0.0)},
            "no_gravity": True,
        },
        "operations": [
            {"action": "tick", "label": "single-animal-seat", "ticks": 1},
        ],
    }

    turn_rows = [
        ("player-callback", 59, 60, 156),
        ("breeze-callback", 61, 62, 17),
    ]
    yield {
        "case_id": "client-world/entity-passenger-turn",
        "family": "math",
        "operation": "client_world_entity_passenger_turn",
        "rows": [
            {
                "label": label,
                "vehicle": {
                    "entity_id": vehicle_id,
                    "type_id": 89,
                    "position": vector_bits(2.0, 2.0, 2.0),
                    "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
                "passenger": {
                    "entity_id": passenger_id,
                    "type_id": passenger_type_id,
                    "position": vector_bits(2.0, 2.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 0.0),
                    "rotation": {"y": float_bits(168.75), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
            }
            for label, vehicle_id, passenger_id, passenger_type_id in turn_rows
        ],
        "operations": [
            {
                "action": "turn",
                "label": "direct-passenger-turn",
                "mouse_x": -100.0,
                "mouse_y": 0.0,
            },
        ],
    }

    def sprint_key_presses(
        forward=False, backward=False, left=False, right=False,
        jump=False, shift=False, sprint=False,
    ):
        return {
            "forward": forward,
            "backward": backward,
            "left": left,
            "right": right,
            "jump": jump,
            "shift": shift,
            "sprint": sprint,
        }

    sprint_trigger_conditions = {
        "mobility_restricted": False,
        "enough_food": True,
        "mayfly": False,
        "shallow_water": False,
        "flying": False,
        "slow_due_to_using_item": False,
        "fall_flying": False,
        "underwater": False,
        "moving_slowly": False,
    }

    sprint_trigger_sequences = [
        {
            "label": "double-tap-within-window",
            "operations": [
                {
                    "label": "first-forward-opens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "release-keeps-window",
                    "key_presses": sprint_key_presses(),
                    "move_vector": vector2_bits(0.0, 0.0),
                    "ticks": 1,
                },
                {
                    "label": "second-forward-starts-sprint",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
            ],
        },
        {
            "label": "window-expiry-and-sprint-key",
            "operations": [
                {
                    "label": "first-forward-opens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "release-expires-window",
                    "key_presses": sprint_key_presses(),
                    "move_vector": vector2_bits(0.0, 0.0),
                    "ticks": 7,
                },
                {
                    "label": "forward-after-expiry-reopens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "sprint-key-starts-immediately",
                    "key_presses": sprint_key_presses(forward=True, sprint=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
            ],
        },
        {
            "label": "shift-resets-window",
            "operations": [
                {
                    "label": "first-forward-opens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "shift-press-is-current-only",
                    "key_presses": sprint_key_presses(shift=True),
                    "move_vector": vector2_bits(0.0, 0.0),
                    "ticks": 1,
                },
                {
                    "label": "previous-shift-clears-window",
                    "key_presses": sprint_key_presses(),
                    "move_vector": vector2_bits(0.0, 0.0),
                    "ticks": 1,
                },
                {
                    "label": "forward-after-shift-opens-new-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
            ],
        },
        {
            "label": "backward-resets-window",
            "operations": [
                {
                    "label": "first-forward-opens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "backward-clears-window",
                    "key_presses": sprint_key_presses(backward=True),
                    "move_vector": vector2_bits(0.0, -1.0),
                    "ticks": 1,
                },
                {
                    "label": "forward-after-backward-opens-new-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
            ],
        },
    ]
    yield {
        "case_id": "client-world/player-sprint-trigger-window",
        "family": "math",
        "operation": "client_world_player_sprint_trigger",
        "player_id": 42,
        "sprint_window": 7,
        "conditions": sprint_trigger_conditions,
        "sequences": sprint_trigger_sequences,
    }

    def character_event_case(label, codepoint):
        return {
            "case_id": f"character-event/{label}", "family": "math",
            "operation": "character_event", "codepoint": codepoint,
        }

    character_event_cases = [
        character_event_case("nul", 0),
        character_event_case("space", 32),
        character_event_case("ascii", 65),
        character_event_case("newline", 10),
        character_event_case("delete", 127),
        character_event_case("section-sign", 167),
        character_event_case("accented", 0x00E9),
        character_event_case("surrogate", 0xD800),
        character_event_case("supplementary", 0x1F600),
        character_event_case("maximum", 0x10FFFF),
    ]
    yield from character_event_cases

    def preedit_event_case(label, codepoints, block_sizes, focused_block, caret):
        return {
            "case_id": f"preedit-event/{label}", "family": "math",
            "operation": "preedit_event", "codepoints": codepoints,
            "block_sizes": block_sizes, "focused_block": focused_block, "caret": caret,
        }

    preedit_event_cases = [
        preedit_event_case("empty", [], [0], 0, 0),
        preedit_event_case("single-block", [65, 66], [2], 0, 1),
        preedit_event_case("multiple-blocks", [65, 66, 67], [1, 2], 1, 2),
        preedit_event_case("supplementary-caret", [0x1F600, 0x00E9, 66], [2, 1], 0, 1),
        preedit_event_case("block-boundary-caret", [65, 0x1F600, 66], [1, 2], 1, 1),
        preedit_event_case("end-caret", [65, 0x1F600], [2], 0, 2),
        preedit_event_case("negative-block-size", [65], [-1, 1], 1, 0),
        preedit_event_case("ignored-trailing-codepoints", [65, 66, 67], [1], 0, 2),
    ]
    yield from preedit_event_cases

    def option_values_case(label, minimum, maximum, apply_value_immediately,
                           int_values, sliders, unit_values):
        return {
            "case_id": f"option-values/{label}", "family": "math",
            "operation": "option_values", "min": minimum, "max": maximum,
            "apply_value_immediately": apply_value_immediately, "int_values": int_values,
            "sliders": [float_bits(value, "f64") for value in sliders],
            "unit_values": [float_bits(value, "f64") for value in unit_values],
        }

    option_value_cases = [
        option_values_case(
            "zero-to-ten", 0, 10, True, [-1, 0, 1, 5, 10, 11],
            [-float("inf"), -0.2, 0.0, 0.05, 0.5, 0.99999, 1.0, 1.5, float("inf"), float("nan")],
            [-0.1, -0.0, 0.0, 0.5, 1.0, 1.1, float("inf"), float("nan")]),
        option_values_case(
            "negative-range", -3, 3, False, [-4, -3, -2, 0, 3, 4],
            [-0.5, 0.0, 0.25, 0.5, 0.75, 1.0],
            [0.0, 0.25, 1.0]),
        option_values_case(
            "integer-edge", 2_147_483_646, 2_147_483_647, True,
            [2_147_483_646, 2_147_483_647], [0.0, 0.5, 0.99999, 1.0], [0.0, 1.0]),
    ]
    yield from option_value_cases

    def sliderable_enum_case(label, probe_values, sliders):
        return {
            "case_id": f"sliderable-enum/{label}", "family": "math",
            "operation": "sliderable_enum",
            "probes": [{"value": value} for value in probe_values],
            "sliders": [float_bits(value, "f64") for value in sliders],
        }

    yield sliderable_enum_case(
        "graphics-presets",
        ["FAST", "FANCY", "FABULOUS", "CUSTOM"],
        [
            -float("inf"), -0.0, 0.0, 0.124999999999, 0.125, 0.249999999999,
            0.25, 0.5, 0.75, 0.99999, 1.0, 1.5, float("inf"), float("nan"),
        ],
    )

    def cycleable_enum_case(label, values, probes):
        return {
            "case_id": f"cycleable-enum/{label}", "family": "math",
            "operation": "cycleable_enum", "values": values, "probes": probes,
        }

    yield cycleable_enum_case(
        "membership-and-order",
        ["off", "fast", "fancy", "fabulous"],
        ["off", "fabulous", "missing", ""],
    )
    yield cycleable_enum_case(
        "duplicate-values",
        ["first", "first", "last"],
        ["first", "last", "outside"],
    )

    def alt_enum_case(label, values, alt_values, conditions, probes):
        return {
            "case_id": f"alt-enum/{label}", "family": "math",
            "operation": "alt_enum", "values": values, "alt_values": alt_values,
            "conditions": conditions, "probes": probes,
        }

    yield alt_enum_case(
        "condition-switch",
        ["default", "shared", "only-default"],
        ["alternate", "shared", "only-alternate"],
        [False, True, False, True],
        ["default", "alternate", "shared", "only-alternate", "missing"],
    )
    yield alt_enum_case(
        "duplicates-and-empty-alternative",
        ["first", "first", "last"],
        [],
        [True, False, True, False],
        ["first", "last", "outside"],
    )

    def lazy_enum_case(label, value_lists, valid_values, probes):
        return {
            "case_id": f"lazy-enum/{label}", "family": "math",
            "operation": "lazy_enum", "value_lists": value_lists,
            "valid_values": valid_values, "probes": probes,
        }

    yield lazy_enum_case(
        "dynamic-values",
        [[""], ["", "speakers"], ["", "headphones"]],
        ["", "speakers"],
        ["", "speakers", "headphones", "missing"],
    )
    yield lazy_enum_case(
        "validation-independent-of-list",
        [["default", "alternate"], ["default"]],
        ["default", "external"],
        ["default", "alternate", "external"],
    )

    def clamping_lazy_max_int_range_case(label, minimum, max_values, encodable_max, probes, sliders):
        return {
            "case_id": f"clamping-lazy-max-int-range/{label}", "family": "math",
            "operation": "clamping_lazy_max_int_range", "min": minimum,
            "max_values": max_values, "encodable_max": encodable_max,
            "probes": probes, "sliders": [float_bits(value, "f64") for value in sliders],
        }

    yield clamping_lazy_max_int_range_case(
        "dynamic-upper-bound", -2, [3, 5], 7,
        [-3, -2, -1, 0, 3, 5, 6],
        [-float("inf"), -0.2, 0.0, 0.1, 0.5, 0.99999, 1.0, 1.5, float("inf"), float("nan")],
    )

    def mapped_int_range_case(label, minimum, maximum, discrete, values, sliders):
        return {
            "case_id": f"mapped-int-range/{label}", "family": "math",
            "operation": "mapped_int_range", "min": minimum, "max": maximum,
            "discrete": discrete, "values": values,
            "sliders": [float_bits(value, "f64") for value in sliders],
        }

    yield mapped_int_range_case(
        "discrete",
        -2,
        3,
        True,
        ["-3", "-2", "-1", "0", "2", "3", "4"],
        [-float("inf"), -0.2, 0.0, 0.1, 0.5, 0.99999, 1.0, 1.5, float("inf"), float("nan")],
    )
    yield mapped_int_range_case(
        "continuous",
        0,
        4,
        False,
        ["-1", "0", "2", "4", "5"],
        [-0.25, 0.0, 0.2, 0.5, 1.0],
    )

    def mapped_unit_double_case(label, values, sliders):
        return {
            "case_id": f"mapped-unit-double/{label}", "family": "math",
            "operation": "mapped_unit_double",
            "values": [float_bits(value, "f64") for value in values],
            "sliders": [float_bits(value, "f64") for value in sliders],
        }

    yield mapped_unit_double_case(
        "square-root",
        [-float("inf"), -1.0, -0.0, 0.0, 0.25, 0.5, 1.0, 1.0000000001, float("inf"), float("nan")],
        [-float("inf"), -0.25, -0.0, 0.0, 0.25, 0.5, 1.0, 1.0000000001, float("inf"), float("nan")],
    )
    yield mapped_unit_double_case(
        "rounding-and-subnormal",
        [5e-324, 0.1, 1.0 / 3.0, 0.9999999999999999],
        [5e-324, 0.1, 1.0 / 3.0, 0.9999999999999999],
    )

    def scroll_wheel_case(label, scrolls, selections):
        return {
            "case_id": f"scroll-wheel/{label}", "family": "math",
            "operation": "scroll_wheel",
            "scrolls": [{"x": float_bits(x, "f64"), "y": float_bits(y, "f64")} for x, y in scrolls],
            "selections": [
                {"wheel": float_bits(wheel, "f64"), "current": current, "limit": limit}
                for wheel, current, limit in selections
            ],
        }

    scroll_wheel_cases = [
        scroll_wheel_case(
            "fractional-and-both-axes",
            [(0.4, 0.0), (0.4, 0.6), (0.4, 0.6), (-0.2, -0.6), (-1.0, -1.0)],
            [(1.0, 0, 9), (-1.0, 0, 9), (0.0, 4, 9), (float("nan"), 4, 9),
             (float("inf"), 0, 9), (float("-inf"), 0, 9)],
        ),
        scroll_wheel_case(
            "negative-and-selection-wrap",
            [(-0.6, -0.6), (-0.6, -0.6), (-0.8, -0.8), (0.1, 0.1), (1.0, 0.0)],
            [(1.0, 0, 9), (-1.0, 0, 9), (1.0, 8, 9), (-1.0, 0, 9),
             (float("nan"), -1, 9), (float("inf"), 8, 9)],
        ),
    ]
    yield from scroll_wheel_cases

    def smooth_double_case(label, steps):
        return {
            "case_id": f"smooth-double/{label}", "family": "math",
            "operation": "smooth_double", "steps": steps,
        }

    def smooth_delta(target_delta, time):
        return {
            "op": "delta", "target_delta": float_bits(target_delta, "f64"),
            "time": float_bits(time, "f64"),
        }

    smooth_double_cases = [
        smooth_double_case("positive-negative", [
            smooth_delta(1.0, 1.0), smooth_delta(1.0, 0.5), smooth_delta(-0.25, 1.0),
            smooth_delta(0.0, 2.0), smooth_delta(-2.0, 0.25),
        ]),
        smooth_double_case("reset", [
            smooth_delta(4.0, 0.25), {"op": "reset"}, smooth_delta(-1.0, 1.0),
            smooth_delta(0.5, 0.0), {"op": "reset"}, smooth_delta(0.0, 1.0),
        ]),
        smooth_double_case("signed-zero", [
            smooth_delta(-0.0, 1.0), smooth_delta(0.0, -0.0), smooth_delta(-1.0, -0.5),
        ]),
        smooth_double_case("special-values", [
            smooth_delta(float("nan"), 1.0), smooth_delta(1.0, 1.0),
            {"op": "reset"}, smooth_delta(float("inf"), 0.5),
            smooth_delta(float("-inf"), 1.0),
        ]),
    ]
    yield from smooth_double_cases

    def input_event_case(label, key, scancode, modifiers, mouse_button, mouse_x, mouse_y):
        return {
            "case_id": f"input-with-modifiers/{label}", "family": "math",
            "operation": "input_with_modifiers", "key": key, "scancode": scancode,
            "modifiers": modifiers, "mouse_button": mouse_button,
            "mouse_x": float_bits(mouse_x, "f64"), "mouse_y": float_bits(mouse_y, "f64"),
        }

    input_event_cases = [
        input_event_case("enter-selection", 257, 1, 0, 0, 1.25, -2.5),
        input_event_case("space-selection", 32, 2, 0, 1, 0.0, 0.0),
        input_event_case("keypad-enter-selection", 335, 3, 0, 2, -0.0, 3.5),
        input_event_case("escape", 256, 4, 0, 3, 4.5, -0.0),
        input_event_case("left", 263, 5, 0, 4, 5.5, 6.5),
        input_event_case("right", 262, 6, 0, 5, 7.5, 8.5),
        input_event_case("up", 265, 7, 0, 6, 9.5, 10.5),
        input_event_case("down", 264, 8, 0, 7, 11.5, 12.5),
        input_event_case("cycle-focus", 258, 9, 0, 0, 13.5, 14.5),
        input_event_case("digit-zero", 48, 10, 0, 1, 15.5, 16.5),
        input_event_case("digit-nine", 57, 11, 0, 2, 17.5, 18.5),
        input_event_case("non-digit", 58, 12, 0, 3, 19.5, 20.5),
        input_event_case("select-all", 65, 13, 2, 4, 21.5, 22.5),
        input_event_case("copy", 67, 14, 2, 5, 23.5, 24.5),
        input_event_case("paste", 86, 15, 2, 6, 25.5, 26.5),
        input_event_case("cut", 88, 16, 2, 7, 27.5, 28.5),
        input_event_case("shortcut-with-shift", 65, 17, 3, 0, 29.5, 30.5),
        input_event_case("modifier-bits", 0, 18, 15, 1, 31.5, 32.5),
        input_event_case("super-only", 65, 19, 8, 2, 33.5, 34.5),
        input_event_case("integer-underflow", -2147483648, 20, 0, 3, 35.5, 36.5),
    ]
    yield from input_event_cases

    def key_spec(kind, value):
        return {"type": kind, "value": value}

    def key_mapping_case(label, primary, secondary, other, rebind, actions,
                         key_event, unknown_key_event, mouse_button):
        return {
            "case_id": f"key-mapping/state/{label}", "family": "math",
            "operation": "key_mapping", "primary": primary, "secondary": secondary,
            "other": other, "rebind": rebind, "actions": actions,
            "key_event": key_event, "unknown_key_event": unknown_key_event,
            "mouse_button": mouse_button,
        }

    key_mapping_cases = []
    base = 1000
    key_mapping_cases.append(key_mapping_case(
        "shared-press-release",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("keysym", base + 1),
        key_spec("scancode", base + 2),
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "set", "key": "primary", "value": False},
         {"op": "click", "key": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "secondary"},
         {"op": "set_down", "target": "other", "value": True},
         {"op": "release_all"},
         {"op": "consume_click", "target": "secondary"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 20))
    base += 100
    key_mapping_cases.append(key_mapping_case(
        "repeated-clicks",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 1),
        key_spec("scancode", base + 2),
        [{"op": "click", "key": "primary"},
         {"op": "click", "key": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "secondary"},
         {"op": "consume_click", "target": "secondary"},
         {"op": "consume_click", "target": "secondary"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 1))
    base += 100
    key_mapping_cases.append(key_mapping_case(
        "rebind-keeps-old-table",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 10),
        key_spec("scancode", base + 20),
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "set_key", "target": "primary", "key": "rebind"},
         {"op": "set", "key": "primary_default", "value": False},
         {"op": "set", "key": "primary", "value": True},
         {"op": "click", "key": "primary"},
         {"op": "click", "key": "primary_default"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "secondary"},
         {"op": "release_all"},
         {"op": "set_key", "target": "primary", "key": "rebind"}],
        {"key": base, "scancode": base + 20},
        {"key": -1, "scancode": base + 20}, base + 10))
    base += 100
    key_mapping_cases.append(key_mapping_case(
        "default-and-unbound",
        key_spec("keysym", -1), key_spec("keysym", -1), key_spec("mouse", base),
        key_spec("keysym", base + 1),
        [{"op": "set_key", "target": "primary", "key": "rebind"}],
        {"key": -1, "scancode": base + 2},
        {"key": -1, "scancode": base + 2}, base))
    base += 100
    key_mapping_cases.append(key_mapping_case(
        "event-matching",
        key_spec("keysym", base), key_spec("scancode", base + 1), key_spec("mouse", base + 2),
        key_spec("keysym", base + 3), [],
        {"key": base, "scancode": base + 1},
        {"key": -1, "scancode": base + 1}, base + 2))
    key_mapping_cases.append(key_mapping_case(
        "known-key-names",
        key_spec("keysym", 87), key_spec("keysym", 65), key_spec("mouse", 0),
        key_spec("keysym", 32), [],
        {"key": 87, "scancode": 65}, {"key": -1, "scancode": 65}, 0))

    def toggle_key_mapping_case(label, primary, secondary, other, settings, actions,
                                key_event, unknown_key_event, mouse_button):
        return {
            "case_id": f"toggle-key-mapping/state/{label}", "family": "math",
            "operation": "toggle_key_mapping", "primary": primary, "secondary": secondary,
            "other": other, **settings, "actions": actions,
            "key_event": key_event, "unknown_key_event": unknown_key_event,
            "mouse_button": mouse_button,
        }

    toggle_key_mapping_cases = []
    base = 1500
    toggle_key_mapping_cases.append(toggle_key_mapping_case(
        "toggle-and-restore",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 1),
        {"primary_needs_toggle": True, "primary_should_restore": True,
         "secondary_needs_toggle": True, "secondary_should_restore": False,
         "other_needs_toggle": False, "other_should_restore": True},
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "set_down", "target": "primary", "value": True},
         {"op": "set", "key": "primary", "value": True},
         {"op": "click", "key": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "release_all"},
         {"op": "restore_toggle_states"},
         {"op": "should_restore", "target": "primary"},
         {"op": "reset_toggle_keys"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 1))
    base += 100
    toggle_key_mapping_cases.append(toggle_key_mapping_case(
        "scancode-does-not-restore",
        key_spec("scancode", base), key_spec("scancode", base), key_spec("keysym", base + 1),
        {"primary_needs_toggle": True, "primary_should_restore": True,
         "secondary_needs_toggle": True, "secondary_should_restore": True,
         "other_needs_toggle": True, "other_should_restore": True},
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "release_all"},
         {"op": "restore_toggle_states"},
         {"op": "should_restore", "target": "primary"},
         {"op": "reset_toggle_keys"}],
        {"key": -1, "scancode": base},
        {"key": -1, "scancode": base}, base + 1))
    base += 100
    toggle_key_mapping_cases.append(toggle_key_mapping_case(
        "direct-set-mode",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 1),
        {"primary_needs_toggle": False, "primary_should_restore": True,
         "secondary_needs_toggle": False, "secondary_should_restore": True,
         "other_needs_toggle": False, "other_should_restore": True},
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "set_down", "target": "primary", "value": False},
         {"op": "set", "key": "primary", "value": True},
         {"op": "release_all"},
         {"op": "restore_toggle_states"},
         {"op": "should_restore", "target": "primary"},
         {"op": "reset_toggle_keys"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 1))
    base += 100
    toggle_key_mapping_cases.append(toggle_key_mapping_case(
        "toggle-without-restore",
        key_spec("keysym", base), key_spec("keysym", base + 1), key_spec("mouse", base + 2),
        {"primary_needs_toggle": True, "primary_should_restore": False,
         "secondary_needs_toggle": True, "secondary_should_restore": False,
         "other_needs_toggle": True, "other_should_restore": False},
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "release_all"},
         {"op": "restore_toggle_states"},
         {"op": "should_restore", "target": "primary"},
         {"op": "reset_toggle_keys"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 2))
    yield from key_mapping_cases
    yield from toggle_key_mapping_cases

    keyboard_inputs = {
        "empty": {},
        "forward": {"up": True},
        "backward": {"down": True},
        "left": {"left": True},
        "right": {"right": True},
        "forward-left": {"up": True, "left": True},
        "forward-right": {"up": True, "right": True},
        "backward-left": {"down": True, "left": True},
        "backward-right": {"down": True, "right": True},
        "forward-backward": {"up": True, "down": True},
        "left-right": {"left": True, "right": True},
        "all-movement": {"up": True, "down": True, "left": True, "right": True},
        "jump": {"jump": True},
        "shift": {"shift": True},
        "sprint": {"sprint": True},
        "movement-and-flags": {
            "up": True, "right": True, "jump": True, "shift": True, "sprint": True,
        },
    }
    keyboard_defaults = {
        "up": False, "down": False, "left": False, "right": False,
        "jump": False, "shift": False, "sprint": False,
    }
    for label, values in keyboard_inputs.items():
        keys = {**keyboard_defaults, **values}
        yield {"case_id": f"keyboard-input/tick/{label}", "family": "math",
               "operation": "keyboard_input", **keys}

    local_player_input_cases = [
        {
            "label": "zero-preserves-signed-zero",
            "x": "80000000", "y": "00000000",
        },
        {
            "label": "axis-applies-base-factor",
            "x": float_bits(0.0), "y": float_bits(1.0),
        },
        {
            "label": "diagonal-clamps-to-unit-square",
            "x": float_bits(1.0), "y": float_bits(1.0),
        },
        {
            "label": "short-vector-keeps-scaled-length",
            "x": float_bits(0.25), "y": float_bits(-0.5),
        },
        {
            "label": "item-use-multiplier",
            "x": float_bits(1.0), "y": float_bits(0.0),
            "using_item": True, "item_use_speed_multiplier": float_bits(0.2),
        },
        {
            "label": "item-use-bypassed-in-passenger",
            "x": float_bits(1.0), "y": float_bits(0.0),
            "using_item": True, "passenger": True,
            "item_use_speed_multiplier": float_bits(0.2),
        },
        {
            "label": "sneaking-attribute-factor",
            "x": float_bits(0.0), "y": float_bits(1.0),
            "moving_slowly": True, "sneaking_movement_factor": float_bits(0.3),
        },
        {
            "label": "item-and-sneaking-order",
            "x": float_bits(1.0), "y": float_bits(1.0),
            "using_item": True, "item_use_speed_multiplier": float_bits(0.5),
            "moving_slowly": True, "sneaking_movement_factor": float_bits(0.3),
        },
        {
            "label": "zero-item-factor-reaches-square-zero",
            "x": float_bits(0.25), "y": float_bits(-0.5),
            "using_item": True, "item_use_speed_multiplier": float_bits(0.0),
        },
        {
            "label": "non-unit-vector-clamps",
            "x": float_bits(2.0), "y": float_bits(1.0),
        },
    ]
    local_player_input_defaults = {
        "using_item": False, "passenger": False, "moving_slowly": False,
        "item_use_speed_multiplier": float_bits(0.2),
        "sneaking_movement_factor": float_bits(0.3),
    }
    local_player_input_inputs = []
    for values in local_player_input_cases:
        input_values = {**local_player_input_defaults, **values}
        local_player_input_inputs.append(input_values)
    yield {"case_id": "player-input/modify", "family": "math",
           "operation": "local_player_input", "inputs": local_player_input_inputs}

    local_player_apply_input_cases = [
        {
            "label": "controlled-axis-updates-impulses-and-bob",
            "move_x": float_bits(0.0), "move_y": float_bits(1.0),
            "jump": False, "xxa": float_bits(0.25), "zza": float_bits(-0.5),
            "jumping": True, "x_bob": float_bits(1.0), "y_bob": float_bits(2.0),
            "x_bob_o": float_bits(3.0), "y_bob_o": float_bits(4.0),
            "x_rot": float_bits(10.0), "y_rot": float_bits(20.0),
            "controlled_camera": True,
        },
        {
            "label": "controlled-diagonal-sets-jumping",
            "move_x": float_bits(0.6), "move_y": float_bits(0.8),
            "jump": True, "xxa": float_bits(-1.0), "zza": float_bits(1.0),
            "jumping": False, "x_bob": float_bits(-2.0), "y_bob": float_bits(4.0),
            "x_bob_o": float_bits(-3.0), "y_bob_o": float_bits(5.0),
            "x_rot": float_bits(-30.0), "y_rot": float_bits(60.0),
            "controlled_camera": True,
        },
        {
            "label": "controlled-zero-preserves-signed-zero",
            "move_x": "80000000", "move_y": "00000000",
            "jump": True, "xxa": float_bits(1.0), "zza": float_bits(-1.0),
            "jumping": False, "x_bob": "80000000", "y_bob": "00000000",
            "x_bob_o": "80000000", "y_bob_o": "00000000",
            "x_rot": float_bits(0.0), "y_rot": float_bits(-0.0),
            "controlled_camera": True,
        },
        {
            "label": "uncontrolled-camera-uses-base-input",
            "move_x": float_bits(0.0), "move_y": float_bits(1.0),
            "jump": True, "xxa": float_bits(0.5), "zza": float_bits(-0.25),
            "jumping": True, "x_bob": float_bits(1.0), "y_bob": float_bits(2.0),
            "x_bob_o": float_bits(3.0), "y_bob_o": float_bits(4.0),
            "x_rot": float_bits(10.0), "y_rot": float_bits(20.0),
            "controlled_camera": False,
        },
        {
            "label": "controlled-overwrites-existing-impulses",
            "move_x": float_bits(-1.0), "move_y": float_bits(0.0),
            "jump": False, "xxa": float_bits(2.0), "zza": float_bits(3.0),
            "jumping": True, "x_bob": float_bits(0.0), "y_bob": float_bits(0.0),
            "x_bob_o": float_bits(1.0), "y_bob_o": float_bits(1.0),
            "x_rot": float_bits(0.0), "y_rot": float_bits(0.0),
            "controlled_camera": True,
        },
    ]
    local_player_apply_input_defaults = {
        "move_x": float_bits(0.0), "move_y": float_bits(0.0),
        "jump": False, "xxa": float_bits(0.0), "zza": float_bits(0.0),
        "jumping": False, "x_bob": float_bits(0.0), "y_bob": float_bits(0.0),
        "x_bob_o": float_bits(0.0), "y_bob_o": float_bits(0.0),
        "x_rot": float_bits(0.0), "y_rot": float_bits(0.0),
        "controlled_camera": True,
    }
    local_player_apply_input_inputs = []
    for values in local_player_apply_input_cases:
        input_values = {**local_player_apply_input_defaults, **values}
        local_player_apply_input_inputs.append(input_values)
    yield {"case_id": "player-input/apply", "family": "math",
           "operation": "local_player_apply_input", "inputs": local_player_apply_input_inputs}

    local_player_sprint_gate_cases = [
        {
            "label": "eligible-ground-forward",
            "move_y": float_bits(1.0),
        },
        {
            "label": "already-sprinting-blocks-start",
            "sprinting": True,
        },
        {
            "label": "forward-threshold-rejects-zero",
            "move_y": float_bits(0.0),
        },
        {
            "label": "forward-threshold-rejects-small",
            "move_y": float_bits(0.00001),
        },
        {
            "label": "mobility-restricted-blocks-start",
            "mobility_restricted": True,
        },
        {
            "label": "insufficient-food-blocks-start",
            "enough_food": False,
        },
        {
            "label": "mayfly-allows-insufficient-food",
            "enough_food": False,
            "mayfly": True,
        },
        {
            "label": "shallow-water-blocks-ground-start",
            "shallow_water": True,
        },
        {
            "label": "flying-allows-shallow-water",
            "shallow_water": True,
            "flying": True,
        },
        {
            "label": "item-use-blocks-sprint",
            "using_item": True,
        },
        {
            "label": "fall-flying-blocks-out-of-water",
            "fall_flying": True,
        },
        {
            "label": "fall-flying-allows-underwater",
            "fall_flying": True,
            "underwater": True,
        },
        {
            "label": "moving-slowly-blocks-out-of-water",
            "moving_slowly": True,
        },
        {
            "label": "moving-slowly-allows-underwater",
            "moving_slowly": True,
            "underwater": True,
        },
    ]
    local_player_sprint_gate_defaults = {
        "move_y": float_bits(1.0),
        "sprinting": False,
        "mobility_restricted": False,
        "enough_food": True,
        "mayfly": False,
        "shallow_water": False,
        "flying": False,
        "using_item": False,
        "fall_flying": False,
        "underwater": False,
        "moving_slowly": False,
    }
    local_player_sprint_gate_inputs = []
    for values in local_player_sprint_gate_cases:
        input_values = {**local_player_sprint_gate_defaults, **values}
        local_player_sprint_gate_inputs.append(input_values)
    yield {"case_id": "player-sprint/can-start", "family": "math",
           "operation": "local_player_can_start_sprinting", "inputs": local_player_sprint_gate_inputs}

    local_player_fluid_adjustment_cases = [
        {
            "label": "zero-gravity-preserves-movement",
            "base_gravity": float_bits(0.0, "f64"),
            "is_falling": True,
            "x": float_bits(-1.25, "f64"),
            "y": float_bits(-0.005, "f64"),
            "z": float_bits(2.5, "f64"),
        },
        {
            "label": "sprinting-bypasses-gravity",
            "sprinting": True,
            "y": float_bits(-0.2, "f64"),
        },
        {
            "label": "falling-near-custom-gravity-uses-clamp",
            "base_gravity": float_bits(0.16, "f64"),
            "is_falling": True,
            "y": float_bits(0.01, "f64"),
        },
        {
            "label": "falling-outside-custom-gravity-window-uses-subtraction",
            "base_gravity": float_bits(0.16, "f64"),
            "is_falling": True,
            "y": float_bits(0.014, "f64"),
        },
        {
            "label": "rising-uses-gravity-subtraction",
            "is_falling": False,
            "y": float_bits(0.3, "f64"),
        },
        {
            "label": "default-gravity-exact-zero",
            "is_falling": True,
            "y": float_bits(0.005, "f64"),
        },
        {
            "label": "negative-zero-gravity-preserves-signed-zero",
            "base_gravity": "8000000000000000",
            "is_falling": True,
            "x": "8000000000000000",
            "y": "8000000000000000",
            "z": "0000000000000000",
        },
        {
            "label": "falling-custom-gravity-ordinary-subtraction",
            "base_gravity": float_bits(0.16, "f64"),
            "is_falling": True,
            "y": float_bits(-0.2, "f64"),
        },
    ]
    local_player_fluid_adjustment_defaults = {
        "base_gravity": float_bits(0.08, "f64"),
        "is_falling": False,
        "sprinting": False,
        "x": float_bits(0.125, "f64"),
        "y": float_bits(-0.05, "f64"),
        "z": float_bits(-0.25, "f64"),
    }
    local_player_fluid_adjustment_inputs = []
    for values in local_player_fluid_adjustment_cases:
        input_values = {**local_player_fluid_adjustment_defaults, **values}
        local_player_fluid_adjustment_inputs.append(input_values)
    yield {"case_id": "player-movement/fluid-falling-adjustment", "family": "math",
           "operation": "local_player_fluid_falling_adjustment",
           "inputs": local_player_fluid_adjustment_inputs}

    local_player_climbable_movement_cases = [
        {
            "label": "not-climbable-preserves-movement",
            "on_climbable": False,
        },
        {
            "label": "climbable-clamps-horizontal-and-downward",
            "on_climbable": True,
        },
        {
            "label": "suppressing-player-stops-ladder-slide",
            "on_climbable": True,
            "suppress_sliding_down_ladder": True,
            "x": float_bits(0.1, "f64"),
            "y": float_bits(-0.1, "f64"),
            "z": float_bits(-0.1, "f64"),
        },
        {
            "label": "scaffolding-keeps-clamped-downward-motion",
            "on_climbable": True,
            "scaffolding": True,
            "suppress_sliding_down_ladder": True,
        },
        {
            "label": "suppressing-player-keeps-upward-motion",
            "on_climbable": True,
            "suppress_sliding_down_ladder": True,
            "x": float_bits(-0.1, "f64"),
            "y": float_bits(0.25, "f64"),
            "z": float_bits(0.1, "f64"),
        },
        {
            "label": "climbable-preserves-signed-zero",
            "on_climbable": True,
            "x": "8000000000000000",
            "y": "8000000000000000",
            "z": "0000000000000000",
        },
    ]
    local_player_climbable_movement_defaults = {
        "on_climbable": True,
        "scaffolding": False,
        "suppress_sliding_down_ladder": False,
        "x": float_bits(0.4, "f64"),
        "y": float_bits(-0.4, "f64"),
        "z": float_bits(-0.4, "f64"),
    }
    local_player_climbable_movement_inputs = []
    for values in local_player_climbable_movement_cases:
        input_values = {**local_player_climbable_movement_defaults, **values}
        local_player_climbable_movement_inputs.append(input_values)
    yield {"case_id": "player-movement/climbable-movement", "family": "math",
           "operation": "local_player_climbable_movement",
           "inputs": local_player_climbable_movement_inputs}

    local_player_friction_influenced_speed_cases = [
        {
            "label": "ground-at-friction-threshold-keeps-speed",
            "on_ground": True,
            "block_friction": float_bits(0.6, "f32"),
        },
        {
            "label": "ground-below-friction-threshold-keeps-speed",
            "on_ground": True,
            "block_friction": float_bits(0.5, "f32"),
        },
        {
            "label": "ground-above-friction-threshold-scales-speed",
            "on_ground": True,
            "block_friction": float_bits(0.8, "f32"),
        },
        {
            "label": "ground-just-above-friction-threshold-scales-speed",
            "on_ground": True,
            "block_friction": float_bits(0.6000001, "f32"),
            "speed": float_bits(0.125, "f32"),
        },
        {
            "label": "air-without-controlling-player-uses-default-flying-speed",
            "on_ground": False,
            "flying": False,
            "sprinting": False,
            "passenger": False,
        },
        {
            "label": "air-sprinting-without-flight-uses-sprint-speed",
            "on_ground": False,
            "flying": False,
            "sprinting": True,
            "passenger": False,
        },
        {
            "label": "air-flying-uses-ability-speed",
            "on_ground": False,
            "flying": True,
            "flying_speed": float_bits(0.07, "f32"),
            "sprinting": False,
            "passenger": False,
        },
        {
            "label": "air-flying-sprint-doubles-ability-speed",
            "on_ground": False,
            "flying": True,
            "flying_speed": float_bits(0.07, "f32"),
            "sprinting": True,
            "passenger": False,
        },
        {
            "label": "air-flying-passenger-uses-grounded-fallback",
            "on_ground": False,
            "flying": True,
            "sprinting": True,
            "passenger": True,
        },
        {
            "label": "ground-scaled-speed-preserves-negative-zero",
            "on_ground": True,
            "block_friction": float_bits(0.8, "f32"),
            "speed": "80000000",
        },
        {
            "label": "nan-friction-fails-strict-comparison-and-keeps-speed",
            "on_ground": True,
            "block_friction": "7fc00001",
            "speed": float_bits(0.25, "f32"),
        },
    ]
    local_player_friction_influenced_speed_defaults = {
        "on_ground": True,
        "block_friction": float_bits(0.6, "f32"),
        "speed": float_bits(0.1, "f32"),
        "flying": False,
        "flying_speed": float_bits(0.05, "f32"),
        "sprinting": False,
        "passenger": False,
    }
    local_player_friction_influenced_speed_inputs = []
    for values in local_player_friction_influenced_speed_cases:
        input_values = {**local_player_friction_influenced_speed_defaults, **values}
        local_player_friction_influenced_speed_inputs.append(input_values)
    yield {"case_id": "player-movement/friction-influenced-speed", "family": "math",
           "operation": "local_player_friction_influenced_speed",
           "inputs": local_player_friction_influenced_speed_inputs}

    local_player_modified_friction_cases = [
        {
            "label": "unit-modifier-keeps-block-friction",
            "friction": float_bits(0.8, "f32"),
            "modifier": float_bits(1.0, "f32"),
        },
        {
            "label": "zero-modifier-reaches-one",
            "friction": float_bits(0.8, "f32"),
            "modifier": float_bits(0.0, "f32"),
        },
        {
            "label": "double-modifier-reduces-friction",
            "friction": float_bits(0.8, "f32"),
            "modifier": float_bits(2.0, "f32"),
        },
        {
            "label": "negative-modifier-clamps-high",
            "friction": float_bits(0.8, "f32"),
            "modifier": float_bits(-1.0, "f32"),
        },
        {
            "label": "large-modifier-clamps-low",
            "friction": float_bits(0.2, "f32"),
            "modifier": float_bits(2.0, "f32"),
        },
        {
            "label": "zero-friction-with-unit-modifier",
            "friction": float_bits(0.0, "f32"),
            "modifier": float_bits(1.0, "f32"),
        },
        {
            "label": "full-friction-ignores-modifier",
            "friction": float_bits(1.0, "f32"),
            "modifier": float_bits(3.0, "f32"),
        },
        {
            "label": "nan-friction-propagates-through-clamp",
            "friction": "7fc01234",
            "modifier": float_bits(1.0, "f32"),
        },
    ]
    local_player_modified_friction_defaults = {
        "friction": float_bits(0.6, "f32"),
        "modifier": float_bits(1.0, "f32"),
    }
    local_player_modified_friction_inputs = []
    for values in local_player_modified_friction_cases:
        input_values = {**local_player_modified_friction_defaults, **values}
        local_player_modified_friction_inputs.append(input_values)
    yield {"case_id": "player-movement/modified-friction", "family": "math",
           "operation": "local_player_modified_friction",
           "inputs": local_player_modified_friction_inputs}

    local_player_travel_flying_cases = [
        {
            "label": "air-branch-rotates-and-drags",
        },
        {
            "label": "water-branch-uses-water-speed",
            "in_water": True,
            "water_speed": float_bits(0.04, "f32"),
        },
        {
            "label": "lava-branch-uses-lava-speed",
            "in_lava": True,
            "lava_speed": float_bits(0.06, "f32"),
        },
        {
            "label": "water-wins-over-lava",
            "in_water": True,
            "in_lava": True,
            "water_speed": float_bits(0.04, "f32"),
            "lava_speed": float_bits(0.06, "f32"),
        },
        {
            "label": "long-input-normalizes-before-speed",
            "input": vector_bits(1.0, 2.0, 2.0),
            "delta": vector_bits(-0.25, 0.125, 0.5),
            "y_rot": float_bits(-120.0, "f32"),
            "air_speed": float_bits(0.07, "f32"),
        },
        {
            "label": "subthreshold-input-skips-relative-movement",
            "input": vector_bits(0.0001, 0.0, 0.0),
            "delta": vector_bits(0.1, -0.2, 0.3),
        },
        {
            "label": "zero-input-preserves-signed-motion",
            "input": vector_bits(-0.0, 0.0, -0.0),
            "delta": vector_bits(-0.0, 0.125, -0.25),
            "y_rot": float_bits(90.0, "f32"),
        },
        {
            "label": "nonfinite-relative-delta-is-rejected",
            "input": {"x": "7ff8000000000001", "y": float_bits(0.0, "f64"),
                      "z": float_bits(0.0, "f64")},
            "delta": vector_bits(0.1, -0.2, 0.3),
        },
    ]
    local_player_travel_flying_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "in_water": False,
        "in_lava": False,
        "water_speed": float_bits(0.02, "f32"),
        "lava_speed": float_bits(0.03, "f32"),
        "air_speed": float_bits(0.04, "f32"),
    }
    local_player_travel_flying_inputs = []
    for values in local_player_travel_flying_cases:
        input_values = {**local_player_travel_flying_defaults, **values}
        local_player_travel_flying_inputs.append(input_values)
    yield {"case_id": "player-movement/travel-flying", "family": "math",
           "operation": "local_player_travel_flying",
           "inputs": local_player_travel_flying_inputs}

    local_player_travel_in_water_cases = [
        {
            "label": "plain-water-applies-drag-and-fluid-gravity",
        },
        {
            "label": "sprinting-uses-sprint-water-slowdown-and-bypasses-gravity",
            "sprinting": True,
        },
        {
            "label": "ground-water-efficiency-blends-speed-and-drag",
            "water_efficiency": float_bits(0.6, "f32"),
            "speed": float_bits(0.12, "f32"),
        },
        {
            "label": "airborne-water-efficiency-is-halved",
            "water_efficiency": float_bits(0.6, "f32"),
            "speed": float_bits(0.12, "f32"),
            "on_ground": False,
        },
        {
            "label": "dolphins-grace-overrides-computed-drag",
            "water_efficiency": float_bits(0.6, "f32"),
            "speed": float_bits(0.12, "f32"),
            "sprinting": True,
            "dolphins_grace": True,
        },
        {
            "label": "horizontal-climbable-sets-fluid-step",
            "horizontal_collision": True,
            "on_climbable": True,
            "base_gravity": float_bits(0.08, "f64"),
            "is_falling": True,
            "free_after_jump": False,
        },
        {
            "label": "free-horizontal-fluid-collision-jumps-out",
            "horizontal_collision": True,
            "base_gravity": float_bits(0.08, "f64"),
            "is_falling": True,
            "free_after_jump": True,
        },
        {
            "label": "negative-water-efficiency-is-ignored",
            "water_efficiency": float_bits(-0.5, "f32"),
            "speed": float_bits(0.12, "f32"),
        },
    ]
    local_player_travel_in_water_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "sprinting": False,
        "water_slow_down": float_bits(0.8, "f32"),
        "water_efficiency": float_bits(0.0, "f32"),
        "on_ground": True,
        "speed": float_bits(0.1, "f32"),
        "dolphins_grace": False,
        "horizontal_collision": False,
        "on_climbable": False,
        "free_after_jump": False,
        "base_gravity": float_bits(0.08, "f64"),
        "is_falling": False,
        "old_y": float_bits(10.0, "f64"),
        "current_y": float_bits(10.0, "f64"),
    }
    local_player_travel_in_water_inputs = []
    for values in local_player_travel_in_water_cases:
        input_values = {**local_player_travel_in_water_defaults, **values}
        local_player_travel_in_water_inputs.append(input_values)
    yield {"case_id": "player-movement/travel-in-water", "family": "math",
           "operation": "local_player_travel_in_water",
           "inputs": local_player_travel_in_water_inputs}

    local_player_travel_in_lava_cases = [
        {
            "label": "shallow-lava-applies-fluid-drag-falling-and-gravity",
        },
        {
            "label": "deep-lava-uses-half-scale-before-gravity",
            "shallow_lava": False,
        },
        {
            "label": "zero-gravity-skips-lava-gravity-step",
            "base_gravity": float_bits(0.0, "f64"),
        },
        {
            "label": "sprinting-bypasses-falling-adjustment-but-keeps-gravity",
            "sprinting": True,
        },
        {
            "label": "horizontal-fluid-collision-without-free-space-keeps-motion",
            "horizontal_collision": True,
            "free_after_jump": False,
        },
        {
            "label": "horizontal-fluid-collision-with-free-space-jumps-out",
            "horizontal_collision": True,
            "free_after_jump": True,
        },
    ]
    local_player_travel_in_lava_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "sprinting": False,
        "shallow_lava": True,
        "horizontal_collision": False,
        "free_after_jump": False,
        "base_gravity": float_bits(0.08, "f64"),
        "is_falling": False,
        "old_y": float_bits(10.0, "f64"),
        "current_y": float_bits(10.0, "f64"),
    }
    local_player_travel_in_lava_inputs = []
    for values in local_player_travel_in_lava_cases:
        input_values = {**local_player_travel_in_lava_defaults, **values}
        local_player_travel_in_lava_inputs.append(input_values)
    yield {"case_id": "player-movement/travel-in-lava", "family": "math",
           "operation": "local_player_travel_in_lava",
           "inputs": local_player_travel_in_lava_inputs}

    local_player_handle_relative_friction_cases = [
        {
            "label": "air-relative-movement-uses-player-flying-speed",
        },
        {
            "label": "ground-friction-speed-enters-relative-movement",
            "on_ground": True,
            "block_friction": float_bits(0.8, "f32"),
            "speed": float_bits(0.12, "f32"),
        },
        {
            "label": "climbable-clamps-delta-before-return",
            "input": vector_bits(0.0, 0.0, 0.0),
            "delta": vector_bits(0.4, -0.4, -0.4),
            "on_climbable": True,
        },
        {
            "label": "horizontal-climbable-collision-sets-step-height",
            "on_climbable": True,
            "horizontal_collision": True,
        },
        {
            "label": "jumping-climbable-sets-step-height",
            "on_climbable": True,
            "jumping": True,
        },
        {
            "label": "jumping-without-climbable-keeps-motion",
            "jumping": True,
        },
    ]
    local_player_handle_relative_friction_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "on_ground": False,
        "block_friction": float_bits(0.6, "f32"),
        "speed": float_bits(0.1, "f32"),
        "flying_speed": float_bits(0.05, "f32"),
        "flying": False,
        "sprinting": False,
        "passenger": False,
        "on_climbable": False,
        "scaffolding": False,
        "suppress_sliding_down_ladder": False,
        "horizontal_collision": False,
        "jumping": False,
    }
    local_player_handle_relative_friction_inputs = []
    for values in local_player_handle_relative_friction_cases:
        input_values = {**local_player_handle_relative_friction_defaults, **values}
        local_player_handle_relative_friction_inputs.append(input_values)
    yield {"case_id": "player-movement/handle-relative-friction", "family": "math",
           "operation": "local_player_handle_relative_friction",
           "inputs": local_player_handle_relative_friction_inputs}

    local_player_travel_in_air_cases = [
        {
            "label": "air-travel-applies-server-gravity-and-drag",
        },
        {
            "label": "ground-travel-uses-below-block-friction",
            "on_ground": True,
            "below_block": "ice",
            "block_friction": float_bits(0.98, "f32"),
            "speed": float_bits(0.12, "f32"),
        },
        {
            "label": "levitation-replaces-gravity-with-effect-target",
            "levitation_amplifier": 2,
        },
        {
            "label": "discard-friction-keeps-relative-horizontal-motion",
            "should_discard_friction": True,
            "air_drag_modifier": float_bits(0.2, "f32"),
        },
        {
            "label": "omnidirectional-air-mover-uses-horizontal-drag-vertically",
            "omnidirectional_air_mover": True,
            "air_drag_modifier": float_bits(0.2, "f32"),
        },
        {
            "label": "air-drag-attribute-modifies-horizontal-and-vertical",
            "air_drag_modifier": float_bits(0.2, "f32"),
        },
        {
            "label": "client-unloaded-above-min-y-uses-fallback-vertical-speed",
            "client_side": True,
            "has_chunk_at": False,
        },
        {
            "label": "client-unloaded-at-min-y-zeroes-vertical-speed",
            "client_side": True,
            "has_chunk_at": False,
            "current_y": float_bits(-64.0, "f64"),
        },
    ]
    local_player_travel_in_air_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "on_ground": False,
        "below_block": "air",
        "block_friction": float_bits(0.6, "f32"),
        "friction_modifier": float_bits(1.0, "f32"),
        "speed": float_bits(0.1, "f32"),
        "flying_speed": float_bits(0.05, "f32"),
        "flying": False,
        "sprinting": False,
        "passenger": False,
        "on_climbable": False,
        "scaffolding": False,
        "suppress_sliding_down_ladder": False,
        "horizontal_collision": False,
        "jumping": False,
        "levitation_amplifier": -1,
        "gravity": float_bits(0.08, "f64"),
        "should_discard_friction": False,
        "air_drag_modifier": float_bits(0.0, "f32"),
        "omnidirectional_air_mover": False,
        "client_side": False,
        "has_chunk_at": True,
        "min_y": -64,
        "current_y": float_bits(10.0, "f64"),
    }
    local_player_travel_in_air_inputs = []
    for values in local_player_travel_in_air_cases:
        input_values = {**local_player_travel_in_air_defaults, **values}
        local_player_travel_in_air_inputs.append(input_values)
    yield {"case_id": "player-movement/travel-in-air", "family": "math",
           "operation": "local_player_travel_in_air",
           "inputs": local_player_travel_in_air_inputs}

    local_player_update_fall_flying_cases = [
        {
            "label": "neutral-glide-applies-lift-and-drag",
        },
        {
            "label": "nose-down-converts-descending-speed",
            "look": vector_bits(0.0, -0.5, 0.8660254037844386),
            "x_rot": float_bits(30.0, "f32"),
            "delta": vector_bits(0.2, -0.4, 0.3),
        },
        {
            "label": "nose-up-adds-lift",
            "look": vector_bits(0.0, 0.5, 0.8660254037844386),
            "x_rot": float_bits(-30.0, "f32"),
            "delta": vector_bits(0.3, 0.1, 0.2),
        },
        {
            "label": "vertical-look-skips-horizontal-conversion",
            "look": vector_bits(0.0, 1.0, 0.0),
            "x_rot": float_bits(15.0, "f32"),
        },
        {
            "label": "zero-gravity-keeps-lift-free-motion",
            "gravity": float_bits(0.0, "f64"),
            "look": vector_bits(-0.6, 0.0, 0.8),
            "x_rot": float_bits(0.0, "f32"),
        },
        {
            "label": "negative-look-direction-aligns-horizontal-motion",
            "look": vector_bits(-0.6, 0.0, 0.8),
            "x_rot": float_bits(0.0, "f32"),
            "delta": vector_bits(0.35, -0.1, -0.25),
        },
    ]
    local_player_update_fall_flying_defaults = {
        "look": vector_bits(0.0, 0.0, 1.0),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "x_rot": float_bits(0.0, "f32"),
        "gravity": float_bits(0.08, "f64"),
    }
    local_player_update_fall_flying_inputs = []
    for values in local_player_update_fall_flying_cases:
        input_values = {**local_player_update_fall_flying_defaults, **values}
        local_player_update_fall_flying_inputs.append(input_values)
    yield {"case_id": "player-movement/update-fall-flying-movement", "family": "math",
           "operation": "local_player_update_fall_flying_movement",
           "inputs": local_player_update_fall_flying_inputs}

    local_player_travel_dispatch_cases = [
        {
            "label": "air-is-default-when-no-fluid-or-glide",
        },
        {
            "label": "water-fluid-branch",
            "in_water": True,
        },
        {
            "label": "lava-fluid-branch",
            "in_lava": True,
            "shallow_lava": False,
        },
        {
            "label": "water-wins-over-lava-in-fluid-branch",
            "in_water": True,
            "in_lava": True,
            "water_efficiency": float_bits(0.4, "f32"),
        },
        {
            "label": "unaffected-water-falls-through-to-air",
            "in_water": True,
            "affected_by_fluids": False,
        },
        {
            "label": "standing-on-lava-falls-through-to-air",
            "in_lava": True,
            "can_stand_on_fluid": True,
        },
        {
            "label": "fall-flying-uses-glide-branch",
            "fall_flying": True,
            "look": vector_bits(0.0, -0.35, 0.9367496997597597),
            "x_rot": float_bits(20.0, "f32"),
        },
        {
            "label": "fall-flying-on-climbable-falls-back-to-air",
            "fall_flying": True,
            "on_climbable": True,
            "on_ground": True,
        },
        {
            "label": "fluid-precedes-fall-flying",
            "in_water": True,
            "fall_flying": True,
        },
    ]
    local_player_travel_dispatch_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "x_rot": float_bits(0.0, "f32"),
        "look": vector_bits(0.0, 0.0, 1.0),
        "in_water": False,
        "in_lava": False,
        "affected_by_fluids": True,
        "can_stand_on_fluid": False,
        "fall_flying": False,
        "on_climbable": False,
        "sprinting": False,
        "water_slow_down": float_bits(0.8, "f32"),
        "water_efficiency": float_bits(0.0, "f32"),
        "on_ground": False,
        "block_friction": float_bits(0.6, "f32"),
        "speed": float_bits(0.1, "f32"),
        "dolphins_grace": False,
        "horizontal_collision": False,
        "free_after_jump": False,
        "shallow_lava": True,
        "flying_speed": float_bits(0.05, "f32"),
        "flying": False,
        "passenger": False,
        "scaffolding": False,
        "suppress_sliding_down_ladder": False,
        "jumping": False,
        "levitation_amplifier": -1,
        "gravity": float_bits(0.08, "f64"),
        "client_side": True,
        "has_chunk_at": True,
        "current_y": float_bits(10.0, "f64"),
        "min_y": -64,
        "should_discard_friction": False,
        "friction_modifier": float_bits(1.0, "f32"),
        "air_drag_modifier": float_bits(0.0, "f32"),
        "omnidirectional_air_mover": False,
        "below_block": "air",
    }
    local_player_travel_dispatch_inputs = []
    for values in local_player_travel_dispatch_cases:
        input_values = {**local_player_travel_dispatch_defaults, **values}
        local_player_travel_dispatch_inputs.append(input_values)
    yield {"case_id": "player-movement/travel-dispatch", "family": "math",
           "operation": "local_player_travel_dispatch",
           "inputs": local_player_travel_dispatch_inputs}

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

    chunkpos_codec_decodes = {
        "basic": ["1", "-2"],
        "zero": ["0", "0"],
        "negative": ["-1", "-2147483648"],
        "boundaries": ["2147483647", "-2147483648"],
        "alternating": ["1437226410", "-1437226411"],
        "empty": [],
        "short": ["1"],
        "long": ["1", "2", "3"],
    }
    for label, values in chunkpos_codec_decodes.items():
        yield {"case_id": f"chunkpos/codec/decode/{label}", "family": "math",
               "operation": "chunkpos_codec", "mode": "decode", "values": values}
    chunkpos_codec_encodes = {
        "basic": {"x": "1", "z": "-2"},
        "zero": {"x": "0", "z": "0"},
        "boundaries": {"x": "2147483647", "z": "-2147483648"},
        "alternating": {"x": "1437226410", "z": "-1437226411"},
    }
    for label, values in chunkpos_codec_encodes.items():
        yield {"case_id": f"chunkpos/codec/encode/{label}", "family": "math",
               "operation": "chunkpos_codec", "mode": "encode", **values}

    chunkpos_block_coordinates = {
        "origin": {"x": "0", "z": "0", "offset": "0"},
        "positive": {"x": "1", "z": "2", "offset": "7"},
        "negative": {"x": "-1", "z": "-2", "offset": "15"},
        "boundary": {"x": "134217727", "z": "-134217728", "offset": "15"},
        "shift-wrap": {"x": "134217728", "z": "-134217729", "offset": "-1"},
        "offset-wrap": {"x": "1", "z": "-2", "offset": "2147483647"},
    }
    for label, values in chunkpos_block_coordinates.items():
        yield {"case_id": f"chunkpos/block-coordinates/{label}", "family": "math",
               "operation": "chunkpos_block_coordinates", **values}

    chunkpos_region_coordinates = {
        "origin": {"x": "0", "z": "0", "region_x": "0", "region_z": "0"},
        "positive-boundary": {"x": "31", "z": "32", "region_x": "1", "region_z": "2"},
        "negative-boundary": {"x": "-1", "z": "-32", "region_x": "-1", "region_z": "-2"},
        "mixed": {"x": "-33", "z": "63", "region_x": "67108863", "region_z": "-67108864"},
        "shift-wrap": {"x": "134217728", "z": "-134217729", "region_x": "67108864", "region_z": "-67108865"},
        "coordinate-boundaries": {"x": "2147483647", "z": "-2147483648", "region_x": "-67108864", "region_z": "67108863"},
    }
    for label, values in chunkpos_region_coordinates.items():
        yield {"case_id": f"chunkpos/region-coordinates/{label}", "family": "math",
               "operation": "chunkpos_region_coordinates", **values}

    chunkpos_block_positions = {
        "origin": {
            "x": "0", "z": "0", "block_x": "0", "y": "0", "block_z": "0",
            "contains": [
                {"x": "0", "y": "-99", "z": "0"},
                {"x": "15", "y": "999", "z": "15"},
                {"x": "-1", "y": "0", "z": "0"},
                {"x": "16", "y": "0", "z": "0"},
            ],
        },
        "positive": {
            "x": "1", "z": "2", "block_x": "7", "y": "-64", "block_z": "12",
            "contains": [
                {"x": "16", "y": "0", "z": "32"},
                {"x": "31", "y": "100", "z": "47"},
                {"x": "15", "y": "0", "z": "32"},
                {"x": "16", "y": "0", "z": "48"},
            ],
        },
        "negative": {
            "x": "-1", "z": "-2", "block_x": "15", "y": "-1", "block_z": "0",
            "contains": [
                {"x": "-16", "y": "0", "z": "-32"},
                {"x": "-1", "y": "1", "z": "-17"},
                {"x": "-17", "y": "0", "z": "-20"},
                {"x": "-1", "y": "0", "z": "-16"},
            ],
        },
        "boundary": {
            "x": "134217727", "z": "-134217728", "block_x": "15", "y": "2147483647", "block_z": "0",
            "contains": [
                {"x": "2147483632", "y": "-2147483648", "z": "-2147483648"},
                {"x": "2147483647", "y": "0", "z": "-2147483633"},
                {"x": "2147483647", "y": "0", "z": "-2147483632"},
            ],
        },
        "offset-wrap": {
            "x": "1", "z": "-2", "block_x": "2147483647", "y": "-2147483648", "block_z": "-2147483648",
            "contains": [
                {"x": "16", "y": "0", "z": "-32"},
                {"x": "31", "y": "0", "z": "-17"},
                {"x": "-2147483633", "y": "0", "z": "-2147483648"},
            ],
        },
        "shift-wrap": {
            "x": "134217728", "z": "-134217729", "block_x": "-1", "y": "42", "block_z": "15",
            "contains": [
                {"x": "-2147483648", "y": "0", "z": "2147483632"},
                {"x": "-2147483633", "y": "0", "z": "2147483647"},
                {"x": "-2147483632", "y": "0", "z": "2147483632"},
            ],
        },
    }
    for label, values in chunkpos_block_positions.items():
        yield {"case_id": f"chunkpos/block-positions/{label}", "family": "math",
               "operation": "chunkpos_block_positions", **values}

    chunkpos_representation = {
        "origin": {"x": "0", "z": "0", "hash_x": "0", "hash_z": "0"},
        "basic": {"x": "1", "z": "-2", "hash_x": "1", "hash_z": "-2"},
        "negative": {"x": "-1", "z": "-2147483648", "hash_x": "-2147483648", "hash_z": "2147483647"},
        "boundaries": {"x": "2147483647", "z": "-2147483648", "hash_x": "2147483647", "hash_z": "-2147483648"},
        "alternating": {"x": "1437226410", "z": "-1437226411", "hash_x": "-559038737", "hash_z": "1013904223"},
        "hash-overflow": {"x": "2147483647", "z": "2147483647", "hash_x": "-2147483648", "hash_z": "-2147483648"},
    }
    for label, values in chunkpos_representation.items():
        yield {"case_id": f"chunkpos/representation/{label}", "family": "math",
               "operation": "chunkpos_representation", **values}

    chunkpos_from_blockpos = {
        "origin": {"x": "0", "y": "0", "z": "0"},
        "positive-boundary": {"x": "15", "y": "99", "z": "31"},
        "negative-boundary": {"x": "-1", "y": "-99", "z": "-16"},
        "mixed": {"x": "-17", "y": "42", "z": "16"},
        "max-boundary": {"x": "2147483647", "y": "2147483647", "z": "-2147483648"},
        "min-boundary": {"x": "-2147483648", "y": "-2147483648", "z": "2147483647"},
    }
    for label, values in chunkpos_from_blockpos.items():
        yield {"case_id": f"chunkpos/from-blockpos/{label}", "family": "math",
               "operation": "chunkpos_from_blockpos", **values}

    chunkpos_distances = {
        "origin": {"x": "0", "z": "0", "other_x": "0", "other_z": "0"},
        "basic": {"x": "1", "z": "-2", "other_x": "4", "other_z": "3"},
        "negative": {"x": "-32", "z": "-17", "other_x": "-1", "other_z": "-64"},
        "axis-dominant": {"x": "100", "z": "-200", "other_x": "-123", "other_z": "-150"},
        "difference-wrap": {"x": "2147483647", "z": "-2147483648", "other_x": "-2147483648", "other_z": "2147483647"},
        "signed-minimum": {"x": "0", "z": "0", "other_x": "-2147483648", "other_z": "-2147483648"},
    }
    for label, values in chunkpos_distances.items():
        yield {"case_id": f"chunkpos/distances/{label}", "family": "math",
               "operation": "chunkpos_distances", **values}

    chunkpos_from_section_node = {
        "origin": {"section_node": "0000000000000000"},
        "positive": {"section_node": "0000040000300002"},
        "negative": {"section_node": "ffffffffffffffff"},
        "minimums": {"section_node": "8000020000080000"},
        "maximums": {"section_node": "7ffffdfffff7ffff"},
        "y-discard": {"section_node": "0000abfffef7ffff"},
    }
    for label, values in chunkpos_from_section_node.items():
        yield {"case_id": f"chunkpos/from-section-node/{label}", "family": "math",
               "operation": "chunkpos_from_section_node", **values}

    chunkpos_range_closed = {
        "from-to-single": {
            "mode": "from-to", "from_x": "0", "from_z": "0", "to_x": "0", "to_z": "0",
        },
        "from-to-increasing": {
            "mode": "from-to", "from_x": "1", "from_z": "2", "to_x": "3", "to_z": "4",
        },
        "from-to-decreasing": {
            "mode": "from-to", "from_x": "3", "from_z": "4", "to_x": "1", "to_z": "2",
        },
        "from-to-fixed-x": {
            "mode": "from-to", "from_x": "-5", "from_z": "7", "to_x": "-5", "to_z": "4",
        },
        "from-to-fixed-z": {
            "mode": "from-to", "from_x": "-2", "from_z": "6", "to_x": "2", "to_z": "6",
        },
        "center-radius-zero": {
            "mode": "center-radius", "x": "9", "z": "-4", "radius": "0",
        },
        "center-radius-positive": {
            "mode": "center-radius", "x": "-2", "z": "3", "radius": "2",
        },
        "center-radius-negative": {
            "mode": "center-radius", "x": "4", "z": "-6", "radius": "-1",
        },
    }
    for label, values in chunkpos_range_closed.items():
        yield {"case_id": f"chunkpos/range-closed/{label}", "family": "math",
               "operation": "chunkpos_range_closed", **values}

    for name in ("zero", "invalid_chunk_pos", "region_size", "region_max_index"):
        yield {"case_id": f"chunkpos/constants/{name}", "family": "math",
               "operation": "chunkpos_constant", "name": name}

    chunkpos_packed_accessors = {
        "zero": "0000000000000000",
        "positive": "0000000200000001",
        "negative": "fffffffeffffffff",
        "x-min-z-max": "7fffffff80000000",
        "x-max-z-min": "800000007fffffff",
        "alternating": "aa55aa5555aa55aa",
    }
    for label, packed in chunkpos_packed_accessors.items():
        yield {"case_id": f"chunkpos/packed-accessors/{label}", "family": "math",
               "operation": "chunkpos_packed_accessors", "packed": packed}

    sectionpos_packed_accessors = {
        "zero": "0000000000000000",
        "positive": "0000040000300002",
        "negative": "ffffffffffffffff",
        "minimums": "8000020000080000",
        "maximums": "7ffffdfffff7ffff",
        "alternating": "aa55aa5555aa55aa",
    }
    for label, packed in sectionpos_packed_accessors.items():
        yield {"case_id": f"sectionpos/packed-accessors/{label}", "family": "math",
               "operation": "sectionpos_packed_accessors", "packed": packed}

    sectionpos_block_coordinates = {
        "origin": {"x": "0", "y": "0", "z": "0"},
        "positive": {"x": "1", "y": "2", "z": "3"},
        "negative": {"x": "-1", "y": "-2", "z": "-3"},
        "boundary": {"x": "134217727", "y": "-134217728", "z": "134217727"},
        "shift-wrap": {"x": "134217728", "y": "-134217729", "z": "-134217728"},
        "coordinate-boundaries": {"x": "2147483647", "y": "-2147483648", "z": "-2147483647"},
    }
    for label, values in sectionpos_block_coordinates.items():
        yield {"case_id": f"sectionpos/block-coordinates/{label}", "family": "math",
               "operation": "sectionpos_block_coordinates", **values}

    sectionpos_relative_to_block = {
        "origin-zero": {"x": "0", "y": "0", "z": "0", "relative": "0"},
        "positive-corner": {"x": "1", "y": "2", "z": "3", "relative": "4095"},
        "negative-all": {"x": "-1", "y": "-2", "z": "-3", "relative": "-1"},
        "mixed-relative": {"x": "-2", "y": "3", "z": "-4", "relative": "1322"},
        "short-min": {"x": "1", "y": "2", "z": "3", "relative": "-32768"},
        "coordinate-boundaries": {
            "x": "134217727", "y": "-134217728", "z": "-134217729", "relative": "4095"
        },
    }
    for label, values in sectionpos_relative_to_block.items():
        yield {"case_id": f"sectionpos/relative-to-block/{label}", "family": "math",
               "operation": "sectionpos_relative_to_block", **values}

    sectionpos_spatial = {
        "origin": {"x": "0", "y": "0", "z": "0"},
        "positive": {"x": "1", "y": "2", "z": "3"},
        "negative": {"x": "-1", "y": "-2", "z": "-3"},
        "boundary": {"x": "134217727", "y": "-134217728", "z": "134217727"},
        "shift-wrap": {"x": "134217728", "y": "-134217729", "z": "-134217728"},
        "coordinate-boundaries": {"x": "2147483647", "y": "-2147483648", "z": "-2147483647"},
    }
    for label, values in sectionpos_spatial.items():
        yield {"case_id": f"sectionpos/spatial/{label}", "family": "math",
               "operation": "sectionpos_spatial", **values}

    sectionpos_packed_transformations = {
        "origin": {
            "block_node": "0000000000000000", "section_node": "0000000000000000",
            "x": "0", "z": "0",
        },
        "positive": {
            "block_node": "0000004000003002", "section_node": "0000040000300002",
            "x": "1", "z": "3",
        },
        "negative": {
            "block_node": "ffffffffffffdffe", "section_node": "ffffffffffdffffe",
            "x": "-1", "z": "-3",
        },
        "block-boundaries": {
            "block_node": "7fffffe0000007ff", "section_node": "7ffffe0000080000",
            "x": "2097151", "z": "-2097152",
        },
        "negative-boundaries": {
            "block_node": "8000001ffffff800", "section_node": "800001fffff7ffff",
            "x": "-2097152", "z": "2097151",
        },
        "arbitrary-bits": {
            "block_node": "aa55aa5555aa55aa", "section_node": "aa55aa5555aa55aa",
            "x": "2147483647", "z": "-2147483648",
        },
    }
    for label, values in sectionpos_packed_transformations.items():
        yield {"case_id": f"sectionpos/packed-transformations/{label}", "family": "math",
               "operation": "sectionpos_packed_transformations", **values}

    sectionpos_as_long = {
        "origin": {
            "block_node": "0000000000000000",
            "x": "0", "y": "0", "z": "0",
            "section_x": "0", "section_y": "0", "section_z": "0",
        },
        "positive-boundaries": {
            "block_node": "000003c00001f010",
            "x": "1", "y": "2", "z": "3",
            "section_x": "1", "section_y": "2", "section_z": "3",
        },
        "negative-boundaries": {
            "block_node": "ffffffffffff0fff",
            "x": "-1", "y": "-2", "z": "-3",
            "section_x": "-1", "section_y": "-2", "section_z": "-3",
        },
        "packed-positive-limits": {
            "block_node": "7fffffdffffff7ff",
            "x": "2097151", "y": "524287", "z": "2097151",
            "section_x": "2097151", "section_y": "524287", "section_z": "2097151",
        },
        "packed-negative-limits": {
            "block_node": "8000002000000800",
            "x": "-2097152", "y": "-524288", "z": "-2097152",
            "section_x": "-2097152", "section_y": "-524288", "section_z": "-2097152",
        },
        "arbitrary-and-i32": {
            "block_node": "deadbeefcafebabe",
            "x": "2147483647", "y": "-2147483648", "z": "-2147483647",
            "section_x": "2147483647", "section_y": "-2147483648", "section_z": "-2147483647",
        },
    }
    for label, values in sectionpos_as_long.items():
        yield {"case_id": f"sectionpos/as-long/{label}", "family": "math",
               "operation": "sectionpos_as_long", **values}

    sectionpos_offset = {
        "origin-zero": {
            "section_node": "0000000000000000", "direction": "north",
            "step_x": "0", "step_y": "0", "step_z": "0",
            "x": "0", "y": "0", "z": "0",
        },
        "positive": {
            "section_node": "0000040000300002", "direction": "east",
            "step_x": "1", "step_y": "2", "step_z": "3",
            "x": "1", "y": "2", "z": "3",
        },
        "negative": {
            "section_node": "ffffffffffffffff", "direction": "down",
            "step_x": "-1", "step_y": "-2", "step_z": "-3",
            "x": "-1", "y": "-2", "z": "-3",
        },
        "packed-positive-limit": {
            "section_node": "7ffffdfffff7ffff", "direction": "up",
            "step_x": "1", "step_y": "1", "step_z": "1",
            "x": "2097151", "y": "524287", "z": "2097151",
        },
        "i32-wrap": {
            "section_node": "8000020000080000", "direction": "west",
            "step_x": "2147483647", "step_y": "-2147483648", "step_z": "-1",
            "x": "2147483647", "y": "-2147483648", "z": "2147483647",
        },
        "arbitrary-zero": {
            "section_node": "aa55aa5555aa55aa", "direction": "south",
            "step_x": "0", "step_y": "0", "step_z": "0",
            "x": "305419896", "y": "-305419896", "z": "1432778632",
        },
    }
    for label, values in sectionpos_offset.items():
        yield {"case_id": f"sectionpos/offset/{label}", "family": "math",
               "operation": "sectionpos_offset", **values}

    sectionpos_ranges = {
        "between-single": {
            "mode": "between",
            "min_x": "2", "min_y": "-3", "min_z": "4",
            "max_x": "2", "max_y": "-3", "max_z": "4",
        },
        "between-order": {
            "mode": "between",
            "min_x": "1", "min_y": "2", "min_z": "3",
            "max_x": "2", "max_y": "3", "max_z": "4",
        },
        "between-negative": {
            "mode": "between",
            "min_x": "-2", "min_y": "-1", "min_z": "-3",
            "max_x": "-1", "max_y": "0", "max_z": "-2",
        },
        "cube-origin": {
            "mode": "cube", "x": "0", "y": "0", "z": "0", "radius": "1",
        },
        "cube-boundary": {
            "mode": "cube", "x": "2147483647", "y": "-2147483648", "z": "-2147483647", "radius": "0",
        },
        "around-chunk": {
            "mode": "around-chunk", "chunk_x": "2", "chunk_z": "-3", "radius": "1",
            "min_section": "-1", "max_section": "0",
        },
    }
    for label, values in sectionpos_ranges.items():
        yield {"case_id": f"sectionpos/ranges/{label}", "family": "math",
               "operation": "sectionpos_ranges", **values}

    sectionpos_blocks_inside = {
        "origin": {"x": "0", "y": "0", "z": "0"},
        "negative": {"x": "-1", "y": "-2", "z": "-3"},
        "coordinate-boundaries": {
            "x": "134217727", "y": "-134217728", "z": "134217727",
        },
    }
    for label, values in sectionpos_blocks_inside.items():
        yield {"case_id": f"sectionpos/blocks-inside/{label}", "family": "math",
               "operation": "sectionpos_blocks_inside", **values}

    sectionpos_around_and_at = {
        "coordinates-interior": {
            "mode": "coordinates", "x": "1", "y": "2", "z": "3",
        },
        "coordinates-origin": {
            "mode": "coordinates", "x": "0", "y": "0", "z": "0",
        },
        "coordinates-x-boundary": {
            "mode": "coordinates", "x": "16", "y": "1", "z": "1",
        },
        "blockpos-overload": {
            "mode": "blockpos", "x": "15", "y": "16", "z": "31",
        },
        "packed-overload": {
            "mode": "packed", "block_node": "ffffffffffffffff",
        },
        "coordinate-overflow": {
            "mode": "coordinates", "x": "2147483647", "y": "1", "z": "1",
        },
    }
    for label, values in sectionpos_around_and_at.items():
        yield {"case_id": f"sectionpos/around-and-at/{label}", "family": "math",
               "operation": "sectionpos_around_and_at_block_pos", **values}

    sectionpos_double_coordinates = {
        "pos-positive-boundary": {"fn": "pos_to_section", "value": "4030000000000000"},
        "pos-negative-subnormal": {"fn": "pos_to_section", "value": "8000000000000001"},
        "pos-maximum-finite": {"fn": "pos_to_section", "value": "7fefffffffffffff"},
        "block-positive-fraction": {"fn": "block_to_section", "value": "403fffffffffffff"},
        "block-negative-boundary": {"fn": "block_to_section", "value": "c030000000000001"},
        "block-nan": {"fn": "block_to_section", "value": "7ff8000000000001"},
    }
    for label, values in sectionpos_double_coordinates.items():
        yield {"case_id": f"sectionpos/double-coordinates/{label}", "family": "math",
               "operation": "sectionpos_double_coordinates", **values}

    sectionpos_of_overloads = {
        "blockpos-boundaries": {
            "mode": "blockpos", "x": "16", "y": "-17", "z": "31",
        },
        "blockpos-int-boundaries": {
            "mode": "blockpos", "x": "2147483647", "y": "-2147483648", "z": "-1",
        },
        "chunkpos-section": {
            "mode": "chunkpos", "x": "-3", "z": "7", "section_y": "-64",
        },
        "packed-sign-extension": {
            "mode": "packed", "section_node": "ffffffffffffffff",
        },
        "position-fractions": {
            "mode": "position", "x": "8000000000000001", "y": "4030800000000000",
            "z": "c030000000000001",
        },
        "position-special-values": {
            "mode": "position", "x": "7fefffffffffffff", "y": "7ff8000000000001",
            "z": "fff0000000000000",
        },
    }
    for label, values in sectionpos_of_overloads.items():
        yield {"case_id": f"sectionpos/of-overloads/{label}", "family": "math",
               "operation": "sectionpos_of_overloads", **values}

    chunkpos_validity = {
        "origin": {"x": "0", "z": "0"},
        "boundary": {"x": "2097061", "z": "-2097061"},
        "above-boundary": {"x": "2097062", "z": "0"},
        "negative-boundary": {"x": "-2097061", "z": "2097060"},
        "signed-minimum-quirk": {"x": "-2147483648", "z": "0"},
        "signed-maximum": {"x": "2147483647", "z": "-2147483648"},
    }
    for label, values in chunkpos_validity.items():
        yield {"case_id": f"chunkpos/validity/{label}", "family": "math",
               "operation": "chunkpos_validity", **values}

    for name in ("pi", "half_pi", "two_pi", "deg_to_rad", "rad_to_deg", "epsilon",
                 "sqrt_of_two", "y_axis", "x_axis", "z_axis"):
        yield {"case_id": f"mth/constants/{name}", "family": "math",
               "operation": "mth_constants", "name": name}

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
    yield {"case_id": "vec3/rotate-clockwise-90/basic", "family": "math",
           "operation": "vec3_rotate_clockwise_90", "first": vector_bits(1.25, -2.5, 3.75)}
    yield {"case_id": "vec3/rotate-clockwise-90/negative-components", "family": "math",
           "operation": "vec3_rotate_clockwise_90", "first": vector_bits(-4.5, 6.25, -8.75)}
    yield {"case_id": "vec3/rotate-clockwise-90/signed-zero", "family": "math",
           "operation": "vec3_rotate_clockwise_90",
           "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"}}
    yield {"case_id": "vec3/rotate-clockwise-90/subnormal-and-large", "family": "math",
           "operation": "vec3_rotate_clockwise_90",
           "first": {"x": "0000000000000001", "y": "8000000000000001", "z": "7fefffffffffffff"}}
    yield {"case_id": "vec3/rotate-clockwise-90/special-values", "family": "math",
           "operation": "vec3_rotate_clockwise_90",
           "first": {"x": float_bits(float("inf"), "f64"), "y": float_bits(float("nan"), "f64"),
                     "z": float_bits(float("-inf"), "f64")}}
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
    yield {"case_id": "vec3/get/x", "family": "math", "operation": "vec3_get",
           "first": vector_bits(1.25, -2.5, 3.75), "axis": "x"}
    yield {"case_id": "vec3/get/y", "family": "math", "operation": "vec3_get",
           "first": vector_bits(-4.5, 6.25, -8.75), "axis": "y"}
    yield {"case_id": "vec3/get/z-signed-zero", "family": "math", "operation": "vec3_get",
           "first": {"x": "0000000000000000", "y": "8000000000000000", "z": "8000000000000000"},
           "axis": "z"}
    yield {"case_id": "vec3/get/subnormal-and-large", "family": "math", "operation": "vec3_get",
           "first": {"x": "0000000000000001", "y": "8000000000000001", "z": "7fefffffffffffff"},
           "axis": "z"}
    yield {"case_id": "vec3/get/special-values", "family": "math", "operation": "vec3_get",
           "first": {"x": float_bits(float("inf"), "f64"), "y": "7ff8000000000001",
                     "z": float_bits(float("-inf"), "f64")}, "axis": "y"}
    yield {"case_id": "vec3/with/x", "family": "math", "operation": "vec3_with",
           "first": vector_bits(1.25, -2.5, 3.75), "axis": "x", "value": float_bits(9.5, "f64")}
    yield {"case_id": "vec3/with/y-signed-zero", "family": "math", "operation": "vec3_with",
           "first": {"x": "8000000000000000", "y": "0000000000000001", "z": "7fefffffffffffff"},
           "axis": "y", "value": "8000000000000000"}
    yield {"case_id": "vec3/with/z-subnormal", "family": "math", "operation": "vec3_with",
           "first": vector_bits(-4.5, 6.25, -8.75), "axis": "z", "value": "0000000000000001"}
    yield {"case_id": "vec3/with/x-special-value", "family": "math", "operation": "vec3_with",
           "first": {"x": float_bits(float("inf"), "f64"), "y": "7ff8000000000001",
                     "z": float_bits(float("-inf"), "f64")},
           "axis": "x", "value": "7ff8000000000001"}
    yield {"case_id": "vec3/with/y-max-finite", "family": "math", "operation": "vec3_with",
           "first": {"x": "0000000000000001", "y": "8000000000000001", "z": "8000000000000000"},
           "axis": "y", "value": "7fefffffffffffff"}
    yield {"case_id": "vec3/relative/east-basic", "family": "math", "operation": "vec3_relative",
           "first": vector_bits(1.25, -2.5, 3.75), "direction": "east", "distance": float_bits(9.5, "f64")}
    yield {"case_id": "vec3/relative/west-signed-zero", "family": "math", "operation": "vec3_relative",
           "first": {"x": "8000000000000000", "y": "0000000000000001", "z": "8000000000000000"},
           "direction": "west", "distance": "0000000000000000"}
    yield {"case_id": "vec3/relative/up-subnormal", "family": "math", "operation": "vec3_relative",
           "first": {"x": "8000000000000000", "y": "8000000000000001", "z": "7fefffffffffffff"},
           "direction": "up", "distance": "0000000000000001"}
    yield {"case_id": "vec3/relative/down-overflow", "family": "math", "operation": "vec3_relative",
           "first": {"x": "0000000000000001", "y": "fff0000000000000", "z": "8000000000000000"},
           "direction": "down", "distance": "7fefffffffffffff"}
    yield {"case_id": "vec3/relative/north-cancellation", "family": "math", "operation": "vec3_relative",
           "first": {"x": "7fefffffffffffff", "y": "8000000000000001", "z": "7fefffffffffffff"},
           "direction": "north", "distance": "7fefffffffffffff"}
    yield {"case_id": "vec3/relative/south-special", "family": "math", "operation": "vec3_relative",
           "first": {"x": "3ff4000000000000", "y": "c004000000000000", "z": "7ff0000000000000"},
           "direction": "south", "distance": "7ff8000000000001"}
    yield {"case_id": "vec3/align/none-preserves", "family": "math", "operation": "vec3_align",
           "first": vector_bits(1.25, -2.5, 3.75), "axes": []}
    yield {"case_id": "vec3/align/all-floors", "family": "math", "operation": "vec3_align",
           "first": vector_bits(1.75, -2.25, 3.0), "axes": ["x", "y", "z"]}
    yield {"case_id": "vec3/align/xz-negative", "family": "math", "operation": "vec3_align",
           "first": vector_bits(-0.25, 1.75, -3.75), "axes": ["x", "z"]}
    yield {"case_id": "vec3/align/y-signed-zero", "family": "math", "operation": "vec3_align",
           "first": vector_bits(-0.0, -0.0, 0.0), "axes": ["y"]}
    yield {"case_id": "vec3/align/selected-special", "family": "math", "operation": "vec3_align",
           "first": {"x": "7ff0000000000000", "y": "7ff8000000000001", "z": "fff0000000000000"},
           "axes": ["x", "y", "z"]}
    yield {"case_id": "vec3/align/reordered-duplicate", "family": "math", "operation": "vec3_align",
           "first": {"x": "7fefffffffffffff", "y": "0000000000000001", "z": "bff0000000000000"},
           "axes": ["z", "x", "z"]}
    yield {"case_id": "vec3/projected-on/basic", "family": "math", "operation": "vec3_projected_on",
           "first": vector_bits(3.0, 4.0, 0.0), "onto": vector_bits(4.0, 0.0, 0.0)}
    yield {"case_id": "vec3/projected-on/orthogonal", "family": "math", "operation": "vec3_projected_on",
           "first": vector_bits(0.0, 1.0, 0.0), "onto": vector_bits(1.0, 0.0, 0.0)}
    yield {"case_id": "vec3/projected-on/zero-onto-preserves", "family": "math", "operation": "vec3_projected_on",
           "first": vector_bits(1.25, -2.5, 3.75),
           "onto": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"}}
    yield {"case_id": "vec3/projected-on/negative-fraction", "family": "math", "operation": "vec3_projected_on",
           "first": vector_bits(1.5, -2.25, 3.75), "onto": vector_bits(-2.0, 4.0, -1.0)}
    yield {"case_id": "vec3/projected-on/special-values", "family": "math", "operation": "vec3_projected_on",
           "first": {"x": "7ff0000000000000", "y": "3ff0000000000000", "z": "fff0000000000000"},
           "onto": vector_bits(1.0, 0.0, 1.0)}
    yield {"case_id": "vec3/projected-on/subnormal-onto", "family": "math", "operation": "vec3_projected_on",
           "first": {"x": "7fefffffffffffff", "y": "8000000000000001", "z": "0000000000000001"},
           "onto": {"x": "0000000000000001", "y": "0000000000000000", "z": "0000000000000000"}}
    yield {"case_id": "vec3/to-vector3f/basic", "family": "math", "operation": "vec3_to_vector3f",
           "first": vector_bits(1.25, -2.5, 3.75)}
    yield {"case_id": "vec3/to-vector3f/rounding", "family": "math", "operation": "vec3_to_vector3f",
           "first": vector_bits(1.00000006, -123456.789, 0.1)}
    yield {"case_id": "vec3/to-vector3f/signed-zero", "family": "math", "operation": "vec3_to_vector3f",
           "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000001"}}
    yield {"case_id": "vec3/to-vector3f/subnormal", "family": "math", "operation": "vec3_to_vector3f",
           "first": {"x": "0000000000000001", "y": "8000000000000001",
                     "z": float_bits(2.0 ** -149, "f64")}}
    yield {"case_id": "vec3/to-vector3f/overflow", "family": "math", "operation": "vec3_to_vector3f",
           "first": vector_bits(1.0e300, -1.0e300, 3.4028235e38)}
    yield {"case_id": "vec3/to-vector3f/special-values", "family": "math", "operation": "vec3_to_vector3f",
           "first": {"x": "7ff0000000000000", "y": "7ff8000000000001", "z": "fff0000000000000"}}
    yield {"case_id": "vec3/coordinates/lower-corner", "family": "math", "operation": "vec3_coordinates",
           "kind": "lower_corner", "x": str(2**31 - 1), "y": str(-(2**31)), "z": "-17"}
    yield {"case_id": "vec3/coordinates/lower-corner-offset", "family": "math", "operation": "vec3_coordinates",
           "kind": "lower_corner_offset", "x": "-4", "y": "5", "z": "-6",
           "offset_x": float_bits(0.25, "f64"), "offset_y": float_bits(-1.5, "f64"),
           "offset_z": float_bits(2.75, "f64")}
    yield {"case_id": "vec3/coordinates/center", "family": "math", "operation": "vec3_coordinates",
           "kind": "center", "x": "-2", "y": "-64", "z": "7"}
    yield {"case_id": "vec3/coordinates/bottom-center", "family": "math", "operation": "vec3_coordinates",
           "kind": "bottom_center", "x": "0", "y": "0", "z": "0"}
    yield {"case_id": "vec3/coordinates/up-from-bottom-center", "family": "math", "operation": "vec3_coordinates",
           "kind": "up_from_bottom_center", "x": "4", "y": "-5", "z": "6",
           "y_offset": float_bits(-0.75, "f64")}
    yield {"case_id": "vec3/coordinates/offset-special-values", "family": "math", "operation": "vec3_coordinates",
           "kind": "lower_corner_offset", "x": "0", "y": "0", "z": "0",
           "offset_x": "8000000000000000", "offset_y": "0000000000000001",
           "offset_z": "7ff8000000000001"}
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
    yield {"case_id": "vec3/rotate/x-basic", "family": "math", "operation": "vec3_rotate",
           "axis": "x", "first": vector_bits(1.25, -2.5, 3.75), "radians": float_bits(0.5)}
    yield {"case_id": "vec3/rotate/y-basic", "family": "math", "operation": "vec3_rotate",
           "axis": "y", "first": vector_bits(-4.5, 6.25, -8.75), "radians": float_bits(-0.75)}
    yield {"case_id": "vec3/rotate/z-basic", "family": "math", "operation": "vec3_rotate",
           "axis": "z", "first": vector_bits(2.0, -3.0, 4.0), "radians": float_bits(1.25)}
    yield {"case_id": "vec3/rotate/x-negative-angle", "family": "math", "operation": "vec3_rotate",
           "axis": "x", "first": vector_bits(-4.5, 6.25, -8.75), "radians": float_bits(-3.0)}
    yield {"case_id": "vec3/rotate/y-negative-angle", "family": "math", "operation": "vec3_rotate",
           "axis": "y", "first": vector_bits(1.25, -2.5, 3.75), "radians": float_bits(2.75)}
    yield {"case_id": "vec3/rotate/z-negative-angle", "family": "math", "operation": "vec3_rotate",
           "axis": "z", "first": vector_bits(-1.5, 2.25, -0.75), "radians": float_bits(-2.25)}
    yield {"case_id": "vec3/rotate/x-quantized-small-angle", "family": "math", "operation": "vec3_rotate",
           "axis": "x", "first": vector_bits(0.125, -0.25, 0.5), "radians": float_bits(0.0001)}
    yield {"case_id": "vec3/rotate/y-quantized-small-angle", "family": "math", "operation": "vec3_rotate",
           "axis": "y", "first": vector_bits(-0.125, 0.25, -0.5), "radians": float_bits(-0.0001)}
    yield {"case_id": "vec3/rotate/z-quarter-turn", "family": "math", "operation": "vec3_rotate",
           "axis": "z", "first": vector_bits(0.25, -0.5, 1.0),
           "radians": float_bits(1.5707963267948966)}
    yield {"case_id": "vec3/rotate/x-signed-zero", "family": "math", "operation": "vec3_rotate",
           "axis": "x", "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"},
           "radians": "80000000"}
    yield {"case_id": "vec3/rotate/y-signed-zero", "family": "math", "operation": "vec3_rotate",
           "axis": "y", "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"},
           "radians": "00000000"}
    yield {"case_id": "vec3/rotate/z-signed-zero", "family": "math", "operation": "vec3_rotate",
           "axis": "z", "first": {"x": "0000000000000000", "y": "8000000000000000", "z": "0000000000000000"},
           "radians": "80000000"}
    yield {"case_id": "vec3/rotate/x-special-vector", "family": "math", "operation": "vec3_rotate",
           "axis": "x", "first": {"x": "7ff0000000000000", "y": "7ff8000000000001", "z": "fff0000000000000"},
           "radians": float_bits(0.5)}
    yield {"case_id": "vec3/rotate/y-special-vector", "family": "math", "operation": "vec3_rotate",
           "axis": "y", "first": {"x": "fff0000000000000", "y": "7ff0000000000000", "z": "7ff8000000000001"},
           "radians": float_bits(-0.75)}
    yield {"case_id": "vec3/rotate/z-special-vector", "family": "math", "operation": "vec3_rotate",
           "axis": "z", "first": {"x": "7ff8000000000001", "y": "7ff0000000000000", "z": "fff0000000000000"},
           "radians": float_bits(1.25)}
    yield {"case_id": "vec3/rotate/x-nan-angle", "family": "math", "operation": "vec3_rotate",
           "axis": "x", "first": vector_bits(1.25, -2.5, 3.75), "radians": "7fc00001"}
    yield {"case_id": "vec3/rotate/y-positive-infinity-angle", "family": "math", "operation": "vec3_rotate",
           "axis": "y", "first": vector_bits(-4.5, 6.25, -8.75), "radians": "7f800000"}
    yield {"case_id": "vec3/rotate/z-negative-infinity-angle", "family": "math", "operation": "vec3_rotate",
           "axis": "z", "first": vector_bits(2.0, -3.0, 4.0), "radians": "ff800000"}
    yield {"case_id": "vec3/direction-from-rotation/components-basic", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": float_bits(30.0), "rot_y": float_bits(45.0)}
    yield {"case_id": "vec3/direction-from-rotation/vec2-basic", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "vec2",
           "rotation": vector2_bits(20.0, -35.0)}
    yield {"case_id": "vec3/direction-from-rotation/components-zero", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": float_bits(0.0), "rot_y": float_bits(0.0)}
    yield {"case_id": "vec3/direction-from-rotation/vec2-yaw-quarter", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "vec2",
           "rotation": vector2_bits(0.0, 90.0)}
    yield {"case_id": "vec3/direction-from-rotation/components-yaw-negative", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": float_bits(0.0), "rot_y": float_bits(-90.0)}
    yield {"case_id": "vec3/direction-from-rotation/vec2-yaw-half", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "vec2",
           "rotation": vector2_bits(0.0, 180.0)}
    yield {"case_id": "vec3/direction-from-rotation/components-pitch-up", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": float_bits(90.0), "rot_y": float_bits(0.0)}
    yield {"case_id": "vec3/direction-from-rotation/vec2-pitch-down", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "vec2",
           "rotation": vector2_bits(-90.0, 0.0)}
    yield {"case_id": "vec3/direction-from-rotation/components-combined", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": float_bits(37.5), "rot_y": float_bits(-123.25)}
    yield {"case_id": "vec3/direction-from-rotation/vec2-wrap", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "vec2",
           "rotation": vector2_bits(360.0, 720.0)}
    yield {"case_id": "vec3/direction-from-rotation/components-small-quantized", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": float_bits(0.0001), "rot_y": float_bits(-0.0001)}
    yield {"case_id": "vec3/direction-from-rotation/vec2-signed-zero", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "vec2",
           "rotation": {"x": "80000000", "y": "00000000"}}
    yield {"case_id": "vec3/direction-from-rotation/components-signed-zero", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": "00000000", "rot_y": "80000000"}
    yield {"case_id": "vec3/direction-from-rotation/components-nan-pitch", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": "7fc00001", "rot_y": float_bits(45.0)}
    yield {"case_id": "vec3/direction-from-rotation/vec2-nan-yaw", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "vec2",
           "rotation": {"x": float_bits(30.0), "y": "7fc00001"}}
    yield {"case_id": "vec3/direction-from-rotation/components-positive-infinity-pitch", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": "7f800000", "rot_y": float_bits(45.0)}
    yield {"case_id": "vec3/direction-from-rotation/vec2-negative-infinity-yaw", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "vec2",
           "rotation": {"x": float_bits(30.0), "y": "ff800000"}}
    yield {"case_id": "vec3/direction-from-rotation/components-max-finite", "family": "math",
           "operation": "vec3_direction_from_rotation", "kind": "components",
           "rot_x": "7f7fffff", "rot_y": "7f7fffff"}
    yield {"case_id": "vec3/local-coordinates/apply-basic", "family": "math",
           "operation": "vec3_local_coordinates", "kind": "apply",
           "rotation": vector2_bits(0.0, 0.0),
           "direction": vector_bits(1.25, -2.5, 3.75)}
    yield {"case_id": "vec3/local-coordinates/apply-combined", "family": "math",
           "operation": "vec3_local_coordinates", "kind": "apply",
           "rotation": vector2_bits(20.0, -35.0),
           "direction": vector_bits(-4.5, 6.25, 8.75)}
    yield {"case_id": "vec3/local-coordinates/apply-axis-turns", "family": "math",
           "operation": "vec3_local_coordinates", "kind": "apply",
           "rotation": vector2_bits(90.0, 180.0),
           "direction": vector_bits(1.0, 2.0, -3.0)}
    yield {"case_id": "vec3/local-coordinates/apply-signed-zero", "family": "math",
           "operation": "vec3_local_coordinates", "kind": "apply",
           "rotation": {"x": "80000000", "y": "00000000"},
           "direction": {"x": "8000000000000000", "y": "0000000000000001", "z": "8000000000000001"}}
    yield {"case_id": "vec3/local-coordinates/add-axis", "family": "math",
           "operation": "vec3_local_coordinates", "kind": "add",
           "first": vector_bits(0.0, 0.0, 1.0),
           "direction": vector_bits(1.0, 2.0, 3.0)}
    yield {"case_id": "vec3/local-coordinates/add-combined", "family": "math",
           "operation": "vec3_local_coordinates", "kind": "add",
           "first": vector_bits(-4.0, 5.0, -6.0),
           "direction": vector_bits(-0.5, 1.25, -2.75)}
    yield {"case_id": "vec3/rotation/basic", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(1.0, -2.0, 3.0)}
    yield {"case_id": "vec3/rotation/x-positive", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(1.0, 0.0, 0.0)}
    yield {"case_id": "vec3/rotation/x-negative", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(-1.0, 0.0, 0.0)}
    yield {"case_id": "vec3/rotation/z-positive", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(0.0, 0.0, 1.0)}
    yield {"case_id": "vec3/rotation/z-negative", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(0.0, 0.0, -1.0)}
    yield {"case_id": "vec3/rotation/y-positive", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(0.0, 1.0, 0.0)}
    yield {"case_id": "vec3/rotation/y-negative", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(0.0, -1.0, 0.0)}
    yield {"case_id": "vec3/rotation/combined", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(1.0, 2.0, 3.0)}
    yield {"case_id": "vec3/rotation/negative-combined", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(-4.0, 5.0, -6.0)}
    yield {"case_id": "vec3/rotation/non-unit", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(0.5, -1.5, 2.25)}
    yield {"case_id": "vec3/rotation/zero", "family": "math", "operation": "vec3_rotation",
           "first": vector_bits(0.0, 0.0, 0.0)}
    yield {"case_id": "vec3/rotation/signed-zero", "family": "math", "operation": "vec3_rotation",
           "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"}}
    yield {"case_id": "vec3/rotation/signed-zero-atan2", "family": "math", "operation": "vec3_rotation",
           "first": {"x": "0000000000000000", "y": "8000000000000000", "z": "8000000000000000"}}
    yield {"case_id": "vec3/rotation/subnormal", "family": "math", "operation": "vec3_rotation",
           "first": {"x": "0000000000000001", "y": "0000000000000001", "z": "0000000000000001"}}
    yield {"case_id": "vec3/rotation/subnormal-normal", "family": "math", "operation": "vec3_rotation",
           "first": {"x": "0000000000000001", "y": "3ff0000000000000", "z": "0000000000000000"}}
    yield {"case_id": "vec3/rotation/max-finite", "family": "math", "operation": "vec3_rotation",
           "first": {"x": "7fefffffffffffff", "y": "7fefffffffffffff", "z": "7fefffffffffffff"}}
    yield {"case_id": "vec3/rotation/infinities", "family": "math", "operation": "vec3_rotation",
           "first": {"x": "7ff0000000000000", "y": "3ff0000000000000", "z": "fff0000000000000"}}
    yield {"case_id": "vec3/rotation/nan", "family": "math", "operation": "vec3_rotation",
           "first": {"x": "7ff8000000000001", "y": "3ff0000000000000", "z": "0000000000000000"}}
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
    for name in ("zero", "x_axis", "y_axis", "z_axis"):
        yield {"case_id": f"vec3/constants/{name}", "family": "math", "operation": "vec3_constant",
               "name": name}
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
    for direction in ("down", "up", "north", "south", "west", "east"):
        for axis in ("x", "y", "z"):
            yield {"case_id": f"direction/axial-rotation/{direction}/{axis}", "family": "math",
                   "operation": "direction_axial_rotation", "name": direction, "axis": axis}
    for direction in ("down", "up", "north", "south", "west", "east"):
        yield {"case_id": f"direction/y-rotation/{direction}", "family": "math",
               "operation": "direction_y_rotation", "name": direction}
    direction_data_values = {
        "3d": [0, 1, 2, 3, 4, 5, -1, -2, -3, -4, -5, -6, -7,
               6, 11, -11, -2147483648, 2147483647],
        "2d": [0, 1, 2, 3, -1, -2, -3, -4, -5, 4, 7, -7,
               -2147483648, 2147483647],
    }
    for width, values in direction_data_values.items():
        for value in values:
            yield {"case_id": f"direction/data/{width}/{value}", "family": "math",
                   "operation": "direction_data", "type": width, "value": str(value)}
    direction_y_rot_values = [
        ("zero", float_bits(0.0, "f64")),
        ("negative-zero", float_bits(-0.0, "f64")),
        ("south-quarter", float_bits(44.99999999999999, "f64")),
        ("west-boundary", float_bits(45.0, "f64")),
        ("west-quarter", float_bits(89.99999999999999, "f64")),
        ("north-boundary", float_bits(135.0, "f64")),
        ("east-boundary", float_bits(225.0, "f64")),
        ("south-wrap", float_bits(315.0, "f64")),
        ("south-full-turn", float_bits(360.0, "f64")),
        ("negative-east-boundary", float_bits(-45.0, "f64")),
        ("negative-east-before", float_bits(-45.00000000000001, "f64")),
        ("negative-south-after", float_bits(-44.99999999999999, "f64")),
        ("negative-north-boundary", float_bits(-135.0, "f64")),
        ("negative-full-turn", float_bits(-360.0, "f64")),
        ("subnormal", "0000000000000001"),
        ("negative-subnormal", "8000000000000001"),
        ("positive-int-bound", float_bits((2**31) * 90.0, "f64")),
        ("negative-int-bound", float_bits(-(2**31) * 90.0, "f64")),
        ("positive-infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("nan", "7ff8000000000001"),
    ]
    for name, value in direction_y_rot_values:
        yield {"case_id": f"direction/y-rot/{name}", "family": "math",
               "operation": "direction_y_rot", "value": value}
    for axis in ("x", "y", "z"):
        for direction in ("positive", "negative"):
            yield {"case_id": f"direction/axis/{axis}/{direction}", "family": "math",
                   "operation": "direction_axis", "axis": axis, "direction": direction}
            yield {"case_id": f"direction/get/{axis}/{direction}", "family": "math",
                   "operation": "direction_get", "axis": axis, "direction": direction}
    for name in ("down", "up", "north", "south", "west", "east"):
        yield {"case_id": f"direction/unit-vectors/{name}", "family": "math",
               "operation": "direction_unit_vectors", "name": name}
        yield {"case_id": f"direction/step/{name}", "family": "math",
               "operation": "direction_step", "name": name}
    for axis in ("x", "y", "z"):
        yield {"case_id": f"direction/axis-orientation/{axis}", "family": "math",
               "operation": "direction_axis_orientation", "axis": axis}
    for axis in ("x", "y", "z"):
        yield {"case_id": f"direction/axis-directions/{axis}", "family": "math",
               "operation": "direction_axis_directions", "axis": axis}
    for direction in ("positive", "negative"):
        yield {"case_id": f"direction/axis-direction-values/{direction}", "family": "math",
               "operation": "direction_axis_direction_values", "direction": direction}
    direction_axis_choose_cases = {
        "distinct": {
            "int_x": "-17", "int_y": "11", "int_z": "29",
            "double_x": float_bits(-1.25, "f64"),
            "double_y": float_bits(2.5, "f64"),
            "double_z": float_bits(3.75, "f64"),
            "bool_x": True, "bool_y": False, "bool_z": True,
        },
        "special": {
            "int_x": str(-(2**31)), "int_y": str(2**31 - 1), "int_z": "0",
            "double_x": "8000000000000000",
            "double_y": "7ff0000000000000",
            "double_z": "7ff8000000000001",
            "bool_x": False, "bool_y": True, "bool_z": False,
        },
    }
    for label, values in direction_axis_choose_cases.items():
        for axis in ("x", "y", "z"):
            yield {"case_id": f"direction/axis-choose/{label}/{axis}", "family": "math",
                   "operation": "direction_axis_choose", "axis": axis, **values}
    for axis in ("x", "y", "z"):
        for direction in ("down", "up", "north", "south", "west", "east", "none"):
            yield {"case_id": f"direction/axis-test/{axis}/{direction}", "family": "math",
                   "operation": "direction_axis_test", "axis": axis, "direction": direction}
    for axis in ("x", "y", "z"):
        yield {"case_id": f"direction/axis-plane/{axis}", "family": "math",
               "operation": "direction_axis_plane", "axis": axis}
    for plane in ("horizontal", "vertical"):
        for direction in ("down", "up", "north", "south", "west", "east", "none"):
            yield {"case_id": f"direction/plane-test/{plane}/{direction}", "family": "math",
                   "operation": "direction_plane_test", "plane": plane, "direction": direction}
        yield {"case_id": f"direction/plane-order/{plane}", "family": "math",
               "operation": "direction_plane_order", "plane": plane}
    for seed in ("0", "1", "-1", "42", "12345", "9223372036854775807",
                 "-9223372036854775808"):
        for plane in ("horizontal", "vertical"):
            yield {"case_id": f"direction/random/{seed}/{plane}", "family": "math",
                   "operation": "direction_random", "seed": seed, "plane": plane}
            yield {"case_id": f"direction/plane-shuffle/{seed}/{plane}", "family": "math",
                   "operation": "direction_plane_shuffle", "seed": seed, "plane": plane}
        yield {"case_id": f"direction/all-shuffle/{seed}", "family": "math",
               "operation": "direction_all_shuffle", "seed": seed}
    yield {"case_id": "direction/stream", "family": "math", "operation": "direction_stream"}
    for name in ("down", "up", "north", "south", "west", "east"):
        yield {"case_id": f"direction/to-y-rot/{name}", "family": "math",
               "operation": "direction_to_y_rot", "name": name}
    for name in ("down", "up", "north", "south", "west", "east"):
        yield {"case_id": f"direction/get-y-rot/{name}", "family": "math",
               "operation": "direction_get_y_rot", "name": name}
    for name in ("down", "up", "north", "south", "west", "east"):
        yield {"case_id": f"direction/rotation/{name}", "family": "math",
               "operation": "direction_rotation", "name": name}
    direction_name_lookup_cases = (
        ("direction", ("down", "up", "north", "south", "west", "east",
                        "NORTH", "North", "north ", "")),
        ("axis", ("x", "y", "z", "X", "Y", "z ", "")),
    )
    for kind, names in direction_name_lookup_cases:
        for name in names:
            label = name if name else "empty"
            yield {"case_id": f"direction/name-lookup/{kind}/{label}", "family": "math",
                   "operation": "direction_name_lookup", "kind": kind, "name": name}
    direction_enum_text_cases = (
        ("axis_direction", ("positive", "negative")),
        ("plane", ("horizontal", "vertical")),
    )
    for kind, names in direction_enum_text_cases:
        for name in names:
            yield {"case_id": f"direction/enum-text/{kind}/{name}", "family": "math",
                   "operation": "direction_enum_text", "kind": kind, "name": name}
    direction_enum_name_cases = (
        ("direction", ("down", "up", "north", "south", "west", "east")),
        ("axis", ("x", "y", "z")),
    )
    for kind, names in direction_enum_name_cases:
        for name in names:
            yield {"case_id": f"direction/enum-name/{kind}/{name}", "family": "math",
                   "operation": "direction_enum_name", "kind": kind, "name": name}
    direction_approximate_nearest_cases = [
        ("south", vector_bits(0.0, 0.0, 1.0)),
        ("north", vector_bits(0.0, 0.0, -1.0)),
        ("east", vector_bits(1.0, 0.0, 0.0)),
        ("west", vector_bits(-1.0, 0.0, 0.0)),
        ("up", vector_bits(0.0, 1.0, 0.0)),
        ("down", vector_bits(0.0, -1.0, 0.0)),
        ("zero", vector_bits(0.0, 0.0, 0.0)),
        ("all-negative", vector_bits(-1.0, -1.0, -1.0)),
        ("tie-horizontal", vector_bits(1.0, 0.0, 1.0)),
        ("tie-vertical-horizontal", vector_bits(1.0, 1.0, 0.0)),
        ("f64-to-f32", vector_bits(1.0000000596046448, 0.0, 0.0)),
        ("positive-infinity-x", {"x": "7ff0000000000000", "y": "0000000000000000", "z": "0000000000000000"}),
        ("negative-infinity-z", {"x": "0000000000000000", "y": "0000000000000000", "z": "fff0000000000000"}),
        ("nan-x", {"x": "7ff8000000000001", "y": "0000000000000000", "z": "0000000000000000"}),
        ("subnormal", {"x": "0000000000000001", "y": "0000000000000000", "z": "0000000000000000"}),
        ("max-finite", {"x": "7fefffffffffffff", "y": "0000000000000000", "z": "0000000000000000"}),
    ]
    for name, vector in direction_approximate_nearest_cases:
        yield {"case_id": f"direction/approximate-nearest/{name}", "family": "math",
               "operation": "direction_approximate_nearest", **vector}
    direction_approximate_nearest_overload_cases = [
        ("cardinal", {"f32_x": float_bits(0.0), "f32_y": float_bits(0.0), "f32_z": float_bits(1.0),
                       "x": float_bits(0.0, "f64"), "y": float_bits(0.0, "f64"), "z": float_bits(1.0, "f64")}),
        ("tie", {"f32_x": float_bits(1.0), "f32_y": float_bits(0.0), "f32_z": float_bits(1.0),
                  "x": float_bits(1.0, "f64"), "y": float_bits(0.0, "f64"), "z": float_bits(1.0, "f64")}),
        ("zero", {"f32_x": float_bits(0.0), "f32_y": float_bits(0.0), "f32_z": float_bits(0.0),
                   "x": float_bits(0.0, "f64"), "y": float_bits(0.0, "f64"), "z": float_bits(0.0, "f64")}),
        ("negative", {"f32_x": float_bits(-1.0), "f32_y": float_bits(-2.0), "f32_z": float_bits(-3.0),
                       "x": float_bits(-1.0, "f64"), "y": float_bits(-2.0, "f64"), "z": float_bits(-3.0, "f64")}),
        ("f32-nan", {"f32_x": "7fc00001", "f32_y": float_bits(0.0), "f32_z": float_bits(0.0),
                      "x": "7ff8000000000001", "y": float_bits(0.0, "f64"), "z": float_bits(0.0, "f64")}),
        ("f32-infinity", {"f32_x": "7f800000", "f32_y": float_bits(0.0), "f32_z": float_bits(0.0),
                           "x": "7ff0000000000000", "y": float_bits(0.0, "f64"), "z": float_bits(0.0, "f64")}),
        ("precision-boundary", {"f32_x": "3f800000", "f32_y": float_bits(0.0), "f32_z": float_bits(0.0),
                                 "x": "3ff0000010000000", "y": float_bits(0.0, "f64"), "z": float_bits(0.0, "f64")}),
        ("signed-zero", {"f32_x": "80000000", "f32_y": "00000000", "f32_z": "00000000",
                          "x": "8000000000000000", "y": "0000000000000000", "z": "0000000000000000"}),
    ]
    for name, values in direction_approximate_nearest_overload_cases:
        yield {"case_id": f"direction/approximate-nearest-overloads/{name}", "family": "math",
               "operation": "direction_approximate_nearest_overloads", **values}
    direction_nearest_cases = [
        ("components/x-positive", "components", 3, 1, 2, "none"),
        ("components/x-negative", "components", -3, 1, 2, "none"),
        ("components/z-positive", "components", 1, 2, 4, "none"),
        ("components/z-negative", "components", 1, 2, -4, "none"),
        ("components/y-positive", "components", 1, 5, 2, "none"),
        ("components/y-negative", "components", 1, -5, 2, "none"),
        ("components/tie-x-z", "components", 4, 1, 4, "west"),
        ("components/tie-x-y", "components", 4, 4, 1, "south"),
        ("components/zero-none", "components", 0, 0, 0, "none"),
        ("components/zero-fallback", "components", 0, 0, 0, "north"),
        ("components/min-x", "components", -2147483648, 0, 0, "none"),
        ("components/min-y", "components", 0, -2147483648, 0, "east"),
        ("vec3i/x-positive", "vec3i", 3, 1, 2, "none"),
        ("vec3i/z-negative", "vec3i", 1, 2, -4, "none"),
        ("vec3i/tie", "vec3i", 2, 2, 0, "down"),
        ("vec3i/zero-none", "vec3i", 0, 0, 0, "none"),
        ("vec3i/min-z", "vec3i", 0, 0, -2147483648, "up"),
    ]
    for name, kind, x, y, z, or_else in direction_nearest_cases:
        yield {"case_id": f"direction/nearest/{name}", "family": "math",
               "operation": "direction_nearest", "kind": kind, "x": str(x),
               "y": str(y), "z": str(z), "or_else": or_else}
    direction_axis_step_values = [
        ("x-smaller", vector_bits(1.0, 0.0, 2.0)),
        ("z-smaller", vector_bits(2.0, 0.0, 1.0)),
        ("tie", vector_bits(2.0, 0.0, 2.0)),
        ("negative-x-smaller", vector_bits(-1.0, 0.0, 2.0)),
        ("negative-both", vector_bits(-2.0, 0.0, -3.0)),
        ("zero", vector_bits(0.0, 0.0, 0.0)),
        ("signed-zero", {"x": "8000000000000000", "y": "0000000000000000", "z": "0000000000000000"}),
        ("subnormal-x", {"x": "0000000000000001", "y": "0000000000000000", "z": "0000000000000002"}),
        ("positive-infinity-x", {"x": "7ff0000000000000", "y": "0000000000000000", "z": float_bits(1.0, "f64")}),
        ("positive-infinity-z", {"x": float_bits(1.0, "f64"), "y": "0000000000000000", "z": "7ff0000000000000"}),
        ("both-infinity", {"x": "7ff0000000000000", "y": "0000000000000000", "z": "7ff0000000000000"}),
        ("nan-x", {"x": "7ff8000000000001", "y": "0000000000000000", "z": float_bits(1.0, "f64")}),
        ("nan-z", {"x": float_bits(1.0, "f64"), "y": "0000000000000000", "z": "7ff8000000000001"}),
    ]
    for name, vector in direction_axis_step_values:
        yield {"case_id": f"direction/axis-step-order/{name}", "family": "math",
               "operation": "direction_axis_step_order", **vector}
    direction_facing_angle_cases = [
        ("south-zero", "south", float_bits(0.0)),
        ("south-quarter", "south", float_bits(90.0)),
        ("south-half-turn", "south", float_bits(180.0)),
        ("south-three-quarter", "south", float_bits(270.0)),
        ("north-zero", "north", float_bits(0.0)),
        ("north-half-turn", "north", float_bits(180.0)),
        ("east-zero", "east", float_bits(0.0)),
        ("east-negative-quarter", "east", float_bits(-90.0)),
        ("west-zero", "west", float_bits(0.0)),
        ("west-quarter", "west", float_bits(90.0)),
        ("up-zero", "up", float_bits(0.0)),
        ("down-negative-quarter", "down", float_bits(-90.0)),
        ("south-before-quarter", "south", float_bits(89.0)),
        ("south-after-quarter", "south", float_bits(91.0)),
        ("west-diagonal", "west", float_bits(45.0)),
        ("east-diagonal", "east", float_bits(-45.0)),
        ("negative-angle", "south", float_bits(-45.0)),
        ("full-turn", "south", float_bits(360.0)),
        ("signed-zero", "south", "80000000"),
        ("positive-subnormal", "south", "00000001"),
        ("negative-subnormal", "south", "80000001"),
        ("positive-infinity", "south", "7f800000"),
        ("negative-infinity", "south", "ff800000"),
        ("nan", "south", "7fc00001"),
        ("max-finite", "east", "7f7fffff"),
        ("min-negative-finite", "west", "ff7fffff"),
    ]
    for name, direction, angle in direction_facing_angle_cases:
        yield {"case_id": f"direction/is-facing-angle/{name}", "family": "math",
               "operation": "direction_is_facing_angle", "name": direction, "angle": angle}
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
    blockpos_packed_helpers = {
        "origin": {
            "x": "0", "y": "0", "z": "0", "neighbor_node": "0000000000000000",
            "direction": "up", "step_x": "1", "step_y": "-2", "step_z": "3",
        },
        "negative-fields": {
            "x": "-1", "y": "-2", "z": "-3", "neighbor_node": "ffffffffffffffff",
            "direction": "north", "step_x": "-4", "step_y": "5", "step_z": "-6",
        },
        "packed-boundaries": {
            "x": "33554431", "y": "2047", "z": "-33554432", "neighbor_node": "8000001ffffff800",
            "direction": "east", "step_x": "1", "step_y": "1", "step_z": "1",
        },
        "masked-overflow": {
            "x": "33554432", "y": "2048", "z": "-33554433", "neighbor_node": "7fffffe0000007ff",
            "direction": "down", "step_x": "2147483647", "step_y": "-2147483648", "step_z": "-1",
        },
        "integer-boundaries": {
            "x": "2147483647", "y": "-2147483648", "z": "2147483647", "neighbor_node": "deadbeefcafebabe",
            "direction": "west", "step_x": "-2147483648", "step_y": "2147483647", "step_z": "0",
        },
    }
    for label, values in blockpos_packed_helpers.items():
        yield {"case_id": f"blockpos/packed-helpers/{label}", "family": "math",
               "operation": "blockpos_packed_helpers", **values}
    blockpos_relative_helpers = {
        "basic": {
            "x": "10", "y": "20", "z": "30", "direction": "north", "axis": "x", "steps": "3",
        },
        "zero-steps": {
            "x": "-10", "y": "-20", "z": "-30", "direction": "up", "axis": "y", "steps": "0",
        },
        "negative-steps": {
            "x": "17", "y": "-16", "z": "1", "direction": "west", "axis": "z", "steps": "-2",
        },
        "overflow": {
            "x": "2147483647", "y": "-2147483648", "z": "2147483647",
            "direction": "east", "axis": "y", "steps": "2147483647",
        },
    }
    for label, values in blockpos_relative_helpers.items():
        yield {"case_id": f"blockpos/relative-helpers/{label}", "family": "math",
               "operation": "blockpos_relative_helpers", **values}
    blockpos_vector_helpers = {
        "basic": {
            "x": "10", "y": "-20", "z": "30", "dx": "2", "dy": "3", "dz": "-4", "scale": "3",
        },
        "zero-delta-scale-one": {
            "x": "-10", "y": "20", "z": "-30", "dx": "0", "dy": "0", "dz": "0", "scale": "1",
        },
        "zero-scale": {
            "x": "17", "y": "-16", "z": "1", "dx": "-5", "dy": "6", "dz": "-7", "scale": "0",
        },
        "negative": {
            "x": "-17", "y": "16", "z": "-1", "dx": "5", "dy": "-6", "dz": "7", "scale": "-3",
        },
        "overflow": {
            "x": "2147483647", "y": "-2147483648", "z": "2147483647",
            "dx": "-2147483648", "dy": "2147483647", "dz": "-2147483648", "scale": "-2147483648",
        },
    }
    for label, values in blockpos_vector_helpers.items():
        yield {"case_id": f"blockpos/vector-helpers/{label}", "family": "math",
               "operation": "blockpos_vector_helpers", **values}
    blockpos_geometry_helpers = {
        "basic": {
            "x": "1", "y": "2", "z": "3", "rotation": "clockwise_90",
            "up_x": "4", "up_y": "5", "up_z": "6", "at_y": "9",
        },
        "none-negative": {
            "x": "-10", "y": "20", "z": "-30", "rotation": "none",
            "up_x": "-2", "up_y": "3", "up_z": "-4", "at_y": "-7",
        },
        "half-turn": {
            "x": "17", "y": "-16", "z": "1", "rotation": "180",
            "up_x": "-5", "up_y": "6", "up_z": "-7", "at_y": "0",
        },
        "counterclockwise": {
            "x": "-17", "y": "16", "z": "-1", "rotation": "counterclockwise_90",
            "up_x": "5", "up_y": "-6", "up_z": "7", "at_y": "31",
        },
        "overflow": {
            "x": "2147483647", "y": "-2147483648", "z": "-2147483648", "rotation": "180",
            "up_x": "-2147483648", "up_y": "2147483647", "up_z": "-2147483648", "at_y": "2147483647",
        },
    }
    for label, values in blockpos_geometry_helpers.items():
        yield {"case_id": f"blockpos/geometry-helpers/{label}", "family": "math",
               "operation": "blockpos_geometry_helpers", **values}
    blockpos_distance_helpers = {
        "basic": {
            "x": "0", "y": "0", "z": "0",
            "other": {"x": "3", "y": "4", "z": "0"},
            "point": vector_bits(1.0, 2.0, 3.0), "distance": float_bits(5.0, "f64"),
        },
        "center": {
            "x": "-10", "y": "20", "z": "-30",
            "other": {"x": "-7", "y": "24", "z": "-30"},
            "point": vector_bits(-9.5, 20.5, -29.5), "distance": float_bits(4.0, "f64"),
        },
        "negative": {
            "x": "17", "y": "-16", "z": "1",
            "other": {"x": "-17", "y": "16", "z": "-1"},
            "point": vector_bits(-16.5, 15.5, -0.5), "distance": float_bits(64.0, "f64"),
        },
        "integer-boundaries": {
            "x": "2147483647", "y": "-2147483648", "z": "2147483647",
            "other": {"x": "-2147483648", "y": "2147483647", "z": "-2147483648"},
            "point": vector_bits(2147483647.5, -2147483648.5, 2147483647.5),
            "distance": float_bits(4294967296.0, "f64"),
        },
        "special-point": {
            "x": "1", "y": "2", "z": "3",
            "other": {"x": "1", "y": "2", "z": "3"},
            "point": {"x": "7ff8000000000002", "y": "7ff0000000000000", "z": "fff0000000000000"},
            "distance": "7ff8000000000002",
        },
    }
    for label, values in blockpos_distance_helpers.items():
        yield {"case_id": f"blockpos/distance-helpers/{label}", "family": "math",
               "operation": "blockpos_distance_helpers", **values}
    blockpos_integer_helpers = {
        "basic": {
            "x": "0", "y": "0", "z": "0", "other": {"x": "3", "y": "-4", "z": "5"},
        },
        "negative": {
            "x": "-10", "y": "20", "z": "-30", "other": {"x": "-7", "y": "24", "z": "-35"},
        },
        "same": {
            "x": "17", "y": "-16", "z": "1", "other": {"x": "17", "y": "-16", "z": "1"},
        },
        "min-abs-edge": {
            "x": "0", "y": "-2147483648", "z": "0",
            "other": {"x": "-2147483648", "y": "0", "z": "-2147483648"},
        },
        "integer-boundaries": {
            "x": "2147483647", "y": "-2147483648", "z": "2147483647",
            "other": {"x": "-2147483648", "y": "2147483647", "z": "-2147483648"},
        },
    }
    for label, values in blockpos_integer_helpers.items():
        yield {"case_id": f"blockpos/integer-helpers/{label}", "family": "math",
               "operation": "blockpos_integer_helpers", **values}
    blockpos_clamp_location = {
        "inside": {
            "x": "10", "y": "20", "z": "30",
            "location": vector_bits(10.5, 20.5, 30.5),
        },
        "outside": {
            "x": "-10", "y": "20", "z": "-30",
            "location": vector_bits(-11.0, 21.0, -29.0),
        },
        "boundary": {
            "x": "0", "y": "-1", "z": "1",
            "location": vector_bits(-0.0, -0.0, 1.0),
        },
        "integer-boundaries": {
            "x": "2147483647", "y": "-2147483648", "z": "2147483647",
            "location": vector_bits(2147483647.5, -2147483648.5, 2147483647.5),
        },
        "special": {
            "x": "1", "y": "2", "z": "3",
            "location": {"x": "7ff8000000000002", "y": "7ff0000000000000", "z": "fff0000000000000"},
        },
    }
    for label, values in blockpos_clamp_location.items():
        yield {"case_id": f"blockpos/clamp-location/{label}", "family": "math",
               "operation": "blockpos_clamp_location", **values}
    blockpos_between_closed = {
        "single": {
            "from": {"x": "2", "y": "-3", "z": "4"},
            "to": {"x": "2", "y": "-3", "z": "4"},
            "min_x": "-1", "min_y": "2", "min_z": "-3", "max_x": "0", "max_y": "3", "max_z": "-2",
        },
        "order": {
            "from": {"x": "1", "y": "2", "z": "3"},
            "to": {"x": "2", "y": "3", "z": "4"},
            "min_x": "-2", "min_y": "-1", "min_z": "-3", "max_x": "-1", "max_y": "0", "max_z": "-2",
        },
        "reversed": {
            "from": {"x": "2", "y": "3", "z": "4"},
            "to": {"x": "1", "y": "2", "z": "3"},
            "min_x": "3", "min_y": "4", "min_z": "5", "max_x": "4", "max_y": "5", "max_z": "6",
        },
        "negative": {
            "from": {"x": "-2", "y": "-1", "z": "-3"},
            "to": {"x": "-1", "y": "0", "z": "-2"},
            "min_x": "-3", "min_y": "-2", "min_z": "-1", "max_x": "-2", "max_y": "-1", "max_z": "0",
        },
        "integer-boundaries": {
            "from": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
            "to": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
            "min_x": "2147483647", "min_y": "-2147483648", "min_z": "31",
            "max_x": "2147483647", "max_y": "-2147483648", "max_z": "31",
        },
    }
    for label, values in blockpos_between_closed.items():
        yield {"case_id": f"blockpos/between-closed/{label}", "family": "math",
               "operation": "blockpos_between_closed", **values}
    blockpos_neighbor_column = {
        "ascending": {
            "start_x": "10", "start_y": "5", "start_z": "-2", "end_y": "7",
        },
        "descending": {
            "start_x": "-3", "start_y": "4", "start_z": "9", "end_y": "2",
        },
        "singleton": {
            "start_x": "0", "start_y": "-2", "start_z": "0", "end_y": "-2",
        },
        "integer-boundaries": {
            "start_x": "2147483647", "start_y": "-2147483648", "start_z": "-2147483648",
            "end_y": "-2147483648",
        },
    }
    for label, values in blockpos_neighbor_column.items():
        yield {"case_id": f"blockpos/neighbor-column/{label}", "family": "math",
               "operation": "blockpos_neighbor_column", **values}
    blockpos_square_out_south_east = {
        "basic": {"x": "1", "y": "2", "z": "3"},
        "negative": {"x": "-4", "y": "5", "z": "-6"},
        "zero": {"x": "0", "y": "0", "z": "0"},
        "integer-boundaries": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
    }
    for label, values in blockpos_square_out_south_east.items():
        yield {"case_id": f"blockpos/square-out-south-east/{label}", "family": "math",
               "operation": "blockpos_square_out_south_east", **values}
    blockpos_random_ranges = {
        "basic": {
            "seed": "42", "limit": "5",
            "min_x": "-2", "min_y": "0", "min_z": "3",
            "max_x": "2", "max_y": "4", "max_z": "7",
            "center": {"x": "10", "y": "20", "z": "-5"}, "size": "2",
        },
        "zero-limit": {
            "seed": "12345", "limit": "0",
            "min_x": "-1", "min_y": "-1", "min_z": "-1",
            "max_x": "1", "max_y": "1", "max_z": "1",
            "center": {"x": "0", "y": "0", "z": "0"}, "size": "1",
        },
        "unit-ranges": {
            "seed": "-1", "limit": "3",
            "min_x": "7", "min_y": "-8", "min_z": "9",
            "max_x": "7", "max_y": "-8", "max_z": "9",
            "center": {"x": "-4", "y": "5", "z": "-6"}, "size": "0",
        },
        "integer-boundaries": {
            "seed": "9223372036854775807", "limit": "3",
            "min_x": "2147483645", "min_y": "-2147483647", "min_z": "2147483645",
            "max_x": "-2147483647", "max_y": "-2147483645", "max_z": "-2147483647",
            "center": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"}, "size": "2",
        },
    }
    for label, values in blockpos_random_ranges.items():
        yield {"case_id": f"blockpos/random-ranges/{label}", "family": "math",
               "operation": "blockpos_random_ranges", **values}
    blockpos_between_closed_aabb = {
        "fractional": box_bits(0.25, 0.25, 0.25, 1.75, 1.75, 1.75),
        "negative-fractional": box_bits(-1.75, -0.2, -3.0, -0.25, 0.8, -1.1),
        "reversed": box_bits(2.5, 1.5, 3.5, 0.5, -0.5, 1.5),
        "integer-boundaries": box_bits(0.0, -1.0, 2.0, 1.0, 0.0, 3.0),
        "signed-zero": box_bits(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0),
    }
    for label, values in blockpos_between_closed_aabb.items():
        yield {"case_id": f"blockpos/between-closed-aabb/{label}", "family": "math",
               "operation": "blockpos_between_closed_aabb", "aabb": values}
    blockpos_within_manhattan = {
        "unit": {
            "origin": {"x": "0", "y": "0", "z": "0"},
            "reach_x": "1", "reach_y": "1", "reach_z": "1",
        },
        "asymmetric": {
            "origin": {"x": "10", "y": "-5", "z": "7"},
            "reach_x": "2", "reach_y": "1", "reach_z": "1",
        },
        "flat": {
            "origin": {"x": "-3", "y": "4", "z": "-2"},
            "reach_x": "1", "reach_y": "0", "reach_z": "0",
        },
        "integer-boundaries": {
            "origin": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
            "reach_x": "1", "reach_y": "1", "reach_z": "0",
        },
    }
    for label, values in blockpos_within_manhattan.items():
        yield {"case_id": f"blockpos/within-manhattan/{label}", "family": "math",
               "operation": "blockpos_within_manhattan", **values}
    blockpos_spiral_around = {
        "radius-one-horizontal": {
            "center": {"x": "0", "y": "0", "z": "0"}, "radius": "1",
            "first_direction": "north", "second_direction": "east",
        },
        "radius-two-horizontal": {
            "center": {"x": "5", "y": "-2", "z": "7"}, "radius": "2",
            "first_direction": "west", "second_direction": "south",
        },
        "vertical": {
            "center": {"x": "2147483647", "y": "-2147483648", "z": "0"}, "radius": "1",
            "first_direction": "up", "second_direction": "north",
        },
        "zero-radius": {
            "center": {"x": "-3", "y": "4", "z": "-5"}, "radius": "0",
            "first_direction": "south", "second_direction": "east",
        },
        "negative-radius": {
            "center": {"x": "1", "y": "2", "z": "3"}, "radius": "-1",
            "first_direction": "north", "second_direction": "east",
        },
    }
    for label, values in blockpos_spiral_around.items():
        yield {"case_id": f"blockpos/spiral-around/{label}", "family": "math",
               "operation": "blockpos_spiral_around", **values}
    blockpos_find_closest_match = {
        "origin": {
            "start": {"x": "0", "y": "0", "z": "0"},
            "horizontal_radius": "1", "vertical_radius": "1", "target": {"x": "0", "y": "0", "z": "0"},
        },
        "reachable": {
            "start": {"x": "10", "y": "-5", "z": "7"},
            "horizontal_radius": "2", "vertical_radius": "1", "target": {"x": "9", "y": "-5", "z": "8"},
        },
        "unreachable": {
            "start": {"x": "-3", "y": "4", "z": "-2"},
            "horizontal_radius": "1", "vertical_radius": "1", "target": {"x": "4", "y": "4", "z": "-2"},
        },
        "zero-radii": {
            "start": {"x": "6", "y": "-7", "z": "8"},
            "horizontal_radius": "0", "vertical_radius": "0", "target": {"x": "6", "y": "-7", "z": "9"},
        },
        "integer-boundaries": {
            "start": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
            "horizontal_radius": "1", "vertical_radius": "0",
            "target": {"x": "-2147483648", "y": "-2147483648", "z": "2147483647"},
        },
    }
    for label, values in blockpos_find_closest_match.items():
        yield {"case_id": f"blockpos/find-closest-match/{label}", "family": "math",
               "operation": "blockpos_find_closest_match", **values}
    blockpos_between_closed_streams = {
        "normal": {
            "from": {"x": "2", "y": "-1", "z": "3"},
            "to": {"x": "1", "y": "0", "z": "4"},
            "bounding_box": {"min_x": "-2", "min_y": "1", "min_z": "3", "max_x": "-1", "max_y": "2", "max_z": "4"},
            "aabb": box_bits(0.25, -1.25, 2.0, 1.75, 0.75, 3.5),
            "min_x": "4", "min_y": "-2", "min_z": "5", "max_x": "5", "max_y": "-1", "max_z": "6",
        },
        "reversed": {
            "from": {"x": "-1", "y": "3", "z": "2"},
            "to": {"x": "1", "y": "1", "z": "0"},
            "bounding_box": {"min_x": "2", "min_y": "0", "min_z": "-1", "max_x": "1", "max_y": "-1", "max_z": "-2"},
            "aabb": box_bits(2.5, 1.5, 0.5, 0.5, -0.5, -1.5),
            "min_x": "-3", "min_y": "-2", "min_z": "-1", "max_x": "-2", "max_y": "-1", "max_z": "0",
        },
        "integer-boundaries": {
            "from": {"x": "2147483647", "y": "-2147483648", "z": "0"},
            "to": {"x": "2147483647", "y": "-2147483648", "z": "0"},
            "bounding_box": {"min_x": "2147483647", "min_y": "-2147483648", "min_z": "0", "max_x": "2147483647", "max_y": "-2147483648", "max_z": "0"},
            "aabb": box_bits(2147483647.0, -2147483648.0, 0.0, 2147483647.0, -2147483648.0, 0.0),
            "min_x": "2147483647", "min_y": "-2147483648", "min_z": "0", "max_x": "2147483647", "max_y": "-2147483648", "max_z": "0",
        },
    }
    for label, values in blockpos_between_closed_streams.items():
        yield {"case_id": f"blockpos/between-closed-streams/{label}", "family": "math",
               "operation": "blockpos_between_closed_streams", **values}
    blockpos_coordinate_helpers = {
        "fractional": {
            "x": float_bits(2.75, "f64"), "y": float_bits(-1.0, "f64"), "z": float_bits(-0.01, "f64"),
            "point": vector_bits(3.1, -2.2, 4.9),
            "first": {"x": "3", "y": "-4", "z": "5"}, "second": {"x": "-2", "y": "6", "z": "1"},
        },
        "negative-fractional": {
            "x": float_bits(-0.01, "f64"), "y": float_bits(-2.0, "f64"), "z": float_bits(3.0, "f64"),
            "point": vector_bits(-1.1, 2.0, -3.9),
            "first": {"x": "-10", "y": "20", "z": "-30"}, "second": {"x": "-7", "y": "24", "z": "-35"},
        },
        "signed-zero": {
            "x": float_bits(-0.0, "f64"), "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64"),
            "point": vector_bits(-0.0, 0.0, -0.0),
            "first": {"x": "0", "y": "-1", "z": "2"}, "second": {"x": "0", "y": "-1", "z": "2"},
        },
        "integer-boundaries": {
            "x": float_bits(2147483647.0, "f64"), "y": float_bits(-2147483648.0, "f64"), "z": float_bits(31.0, "f64"),
            "point": vector_bits(2147483646.5, -2147483647.5, 31.5),
            "first": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
            "second": {"x": "-2147483648", "y": "2147483647", "z": "-2147483648"},
        },
    }
    for label, values in blockpos_coordinate_helpers.items():
        yield {"case_id": f"blockpos/coordinate-helpers/{label}", "family": "math",
               "operation": "blockpos_coordinate_helpers", **values}
    blockpos_between_corners_direction = {
        "positive-z": {
            "aabb": box_bits(1.2, -1.8, 2.0, 3.9, 1.1, 4.2),
            "first": {"x": "1", "y": "-1", "z": "2"},
            "second": {"x": "3", "y": "1", "z": "4"},
            "first_x": "1", "first_y": "-1", "first_z": "2",
            "second_x": "3", "second_y": "1", "second_z": "4",
            "direction": vector_bits(0.5, 0.25, 2.0),
        },
        "negative-x": {
            "aabb": box_bits(4.8, -1.2, -2.1, -0.2, 2.8, 1.7),
            "first": {"x": "4", "y": "2", "z": "-3"},
            "second": {"x": "-1", "y": "-1", "z": "0"},
            "first_x": "4", "first_y": "2", "first_z": "-3",
            "second_x": "-1", "second_y": "-1", "second_z": "0",
            "direction": vector_bits(-2.0, -0.5, 0.25),
        },
        "negative-z": {
            "aabb": box_bits(-0.2, 0.0, -0.2, 1.8, 1.9, 1.8),
            "first": {"x": "0", "y": "0", "z": "0"},
            "second": {"x": "1", "y": "1", "z": "1"},
            "first_x": "0", "first_y": "0", "first_z": "0",
            "second_x": "1", "second_y": "1", "second_z": "1",
            "direction": vector_bits(0.5, -1.0, -2.0),
        },
        "signed-zero": {
            "aabb": box_bits(-0.0, -0.0, -0.0, 1.0, 1.0, 1.0),
            "first": {"x": "-1", "y": "2", "z": "-3"},
            "second": {"x": "0", "y": "3", "z": "-2"},
            "first_x": "-1", "first_y": "2", "first_z": "-3",
            "second_x": "0", "second_y": "3", "second_z": "-2",
            "direction": vector_bits(-0.0, -0.0, 0.0),
        },
        "nan-direction": {
            "aabb": box_bits(0.0, -2.0, 3.0, 2.0, 0.0, 4.0),
            "first": {"x": "0", "y": "-2", "z": "3"},
            "second": {"x": "2", "y": "0", "z": "4"},
            "first_x": "0", "first_y": "-2", "first_z": "3",
            "second_x": "2", "second_y": "0", "second_z": "4",
            "direction": vector_bits(float("nan"), float("nan"), 1.0),
        },
        "integer-boundary-overflow": {
            "aabb": box_bits(2147483647.0, 0.0, 0.0, -2147483648.0, 0.0, 0.0),
            "first": {"x": "2147483647", "y": "0", "z": "0"},
            "second": {"x": "-2147483648", "y": "0", "z": "0"},
            "first_x": "2147483647", "first_y": "0", "first_z": "0",
            "second_x": "-2147483648", "second_y": "0", "second_z": "0",
            "direction": vector_bits(-1.0, 0.0, 0.0),
        },
    }
    for label, values in blockpos_between_corners_direction.items():
        yield {"case_id": f"blockpos/between-corners-direction/{label}", "family": "math",
               "operation": "blockpos_between_corners_direction", **values}
    def traversal_node(position, status, neighbors):
        return {"position": {"x": str(position[0]), "y": str(position[1]), "z": str(position[2])},
                "status": status,
                "neighbors": [{"x": str(x), "y": str(y), "z": str(z)} for x, y, z in neighbors]}

    blockpos_breadth_first_traversal = {
        "breadth-first": {
            "start": {"x": "0", "y": "0", "z": "0"}, "max_depth": "2", "max_count": "99",
            "nodes": [
                traversal_node((0, 0, 0), "accept", [(1, 0, 0), (0, 0, 1)]),
                traversal_node((1, 0, 0), "accept", [(2, 0, 0), (0, 0, 0)]),
                traversal_node((0, 0, 1), "accept", [(2, 0, 0), (0, 0, 2)]),
                traversal_node((2, 0, 0), "skip", [(3, 0, 0)]),
                traversal_node((0, 0, 2), "accept", [(0, 0, 3)]),
            ],
        },
        "stop": {
            "start": {"x": "0", "y": "0", "z": "0"}, "max_depth": "3", "max_count": "99",
            "nodes": [
                traversal_node((0, 0, 0), "accept", [(1, 0, 0), (0, 0, 1)]),
                traversal_node((1, 0, 0), "stop", [(2, 0, 0)]),
                traversal_node((0, 0, 1), "accept", [(0, 0, 2)]),
            ],
        },
        "skip-root": {
            "start": {"x": "4", "y": "-2", "z": "7"}, "max_depth": "3", "max_count": "1",
            "nodes": [
                traversal_node((4, -2, 7), "skip", [(5, -2, 7)]),
                traversal_node((5, -2, 7), "accept", []),
            ],
        },
        "max-count": {
            "start": {"x": "0", "y": "0", "z": "0"}, "max_depth": "4", "max_count": "2",
            "nodes": [
                traversal_node((0, 0, 0), "accept", [(1, 0, 0), (2, 0, 0)]),
                traversal_node((1, 0, 0), "accept", [(3, 0, 0)]),
                traversal_node((2, 0, 0), "accept", []),
            ],
        },
        "depth-limit": {
            "start": {"x": "-1", "y": "3", "z": "5"}, "max_depth": "1", "max_count": "99",
            "nodes": [
                traversal_node((-1, 3, 5), "accept", [(0, 3, 5)]),
                traversal_node((0, 3, 5), "accept", [(1, 3, 5)]),
            ],
        },
        "packed-visited": {
            "start": {"x": "0", "y": "0", "z": "0"}, "max_depth": "1", "max_count": "99",
            "nodes": [
                traversal_node((0, 0, 0), "accept", [(33554432, 0, 0)]),
                traversal_node((33554432, 0, 0), "stop", []),
            ],
        },
    }
    for label, values in blockpos_breadth_first_traversal.items():
        yield {"case_id": f"blockpos/breadth-first-traversal/{label}", "family": "math",
               "operation": "blockpos_breadth_first_traversal", **values}
    blockpos_mutable_helpers = {
        "all-overloads": {
            "initial": {"x": "1", "y": "-2", "z": "3"},
            "double_x": float_bits(-0.1, "f64"), "double_y": float_bits(2.0, "f64"),
            "double_z": float_bits(-3.9, "f64"),
            "steps": [
                {"op": "set", "x": "4", "y": "-5", "z": "6"},
                {"op": "set_double", "x": float_bits(1.9, "f64"), "y": float_bits(-2.0, "f64"),
                 "z": float_bits(-3.1, "f64")},
                {"op": "set_block_pos", "value": {"x": "-7", "y": "8", "z": "-9"}},
                {"op": "set_packed", "value": "ffffffffffffffff"},
                {"op": "set_with_offset_direction", "position": {"x": "10", "y": "20", "z": "30"},
                 "direction": "west"},
                {"op": "set_with_offset_components", "position": {"x": "-4", "y": "5", "z": "-6"},
                 "x": "1", "y": "-2", "z": "3"},
                {"op": "set_with_offset_block_pos", "position": {"x": "7", "y": "-8", "z": "9"},
                 "offset": {"x": "-2", "y": "3", "z": "-4"}},
                {"op": "move_direction", "direction": "east"},
                {"op": "move_direction_steps", "direction": "down", "steps": "-2"},
                {"op": "move_components", "x": "1", "y": "-2", "z": "3"},
                {"op": "move_block_pos", "value": {"x": "-1", "y": "2", "z": "-3"}},
                {"op": "clamp", "axis": "x", "minimum": "0", "maximum": "4"},
                {"op": "clamp", "axis": "y", "minimum": "-4", "maximum": "-2"},
                {"op": "clamp", "axis": "z", "minimum": "6", "maximum": "9"},
                {"op": "set_x", "value": "2147483647"},
                {"op": "set_y", "value": "-2147483648"},
                {"op": "set_z", "value": "0"},
                {"op": "offset_return", "x": "1", "y": "-1", "z": "2"},
                {"op": "multiply_return", "scale": "-2"},
                {"op": "relative_return", "direction": "north", "steps": "3"},
                {"op": "relative_axis_return", "axis": "z", "steps": "-4"},
                {"op": "rotate_return", "rotation": "clockwise_90"},
            ],
        },
        "overflow-and-clamp": {
            "initial": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
            "double_x": float_bits(2147483647.9, "f64"), "double_y": float_bits(-2147483648.1, "f64"),
            "double_z": float_bits(-0.01, "f64"),
            "steps": [
                {"op": "move_components", "x": "1", "y": "-1", "z": "1"},
                {"op": "set_with_offset_components", "position": {"x": "2147483647", "y": "-2147483648", "z": "2147483647"},
                 "x": "1", "y": "-1", "z": "1"},
                {"op": "clamp", "axis": "x", "minimum": "-2", "maximum": "-1"},
                {"op": "clamp", "axis": "y", "minimum": "1", "maximum": "2"},
                {"op": "clamp", "axis": "z", "minimum": "-4", "maximum": "-3"},
            ],
        },
    }
    for label, values in blockpos_mutable_helpers.items():
        yield {"case_id": f"blockpos/mutable-helpers/{label}", "family": "math",
               "operation": "blockpos_mutable_helpers", **values}
    axis_cycle_helpers = {
        "basic": {
            "x": "10", "y": "-20", "z": "30",
            "double_x": float_bits(1.25, "f64"), "double_y": float_bits(-2.5, "f64"),
            "double_z": float_bits(3.75, "f64"),
        },
        "boundaries": {
            "x": "2147483647", "y": "-2147483648", "z": "-1",
            "double_x": "8000000000000000", "double_y": "7ff8000000000002",
            "double_z": "7ff0000000000000",
        },
    }
    for label, values in axis_cycle_helpers.items():
        yield {"case_id": f"axis-cycle/helpers/{label}", "family": "math",
               "operation": "axis_cycle_helpers", **values}
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
    yield {"case_id": "shape/collide-movement/empty", "family": "math", "operation": "shape",
           "kind": "collide_movement", "movement": vector_bits(1.25, -2.5, 3.75),
           "moving": box_bits(0, 0, 0, 1, 1, 1), "shapes": []}
    yield {"case_id": "shape/collide-movement/x-then-z", "family": "math", "operation": "shape",
           "kind": "collide_movement", "movement": vector_bits(2, 0, 1),
           "moving": box_bits(0, 0, 0, 1, 1, 1),
           "shapes": [box_bits(2, 0, 1.5, 3, 1, 2.5), box_bits(1.5, 0, 2, 2.5, 1, 3)]}
    yield {"case_id": "shape/collide-movement/z-then-x", "family": "math", "operation": "shape",
           "kind": "collide_movement", "movement": vector_bits(1, 0, 2),
           "moving": box_bits(0, 0, 0, 1, 1, 1),
           "shapes": [box_bits(2, 0, 1.5, 3, 1, 2.5), box_bits(1.5, 0, 2, 2.5, 1, 3)]}
    yield {"case_id": "shape/collide-movement/y-first", "family": "math", "operation": "shape",
           "kind": "collide_movement", "movement": vector_bits(2, -2, 0),
           "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
           "shapes": [box_bits(-1, 0, -1, 2, 1, 2), box_bits(1, 1, -1, 2, 4, 2)]}
    yield {"case_id": "shape/entity-movement-state/base-flags", "family": "math",
           "operation": "shape", "kind": "entity_movement_state", "inputs": [
               {"label": "empty", "movement": vector_bits(1.25, -2.5, 3.75),
                "moving": box_bits(0, 0, 0, 1, 1, 1), "shapes": []},
               {"label": "horizontal-x", "movement": vector_bits(2, 0, 0),
                "moving": box_bits(0, 0, 0, 1, 1, 1),
                "shapes": [box_bits(1, -1, -1, 2, 4, 2)]},
               {"label": "horizontal-within-epsilon", "movement": vector_bits(2, 0, 0),
                "moving": box_bits(0, 0, 0, 1, 1, 1),
                "shapes": [box_bits(2.999995, -1, -1, 4, 4, 2)]},
               {"label": "falling-onto-floor", "movement": vector_bits(0, -2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 0, -1, 2, 1, 2)]},
               {"label": "rising-into-ceiling", "movement": vector_bits(0, 2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 4, -1, 2, 5, 2)]},
               {"label": "horizontal-and-falling", "movement": vector_bits(2, -2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 0, -1, 2, 1, 2),
                           box_bits(1, 1, -1, 2, 4, 2)]},
           ]}
    yield {"case_id": "shape/entity-movement-state/zero-y-retention", "family": "math",
           "operation": "shape", "kind": "entity_movement_state_retention", "inputs": [
               {"label": "zero-y-remote-retains-ground", "delta": vector_bits(2, 0, 0),
                "moving": box_bits(0, 0, 0, 1, 1, 1),
                "shapes": [box_bits(1, -1, -1, 2, 4, 2)],
                "previous": {"vertical_collision": True, "vertical_collision_below": True,
                              "on_ground": True}, "authoritative": False},
               {"label": "zero-y-authoritative-recomputes", "delta": vector_bits(2, 0, 0),
                "moving": box_bits(0, 0, 0, 1, 1, 1),
                "shapes": [box_bits(1, -1, -1, 2, 4, 2)],
                "previous": {"vertical_collision": True, "vertical_collision_below": True,
                              "on_ground": True}, "authoritative": True},
               {"label": "nonzero-y-remote-recomputes-falling", "delta": vector_bits(0, -2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 0, -1, 2, 1, 2)],
                "previous": {"vertical_collision": False, "vertical_collision_below": False,
                              "on_ground": False}, "authoritative": False},
               {"label": "nonzero-y-remote-recomputes-rising", "delta": vector_bits(0, 2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 4, -1, 2, 5, 2)],
                "previous": {"vertical_collision": True, "vertical_collision_below": True,
                              "on_ground": True}, "authoritative": False},
           ]}
    yield {"case_id": "shape/entity-collision-shapes/empty-test-area", "family": "math",
           "operation": "shape", "kind": "entity_collision_shapes",
           "test_area": box_bits(0, 0, 0, 0.00000001, 0.00000001, 0.00000001),
           "source_present": False,
           "candidates": [{"box": box_bits(0, 0, 0, 1, 1, 1),
                            "can_be_collided_with": True}]}
    yield {"case_id": "shape/entity-collision-shapes/source-free-filter-order", "family": "math",
           "operation": "shape", "kind": "entity_collision_shapes",
           "test_area": box_bits(0, 0, 0, 2, 1, 1), "source_present": False,
           "candidates": [
               {"is_source": True, "box": box_bits(0.25, 0, 0, 0.75, 1, 1),
                "can_be_collided_with": True},
               {"box": box_bits(0.5, 0, 0, 1.5, 1, 1), "spectator": True,
                "can_be_collided_with": True},
               {"box": box_bits(0.75, 0, 0, 1.75, 1, 1),
                "can_be_collided_with": False},
               {"box": box_bits(1, 0, 0, 1.5, 1, 1),
                "can_be_collided_with": True, "same_vehicle": True},
               {"box": box_bits(2.00000005, 0, 0, 2.5, 1, 1),
                "can_be_collided_with": True},
           ]}
    yield {"case_id": "shape/entity-collision-shapes/source-filter-and-except", "family": "math",
           "operation": "shape", "kind": "entity_collision_shapes",
           "test_area": box_bits(0, 0, 0, 2, 1, 1), "source_present": True,
           "source_box": box_bits(0.25, 0, 0, 0.75, 1, 1),
           "candidates": [
               {"is_source": True, "box": box_bits(0.25, 0, 0, 0.75, 1, 1),
                "can_be_collided_with": True},
               {"box": box_bits(0.5, 0, 0, 1.5, 1, 1),
                "can_be_collided_with": True},
               {"box": box_bits(0.75, 0, 0, 1.75, 1, 1),
                "can_be_collided_with": True, "same_vehicle": True},
               {"box": box_bits(1, 0, 0, 1.5, 1, 1), "spectator": True,
                "can_be_collided_with": True},
               {"box": box_bits(1.25, 0, 0, 1.75, 1, 1),
                "can_be_collided_with": False},
               {"box": box_bits(2.00000005, 0, 0, 2.5, 1, 1),
                "can_be_collided_with": True},
           ]}
    yield {"case_id": "shape/world-border-collision-shape/static-extents", "family": "math",
           "operation": "shape", "kind": "world_border_collision_shape",
           "inputs": [
            {"center_x": float_bits(0.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "size": float_bits(5.5, "f64"), "absolute_max_size": 100},
            {"center_x": float_bits(10.25, "f64"), "center_z": float_bits(-4.75, "f64"),
             "size": float_bits(9.5, "f64"), "absolute_max_size": 100},
            {"center_x": float_bits(90.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "size": float_bits(50.0, "f64"), "absolute_max_size": 100},
            {"center_x": float_bits(0.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "size": float_bits(300.0, "f64"), "absolute_max_size": 100},
           ]}
    yield {"case_id": "shape/world-border-collision-shape/moving-extents", "family": "math",
           "operation": "shape", "kind": "world_border_collision_shape_moving",
           "inputs": [
            {"center_x": float_bits(1.25, "f64"), "center_z": float_bits(-2.5, "f64"),
             "from": float_bits(20.0, "f64"), "to": float_bits(40.0, "f64"),
             "absolute_max_size": 100, "duration": 4, "tick_count": 4,
             "partial_ticks": [
                 float_bits(0.0, "f32"), float_bits(0.25, "f32"),
                 float_bits(0.5, "f32"), float_bits(1.0, "f32"),
             ]},
            {"center_x": float_bits(-10.25, "f64"), "center_z": float_bits(4.75, "f64"),
             "from": float_bits(40.0, "f64"), "to": float_bits(10.0, "f64"),
             "absolute_max_size": 15, "duration": 3, "tick_count": 3,
             "partial_ticks": [
                 float_bits(0.0, "f32"), float_bits(0.25, "f32"),
                 float_bits(0.5, "f32"), float_bits(1.0, "f32"),
             ]},
            {"center_x": float_bits(90.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "from": float_bits(50.0, "f64"), "to": float_bits(300.0, "f64"),
             "absolute_max_size": 100, "duration": 2, "tick_count": 2,
             "partial_ticks": [
                 float_bits(0.0, "f32"), float_bits(0.5, "f32"),
                 float_bits(1.0, "f32"),
             ]},
           ]}
    yield {"case_id": "world/client-border-state/packet-application", "family": "math",
           "operation": "shape", "kind": "client_world_border_state",
           "inputs": [
            {"events": [
                {"op": "initialize", "center_x": float_bits(2.25, "f64"),
                 "center_z": float_bits(-3.5, "f64"), "old_size": float_bits(20.0, "f64"),
                 "new_size": float_bits(40.0, "f64"), "lerp_time": 4,
                 "absolute_max_size": 30, "warning_blocks": 7, "warning_time": 20},
                {"op": "tick"},
                {"op": "tick"},
                {"op": "center", "center_x": float_bits(4.0, "f64"),
                 "center_z": float_bits(-1.0, "f64")},
                {"op": "lerp", "old_size": float_bits(30.0, "f64"),
                 "new_size": float_bits(10.0, "f64"), "lerp_time": 3},
                {"op": "tick"},
                {"op": "tick"},
                {"op": "tick"},
                {"op": "warning_blocks", "value": 12},
                {"op": "warning_time", "value": 40},
            ]},
            {"events": [
                {"op": "initialize", "center_x": float_bits(-8.0, "f64"),
                 "center_z": float_bits(5.0, "f64"), "old_size": float_bits(1.0, "f64"),
                 "new_size": float_bits(24.0, "f64"), "lerp_time": 0,
                 "absolute_max_size": 12, "warning_blocks": 3, "warning_time": 9},
                {"op": "size", "size": float_bits(8.0, "f64")},
                {"op": "center", "center_x": float_bits(-5.5, "f64"),
                 "center_z": float_bits(2.5, "f64")},
                {"op": "lerp", "old_size": float_bits(8.0, "f64"),
                 "new_size": float_bits(8.0, "f64"), "lerp_time": 5},
                {"op": "warning_blocks", "value": 0},
                {"op": "warning_time", "value": 0},
            ]},
           ]}
    yield {"case_id": "world/client-collision/no-collision-sources", "family": "math",
           "operation": "shape", "kind": "client_world_collision", "inputs": [
            {"label": "empty-null-source",
             "test_area": box_bits(0, 0, 0, 1, 1, 1), "source_present": False,
             "placements": [], "shape_states": [], "candidates": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(4, "f64"), "absolute_max_size": 100}},
            {"label": "block-source",
             "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
             "source_present": False,
             "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"}],
             "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}],
             "candidates": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(20, "f64"), "absolute_max_size": 100}},
            {"label": "entity-source",
             "test_area": box_bits(0, 0, 0, 1, 2, 1), "source_present": True,
             "source_box": box_bits(0, 0, 0, 1, 2, 1),
             "source_position": vector_bits(0.5, 1, 0.5),
             "placements": [], "shape_states": [],
             "candidates": [
                 {"is_source": True, "box": box_bits(0, 0, 0, 1, 2, 1),
                  "can_be_collided_with": True},
                 {"box": box_bits(0.5, 0, 0.5, 1.5, 2, 1.5),
                  "can_be_collided_with": True},
             ],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(20, "f64"), "absolute_max_size": 100}},
            {"label": "border-source",
             "test_area": box_bits(1.5, 0, 0, 2.5, 2, 1), "source_present": True,
             "source_box": box_bits(1.5, 0, 0, 2.5, 2, 1),
             "source_position": vector_bits(1.75, 1, 0.5),
             "placements": [], "shape_states": [], "candidates": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(4, "f64"), "absolute_max_size": 100}},
            {"label": "far-outside-source",
             "test_area": box_bits(10, 0, 0, 11, 2, 1), "source_present": True,
             "source_box": box_bits(10, 0, 0, 11, 2, 1),
             "source_position": vector_bits(10.5, 1, 0.5),
             "placements": [], "shape_states": [], "candidates": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(4, "f64"), "absolute_max_size": 100}},
           ]}
    yield {"case_id": "world/client-collision/indexed-entity-state", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "empty-index", "test_area": box_bits(0, 0, 0, 1, 1, 1),
             "source_entity_id": None, "entities": [], "placements": [],
             "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "indexed-boat-collision", "test_area": box_bits(0, 0, 0, 1, 1, 1),
             "source_entity_id": None,
             "entities": [{"entity_id": 7, "type_id": 0,
                           "position": vector_bits(0.5, 0, 0.5),
                           "can_be_collided_with": True}],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "indexed-boat-outside", "test_area": box_bits(0, 0, 0, 1, 1, 1),
             "source_entity_id": None,
             "entities": [{"entity_id": 7, "type_id": 0,
                           "position": vector_bits(3.5, 0, 3.5),
                           "can_be_collided_with": True}],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "source-excluded", "test_area": box_bits(0, 0, 0, 1, 1, 1),
             "source_entity_id": 7,
             "entities": [{"entity_id": 7, "type_id": 0,
                           "position": vector_bits(0.5, 0, 0.5),
                           "can_be_collided_with": True}],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "indexed-player-no-collider", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": None,
             "entities": [{"entity_id": 8, "type_id": 156,
                           "position": vector_bits(0.5, 0, 0.5),
                           "can_be_collided_with": False}],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
           ]}
    yield {"case_id": "world/client-collision/indexed-shulker-state", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "shulker-override-collision", "test_area": box_bits(0, 0, 0, 1, 1, 1),
             "source_entity_id": None,
             "entities": [{"entity_id": 9, "type_id": 112,
                           "position": vector_bits(0.5, 0, 0.5),
                           "can_be_collided_with": False}],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "shulker-outside-query", "test_area": box_bits(0, 0, 0, 1, 1, 1),
             "source_entity_id": None,
             "entities": [{"entity_id": 9, "type_id": 112,
                           "position": vector_bits(2, 0, 2),
                           "can_be_collided_with": False}],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
           ]}
    yield {"case_id": "world/client-collision/source-vehicle-pushable", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "boat-source-pushable-player", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "base-source-rejects-pushable-player", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": False},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "minecart-source-pushable-player", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 85,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": False},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "boat-source-pushable-minecart", "test_area": box_bits(0, 0, 0, 1, 1, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 85,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
           ]}
    yield {"case_id": "world/client-collision/living-pushable-state", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "alive-player-is-pushable", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "alive": True, "spectator": False,
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "spectator-player-is-not-pushable", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "alive": True, "spectator": True,
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "dead-player-is-not-pushable", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "alive": False, "spectator": False,
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
           ]}
    yield {"case_id": "world/client-collision/living-pushable-climbable", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "non-climbable-player-is-pushable", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "alive": True, "spectator": False, "on_climbable": False,
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "climbable-player-is-not-pushable", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "alive": True, "spectator": False, "on_climbable": True,
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
           ]}
    yield {"case_id": "world/client-collision/living-pushable-flying", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "climbable-grounded-player-is-not-pushable", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "alive": True, "spectator": False, "on_climbable": True,
                  "flying": False, "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "climbable-flying-player-is-pushable", "test_area": box_bits(0, 0, 0, 1, 2, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 0, 0.5),
                  "alive": True, "spectator": False, "on_climbable": True,
                  "flying": True, "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
           ]}
    yield {"case_id": "world/client-collision/living-pushable-trapdoor-context", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "open-matching-ladder-player-is-not-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "trapdoor_ladder", "trapdoor_open": True,
                                        "trapdoor_facing": "north", "ladder_facing": "north"},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "closed-trapdoor-player-is-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "trapdoor_ladder", "trapdoor_open": False,
                                        "trapdoor_facing": "north", "ladder_facing": "north"},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "open-mismatched-ladder-player-is-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "trapdoor_ladder", "trapdoor_open": True,
                                        "trapdoor_facing": "north", "ladder_facing": "south"},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "open-without-ladder-player-is-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "trapdoor_ladder", "trapdoor_open": True,
                                        "trapdoor_facing": "north", "ladder_facing": "north",
                                        "below_is_ladder": False},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
           ]}
    yield {"case_id": "world/client-collision/living-pushable-climbable-tag", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "ladder-state-is-climbable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "climbable_tag", "state_id": 5720},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "waterlogged-ladder-state-is-climbable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "climbable_tag", "state_id": 5719},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "stone-state-is-not-climbable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "climbable_tag", "state_id": 1},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "air-state-is-not-climbable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "climbable_tag", "state_id": 0},
                  "can_be_collided_with": False},
           ],
           "placements": [], "shape_states": [],
           "border": {"center_x": float_bits(0, "f64"),
                      "center_z": float_bits(0, "f64"),
                      "size": float_bits(40, "f64"), "absolute_max_size": 100}},
          ]}
    yield {"case_id": "world/client-collision/living-pushable-glide-through", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "vine-grounded-player-is-not-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "glide_through_tag", "state_id": 8389,
                                        "fall_flying": False},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "vine-fall-flying-player-is-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "glide_through_tag", "state_id": 8389,
                                        "fall_flying": True},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "stone-fall-flying-player-is-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "glide_through_tag", "state_id": 1,
                                        "fall_flying": True},
                  "can_be_collided_with": False},
           ],
           "placements": [], "shape_states": [],
           "border": {"center_x": float_bits(0, "f64"),
                      "center_z": float_bits(0, "f64"),
                      "size": float_bits(40, "f64"), "absolute_max_size": 100}},
          ]}
    yield {"case_id": "world/client-collision/living-pushable-update-tags-packet", "family": "math",
           "operation": "shape", "kind": "client_world_indexed_collision", "inputs": [
            {"label": "packet-vine-grounded-player-is-not-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "update_tags_packet", "state_id": 8389,
                                        "fall_flying": False,
                                        "block_tags": {"minecraft:climbable": [221, 366],
                                                        "minecraft:can_glide_through": [366]}},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "packet-vine-fall-flying-player-is-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "update_tags_packet", "state_id": 8389,
                                        "fall_flying": True,
                                        "block_tags": {"minecraft:climbable": [221, 366],
                                                        "minecraft:can_glide_through": [366]}},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
            {"label": "packet-ladder-grounded-player-is-not-pushable", "test_area": box_bits(0, 0, 0, 1, 3, 1),
             "source_entity_id": 7,
             "entities": [
                 {"entity_id": 7, "type_id": 0,
                  "position": vector_bits(0.5, 1, 0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 8, "type_id": 156,
                  "position": vector_bits(0.5, 1, 0.5),
                  "alive": True, "spectator": False, "flying": False,
                  "climbable_context": {"kind": "update_tags_packet", "state_id": 5720,
                                        "fall_flying": False,
                                        "block_tags": {"minecraft:climbable": [221, 366],
                                                        "minecraft:can_glide_through": [366]}},
                  "can_be_collided_with": False},
             ],
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(40, "f64"), "absolute_max_size": 100}},
           ]}
    yield {"case_id": "world/client-collision/entity-section-index-updates", "family": "math",
           "operation": "shape", "kind": "client_world_entity_section_index", "inputs": [
            {"label": "move-across-horizontal-and-vertical-sections",
             "entities": [
                 {"entity_id": 21, "type_id": 0,
                  "position": vector_bits(0.5, 0, 0.5),
                  "can_be_collided_with": True},
             ],
             "events": [
                 {"op": "query", "label": "initial-section",
                  "test_area": box_bits(-1, 0, -1, 2, 1, 2)},
                 {"op": "move", "entity_id": 21,
                  "position": vector_bits(1.5, 0, 0.5)},
                 {"op": "query", "label": "same-section-move",
                  "test_area": box_bits(0, 0, 0, 3, 1, 2)},
                 {"op": "move", "entity_id": 21,
                  "position": vector_bits(16.5, 0, 0.5)},
                 {"op": "query", "label": "old-horizontal-section",
                  "test_area": box_bits(-1, 0, -1, 3, 1, 2)},
                 {"op": "query", "label": "new-horizontal-section",
                  "test_area": box_bits(16, 0, 0, 18, 1, 2)},
                 {"op": "move", "entity_id": 21,
                  "position": vector_bits(16.5, 16, 0.5)},
                 {"op": "query", "label": "new-vertical-section",
                  "test_area": box_bits(16, 16, 0, 18, 18, 2)},
                 {"op": "remove", "entity_id": 21},
                 {"op": "query", "label": "removed-section",
                  "test_area": box_bits(16, 16, 0, 18, 18, 2)},
             ],
             "placements": [],
             "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
            {"label": "section-removal-keeps-neighbor",
             "entities": [
                 {"entity_id": 22, "type_id": 0,
                  "position": vector_bits(-0.5, -16, -0.5),
                  "can_be_collided_with": True},
                 {"entity_id": 23, "type_id": 0,
                  "position": vector_bits(31.5, 0, 0.5),
                  "can_be_collided_with": True},
             ],
             "events": [
                 {"op": "query", "label": "two-sections",
                  "test_area": box_bits(-2, -17, -2, 1, -15, 1)},
                 {"op": "remove", "entity_id": 22},
                 {"op": "query", "label": "neighbor-section-remains",
                  "test_area": box_bits(30, -1, -1, 33, 1, 2)},
                 {"op": "remove", "entity_id": 23},
                 {"op": "query", "label": "all-sections-removed",
                  "test_area": box_bits(30, -1, -1, 33, 1, 2)},
             ],
             "placements": [],
             "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
           ]}
    yield {"case_id": "world/client-collision/happy-ghast-collider-override", "family": "math",
           "operation": "shape", "kind": "client_world_happy_ghast_collision", "inputs": [
            {"label": "adult-still-timeout",
             "entities": [{"entity_id": 31, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": True,
                            "still_timeout": True, "is_vehicle": False}],
             "source_entity_id": None,
             "test_area": box_bits(-3, -1, -3, 3, 5, 3),
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
            {"label": "adult-free-no-source-collision",
             "entities": [{"entity_id": 32, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": True,
                            "still_timeout": False, "is_vehicle": False}],
             "source_entity_id": None,
             "test_area": box_bits(-3, -1, -3, 3, 5, 3),
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
            {"label": "baby-still-timeout-rejected",
             "entities": [{"entity_id": 33, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": True, "alive": True,
                            "still_timeout": True, "is_vehicle": False}],
             "source_entity_id": None,
             "test_area": box_bits(-1, -1, -1, 2, 2, 2),
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
            {"label": "dead-adult-still-timeout-rejected",
             "entities": [{"entity_id": 42, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": False,
                            "still_timeout": True, "is_vehicle": False}],
             "source_entity_id": None,
             "test_area": box_bits(-3, -1, -3, 3, 5, 3),
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
            {"label": "client-player-above-bounding-box",
             "entities": [{"entity_id": 34, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": True,
                            "still_timeout": False, "is_vehicle": False},
                           {"entity_id": 35, "type_id": 156,
                            "position": vector_bits(0, 4, 0)}],
             "source_entity_id": 35,
             "test_area": box_bits(-3, -1, -3, 3, 5, 3),
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
            {"label": "client-player-below-bounding-box",
             "entities": [{"entity_id": 36, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": True,
                            "still_timeout": False, "is_vehicle": False},
                           {"entity_id": 37, "type_id": 156,
                            "position": vector_bits(0, 3.999, 0)}],
             "source_entity_id": 37,
             "test_area": box_bits(-3, -1, -3, 3, 5, 3),
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
            {"label": "happy-ghast-vehicle-collides-with-happy-ghast",
             "entities": [{"entity_id": 38, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": True,
                            "still_timeout": False, "is_vehicle": False},
                           {"entity_id": 39, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": True,
                            "still_timeout": False, "is_vehicle": True}],
             "source_entity_id": 38,
             "test_area": box_bits(-3, -1, -3, 3, 5, 3),
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
            {"label": "happy-ghast-source-does-not-match-vehicle-branch",
             "entities": [{"entity_id": 40, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": True,
                            "still_timeout": False, "is_vehicle": False},
                           {"entity_id": 41, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": True,
                            "still_timeout": False, "is_vehicle": False}],
             "source_entity_id": 40,
             "test_area": box_bits(-3, -1, -3, 3, 5, 3),
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
            {"label": "source-boat-pushes-live-happy-ghast",
             "entities": [{"entity_id": 43, "type_id": 58,
                            "position": vector_bits(0, 0, 0),
                            "baby": False, "alive": True,
                            "still_timeout": False, "is_vehicle": False},
                           {"entity_id": 44, "type_id": 0,
                            "position": vector_bits(0, 0, 0)}],
             "source_entity_id": 44,
             "test_area": box_bits(-3, -1, -3, 3, 5, 3),
             "placements": [], "shape_states": [],
             "border": {"center_x": float_bits(0, "f64"),
                        "center_z": float_bits(0, "f64"),
                        "size": float_bits(1000, "f64"), "absolute_max_size": 100000}},
           ]}
    yield {"case_id": "world/client-collision/registry-state-context", "family": "math",
           "operation": "shape", "kind": "client_world_registry_collision",
           "state_ids": [0, 1, 13331, 13333, 13335, 99999],
           "test_area": box_bits(-0.25, -0.25, -0.25, 4.75, 1.25, 1.25),
           "placements": [
               {"x": 0, "y": 0, "z": 0, "state_id": 1},
               {"x": 1, "y": 0, "z": 0, "state_id": 13333},
               {"x": 2, "y": 0, "z": 0, "state_id": 13331},
               {"x": 3, "y": 0, "z": 0, "state_id": 13335},
               {"x": 4, "y": 0, "z": 0, "state_id": 99999},
           ]}
    yield {"case_id": "world/client-collision/stair-state-shapes", "family": "math",
           "operation": "shape", "kind": "client_world_registry_collision",
           "state_ids": [3908, 3918, 3920, 3926, 99999],
           "test_area": box_bits(-0.25, -0.25, -0.25, 4.75, 1.25, 1.25),
           "placements": [
               {"x": 0, "y": 0, "z": 0, "state_id": 3908},
               {"x": 1, "y": 0, "z": 0, "state_id": 3918},
               {"x": 2, "y": 0, "z": 0, "state_id": 3920},
               {"x": 3, "y": 0, "z": 0, "state_id": 3926},
               {"x": 4, "y": 0, "z": 0, "state_id": 99999},
           ]}
    yield {"case_id": "world/client-collision/stair-neighbor-shape-updates", "family": "math",
           "operation": "shape", "kind": "client_world_stair_neighbor_collision",
           "inputs": [
               {"label": "behind-outer-left", "target": {"x": 0, "y": 0, "z": 0},
                "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 3918},
                               {"x": 0, "y": 0, "z": -1, "state_id": 3958}]},
               {"label": "behind-outer-right", "target": {"x": 0, "y": 0, "z": 0},
                "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 3918},
                               {"x": 0, "y": 0, "z": -1, "state_id": 3978}]},
               {"label": "front-inner-left", "target": {"x": 0, "y": 0, "z": 0},
                "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 3918},
                               {"x": 0, "y": 0, "z": 1, "state_id": 3958}]},
               {"label": "front-inner-right", "target": {"x": 0, "y": 0, "z": 0},
                "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 3918},
                               {"x": 0, "y": 0, "z": 1, "state_id": 3978}]},
               {"label": "half-mismatch-keeps-straight", "target": {"x": 0, "y": 0, "z": 0},
                "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 3918},
                               {"x": 0, "y": 0, "z": -1, "state_id": 3908}]},
               {"label": "can-take-shape-blocked", "target": {"x": 0, "y": 0, "z": 0},
                "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 3918},
                               {"x": 0, "y": 0, "z": -1, "state_id": 3958},
                               {"x": 1, "y": 0, "z": 0, "state_id": 3918}]},
               {"label": "waterlogged-outer-left", "target": {"x": 0, "y": 0, "z": 0},
                "test_area": box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75),
                "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 3917},
                               {"x": 0, "y": 0, "z": -1, "state_id": 3958}]},
           ]}
    yield {"case_id": "world/client-collision/local-player-minor-horizontal", "family": "math",
           "operation": "shape", "kind": "local_player_minor_horizontal_collision",
           "inputs": [
               {"label": "aligned-forward", "y_rot": float_bits(0, "f32"),
                "xxa": float_bits(0, "f32"), "zza": float_bits(1, "f32"),
                "movement": vector_bits(0, 0, 1)},
               {"label": "aligned-yaw-90", "y_rot": float_bits(90, "f32"),
                "xxa": float_bits(0, "f32"), "zza": float_bits(1, "f32"),
                "movement": vector_bits(-1, 0, 0)},
               {"label": "aligned-strafe-yaw-45", "y_rot": float_bits(45, "f32"),
                "xxa": float_bits(1, "f32"), "zza": float_bits(0, "f32"),
                "movement": vector_bits(math.sqrt(0.5), 0, math.sqrt(0.5))},
               {"label": "seven-point-nine-degrees", "y_rot": float_bits(0, "f32"),
                "xxa": float_bits(0, "f32"), "zza": float_bits(1, "f32"),
                "movement": vector_bits(math.sin(math.radians(7.9)), 0,
                                         math.cos(math.radians(7.9)))},
               {"label": "eight-degree-threshold", "y_rot": float_bits(0, "f32"),
                "xxa": float_bits(0, "f32"), "zza": float_bits(1, "f32"),
                "movement": vector_bits(math.sin(math.radians(8.0)), 0,
                                         math.cos(math.radians(8.0)))},
               {"label": "eight-point-one-degrees", "y_rot": float_bits(0, "f32"),
                "xxa": float_bits(0, "f32"), "zza": float_bits(1, "f32"),
                "movement": vector_bits(math.sin(math.radians(8.1)), 0,
                                         math.cos(math.radians(8.1)))},
               {"label": "reverse-movement", "y_rot": float_bits(0, "f32"),
                "xxa": float_bits(0, "f32"), "zza": float_bits(1, "f32"),
                "movement": vector_bits(0, 0, -1)},
               {"label": "zero-input", "y_rot": float_bits(0, "f32"),
                "xxa": float_bits(0, "f32"), "zza": float_bits(0, "f32"),
                "movement": vector_bits(0, 0, 1)},
               {"label": "zero-movement", "y_rot": float_bits(0, "f32"),
                "xxa": float_bits(0, "f32"), "zza": float_bits(1, "f32"),
                "movement": vector_bits(0, 0, 0)},
               {"label": "subthreshold-input", "y_rot": float_bits(0, "f32"),
                "xxa": float_bits(0, "f32"), "zza": float_bits(0.001, "f32"),
                "movement": vector_bits(0, 0, 1)},
           ]}
    yield {"case_id": "shape/collect-block-collisions/empty", "family": "math",
           "operation": "shape", "kind": "collect_block_collisions",
           "box": box_bits(-0.25, -0.25, -0.25, 0.25, 0.25, 0.25),
           "placements": [], "shape_states": []}
    yield {"case_id": "shape/collect-block-collisions/full-and-slabs", "family": "math",
           "operation": "shape", "kind": "collect_block_collisions",
           "box": box_bits(0.25, 0.25, 0.25, 2.75, 1.75, 1.75),
           "placements": [
               {"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"},
               {"x": 1, "y": 0, "z": 0, "state_id": 2, "kind": "slab_bottom"},
               {"x": 1, "y": 1, "z": 0, "state_id": 3, "kind": "slab_top"},
               {"x": 2, "y": 0, "z": 0, "state_id": 4, "kind": "slab_double"},
           ],
           "shape_states": [
               {"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)},
               {"state_id": 2, "shape": box_bits(0, 0, 0, 1, 0.5, 1)},
               {"state_id": 3, "shape": box_bits(0, 0.5, 0, 1, 1, 1)},
               {"state_id": 4, "shape": box_bits(0, 0, 0, 1, 1, 1)},
           ]}
    yield {"case_id": "shape/collect-block-collisions/touching-faces", "family": "math",
           "operation": "shape", "kind": "collect_block_collisions",
           "box": box_bits(0, 0, 0, 1, 1, 1),
           "placements": [
               {"x": 0, "y": 0, "z": 0, "state_id": 2, "kind": "slab_bottom"},
               {"x": 1, "y": 0, "z": 0, "state_id": 1, "kind": "stone"},
               {"x": 0, "y": 1, "z": 0, "state_id": 1, "kind": "stone"},
               {"x": 0, "y": 0, "z": 1, "state_id": 1, "kind": "stone"},
           ],
           "shape_states": [
               {"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)},
               {"state_id": 2, "shape": box_bits(0, 0, 0, 1, 0.5, 1)},
           ]}
    yield {"case_id": "shape/collect-block-collisions/chunk-boundary", "family": "math",
           "operation": "shape", "kind": "collect_block_collisions",
           "box": box_bits(14.25, 0.25, 0.25, 17.75, 1.25, 1.25),
           "placements": [
               {"x": 15, "y": 0, "z": 0, "state_id": 1, "kind": "stone"},
               {"x": 16, "y": 0, "z": 0, "state_id": 3, "kind": "slab_top"},
           ],
           "shape_states": [
               {"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)},
               {"state_id": 3, "shape": box_bits(0, 0.5, 0, 1, 1, 1)},
           ]}
    yield {"case_id": "shape/step-up-candidates/empty", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 2, 0, 1, 3, 1), "max_step_height": float_bits(0.6),
           "step_height_to_skip": float_bits(0.0), "shapes": []}
    yield {"case_id": "shape/step-up-candidates/sorted-unique", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 2, 0, 1, 3, 1), "max_step_height": float_bits(0.75),
           "step_height_to_skip": float_bits(-1.0),
           "shapes": [box_bits(0, 2.5, 0, 1, 2.75, 1),
                      box_bits(0, 2.0, 0, 1, 2.5, 1),
                      box_bits(0, 2.25, 0, 1, 2.5, 1)]}
    yield {"case_id": "shape/step-up-candidates/negative-skip", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 10, 0, 1, 11, 1), "max_step_height": float_bits(0.5),
           "step_height_to_skip": float_bits(0.25),
           "shapes": [box_bits(0, 9, 0, 1, 10.25, 1),
                      box_bits(0, 10.125, 0, 1, 10.5, 1),
                      box_bits(0, 10.375, 0, 1, 11, 1)]}
    yield {"case_id": "shape/step-up-candidates/zero-skipped-and-max-break", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 0, 0, 1, 1, 1), "max_step_height": float_bits(0.5),
           "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(0, 0, 0, 1, 0.75, 1),
                      box_bits(0, 0.25, 0, 1, 1, 1)]}
    yield {"case_id": "shape/step-up-candidates/signed-zero-sort", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 0, 0, 1, 3, 1), "max_step_height": float_bits(2.0),
           "step_height_to_skip": float_bits(3.0),
           "shapes": [box_bits(0, -0.0, 0, 1, 2, 1),
                      box_bits(0, 0.0, 0, 1, 2, 1)]}
    yield {"case_id": "shape/step-up-selection/low-obstacle", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
           "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
           "movement": vector_bits(2, 0, 0), "movement_step": vector_bits(0, 0, 0),
           "max_step_height": float_bits(0.6), "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(1, 0, 0, 2, 0.5, 1)]}
    yield {"case_id": "shape/step-up-selection/first-better-candidate-wins", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
           "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
           "movement": vector_bits(3, 0, 0), "movement_step": vector_bits(0, 0, 0),
           "max_step_height": float_bits(0.75), "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(1, 0, 0, 2, 0.25, 1),
                      box_bits(2.5, 0, 0, 3.5, 0.5, 1)]}
    yield {"case_id": "shape/step-up-selection/equal-horizontal-keeps-step", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
           "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
           "movement": vector_bits(2, 0, 0), "movement_step": vector_bits(2, 0, 0),
           "max_step_height": float_bits(0.6), "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(1, 0, 0, 2, 0.5, 1)]}
    yield {"case_id": "shape/step-up-selection/ground-offset", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 1, 0, 1, 2, 1),
           "grounded_box": box_bits(0, 0.5, 0, 1, 1.5, 1),
           "movement": vector_bits(2, -1, 0), "movement_step": vector_bits(0, -0.5, 0),
           "max_step_height": float_bits(0.6), "step_height_to_skip": float_bits(-0.5),
           "shapes": [box_bits(1, 0.5, 0, 2, 1, 1)]}
    yield {"case_id": "shape/step-up-selection/no-candidate-falls-back", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
           "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
           "movement": vector_bits(2, 0, 0), "movement_step": vector_bits(0, 0, 0),
           "max_step_height": float_bits(0.6), "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(1, 0, 0, 2, 1, 1)]}
    yield {"case_id": "shape/entity-fit/empty-default", "family": "math",
           "operation": "shape", "kind": "entity_fit",
           "position": vector_bits(0.5, 0.0, 0.5),
           "dimensions": {"width": float_bits(0.6), "height": float_bits(1.8)},
           "placements": [],
           "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}]}
    yield {"case_id": "shape/entity-fit/scaled-and-eye-height", "family": "math",
           "operation": "shape", "kind": "entity_fit",
           "position": vector_bits(-1.25, 2.0, 3.75),
           "dimensions": {"width": float_bits(0.6), "height": float_bits(1.8),
                          "width_scale": float_bits(0.5), "height_scale": float_bits(2.0),
                          "eye_height": float_bits(2.75)},
           "placements": [],
           "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}]}
    yield {"case_id": "shape/entity-fit/fixed-ignores-scale", "family": "math",
           "operation": "shape", "kind": "entity_fit",
           "position": vector_bits(2.0, 0.0, -2.0),
           "dimensions": {"width": float_bits(1.0), "height": float_bits(2.0),
                          "fixed": True, "width_scale": float_bits(0.5),
                          "height_scale": float_bits(0.5)},
           "placements": [],
           "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}]}
    yield {"case_id": "shape/entity-fit/overlap", "family": "math",
           "operation": "shape", "kind": "entity_fit",
           "position": vector_bits(0.5, 0.25, 0.5),
           "dimensions": {"width": float_bits(0.6), "height": float_bits(1.0)},
           "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"}],
           "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}]}
    yield {"case_id": "shape/entity-fit/touching-face-after-deflate", "family": "math",
           "operation": "shape", "kind": "entity_fit",
           "position": vector_bits(1.5, 0.25, 0.5),
           "dimensions": {"width": float_bits(1.0), "height": float_bits(1.0)},
           "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"}],
           "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}]}
    yield {"case_id": "shape/entity-fit/epsilon-intrusion", "family": "math",
           "operation": "shape", "kind": "entity_fit",
           "position": vector_bits(1.4999997, 0.25, 0.5),
           "dimensions": {"width": float_bits(1.0), "height": float_bits(1.0)},
           "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"}],
           "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}]}
    yield {"case_id": "shape/living-entity-dimensions/scale-branches", "family": "math",
           "operation": "shape", "kind": "living_entity_dimensions_scale", "inputs": [
               {"label": "regular-age-then-entity-scale", "pose_id": 0,
                "position": vector_bits(-1.25, 2.0, 3.75),
                "default_dimensions": {"width": float_bits(0.6), "height": float_bits(1.8),
                                        "eye_height": float_bits(1.62)},
                "age_scale": float_bits(0.5), "entity_scale": float_bits(1.5)},
               {"label": "sleeping-ignores-both-scales", "pose_id": 2,
                "position": vector_bits(4.0, -2.0, -3.0),
                "default_dimensions": {"width": float_bits(0.6), "height": float_bits(1.8),
                                        "eye_height": float_bits(1.62)},
                "age_scale": float_bits(0.5), "entity_scale": float_bits(2.0)},
               {"label": "fixed-default-ignores-entity-scale", "pose_id": 5,
                "position": vector_bits(0.25, 1.0, -0.75),
                "default_dimensions": {"width": float_bits(1.0), "height": float_bits(2.0),
                                        "eye_height": float_bits(1.7), "fixed": True},
                "age_scale": float_bits(0.5), "entity_scale": float_bits(2.0)},
               {"label": "regular-height-only-rounding", "pose_id": 3,
                "position": vector_bits(2.5, 0.0, 1.25),
                "default_dimensions": {"width": float_bits(0.75), "height": float_bits(1.25),
                                        "eye_height": float_bits(0.9)},
                "age_scale": float_bits(0.7), "entity_scale": float_bits(1.3)},
           ]}
    yield {"case_id": "shape/entity-dimensions-refresh/client-reapply", "family": "math",
           "operation": "shape", "kind": "entity_dimensions_refresh", "inputs": [
               {"label": "client-growth-reapplies-box", "position": vector_bits(-1.25, 2.0, 3.75),
                "old_dimensions": {"width": float_bits(0.6), "height": float_bits(1.5),
                                   "eye_height": float_bits(1.27)},
                "new_dimensions": {"width": float_bits(0.6), "height": float_bits(1.8),
                                   "eye_height": float_bits(1.62)},
                "level_client_side": True, "first_tick": False, "no_physics": False,
                "is_player": False},
               {"label": "client-shrink-to-sleeping", "position": vector_bits(4.0, -2.0, -3.0),
                "old_dimensions": {"width": float_bits(0.6), "height": float_bits(1.8),
                                   "eye_height": float_bits(1.62)},
                "new_dimensions": {"width": float_bits(0.2), "height": float_bits(0.2),
                                   "eye_height": float_bits(0.2), "fixed": True},
                "level_client_side": True, "first_tick": False, "no_physics": False,
                "is_player": False},
               {"label": "server-growth-eligible", "position": vector_bits(0.25, 1.0, -0.75),
                "old_dimensions": {"width": float_bits(0.6), "height": float_bits(1.5),
                                   "eye_height": float_bits(1.27)},
                "new_dimensions": {"width": float_bits(0.6), "height": float_bits(1.8),
                                   "eye_height": float_bits(1.62)},
                "level_client_side": False, "first_tick": False, "no_physics": False,
                "is_player": False},
               {"label": "server-player-excluded", "position": vector_bits(2.5, 0.0, 1.25),
                "old_dimensions": {"width": float_bits(0.6), "height": float_bits(1.5),
                                   "eye_height": float_bits(1.27)},
                "new_dimensions": {"width": float_bits(0.6), "height": float_bits(1.8),
                                   "eye_height": float_bits(1.62)},
                "level_client_side": False, "first_tick": False, "no_physics": False,
                "is_player": True},
               {"label": "server-first-tick-gate", "position": vector_bits(-2.0, 3.0, 0.5),
                "old_dimensions": {"width": float_bits(0.6), "height": float_bits(1.5),
                                   "eye_height": float_bits(1.27)},
                "new_dimensions": {"width": float_bits(0.6), "height": float_bits(1.8),
                                   "eye_height": float_bits(1.62)},
                "level_client_side": False, "first_tick": True, "no_physics": False,
                "is_player": False},
               {"label": "server-no-physics-gate", "position": vector_bits(1.5, -1.0, 2.25),
                "old_dimensions": {"width": float_bits(0.6), "height": float_bits(1.5),
                                   "eye_height": float_bits(1.27)},
                "new_dimensions": {"width": float_bits(0.6), "height": float_bits(1.8),
                                   "eye_height": float_bits(1.62)},
                "level_client_side": False, "first_tick": False, "no_physics": True,
                "is_player": False},
               {"label": "server-large-dimensions-gate", "position": vector_bits(0.0, 0.0, 0.0),
                "old_dimensions": {"width": float_bits(0.6), "height": float_bits(1.5),
                                   "eye_height": float_bits(1.27)},
                "new_dimensions": {"width": float_bits(4.1), "height": float_bits(1.8),
                                   "eye_height": float_bits(1.62)},
                "level_client_side": False, "first_tick": False, "no_physics": False,
                "is_player": False},
           ]}
    yield {"case_id": "shape/player-abilities/clientbound-state", "family": "math",
           "operation": "shape", "kind": "player_abilities_packet", "inputs": [
               {"label": "defaults", "hex": "003d4ccccd3dcccccd"},
               {"label": "invulnerable-only", "hex": "013d4ccccd3dcccccd"},
               {"label": "flying-and-can-fly", "hex": "063e4ccccd3d4ccccd"},
               {"label": "instabuild-only", "hex": "083d4ccccd3dcccccd"},
               {"label": "all-flags-and-unknown-bit", "hex": "ff3e4ccccd3e99999a"},
               {"label": "raw-speed-bits", "hex": "02800000007fc00001"},
           ]}
    yield {"case_id": "shape/player-game-mode/change-game-mode", "family": "math",
           "operation": "shape", "kind": "player_game_mode_event", "inputs": [
               {"label": "survival", "hex": "0300000000"},
               {"label": "creative", "hex": "033f800000"},
               {"label": "creative-preserves-existing-flying", "hex": "033f800000",
                "initial_flying": True, "initial_flying_speed": "3f000000",
                "initial_walking_speed": "3e800000"},
               {"label": "adventure", "hex": "0340000000"},
               {"label": "spectator", "hex": "0340400000"},
               {"label": "rounds-to-creative", "hex": "033fbeb852"},
               {"label": "exact-half-rounds-up", "hex": "033fc00000"},
               {"label": "high-id-falls-back-to-survival", "hex": "0340666666"},
               {"label": "negative-id-falls-back-to-survival", "hex": "03bf028f5c"},
           ]}
    yield {"case_id": "shape/player-pose-dimensions/all-poses-and-fallback", "family": "math",
           "operation": "shape", "kind": "player_pose_dimensions",
           "position": vector_bits(-2.25, 4.0, 1.75),
           "pose_ids": list(range(18)) + [-1, 18]}
    yield {"case_id": "shape/player-pose-selection/priority-fit-fallbacks", "family": "math",
           "operation": "shape", "kind": "player_pose_selection", "inputs": [
               {"label": "sleeping-priority", "sleeping": True, "swimming": True,
                "fall_flying": True, "auto_spin_attack": True, "shift_key_down": True,
                "flying": False, "spectator": False, "passenger": False,
                "swimming_fits": True, "desired_fits": True, "crouching_fits": True},
               {"label": "swimming-priority", "sleeping": False, "swimming": True,
                "fall_flying": True, "auto_spin_attack": True, "shift_key_down": True,
                "flying": False, "spectator": False, "passenger": False,
                "swimming_fits": True, "desired_fits": True, "crouching_fits": True},
               {"label": "fall-flying-priority", "sleeping": False, "swimming": False,
                "fall_flying": True, "auto_spin_attack": True, "shift_key_down": False,
                "flying": False, "spectator": False, "passenger": False,
                "swimming_fits": True, "desired_fits": True, "crouching_fits": True},
               {"label": "spin-attack-priority", "sleeping": False, "swimming": False,
                "fall_flying": False, "auto_spin_attack": True, "shift_key_down": False,
                "flying": False, "spectator": False, "passenger": False,
                "swimming_fits": True, "desired_fits": True, "crouching_fits": True},
               {"label": "crouching", "sleeping": False, "swimming": False,
                "fall_flying": False, "auto_spin_attack": False, "shift_key_down": True,
                "flying": False, "spectator": False, "passenger": False,
                "swimming_fits": True, "desired_fits": True, "crouching_fits": True},
               {"label": "flying-clears-crouch", "sleeping": False, "swimming": False,
                "fall_flying": False, "auto_spin_attack": False, "shift_key_down": True,
                "flying": True, "spectator": False, "passenger": False,
                "swimming_fits": True, "desired_fits": True, "crouching_fits": True},
               {"label": "standing-default", "sleeping": False, "swimming": False,
                "fall_flying": False, "auto_spin_attack": False, "shift_key_down": False,
                "flying": False, "spectator": False, "passenger": False,
                "swimming_fits": True, "desired_fits": True, "crouching_fits": True},
               {"label": "swimming-fit-gate", "sleeping": False, "swimming": False,
                "fall_flying": False, "auto_spin_attack": False, "shift_key_down": False,
                "flying": False, "spectator": False, "passenger": False,
                "swimming_fits": False, "desired_fits": False, "crouching_fits": False},
               {"label": "fallback-to-crouching", "sleeping": False, "swimming": False,
                "fall_flying": False, "auto_spin_attack": False, "shift_key_down": False,
                "flying": False, "spectator": False, "passenger": False,
                "swimming_fits": True, "desired_fits": False, "crouching_fits": True},
               {"label": "fallback-to-swimming", "sleeping": False, "swimming": False,
                "fall_flying": False, "auto_spin_attack": False, "shift_key_down": False,
                "flying": False, "spectator": False, "passenger": False,
                "swimming_fits": True, "desired_fits": False, "crouching_fits": False},
               {"label": "spectator-bypass", "sleeping": False, "swimming": False,
                "fall_flying": False, "auto_spin_attack": False, "shift_key_down": False,
                "flying": False, "spectator": True, "passenger": False,
                "swimming_fits": True, "desired_fits": False, "crouching_fits": False},
               {"label": "passenger-bypass", "sleeping": False, "swimming": False,
                "fall_flying": False, "auto_spin_attack": False, "shift_key_down": False,
                "flying": False, "spectator": False, "passenger": True,
                "swimming_fits": True, "desired_fits": False, "crouching_fits": False},
           ]}
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

    mth_trigonometric_values = [
        ("zero", float_bits(0.0, "f64")),
        ("negative-zero", "8000000000000000"),
        ("quarter-turn", float_bits(math.pi / 2.0, "f64")),
        ("negative-quarter-turn", float_bits(-math.pi / 2.0, "f64")),
        ("pi", float_bits(math.pi, "f64")),
        ("negative-pi", float_bits(-math.pi, "f64")),
        ("large-finite", float_bits(1.0e300, "f64")),
        ("positive-infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("nan-payload", "7ff8000000000001"),
    ]
    for function in ("sin", "cos"):
        for label, value in mth_trigonometric_values:
            yield {"case_id": f"mth/{function}/{label}", "family": "math",
                   "operation": "mth_floating_primitives", "fn": function,
                   "type": "f64", "value": value}

    mth_sqrt_values = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("one", float_bits(1.0, "f32")),
        ("two", float_bits(2.0, "f32")),
        ("min-subnormal", "00000001"),
        ("max-finite", "7f7fffff"),
        ("negative-one", float_bits(-1.0, "f32")),
        ("positive-infinity", "7f800000"),
        ("negative-infinity", "ff800000"),
        ("nan-payload", "7fc00001"),
    ]
    for label, value in mth_sqrt_values:
        yield {"case_id": f"mth/sqrt/{label}", "family": "math",
               "operation": "mth_floating_primitives", "fn": "sqrt",
               "type": "f32", "value": value}

    mth_abs_values = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("one", float_bits(1.0, "f32")),
        ("negative-one", float_bits(-1.0, "f32")),
        ("min-subnormal", "00000001"),
        ("negative-min-subnormal", "80000001"),
        ("negative-max-finite", "ff7fffff"),
        ("positive-infinity", "7f800000"),
        ("negative-infinity", "ff800000"),
        ("negative-nan-payload", "ffc01234"),
    ]
    for label, value in mth_abs_values:
        yield {"case_id": f"mth/abs-f32/{label}", "family": "math",
               "operation": "mth_floating_primitives", "fn": "abs",
               "type": "f32", "value": value}

    mth_clamp_f32 = [
        ("inside", 0.25, 0.0, 1.0),
        ("below", -1.0, 0.0, 1.0),
        ("above", 2.0, 0.0, 1.0),
        ("min-equal", 0.0, 0.0, 1.0),
        ("max-equal", 1.0, 0.0, 1.0),
        ("reversed-bounds", 0.0, 1.0, -1.0),
        ("negative-zero-value", "80000000", "00000000", "3f800000"),
        ("negative-zero-max", "00000000", "bf800000", "80000000"),
        ("nan-value", "7fc01234", "00000000", "3f800000"),
        ("nan-max", "3f800000", "00000000", "7fc01234"),
    ]
    for label, value, minimum, maximum in mth_clamp_f32:
        yield {"case_id": f"mth/clamp-f32/{label}", "family": "math",
               "operation": "mth_floating_primitives", "fn": "clamp", "type": "f32",
               "value": value if isinstance(value, str) else float_bits(value, "f32"),
               "min": minimum if isinstance(minimum, str) else float_bits(minimum, "f32"),
               "max": maximum if isinstance(maximum, str) else float_bits(maximum, "f32")}

    mth_clamp_f64 = [
        ("inside", 0.25, 0.0, 1.0),
        ("below", -1.0, 0.0, 1.0),
        ("above", 2.0, 0.0, 1.0),
        ("min-equal", 0.0, 0.0, 1.0),
        ("max-equal", 1.0, 0.0, 1.0),
        ("reversed-bounds", 0.0, 1.0, -1.0),
        ("negative-zero-value", "8000000000000000", "0000000000000000", "3ff0000000000000"),
        ("negative-zero-max", "0000000000000000", "bff0000000000000", "8000000000000000"),
        ("nan-value", "7ff8000000001234", "0000000000000000", "3ff0000000000000"),
        ("nan-max", "3ff0000000000000", "0000000000000000", "7ff8000000001234"),
    ]
    for label, value, minimum, maximum in mth_clamp_f64:
        yield {"case_id": f"mth/clamp-f64/{label}", "family": "math",
               "operation": "mth_floating_primitives", "fn": "clamp", "type": "f64",
               "value": value if isinstance(value, str) else float_bits(value, "f64"),
               "min": minimum if isinstance(minimum, str) else float_bits(minimum, "f64"),
               "max": maximum if isinstance(maximum, str) else float_bits(maximum, "f64")}

    mth_power_helpers = [
        ("smallest-power/zero", "i32", "smallest_encompassing_power_of_two", "0"),
        ("smallest-power/one", "i32", "smallest_encompassing_power_of_two", "1"),
        ("smallest-power/two", "i32", "smallest_encompassing_power_of_two", "2"),
        ("smallest-power/three", "i32", "smallest_encompassing_power_of_two", "3"),
        ("smallest-power/four", "i32", "smallest_encompassing_power_of_two", "4"),
        ("smallest-power/fifteen", "i32", "smallest_encompassing_power_of_two", "15"),
        ("smallest-power/sixteen", "i32", "smallest_encompassing_power_of_two", "16"),
        ("smallest-power/seventeen", "i32", "smallest_encompassing_power_of_two", "17"),
        ("smallest-power/max", "i32", "smallest_encompassing_power_of_two", "2147483647"),
        ("smallest-power/min", "i32", "smallest_encompassing_power_of_two", "-2147483648"),
        ("smallest-square/zero", "i32", "smallest_square_side", "0"),
        ("smallest-square/one", "i32", "smallest_square_side", "1"),
        ("smallest-square/two", "i32", "smallest_square_side", "2"),
        ("smallest-square/three", "i32", "smallest_square_side", "3"),
        ("smallest-square/four", "i32", "smallest_square_side", "4"),
        ("smallest-square/eight", "i32", "smallest_square_side", "8"),
        ("smallest-square/fifteen", "i32", "smallest_square_side", "15"),
        ("smallest-square/sixteen", "i32", "smallest_square_side", "16"),
        ("smallest-square/seventeen", "i32", "smallest_square_side", "17"),
        ("smallest-square/max", "i32", "smallest_square_side", "2147483647"),
        ("is-power-i32/zero", "i32", "is_power_of_two", "0"),
        ("is-power-i32/one", "i32", "is_power_of_two", "1"),
        ("is-power-i32/two", "i32", "is_power_of_two", "2"),
        ("is-power-i32/three", "i32", "is_power_of_two", "3"),
        ("is-power-i32/four", "i32", "is_power_of_two", "4"),
        ("is-power-i32/five", "i32", "is_power_of_two", "5"),
        ("is-power-i32/high-power", "i32", "is_power_of_two", "1073741824"),
        ("is-power-i32/max", "i32", "is_power_of_two", "2147483647"),
        ("is-power-i32/min", "i32", "is_power_of_two", "-2147483648"),
        ("is-power-i32/negative-one", "i32", "is_power_of_two", "-1"),
        ("is-power-i64/zero", "i64", "is_power_of_two", "0"),
        ("is-power-i64/one", "i64", "is_power_of_two", "1"),
        ("is-power-i64/two", "i64", "is_power_of_two", "2"),
        ("is-power-i64/three", "i64", "is_power_of_two", "3"),
        ("is-power-i64/four", "i64", "is_power_of_two", "4"),
        ("is-power-i64/five", "i64", "is_power_of_two", "5"),
        ("is-power-i64/high-power", "i64", "is_power_of_two", "4611686018427387904"),
        ("is-power-i64/max", "i64", "is_power_of_two", "9223372036854775807"),
        ("is-power-i64/min", "i64", "is_power_of_two", "-9223372036854775808"),
        ("is-power-i64/negative-one", "i64", "is_power_of_two", "-1"),
        ("ceillog2/zero", "i32", "ceillog2", "0"),
        ("ceillog2/one", "i32", "ceillog2", "1"),
        ("ceillog2/two", "i32", "ceillog2", "2"),
        ("ceillog2/three", "i32", "ceillog2", "3"),
        ("ceillog2/four", "i32", "ceillog2", "4"),
        ("ceillog2/five", "i32", "ceillog2", "5"),
        ("ceillog2/seven", "i32", "ceillog2", "7"),
        ("ceillog2/eight", "i32", "ceillog2", "8"),
        ("ceillog2/high-power", "i32", "ceillog2", "1073741824"),
        ("ceillog2/max", "i32", "ceillog2", "2147483647"),
        ("log2/zero", "i32", "log2", "0"),
        ("log2/one", "i32", "log2", "1"),
        ("log2/two", "i32", "log2", "2"),
        ("log2/three", "i32", "log2", "3"),
        ("log2/four", "i32", "log2", "4"),
        ("log2/five", "i32", "log2", "5"),
        ("log2/seven", "i32", "log2", "7"),
        ("log2/eight", "i32", "log2", "8"),
        ("log2/high-power", "i32", "log2", "1073741824"),
        ("log2/max", "i32", "log2", "2147483647"),
    ]
    for label, width, function, input_value in mth_power_helpers:
        yield {"case_id": f"mth/power-helpers/{label}", "family": "math",
               "operation": "mth_power_helpers", "type": width, "fn": function,
               "input": input_value}

    mth_frac_helpers = [
        ("f32/zero", "f32", float_bits(0.0, "f32")),
        ("f32/negative-zero", "f32", "80000000"),
        ("f32/positive-fraction", "f32", float_bits(1.25, "f32")),
        ("f32/negative-fraction", "f32", float_bits(-1.25, "f32")),
        ("f32/positive-exact", "f32", float_bits(7.0, "f32")),
        ("f32/negative-exact", "f32", float_bits(-7.0, "f32")),
        ("f32/positive-subnormal", "f32", float_bits(float.fromhex("0x0.000002p-126"), "f32")),
        ("f32/negative-subnormal", "f32", float_bits(-float.fromhex("0x0.000002p-126"), "f32")),
        ("f32/int-upper-overflow", "f32", float_bits(2147483648.0, "f32")),
        ("f32/infinity", "f32", float_bits(float("inf"), "f32")),
        ("f32/negative-infinity", "f32", float_bits(-float("inf"), "f32")),
        ("f32/nan", "f32", float_bits(float("nan"), "f32")),
        ("f64/zero", "f64", float_bits(0.0, "f64")),
        ("f64/negative-zero", "f64", "8000000000000000"),
        ("f64/positive-fraction", "f64", float_bits(1.25, "f64")),
        ("f64/negative-fraction", "f64", float_bits(-1.25, "f64")),
        ("f64/positive-exact", "f64", float_bits(7.0, "f64")),
        ("f64/negative-exact", "f64", float_bits(-7.0, "f64")),
        ("f64/positive-subnormal", "f64", float_bits(float.fromhex("0x0.0000000000001p-1022"), "f64")),
        ("f64/long-upper-overflow", "f64", float_bits(2.0 ** 63, "f64")),
        ("f64/long-lower-bound", "f64", float_bits(-(2.0 ** 63), "f64")),
        ("f64/infinity", "f64", float_bits(float("inf"), "f64")),
        ("f64/negative-infinity", "f64", float_bits(-float("inf"), "f64")),
        ("f64/nan", "f64", float_bits(float("nan"), "f64")),
    ]
    for label, width, input_value in mth_frac_helpers:
        yield {"case_id": f"mth/frac/{label}", "family": "math",
               "operation": "mth_frac", "type": width, "fn": "frac",
               "input": input_value}

    for label, seed in (
        ("zero", 0),
        ("one", 1),
        ("negative-one", -1),
        ("positive", 42),
        ("negative", -42),
        ("large-positive", 987654321),
        ("large-negative", -987654321),
        ("long-max", 9223372036854775807),
        ("long-min", -9223372036854775808),
        ("alternating-positive", 305419896),
        ("alternating-negative", -6148914691236517206),
        ("high-bit", 4611686018427387904),
    ):
        yield {"case_id": f"mth/create-insecure-uuid/{label}", "family": "math",
               "operation": "mth_create_insecure_uuid", "seed": str(seed)}

    for label, input_value in (
        ("zero", 0),
        ("one", 1),
        ("negative-one", -1),
        ("i32-max", 2147483647),
        ("i32-min", -2147483648),
        ("high-bit-plus-one", -2147483647),
        ("low-31-bit", 1073741824),
        ("alternating-positive", 1431655765),
        ("alternating-negative", -1431655766),
        ("byte-alternating-positive", 16711935),
        ("byte-alternating-negative", -16711936),
        ("ascending-hex", 305419896),
        ("descending-hex", -2023406815),
        ("shift-16", 65536),
        ("shift-13", 8192),
        ("shift-1", 2),
        ("near-high-bit", 1073741823),
        ("near-low-bit", -1073741825),
        ("mixed-positive", 19088743),
        ("mixed-negative", -19088744),
        ("large-positive", 987654321),
        ("large-negative", -987654321),
        ("signed-pattern", -305419896),
        ("all-zero-high-byte", 16777215),
    ):
        yield {"case_id": f"mth/murmur-hash3-mixer/{label}", "family": "math",
               "operation": "mth_murmur_hash3_mixer", "input": str(input_value)}

    for label, from_value, to_value, condition, threshold in (
        ("empty", 0, 0, "ge", 0),
        ("single-true", 0, 1, "ge", 0),
        ("single-false", 0, 1, "ge", 1),
        ("single-gt-true", 0, 1, "gt", -1),
        ("basic-ge", 0, 16, "ge", 7),
        ("basic-gt", 0, 16, "gt", 7),
        ("threshold-at-to", -10, 10, "ge", 10),
        ("all-true", -10, 10, "ge", -20),
        ("negative-ge", -20, -5, "ge", -12),
        ("negative-gt", -20, -5, "gt", -12),
        ("cross-zero-ge", -10, 11, "ge", -1),
        ("cross-zero-gt", -10, 11, "gt", -1),
        ("offset-range", 5, 25, "ge", 13),
        ("descending-range", 25, 5, "ge", 13),
        ("max-empty", 2147483647, 2147483647, "ge", 2147483647),
        ("min-empty", -2147483648, -2147483648, "ge", -2147483648),
        ("full-range-overflow", -2147483648, 2147483647, "ge", 0),
        ("max-to-min-wrap", 2147483647, -2147483648, "gt", 2147483647),
        ("high-wrap-two", 2147483646, -2147483648, "gt", 2147483647),
        ("low-range-all-true", -2147483648, -2147483640, "ge", -2147483644),
        ("high-range-ge", 2147483630, 2147483647, "ge", 2147483640),
        ("high-range-gt", 2147483630, 2147483647, "gt", 2147483640),
        ("low-range-ge", -2147483648, -2147483630, "ge", -2147483640),
        ("low-range-gt", -2147483648, -2147483630, "gt", -2147483640),
    ):
        yield {"case_id": f"mth/binary-search/{label}", "family": "math",
               "operation": "mth_binary_search", "from": str(from_value),
               "to": str(to_value), "condition": condition, "threshold": str(threshold)}

    ray_intersects_aabb = [
        ("inside-positive-x", vector_bits(0.5, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("inside-negative-x", vector_bits(0.5, 0.5, 0.5), vector_bits(-1.0, -0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-left-toward", vector_bits(-1.0, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-left-away", vector_bits(-1.0, 0.5, 0.5), vector_bits(-1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-right-toward", vector_bits(2.0, 0.5, 0.5), vector_bits(-1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-right-away", vector_bits(2.0, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-below-toward", vector_bits(0.5, -1.0, 0.5), vector_bits(0.1, 1.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-above-away", vector_bits(0.5, 2.0, 0.5), vector_bits(0.1, 1.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-front-toward", vector_bits(0.5, 0.5, -1.0), vector_bits(0.0, 0.1, 1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("outside-back-away", vector_bits(0.5, 0.5, 2.0), vector_bits(0.0, 0.1, 1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("parallel-x-miss", vector_bits(2.0, 0.5, 0.5), vector_bits(0.0, 1.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("parallel-negative-zero-x", vector_bits(2.0, 0.5, 0.5), vector_bits(-0.0, 1.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("diagonal-hit", vector_bits(-1.0, -1.0, -1.0), vector_bits(1.0, 1.0, 1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("diagonal-skew-miss", vector_bits(-1.0, -1.0, 2.0), vector_bits(1.0, 1.0, -1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("tangent-top-edge", vector_bits(-1.0, 1.0, 0.5), vector_bits(1.0, 0.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("tangent-corner", vector_bits(-1.0, 1.0, 1.0), vector_bits(1.0, 0.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("boundary-outward", vector_bits(0.5, 0.5, 0.0), vector_bits(0.0, 0.1, -1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("boundary-inward", vector_bits(0.5, 0.5, 0.0), vector_bits(0.0, 0.1, 1.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("near-axis-hit", vector_bits(-1.0, 0.5, 0.5), vector_bits(1.0, 0.000001, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("zero-direction-inside", vector_bits(0.5, 0.5, 0.5), vector_bits(0.0, 0.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("zero-direction-outside", vector_bits(2.0, 0.5, 0.5), vector_bits(0.0, 0.0, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("large-coordinates", vector_bits(1000000000.0, 0.5, 0.5),
         vector_bits(-1.0, 0.1, 0.0), box_bits(999999999.0, 0.0, 0.0, 1000000001.0, 1.0, 1.0)),
        ("reversed-box", vector_bits(-1.0, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(1.0, 1.0, 1.0, 0.0, 0.0, 0.0)),
        ("point-box", vector_bits(-1.0, 0.0, 0.0), vector_bits(1.0, 0.1, 0.1),
         box_bits(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
        ("negative-box", vector_bits(-3.0, -2.5, -2.0), vector_bits(1.0, 0.2, 0.1),
         box_bits(-2.0, -2.0, -1.0, -1.0, -1.0, 0.0)),
        ("nan-start", vector_bits(float("nan"), 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("nan-direction", vector_bits(-1.0, 0.5, 0.5), vector_bits(float("nan"), 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("infinite-direction", vector_bits(-1.0, 0.5, 0.5), vector_bits(float("inf"), 0.1, 0.0),
         box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)),
        ("infinite-box", vector_bits(-1.0, 0.5, 0.5), vector_bits(1.0, 0.1, 0.0),
         box_bits(float("-inf"), 0.0, 0.0, float("inf"), 1.0, 1.0)),
    ]
    for label, ray_start, ray_dir, aabb in ray_intersects_aabb:
        yield {"case_id": f"mth/ray-intersects-aabb/{label}", "family": "math",
               "operation": "mth_ray_intersects_aabb", "ray_start": ray_start,
               "ray_dir": ray_dir, "aabb": aabb}

    mth_fast_inv_sqrt = [
        ("zero", float_bits(0.0, "f64")),
        ("negative-zero", "8000000000000000"),
        ("one", float_bits(1.0, "f64")),
        ("one-below", "3fefffffffffffff"),
        ("one-above", "3ff0000000000001"),
        ("two", float_bits(2.0, "f64")),
        ("quarter", float_bits(0.25, "f64")),
        ("four", float_bits(4.0, "f64")),
        ("pi", float_bits(math.pi, "f64")),
        ("large", float_bits(1.0e20, "f64")),
        ("max", "7fefffffffffffff"),
        ("min-normal", "0010000000000000"),
        ("positive-subnormal", "0000000000000001"),
        ("negative-one", float_bits(-1.0, "f64")),
        ("negative-two", float_bits(-2.0, "f64")),
        ("negative-max", "ffefffffffffffff"),
        ("negative-min-normal", "8010000000000000"),
        ("negative-subnormal", "8000000000000001"),
        ("infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("nan", "7ff8000000000000"),
        ("nan-payload", "7ff0000000000001"),
        ("negative-nan-payload", "fff0000000000001"),
        ("tiny-neighbor", "0000000000000002"),
    ]
    for label, input_value in mth_fast_inv_sqrt:
        yield {"case_id": f"mth/fast-inv-sqrt/{label}", "family": "math",
               "operation": "mth_fast_inv_sqrt", "input": input_value}

    mth_fast_inv_cube_root = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("one", "3f800000"),
        ("one-below", "3f7fffff"),
        ("one-above", "3f800001"),
        ("eight", "41000000"),
        ("eighth", "3e000000"),
        ("twenty-seven", "41d80000"),
        ("pi", float_bits(math.pi, "f32")),
        ("large", float_bits(1.0e20, "f32")),
        ("max", "7f7fffff"),
        ("min-normal", "00800000"),
        ("positive-subnormal", "00000001"),
        ("negative-one", "bf800000"),
        ("negative-eight", "c1000000"),
        ("negative-max", "ff7fffff"),
        ("negative-min-normal", "80800000"),
        ("negative-subnormal", "80000001"),
        ("infinity", "7f800000"),
        ("negative-infinity", "ff800000"),
        ("nan", "7fc00000"),
        ("nan-payload", "7fc12345"),
        ("negative-nan-payload", "ffc12345"),
        ("tiny-neighbor", "00000002"),
    ]
    for label, input_value in mth_fast_inv_cube_root:
        yield {"case_id": f"mth/fast-inv-cube-root/{label}", "family": "math",
               "operation": "mth_fast_inv_cube_root", "input": input_value}

    mth_hsv_to_color = [
        ("red-sector", 0.0, 1.0, 1.0, 255),
        ("yellow-sector", 1.0 / 6.0, 1.0, 1.0, 128),
        ("green-sector", 1.0 / 3.0, 1.0, 1.0, 64),
        ("cyan-sector", 0.5, 1.0, 1.0, 32),
        ("blue-sector", 2.0 / 3.0, 1.0, 1.0, 16),
        ("magenta-sector", 5.0 / 6.0, 1.0, 1.0, 8),
        ("wrapped-one", 1.0, 1.0, 1.0, 255),
        ("boundary-below", math.nextafter(1.0 / 6.0, 0.0), 1.0, 1.0, 255),
        ("zero-saturation", 0.5, 0.0, 0.5, 17),
        ("fractional", 0.23, 0.61, 0.73, 200),
        ("small-negative-hue", -0.1, 1.0, 1.0, -1),
        ("wrapped-positive-hue", 1.2, 1.0, 1.0, 256),
        ("negative-sector", -0.2, 1.0, 1.0, 0),
        ("nan-hue", "7fc00000", 0.5, 0.5, 42),
        ("positive-infinity-hue", "7f800000", 0.5, 0.5, 42),
        ("negative-infinity-hue", "ff800000", 0.5, 0.5, 42),
        ("nan-saturation", 0.25, "7fc00000", 0.75, 255),
        ("nan-value", 0.25, 0.5, "7fc00000", 255),
        ("outside-components", 0.75, -0.5, 1.5, 256),
        ("negative-value", 0.75, 0.5, -1.0, -128),
        ("signed-zero", "80000000", "80000000", "80000000", 0),
    ]
    for label, hue, saturation, value, alpha in mth_hsv_to_color:
        hue = hue if isinstance(hue, str) else float_bits(hue, "f32")
        saturation = saturation if isinstance(saturation, str) else float_bits(saturation, "f32")
        value = value if isinstance(value, str) else float_bits(value, "f32")
        for mode in ("rgb", "argb"):
            yield {"case_id": f"mth/hsv-to-color/{label}/{mode}", "family": "math",
                   "operation": "mth_hsv_to_color", "mode": mode, "hue": hue,
                   "saturation": saturation, "value": value, "alpha": str(alpha)}

    mth_wrap_degrees_90 = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("positive-boundary-below", "4233ffff"),
        ("positive-boundary", "42340000"),
        ("positive-boundary-above", "42340001"),
        ("negative-boundary-above", "c233ffff"),
        ("negative-boundary", "c2340000"),
        ("negative-boundary-below", "c2340001"),
        ("positive-quarter-turn", float_bits(90.0, "f32")),
        ("negative-quarter-turn", float_bits(-90.0, "f32")),
        ("positive-half-turn", float_bits(180.0, "f32")),
        ("negative-half-turn", float_bits(-180.0, "f32")),
        ("wrapped-positive", float_bits(360.25, "f32")),
        ("wrapped-negative", float_bits(-360.25, "f32")),
        ("positive-subnormal", "00000001"),
        ("negative-subnormal", "80000001"),
        ("positive-infinity", "7f800000"),
        ("nan-payload", "7fc12345"),
    ]
    for label, value in mth_wrap_degrees_90:
        yield {"case_id": f"mth/angle-wrap-difference/wrap-degrees-90/{label}", "family": "math",
               "operation": "mth_angle_wrap_difference", "type": "f32",
               "fn": "wrap_degrees_90", "value": value}

    mth_degrees_difference = [
        ("same", float_bits(10.0, "f32"), float_bits(10.0, "f32")),
        ("signed-zero", "80000000", "00000000"),
        ("simple-positive", float_bits(10.0, "f32"), float_bits(40.0, "f32")),
        ("simple-negative", float_bits(40.0, "f32"), float_bits(10.0, "f32")),
        ("upper-bound", float_bits(0.0, "f32"), float_bits(180.0, "f32")),
        ("lower-bound", float_bits(0.0, "f32"), float_bits(-180.0, "f32")),
        ("cross-positive", float_bits(170.0, "f32"), float_bits(-170.0, "f32")),
        ("cross-negative", float_bits(-170.0, "f32"), float_bits(170.0, "f32")),
        ("whole-turn", float_bits(10.0, "f32"), float_bits(370.0, "f32")),
        ("fractional", float_bits(-12.5, "f32"), float_bits(289.25, "f32")),
        ("large-finite", float_bits(1.0e10, "f32"), float_bits(-1.0e10, "f32")),
        ("positive-infinity", float_bits(0.0, "f32"), "7f800000"),
        ("negative-infinity", float_bits(0.0, "f32"), "ff800000"),
        ("nan-from", "7fc12345", float_bits(1.0, "f32")),
        ("nan-to", float_bits(1.0, "f32"), "7fc12345"),
        ("overflow", "ff7fffff", "7f7fffff"),
    ]
    for label, from_value, to_value in mth_degrees_difference:
        yield {"case_id": f"mth/angle-wrap-difference/degrees-difference/{label}", "family": "math",
               "operation": "mth_angle_wrap_difference", "type": "f32",
               "fn": "degrees_difference", "from": from_value, "to": to_value}

    mth_degrees_difference_abs = [
        ("zero", float_bits(0.0, "f32"), float_bits(0.0, "f32")),
        ("negative-zero", "80000000", "00000000"),
        ("simple-positive", float_bits(10.0, "f32"), float_bits(40.0, "f32")),
        ("simple-negative", float_bits(40.0, "f32"), float_bits(10.0, "f32")),
        ("upper-bound", float_bits(0.0, "f32"), float_bits(180.0, "f32")),
        ("lower-bound", float_bits(0.0, "f32"), float_bits(-180.0, "f32")),
        ("cross-positive", float_bits(170.0, "f32"), float_bits(-170.0, "f32")),
        ("cross-negative", float_bits(-170.0, "f32"), float_bits(170.0, "f32")),
        ("whole-turn", float_bits(10.0, "f32"), float_bits(370.0, "f32")),
        ("fractional", float_bits(-12.5, "f32"), float_bits(289.25, "f32")),
        ("negative-result-zero", "80000000", "80000000"),
        ("positive-infinity", float_bits(0.0, "f32"), "7f800000"),
        ("nan-from", "7fc12345", float_bits(1.0, "f32")),
        ("nan-to", float_bits(1.0, "f32"), "7fc12345"),
        ("overflow", "ff7fffff", "7f7fffff"),
        ("subnormal", "00000001", "80000001"),
    ]
    for label, from_value, to_value in mth_degrees_difference_abs:
        yield {"case_id": f"mth/angle-wrap-difference/degrees-difference-abs/{label}", "family": "math",
               "operation": "mth_angle_wrap_difference", "type": "f32",
               "fn": "degrees_difference_abs", "from": from_value, "to": to_value}

    mth_rotate_if_necessary = [
        ("same", 10.0, 10.0, 5.0),
        ("positive-under", 10.0, 12.0, 5.0),
        ("positive-boundary", 10.0, 15.0, 5.0),
        ("positive-over", 10.0, 20.0, 5.0),
        ("negative-under", 20.0, 18.0, 5.0),
        ("negative-boundary", 20.0, 15.0, 5.0),
        ("negative-over", 20.0, 10.0, 5.0),
        ("cross-positive", 170.0, -170.0, 30.0),
        ("cross-negative", -170.0, 170.0, 30.0),
        ("upper-bound", 0.0, 180.0, 45.0),
        ("lower-bound", 0.0, -180.0, 45.0),
        ("zero-limit", 10.0, 80.0, 0.0),
        ("large-limit", 10.0, 80.0, 100.0),
        ("negative-limit", 10.0, 20.0, -5.0),
        ("signed-zero", "80000000", "00000000", 1.0),
        ("positive-infinity-target", 0.0, "7f800000", 30.0),
        ("nan-base", "7fc12345", 0.0, 30.0),
        ("overflow", "7f7fffff", "ff7fffff", "7f7fffff"),
    ]
    for label, base, target, maximum in mth_rotate_if_necessary:
        base = base if isinstance(base, str) else float_bits(base, "f32")
        target = target if isinstance(target, str) else float_bits(target, "f32")
        maximum = maximum if isinstance(maximum, str) else float_bits(maximum, "f32")
        yield {"case_id": f"mth/angle-approach/rotate-if-necessary/{label}", "family": "math",
               "operation": "mth_angle_approach", "type": "f32", "fn": "rotate_if_necessary",
               "base": base, "target": target, "max": maximum}

    mth_approach = [
        ("same", 10.0, 10.0, 2.0),
        ("below", 0.0, 10.0, 2.0),
        ("exact", 0.0, 10.0, 10.0),
        ("over", 0.0, 10.0, 20.0),
        ("above", 10.0, 0.0, 2.0),
        ("negative-increment", 0.0, 10.0, -2.0),
        ("zero-increment", 0.0, 10.0, 0.0),
        ("signed-zero", "80000000", "00000000", 1.0),
        ("positive-subnormal", 0.0, "00000001", 1.0),
        ("negative-subnormal", "80000001", 0.0, 1.0),
        ("max-finite-current", "7f7fffff", 0.0, 1.0),
        ("infinity-increment", 0.0, 10.0, "7f800000"),
        ("infinity-current", "7f800000", 10.0, 1.0),
        ("infinity-target", 0.0, "7f800000", 1.0),
        ("nan-current", "7fc12345", 0.0, 1.0),
        ("nan-target", 0.0, "7fc12345", 1.0),
        ("nan-increment", 0.0, 10.0, "7fc12345"),
        ("rounding", "3f800001", "3f800101", "33800000"),
    ]
    for label, current, target, increment in mth_approach:
        current = current if isinstance(current, str) else float_bits(current, "f32")
        target = target if isinstance(target, str) else float_bits(target, "f32")
        increment = increment if isinstance(increment, str) else float_bits(increment, "f32")
        yield {"case_id": f"mth/angle-approach/approach/{label}", "family": "math",
               "operation": "mth_angle_approach", "type": "f32", "fn": "approach",
               "current": current, "target": target, "increment": increment}

    mth_approach_degrees = [
        ("same", 10.0, 10.0, 5.0),
        ("simple-positive", 10.0, 40.0, 5.0),
        ("simple-negative", 40.0, 10.0, 5.0),
        ("cross-positive", 170.0, -170.0, 5.0),
        ("cross-negative", -170.0, 170.0, 5.0),
        ("upper-bound", 0.0, 180.0, 10.0),
        ("lower-bound", 0.0, -180.0, 10.0),
        ("whole-turn", 10.0, 370.0, 5.0),
        ("fractional", -12.5, 289.25, 7.0),
        ("negative-increment", 0.0, 90.0, -10.0),
        ("zero-increment", 0.0, 90.0, 0.0),
        ("signed-zero", "80000000", "00000000", 1.0),
        ("positive-subnormal", 0.0, "00000001", "00000001"),
        ("large-finite", 1.0e10, -1.0e10, 1.0e5),
        ("infinity-increment", 0.0, 90.0, "7f800000"),
        ("infinity-target", 0.0, "7f800000", 1.0),
        ("nan-current", "7fc12345", 0.0, 1.0),
        ("nan-target", 0.0, "7fc12345", 1.0),
    ]
    for label, current, target, increment in mth_approach_degrees:
        current = current if isinstance(current, str) else float_bits(current, "f32")
        target = target if isinstance(target, str) else float_bits(target, "f32")
        increment = increment if isinstance(increment, str) else float_bits(increment, "f32")
        yield {"case_id": f"mth/angle-approach/approach-degrees/{label}", "family": "math",
               "operation": "mth_angle_approach", "type": "f32", "fn": "approach_degrees",
               "current": current, "target": target, "increment": increment}

    mth_get_int = [
        ("null-zero-default", None, "0"),
        ("null-negative-default", None, "-2147483648"),
        ("empty", "", "7"),
        ("decimal", "42", "7"),
        ("negative", "-42", "7"),
        ("positive-sign", "+42", "7"),
        ("leading-zeroes", "00042", "7"),
        ("negative-zero", "-0", "7"),
        ("minimum", "-2147483648", "7"),
        ("maximum", "2147483647", "-7"),
        ("underflow", "-2147483649", "11"),
        ("overflow", "2147483648", "11"),
        ("positive-overflow", "+2147483648", "11"),
        ("leading-space", " 42", "11"),
        ("trailing-space", "42 ", "11"),
        ("tab-prefix", "\t42", "11"),
        ("internal-space", "4 2", "11"),
        ("decimal-point", "42.0", "11"),
        ("hex-prefix", "0x2a", "11"),
        ("underscore", "1_000", "11"),
        ("plus-only", "+", "11"),
        ("minus-only", "-", "11"),
        ("arabic-indic", "٤٢", "11"),
        ("fullwidth", "４２", "11"),
        ("devanagari", "४२", "11"),
        ("unicode-minus", "−42", "11"),
        ("newline-suffix", "42\n", "11"),
        ("invalid-max-default", "invalid", "2147483647"),
        ("invalid-min-default", "invalid", "-2147483648"),
    ]
    for label, input_value, default in mth_get_int:
        yield {"case_id": f"mth/get-int/{label}", "family": "math",
               "operation": "mth_get_int", "input": input_value, "default": default}

    mth_get_seed = [
        ("zero", 0, 0, 0),
        ("simple-positive", 1, 2, 3),
        ("simple-negative", -1, -2, -3),
        ("mixed-signs", -123, 456, -789),
        ("x-only", 123456, 0, 0),
        ("y-only", 0, -789, 0),
        ("z-only", 0, 0, 654321),
        ("x-min", -2147483648, 0, 0),
        ("x-max", 2147483647, 0, 0),
        ("y-min", 0, -2147483648, 0),
        ("y-max", 0, 2147483647, 0),
        ("z-min", 0, 0, -2147483648),
        ("z-max", 0, 0, 2147483647),
        ("all-min", -2147483648, -2147483648, -2147483648),
        ("all-max", 2147483647, 2147483647, 2147483647),
        ("alternating-boundaries", -2147483648, 2147483647, -2147483648),
        ("reversed-boundaries", 2147483647, -2147483648, 2147483647),
        ("x-product-within-i32", 686, 0, 0),
        ("x-product-wraps-positive", 687, 0, 0),
        ("x-product-wraps-negative", -687, 0, 0),
        ("large-mixed-positive", 1000000000, -1000000000, 1000000000),
        ("large-mixed-negative", -1000000000, 1000000000, -1000000000),
        ("hash-like", 123456789, -987654321, 135792468),
        ("section-like", -2097152, 319, 2097151),
    ]
    for label, x, y, z in mth_get_seed:
        yield {"case_id": f"mth/get-seed/{label}", "family": "math",
               "operation": "mth_get_seed", "x": x, "y": y, "z": z}

    mth_atan2 = [
        ("zero-zero", 0.0, 0.0),
        ("negative-zero-y", -0.0, 0.0),
        ("negative-zero-x", 0.0, -0.0),
        ("both-negative-zero", -0.0, -0.0),
        ("positive-x-axis", 0.0, 1.0),
        ("negative-x-axis", 0.0, -1.0),
        ("positive-y-axis", 1.0, 0.0),
        ("negative-y-axis", -1.0, 0.0),
        ("first-quadrant", 1.0, 1.0),
        ("second-quadrant", 1.0, -1.0),
        ("third-quadrant", -1.0, -1.0),
        ("fourth-quadrant", -1.0, 1.0),
        ("shallow-positive", 0.25, 1.0),
        ("steep-positive", 1.0, 0.25),
        ("shallow-negative-y", -0.25, 1.0),
        ("steep-negative-y", -1.0, 0.25),
        ("table-boundary-low", math.nextafter(1.0 / 256.0, 0.0), 1.0),
        ("table-boundary", 1.0 / 256.0, 1.0),
        ("table-boundary-high", math.nextafter(1.0 / 256.0, math.inf), 1.0),
        ("steep-boundary-low", 1.0, math.nextafter(1.0, 0.0)),
        ("steep-boundary-high", 1.0, math.nextafter(1.0, math.inf)),
        ("tiny-y", float.fromhex("0x0.0000000000001p-1022"), 1.0),
        ("both-min-subnormal", float.fromhex("0x0.0000000000001p-1022"),
         float.fromhex("0x0.0000000000001p-1022")),
        ("mixed-exponents", 2.0 ** -500, 2.0 ** 500),
        ("large-finite", 1.0e200, -1.0e200),
        ("maximum-y", 1.0e308, 0.0),
        ("positive-infinity-x", 1.0, float("inf")),
        ("negative-infinity-x", 1.0, float("-inf")),
        ("positive-infinity-y", float("inf"), 1.0),
        ("negative-infinity-y", float("-inf"), 1.0),
        ("both-positive-infinity", float("inf"), float("inf")),
        ("nan-y", float("nan"), 1.0),
        ("nan-x", 1.0, float("nan")),
        ("nan-y-payload", "7ff0000000000001", 1.0),
    ]
    for label, y, x in mth_atan2:
        y = y if isinstance(y, str) else float_bits(y, "f64")
        x = x if isinstance(x, str) else float_bits(x, "f64")
        yield {"case_id": f"mth/atan2/{label}", "family": "math",
               "operation": "mth_atan2", "y": y, "x": x}

    mth_inv_sqrt_f32 = [
        ("zero", "00000000"),
        ("negative-zero", "80000000"),
        ("one", "3f800000"),
        ("two", "40000000"),
        ("quarter", "3e800000"),
        ("maximum-finite", "7f7fffff"),
        ("minimum-normal", "00800000"),
        ("positive-subnormal", "00000001"),
        ("negative-one", "bf800000"),
        ("negative-subnormal", "80000001"),
        ("infinity", "7f800000"),
        ("negative-infinity", "ff800000"),
        ("nan", "7fc00000"),
        ("nan-payload", "7fc12345"),
        ("negative-nan-payload", "ffc12345"),
    ]
    for label, input_value in mth_inv_sqrt_f32:
        yield {"case_id": f"mth/inv-sqrt/f32/{label}", "family": "math",
               "operation": "mth_inv_sqrt", "type": "f32", "input": input_value}

    mth_inv_sqrt_f64 = [
        ("zero", "0000000000000000"),
        ("negative-zero", "8000000000000000"),
        ("one", "3ff0000000000000"),
        ("two", "4000000000000000"),
        ("quarter", "3fd0000000000000"),
        ("maximum-finite", "7fefffffffffffff"),
        ("minimum-normal", "0010000000000000"),
        ("positive-subnormal", "0000000000000001"),
        ("negative-one", "bff0000000000000"),
        ("negative-subnormal", "8000000000000001"),
        ("infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("nan", "7ff8000000000000"),
        ("nan-payload", "7ff0000000000001"),
        ("negative-nan-payload", "fff0000000000001"),
    ]
    for label, input_value in mth_inv_sqrt_f64:
        yield {"case_id": f"mth/inv-sqrt/f64/{label}", "family": "math",
               "operation": "mth_inv_sqrt", "type": "f64", "input": input_value}

    mth_integer_helpers = [
        ("floor-div/int/negative-remainder", "i32", "floor_div", "-7", "3"),
        ("floor-div/int/negative-divisor", "i32", "floor_div", "7", "-3"),
        ("floor-div/int/min-overflow", "i32", "floor_div", "-2147483648", "-1"),
        ("positive-modulo/int/negative-remainder", "i32", "positive_modulo", "-7", "3"),
        ("positive-modulo/int/negative-divisor", "i32", "positive_modulo", "7", "-3"),
        ("positive-modulo/int/min-overflow", "i32", "positive_modulo", "-2147483648", "-1"),
        ("positive-ceil-div/int/positive-remainder", "i32", "positive_ceil_div", "7", "3"),
        ("positive-ceil-div/int/negative-input", "i32", "positive_ceil_div", "-7", "3"),
        ("positive-ceil-div/int/min-input", "i32", "positive_ceil_div", "-2147483648", "3"),
        ("positive-ceil-div/long/min-input", "i64", "positive_ceil_div", "-9223372036854775808", "3"),
        ("round-toward/int/positive-remainder", "i32", "round_toward", "7", "3"),
        ("round-toward/int/negative-input", "i32", "round_toward", "-7", "3"),
        ("round-toward/int/multiplication-overflow", "i32", "round_toward", "2147483647", "2"),
        ("round-toward/long/negative-input", "i64", "round_toward", "-7", "3"),
        ("round-toward/long/multiplication-overflow", "i64", "round_toward", "9223372036854775807", "2"),
    ]
    for label, width, function, input_value, divisor in mth_integer_helpers:
        yield {"case_id": f"mth/integer-helpers/{label}", "family": "math",
               "operation": "mth_integer_helpers", "type": width, "fn": function,
               "input": input_value, "divisor": divisor}

    mth_integer_basics = [
        ("abs/min", "i32", "abs", "-2147483648", None, None, None),
        ("abs/negative", "i32", "abs", "-1", None, None, None),
        ("abs/zero", "i32", "abs", "0", None, None, None),
        ("abs/positive", "i32", "abs", "1", None, None, None),
        ("abs/max", "i32", "abs", "2147483647", None, None, None),
        ("clamp-i32/below", "i32", "clamp", "-5", "0", "10", None),
        ("clamp-i32/inside", "i32", "clamp", "5", "0", "10", None),
        ("clamp-i32/above", "i32", "clamp", "15", "0", "10", None),
        ("clamp-i32/full-range", "i32", "clamp", "-2147483648", "-2147483648", "2147483647", None),
        ("clamp-i32/reversed-bounds", "i32", "clamp", "5", "10", "-10", None),
        ("clamp-i64/min", "i64", "clamp", "-9223372036854775808", "-9223372036854775808", "9223372036854775807", None),
        ("clamp-i64/below", "i64", "clamp", "-20", "-10", "10", None),
        ("clamp-i64/above", "i64", "clamp", "20", "-10", "10", None),
        ("clamp-i64/reversed-bounds", "i64", "clamp", "5", "10", "-10", None),
        ("multiple/positive", "i32", "is_multiple", "12", None, None, "3"),
        ("multiple/remainder", "i32", "is_multiple", "13", None, None, "3"),
        ("multiple/negative", "i32", "is_multiple", "-12", None, None, "3"),
        ("multiple/min-overflow", "i32", "is_multiple", "-2147483648", None, None, "-1"),
        ("wrap/min", "i32", "wrap_degrees", "-2147483648", None, None, None),
        ("wrap/negative", "i32", "wrap_degrees", "-721", None, None, None),
        ("wrap/exact-negative-turn", "i32", "wrap_degrees", "-360", None, None, None),
        ("wrap/lower-bound", "i32", "wrap_degrees", "-180", None, None, None),
        ("wrap/just-above-lower-bound", "i32", "wrap_degrees", "-179", None, None, None),
        ("wrap/upper-bound", "i32", "wrap_degrees", "180", None, None, None),
        ("wrap/max", "i32", "wrap_degrees", "2147483647", None, None, None),
    ]
    for label, width, function, input_value, min_value, max_value, divisor in mth_integer_basics:
        case = {"case_id": f"mth/integer-basics/{label}", "family": "math",
                "operation": "mth_integer_basics", "type": width, "fn": function,
                "input": input_value}
        if min_value is not None:
            case.update({"min": min_value, "max": max_value})
        if divisor is not None:
            case["divisor"] = divisor
        yield case

    mth_angle_helpers = [
        ("pack/zero", "f32", "pack_degrees", float_bits(0.0, "f32")),
        ("pack/negative-quarter", "f32", "pack_degrees", float_bits(-90.0, "f32")),
        ("pack/negative-half", "f32", "pack_degrees", float_bits(-180.0, "f32")),
        ("pack/quarter", "f32", "pack_degrees", float_bits(90.0, "f32")),
        ("pack/half-byte-wrap", "f32", "pack_degrees", float_bits(180.0, "f32")),
        ("pack/full-byte-wrap", "f32", "pack_degrees", float_bits(360.0, "f32")),
        ("pack/near-full-turn", "f32", "pack_degrees", float_bits(359.9, "f32")),
        ("pack/infinity", "f32", "pack_degrees", float_bits(float("inf"), "f32")),
        ("pack/nan", "f32", "pack_degrees", float_bits(float("nan"), "f32")),
        ("unpack/min", "i8", "unpack_degrees", "-128"),
        ("unpack/negative", "i8", "unpack_degrees", "-1"),
        ("unpack/zero", "i8", "unpack_degrees", "0"),
        ("unpack/positive", "i8", "unpack_degrees", "1"),
        ("unpack/max", "i8", "unpack_degrees", "127"),
        ("wrap-long/min", "i64", "wrap_degrees", "-9223372036854775808"),
        ("wrap-long/negative", "i64", "wrap_degrees", "-721"),
        ("wrap-long/exact-negative-turn", "i64", "wrap_degrees", "-360"),
        ("wrap-long/lower-bound", "i64", "wrap_degrees", "-180"),
        ("wrap-long/upper-bound", "i64", "wrap_degrees", "180"),
        ("wrap-long/exact-turn", "i64", "wrap_degrees", "360"),
        ("wrap-long/max", "i64", "wrap_degrees", "9223372036854775807"),
    ]
    for label, width, function, input_value in mth_angle_helpers:
        yield {"case_id": f"mth/angle-helpers/{label}", "family": "math",
               "operation": "mth_angle_helpers", "type": width, "fn": function,
               "input": input_value}

    mth_rounding_helpers = [
        ("ceil-f32/zero", "f32", "ceil", float_bits(0.0, "f32")),
        ("ceil-f32/negative-zero", "f32", "ceil", float_bits(-0.0, "f32")),
        ("ceil-f32/positive-fraction", "f32", "ceil", float_bits(1.25, "f32")),
        ("ceil-f32/negative-fraction", "f32", "ceil", float_bits(-1.25, "f32")),
        ("ceil-f32/exact", "f32", "ceil", float_bits(7.0, "f32")),
        ("ceil-f32/lower-bound", "f32", "ceil", float_bits(-2147483648.0, "f32")),
        ("ceil-f32/upper-overflow", "f32", "ceil", float_bits(2147483648.0, "f32")),
        ("ceil-f32/infinity", "f32", "ceil", float_bits(float("inf"), "f32")),
        ("ceil-f32/negative-infinity", "f32", "ceil", float_bits(-float("inf"), "f32")),
        ("ceil-f32/nan", "f32", "ceil", float_bits(float("nan"), "f32")),
        ("ceil-f64/zero", "f64", "ceil", float_bits(0.0, "f64")),
        ("ceil-f64/positive-fraction", "f64", "ceil", float_bits(1.25, "f64")),
        ("ceil-f64/negative-fraction", "f64", "ceil", float_bits(-1.25, "f64")),
        ("ceil-f64/upper-bound", "f64", "ceil", float_bits(2147483647.0, "f64")),
        ("ceil-f64/upper-overflow", "f64", "ceil", float_bits(2147483647.0000001, "f64")),
        ("ceil-f64/lower-bound", "f64", "ceil", float_bits(-2147483648.0, "f64")),
        ("ceil-f64/lower-overflow", "f64", "ceil", float_bits(-2147483648.0000001, "f64")),
        ("ceil-f64/infinity", "f64", "ceil", float_bits(float("inf"), "f64")),
        ("ceil-f64/negative-infinity", "f64", "ceil", float_bits(-float("inf"), "f64")),
        ("ceil-f64/nan", "f64", "ceil", float_bits(float("nan"), "f64")),
        ("lfloor/zero", "f64", "lfloor", float_bits(0.0, "f64")),
        ("lfloor/positive-fraction", "f64", "lfloor", float_bits(1.25, "f64")),
        ("lfloor/negative-fraction", "f64", "lfloor", float_bits(-1.25, "f64")),
        ("lfloor/positive-long-overflow", "f64", "lfloor", float_bits(2.0 ** 63, "f64")),
        ("lfloor/negative-long-bound", "f64", "lfloor", float_bits(-(2.0 ** 63), "f64")),
        ("lfloor/infinity", "f64", "lfloor", float_bits(float("inf"), "f64")),
        ("lfloor/negative-infinity", "f64", "lfloor", float_bits(-float("inf"), "f64")),
        ("lfloor/nan", "f64", "lfloor", float_bits(float("nan"), "f64")),
        ("ceil-long/zero", "f64", "ceil_long", float_bits(0.0, "f64")),
        ("ceil-long/positive-fraction", "f64", "ceil_long", float_bits(1.25, "f64")),
        ("ceil-long/negative-fraction", "f64", "ceil_long", float_bits(-1.25, "f64")),
        ("ceil-long/positive-long-overflow", "f64", "ceil_long", float_bits(2.0 ** 63, "f64")),
        ("ceil-long/negative-long-bound", "f64", "ceil_long", float_bits(-(2.0 ** 63), "f64")),
        ("ceil-long/infinity", "f64", "ceil_long", float_bits(float("inf"), "f64")),
        ("ceil-long/negative-infinity", "f64", "ceil_long", float_bits(-float("inf"), "f64")),
        ("ceil-long/nan", "f64", "ceil_long", float_bits(float("nan"), "f64")),
    ]
    for label, width, function, input_value in mth_rounding_helpers:
        yield {"case_id": f"mth/rounding-helpers/{label}", "family": "math",
               "operation": "mth_rounding_helpers", "type": width, "fn": function,
               "input": input_value}

    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_positive_modulo = [
            ("basic-positive", 1.25, 3.0),
            ("negative-input", -1.25, 3.0),
            ("input-over-modulus", 5.5, 3.0),
            ("positive-exact-multiple", 6.0, 3.0),
            ("negative-exact-multiple", -6.0, 3.0),
            ("negative-zero-input", -0.0, 3.0),
            ("negative-modulus", 1.25, -3.0),
            ("both-negative", -1.25, -3.0),
            ("subnormal-input", subnormal, 1.0),
            ("maximum-finite-input", maximum_finite, 3.0),
            ("zero-modulus", 1.0, 0.0),
            ("negative-zero-modulus", 1.0, -0.0),
            ("positive-infinity-input", float("inf"), 3.0),
            ("negative-infinity-input", -float("inf"), 3.0),
            ("infinite-modulus", 1.0, float("inf")),
            ("negative-input-infinite-modulus", -1.0, float("inf")),
            ("negative-infinite-modulus", 1.0, -float("inf")),
            ("negative-input-negative-infinite-modulus", -1.0, -float("inf")),
            ("nan-input", float("nan"), 3.0),
            ("nan-input-negative-modulus", float("nan"), -3.0),
            ("nan-modulus", 1.0, float("nan")),
            ("negative-zero-input-zero-modulus", -0.0, 0.0),
        ]
        for label, input_value, modulus in mth_positive_modulo:
            yield {"case_id": f"mth/positive-modulo/{width}/{label}", "family": "math",
                   "operation": "mth_positive_modulo", "type": width,
                   "input": float_bits(input_value, width), "mod": float_bits(modulus, width)}

    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_abs_max = [
            ("basic", -3.0, 2.0),
            ("second-larger", -2.0, 4.5),
            ("both-negative", -7.25, -4.5),
            ("same-magnitude", -3.0, 3.0),
            ("zero", 0.0, 0.0),
            ("signed-zero", -0.0, 0.0),
            ("subnormal", subnormal, 0.0),
            ("maximum-finite", -maximum_finite, maximum_finite),
            ("positive-infinity", float("inf"), 3.0),
            ("negative-infinity", -float("inf"), -2.0),
            ("nan-first", float("nan"), 3.0),
            ("negative-nan-first", -float("nan"), 3.0),
            ("nan-second", 3.0, float("nan")),
            ("negative-nan-second", 3.0, -float("nan")),
        ]
        for label, first, second in mth_abs_max:
            yield {"case_id": f"mth/abs-max/{width}/{label}", "family": "math",
                   "operation": "mth_abs_max", "type": width,
                   "first": float_bits(first, width), "second": float_bits(second, width)}

    equal_threshold = struct.unpack(">f", struct.pack(">f", 1.0e-5))[0]
    for width in ("f32", "f64"):
        threshold = equal_threshold
        mth_equal = [
            ("zero", 0.0, 0.0),
            ("signed-zero", -0.0, 0.0),
            ("same-finite", 1.25, 1.25),
            ("within-threshold", 0.0, threshold * 0.5),
            ("negative-within-threshold", 0.0, -threshold * 0.5),
            ("threshold", 0.0, threshold),
            ("above-threshold", 0.0, threshold * 2.0),
            ("subnormal", 0.0, (float.fromhex("0x0.000002p-126") if width == "f32"
                                else float.fromhex("0x0.0000000000001p-1022"))),
            ("large-difference", 1.0e20, 1.0e20 + 1.0e6),
            ("positive-infinity", float("inf"), float("inf")),
            ("negative-infinity", -float("inf"), -float("inf")),
            ("opposite-infinity", float("inf"), -float("inf")),
            ("nan-first", float("nan"), 1.0),
            ("nan-second", 1.0, float("nan")),
        ]
        for label, first, second in mth_equal:
            yield {"case_id": f"mth/equal/{width}/{label}", "family": "math",
                   "operation": "mth_equal", "type": width,
                   "first": float_bits(first, width), "second": float_bits(second, width)}

    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_clamped_lerp = [
            ("negative-factor", -0.5, -2.5, 4.5),
            ("negative-infinity-factor", -float("inf"), -2.5, 4.5),
            ("negative-zero-factor", -0.0, -0.0, 1.0),
            ("zero-factor", 0.0, -3.25, 7.5),
            ("within-factor", 0.25, -4.0, 12.0),
            ("half-factor", 0.5, -4.0, 12.0),
            ("one-factor", 1.0, -4.0, 12.0),
            ("above-factor", 1.5, -4.0, 12.0),
            ("positive-infinity-factor", float("inf"), -4.0, 12.0),
            ("nan-factor", float("nan"), 1.0, 2.0),
            ("reversed-range", 0.25, 10.0, -10.0),
            ("precision-order", 0.1, 1.0e20, 1.0e20 + 1.0e6),
            ("subnormal-range", 0.5, 0.0, subnormal),
            ("maximum-finite-range", 0.5, -maximum_finite, maximum_finite),
            ("infinite-range", 0.5, -float("inf"), float("inf")),
            ("nan-min-out-of-range", -0.5, float("nan"), 2.0),
        ]
        for label, factor, minimum, maximum in mth_clamped_lerp:
            yield {"case_id": f"mth/clamped-lerp/{width}/{label}", "family": "math",
                   "operation": "mth_clamped_lerp", "type": width,
                   "factor": float_bits(factor, width), "min": float_bits(minimum, width),
                   "max": float_bits(maximum, width)}

    mth_next_int = [
        ("normal", 42, -10, 10),
        ("same-bounds", 42, 5, 5),
        ("inverted-bounds", 42, 10, -10),
        ("positive-boundary", 123, 2147483640, 2147483647),
        ("negative-boundary", -123, -2147483648, -2147483641),
        ("single-bit-range", 0, 0, 1),
    ]
    for label, seed, minimum, maximum in mth_next_int:
        yield {"case_id": f"mth/next-random/i32/{label}", "family": "math",
               "operation": "mth_next_random", "type": "i32", "seed": str(seed),
               "min": str(minimum), "max": str(maximum)}

    for width in ("f32", "f64"):
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_next_float = [
            ("normal", 42, -1.0, 1.0),
            ("same-bounds", 42, 0.5, 0.5),
            ("inverted-bounds", 42, 1.0, -1.0),
            ("signed-zero", 42, -0.0, 0.0),
            ("wide-finite", 123, -maximum_finite, maximum_finite),
            ("infinite-max", 123, 1.0, float("inf")),
            ("nan-min", 123, float("nan"), 1.0),
        ]
        for label, seed, minimum, maximum in mth_next_float:
            yield {"case_id": f"mth/next-random/{width}/{label}", "family": "math",
                   "operation": "mth_next_random", "type": width, "seed": str(seed),
                   "min": float_bits(minimum, width), "max": float_bits(maximum, width)}

    mth_random_between_inclusive = [
        ("normal", 42, -10, 10),
        ("same-bounds", 42, 5, 5),
        ("positive-boundary", 123, 2147483640, 2147483647),
        ("negative-boundary", -123, -2147483648, -2147483641),
        ("wide-finite", 7, -100, 100),
        ("single-bit-range", 0, 0, 1),
    ]
    for label, seed, minimum, maximum in mth_random_between_inclusive:
        yield {"case_id": f"mth/random-helpers/inclusive/{label}", "family": "math",
               "operation": "mth_random_helpers", "type": "i32", "fn": "inclusive",
               "seed": str(seed), "min": str(minimum), "max": str(maximum)}

    mth_random_between = [
        ("normal", 42, -1.0, 1.0),
        ("same-bounds", 42, 0.5, 0.5),
        ("reversed-bounds", 42, 1.0, -1.0),
        ("signed-zero", 42, -0.0, 0.0),
        ("infinite-max", 123, 1.0, float("inf")),
        ("nan-min", 123, float("nan"), 1.0),
    ]
    for label, seed, minimum, maximum in mth_random_between:
        yield {"case_id": f"mth/random-helpers/between/{label}", "family": "math",
               "operation": "mth_random_helpers", "type": "f32", "fn": "between",
               "seed": str(seed), "min": float_bits(minimum, "f32"),
               "max": float_bits(maximum, "f32")}

    mth_normal = [
        ("normal", 42, 0.0, 1.0),
        ("shifted", 123, 2.5, 0.75),
        ("zero-deviation", 42, 4.0, 0.0),
        ("negative-deviation", -123, 1.0, -2.0),
        ("negative-zero-mean", 42, -0.0, 0.0),
        ("infinite-mean", 7, float("inf"), 1.0),
    ]
    for label, seed, mean, deviation in mth_normal:
        yield {"case_id": f"mth/random-helpers/normal/{label}", "family": "math",
               "operation": "mth_random_helpers", "type": "f32", "fn": "normal",
               "seed": str(seed), "mean": float_bits(mean, "f32"),
               "deviation": float_bits(deviation, "f32")}

    mth_lengths = [
        ("f64-2/squared-basic", "f64_2", "length_squared", (3.0, 4.0)),
        ("f64-2/length-basic", "f64_2", "length", (3.0, 4.0)),
        ("f64-2/squared-signed-zero", "f64_2", "length_squared", (-0.0, 0.0)),
        ("f64-2/length-infinity", "f64_2", "length", (float("inf"), 1.0)),
        ("f64-2/length-nan", "f64_2", "length", (float("nan"), 1.0)),
        ("f32-2/length-basic", "f32_2", "length", (3.0, 4.0)),
        ("f32-2/length-signed-zero", "f32_2", "length", (-0.0, 0.0)),
        ("f32-2/length-overflow", "f32_2", "length",
         (float.fromhex("0x1.fffffep127"), 1.0)),
        ("f64-3/squared-basic", "f64_3", "length_squared", (1.0, 2.0, 2.0)),
        ("f64-3/length-basic", "f64_3", "length", (1.0, 2.0, 2.0)),
        ("f64-3/squared-overflow", "f64_3", "length_squared", (1.0e308, 1.0e-308, 0.0)),
        ("f64-3/length-nan", "f64_3", "length", (float("inf"), 0.0, float("nan"))),
        ("f64-3/length-signed-zero", "f64_3", "length", (-0.0, 0.0, -0.0)),
        ("f32-3/squared-basic", "f32_3", "length_squared", (1.0, 2.0, 2.0)),
        ("f32-3/squared-overflow", "f32_3", "length_squared",
         (float.fromhex("0x1.fffffep127"), float.fromhex("0x1.fffffep127"), 0.0)),
        ("f32-3/squared-signed-zero", "f32_3", "length_squared", (-0.0, 0.0, -0.0)),
    ]
    for label, width, function, values in mth_lengths:
        precision = "f32" if width.startswith("f32") else "f64"
        yield {"case_id": f"mth/length/{label}", "family": "math",
               "operation": "mth_length", "type": width, "fn": function,
               "x": float_bits(values[0], precision), "y": float_bits(values[1], precision),
               **({"z": float_bits(values[2], precision)} if len(values) == 3 else {})}

    mth_quantize = [
        ("positive-fraction", 5.9, 2),
        ("exact-multiple", 6.0, 2),
        ("negative-fraction", -5.9, 2),
        ("negative-resolution", 5.9, -2),
        ("near-integer-rounding", 5.999999999999999, 2),
        ("zero", 0.0, 7),
        ("negative-zero", -0.0, 7),
        ("subnormal", float.fromhex("0x0.0000000000001p-1022"), 3),
        ("positive-infinity", float("inf"), 2),
        ("negative-infinity", -float("inf"), 2),
        ("nan", float("nan"), 2),
        ("above-i32-range", 2147483648.5, 1),
        ("below-i32-range", -2147483649.5, 1),
        ("multiplication-overflow", 4000000000.0, 2),
        ("zero-resolution-positive", 1.5, 0),
        ("zero-resolution-negative", -1.5, 0),
        ("zero-resolution-zero", 0.0, 0),
        ("minimum-resolution", 1.0, -2147483648),
        ("minimum-resolution-overflow", -1.0, -2147483648),
    ]
    for label, value, resolution in mth_quantize:
        yield {"case_id": f"mth/quantize/{label}", "family": "math",
               "operation": "mth_quantize", "value": float_bits(value, "f64"),
               "resolution": str(resolution)}

    mth_out_from_origin = [
        ("default-symmetric", {"mode": "default", "origin": "0", "lower": "-3", "upper": "3"}),
        ("default-negative-symmetric", {"mode": "default", "origin": "-1", "lower": "-4", "upper": "2"}),
        ("default-singleton", {"mode": "default", "origin": "10", "lower": "4", "upper": "4"}),
        ("default-origin-below", {"mode": "default", "origin": "-20", "lower": "2", "upper": "6"}),
        ("default-origin-above", {"mode": "default", "origin": "20", "lower": "-6", "upper": "-2"}),
        ("default-origin-at-lower", {"mode": "default", "origin": "-3", "lower": "-3", "upper": "4"}),
        ("default-origin-at-upper", {"mode": "default", "origin": "5", "lower": "-4", "upper": "5"}),
        ("default-left-asymmetric", {"mode": "default", "origin": "-1", "lower": "-5", "upper": "1"}),
        ("default-right-asymmetric", {"mode": "default", "origin": "1", "lower": "-1", "upper": "5"}),
        ("default-i32-origin-min", {"mode": "default", "origin": str(-(1 << 31)), "lower": "-2", "upper": "2"}),
        ("default-i32-origin-max", {"mode": "default", "origin": str((1 << 31) - 1), "lower": "-2", "upper": "2"}),
        ("step-two-symmetric", {"mode": "step", "origin": "0", "lower": "-6", "upper": "6", "step": "2"}),
        ("step-two-offset", {"mode": "step", "origin": "1", "lower": "-5", "upper": "7", "step": "2"}),
        ("step-three-asymmetric", {"mode": "step", "origin": "2", "lower": "-7", "upper": "8", "step": "3"}),
        ("step-exact-upper", {"mode": "step", "origin": "0", "lower": "-4", "upper": "4", "step": "4"}),
        ("step-exceeds-range", {"mode": "step", "origin": "0", "lower": "-2", "upper": "2", "step": "5"}),
        ("step-clamped-below", {"mode": "step", "origin": "-20", "lower": "2", "upper": "9", "step": "3"}),
        ("step-clamped-above", {"mode": "step", "origin": "20", "lower": "-9", "upper": "2", "step": "3"}),
        ("step-singleton", {"mode": "step", "origin": "0", "lower": "7", "upper": "7", "step": "1"}),
        ("step-i32-max", {"mode": "step", "origin": "0", "lower": "-1", "upper": "1", "step": str((1 << 31) - 1)}),
        ("invalid-reversed", {"mode": "default", "origin": "0", "lower": "4", "upper": "-4"}),
        ("invalid-zero-step", {"mode": "step", "origin": "0", "lower": "-1", "upper": "1", "step": "0"}),
        ("invalid-negative-step", {"mode": "step", "origin": "0", "lower": "-1", "upper": "1", "step": "-2"}),
        ("invalid-both", {"mode": "step", "origin": "0", "lower": "2", "upper": "-2", "step": "0"}),
    ]
    for label, values in mth_out_from_origin:
        yield {"case_id": f"mth/out-from-origin/{label}", "family": "math",
               "operation": "mth_out_from_origin", **values}

    mth_mul_and_truncate = [
        ("zero-numerator", "0", "7", "94"),
        ("unit", "1", "1", "12"),
        ("positive-truncate", "1", "3", "10"),
        ("negative-truncate", "-1", "3", "10"),
        ("negative-factor", "3", "2", "-5"),
        ("both-negative", "-3", "2", "-5"),
        ("non-reduced", "6", "4", "10"),
        ("zero-factor", "123", "7", "0"),
        ("denominator-one-max-numerator", str((1 << 31) - 1), "1", "1"),
        ("numerator-max-overflow", str((1 << 31) - 1), "1", "2"),
        ("numerator-min-overflow", str(-(1 << 31)), "1", "2"),
        ("min-times-negative-one", str(-(1 << 31)), "1", "-1"),
        ("factor-min", "1", "1", str(-(1 << 31))),
        ("product-to-min", str(1 << 30), "1", "2"),
        ("large-denominator", str((1 << 31) - 1), str((1 << 31) - 1), str((1 << 31) - 1)),
        ("negative-numerator-large", str(-(1 << 31)), str((1 << 31) - 1), str((1 << 31) - 1)),
        ("fractional-negative-result", "5", "3", str(-(1 << 31))),
        ("factor-maximum", "3", "7", str((1 << 31) - 1)),
        ("denominator-two", str((1 << 31) - 1), "2", "3"),
        ("mixed-overflow", "123456789", "97", "-987654321"),
    ]
    for label, numerator, denominator, factor in mth_mul_and_truncate:
        yield {"case_id": f"mth/mul-and-truncate/{label}", "family": "math",
               "operation": "mth_mul_and_truncate", "numerator": numerator,
               "denominator": denominator, "factor": factor}

    mth_rotation_around_axis = [
        ("axis-x", (1.0, 0.0, 0.0), (0.5, -0.25, 0.75, 1.0)),
        ("axis-y", (0.0, 1.0, 0.0), (-0.5, 0.25, -0.75, 1.0)),
        ("axis-z", (0.0, 0.0, 1.0), (0.5, 0.25, -0.75, -1.0)),
        ("non-unit-axis", (1.0, 2.0, 3.0), (0.25, -0.5, 0.75, 1.0)),
        ("negative-axis", (-1.0, 2.0, -3.0), (0.25, 0.5, -0.75, -0.5)),
        ("zero-axis", (0.0, 0.0, 0.0), (0.25, -0.5, 0.75, 1.0)),
        ("zero-vector", (1.0, -2.0, 3.0), (0.0, 0.0, 0.0, 1.0)),
        ("zero-scalar", (1.0, 2.0, 3.0), (0.25, -0.5, 0.75, 0.0)),
        ("projection-negative", (1.0, 2.0, 3.0), (-1.0, -2.0, -3.0, 0.5)),
        ("fma-cancellation", (1.0, 1.0, 1.0), (1.0, 0.00000011920928955078125, -1.0, 0.25)),
        ("fma-order", (3.0, -2.0, 0.5), (0.125, 0.25, 0.75, -0.25)),
        ("fractional-axis", (0.125, -0.25, 0.5), (0.75, 0.5, -0.25, 0.125)),
        ("signed-zero", (-0.0, 0.0, -0.0), (-0.0, 0.0, -0.0, -0.0)),
        ("signed-zero-projection", (-0.0, 1.0, 0.0), (0.0, -0.0, 0.0, 1.0)),
        ("subnormal", (float.fromhex("0x0.000002p-126"), 1.0, -1.0),
         (float.fromhex("0x0.000004p-126"), -0.5, 0.25, 0.5)),
        ("tiny-quaternion", (1.0, -1.0, 0.5),
         (float.fromhex("0x0.000002p-126"), float.fromhex("-0x0.000002p-126"), 0.0, 1.0)),
        ("maximum-finite", (float.fromhex("0x1.fffffep127"), 1.0, -1.0),
         (1.0, 0.5, -0.25, 0.75)),
        ("infinite-axis", (float("inf"), 1.0, 0.0), (0.0, 1.0, 0.0, 1.0)),
        ("infinite-rotation", (1.0, 0.0, 0.0), (float("inf"), 1.0, 0.0, 1.0)),
        ("nan-axis", (float("nan"), 1.0, 0.0), (1.0, 0.0, 0.0, 1.0)),
        ("nan-rotation", (1.0, 0.0, 0.0), (float("nan"), 1.0, 0.0, 1.0)),
        ("zero-quaternion", (1.0, 2.0, 3.0), (0.0, 0.0, 0.0, 0.0)),
        ("large-components", (4096.0, -2048.0, 1024.0), (512.0, 256.0, -128.0, 64.0)),
        ("mixed-sign-finite", (-0.75, 1.25, -1.5), (2.5, -3.5, 4.5, -5.5)),
    ]
    for label, axis, rotation in mth_rotation_around_axis:
        yield {"case_id": f"mth/rotation-around-axis/{label}", "family": "math",
               "operation": "mth_rotation_around_axis",
               "axis_x": float_bits(axis[0]), "axis_y": float_bits(axis[1]),
               "axis_z": float_bits(axis[2]), "rotation_x": float_bits(rotation[0]),
               "rotation_y": float_bits(rotation[1]), "rotation_z": float_bits(rotation[2]),
               "rotation_w": float_bits(rotation[3])}

    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_inverse_lerp = [
            ("basic", 1.25, 0.0, 5.0),
            ("below", -1.0, 0.0, 5.0),
            ("above", 7.0, 0.0, 5.0),
            ("negative-range", -2.0, -5.0, 5.0),
            ("reverse-bounds", 2.0, 5.0, -5.0),
            ("exact-min", -2.0, -2.0, 6.0),
            ("exact-max", 6.0, -2.0, 6.0),
            ("signed-zero", -0.0, 0.0, 1.0),
            ("zero-range", 1.0, 1.0, 1.0),
            ("subnormal", 2.0 * subnormal, subnormal, 3.0 * subnormal),
            ("maximum-finite-overflow", maximum_finite, -maximum_finite, maximum_finite),
            ("infinite-value", float("inf"), 0.0, 1.0),
            ("nan-value", float("nan"), 0.0, 1.0),
            ("infinite-bound", 1.0, 0.0, float("inf")),
        ]
        for label, value, minimum, maximum in mth_inverse_lerp:
            yield {"case_id": f"mth/inverse-lerp/{width}/{label}", "family": "math",
                   "operation": "mth_inverse_lerp", "type": width,
                   "value": float_bits(value, width), "min": float_bits(minimum, width),
                   "max": float_bits(maximum, width)}

        mth_lerp = [
            ("basic", 0.25, 0.0, 8.0),
            ("negative-extrapolation", -0.5, 2.0, 10.0),
            ("over-one", 1.5, 2.0, 10.0),
            ("negative-range", 0.25, -4.0, 8.0),
            ("reverse-endpoints", 0.25, 8.0, -4.0),
            ("same-endpoints", 0.75, 3.0, 3.0),
            ("alpha-signed-zero", -0.0, 0.0, 1.0),
            ("negative-zero-start", 0.0, -0.0, 1.0),
            ("subnormal", 2.0 * subnormal, subnormal, 3.0 * subnormal),
            ("maximum-finite-difference", 0.5, -maximum_finite, maximum_finite),
            ("infinite-alpha", float("inf"), 0.0, 1.0),
            ("nan-alpha", float("nan"), 0.0, 1.0),
            ("infinite-endpoint", 0.5, 0.0, float("inf")),
            ("nan-endpoint", 0.5, float("nan"), 1.0),
        ]
        for label, alpha, p0, p1 in mth_lerp:
            yield {"case_id": f"mth/lerp/{width}/{label}", "family": "math",
                   "operation": "mth_lerp", "type": width,
                   "alpha": float_bits(alpha, width), "p0": float_bits(p0, width),
                   "p1": float_bits(p1, width)}

    mth_integer_lerp = [
        ("basic", 0.25, 0, 8),
        ("zero-alpha", 0.0, 5, 17),
        ("one-alpha", 1.0, -4, 12),
        ("over-one", 1.5, 2, 10),
        ("negative-extrapolation", -0.5, 2, 10),
        ("negative-range", 0.25, -12, -4),
        ("signed-zero-alpha", -0.0, 5, 17),
        ("subnormal-alpha", float.fromhex("0x0.000002p-126"), -7, 19),
        ("nan-alpha", float("nan"), 7, 23),
        ("positive-infinity-alpha", float("inf"), 1, 3),
        ("negative-infinity-alpha", -float("inf"), 1, 3),
        ("delta-overflow-half", 0.5, (1 << 31) - 1, -(1 << 31)),
        ("delta-min-half", 0.5, 0, -(1 << 31)),
        ("start-overflow-one", 1.0, (1 << 31) - 1, -(1 << 31)),
    ]
    for operation, path_name in (("mth_lerp_int", "lerp-int"),
                                 ("mth_lerp_discrete", "lerp-discrete")):
        for label, alpha, p0, p1 in mth_integer_lerp:
            yield {"case_id": f"mth/{path_name}/{label}", "family": "math",
                   "operation": operation, "alpha": float_bits(alpha, "f32"),
                   "p0": str(p0), "p1": str(p1)}

    mth_smoothstep = [
        ("zero", 0.0),
        ("one", 1.0),
        ("midpoint", 0.5),
        ("quarter", 0.25),
        ("negative", -0.5),
        ("over-one", 1.5),
        ("signed-zero", -0.0),
        ("subnormal", float.fromhex("0x0.0000000000001p-1022")),
        ("maximum-finite", float.fromhex("0x1.fffffffffffffp1023")),
        ("positive-infinity", float("inf")),
        ("negative-infinity", -float("inf")),
        ("nan", float("nan")),
        ("below-one", math.nextafter(1.0, 0.0)),
        ("above-one", math.nextafter(1.0, math.inf)),
    ]
    for operation, path_name in (("mth_smoothstep", "smoothstep"),
                                 ("mth_smoothstep_derivative", "smoothstep-derivative")):
        for label, value in mth_smoothstep:
            yield {"case_id": f"mth/{path_name}/{label}", "family": "math",
                   "operation": operation, "x": float_bits(value, "f64")}

    mth_sign = [
        ("negative-one", -1.0),
        ("negative-quarter", -0.25),
        ("negative-zero", -0.0),
        ("positive-zero", 0.0),
        ("positive-quarter", 0.25),
        ("positive-one", 1.0),
        ("negative-subnormal", -float.fromhex("0x0.0000000000001p-1022")),
        ("positive-subnormal", float.fromhex("0x0.0000000000001p-1022")),
        ("negative-maximum-finite", -float.fromhex("0x1.fffffffffffffp1023")),
        ("positive-maximum-finite", float.fromhex("0x1.fffffffffffffp1023")),
        ("negative-infinity", -float("inf")),
        ("positive-infinity", float("inf")),
        ("nan", float("nan")),
        ("negative-small", -1.0e-300),
    ]
    for label, value in mth_sign:
        yield {"case_id": f"mth/sign/{label}", "family": "math",
               "operation": "mth_sign", "x": float_bits(value, "f64")}

    for width in ("f32", "f64"):
        subnormal = (float.fromhex("0x0.000002p-126") if width == "f32"
                     else float.fromhex("0x0.0000000000001p-1022"))
        maximum_finite = (float.fromhex("0x1.fffffep127") if width == "f32"
                          else float.fromhex("0x1.fffffffffffffp1023"))
        mth_rot_lerp = [
            ("basic", 0.25, 10.0, 50.0),
            ("zero-alpha", -0.0, 45.0, 90.0),
            ("one-alpha", 1.0, 45.0, 90.0),
            ("negative-alpha", -0.5, 10.0, 50.0),
            ("over-one-alpha", 1.5, 10.0, 50.0),
            ("positive-crossing", 0.5, 170.0, -170.0),
            ("negative-crossing", 0.5, -170.0, 170.0),
            ("positive-half-turn", 1.0, 0.0, 180.0),
            ("negative-half-turn", 1.0, 0.0, -180.0),
            ("full-turn", 0.5, 12.0, 372.0),
            ("signed-zero", 0.5, -0.0, 0.0),
            ("subnormal-difference", 1.0, 0.0, subnormal),
            ("maximum-finite", 0.5, -maximum_finite, maximum_finite),
            ("infinite-to", 0.5, 0.0, float("inf")),
            ("nan-to", 0.5, 0.0, float("nan")),
        ]
        for label, alpha, from_value, to_value in mth_rot_lerp:
            yield {"case_id": f"mth/rot-lerp/{width}/{label}", "family": "math",
                   "operation": "mth_rot_lerp", "type": width,
                   "alpha": float_bits(alpha, width),
                   "from": float_bits(from_value, width),
                   "to": float_bits(to_value, width)}

    mth_rot_lerp_rad = [
        ("basic", 0.25, 0.0, 1.0),
        ("zero-alpha", -0.0, 0.5, 1.0),
        ("one-alpha", 1.0, 0.5, 1.0),
        ("negative-alpha", -0.5, 0.5, 1.0),
        ("over-one-alpha", 1.5, 0.5, 1.0),
        ("positive-crossing", 0.5, 3.0, -3.0),
        ("negative-crossing", 0.5, -3.0, 3.0),
        ("positive-pi-boundary", 1.0, 0.0, math.pi),
        ("negative-pi-boundary", 1.0, 0.0, -math.pi),
        ("full-turn", 0.5, 0.0, math.pi * 2),
        ("signed-zero", 0.5, -0.0, 0.0),
        ("subnormal-difference", 1.0, 0.0, float.fromhex("0x0.000002p-126")),
        ("large-finite", 0.5, -100.0, 100.0),
        ("nan-alpha", float("nan"), 0.0, 1.0),
        ("nan-to", 0.5, 0.0, float("nan")),
    ]
    for label, alpha, from_value, to_value in mth_rot_lerp_rad:
        yield {"case_id": f"mth/rot-lerp-rad/{label}", "family": "math",
               "operation": "mth_rot_lerp_rad",
               "alpha": float_bits(alpha, "f32"),
               "from": float_bits(from_value, "f32"),
               "to": float_bits(to_value, "f32")}

    mth_triangle_wave = [
        ("basic", 0.0, 4.0),
        ("quarter", 1.0, 4.0),
        ("midpoint", 2.0, 4.0),
        ("three-quarter", 3.0, 4.0),
        ("period-boundary", 4.0, 4.0),
        ("negative-index", -1.0, 4.0),
        ("negative-period", 2.0, -4.0),
        ("fractional-period", 0.75, 3.0),
        ("negative-fractional", -0.75, 3.0),
        ("signed-zero-index", -0.0, 4.0),
        ("subnormal-period", 1.0, float.fromhex("0x0.000002p-126")),
        ("maximum-finite-index", float.fromhex("0x1.fffffep127"), 3.0),
        ("zero-period", 1.0, 0.0),
        ("infinite-index", float("inf"), 4.0),
        ("nan-index", float("nan"), 4.0),
    ]
    for label, index, period in mth_triangle_wave:
        yield {"case_id": f"mth/triangle-wave/{label}", "family": "math",
               "operation": "mth_triangle_wave",
               "index": float_bits(index, "f32"),
               "period": float_bits(period, "f32")}

    mth_square_f32 = [
        ("basic", 3.0),
        ("zero", 0.0),
        ("signed-zero", -0.0),
        ("negative-one", -1.0),
        ("negative-fractional", -1.5),
        ("fractional", 1.5),
        ("subnormal-input", float.fromhex("0x0.000002p-126")),
        ("subnormal-result", float.fromhex("0x1.0p-74")),
        ("large-finite", float.fromhex("0x1.0p64")),
        ("maximum-finite", float.fromhex("0x1.fffffep127")),
        ("negative-maximum-finite", -float.fromhex("0x1.fffffep127")),
        ("positive-infinity", float("inf")),
        ("negative-infinity", float("-inf")),
        ("nan", float("nan")),
        ("rounding-boundary", float.fromhex("0x1.000002p0")),
    ]
    for label, x in mth_square_f32:
        yield {"case_id": f"mth/square-f32/{label}", "family": "math",
               "operation": "mth_square_f32", "x": float_bits(x, "f32")}

    mth_cube_f32 = [
        ("basic", 2.0),
        ("zero", 0.0),
        ("signed-zero", -0.0),
        ("negative-one", -1.0),
        ("fractional", 1.5),
        ("negative-fractional", -1.5),
        ("subnormal-input", float.fromhex("0x0.000002p-126")),
        ("subnormal-result", float.fromhex("0x1.0p-49")),
        ("large-finite", float.fromhex("0x1.0p42")),
        ("overflow-finite", float.fromhex("0x1.0p43")),
        ("maximum-finite", float.fromhex("0x1.fffffep127")),
        ("negative-maximum-finite", -float.fromhex("0x1.fffffep127")),
        ("positive-infinity", float("inf")),
        ("negative-infinity", float("-inf")),
        ("nan", float("nan")),
    ]
    for label, x in mth_cube_f32:
        yield {"case_id": f"mth/cube-f32/{label}", "family": "math",
               "operation": "mth_cube_f32", "x": float_bits(x, "f32")}

    mth_square_f64 = [
        ("basic", 3.0),
        ("zero", 0.0),
        ("signed-zero", -0.0),
        ("negative-fractional", -1.5),
        ("fractional", 1.5),
        ("minimum-positive-subnormal", float.fromhex("0x0.0000000000001p-1022")),
        ("subnormal-result", float.fromhex("0x1.0p-537")),
        ("large-finite", float.fromhex("0x1.0p511")),
        ("overflow-finite", float.fromhex("0x1.0p512")),
        ("maximum-finite", float.fromhex("0x1.fffffffffffffp1023")),
        ("negative-maximum-finite", -float.fromhex("0x1.fffffffffffffp1023")),
        ("positive-infinity", float("inf")),
        ("negative-infinity", float("-inf")),
        ("nan", float("nan")),
        ("rounding-boundary", float.fromhex("0x1.0000000000001p0")),
    ]
    for label, x in mth_square_f64:
        yield {"case_id": f"mth/square-f64/{label}", "family": "math",
               "operation": "mth_square_f64", "x": float_bits(x, "f64")}

    mth_square_i32 = [
        ("basic", 3),
        ("zero", 0),
        ("positive-one", 1),
        ("negative-one", -1),
        ("negative", -3),
        ("exact-int-limit", 46340),
        ("positive-overflow", 46341),
        ("negative-overflow", -46341),
        ("power-of-two-wrap", 65536),
        ("negative-power-of-two-wrap", -65536),
        ("maximum", 2147483647),
        ("minimum", -2147483648),
        ("maximum-minus-one", 2147483646),
        ("minimum-plus-one", -2147483647),
        ("mixed", 123456789),
    ]
    for label, x in mth_square_i32:
        yield {"case_id": f"mth/square-i32/{label}", "family": "math",
               "operation": "mth_square_i32", "x": str(x)}

    mth_square_i64 = [
        ("basic", 3),
        ("zero", 0),
        ("positive-one", 1),
        ("negative-one", -1),
        ("negative", -3),
        ("exact-long-limit", 3037000499),
        ("positive-overflow", 3037000500),
        ("negative-overflow", -3037000500),
        ("power-of-two-wrap", 4294967296),
        ("negative-power-of-two-wrap", -4294967296),
        ("maximum", 9223372036854775807),
        ("minimum", -9223372036854775808),
        ("maximum-minus-one", 9223372036854775806),
        ("minimum-plus-one", -9223372036854775807),
        ("mixed", 1234567890123456789),
    ]
    for label, x in mth_square_i64:
        yield {"case_id": f"mth/square-i64/{label}", "family": "math",
               "operation": "mth_square_i64", "x": str(x)}

    min_subnormal_f64 = float.fromhex("0x0.0000000000001p-1022")
    maximum_finite_f64 = float.fromhex("0x1.fffffffffffffp1023")
    mth_clamped_map_f64 = [
        ("basic", 5.0, 0.0, 10.0, 0.0, 100.0),
        ("below-source", -5.0, 0.0, 10.0, 100.0, 200.0),
        ("above-source", 15.0, 0.0, 10.0, 100.0, 200.0),
        ("lower-endpoint", 0.0, 0.0, 10.0, -4.0, 12.0),
        ("upper-endpoint", 10.0, 0.0, 10.0, -4.0, 12.0),
        ("negative-range", -5.0, -10.0, 0.0, 10.0, 30.0),
        ("reversed-source", 5.0, 10.0, 0.0, 0.0, 100.0),
        ("reversed-target", 0.25, 0.0, 1.0, 10.0, -10.0),
        ("reversed-both", 5.0, 10.0, 0.0, 100.0, 0.0),
        ("negative-zero", -0.0, 0.0, 1.0, -0.0, 1.0),
        ("source-subnormal", min_subnormal_f64, 0.0, min_subnormal_f64, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, min_subnormal_f64),
        ("precision-order", 0.1, 1.0e20, 1.0e20 + 1.0e6, -1.0e20, 1.0e20),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, -float("inf"), float("inf"), 0.0, 1.0),
        ("positive-infinity-value", float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", -float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("nan-value", float("nan"), 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, float("nan"), 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -maximum_finite_f64, maximum_finite_f64),
    ]
    for label, value, from_min, from_max, to_min, to_max in mth_clamped_map_f64:
        yield {"case_id": f"mth/clamped-map-f64/{label}", "family": "math",
               "operation": "mth_clamped_map_f64",
               "value": float_bits(value, "f64"), "from_min": float_bits(from_min, "f64"),
               "from_max": float_bits(from_max, "f64"), "to_min": float_bits(to_min, "f64"),
               "to_max": float_bits(to_max, "f64")}

    min_subnormal_f32 = float.fromhex("0x0.000002p-126")
    maximum_finite_f32 = float.fromhex("0x1.fffffep127")
    mth_clamped_map_f32 = [
        ("basic", 5.0, 0.0, 10.0, 0.0, 100.0),
        ("below-source", -5.0, 0.0, 10.0, 100.0, 200.0),
        ("above-source", 15.0, 0.0, 10.0, 100.0, 200.0),
        ("lower-endpoint", 0.0, 0.0, 10.0, -4.0, 12.0),
        ("upper-endpoint", 10.0, 0.0, 10.0, -4.0, 12.0),
        ("negative-range", -5.0, -10.0, 0.0, 10.0, 30.0),
        ("reversed-source", 5.0, 10.0, 0.0, 0.0, 100.0),
        ("reversed-target", 0.25, 0.0, 1.0, 10.0, -10.0),
        ("reversed-both", 5.0, 10.0, 0.0, 100.0, 0.0),
        ("negative-zero", -0.0, 0.0, 1.0, -0.0, 1.0),
        ("source-subnormal", min_subnormal_f32, 0.0, min_subnormal_f32, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, min_subnormal_f32),
        ("precision-order", 0.1, 1.0e10, 1.0e10 + 1.0e6, -1.0e10, 1.0e10),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, -float("inf"), float("inf"), 0.0, 1.0),
        ("positive-infinity-value", float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", -float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("nan-value", float("nan"), 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, float("nan"), 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -maximum_finite_f32, maximum_finite_f32),
    ]
    for label, value, from_min, from_max, to_min, to_max in mth_clamped_map_f32:
        yield {"case_id": f"mth/clamped-map-f32/{label}", "family": "math",
               "operation": "mth_clamped_map_f32",
               "value": float_bits(value, "f32"), "from_min": float_bits(from_min, "f32"),
               "from_max": float_bits(from_max, "f32"), "to_min": float_bits(to_min, "f32"),
               "to_max": float_bits(to_max, "f32")}

    mth_map_f64 = [
        ("basic", 5.0, 0.0, 10.0, 0.0, 100.0),
        ("below-source", -5.0, 0.0, 10.0, 100.0, 200.0),
        ("above-source", 15.0, 0.0, 10.0, 100.0, 200.0),
        ("lower-endpoint", 0.0, 0.0, 10.0, -4.0, 12.0),
        ("upper-endpoint", 10.0, 0.0, 10.0, -4.0, 12.0),
        ("negative-range", -5.0, -10.0, 0.0, 10.0, 30.0),
        ("reversed-source", 5.0, 10.0, 0.0, 0.0, 100.0),
        ("reversed-target", 0.25, 0.0, 1.0, 10.0, -10.0),
        ("reversed-both", 5.0, 10.0, 0.0, 100.0, 0.0),
        ("negative-zero", -0.0, 0.0, 1.0, -0.0, 1.0),
        ("source-subnormal", min_subnormal_f64, 0.0, min_subnormal_f64, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, min_subnormal_f64),
        ("precision-order", 0.1, 1.0e20, 1.0e20 + 1.0e6, -1.0e20, 1.0e20),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, -float("inf"), float("inf"), 0.0, 1.0),
        ("positive-infinity-value", float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", -float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("nan-value", float("nan"), 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, float("nan"), 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -maximum_finite_f64, maximum_finite_f64),
    ]
    for label, value, from_min, from_max, to_min, to_max in mth_map_f64:
        yield {"case_id": f"mth/map-f64/{label}", "family": "math",
               "operation": "mth_map_f64",
               "value": float_bits(value, "f64"), "from_min": float_bits(from_min, "f64"),
               "from_max": float_bits(from_max, "f64"), "to_min": float_bits(to_min, "f64"),
               "to_max": float_bits(to_max, "f64")}

    mth_map_f32 = [
        ("basic", 5.0, 0.0, 10.0, 0.0, 100.0),
        ("below-source", -5.0, 0.0, 10.0, 100.0, 200.0),
        ("above-source", 15.0, 0.0, 10.0, 100.0, 200.0),
        ("lower-endpoint", 0.0, 0.0, 10.0, -4.0, 12.0),
        ("upper-endpoint", 10.0, 0.0, 10.0, -4.0, 12.0),
        ("negative-range", -5.0, -10.0, 0.0, 10.0, 30.0),
        ("reversed-source", 5.0, 10.0, 0.0, 0.0, 100.0),
        ("reversed-target", 0.25, 0.0, 1.0, 10.0, -10.0),
        ("reversed-both", 5.0, 10.0, 0.0, 100.0, 0.0),
        ("negative-zero", -0.0, 0.0, 1.0, -0.0, 1.0),
        ("source-subnormal", min_subnormal_f32, 0.0, min_subnormal_f32, 0.0, 1.0),
        ("target-subnormal", 0.5, 0.0, 1.0, 0.0, min_subnormal_f32),
        ("precision-order", 0.1, 1.0e10, 1.0e10 + 1.0e6, -1.0e10, 1.0e10),
        ("zero-source-range", 1.0, 1.0, 1.0, 0.0, 1.0),
        ("infinite-source-range", 0.0, -float("inf"), float("inf"), 0.0, 1.0),
        ("positive-infinity-value", float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("negative-infinity-value", -float("inf"), 0.0, 1.0, 0.0, 1.0),
        ("nan-value", float("nan"), 0.0, 1.0, 0.0, 1.0),
        ("nan-from-min", 0.0, float("nan"), 1.0, 0.0, 1.0),
        ("overflow-output", 0.5, 0.0, 1.0, -maximum_finite_f32, maximum_finite_f32),
    ]
    for label, value, from_min, from_max, to_min, to_max in mth_map_f32:
        yield {"case_id": f"mth/map-f32/{label}", "family": "math",
               "operation": "mth_map_f32",
               "value": float_bits(value, "f32"), "from_min": float_bits(from_min, "f32"),
               "from_max": float_bits(from_max, "f32"), "to_min": float_bits(to_min, "f32"),
               "to_max": float_bits(to_max, "f32")}

    maximum_finite_f64 = float.fromhex("0x1.fffffffffffffp1023")
    wobble_coords = [
        ("zero", float_bits(0.0, "f64")),
        ("negative-zero", "8000000000000000"),
        ("positive-integer", float_bits(1.0, "f64")),
        ("negative-integer", float_bits(-1.0, "f64")),
        ("positive-fraction", float_bits(0.25, "f64")),
        ("negative-fraction", float_bits(-0.25, "f64")),
        ("positive-boundary", float_bits(1.0 / 3000.0, "f64")),
        ("negative-boundary", float_bits(-1.0 / 3000.0, "f64")),
        ("below-positive-boundary", float_bits(math.nextafter(1.0 / 3000.0, 0.0), "f64")),
        ("above-positive-boundary", float_bits(math.nextafter(1.0 / 3000.0, math.inf), "f64")),
        ("large-positive", float_bits(1_000_000.25, "f64")),
        ("large-negative", float_bits(-1_000_000.25, "f64")),
        ("positive-floor-overflow", float_bits(715_827.883, "f64")),
        ("negative-floor-overflow", float_bits(-715_827.883, "f64")),
        ("maximum-finite", float_bits(maximum_finite_f64, "f64")),
        ("positive-subnormal", "0000000000000001"),
        ("negative-subnormal", "8000000000000001"),
        ("positive-infinity", "7ff0000000000000"),
        ("negative-infinity", "fff0000000000000"),
        ("negative-nan-payload", "fff8000000000001"),
    ]
    for label, coord in wobble_coords:
        yield {"case_id": f"mth/wobble-f64/{label}", "family": "math",
               "operation": "mth_wobble_f64", "coord": coord}

    def interpolation_bits(value, width):
        return value if isinstance(value, str) else float_bits(value, width)

    mth_lerp2 = [
        ("basic", 0.25, 0.5, 0.0, 10.0, 20.0, 30.0),
        ("zero-alpha", 0.0, 0.0, 1.0, 2.0, 3.0, 4.0),
        ("one-alpha", 1.0, 1.0, 1.0, 2.0, 3.0, 4.0),
        ("extrapolation", 1.5, -0.5, 0.0, 10.0, 20.0, 30.0),
        ("negative-alpha", -0.25, 0.75, -1.0, 3.0, -5.0, 7.0),
        ("reversed-grid", 0.75, 0.25, 10.0, 0.0, -10.0, -20.0),
        ("signed-zero-alpha", -0.0, 0.0, -0.0, 1.0, -0.0, 1.0),
        ("signed-zero-grid", 0.5, 0.5, "8000000000000000", "0000000000000000",
         "8000000000000000", "0000000000000000"),
        ("subnormal-grid", 0.5, 0.5, min_subnormal_f64, 2.0 * min_subnormal_f64,
         -min_subnormal_f64, -2.0 * min_subnormal_f64),
        ("tiny-alpha", float.fromhex("0x1.0000000000001p-1022"), 0.5,
         1.0, 2.0, 3.0, 4.0),
        ("large-finite", 0.25, 0.75, 1.0e300, -1.0e300, 1.0e300, -1.0e300),
        ("overflow-grid", 0.5, 0.5, maximum_finite_f64, -maximum_finite_f64,
         maximum_finite_f64, -maximum_finite_f64),
        ("positive-infinity-grid", 0.25, 0.75, float("inf"), 1.0, 2.0, float("inf")),
        ("negative-infinity-grid", 0.25, 0.75, float("-inf"), -1.0, -2.0, float("-inf")),
        ("nan-alpha", float("nan"), 0.5, 0.0, 1.0, 2.0, 3.0),
        ("nan-first-corner", 0.5, 0.5, float("nan"), 1.0, 2.0, 3.0),
        ("nan-last-corner", 0.5, 0.5, 0.0, 1.0, 2.0, float("nan")),
        ("precision-order", 0.1, 0.9, 1.0e20, 1.0e20 + 1.0e6,
         -1.0e20, -1.0e20 + 1.0e6),
        ("mixed-exponents", 1.0 / 3.0, 2.0 / 3.0, 2.0 ** -500, 2.0 ** 500,
         -2.0 ** -500, -2.0 ** 500),
        ("underflow", min_subnormal_f64, 0.5, 1.0, 2.0, -1.0, -2.0),
    ]
    for label, alpha1, alpha2, x00, x10, x01, x11 in mth_lerp2:
        yield {"case_id": f"mth/interpolation/lerp2/{label}", "family": "math",
               "operation": "mth_interpolation", "fn": "lerp2",
               "alpha1": interpolation_bits(alpha1, "f64"),
               "alpha2": interpolation_bits(alpha2, "f64"),
               "x00": interpolation_bits(x00, "f64"),
               "x10": interpolation_bits(x10, "f64"),
               "x01": interpolation_bits(x01, "f64"),
               "x11": interpolation_bits(x11, "f64")}

    mth_lerp3 = [
        ("basic", 0.25, 0.5, 0.75, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0),
        ("zero-alpha", 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0),
        ("one-alpha", 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0),
        ("extrapolation", 1.5, -0.5, 1.25, 0.0, 10.0, 20.0, 30.0,
         40.0, 50.0, 60.0, 70.0),
        ("negative-alpha", -0.25, 0.75, -0.5, -1.0, 3.0, -5.0, 7.0,
         -9.0, 11.0, -13.0, 15.0),
        ("reversed-grid", 0.75, 0.25, 0.5, 10.0, 0.0, -10.0, -20.0,
         20.0, 30.0, 40.0, 50.0),
        ("signed-zero-alpha", -0.0, 0.0, -0.0, -0.0, 1.0, -0.0, 1.0,
         -0.0, 1.0, -0.0, 1.0),
        ("signed-zero-cube", 0.5, 0.5, 0.5, "8000000000000000", "0000000000000000",
         "8000000000000000", "0000000000000000", "8000000000000000",
         "0000000000000000", "8000000000000000", "0000000000000000"),
        ("subnormal-cube", 0.5, 0.5, 0.5, min_subnormal_f64, 2.0 * min_subnormal_f64,
         3.0 * min_subnormal_f64, 4.0 * min_subnormal_f64, -min_subnormal_f64,
         -2.0 * min_subnormal_f64, -3.0 * min_subnormal_f64, -4.0 * min_subnormal_f64),
        ("tiny-alpha", float.fromhex("0x1.0000000000001p-1022"), 0.5, 0.75,
         1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0),
        ("large-finite", 0.25, 0.75, 0.5, 1.0e300, -1.0e300, 1.0e300, -1.0e300,
         1.0e300, -1.0e300, 1.0e300, -1.0e300),
        ("overflow-cube", 0.5, 0.5, 0.5, maximum_finite_f64, -maximum_finite_f64,
         maximum_finite_f64, -maximum_finite_f64, maximum_finite_f64,
         -maximum_finite_f64, maximum_finite_f64, -maximum_finite_f64),
        ("positive-infinity-cube", 0.25, 0.5, 0.75, float("inf"), 1.0, 2.0, 3.0,
         4.0, 5.0, 6.0, float("inf")),
        ("negative-infinity-cube", 0.25, 0.5, 0.75, float("-inf"), -1.0, -2.0, -3.0,
         -4.0, -5.0, -6.0, float("-inf")),
        ("nan-alpha", float("nan"), 0.5, 0.75, 0.0, 1.0, 2.0, 3.0,
         4.0, 5.0, 6.0, 7.0),
        ("nan-first-corner", 0.5, 0.5, 0.5, float("nan"), 1.0, 2.0, 3.0,
         4.0, 5.0, 6.0, 7.0),
        ("nan-last-corner", 0.5, 0.5, 0.5, 0.0, 1.0, 2.0, 3.0,
         4.0, 5.0, 6.0, float("nan")),
        ("precision-order", 0.1, 0.9, 0.3, 1.0e20, 1.0e20 + 1.0e6,
         -1.0e20, -1.0e20 + 1.0e6, 2.0e20, 2.0e20 + 1.0e6,
         -2.0e20, -2.0e20 + 1.0e6),
        ("mixed-exponents", 1.0 / 3.0, 2.0 / 3.0, 0.125, 2.0 ** -500, 2.0 ** 500,
         -2.0 ** -500, -2.0 ** 500, 2.0 ** -400, 2.0 ** 400,
         -2.0 ** -400, -2.0 ** 400),
        ("underflow", min_subnormal_f64, 0.5, 0.75, 1.0, 2.0, 3.0, 4.0,
         5.0, 6.0, 7.0, 8.0),
    ]
    for label, alpha1, alpha2, alpha3, x000, x100, x010, x110, x001, x101, x011, x111 in mth_lerp3:
        yield {"case_id": f"mth/interpolation/lerp3/{label}", "family": "math",
               "operation": "mth_interpolation", "fn": "lerp3",
               "alpha1": interpolation_bits(alpha1, "f64"),
               "alpha2": interpolation_bits(alpha2, "f64"),
               "alpha3": interpolation_bits(alpha3, "f64"),
               "x000": interpolation_bits(x000, "f64"),
               "x100": interpolation_bits(x100, "f64"),
               "x010": interpolation_bits(x010, "f64"),
               "x110": interpolation_bits(x110, "f64"),
               "x001": interpolation_bits(x001, "f64"),
               "x101": interpolation_bits(x101, "f64"),
               "x011": interpolation_bits(x011, "f64"),
               "x111": interpolation_bits(x111, "f64")}

    maximum_finite_f32 = float.fromhex("0x1.fffffep127")
    min_subnormal_f32 = float.fromhex("0x0.000002p-126")
    mth_catmullrom = [
        ("basic", 0.5, 0.0, 1.0, 2.0, 3.0),
        ("zero-alpha", 0.0, -1.0, 0.0, 1.0, 2.0),
        ("one-alpha", 1.0, -1.0, 0.0, 1.0, 2.0),
        ("extrapolation-negative", -0.5, 0.0, 1.0, 2.0, 3.0),
        ("extrapolation-positive", 1.5, 0.0, 1.0, 2.0, 3.0),
        ("constant", 0.25, 4.0, 4.0, 4.0, 4.0),
        ("peak", 0.5, 0.0, 0.0, 1.0, 0.0),
        ("symmetric", 0.5, -2.0, -1.0, 1.0, 2.0),
        ("negative", 0.25, -1.0, -3.0, -5.0, -7.0),
        ("signed-zero-alpha", "80000000", -0.0, 0.0, -0.0, 0.0),
        ("signed-zero-points", 0.5, "80000000", "00000000", "80000000", "00000000"),
        ("subnormal-points", 0.5, min_subnormal_f32, 2.0 * min_subnormal_f32,
         3.0 * min_subnormal_f32, 4.0 * min_subnormal_f32),
        ("subnormal-alpha", min_subnormal_f32, 1.0, 2.0, 3.0, 4.0),
        ("large-finite", 0.25, 1.0e10, -1.0e10, 1.0e10, -1.0e10),
        ("maximum-finite", 0.5, maximum_finite_f32, -maximum_finite_f32,
         maximum_finite_f32, -maximum_finite_f32),
        ("overflow", maximum_finite_f32, 1.0, -1.0, 1.0, -1.0),
        ("positive-infinity", 0.5, float("inf"), 1.0, 2.0, 3.0),
        ("negative-infinity", 0.5, float("-inf"), -1.0, -2.0, -3.0),
        ("nan-alpha", float("nan"), 0.0, 1.0, 2.0, 3.0),
        ("nan-point", 0.5, "7fc00001", 1.0, 2.0, 3.0),
    ]
    for label, alpha, p0, p1, p2, p3 in mth_catmullrom:
        yield {"case_id": f"mth/interpolation/catmullrom/{label}", "family": "math",
               "operation": "mth_interpolation", "fn": "catmullrom",
               "alpha": interpolation_bits(alpha, "f32"),
               "p0": interpolation_bits(p0, "f32"),
               "p1": interpolation_bits(p1, "f32"),
               "p2": interpolation_bits(p2, "f32"),
               "p3": interpolation_bits(p3, "f32")}


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


def resources_cases():
    def string_frame(value):
        encoded = value.encode("utf-8")
        length = len(encoded)
        prefix = bytearray()
        while True:
            part = length & 0x7f
            length >>= 7
            prefix.append(part | (0x80 if length else 0))
            if not length:
                return bytes(prefix + encoded).hex()

    hashes = {
        "a": "00112233445566778899aabbccddeeff00112233",
        "b": "abcdefabcdefabcdefabcdefabcdefabcdefabcd",
        "c": "102030405060708090a0b0c0d0e0f00010203040",
        "missing": "fedcba9876543210fedcba9876543210fedcba98",
    }

    yield {
        "case_id": "resources/index/basic-mappings",
        "family": "resources",
        "operation": "index",
        "index": "32",
        "entries": [
            {"path": "minecraft/textures/a.png", "hash": hashes["a"]},
            {"path": "minecraft/lang/en_us.json", "hash": hashes["b"]},
            {"path": "realms/icons/realm.png", "hash": hashes["c"]},
        ],
        "objects": [
            {"hash": hashes["a"], "hex": "89504e470d0a"},
            {"hash": hashes["b"], "hex": "7b226c616e67223a7d"},
            {"hash": hashes["c"], "hex": "5245414c4d53"},
        ],
        "queries": [
            "minecraft/textures/a.png", "minecraft/lang/en_us.json",
            "realms/icons/realm.png", "minecraft/textures", "minecraft",
            "minecraft/textures/missing.png", "unknown",
        ],
    }
    yield {
        "case_id": "resources/index/file-directory-collision",
        "family": "resources",
        "operation": "index",
        "index": "collision",
        "entries": [
            {"path": "foo", "hash": hashes["a"]},
            {"path": "foo/bar", "hash": hashes["b"]},
            {"path": "foo/bar/baz", "hash": hashes["c"]},
            {"path": "foo/qux", "hash": hashes["missing"]},
        ],
        "objects": [
            {"hash": hashes["a"], "hex": "41"},
            {"hash": hashes["b"], "hex": "42"},
            {"hash": hashes["c"], "hex": "4343"},
        ],
        "queries": ["foo", "foo/bar", "foo/bar/baz", "foo/qux", "bar"],
    }
    yield {
        "case_id": "resources/index/missing-object",
        "family": "resources",
        "operation": "index",
        "index": "missing",
        "entries": [{"path": "minecraft/data/missing.bin", "hash": hashes["missing"]}],
        "objects": [],
        "queries": ["minecraft/data/missing.bin", "minecraft/data", "minecraft/data/other.bin"],
    }
    yield {
        "case_id": "resources/index/empty",
        "family": "resources",
        "operation": "index",
        "index": "empty",
        "entries": [],
        "objects": [],
        "queries": ["minecraft", "minecraft/data"],
    }
    yield {
        "case_id": "resources/packs/priority-and-stack",
        "family": "resources",
        "operation": "packs",
        "packs": [
            {
                "id": "base",
                "resources": [
                    {"id": "minecraft:textures/shared.png", "hex": "42415345"},
                    {"id": "minecraft:textures/base.png", "hex": "42415345"},
                ],
            },
            {
                "id": "overlay",
                "resources": [
                    {"id": "minecraft:textures/shared.png", "hex": "4f564552"},
                    {"id": "minecraft:textures/overlay.png", "hex": "4f564552"},
                ],
            },
        ],
        "queries": [
            "minecraft:textures/shared.png", "minecraft:textures/base.png",
            "minecraft:textures/overlay.png", "minecraft:textures/missing.png",
        ],
    }
    yield {
        "case_id": "resources/packs/three-layer-precedence",
        "family": "resources",
        "operation": "packs",
        "packs": [
            {"id": "low", "resources": [{"id": "minecraft:config/value.json", "hex": "4c4f57"}]},
            {"id": "middle", "resources": [{"id": "minecraft:config/value.json", "hex": "4d4944"}]},
            {"id": "high", "resources": [{"id": "minecraft:config/value.json", "hex": "48494748"}]},
        ],
        "queries": ["minecraft:config/value.json"],
    }
    yield {
        "case_id": "resources/reload/replaces-active-stack",
        "family": "resources",
        "operation": "reload",
        "initial_queries": ["minecraft:textures/shared.png", "other:value.json"],
        "rounds": [
            {
                "packs": [
                    {
                        "id": "base-r1",
                        "resources": [
                            {"id": "minecraft:textures/shared.png", "hex": "4f4c44"},
                            {"id": "minecraft:textures/legacy.png", "hex": "4c4547414359"},
                        ],
                    },
                    {
                        "id": "overlay-r1",
                        "resources": [
                            {"id": "minecraft:textures/shared.png", "hex": "4f5631"},
                            {"id": "custom:config/value.json", "hex": "435553544f4d"},
                        ],
                    },
                ],
                "queries": [
                    "minecraft:textures/shared.png", "minecraft:textures/legacy.png",
                    "custom:config/value.json", "other:value.json",
                ],
            },
            {
                "packs": [
                    {
                        "id": "replacement-r2",
                        "resources": [
                            {"id": "minecraft:textures/shared.png", "hex": "4e4557"},
                            {"id": "other:value.json", "hex": "5245504c41434544"},
                        ],
                    },
                ],
                "queries": [
                    "minecraft:textures/shared.png", "minecraft:textures/legacy.png",
                    "custom:config/value.json", "other:value.json",
                ],
            },
        ],
    }
    yield {
        "case_id": "resources/reload/empty-remove-readd",
        "family": "resources",
        "operation": "reload",
        "initial_queries": ["minecraft:config/value.json"],
        "rounds": [
            {"packs": [], "queries": ["minecraft:config/value.json"]},
            {
                "packs": [
                    {
                        "id": "fresh",
                        "resources": [{"id": "minecraft:config/value.json", "hex": "4652455348"}],
                    },
                ],
                "queries": ["minecraft:config/value.json"],
            },
            {"packs": [], "queries": ["minecraft:config/value.json"]},
        ],
    }
    yield {
        "case_id": "resources/metadata/compatibility-boundaries",
        "family": "resources",
        "operation": "compatibility",
        "probes": [
            {"min_major": 2147483647, "min_minor": 0, "max_major": 2147483647, "max_minor": 0},
            {"min_major": 0, "min_minor": 0, "max_major": 87, "max_minor": 0},
            {"min_major": 87, "min_minor": 5, "max_major": 87, "max_minor": 99},
            {"min_major": 87, "min_minor": 0, "max_major": 88, "max_minor": 0},
            {"min_major": 88, "min_minor": 0, "max_major": 88, "max_minor": 0},
            {"min_major": 88, "min_minor": 1, "max_major": 88, "max_minor": 2},
            {"min_major": 89, "min_minor": 0, "max_major": 89, "max_minor": 0},
            {"min_major": 87, "min_minor": 99, "max_major": 88, "max_minor": 0},
        ],
    }
    yield {
        "case_id": "resources/vanilla/external-root-precedence",
        "family": "resources",
        "operation": "vanilla",
        "roots": [
            {
                "id": "jar",
                "resources": [
                    {"id": "minecraft:textures/shared.png", "hex": "4a4152"},
                    {"id": "minecraft:textures/jar-only.png", "hex": "4a4152"},
                ],
            },
            {
                "id": "external",
                "resources": [
                    {"id": "minecraft:textures/shared.png", "hex": "455854"},
                    {"id": "minecraft:textures/external-only.png", "hex": "455854"},
                ],
            },
        ],
        "namespaces": ["minecraft"],
        "queries": [
            "minecraft:textures/shared.png", "minecraft:textures/jar-only.png",
            "minecraft:textures/external-only.png", "minecraft:textures/missing.png",
        ],
    }
    yield {
        "case_id": "resources/vanilla/explicit-namespaces",
        "family": "resources",
        "operation": "vanilla",
        "roots": [
            {
                "id": "assets",
                "resources": [
                    {"id": "minecraft:lang/en_us.json", "hex": "4d494e"},
                    {"id": "realms:icons/realm.png", "hex": "5245414c4d53"},
                    {"id": "custom:data/value.json", "hex": "435553544f4d"},
                ],
            },
        ],
        "namespaces": ["realms", "minecraft"],
        "queries": [
            "minecraft:lang/en_us.json", "realms:icons/realm.png",
            "custom:data/value.json", "unknown:data/value.json",
        ],
    }
    yield {
        "case_id": "resources/metadata/known-pack-values",
        "family": "resources",
        "operation": "known_pack",
        "probes": [
            {"kind": "vanilla", "id": "vanilla"},
            {"kind": "vanilla", "id": "custom-pack"},
            {"namespace": "minecraft", "id": "manual", "version": "26.2"},
            {"namespace": "custom", "id": "manual", "version": "v1"},
            {"namespace": "Minecraft", "id": "case-sensitive", "version": "26.2"},
            {"namespace": "", "id": "", "version": ""},
        ],
    }
    codec_encodes = [
        ("ascii", "minecraft", "vanilla", "26.2"),
        ("unicode", "mïnecraft", "pack-😀", "v1́"),
        ("empty-fields", "", "", ""),
        ("long-namespace", "a" * 32768, "id", "v"),
        ("long-id", "minecraft", "b" * 32768, "v"),
        ("long-version", "minecraft", "id", "c" * 32768),
    ]
    for label, namespace, pack_id, version in codec_encodes:
        yield {
            "case_id": f"resources/metadata/known-pack-codec/encode/{label}",
            "family": "resources",
            "operation": "known_pack_codec",
            "mode": "encode",
            "namespace": namespace,
            "id": pack_id,
            "version": version,
            "prefix_hex": "a55a",
        }

    codec_decodes = [
        (
            "ascii",
            string_frame("minecraft") + string_frame("vanilla") + string_frame("26.2"),
        ),
        (
            "unicode",
            string_frame("mïnecraft") + string_frame("pack-😀") + string_frame("v1́"),
        ),
        ("empty-fields", "000000"),
        ("trailing", "00000000aa"),
        ("malformed-body", "01ff000000"),
        ("second-prefix-truncated", "0080"),
        ("third-body-missing", "000001"),
        ("second-body-short", "000241"),
        ("first-prefix-truncated", "80"),
        ("first-prefix-too-big", "808080808000"),
        ("first-encoded-limit", "feff05"),
    ]
    for label, framed in codec_decodes:
        yield {
            "case_id": f"resources/metadata/known-pack-codec/decode/{label}",
            "family": "resources",
            "operation": "known_pack_codec",
            "mode": "decode",
            "hex": framed,
            "prefix_hex": "a55a",
        }

    def known_pack_value(namespace, pack_id, version):
        return {"namespace": namespace, "id": pack_id, "version": version}

    def varint_frame(value):
        encoded = bytearray()
        value &= 0xffffffff
        while True:
            part = value & 0x7f
            value >>= 7
            encoded.append(part | (0x80 if value else 0))
            if not value:
                return bytes(encoded).hex()

    def known_pack_frame(pack):
        return string_frame(pack["namespace"]) + string_frame(pack["id"]) + string_frame(pack["version"])

    basic_packs = [
        known_pack_value("minecraft", "vanilla", "26.2"),
        known_pack_value("custom", "example", "v1"),
    ]
    list_encodes = [
        ("clientbound-empty", "clientbound", []),
        ("clientbound-basic", "clientbound", basic_packs),
        ("serverbound-empty", "serverbound", []),
        ("serverbound-64", "serverbound", [known_pack_value("minecraft", "p", "1")] * 64),
        ("serverbound-65", "serverbound", [known_pack_value("minecraft", "p", "1")] * 65),
        (
            "serverbound-member-partial",
            "serverbound",
            [known_pack_value("minecraft", "a" * 32768, "1")],
        ),
    ]
    for label, packet, packs in list_encodes:
        yield {
            "case_id": f"resources/metadata/known-pack-list-codec/encode/{label}",
            "family": "resources",
            "operation": "known_pack_list_codec",
            "mode": "encode",
            "packet": packet,
            "packs": packs,
            "prefix_hex": "a55a",
        }

    client_basic_frame = varint_frame(len(basic_packs)) + "".join(
        known_pack_frame(pack) for pack in basic_packs
    )
    empty_pack_frame = "000000"
    list_decodes = [
        ("clientbound-basic", "clientbound", client_basic_frame),
        ("clientbound-trailing", "clientbound", "00aa"),
        ("serverbound-basic", "serverbound", varint_frame(1) + known_pack_frame(basic_packs[0])),
        ("serverbound-64", "serverbound", varint_frame(64) + empty_pack_frame * 64),
        ("serverbound-65", "serverbound", "41"),
        ("count-truncated", "serverbound", "80"),
        ("negative-count", "serverbound", "ffffffff0f"),
        ("member-malformed-body", "serverbound", "0101ff0000"),
        ("member-truncated", "serverbound", "0200000000"),
    ]
    for label, packet, framed in list_decodes:
        yield {
            "case_id": f"resources/metadata/known-pack-list-codec/decode/{label}",
            "family": "resources",
            "operation": "known_pack_list_codec",
            "mode": "decode",
            "packet": packet,
            "hex": framed,
            "prefix_hex": "a55a",
        }



def window_cases():
    """Java `Window` GUI scale selection and scaled-size rounding.

    `calculateScale` walks the scale up while both axes still leave at least the
    320x240 base area, then rounds odd scales up when unicode is enforced.
    `setGuiScale` divides the framebuffer by the scale as a double and rounds the
    result up whenever the division left a remainder, so the interesting inputs
    are the ones that divide exactly, the ones that do not, and the scale
    boundaries on each axis.
    """
    yield {
        "case_id": "window/base-size",
        "family": "window",
        "operation": "base_size",
    }

    yield {
        "case_id": "window/min-size",
        "family": "window",
        "operation": "min_size",
        "rows": [
            {"label": label, "requested": requested}
            for label, requested in [
                ("zero", 0),
                ("one", 1),
                ("negative", -1),
                ("min-int", -2147483648),
                ("default-width", 854),
                ("default-height", 480),
                ("max-int", 2147483647),
            ]
        ],
    }

    rows = [
        # label, framebuffer width, height, max scale, enforce unicode
        ("auto-1920x1080", 1920, 1080, 0, False),
        ("auto-2560x1440", 2560, 1440, 0, False),
        ("auto-3840x2160", 3840, 2160, 0, False),
        ("auto-1366x768", 1366, 768, 0, False),
        ("auto-800x600", 800, 600, 0, False),
        ("auto-1024x768", 1024, 768, 0, False),
        # exact base area and its neighbours drive the loop guard
        ("base-exact", 320, 240, 0, False),
        ("base-minus-one-x", 319, 240, 0, False),
        ("base-minus-one-y", 320, 239, 0, False),
        ("base-double", 640, 480, 0, False),
        ("base-double-minus-one", 639, 479, 0, False),
        # degenerate framebuffers still have to produce a usable scale
        ("tiny-1x1", 1, 1, 0, False),
        ("tiny-2x2", 2, 2, 0, False),
        ("narrow-320x2160", 320, 2160, 0, False),
        ("wide-3840x240", 3840, 240, 0, False),
        # capped scales stop the loop before the area guard does
        ("cap-1", 1920, 1080, 1, False),
        ("cap-2", 1920, 1080, 2, False),
        ("cap-3", 1920, 1080, 3, False),
        ("cap-4", 1920, 1080, 4, False),
        ("cap-above-auto", 1920, 1080, 9, False),
        # unicode rounds an odd result up, including past the cap
        ("unicode-even-result", 1920, 1080, 0, True),
        ("unicode-odd-result", 1000, 700, 0, True),
        ("unicode-cap-1", 1920, 1080, 1, True),
        ("unicode-cap-3", 1920, 1080, 3, True),
        ("unicode-base-exact", 320, 240, 0, True),
        ("unicode-tiny", 1, 1, 0, True),
        # non-divisible sizes exercise the scaled-size rounding branch
        ("remainder-1001x701", 1001, 701, 0, False),
        ("remainder-1920x1081", 1920, 1081, 0, False),
        ("remainder-cap-3", 1001, 701, 3, False),
        ("remainder-cap-7", 1001, 701, 7, False),
        ("divides-exactly", 1280, 720, 4, False),
    ]
    yield {
        "case_id": "window/gui-scale",
        "family": "window",
        "operation": "gui_scale",
        "rows": [
            {
                "label": label,
                "framebuffer_width": width,
                "framebuffer_height": height,
                "max_scale": max_scale,
                "enforce_unicode": enforce_unicode,
            }
            for label, width, height, max_scale, enforce_unicode in rows
        ],
    }


def atlas_cases():
    """Java `Stitcher`: where each sprite lands on the texture atlas.

    The packer sorts by descending height, then descending width, then sprite
    name, and grows the atlas by powers of two on whichever axis is behind.
    The inputs below drive the parts that decide a layout: equal-size sprites
    where only the name breaks the tie, mixed sizes that force subdivision,
    non-power-of-two sizes rounded up by the mip level, padding that grows with
    mip level and anisotropy, and sets that cannot fit at all.
    """
    def sprite(name, width, height):
        return {"name": name, "width": width, "height": height}

    def row(label, sprites, max_width=256, max_height=256, mip_level=0, anisotropy_bit=0):
        return {
            "label": label,
            "sprites": sprites,
            "max_width": max_width,
            "max_height": max_height,
            "mip_level": mip_level,
            "anisotropy_bit": anisotropy_bit,
        }

    uniform = [sprite(f"minecraft:block/b{index:02d}", 16, 16) for index in range(8)]
    # Same size and same path, different namespace: the comparator falls through
    # to the identifier, which orders on path before namespace.
    namespaces = [
        sprite("zzz:block/same", 16, 16),
        sprite("aaa:block/same", 16, 16),
        sprite("minecraft:block/same", 16, 16),
    ]
    mixed = [
        sprite("minecraft:block/tall", 16, 64),
        sprite("minecraft:block/wide", 64, 16),
        sprite("minecraft:block/small", 8, 8),
        sprite("minecraft:block/big", 32, 32),
        sprite("minecraft:block/tiny", 4, 4),
    ]
    # Path ordering and namespace ordering disagree here: by path `block/a`
    # (namespace zzz) comes first, by namespace `aaa` would. This is the row
    # that pins path-before-namespace rather than just the fallback.
    path_before_namespace = [
        sprite("zzz:block/a", 16, 16),
        sprite("aaa:block/b", 16, 16),
    ]
    ragged = [
        sprite("minecraft:block/odd", 13, 7),
        sprite("minecraft:block/odder", 5, 11),
        sprite("minecraft:block/oddest", 3, 3),
    ]

    rows = [
        row("single", [sprite("minecraft:block/only", 16, 16)]),
        row("uniform-8", uniform),
        row("name-tiebreak", namespaces),
        row("path-before-namespace", path_before_namespace),
        row("mixed-sizes", mixed),
        row("ragged-sizes", ragged),
        row("mip-1", mixed, mip_level=1),
        row("mip-2", mixed, mip_level=2),
        row("mip-4", mixed, mip_level=4),
        row("anisotropy-1", mixed, anisotropy_bit=1),
        row("anisotropy-3", mixed, anisotropy_bit=3),
        row("anisotropy-clamped-9", mixed, anisotropy_bit=9),
        row("mip-2-anisotropy-2", mixed, mip_level=2, anisotropy_bit=2),
        row("narrow-atlas", mixed, max_width=64, max_height=256),
        row("short-atlas", mixed, max_width=256, max_height=64),
        row("exact-fit", [sprite("minecraft:block/full", 64, 64)], max_width=64, max_height=64),
        row("too-large", [sprite("minecraft:block/huge", 128, 128)], max_width=64, max_height=64),
        row("overflow", uniform * 4, max_width=32, max_height=32),
        row("power-of-two-growth", [sprite(f"minecraft:block/g{i:02d}", 16, 16) for i in range(5)]),
    ]
    yield {
        "case_id": "atlas/stitch",
        "family": "atlas",
        "operation": "stitch",
        "rows": rows,
    }


def _png_chunk(kind, payload):
    import struct, zlib
    return (struct.pack(">I", len(payload)) + kind + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF))


def _encode_png(width, height, colour_type, bit_depth, rows, palette=None,
                transparency=None, filters=None):
    """Build a synthetic PNG. Original content, so the corpus stays traceable.

    `rows` holds the raw sample bytes per scanline, unfiltered; `filters` picks
    the filter byte for each row so every filter type can be exercised.
    """
    import struct, zlib
    header = struct.pack(">IIBBBBB", width, height, bit_depth, colour_type, 0, 0, 0)
    chunks = [_png_chunk(b"IHDR", header)]
    if palette is not None:
        chunks.append(_png_chunk(b"PLTE", bytes(palette)))
    if transparency is not None:
        chunks.append(_png_chunk(b"tRNS", bytes(transparency)))

    filters = filters or [0] * height
    stride = len(rows[0])
    raw = bytearray()
    previous = bytes(stride)
    # Sample width in bytes, used by Sub/Average/Paeth; sub-byte depths use 1.
    sample = max(1, (bit_depth * {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[colour_type]) // 8)
    for row, kind in zip(rows, filters):
        raw.append(kind)
        line = bytearray(len(row))
        for index, value in enumerate(row):
            left = row[index - sample] if index >= sample else 0
            up = previous[index]
            up_left = previous[index - sample] if index >= sample else 0
            if kind == 0:
                line[index] = value
            elif kind == 1:
                line[index] = (value - left) & 0xFF
            elif kind == 2:
                line[index] = (value - up) & 0xFF
            elif kind == 3:
                line[index] = (value - (left + up) // 2) & 0xFF
            else:
                estimate = left + up - up_left
                to_left = abs(estimate - left)
                to_up = abs(estimate - up)
                to_up_left = abs(estimate - up_left)
                if to_left <= to_up and to_left <= to_up_left:
                    best = left
                elif to_up <= to_up_left:
                    best = up
                else:
                    best = up_left
                line[index] = (value - best) & 0xFF
        raw.extend(line)
        previous = bytes(row)
    chunks.append(_png_chunk(b"IDAT", zlib.compress(bytes(raw), 9)))
    chunks.append(_png_chunk(b"IEND", b""))
    return b"\x89PNG\r\n\x1a\n" + b"".join(chunks)


def image_cases():
    """Java `NativeImage.read`: every PNG colour type reduced to RGBA8.

    The reference hands the bytes to stb with four components requested, so
    grayscale, palette and RGB all come back as RGBA and 16-bit samples are cut
    to their high byte. The inputs below cover each colour type, both bit
    depths, all five row filters and the transparency chunk.
    """
    def rgba_rows(width, height):
        return [bytes(b for x in range(width)
                      for b in ((x * 17 + y) % 256, (y * 23 + x) % 256,
                                ((x + y) * 5) % 256, (255 - x * 9) % 256))
                for y in range(height)]

    def rgb_rows(width, height):
        return [bytes(b for x in range(width)
                      for b in ((x * 31 + y) % 256, (y * 13) % 256, (x * y) % 256))
                for y in range(height)]

    def gray_rows(width, height):
        return [bytes((x * 37 + y * 11) % 256 for x in range(width))
                for y in range(height)]

    def gray_alpha_rows(width, height):
        return [bytes(b for x in range(width)
                      for b in ((x * 37 + y * 11) % 256, (x * 40) % 256))
                for y in range(height)]

    def indexed_rows(width, height, count):
        return [bytes((x + y) % count for x in range(width)) for y in range(height)]

    def rgba16_rows(width, height):
        return [bytes(b for x in range(width)
                      for channel in ((x * 257 + y) % 65536, (y * 999) % 65536,
                                      (x * y * 13) % 65536, 65535 - (x * 1234) % 65536)
                      for b in (channel >> 8, channel & 0xFF))
                for y in range(height)]

    def packed_rows(width, height, bit_depth, value_at):
        """Pack samples most significant bits first, padding each row."""
        rows = []
        for y in range(height):
            bits = []
            for x in range(width):
                value = value_at(x, y) & ((1 << bit_depth) - 1)
                bits.extend((value >> (bit_depth - 1 - b)) & 1 for b in range(bit_depth))
            while len(bits) % 8:
                bits.append(0)
            rows.append(bytes(
                sum(bit << (7 - offset) for offset, bit in enumerate(bits[base:base + 8]))
                for base in range(0, len(bits), 8)
            ))
        return rows

    palette = [c for index in range(4)
               for c in (index * 60 % 256, 255 - index * 40 % 256, index * 85 % 256)]

    rows = [
        ("rgba8-filter-none", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [0, 0, 0, 0]),
        ("rgba8-filter-sub", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [1, 1, 1, 1]),
        ("rgba8-filter-up", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [0, 2, 2, 2]),
        ("rgba8-filter-average", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [0, 3, 3, 3]),
        ("rgba8-filter-paeth", 4, 4, 6, 8, rgba_rows(4, 4), None, None, [0, 4, 4, 4]),
        ("rgba8-mixed-filters", 6, 5, 6, 8, rgba_rows(6, 5), None, None, [0, 1, 2, 3, 4]),
        ("rgb8", 4, 4, 2, 8, rgb_rows(4, 4), None, None, [0, 1, 2, 4]),
        ("gray8", 4, 4, 0, 8, gray_rows(4, 4), None, None, [0, 1, 2, 4]),
        ("gray-alpha8", 4, 4, 4, 8, gray_alpha_rows(4, 4), None, None, [0, 1, 2, 4]),
        ("indexed", 4, 4, 3, 8, indexed_rows(4, 4, 4), palette, None, [0, 1, 2, 4]),
        ("indexed-trns", 4, 4, 3, 8, indexed_rows(4, 4, 4), palette, [0, 80, 170, 255], [0] * 4),
        ("gray1", 8, 4, 0, 1, packed_rows(8, 4, 1, lambda x, y: (x + y) % 2), None, None, [0, 1, 2, 4]),
        ("gray2", 8, 4, 0, 2, packed_rows(8, 4, 2, lambda x, y: (x + y) % 4), None, None, [0, 1, 2, 4]),
        ("gray4", 8, 4, 0, 4, packed_rows(8, 4, 4, lambda x, y: (x * 3 + y) % 16), None, None, [0, 1, 2, 4]),
        ("indexed1", 8, 4, 3, 1, packed_rows(8, 4, 1, lambda x, y: (x + y) % 2), palette, None, [0, 1, 2, 4]),
        ("indexed2", 8, 4, 3, 2, packed_rows(8, 4, 2, lambda x, y: (x + y) % 4), palette, None, [0, 1, 2, 4]),
        ("indexed4", 8, 4, 3, 4, packed_rows(8, 4, 4, lambda x, y: (x + y) % 4), palette, None, [0, 1, 2, 4]),
        ("indexed4-trns", 8, 4, 3, 4, packed_rows(8, 4, 4, lambda x, y: (x + y) % 4), palette, [0, 80, 170, 255], [0, 1, 2, 4]),
        ("gray1-odd-width", 5, 3, 0, 1, packed_rows(5, 3, 1, lambda x, y: (x * y) % 2), None, None, [0, 1, 4]),
        ("indexed2-odd-width", 3, 3, 3, 2, packed_rows(3, 3, 2, lambda x, y: (x + 2 * y) % 4), palette, None, [0, 1, 4]),
        ("gray8-trns", 4, 4, 0, 8, gray_rows(4, 4), None, [0, 0], [0, 1, 2, 4]),
        ("gray8-trns-hit", 4, 4, 0, 8, [bytes([0, 37, 74, 111]) for _ in range(4)], None, [0, 0], [0] * 4),
        ("gray2-trns", 8, 4, 0, 2, packed_rows(8, 4, 2, lambda x, y: (x + y) % 4), None, [0, 0], [0, 1, 2, 4]),
        ("gray1-trns", 8, 4, 0, 1, packed_rows(8, 4, 1, lambda x, y: (x + y) % 2), None, [0, 1], [0, 1, 2, 4]),
        ("rgb8-trns", 4, 4, 2, 8, rgb_rows(4, 4), None, [0, 0, 0, 0, 0, 0], [0, 1, 2, 4]),
        ("rgba16", 3, 3, 6, 16, rgba16_rows(3, 3), None, None, [0, 1, 4]),
        ("single-pixel", 1, 1, 6, 8, rgba_rows(1, 1), None, None, [0]),
        ("tall", 2, 8, 6, 8, rgba_rows(2, 8), None, None, [0, 1, 2, 3, 4, 1, 2, 3]),
        ("wide", 8, 2, 6, 8, rgba_rows(8, 2), None, None, [4, 3]),
    ]

    yield {
        "case_id": "image/item-side-faces",
        "family": "image",
        "operation": "item_side_faces",
        "rows": [
            {"label": name, "texture": f"assets/minecraft/textures/item/{name}.png"}
            for name in ("diamond_sword", "stick", "apple", "bow", "fishing_rod", "glass_bottle",
                         "leather_helmet", "leather_helmet_overlay", "potion_overlay", "bucket",
                         "iron_ingot", "torch" if False else "coal")
        ],
    }

    yield {
        "case_id": "image/srgb-tables",
        "family": "image",
        "operation": "srgb_tables",
        "rows": [{"label": "tables"}],
    }

    def gradient_rows(width, height):
        return [bytes(b for x in range(width)
                      for b in ((x * 255 // max(width - 1, 1)),
                                (y * 255 // max(height - 1, 1)),
                                ((x + y) * 255 // max(width + height - 2, 1)),
                                255))
                for y in range(height)]

    def noisy_rows(width, height):
        state = [12345]
        def step():
            state[0] = (state[0] * 1103515245 + 12345) & 0x7FFFFFFF
            return (state[0] >> 16) & 0xFF
        return [bytes(step() for _ in range(width * 4)) for _ in range(height)]

    def cutout_rows(width, height):
        """A shape with hard transparent edges, like a plant or a leaf."""
        rows = []
        for y in range(height):
            row = bytearray()
            for x in range(width):
                inside = abs(x - width // 2) + abs(y - height // 2) < width // 3
                row += bytes([(x * 13) % 256, (y * 29) % 256, 180, 255 if inside else 0])
            rows.append(bytes(row))
        return rows

    def soft_alpha_rows(width, height):
        """Alpha that ramps rather than jumping, so the coverage search has to
        converge on a scale instead of reproducing one exactly."""
        return [bytes(b for x in range(width)
                      for b in (200, 120, 60, (x * 255) // max(width - 1, 1)))
                for _ in range(height)]

    def clear_rows(width, height):
        """No opaque texel at all, which leaves the fill with no colour to
        spread and the dark fill with no darkest colour to find."""
        return [bytes([90, 140, 200, 0] * width) for _ in range(height)]

    # label, width, height, texels, levels, strategy, bias, path, transparency
    mip_rows = [
        ("gradient-16", 16, 16, gradient_rows(16, 16), 4, "MEAN", 0.0, "block/parity", (False, False)),
        ("gradient-8", 8, 8, gradient_rows(8, 8), 3, "MEAN", 0.0, "block/parity", (False, False)),
        ("noisy-16", 16, 16, noisy_rows(16, 16), 4, "MEAN", 0.0, "block/parity", (False, False)),
        ("noisy-32", 32, 32, noisy_rows(32, 32), 4, "MEAN", 0.0, "block/parity", (False, False)),
        ("flat-4", 4, 4, [bytes([200, 100, 50, 255]) * 4 for _ in range(4)], 2, "MEAN", 0.0,
         "block/parity", (False, False)),
        # A single level requested means the source comes back untouched.
        ("no-mips", 16, 16, gradient_rows(16, 16), 0, "MEAN", 0.0, "block/parity", (False, False)),
        # The cutout strategies spread colour into the transparent area before
        # halving, then rescale alpha so the visible area keeps its size.
        ("cutout-16", 16, 16, cutout_rows(16, 16), 4, "CUTOUT", 0.0, "block/parity", (False, False)),
        ("cutout-32", 32, 32, cutout_rows(32, 32), 4, "CUTOUT", 0.0, "block/parity", (False, False)),
        # A stricter reference alpha moves the coverage the search aims at.
        ("strict-cutout-16", 16, 16, cutout_rows(16, 16), 4, "STRICT_CUTOUT", 0.0,
         "block/parity", (False, False)),
        ("cutout-bias", 16, 16, cutout_rows(16, 16), 4, "CUTOUT", 0.1, "block/parity", (False, False)),
        ("cutout-soft", 16, 16, soft_alpha_rows(16, 16), 4, "CUTOUT", 0.0, "block/parity", (False, False)),
        ("dark-cutout-16", 16, 16, cutout_rows(16, 16), 4, "DARK_CUTOUT", 0.0,
         "block/parity", (False, False)),
        # Filling is skipped for item textures, so level zero keeps its holes.
        ("cutout-item", 16, 16, cutout_rows(16, 16), 4, "CUTOUT", 0.0, "item/parity", (False, False)),
        # Asking for no mips still fills, because that runs before the early exit.
        ("cutout-no-mips", 16, 16, cutout_rows(16, 16), 0, "CUTOUT", 0.0, "block/parity", (False, False)),
        ("dark-cutout-no-mips", 16, 16, cutout_rows(16, 16), 0, "DARK_CUTOUT", 0.0,
         "block/parity", (False, False)),
        ("cutout-clear", 8, 8, clear_rows(8, 8), 3, "CUTOUT", 0.0, "block/parity", (False, False)),
        ("dark-cutout-clear", 8, 8, clear_rows(8, 8), 3, "DARK_CUTOUT", 0.0,
         "block/parity", (False, False)),
        # AUTO picks its strategy from the transparency it is told about.
        ("auto-opaque", 16, 16, cutout_rows(16, 16), 4, "AUTO", 0.0, "block/parity", (False, False)),
        ("auto-transparent", 16, 16, cutout_rows(16, 16), 4, "AUTO", 0.0, "block/parity", (True, False)),
        ("auto-translucent", 16, 16, cutout_rows(16, 16), 4, "AUTO", 0.0, "block/parity", (False, True)),
    ]
    yield {
        "case_id": "image/mip-levels",
        "family": "image",
        "operation": "mip_levels",
        "rows": [
            {
                "label": label,
                "name": f"minecraft:{path}",
                "levels": levels,
                "strategy": strategy,
                "bias": float_bits(bias),
                "transparent": transparency[0],
                "translucent": transparency[1],
                "png": _encode_png(w, h, 6, 8, data, None, None, [0] * h).hex(),
            }
            for label, w, h, data, levels, strategy, bias, path, transparency in mip_rows
        ],
    }

    # What the atlas measures from a sprite's pixels to resolve AUTO. The scan
    # separates fully transparent from partly transparent, and a texture can be
    # both at once.
    transparency_rows = [
        ("opaque", 8, 8, [bytes([10, 20, 30, 255] * 8) for _ in range(8)]),
        ("transparent", 8, 8, clear_rows(8, 8)),
        ("cutout", 16, 16, cutout_rows(16, 16)),
        ("translucent", 8, 8, [bytes([10, 20, 30, 128] * 8) for _ in range(8)]),
        ("both", 8, 8, [bytes([10, 20, 30, 0 if y % 2 else 128] * 8) for y in range(8)]),
        # 254 counts as partly transparent; only exactly 255 is opaque.
        ("almost-opaque", 8, 8, [bytes([10, 20, 30, 254] * 8) for _ in range(8)]),
        ("single-hole", 8, 8, [bytes(b for x in range(8)
                                    for b in (10, 20, 30, 0 if (x, y) == (3, 5) else 255))
                               for y in range(8)]),
    ]
    yield {
        "case_id": "image/compute-transparency",
        "family": "image",
        "operation": "compute_transparency",
        "rows": [
            {
                "label": label,
                "png": _encode_png(w, h, 6, 8, data, None, None, [0] * h).hex(),
            }
            for label, w, h, data in transparency_rows
        ],
    }

    # The texture section of a sprite's .mcmeta. Every field is optional, so
    # what an absent field falls back to is the substance here.
    metadata_rows = [
        ("empty", "{}"),
        ("blur", '{"blur": true}'),
        ("clamp", '{"clamp": true}'),
        ("strategy-mean", '{"mipmap_strategy": "mean"}'),
        ("strategy-cutout", '{"mipmap_strategy": "cutout"}'),
        ("strategy-strict", '{"mipmap_strategy": "strict_cutout"}'),
        ("strategy-dark", '{"mipmap_strategy": "dark_cutout"}'),
        ("strategy-auto", '{"mipmap_strategy": "auto"}'),
        ("bias", '{"alpha_cutoff_bias": 0.25}'),
        ("bias-negative", '{"alpha_cutoff_bias": -0.5}'),
        ("bias-integer", '{"alpha_cutoff_bias": 1}'),
        ("all", '{"blur": true, "clamp": true, "mipmap_strategy": "dark_cutout",'
                ' "alpha_cutoff_bias": 0.125}'),
        # Names the enum does not carry, and shapes the codec does not take.
        ("strategy-unknown", '{"mipmap_strategy": "smooth"}'),
        ("strategy-uppercase", '{"mipmap_strategy": "MEAN"}'),
        ("strategy-number", '{"mipmap_strategy": 1}'),
        ("blur-string", '{"blur": "true"}'),
        ("bias-string", '{"alpha_cutoff_bias": "0.25"}'),
        ("unknown-field", '{"unexpected": 1}'),
        ("array", "[]"),
        ("string", '"texture"'),
    ]
    yield {
        "case_id": "image/texture-metadata",
        "family": "image",
        "operation": "texture_metadata",
        "rows": [{"label": label, "json": text} for label, text in metadata_rows],
    }

    # Where a stitched sprite's texels land. The packer hands back the padded
    # slot's origin, not the sprite's own, so the padding has to be added back
    # before anything is written or any UV is taken.
    sprite_uv_rows = [
        ("origin", 16, 16, 256, 256, 0, 0, 0),
        ("padded", 16, 16, 256, 256, 0, 0, 1),
        ("offset", 16, 16, 256, 256, 18, 36, 1),
        ("mip-padding", 16, 16, 256, 256, 32, 64, 4),
        ("anisotropic-padding", 16, 16, 512, 512, 64, 128, 16),
        ("oblong", 8, 32, 128, 512, 10, 40, 2),
        # An atlas whose side is not a power of two still divides exactly.
        ("odd-atlas", 16, 16, 300, 200, 20, 30, 1),
    ]
    offsets = [float_bits(value) for value in (0.0, 0.25, 0.5, 1.0, -0.5, 1.5)]
    yield {
        "case_id": "image/sprite-uv",
        "family": "image",
        "operation": "sprite_uv",
        "rows": [
            {
                "label": label,
                "width": w,
                "height": h,
                "atlas_width": atlas_w,
                "atlas_height": atlas_h,
                "x": x,
                "y": y,
                "padding": padding,
                "offsets": offsets,
            }
            for label, w, h, atlas_w, atlas_h, x, y, padding in sprite_uv_rows
        ],
    }

    # The biome tints index a 256x256 colour map by temperature and rainfall.
    # A synthetic map keeps the case self-contained; what is being pinned is
    # the indexing, not the game's own artwork.
    colour_map_png = _encode_png(
        256, 256, 6, 8,
        [bytes(b for x in range(256) for b in (x, y, (x * y) % 256, 255)) for y in range(256)],
        None, None, [0] * 256,
    ).hex()
    colour_map_rows = [
        ("grass/default", "grass", 0.5, 1.0),
        ("grass/cold-wet", "grass", 0.0, 1.0),
        ("grass/hot-dry", "grass", 1.0, 0.0),
        ("grass/hot-wet", "grass", 1.0, 1.0),
        ("grass/mid", "grass", 0.8, 0.4),
        ("foliage/default", "foliage", 0.5, 1.0),
        ("foliage/plains", "foliage", 0.8, 0.4),
        ("dry-foliage/default", "dry_foliage", 0.5, 1.0),
        # Rain is scaled by temperature first, so the same rain lands elsewhere
        # at a different temperature.
        ("grass/rain-scaling", "grass", 0.25, 1.0),
        # The truncation is toward zero, so a fraction of a step is dropped.
        ("grass/just-under-a-step", "grass", 0.5 + 1.0 / 512.0, 1.0),
        # The lookup guards the upper bound but not the lower one.
        ("grass/above-one", "grass", 1.5, 1.0),
        ("grass/below-zero", "grass", -0.5, 1.0),
        ("grass/negative-rain", "grass", 0.5, -1.0),
    ]
    yield {
        "case_id": "image/colour-map",
        "family": "image",
        "operation": "colour_map",
        "colour_map": colour_map_png,
        "rows": [
            {
                "label": label,
                "kind": kind,
                "temperature": float_bits(temperature, "f64"),
                "rain": float_bits(rain, "f64"),
            }
            for label, kind, temperature, rain in colour_map_rows
        ],
    }

    # How a tint folds into a quad's colour: per channel, with an integer
    # division, and with white on either side short-circuiting.
    multiply_rows = [
        ("white-left", "ffffffff", "ff48b518"),
        ("white-right", "ff48b518", "ffffffff"),
        ("both-white", "ffffffff", "ffffffff"),
        ("foliage-on-grey", "ff48b518", "ff808080"),
        ("halves", "80808080", "80808080"),
        ("black", "ff000000", "ff48b518"),
        ("one-each", "01010101", "01010101"),
        # The top byte is unsigned, so a full alpha survives the divide.
        ("full-alpha", "ff010203", "ff040506"),
        ("max", "fefefefe", "fefefefe"),
    ]
    yield {
        "case_id": "image/argb-multiply",
        "family": "image",
        "operation": "argb_multiply",
        "rows": [
            {"label": label, "lhs": lhs, "rhs": rhs}
            for label, lhs, rhs in multiply_rows
        ],
    }

    yield {
        "case_id": "image/tint-constants",
        "family": "image",
        "operation": "tint_constants",
        "rows": [{"label": "constants"}],
    }

    yield {
        "case_id": "image/png-decode",
        "family": "image",
        "operation": "png_decode",
        "rows": [
            {
                "label": label,
                "png": _encode_png(w, h, ct, bd, data, pal, trns, flt).hex(),
            }
            for label, w, h, ct, bd, data, pal, trns, flt in rows
        ],
    }


def model_cases():
    """Java `TextureSlots`: how a model's texture variables resolve.

    A model's texture map may name a sprite directly or point at another slot
    with a leading `#`. The chain is walked child first, but applied parent
    first, so a child overrides its parent; a reference clears an inherited
    value and a value clears an inherited reference. References then resolve to
    a fixed point, and anything still dangling is dropped rather than failing.
    """
    def ref(slot, target):
        return {"slot": slot, "raw": "#" + target}

    def raw(slot, value):
        return {"slot": slot, "raw": value}

    def value(slot, sprite, force_translucent=False):
        return {"slot": slot, "sprite": sprite, "force_translucent": force_translucent}

    stone = "minecraft:block/stone"
    dirt = "minecraft:block/dirt"
    oak = "minecraft:block/oak_planks"

    rows = [
        ("empty", []),
        ("single-value", [[raw("all", stone)]]),
        ("object-form", [[value("all", stone, True)]]),
        # A child naming a sprite wins over the parent's sprite.
        ("child-overrides-parent", [[raw("all", dirt)], [raw("all", stone)]]),
        # cube_all shape: parent points every face at #all, child fills it in.
        ("cube-all", [
            [raw("all", stone)],
            [ref("top", "all"), ref("bottom", "all"), ref("side", "all"),
             ref("particle", "all")],
        ]),
        # The reference target itself arrives from a grandparent.
        ("three-level", [
            [raw("all", oak)],
            [ref("side", "all")],
            [ref("north", "side"), ref("particle", "side")],
        ]),
        # A chain of references collapsing to one sprite.
        ("reference-chain", [[raw("all", stone)],
                             [ref("top", "all")],
                             [ref("side", "top")],
                             [ref("north", "side")]]),
        # A child reference replaces an inherited concrete value.
        ("reference-clears-value", [[ref("all", "side")],
                                    [raw("all", stone), raw("side", dirt)]]),
        # A child value replaces an inherited reference.
        ("value-clears-reference", [[raw("all", stone)],
                                    [ref("all", "side"), raw("side", dirt)]]),
        # Nothing to point at: the slot is dropped, not an error.
        ("dangling-reference", [[ref("top", "nothing")]]),
        ("self-reference", [[ref("top", "top")]]),
        ("cycle", [[ref("top", "side"), ref("side", "top")]]),
        # A cycle next to a resolvable reference: only the good one survives.
        ("cycle-plus-resolvable", [[raw("all", stone)],
                                   [ref("top", "side"), ref("side", "top"),
                                    ref("bottom", "all")]]),
        ("force-translucent-inherited", [[value("all", stone, True)],
                                         [ref("side", "all")]]),
        ("mixed-levels", [
            [raw("all", stone), value("particle", dirt, True)],
            [ref("top", "all"), raw("bottom", oak)],
            [ref("side", "bottom"), ref("north", "missing")],
        ]),
        # Same slot named at every level: the deepest child still wins.
        ("deep-override", [[raw("all", oak)], [raw("all", dirt)], [raw("all", stone)]]),
    ]

    def bits(value):
        import struct
        return f"{struct.unpack('>I', struct.pack('>f', value))[0]:08x}"

    def box(from_xyz, to_xyz):
        return ([bits(v) for v in from_xyz], [bits(v) for v in to_xyz])

    facings = ["down", "up", "north", "south", "west", "east"]
    boxes = [
        ("full", (0.0, 0.0, 0.0), (16.0, 16.0, 16.0)),
        ("slab", (0.0, 0.0, 0.0), (16.0, 8.0, 16.0)),
        ("offset", (2.0, 3.0, 4.0), (14.0, 13.0, 12.0)),
        ("thin", (7.5, 0.0, 7.5), (8.5, 16.0, 8.5)),
        ("degenerate", (4.0, 4.0, 4.0), (4.0, 12.0, 12.0)),
        # The element grammar allows reaching outside the block.
        ("oversized", (-16.0, -16.0, -16.0), (32.0, 32.0, 32.0)),
        ("fractional", (0.5, 1.25, 2.125), (15.5, 14.75, 13.875)),
        # Inverted bounds are not rejected by the corner layout itself.
        ("inverted", (12.0, 12.0, 12.0), (4.0, 4.0, 4.0)),
    ]
    geometry_rows = []
    for name, from_xyz, to_xyz in boxes:
        from_bits, to_bits = box(from_xyz, to_xyz)
        for facing in facings:
            geometry_rows.append({
                "label": f"{name}-{facing}",
                "from": from_bits,
                "to": to_bits,
                "facing": facing,
            })

    rotation_rows = []
    # The angles real models use, plus values the grammar allows but no
    # vanilla model writes.
    angles = [0.0, 22.5, -22.5, 45.0, -45.0, 90.0, 180.0, 270.0, 360.0, 30.0, -7.5]
    for axis in ("x", "y", "z"):
        for angle in angles:
            for rescale in (False, True):
                rotation_rows.append({
                    "label": f"{axis}-{angle}-{'rescale' if rescale else 'plain'}",
                    "from": [bits(v) for v in (0.0, 0.0, 0.0)],
                    "to": [bits(v) for v in (16.0, 16.0, 16.0)],
                    "facing": "up",
                    "origin": [bits(v) for v in (8.0, 8.0, 8.0)],
                    "axis": axis,
                    "angle": bits(angle),
                    "rescale": rescale,
                })
    # A non-centred origin and a partial box, which is what stairs and torches
    # actually look like.
    for facing in ("down", "north", "east"):
        rotation_rows.append({
            "label": f"offset-origin-{facing}",
            "from": [bits(v) for v in (4.0, 0.0, 4.0)],
            "to": [bits(v) for v in (12.0, 10.0, 12.0)],
            "facing": facing,
            "origin": [bits(v) for v in (0.0, 8.0, 16.0)],
            "axis": "y",
            "angle": bits(22.5),
            "rescale": True,
        })

    def corner(x, y, z):
        return [bits(x), bits(y), bits(z)]

    winding_rows = [
        # A full-block face in the reference's own corner order for each side.
        ("up-ordered", [corner(0, 1, 0), corner(0, 1, 1), corner(1, 1, 1), corner(1, 1, 0)]),
        ("down-ordered", [corner(0, 0, 1), corner(0, 0, 0), corner(1, 0, 0), corner(1, 0, 1)]),
        ("north-ordered", [corner(1, 1, 0), corner(1, 0, 0), corner(0, 0, 0), corner(0, 1, 0)]),
        ("south-ordered", [corner(0, 1, 1), corner(0, 0, 1), corner(1, 0, 1), corner(1, 1, 1)]),
        ("west-ordered", [corner(0, 1, 0), corner(0, 0, 0), corner(0, 0, 1), corner(0, 1, 1)]),
        ("east-ordered", [corner(1, 1, 1), corner(1, 0, 1), corner(1, 0, 0), corner(1, 1, 0)]),
        # The same face wound backwards: the normal flips and so does the
        # direction the bakery settles on.
        ("up-reversed", [corner(1, 1, 0), corner(1, 1, 1), corner(0, 1, 1), corner(0, 1, 0)]),
        # Shuffled corners of a real face, which is what a rotation leaves
        # behind and what the rewind exists to repair.
        ("up-shuffled", [corner(1, 1, 1), corner(0, 1, 0), corner(1, 1, 0), corner(0, 1, 1)]),
        # A partial box rather than the whole block.
        ("slab-top", [corner(0, 0.5, 0), corner(0, 0.5, 1), corner(1, 0.5, 1), corner(1, 0.5, 0)]),
        # Degenerate input: all four corners equal, so the normal is not finite.
        ("degenerate", [corner(0.5, 0.5, 0.5)] * 4),
        # Collinear corners, another way to lose the normal.
        ("collinear", [corner(0, 0, 0), corner(1, 0, 0), corner(2, 0, 0), corner(3, 0, 0)]),
        # A diagonal quad, whose nearest direction is not exact.
        ("diagonal", [corner(0, 0, 0), corner(1, 0, 0), corner(1, 1, 1), corner(0, 1, 1)]),
    ]

    euler_rows = []
    triples = [
        (0.0, 0.0, 0.0),
        (22.5, 0.0, 0.0), (0.0, 22.5, 0.0), (0.0, 0.0, 22.5),
        (90.0, 0.0, 0.0), (0.0, 90.0, 0.0), (0.0, 0.0, 90.0),
        (22.5, 45.0, 0.0), (0.0, 22.5, 45.0), (45.0, 0.0, 22.5),
        (22.5, 22.5, 22.5), (45.0, 45.0, 45.0), (90.0, 90.0, 90.0),
        (-22.5, 30.0, -7.5), (180.0, 90.0, -45.0), (360.0, 360.0, 360.0),
    ]
    for x, y, z in triples:
        for rescale in (False, True):
            euler_rows.append({
                "label": f"{x}-{y}-{z}-{'rescale' if rescale else 'plain'}",
                "x": bits(x),
                "y": bits(y),
                "z": bits(z),
                "rescale": rescale,
            })

    uv_rows = []
    uv_sets = [
        ("full", (0.0, 0.0, 16.0, 16.0)),
        ("partial", (2.0, 4.0, 14.0, 12.0)),
        # A flipped rectangle, which models use to mirror a texture.
        ("flipped-u", (16.0, 0.0, 0.0, 16.0)),
        ("flipped-both", (16.0, 16.0, 0.0, 0.0)),
        ("fractional", (0.5, 1.25, 15.5, 14.75)),
    ]
    # Accepted rotations, values that wrap onto them, and values that do not.
    degrees_list = [0, 90, 180, 270, 360, 450, -90, -180, -270, -360, 720, 45, 30, 1, -45, 100]
    for name, uvs in uv_sets:
        for degrees in degrees_list:
            uv_rows.append({
                "label": f"{name}-{degrees}",
                "uvs": [bits(v) for v in uvs],
                "degrees": degrees,
            })

    cube_all = (
        '{"parent":"minecraft:block/cube_all",'
        '"textures":{"all":"minecraft:block/stone"}}'
    )
    full_element = (
        '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
        '"faces":{"down":{"texture":"#all","cullface":"down"},'
        '"up":{"texture":"#all","cullface":"up"},'
        '"north":{"texture":"#all","cullface":"north"},'
        '"south":{"texture":"#all","cullface":"south"},'
        '"west":{"texture":"#all","cullface":"west"},'
        '"east":{"texture":"#all","cullface":"east"}}}]}'
    )
    display_right = ('{"display":{"thirdperson_righthand":{"rotation":[75,45,0],'
                     '"translation":[0,2.5,0],"scale":[0.375,0.375,0.375]},'
                     '"firstperson_righthand":{"rotation":[0,45,0],"scale":[0.4,0.4,0.4]},'
                     '"gui":{"rotation":[30,225,0],"scale":[0.625,0.625,0.625]},'
                     '"ground":{"translation":[0,3,0],"scale":[0.25,0.25,0.25]}}}')
    display_clamped = ('{"display":{"fixed":{"translation":[100,-100,8],"scale":[9,-9,0.5]},'
                       '"head":{"rotation":[0,180,0],"translation":[0,13,7]},'
                       '"on_shelf":{"translation":[0,0,-1]},'
                       '"firstperson_lefthand":{"rotation":[0,-90,25]},'
                       '"thirdperson_lefthand":{"scale":[1,1,1]}}}')
    model_texts = [
        ("display-right-hands", display_right),
        ("display-clamped-and-left", display_clamped),
        ("empty-object", "{}"),
        ("parent-only", cube_all),
        ("full-cube", full_element),
        ("ambient-occlusion-false", '{"ambientocclusion":false,' + full_element[1:]),
        ("gui-light-front", '{"gui_light":"front",' + full_element[1:]),
        # A face may carry its own uv rectangle, a tint and a rotation.
        ("face-detail", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                        '"faces":{"up":{"texture":"#t","uv":[1,2,15,14],'
                        '"tintindex":3,"rotation":270}}}]}'),
        # Rotation blocks, both spellings.
        ("rotation-axis", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                          '"rotation":{"origin":[8,8,8],"axis":"y","angle":22.5,"rescale":true},'
                          '"faces":{"up":{"texture":"#t"}}}]}'),
        ("rotation-euler", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                           '"rotation":{"origin":[0,8,16],"x":22.5,"y":45,"z":0},'
                           '"faces":{"up":{"texture":"#t"}}}]}'),
        ("shade-false", '{"elements":[{"from":[0,0,0],"to":[16,16,16],"shade":false,'
                        '"faces":{"up":{"texture":"#t"}}}]}'),
        ("light-emission", '{"elements":[{"from":[0,0,0],"to":[16,16,16],"light_emission":15,'
                           '"faces":{"up":{"texture":"#t"}}}]}'),
        # The element grammar permits reaching outside the block, up to a limit.
        ("bounds-edge", '{"elements":[{"from":[-16,-16,-16],"to":[32,32,32],'
                        '"faces":{"up":{"texture":"#t"}}}]}'),
        ("textures-mixed", '{"textures":{"all":"minecraft:block/stone","side":"#all",'
                           '"particle":{"sprite":"minecraft:block/dirt","force_translucent":true}}}'),
        ("multiple-elements", '{"elements":[' +
            '{"from":[0,0,0],"to":[16,8,16],"faces":{"up":{"texture":"#a"}}},' +
            '{"from":[0,8,0],"to":[16,16,16],"faces":{"down":{"texture":"#b"}}}]}'),
    ]
    rejected_models = [
        ("bounds-too-low", '{"elements":[{"from":[-17,0,0],"to":[16,16,16],'
                           '"faces":{"up":{"texture":"#t"}}}]}'),
        ("bounds-too-high", '{"elements":[{"from":[0,0,0],"to":[33,16,16],'
                            '"faces":{"up":{"texture":"#t"}}}]}'),
        ("short-vector", '{"elements":[{"from":[0,0],"to":[16,16,16],'
                         '"faces":{"up":{"texture":"#t"}}}]}'),
        ("no-faces", '{"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{}}]}'),
        ("unknown-facing", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                           '"faces":{"sideways":{"texture":"#t"}}}]}'),
        ("shade-not-boolean", '{"elements":[{"from":[0,0,0],"to":[16,16,16],"shade":1,'
                              '"faces":{"up":{"texture":"#t"}}}]}'),
        ("light-emission-high", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                                '"light_emission":16,"faces":{"up":{"texture":"#t"}}}]}'),
        ("light-emission-negative", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                                    '"light_emission":-1,"faces":{"up":{"texture":"#t"}}}]}'),
        ("rotation-empty", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                           '"rotation":{"origin":[8,8,8]},"faces":{"up":{"texture":"#t"}}}]}'),
        ("rotation-bad-axis", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                              '"rotation":{"origin":[8,8,8],"axis":"w","angle":22.5},'
                              '"faces":{"up":{"texture":"#t"}}}]}'),
        ("uv-short", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                     '"faces":{"up":{"texture":"#t","uv":[1,2,3]}}}]}'),
        ("face-rotation-invalid", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                                  '"faces":{"up":{"texture":"#t","rotation":45}}}]}'),
        ("missing-texture", '{"elements":[{"from":[0,0,0],"to":[16,16,16],'
                            '"faces":{"up":{}}}]}'),
        ("not-an-object", "[]"),
        ("trailing-comma", '{"elements":[],}'),
    ]

    state_rows = []
    # UV lock is left out; the reason is recorded in the slice document.
    for x in (0, 90, 180, 270, 360, -90, -180, 450):
        for y in (0, 90, 180, 270, 360, -90, -180, 450):
            state_rows.append({
                "label": f"x{x}-y{y}",
                "x": x,
                "y": y,
                "z": 0,
                "uvlock": False,
            })
    # With UV lock on, the face transformations stop being identity and come
    # from a table the reference builds once from quaternions. Every quadrant
    # combination is driven, because the table is keyed by the composed group
    # and several combinations land on the same element.
    for x in (0, 90, 180, 270):
        for y in (0, 90, 180, 270):
            for z in (0, 90, 180, 270):
                state_rows.append({
                    "label": f"uvlock-x{x}-y{y}-z{z}",
                    "x": x,
                    "y": y,
                    "z": z,
                    "uvlock": True,
                })
    # The third axis on its own, and combined with the other two, because the
    # group composition is not commutative.
    for z in (90, 180, 270):
        state_rows.append({"label": f"z{z}", "x": 0, "y": 0, "z": z, "uvlock": False})
    for x in (0, 90, 180, 270):
        for y in (0, 90, 180, 270):
            for z in (90, 180, 270):
                state_rows.append({
                    "label": f"x{x}-y{y}-z{z}",
                    "x": x,
                    "y": y,
                    "z": z,
                    "uvlock": False,
                })

    blockstate_texts = [
        # The plain shape: one key, one model.
        ("single", '{"variants": {"": {"model": "minecraft:block/stone"}}}'),
        ("property-key", '{"variants": {"facing=north": {"model": "minecraft:block/furnace"}}}'),
        ("rotation", '{"variants": {"": {"model": "minecraft:block/stairs",'
                     ' "x": 90, "y": 180, "z": 270, "uvlock": true}}}'),
        # Angles are taken modulo a full turn before the quadrant is chosen.
        ("rotation-wrapped", '{"variants": {"": {"model": "minecraft:block/a",'
                             ' "x": 450, "y": -90}}}'),
        ("rotation-bad", '{"variants": {"": {"model": "minecraft:block/a", "x": 45}}}'),
        # A list is a weighted choice; the weight defaults to one.
        ("weighted", '{"variants": {"": [{"model": "minecraft:block/a"},'
                     ' {"model": "minecraft:block/b", "weight": 3}]}}'),
        ("weighted-zero", '{"variants": {"": [{"model": "minecraft:block/a", "weight": 0}]}}'),
        ("weighted-negative",
         '{"variants": {"": [{"model": "minecraft:block/a", "weight": -1}]}}'),
        ("weighted-empty", '{"variants": {"": []}}'),
        ("several-keys", '{"variants": {"half=bottom": {"model": "minecraft:block/a"},'
                         ' "half=top": {"model": "minecraft:block/b"}}}'),
        # A model name is required and must be a valid identifier.
        ("no-model", '{"variants": {"": {}}}'),
        ("bad-model", '{"variants": {"": {"model": "Not An Id"}}}'),
        ("empty-variants", '{"variants": {}}'),
        ("neither", "{}"),
        ("both", '{"variants": {"": {"model": "minecraft:block/a"}},'
                 ' "multipart": [{"apply": {"model": "minecraft:block/b"}}]}'),
        # Multipart: a condition is optional, and its two forms nest.
        ("multipart-plain", '{"multipart": [{"apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-when", '{"multipart": [{"when": {"north": "true"},'
                           ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-multi-term", '{"multipart": [{"when": {"facing": "north|south"},'
                                 ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-negated", '{"multipart": [{"when": {"facing": "!north"},'
                              ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-or", '{"multipart": [{"when": {"OR": [{"north": "true"},'
                         ' {"south": "true"}]}, "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-and", '{"multipart": [{"when": {"AND": [{"north": "true"},'
                          ' {"south": "true"}]}, "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-nested", '{"multipart": [{"when": {"OR": [{"north": "true"},'
                             ' {"AND": [{"south": "true"}, {"east": "false"}]}]},'
                             ' "apply": {"model": "minecraft:block/a"}}]}'),
        # A number or a boolean in a term is taken as its text.
        ("multipart-boolean", '{"multipart": [{"when": {"north": true},'
                              ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-number", '{"multipart": [{"when": {"level": 3},'
                             ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-float", '{"multipart": [{"when": {"level": 3.5},'
                            ' "apply": {"model": "minecraft:block/a"}}]}'),
        # Empty terms and empty maps are refused.
        ("multipart-empty-term", '{"multipart": [{"when": {"facing": ""},'
                                 ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-empty-piece", '{"multipart": [{"when": {"facing": "north|"},'
                                  ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-empty-when", '{"multipart": [{"when": {},'
                                 ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-empty", '{"multipart": []}'),
        ("multipart-no-apply", '{"multipart": [{"when": {"north": "true"}}]}'),
        # A combiner holds exactly one operation.
        ("multipart-two-operations", '{"multipart": [{"when": {"OR": [{"north": "true"}],'
                                     ' "AND": [{"south": "true"}]},'
                                     ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-lowercase-operation", '{"multipart": [{"when": {"or": [{"north": "true"}]},'
                                          ' "apply": {"model": "minecraft:block/a"}}]}'),
        # An operation alongside a property: does the combiner take the one
        # key it knows and drop the rest, or refuse the whole condition?
        ("multipart-operation-and-property",
         '{"multipart": [{"when": {"OR": [{"north": "true"}], "north": "true"},'
         ' "apply": {"model": "minecraft:block/a"}}]}'),
        ("multipart-empty-operation-list",
         '{"multipart": [{"when": {"AND": []}, "apply": {"model": "minecraft:block/a"}}]}'),
        # The selector string is not validated while parsing, only later when
        # it meets a state definition.
        ("nonsense-selector", '{"variants": {"=,,=nope": {"model": "minecraft:block/a"}}}'),
        ("multipart-weighted", '{"multipart": [{"apply": [{"model": "minecraft:block/a"},'
                               ' {"model": "minecraft:block/b", "weight": 2}]}]}'),
        ("not-an-object", "[]"),
    ]
    # What a UV-locked face does to a texture coordinate. Every rotation and
    # every face is driven, because the tables differ per pair, and the sample
    # coordinates include the corners, the centre and points off the face.
    uv_lock_rows = []
    faces = ("down", "up", "north", "south", "west", "east")
    samples = (0.0, 1.0, 0.5, 0.25, 0.125, -0.5, 1.5)
    for x in (0, 90, 180, 270):
        for y in (0, 90, 180, 270):
            for facing in faces:
                for u in samples:
                    uv_lock_rows.append({
                        "label": f"x{x}-y{y}-{facing}-u{u}",
                        "x": x, "y": y, "z": 0, "uvlock": True, "facing": facing,
                        "u": float_bits(u), "v": float_bits(0.375),
                    })
    # The third axis, and the unlocked case where the coordinate passes
    # through untouched.
    for z in (90, 180, 270):
        for facing in faces:
            uv_lock_rows.append({
                "label": f"z{z}-{facing}",
                "x": 0, "y": 0, "z": z, "uvlock": True, "facing": facing,
                "u": float_bits(0.25), "v": float_bits(0.375),
            })
    for facing in faces:
        uv_lock_rows.append({
            "label": f"unlocked-{facing}",
            "x": 90, "y": 180, "z": 0, "uvlock": False, "facing": facing,
            "u": float_bits(0.25), "v": float_bits(0.375),
        })
    # Which face a vector points at. The search starts at north with the
    # smallest positive float as the bar, so a vector with no positive
    # component at all comes back north rather than as nothing.
    nearest_rows = [
        ("down", 0.0, -1.0, 0.0), ("up", 0.0, 1.0, 0.0),
        ("north", 0.0, 0.0, -1.0), ("south", 0.0, 0.0, 1.0),
        ("west", -1.0, 0.0, 0.0), ("east", 1.0, 0.0, 0.0),
        ("zero", 0.0, 0.0, 0.0),
        ("all-negative", -1.0, -1.0, -1.0),
        ("diagonal", 1.0, 1.0, 0.0),
        ("tie-up-east", 1.0, 1.0, 1.0),
        ("tiny-positive", 1e-44, 0.0, 0.0),
        ("below-the-bar", 1e-46, 0.0, 0.0),
        ("negative-zero", -0.0, -0.0, -0.0),
        ("nan", float("nan"), 0.0, 0.0),
        ("infinity", float("inf"), 0.0, 0.0),
        ("negative-infinity", float("-inf"), 0.0, 0.0),
        ("mixed", 0.5, -0.9, 0.2),
    ]
    yield {
        "case_id": "model/nearest-direction",
        "family": "model",
        "operation": "nearest_direction",
        "rows": [
            {"label": label, "x": float_bits(x), "y": float_bits(y), "z": float_bits(z)}
            for label, x, y, z in nearest_rows
        ],
    }

    # Where a face's cullface direction lands once the variant is turned.
    cull_rows = []
    for x in (0, 90, 180, 270):
        for y in (0, 90, 180, 270):
            for z in (0, 90, 180, 270):
                for facing in ("down", "up", "north", "south", "west", "east"):
                    cull_rows.append({
                        "label": f"x{x}-y{y}-z{z}-{facing}",
                        "x": x, "y": y, "z": z, "facing": facing,
                    })
    yield {
        "case_id": "model/cull-rotation",
        "family": "model",
        "operation": "cull_rotation",
        "rows": cull_rows,
    }

    yield {
        "case_id": "model/uv-lock",
        "family": "model",
        "operation": "uv_lock",
        "rows": uv_lock_rows,
    }

    yield {
        "case_id": "model/blockstate-parse",
        "family": "model",
        "operation": "blockstate_parse",
        "rows": [{"label": label, "json": text} for label, text in blockstate_texts],
    }

    yield {
        "case_id": "model/model-state",
        "family": "model",
        "operation": "model_state",
        "rows": state_rows,
    }

    yield {
        "case_id": "model/parse-model",
        "family": "model",
        "operation": "parse_model",
        "rows": [
            {"label": label, "text": text.encode("utf-8").hex()}
            for label, text in model_texts + rejected_models
        ],
    }

    yield {
        "case_id": "model/face-uv-rotation",
        "family": "model",
        "operation": "face_uv_rotation",
        "rows": uv_rows,
    }

    yield {
        "case_id": "model/euler-rotation",
        "family": "model",
        "operation": "euler_rotation",
        "rows": euler_rows,
    }

    yield {
        "case_id": "model/quad-winding",
        "family": "model",
        "operation": "quad_winding",
        "rows": [{"label": label, "corners": corners} for label, corners in winding_rows],
    }

    yield {
        "case_id": "model/element-rotation",
        "family": "model",
        "operation": "element_rotation",
        "rows": rotation_rows,
    }

    yield {
        "case_id": "model/face-geometry",
        "family": "model",
        "operation": "face_geometry",
        "rows": geometry_rows,
    }

    yield {
        "case_id": "model/texture-slots",
        "family": "model",
        "operation": "texture_slots",
        "rows": [{"label": label, "chain": chain} for label, chain in rows],
    }


def json_cases():
    """Gson in strict mode: what the reference accepts as a model file.

    The client reads every model, blockstate and pack file through a reader
    with `Strictness.STRICT`, so the leniencies Gson allows elsewhere - single
    quotes, comments, trailing commas, unquoted keys, NaN - are all rejections
    here. The rows below cover the accepted grammar, the number literals kept
    verbatim, escape and surrogate handling, and the shapes that must fail.
    """
    accepted = [
        ("empty-object", "{}"),
        ("empty-array", "[]"),
        ("flat-object", '{"a":1,"b":"two","c":true,"d":null}'),
        ("nested", '{"a":{"b":[1,2,{"c":3}]}}'),
        ("model-shaped", '{"parent":"minecraft:block/cube_all",'
                         '"textures":{"all":"minecraft:block/stone"}}'),
        ("whitespace", ' \t\r\n { "a" : [ 1 , 2 ] } \n '),
        # Key order is preserved, and a repeated key keeps the last value.
        ("key-order", '{"z":1,"a":2,"m":3}'),
        ("duplicate-key", '{"a":1,"a":2}'),
        # Does the repeated key keep its original slot or move to the end?
        ("duplicate-key-order", '{"a":1,"b":2,"a":3}'),
        # Number literals are kept as written until a caller asks for a type.
        ("integers", '[0,-1,42,1000000]'),
        ("decimals", '[1.5,-0.25,0.0,100.125]'),
        ("exponents", '[1e3,1E3,1e+3,1e-3,1.5e10,-2.5E-7]'),
        ("large-number", '[9007199254740993,1e400]'),
        ("negative-zero", "[-0,-0.0]"),
        ("deep-nesting", "[" * 20 + "1" + "]" * 20),
        ("unicode-escape", '["\\u0041\\u00e7\\u4e2d"]'),
        ("surrogate-pair", '["\\ud83d\\ude00"]'),
        ("lone-surrogate", '["\\ud83d"]'),
        ("escapes", '["\\"\\\\\\/\\b\\f\\n\\r\\t"]'),
        ("raw-utf8", '["\u00e7\u4e2d\U0001f600"]'),
        ("empty-string-key", '{"":1}'),
        ("string-with-space", '{"a b":"c d"}'),
        ("top-level-string", '"bare"'),
        ("top-level-number", "42"),
        ("top-level-true", "true"),
        ("top-level-null", "null"),
    ]
    rejected = [
        ("trailing-comma-object", '{"a":1,}'),
        ("trailing-comma-array", "[1,2,]"),
        ("single-quotes", "{'a':1}"),
        ("unquoted-key", "{a:1}"),
        ("line-comment", '{"a":1} // note'),
        ("block-comment", '{/* note */"a":1}'),
        ("nan", "[NaN]"),
        ("infinity", "[Infinity]"),
        ("leading-plus", "[+1]"),
        ("leading-zero", "[01]"),
        ("hex-number", "[0x10]"),
        ("bare-decimal-point", "[.5]"),
        ("trailing-decimal-point", "[1.]"),
        ("unterminated-string", '{"a":"b}'),
        ("unterminated-object", '{"a":1'),
        ("bad-escape", '["\\x41"]'),
        ("raw-control-char", '["a\u0001b"]'),
        ("trailing-content", '{"a":1} {"b":2}'),
        ("trailing-garbage", "[1] x"),
        ("empty-input", ""),
        ("only-whitespace", "   "),
        ("missing-colon", '{"a" 1}'),
        ("missing-comma", '{"a":1 "b":2}'),
        ("double-comma", "[1,,2]"),
    ]

    literals = [
        # plain integers, including the boundaries of each width
        "0", "-0", "1", "-1", "42", "2147483647", "-2147483648",
        "2147483648", "-2147483649", "9223372036854775807", "-9223372036854775808",
        "9223372036854775808", "9007199254740993", "123456789012345678901234567890",
        # decimals, where an integer request has to drop the fraction
        "1.0", "1.5", "-1.5", "0.0", "-0.0", "2.5", "-2.5", "0.1",
        "16.0", "0.0625", "-0.0625", "1.9999999", "2147483647.5",
        # exponents, including values that overflow one width but not another
        "1e0", "1E0", "1e1", "1e-1", "1e10", "1e38", "1e39", "1e308", "1e309",
        "1e400", "-1e400", "1e-45", "1e-46", "1e-324", "3.4028235e38",
        "3.4028236e38", "1.7976931348623157e308",
        # the smallest steps a float can still tell apart
        "0.1000000014901161", "16777217", "16777216",
        # an exponent that moves the point back across the digits
        "12345e-3", "12345e-5", "12345e-6", "0.0000001e10", "123e20",
        "1e18", "1e19", "1e20", "999999999999999999999e-3",
        # long digit strings, where the low bits have to survive the wrap
        "18446744073709551616", "18446744073709551617",
        "340282366920938463463374607431768211456",
        "-123456789012345678901234567890", "-9007199254740993",
    ]
    yield {
        "case_id": "json/number",
        "family": "json",
        "operation": "number",
        "rows": [{"label": literal, "literal": literal} for literal in literals],
    }

    yield {
        "case_id": "json/parse",
        "family": "json",
        "operation": "parse",
        "rows": [
            {"label": label, "text": text.encode("utf-8").hex()}
            for label, text in accepted + rejected
        ],
    }

def block_cases():
    """The block registry, read from the reference rather than rebuilt.

    The input is tiny because the oracle enumerates the registry itself; the
    expected output is the whole table, which is what the Rust side has to
    reproduce."""
    yield {
        "case_id": "block/state-definitions",
        "family": "block",
        "operation": "state_definitions",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/declarations",
        "family": "block",
        "operation": "block_declarations",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/item-declarations",
        "family": "block",
        "operation": "item_declarations",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/collision-shapes",
        "family": "block",
        "operation": "collision_shapes",
        "rows": [{"label": "registry"}],
    }

    yield {
        "case_id": "block/outline-shapes",
        "family": "block",
        "operation": "outline_shapes",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/movement-properties",
        "family": "block",
        "operation": "movement_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/offset-properties",
        "family": "block",
        "operation": "offset_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/fluid-overlay-blocks",
        "family": "block",
        "operation": "fluid_overlay_blocks",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/motion-blocking",
        "family": "block",
        "operation": "motion_blocking",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/fluid-flow-properties",
        "family": "block",
        "operation": "fluid_flow_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/light-properties",
        "family": "block",
        "operation": "light_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/face-occlusion",
        "family": "block",
        "operation": "face_occlusion",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/destroy-properties",
        "family": "block",
        "operation": "destroy_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/item-tools",
        "family": "block",
        "operation": "item_tools",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/item-stacking",
        "family": "block",
        "operation": "item_stacking",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/item-components",
        "family": "block",
        "operation": "item_components",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/tooltip-registries",
        "family": "block",
        "operation": "tooltip_registries",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/potions",
        "family": "block",
        "operation": "potions",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/creative-tabs",
        "family": "block",
        "operation": "creative_tabs",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/behaviours",
        "family": "block",
        "operation": "block_behaviours",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/tints",
        "family": "block",
        "operation": "block_tints",
        "rows": [{"label": "registry"}],
    }


def movement_cases():
    """The reference's own client player, ticked through scenes.

    Each row is a scene, a start pose and the keys held on every tick. The
    expected output is everything the tick leaves behind — position, motion,
    the collision flags, the pose — and every packet the player asked to
    send, as the bytes the packet's own codec writes. This is the whole
    client tick, not its parts: input, `aiStep`, `travel`, `move` and the
    position packets, in the order the reference runs them."""

    def floor(y=63, block="minecraft:stone", low=-16, high=31):
        return [{"x": x, "y": y, "z": z, "block": block}
                for x in range(low, high + 1) for z in range(low, high + 1)]

    def column(x, z, low, high, block="minecraft:stone", properties=None):
        out = []
        for y in range(low, high + 1):
            placed = {"x": x, "y": y, "z": z, "block": block}
            if properties:
                placed["properties"] = properties
            out.append(placed)
        return out

    def keys(count, **held):
        tick = {name: held.get(name, False)
                for name in ("forward", "backward", "left", "right", "jump", "sneak", "sprint")}
        return [dict(tick) for _ in range(count)]

    def turned(ticks, y_rot=None, x_rot=None):
        for tick in ticks:
            if y_rot is not None:
                tick["y_rot"] = float_bits(y_rot)
            if x_rot is not None:
                tick["x_rot"] = float_bits(x_rot)
        return ticks

    def scene(label, blocks, ticks, start=(0.5, 64.0, 0.5), y_rot=0.0, x_rot=0.0,
              game_mode="survival", auto_jump=False):
        return {
            "label": label,
            "blocks": blocks,
            "chunk_radius": 1,
            "entity_id": 7,
            "x": float_bits(start[0], "f64"),
            "y": float_bits(start[1], "f64"),
            "z": float_bits(start[2], "f64"),
            "y_rot": float_bits(y_rot),
            "x_rot": float_bits(x_rot),
            "game_mode": game_mode,
            "auto_jump": auto_jump,
            "ticks": ticks,
        }

    wall = [b for x in range(-3, 4) for b in column(x, 4, 64, 66)]
    slab_step = [{"x": x, "y": 64, "z": z, "block": "minecraft:stone_slab"}
                 for x in range(-3, 4) for z in range(3, 8)]
    block_step = [b for x in range(-3, 4) for z in range(3, 8) for b in column(x, z, 64, 64)]
    ledge = [{"x": x, "y": 63, "z": z, "block": "minecraft:stone"}
             for x in range(-3, 4) for z in range(-3, 2)]
    ladder = column(0, 1, 64, 70, "minecraft:ladder", {"facing": "north"})
    ladder_wall = column(0, 2, 64, 70)
    pool = [{"x": x, "y": y, "z": z, "block": "minecraft:water"}
            for x in range(-6, 7) for z in range(-6, 7) for y in range(64, 67)]
    ceiling = [{"x": x, "y": 65, "z": z, "block": "minecraft:stone"}
               for x in range(-4, 5) for z in range(-4, 9)]
    low_roof = [{"x": x, "y": 66, "z": z, "block": "minecraft:stone"}
                for x in range(-2, 3) for z in range(-2, 3)]
    honey_wall = column(1, 0, 64, 72, "minecraft:honey_block")
    stream = ([{"x": x, "y": 64, "z": -3, "block": "minecraft:water"} for x in range(-1, 2)]
              + [{"x": x, "y": 64, "z": z, "block": "minecraft:water",
                  "properties": {"level": str(z + 3)}}
                 for x in range(-1, 2) for z in range(-2, 4)])
    lava = [{"x": x, "y": 64, "z": z, "block": "minecraft:lava"}
            for x in range(-3, 4) for z in range(-3, 8)]
    trapdoor_ladder = (column(0, 1, 64, 65, "minecraft:ladder", {"facing": "north"})
                       + [{"x": 0, "y": 66, "z": 1, "block": "minecraft:oak_trapdoor",
                           "properties": {"facing": "north", "open": "true"}}])
    side_wall = [b for x in range(-16, 4) for b in column(x, 1, 64, 65)]
    # A waterfall two wide against a wall, and a stream flowing past fences.
    waterfall = ([{"x": x, "y": y, "z": 0, "block": "minecraft:water", "properties": {"level": "8"}}
                  for x in range(0, 2) for y in range(64, 72)]
                 + [b for x in range(-1, 3) for b in column(x, 1, 64, 72)])
    fenced_stream = stream + [{"x": 2, "y": 64, "z": z, "block": "minecraft:oak_fence"}
                              for z in range(-2, 4)]
    edge_ladder = column(0, 2, 56, 62, "minecraft:ladder", {"facing": "north"}) + column(0, 3, 56, 63)
    plant_names = ["minecraft:short_grass", "minecraft:fern", "minecraft:dandelion",
                   "minecraft:sweet_berry_bush", "minecraft:oak_sapling"]
    plants = [{"x": x, "y": 64, "z": z, "block": plant_names[(x + z) % len(plant_names)]}
              for x in range(-2, 3) for z in range(-2, 12)]
    plants += [{"x": x, "y": y, "z": 12, "block": "minecraft:tall_grass",
                "properties": {"half": "lower" if y == 64 else "upper"}}
               for x in range(-2, 3) for y in (64, 65)]
    # Blocks that hold a player back from inside them.
    powder_pit = [{"x": x, "y": y, "z": z, "block": "minecraft:powder_snow"}
                  for x in range(-3, 4) for z in range(-3, 8) for y in range(61, 64)]
    powder_ahead = [{"x": x, "y": 64, "z": z, "block": "minecraft:powder_snow"}
                    for x in range(-3, 4) for z in range(2, 5)]
    webs = [{"x": x, "y": y, "z": 2, "block": "minecraft:cobweb"}
            for x in range(-1, 2) for y in range(64, 66)]
    berries = [{"x": x, "y": 64, "z": z, "block": "minecraft:sweet_berry_bush",
                "properties": {"age": "1"}} for x in range(-1, 2) for z in range(2, 4)]
    bubbles = ([{"x": 0, "y": 63, "z": 0, "block": "minecraft:soul_sand"}]
               + column(0, 0, 64, 70, "minecraft:bubble_column", {"drag": "false"})
               + [{"x": 1, "y": 63, "z": 0, "block": "minecraft:magma_block"}]
               + column(1, 0, 64, 70, "minecraft:bubble_column", {"drag": "true"})
               + [b for x in range(-1, 3) for z in (-1, 1) for b in column(x, z, 64, 70)]
               + column(-1, 0, 64, 70) + column(2, 0, 64, 70))

    rows = [
        scene("stand-still", floor(), keys(6)),
        scene("walk-forward", floor(), keys(20, forward=True)),
        scene("walk-backward", floor(), keys(12, backward=True)),
        scene("strafe-left", floor(), keys(12, left=True)),
        scene("walk-diagonal", floor(), keys(12, forward=True, right=True)),
        scene("walk-turned-45", floor(), keys(12, forward=True), y_rot=45.0),
        scene("walk-then-turn", floor(),
              keys(6, forward=True) + turned(keys(8, forward=True), y_rot=90.0)),
        scene("walk-then-stop", floor(), keys(8, forward=True) + keys(10)),
        scene("sprint", floor(), keys(20, forward=True, sprint=True)),
        scene("sprint-then-release", floor(),
              keys(10, forward=True, sprint=True) + keys(6, forward=True)),
        scene("sneak-walk", floor(), keys(14, forward=True, sneak=True)),
        scene("jump-once", floor(), keys(1, jump=True) + keys(14)),
        scene("jump-held", floor(), keys(30, jump=True)),
        scene("walk-jump", floor(), keys(20, forward=True, jump=True)),
        scene("sprint-jump", floor(), keys(24, forward=True, sprint=True, jump=True)),
        scene("walk-into-wall", floor() + wall, keys(20, forward=True)),
        scene("step-onto-slab", floor() + slab_step, keys(16, forward=True)),
        scene("walk-into-block-step", floor() + block_step, keys(16, forward=True)),
        scene("jump-onto-block", floor() + block_step,
              keys(4, forward=True) + keys(10, forward=True, jump=True)),
        scene("fall-onto-floor", floor(), keys(30), start=(0.5, 72.0, 0.5)),
        scene("walk-off-ledge", ledge, keys(20, forward=True)),
        scene("sneak-at-ledge", ledge, keys(24, forward=True, sneak=True)),
        scene("walk-on-ice", floor(block="minecraft:ice"), keys(10, forward=True) + keys(12)),
        scene("walk-on-soul-sand", floor(block="minecraft:soul_sand"), keys(16, forward=True)),
        scene("land-on-slime", floor(block="minecraft:slime_block"), keys(40),
              start=(0.5, 72.0, 0.5)),
        scene("walk-on-honey", floor(block="minecraft:honey_block"), keys(16, forward=True)),
        scene("climb-ladder", floor() + ladder + ladder_wall,
              keys(30, forward=True), y_rot=0.0),
        scene("swim", floor(block="minecraft:stone") + pool, keys(20, forward=True),
              start=(0.5, 65.0, 0.5)),
        scene("swim-up", floor(block="minecraft:stone") + pool, keys(20, jump=True),
              start=(0.5, 64.2, 0.5)),
        scene("auto-jump", floor() + block_step, keys(20, forward=True), auto_jump=True),
        scene("look-down-walk", floor(), turned(keys(10, forward=True), x_rot=90.0)),
        scene("creative-double-jump-fly", floor(),
              keys(1, jump=True) + keys(2) + keys(1, jump=True) + keys(10, jump=True)
              + keys(10, forward=True), game_mode="creative"),
        scene("spectator-float", floor(), keys(10, forward=True, jump=True),
              game_mode="spectator"),
        scene("sprint-swim", floor() + pool, keys(24, forward=True, sprint=True)),
        scene("sneak-sink", floor() + pool, keys(16, sneak=True), start=(0.5, 65.5, 0.5)),
        scene("fall-into-water", floor() + pool, keys(40), start=(0.5, 90.0, 0.5)),
        scene("fall-far", floor(), keys(40), start=(0.5, 95.0, 0.5)),
        scene("crawl-under-ceiling", floor() + ceiling, keys(16, forward=True)),
        scene("jump-under-ceiling", floor() + low_roof, keys(14, jump=True)),
        scene("honey-wall-slide", floor() + honey_wall,
              keys(30, forward=True), start=(0.5, 72.0, 0.5), y_rot=-90.0),
        scene("flowing-water-push", floor() + stream, keys(20)),
        scene("waterfall-swim", floor() + waterfall,
              keys(10, forward=True) + keys(20, forward=True, jump=True),
              start=(0.5, 66.0, 0.5)),
        scene("stream-past-fence", floor() + fenced_stream, keys(20, left=True)),
        scene("lava-wade", floor() + lava, keys(16, forward=True)),
        scene("trapdoor-ladder", floor() + trapdoor_ladder + ladder_wall,
              keys(40, forward=True)),
        scene("sprint-along-wall", floor() + side_wall,
              keys(16, forward=True, sprint=True), y_rot=85.0),
        scene("sneak-onto-slime", floor(block="minecraft:slime_block"),
              keys(24, sneak=True), start=(0.5, 68.0, 0.5)),
        scene("creative-fly-sprint-land", floor(),
              keys(1, jump=True) + keys(1) + keys(1, jump=True)
              + keys(8, forward=True, sprint=True) + keys(10, sneak=True) + keys(4),
              game_mode="creative"),
        scene("auto-jump-sprint", floor() + block_step,
              keys(20, forward=True, sprint=True), auto_jump=True),
        scene("walk-off-ledge-into-ladder", ledge + edge_ladder, keys(30, forward=True)),
        scene("sink-in-powder-snow", [b for b in floor() if b["x"] < -3 or b["x"] > 3
                                      or b["z"] < -3 or b["z"] > 7] + powder_pit + floor(y=60),
              keys(10) + keys(20, forward=True) + keys(20, forward=True, jump=True)),
        scene("walk-into-powder-snow", floor() + powder_ahead, keys(30, forward=True)),
        scene("walk-through-cobweb", floor() + webs,
              keys(30, forward=True) + keys(6, forward=True, jump=True)),
        scene("walk-through-berry-bush", floor() + berries, keys(30, forward=True)),
        scene("sneak-in-plants", floor() + plants,
              keys(4, forward=True) + keys(10, forward=True, sneak=True) + keys(10, forward=True)
              + keys(6, sneak=True) + keys(6)),
        scene("ride-bubble-column", floor() + bubbles, keys(40), start=(0.5, 64.0, 0.5)),
        scene("sink-in-bubble-column", floor() + bubbles, keys(30), start=(1.5, 69.0, 0.5)),
    ]
    yield {
        "case_id": "movement/local-player-ticks",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": rows,
    }

    # Breaking: the attack key as `handleKeybinds` spends it, into the
    # reference's own game mode. The target is named, not picked. Each tick
    # reports the game mode's destroy state, the crack stage the level keeps,
    # the prediction sequence and the watched block states.
    target = {"x": 0, "y": 64, "z": 2, "face": "north"}
    second = {"x": 1, "y": 64, "z": 2, "face": "north"}
    watch = [target, second]

    def hits(count, aim=target, down=True, clicks=0, server=None, **held):
        ticks = keys(count, **held)
        for index, tick in enumerate(ticks):
            attack = {"clicks": clicks if index == 0 else 0, "down": down,
                      "watch": [{"x": w["x"], "y": w["y"], "z": w["z"]} for w in watch]}
            if aim is not None:
                attack["target"] = aim
            if server and index == 0:
                attack["server"] = server
            tick["attack"] = attack
        return ticks

    def breaking(label, blocks, ticks, main_hand=None, effects=None, **kwargs):
        row = scene(label, floor() + blocks, ticks, **kwargs)
        row["breaking"] = {}
        if main_hand:
            row["breaking"]["main_hand"] = main_hand
        if effects:
            row["breaking"]["effects"] = effects
        return row

    def at(place, block, properties=None):
        placed = {"x": place["x"], "y": place["y"], "z": place["z"], "block": block}
        if properties:
            placed["properties"] = properties
        return placed

    air = {"x": 0, "y": 64, "z": 2, "block": "minecraft:air"}
    dirt = [at(target, "minecraft:dirt")]
    stone = [at(target, "minecraft:stone")]
    pool = [{"x": x, "y": y, "z": z, "block": "minecraft:water"}
            for x in range(-3, 4) for z in range(-3, 2) for y in range(64, 67)]
    breaking_rows = [
        breaking("dirt-bare-hand", dirt,
                 hits(1, clicks=1) + hits(19) + hits(1, server=[air, {"ack": 2}]) + hits(3)),
        breaking("stone-wooden-pickaxe", stone, hits(1, clicks=1) + hits(26),
                 main_hand="minecraft:wooden_pickaxe"),
        breaking("stone-wooden-shovel", stone, hits(1, clicks=1) + hits(12),
                 main_hand="minecraft:wooden_shovel"),
        breaking("stone-release", stone, hits(1, clicks=1) + hits(9) + hits(3, down=False)),
        breaking("switch-target", dirt + [at(second, "minecraft:dirt")],
                 hits(1, clicks=1) + hits(5) + hits(20, aim=second)),
        breaking("dirt-rejected", dirt,
                 hits(1, clicks=1) + hits(16)
                 + hits(1, server=[at(target, "minecraft:dirt"), {"ack": 2}]) + hits(4)),
        breaking("dirt-haste", dirt, hits(1, clicks=1) + hits(12),
                 effects=[{"effect": "minecraft:haste", "amplifier": 1}]),
        breaking("dirt-mining-fatigue", dirt, hits(1, clicks=1) + hits(12),
                 effects=[{"effect": "minecraft:mining_fatigue", "amplifier": 0}]),
        breaking("dirt-jumping", dirt, hits(1, clicks=1, jump=True) + hits(20, jump=True)),
        breaking("dirt-underwater", dirt + pool, hits(1, clicks=1) + hits(20),
                 start=(0.5, 64.0, 0.5)),
        breaking("waterlogged-slab-axe",
                 [at(target, "minecraft:oak_slab", {"waterlogged": "true"})],
                 hits(1, clicks=1) + hits(34), main_hand="minecraft:wooden_axe"),
        breaking("miss-then-hold", dirt,
                 hits(1, aim=None, clicks=1) + hits(3, aim=None, clicks=1)
                 + hits(8, clicks=1) + hits(2, down=False) + hits(1, clicks=1) + hits(3)),
        breaking("creative-hold", dirt + [at(second, "minecraft:dirt")],
                 hits(1, clicks=1) + hits(3) + hits(8, aim=second), game_mode="creative"),
        breaking("creative-sword", dirt, hits(1, clicks=1) + hits(6),
                 main_hand="minecraft:wooden_sword", game_mode="creative"),
        breaking("adventure-bare-hand", dirt, hits(1, clicks=1) + hits(4), game_mode="adventure"),
    ]
    # Light: the client level lights itself from nothing, then relights as
    # block updates arrive. Each row reports a box's block and sky light
    # after the start and after every update.
    def slab(low_x, high_x, low_z, high_z, low_y, high_y, block="minecraft:stone"):
        return [{"x": x, "y": y, "z": z, "block": block}
                for x in range(low_x, high_x + 1) for z in range(low_z, high_z + 1)
                for y in range(low_y, high_y + 1)]

    def lit(label, blocks, events, box):
        return {
            "label": label,
            "blocks": blocks,
            "chunk_radius": 1,
            "events": events,
            "box": {"x": box[0], "y": box[1], "z": box[2],
                    "size_x": box[3], "size_y": box[4], "size_z": box[5]},
        }

    def put(x, y, z, block, properties=None):
        placed = {"x": x, "y": y, "z": z, "block": block}
        if properties:
            placed["properties"] = properties
        return placed

    ground = slab(-8, 8, -8, 8, 61, 63)
    walls = ([b for b in slab(-3, 3, -3, 3, 64, 67)
              if abs(b["x"]) == 3 or abs(b["z"]) == 3 or b["y"] == 67])
    light_rows = [
        lit("dig-to-the-void", ground,
            [put(0, 63, 0, "minecraft:air"), put(0, 62, 0, "minecraft:air"),
             put(0, 61, 0, "minecraft:air"), put(0, 61, 0, "minecraft:stone")],
            (-3, 40, -3, 7, 28, 7)),
        lit("closed-room-glowstone", ground + walls,
            [put(0, 64, 0, "minecraft:glowstone"), put(0, 64, 0, "minecraft:air"),
             put(0, 67, 0, "minecraft:air"), put(0, 67, 0, "minecraft:stone"),
             put(2, 65, 2, "minecraft:glowstone")],
            (-4, 60, -4, 9, 10, 9)),
        lit("materials-under-sky", ground,
            [put(0, 64, 0, "minecraft:torch"),
             put(2, 64, 0, "minecraft:oak_slab", {"type": "bottom"}),
             put(2, 65, 0, "minecraft:oak_slab", {"type": "top"}),
             put(4, 64, 0, "minecraft:oak_leaves", {"persistent": "true"}),
             put(4, 65, 0, "minecraft:oak_leaves", {"persistent": "true"}),
             put(-2, 64, 0, "minecraft:water"),
             put(-4, 64, 0, "minecraft:glass"),
             put(-4, 65, 0, "minecraft:stone_stairs", {"facing": "north", "half": "bottom"}),
             put(0, 66, 2, "minecraft:stone"), put(0, 64, 0, "minecraft:air")],
            (-6, 60, -3, 13, 9, 7)),
        lit("chunk-edge", slab(8, 24, -4, 4, 61, 63),
            [put(15, 63, 0, "minecraft:air"), put(16, 63, 0, "minecraft:air"),
             put(16, 62, 0, "minecraft:glowstone"), put(16, 62, 0, "minecraft:air")],
            (12, 58, -2, 8, 9, 5)),
    ]
    yield {
        "case_id": "movement/light-changes",
        "family": "movement",
        "operation": "light_changes",
        "rows": light_rows,
    }
    # Using: the use key as `handleKeybinds` spends it, into the reference's
    # own game mode and block items. The aim is named, with where on the
    # face it lands.
    def aim_at(x, y, z, face, at):
        return {"x": x, "y": y, "z": z, "face": face,
                "location": {"x": float_bits(at[0], "f64"), "y": float_bits(at[1], "f64"),
                             "z": float_bits(at[2], "f64")}}

    def uses(count, aim, clicks=0, down=False, watch_at=(), **held):
        ticks = keys(count, **held)
        for index, tick in enumerate(ticks):
            attack = {"clicks": 0, "down": False,
                      "use": {"clicks": clicks if index == 0 else 0, "down": down},
                      "watch": [{"x": w[0], "y": w[1], "z": w[2]} for w in watch_at]}
            if aim is not None:
                attack["target"] = aim
            tick["attack"] = attack
        return ticks

    def using(label, blocks, ticks, main_hand=None, count=1, off_hand=None, **kwargs):
        row = scene(label, floor() + blocks, ticks, **kwargs)
        row["breaking"] = {}
        if main_hand:
            row["breaking"]["main_hand"] = main_hand
            row["breaking"]["main_hand_count"] = count
        if off_hand:
            row["breaking"]["off_hand"] = off_hand
        return row

    top_of = aim_at(0, 63, 2, "up", (0.5, 64.0, 2.5))
    around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3)]
    side_high = aim_at(0, 64, 3, "north", (0.5, 64.75, 3.0))
    side_low = aim_at(0, 64, 3, "north", (0.5, 64.25, 3.0))
    pool_under = [put(x, 64, z, "minecraft:water") for x in range(-2, 3) for z in range(1, 5)]
    using_rows = [
        using("stone-on-floor", [], uses(1, top_of, clicks=1, watch_at=around) + uses(12, top_of, down=True, watch_at=around),
              main_hand="minecraft:stone", count=4),
        using("empty-hand-on-stone", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around)),
        using("use-in-air", [], uses(1, None, clicks=1, watch_at=around) + uses(6, None, down=True, watch_at=around),
              main_hand="minecraft:stone", count=4),
        using("slab-bottom-then-double", [],
              uses(1, top_of, clicks=1, watch_at=around) + uses(5, top_of, watch_at=around)
              + uses(1, aim_at(0, 64, 2, "up", (0.5, 64.5, 2.5)), clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:oak_slab", count=4),
        using("slab-on-side-halves", [put(0, 64, 3, "minecraft:stone")],
              uses(1, side_high, clicks=1, watch_at=around) + uses(5, side_low, watch_at=around)
              + uses(1, aim_at(0, 64, 3, "west", (0.0, 64.25, 3.5)), clicks=1, watch_at=around + [(-1, 64, 3)]),
              main_hand="minecraft:stone_slab", count=4),
        using("log-axis-from-face", [put(0, 64, 3, "minecraft:stone")],
              uses(1, side_high, clicks=1, watch_at=around) + uses(5, top_of, watch_at=around)
              + uses(1, aim_at(0, 64, 3, "east", (1.0, 64.5, 3.5)), clicks=1, watch_at=around + [(1, 64, 3)]),
              main_hand="minecraft:oak_log", count=4),
        using("stairs-meet-stairs",
              [put(1, 64, 2, "minecraft:oak_stairs", {"facing": "east", "half": "bottom", "shape": "straight"})],
              uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:oak_stairs", count=4, y_rot=90.0),
        using("stairs-top-half-facing", [put(0, 64, 3, "minecraft:stone")],
              uses(1, side_high, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone_stairs", count=4, y_rot=-45.0),
        using("slab-into-water", pool_under,
              uses(1, aim_at(0, 63, 2, "up", (0.5, 64.0, 2.5)), clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:oak_slab", count=4),
        using("place-where-standing", [],
              uses(1, aim_at(0, 63, 0, "up", (0.5, 64.0, 0.5)), clicks=1, watch_at=[(0, 64, 0)])
              + uses(3, top_of, watch_at=[(0, 64, 0)]),
              main_hand="minecraft:stone", count=4),
        using("creative-keeps-count", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone", count=4, game_mode="creative"),
        using("crafting-table-use", [put(0, 64, 2, "minecraft:crafting_table")],
              uses(1, aim_at(0, 64, 2, "up", (0.5, 65.0, 2.5)), clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone", count=4),
        using("sneak-place-on-crafting-table", [put(0, 64, 2, "minecraft:crafting_table")],
              uses(1, aim_at(0, 64, 2, "up", (0.5, 65.0, 2.5)), clicks=1, watch_at=around, sneak=True)
              + uses(3, top_of, watch_at=around, sneak=True),
              main_hand="minecraft:stone", count=4),
        using("off-hand-places", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              off_hand="minecraft:cobblestone"),
        using("adventure-cannot-place", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone", count=4, game_mode="adventure"),
        using("place-last-block", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone", count=1),
    ]
    yield {
        "case_id": "movement/using",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": using_rows,
    }
    yield {
        "case_id": "movement/breaking",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": breaking_rows,
    }
    # Container clicks: the inventory screen's clicks into the reference's
    # own inventory menu and game mode, which predicts each click and sends
    # it. Stacks are named by item and count.
    def st(slot, item, count):
        return {"slot": slot, "item": f"minecraft:{item}", "count": count}

    def cl(slot, button, kind):
        return {"slot": slot, "button": button, "type": kind}

    def clicks(label, slots, click_list, carried=None, game_mode="survival", selected=0, state_id=0,
               menu=None, furnace_inputs=None, extra=None):
        row = {"label": label, "blocks": [], "chunk_radius": 0, "game_mode": game_mode,
               "selected": selected, "state_id": state_id, "slots": slots, "clicks": click_list}
        if menu is not None:
            row["menu"] = f"minecraft:{menu}"
            row["container_id"] = 4
        if extra is not None:
            row.update(extra)
        if furnace_inputs is not None:
            row["furnace_inputs"] = [f"minecraft:{name}" for name in furnace_inputs]
        if carried is not None:
            row["carried"] = {"item": f"minecraft:{carried[0]}", "count": carried[1]}
        return row

    P, Q, SW, CL, TH, QC, PA = "PICKUP", "QUICK_MOVE", "SWAP", "CLONE", "THROW", "QUICK_CRAFT", "PICKUP_ALL"
    click_rows = [
        clicks("pick up and put down", [st(36, "stone", 64)], [cl(36, 0, P), cl(10, 0, P)], state_id=5),
        clicks("halves and single placements", [st(36, "stone", 63)],
               [cl(36, 1, P), cl(11, 1, P), cl(12, 1, P), cl(11, 0, P), cl(36, 0, P)]),
        clicks("swap unlike stacks", [st(36, "stone", 12)], [cl(36, 0, P), cl(37, 0, P)],
               carried=("dirt", 10)),
        clicks("merge over the limit", [st(20, "stone", 50)], [cl(20, 0, P), cl(21, 1, P)],
               carried=("stone", 40)),
        clicks("sixteen stack limit", [st(20, "ender_pearl", 10)], [cl(20, 0, P)],
               carried=("ender_pearl", 10)),
        clicks("unstackable swap", [st(20, "diamond_sword", 1)], [cl(20, 0, P), cl(21, 0, P)],
               carried=("iron_sword", 1)),
        clicks("outside the window", [], [cl(-999, 1, P), cl(-999, 0, P)], carried=("stone", 5)),
        clicks("quick move rows", [st(36, "stone", 64), st(10, "dirt", 30), st(37, "dirt", 50)],
               [cl(36, 0, Q), cl(10, 0, Q), cl(9, 0, Q)]),
        clicks("quick move merges widely",
               [st(i, "stone", 60) for i in range(9, 36)] + [st(36, "stone", 64), st(40, "stone", 1)],
               [cl(36, 0, Q)]),
        clicks("quick move armour and shield",
               [st(20, "diamond_helmet", 1), st(21, "iron_boots", 1), st(22, "shield", 1),
                st(23, "carved_pumpkin", 1), st(24, "white_carpet", 3)],
               [cl(20, 0, Q), cl(21, 0, Q), cl(22, 0, Q), cl(23, 0, Q), cl(24, 0, Q), cl(5, 0, Q)]),
        clicks("armour slots refuse and cap",
               [st(36, "stone", 64), st(37, "diamond_chestplate", 1), st(38, "leather_leggings", 1)],
               [cl(36, 0, P), cl(6, 0, P), cl(36, 0, P), cl(37, 0, P), cl(6, 0, P), cl(38, 0, P), cl(6, 0, P)]),
        clicks("number keys", [st(15, "stone", 20), st(38, "dirt", 5), st(45, "torch", 3)],
               [cl(15, 0, SW), cl(15, 2, SW), cl(16, 5, SW), cl(20, 40, SW), cl(36, 40, SW)]),
        clicks("number key into armour",
               [st(36, "diamond_helmet", 1), st(37, "stone", 64), st(5, "iron_helmet", 1)],
               [cl(5, 0, SW), cl(5, 1, SW), cl(8, 1, SW)]),
        clicks("throw one and all", [st(36, "stone", 5), st(20, "dirt", 3)],
               [cl(36, 0, TH), cl(36, 1, TH), cl(20, 0, TH), cl(21, 1, TH)]),
        clicks("double click gathers",
               [st(10, "stone", 30), st(11, "stone", 64), st(12, "stone", 10), st(40, "stone", 2),
                st(13, "dirt", 5)],
               [cl(20, 0, PA)], carried=("stone", 5)),
        clicks("double click from the end",
               [st(10, "stone", 30), st(44, "stone", 40), st(12, "stone", 10)],
               [cl(20, 1, PA)], carried=("stone", 3)),
        clicks("left drag splits", [],
               [cl(-999, 0, QC), cl(10, 1, QC), cl(11, 1, QC), cl(12, 1, QC), cl(11, 1, QC), cl(-999, 2, QC)],
               carried=("stone", 64)),
        clicks("right drag places one each", [st(11, "stone", 63)],
               [cl(-999, 4, QC), cl(10, 5, QC), cl(11, 5, QC), cl(12, 5, QC), cl(-999, 6, QC)],
               carried=("stone", 10)),
        clicks("drag over one slot is a click", [],
               [cl(-999, 0, QC), cl(10, 1, QC), cl(-999, 2, QC)], carried=("stone", 7)),
        clicks("middle drag refused in survival", [],
               [cl(-999, 8, QC), cl(10, 9, QC), cl(-999, 10, QC), cl(10, 0, P)], carried=("stone", 7)),
        clicks("broken drag resets", [],
               [cl(-999, 0, QC), cl(10, 1, QC), cl(11, 0, P)], carried=("stone", 7)),
        clicks("take the crafting result",
               [st(0, "oak_planks", 4), st(1, "oak_log", 3), st(2, "water_bucket", 1)],
               [cl(0, 0, P), cl(0, 0, P), cl(20, 0, P)]),
        clicks("quick move the crafting result",
               [st(0, "oak_planks", 4), st(1, "oak_log", 2), st(36, "oak_planks", 62)],
               [cl(0, 0, Q)]),
        clicks("crafting grid remainders",
               [st(0, "cake", 1), st(1, "milk_bucket", 1), st(2, "sugar", 2), st(3, "egg", 1),
                st(4, "honey_bottle", 3)],
               [cl(0, 0, P)]),
        clicks("result refuses placing", [st(0, "stick", 4)], [cl(0, 0, P)], carried=("stick", 2)),
        clicks("number key from the result", [st(0, "oak_planks", 4), st(1, "oak_log", 2)],
               [cl(0, 3, SW)]),
        clicks("clone needs creative", [st(10, "stone", 3)], [cl(10, 2, CL)]),
        clicks("right drag over thirty slots grows the changed map", [],
               [cl(-999, 4, QC)] + [cl(i, 5, QC) for i in range(9, 45)] + [cl(-999, 6, QC)],
               carried=("stone", 64)),
        clicks("left drag with the zero slot key", [st(0, "stick", 1)],
               [cl(0, 0, TH), cl(-999, 0, QC), cl(1, 1, QC), cl(2, 1, QC), cl(-999, 2, QC)],
               carried=("stone", 9)),
    ]
    # Menus the server opens: the container's own slots first, then the 27 of
    # the main inventory and the 9 of the hotbar.
    chest = "generic_9x3"
    click_rows += [
        clicks("chest pick up and put down", [st(0, "stone", 64)],
               [cl(0, 0, P), cl(5, 0, P), cl(5, 1, P), cl(30, 0, P)], menu=chest, state_id=2),
        clicks("chest quick move out and in",
               [st(0, "stone", 40), st(1, "diamond_sword", 1), st(30, "dirt", 3), st(57, "stone", 50)],
               [cl(0, 0, Q), cl(1, 0, Q), cl(30, 0, Q), cl(57, 0, Q)], menu=chest),
        clicks("chest quick move merges and fills",
               [st(i, "stone", 64) for i in range(0, 26)] + [st(26, "stone", 60), st(40, "stone", 30)],
               [cl(40, 0, Q)], menu=chest),
        clicks("chest number keys", [st(3, "stone", 20), st(55, "dirt", 5)],
               [cl(3, 1, SW), cl(4, 1, SW), cl(3, 8, SW), cl(28, 0, SW), cl(3, 40, SW)], menu=chest),
        clicks("chest throw", [st(2, "stone", 5)], [cl(2, 0, TH), cl(2, 1, TH)], menu=chest),
        clicks("chest left drag over both parts", [],
               [cl(-999, 0, QC), cl(3, 1, QC), cl(4, 1, QC), cl(40, 1, QC), cl(58, 1, QC), cl(-999, 2, QC)],
               carried=("stone", 64), menu=chest),
        clicks("chest right drag", [st(4, "stone", 63)],
               [cl(-999, 4, QC), cl(3, 5, QC), cl(4, 5, QC), cl(62, 5, QC), cl(-999, 6, QC)],
               carried=("stone", 10), menu=chest),
        clicks("chest double click gathers across both parts",
               [st(0, "stone", 30), st(40, "stone", 64), st(41, "stone", 10), st(62, "stone", 2)],
               [cl(5, 0, PA), cl(5, 1, PA)], carried=("stone", 5), menu=chest),
        clicks("clone in a chest", [st(1, "stone", 3)], [cl(1, 2, CL)], menu=chest),
        clicks("one row chest quick move", [st(0, "stone", 5), st(9, "dirt", 7)],
               [cl(0, 0, Q), cl(9, 0, Q)], menu="generic_9x1"),
        clicks("six row chest quick move", [st(0, "stone", 5), st(54, "dirt", 7)],
               [cl(0, 0, Q), cl(54, 0, Q)], menu="generic_9x6"),
        clicks("hopper quick move", [st(0, "stone", 5), st(1, "stone", 64), st(5, "dirt", 7)],
               [cl(0, 0, Q), cl(5, 0, Q), cl(1, 0, Q)], menu="hopper"),
        clicks("dispenser quick move gives up on no change",
               [st(0, "stone", 5), st(9, "dirt", 7), st(10, "stone", 64)] +
               [st(i, "cobblestone", 64) for i in range(1, 9)] + [st(11, "diamond_sword", 1)],
               [cl(0, 0, Q), cl(9, 0, Q), cl(11, 0, Q)], menu="generic_3x3"),
        clicks("shulker box refuses shulker boxes",
               [st(27, "shulker_box", 1), st(28, "red_shulker_box", 1), st(29, "stone", 4), st(0, "dirt", 2)],
               [cl(27, 0, Q), cl(28, 0, P), cl(1, 0, P), cl(29, 0, Q), cl(0, 0, Q)], menu="shulker_box"),
        clicks("shulker box pick up a shulker box", [st(27, "shulker_box", 1)],
               [cl(27, 0, P), cl(3, 0, P), cl(40, 0, P)], menu="shulker_box"),
    ]
    # The crafting table: the result at 0, the grid at 1 to 9, then the player.
    crafting = "crafting"
    click_rows += [
        clicks("table take the result", [st(0, "oak_planks", 4), st(1, "oak_log", 3), st(2, "water_bucket", 1)],
               [cl(0, 0, P), cl(0, 0, P), cl(20, 0, P)], menu=crafting),
        clicks("table quick move the result",
               [st(0, "oak_planks", 4), st(1, "oak_log", 2), st(36, "oak_planks", 62)],
               [cl(0, 0, Q)], menu=crafting),
        clicks("table grid remainders",
               [st(0, "cake", 1), st(1, "milk_bucket", 1), st(9, "sugar", 2), st(5, "egg", 1),
                st(8, "honey_bottle", 3)],
               [cl(0, 0, P)], menu=crafting),
        clicks("table quick move into the grid", [st(20, "stone", 30), st(40, "dirt", 3), st(3, "stone", 60)],
               [cl(20, 0, Q), cl(40, 0, Q)], menu=crafting),
        clicks("table quick move out of the grid", [st(4, "stone", 5), st(9, "dirt", 7)],
               [cl(4, 0, Q), cl(9, 0, Q)], menu=crafting),
        clicks("table result refuses placing", [st(0, "stick", 4)], [cl(0, 0, P)],
               carried=("stick", 2), menu=crafting),
        clicks("table number key from the result", [st(0, "oak_planks", 4), st(1, "oak_log", 2)],
               [cl(0, 3, SW)], menu=crafting),
        clicks("table double click skips the result",
               [st(0, "stone", 30), st(10, "stone", 10), st(45, "stone", 4)],
               [cl(11, 0, PA)], carried=("stone", 5), menu=crafting),
        clicks("table drag over the grid", [],
               [cl(-999, 0, QC), cl(1, 1, QC), cl(2, 1, QC), cl(30, 1, QC), cl(-999, 2, QC)],
               carried=("stone", 9), menu=crafting),
    ]
    # The furnaces: the ingredient at 0, the fuel at 1, the result at 2.
    smelts = ["raw_iron", "beef", "cobblestone", "sand"]
    click_rows += [
        clicks("furnace quick move sorts by use",
               [st(3, "raw_iron", 10), st(4, "coal", 20), st(5, "dirt", 7), st(30, "beef", 3),
                st(31, "charcoal", 5), st(6, "lava_bucket", 1)],
               [cl(3, 0, Q), cl(4, 0, Q), cl(5, 0, Q), cl(30, 0, Q), cl(31, 0, Q), cl(6, 0, Q)],
               menu="furnace", furnace_inputs=smelts),
        clicks("furnace quick move out", [st(0, "raw_iron", 5), st(1, "coal", 3), st(2, "iron_ingot", 7)],
               [cl(0, 0, Q), cl(1, 0, Q), cl(2, 0, Q)], menu="furnace", furnace_inputs=smelts),
        clicks("furnace fuel slot takes fuel and buckets only",
               [st(3, "coal", 10), st(4, "dirt", 5), st(5, "bucket", 3), st(6, "water_bucket", 1)],
               [cl(3, 0, P), cl(1, 0, P), cl(3, 0, P), cl(4, 0, P), cl(1, 0, P), cl(4, 0, P),
                cl(5, 0, P), cl(1, 1, P), cl(5, 0, P), cl(6, 0, P), cl(1, 0, P)],
               menu="furnace", furnace_inputs=smelts),
        clicks("furnace ingredient slot takes anything", [st(3, "dirt", 10)],
               [cl(3, 0, P), cl(0, 0, P), cl(0, 1, P)], menu="furnace", furnace_inputs=smelts),
        clicks("furnace result only comes out", [st(2, "iron_ingot", 7), st(3, "iron_ingot", 4)],
               [cl(2, 1, P), cl(3, 0, P), cl(2, 0, P), cl(2, 0, P), cl(10, 0, P)],
               menu="furnace", furnace_inputs=smelts),
        clicks("furnace double click gathers the result too",
               [st(2, "iron_ingot", 9), st(3, "iron_ingot", 10), st(4, "iron_ingot", 20)],
               [cl(12, 0, PA)], carried=("iron_ingot", 5), menu="furnace", furnace_inputs=smelts),
        clicks("blast furnace quick move", [st(3, "raw_iron", 10), st(4, "coal", 20), st(5, "beef", 4)],
               [cl(3, 0, Q), cl(4, 0, Q), cl(5, 0, Q)], menu="blast_furnace", furnace_inputs=smelts),
        clicks("smoker quick move and number key", [st(3, "beef", 10), st(4, "stick", 20), st(2, "cooked_beef", 4)],
               [cl(3, 0, Q), cl(4, 0, Q), cl(2, 2, SW), cl(2, 0, TH)], menu="smoker", furnace_inputs=smelts),
    ]
    # The menus that turn their items into something: the client runs the
    # slots' rules, what taking a result uses up and quickMoveStack.
    smith_sets = {"recipe_sets": {
        "minecraft:smithing_template": ["netherite_upgrade_smithing_template"],
        "minecraft:smithing_base": ["diamond_sword", "diamond_chestplate"],
        "minecraft:smithing_addition": ["netherite_ingot"],
    }}
    click_rows += [
        clicks("grindstone takes damageable and enchanted only",
               [st(30, "diamond_sword", 1), st(31, "stone", 4), st(32, "iron_pickaxe", 1)],
               [cl(30, 0, P), cl(0, 0, P), cl(31, 0, P), cl(1, 0, P), cl(1, 1, P), cl(32, 0, P), cl(1, 0, P)],
               menu="grindstone"),
        clicks("grindstone quick move",
               [st(5, "diamond_sword", 1), st(6, "iron_pickaxe", 1), st(7, "golden_axe", 1), st(8, "stone", 3)],
               [cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(8, 0, Q), cl(0, 0, Q)], menu="grindstone"),
        clicks("grindstone take the result",
               [st(0, "diamond_sword", 1), st(1, "diamond_sword", 1), st(2, "diamond_sword", 1)],
               [cl(2, 0, P), cl(20, 0, P)], menu="grindstone"),
        clicks("grindstone quick move the result", [st(0, "iron_sword", 1), st(2, "iron_sword", 1)],
               [cl(2, 0, Q)], menu="grindstone"),
        clicks("cartography slots", [st(30, "paper", 8), st(31, "map", 1), st(32, "glass_pane", 2),
                                      st(33, "stone", 5)],
               [cl(30, 0, P), cl(1, 0, P), cl(31, 0, P), cl(0, 0, P), cl(0, 0, P), cl(32, 0, P), cl(1, 0, P),
                cl(33, 0, P), cl(1, 0, P)], menu="cartography_table"),
        clicks("cartography quick move", [st(5, "paper", 8), st(6, "map", 1), st(7, "stone", 3),
                                           st(31, "glass_pane", 4)],
               [cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(31, 0, Q)], menu="cartography_table"),
        clicks("cartography take the result", [st(0, "filled_map", 2), st(1, "paper", 3), st(2, "filled_map", 2)],
               [cl(2, 0, P), cl(20, 0, P)], menu="cartography_table"),
        clicks("smithing slots by set",
               [st(30, "netherite_upgrade_smithing_template", 1), st(31, "diamond_sword", 1),
                st(32, "netherite_ingot", 2), st(33, "stone", 3), st(34, "diamond_sword", 1)],
               [cl(30, 0, P), cl(0, 0, P), cl(31, 0, P), cl(1, 0, P), cl(32, 0, P), cl(2, 0, P),
                cl(33, 0, P), cl(2, 0, P), cl(34, 0, P), cl(0, 0, P)],
               menu="smithing", extra=smith_sets),
        clicks("smithing quick move",
               [st(5, "netherite_upgrade_smithing_template", 1), st(6, "diamond_sword", 1),
                st(7, "netherite_ingot", 2), st(8, "stone", 3), st(9, "diamond_chestplate", 1)],
               [cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(8, 0, Q), cl(9, 0, Q), cl(0, 0, Q)],
               menu="smithing", extra=smith_sets),
        clicks("smithing take the result",
               [st(0, "netherite_upgrade_smithing_template", 1), st(1, "diamond_sword", 1),
                st(2, "netherite_ingot", 3), st(3, "netherite_sword", 1)],
               [cl(3, 0, P), cl(20, 0, P)], menu="smithing", extra=smith_sets),
        clicks("smithing result leaves by quick move",
               [st(0, "netherite_upgrade_smithing_template", 1), st(1, "diamond_sword", 1),
                st(3, "netherite_sword", 1)],
               [cl(3, 0, Q)], menu="smithing", extra=smith_sets),
        clicks("stonecutter quick move", [st(2, "stone", 10), st(3, "dirt", 4), st(30, "cobblestone", 8),
                                           st(1, "stone_slab", 3)],
               [cl(2, 0, Q), cl(3, 0, Q), cl(30, 0, Q), cl(1, 0, Q)],
               menu="stonecutter", extra={"stonecutter_inputs": ["stone", "cobblestone"]}),
        clicks("stonecutter take the result", [st(0, "stone", 5), st(1, "stone_slab", 2)],
               [cl(1, 0, P), cl(20, 0, P), cl(0, 0, P)],
               menu="stonecutter", extra={"stonecutter_inputs": ["stone"]}),
        clicks("stonecutter double click skips the result",
               [st(1, "stone_slab", 2), st(10, "stone_slab", 5), st(30, "stone_slab", 9)],
               [cl(12, 0, PA)], carried=("stone_slab", 3),
               menu="stonecutter", extra={"stonecutter_inputs": ["stone"]}),
        clicks("loom slots by tag and component",
               [st(31, "white_banner", 1), st(32, "red_dye", 4), st(33, "creeper_banner_pattern", 1),
                st(34, "stone", 2), st(35, "sugar", 2)],
               [cl(31, 0, P), cl(0, 0, P), cl(32, 0, P), cl(0, 0, P), cl(1, 0, P), cl(34, 0, P), cl(1, 0, P),
                cl(35, 0, P), cl(2, 0, P)], menu="loom"),
        clicks("loom pattern with the banner",
               [st(31, "white_banner", 1), st(33, "creeper_banner_pattern", 1), st(34, "flower_banner_pattern", 1)],
               [cl(31, 0, P), cl(0, 0, P), cl(33, 0, P), cl(2, 0, P), cl(34, 0, P), cl(2, 0, P)], menu="loom"),
        clicks("loom quick move",
               [st(5, "white_banner", 1), st(6, "blue_dye", 3), st(7, "flower_banner_pattern", 1),
                st(8, "stone", 3), st(9, "black_banner", 1)],
               [cl(5, 0, Q), cl(8, 0, Q), cl(9, 0, Q), cl(3, 0, Q), cl(6, 0, Q)], menu="loom"),
        clicks("loom pattern quick move", [st(7, "flower_banner_pattern", 1), st(8, "stone", 3)],
               [cl(7, 0, Q), cl(8, 0, Q)], menu="loom"),
        clicks("loom take the result",
               [st(0, "white_banner", 2), st(1, "red_dye", 1), st(3, "white_banner", 1)],
               [cl(3, 0, P), cl(20, 0, P)], menu="loom"),
        clicks("loom result keeps splitting",
               [st(0, "white_banner", 1), st(1, "red_dye", 4), st(3, "white_banner", 3)],
               [cl(3, 1, P), cl(20, 0, P), cl(3, 0, P)], menu="loom"),
        clicks("anvil takes anything in its inputs", [st(30, "stone", 4), st(31, "diamond_sword", 1)],
               [cl(30, 0, P), cl(0, 0, P), cl(31, 0, P), cl(1, 0, P)], menu="anvil"),
        clicks("anvil result needs the levels",
               [st(0, "iron_sword", 1), st(2, "iron_sword", 1)],
               [cl(2, 0, P), cl(20, 0, P), cl(2, 0, P)], menu="anvil",
               extra={"data": [5], "xp_level": 3}),
        clicks("anvil result with the levels",
               [st(0, "iron_sword", 1), st(1, "iron_ingot", 2), st(2, "iron_sword", 1)],
               [cl(2, 0, P), cl(20, 0, P)], menu="anvil", extra={"data": [5], "xp_level": 12}),
        clicks("anvil result costs nothing in creative",
               [st(0, "iron_sword", 1), st(2, "iron_sword", 1)],
               [cl(2, 0, P), cl(20, 0, P)], menu="anvil", game_mode="creative", extra={"data": [39]}),
        clicks("anvil quick move", [st(0, "iron_sword", 1), st(2, "iron_sword", 1), st(10, "stone", 2),
                                     st(30, "diamond_sword", 1)],
               [cl(0, 0, Q), cl(2, 0, Q), cl(10, 0, Q), cl(30, 0, Q)], menu="anvil",
               extra={"data": [1], "xp_level": 30}),
        clicks("enchanting table item slot holds one",
               [st(30, "diamond_sword", 3), st(31, "lapis_lazuli", 64), st(32, "stone", 9)],
               [cl(30, 0, P), cl(0, 0, P), cl(0, 0, P), cl(31, 0, P), cl(1, 0, P), cl(32, 0, P), cl(1, 0, P)],
               menu="enchantment"),
        clicks("enchanting table quick move",
               [st(5, "diamond_sword", 4), st(6, "lapis_lazuli", 20), st(7, "stone", 3), st(0, "iron_axe", 1)],
               [cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(0, 0, Q), cl(1, 0, Q), cl(5, 0, Q), cl(5, 0, Q)],
               menu="enchantment"),
        clicks("enchanting table number keys", [st(0, "iron_sword", 1), st(1, "lapis_lazuli", 10)],
               [cl(0, 2, SW), cl(1, 3, SW)], menu="enchantment"),
    ]
    yield {
        "case_id": "movement/container-clicks",
        "family": "movement",
        "operation": "container_clicks",
        "rows": click_rows,
    }


def entity_cases():
    yield from entity_player_model_cases()
    yield from entity_walk_animation_cases()
    yield from entity_model_layer_cases()
    yield from entity_type_cases()


def entity_type_cases():
    """The entity type registry: every type's name and its id."""
    yield {
        "case_id": "entity/types",
        "family": "entity",
        "operation": "entity_types",
        "rows": [{"label": "all"}],
    }


def entity_model_layer_cases():
    """Every model layer the game builds (`LayerDefinitions.createRoots`):
    its texture size and the tree of its parts, each with its pose, cubes
    (origin, size, growth, mirroring, texture offset and scale, faces) and
    children. The reference answers one row for each layer, by name."""
    yield {
        "case_id": "entity/model-layers",
        "family": "entity",
        "operation": "model_layers",
        "rows": [{"label": "all"}],
    }


def entity_player_model_cases():
    """The player model, posed by the reference's `setupAnim` and drawn by its
    part renderer: every quad's corner positions, texture coordinates and
    normals. Both arm widths; standing, walking, crouching, looking about,
    swinging either arm at several points of the swing, swimming through
    each stroke phase, and a spectator's head alone."""
    def pose(label, slim=False, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             age_in_ticks=0.0, attack_time=0.0, attack_arm="right", swim_amount=0.0,
             crouching=False, spectator=False):
        return {
            "label": label,
            "slim": slim,
            "head_y_rot": float_bits(head_y_rot),
            "x_rot": float_bits(x_rot),
            "walk_position": float_bits(walk_position),
            "walk_speed": float_bits(walk_speed),
            "age_in_ticks": float_bits(age_in_ticks),
            "attack_time": float_bits(attack_time),
            "attack_arm": attack_arm,
            "swim_amount": float_bits(swim_amount),
            "crouching": crouching,
            "spectator": spectator,
        }

    rows = [
        pose("standing"),
        pose("standing-slim", slim=True),
        pose("breathing", age_in_ticks=37.25),
        pose("walking", walk_position=3.7, walk_speed=0.83, age_in_ticks=12.5),
        pose("walking-slim", slim=True, walk_position=11.2, walk_speed=0.5, age_in_ticks=4.0),
        pose("looking", head_y_rot=-35.5, x_rot=48.0, age_in_ticks=100.75),
        pose("crouch-walk", crouching=True, walk_position=6.1, walk_speed=0.3, x_rot=20.0),
        pose("swing-early", attack_time=0.1, x_rot=-10.0),
        pose("swing-mid", attack_time=0.5, age_in_ticks=3.3),
        pose("swing-late-left", attack_time=0.85, attack_arm="left", slim=True),
        pose("swim-reach", swim_amount=1.0, walk_position=7.0, walk_speed=0.6, x_rot=30.0),
        pose("swim-pull", swim_amount=0.64, walk_position=17.5, walk_speed=0.9),
        pose("swim-return", swim_amount=1.0, walk_position=24.25, walk_speed=0.4),
        pose("swim-swinging", swim_amount=1.0, walk_position=3.0, attack_time=0.4),
        pose("spectator", spectator=True, head_y_rot=12.0),
    ]
    yield {
        "case_id": "entity/player-model",
        "family": "entity",
        "operation": "player_model",
        "rows": rows,
    }


def entity_walk_animation_cases():
    """The limb animation state: a player walking up to speed and stopping,
    standing, the position scale a baby has, a stop in the middle of a
    walk, and reads between two ticks."""
    def step(target, factor=0.4, scale=1.0, stop=False):
        return {"target": float_bits(target), "factor": float_bits(factor),
                "scale": float_bits(scale), "stop": stop}

    partials = [float_bits(value) for value in (0.0, 0.25, 0.5, 0.75, 1.0)]

    def row(label, steps):
        return {"label": label, "steps": steps, "partials": partials}

    walk = lambda distance: step(min(distance * 4.0, 1.0))
    rows = [
        row("standing", [step(0.0)] * 5),
        row("walk-up-to-speed", [walk(0.2158)] * 12),
        row("walk-then-stop", [walk(0.2158)] * 6 + [walk(0.0)] * 8),
        row("sprint", [walk(0.28)] * 10),
        row("slow-steps", [walk(0.03), walk(0.06), walk(0.0), walk(0.09), walk(0.05)]),
        row("baby-scale", [step(1.0, 0.4, 3.0)] * 6),
        row("stop-midway", [walk(0.2)] * 4 + [step(0.0, stop=True)] + [walk(0.2)] * 3),
        row("factor-one", [step(1.0, 1.0)] * 3),
        row("factor-zero", [step(1.0, 0.0)] * 3),
        row("long-run", [walk(0.2158)] * 200),
    ]
    yield {
        "case_id": "entity/walk-animation",
        "family": "entity",
        "operation": "walk_animation",
        "rows": rows,
    }


def build_cases(families):
    generators = {"wire": wire_cases, "math": math_cases, "timer": timer_cases, "utf8": utf8_cases,
                  "binary": binary_cases, "nbt": nbt_cases, "resources": resources_cases,
                  "window": window_cases,
                  "atlas": atlas_cases,
                  "image": image_cases,
                  "model": model_cases,
                  "json": json_cases,
                  "block": block_cases,
                  "movement": movement_cases,
                  "entity": entity_cases,
                  "gui": None}
    from cases_gui import gui_generators
    generators["gui"] = gui_generators
    cases = [dict(schema_version=1, **case)
             for family in sorted(families) for case in generators[family]()]
    ids = [case["case_id"] for case in cases]
    if not cases or len(ids) != len(set(ids)):
        raise ValueError("The corpus must be nonempty with unique case IDs")
    return cases
