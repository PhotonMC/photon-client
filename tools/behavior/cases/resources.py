"""The `resources` family: original synthetic inputs."""

from cases.helpers import *


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
