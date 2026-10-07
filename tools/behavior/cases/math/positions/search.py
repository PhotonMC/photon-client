"""The `math` family: the `positions/search` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
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
