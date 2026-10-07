"""The `math` family: the `client_world/level` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.client_world.shared import *


def cases():
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
