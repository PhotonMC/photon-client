"""The `wire` family: the `configuration` cases."""

from cases.wire.common import *
from cases.wire.shared import *


def cases():
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

    client_information_writes = [
        ("basic", "en_us", 2, "full", True, 0, "right", False, False, "all", ""),
        ("empty", "", 0, "hidden", False, 255, "left", True, True, "minimal", "aa55"),
        ("unicode", "tr_TR😀", -1, "system", True, 3, "right", True, False, "decreased", ""),
        ("view-overflow", "en_us", 256, "full", False, 0, "left", False, False, "all", ""),
        ("model-overflow", "en_us", 2, "full", False, 511, "right", False, False, "all", ""),
        ("language-encode-limit", "a" * 32768, 2, "full", True, 0, "right", False, False, "all", ""),
    ]

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
