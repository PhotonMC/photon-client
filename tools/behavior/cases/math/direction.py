"""The `math` family: the `direction` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
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
