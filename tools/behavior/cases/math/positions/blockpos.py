"""The `math` family: the `positions/blockpos` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
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
