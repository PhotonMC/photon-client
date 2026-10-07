"""The `math` family: the `positions/chunkpos` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
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
