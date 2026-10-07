"""The `math` family: the `vec3/rotation` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
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

    for name in ("zero", "x_axis", "y_axis", "z_axis"):
        yield {"case_id": f"vec3/constants/{name}", "family": "math", "operation": "vec3_constant",
               "name": name}
