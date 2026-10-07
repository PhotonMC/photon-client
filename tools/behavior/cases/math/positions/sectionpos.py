"""The `math` family: the `positions/sectionpos` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
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
