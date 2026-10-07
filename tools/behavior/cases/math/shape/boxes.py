"""The `math` family: the `shape/boxes` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
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
