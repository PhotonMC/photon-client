"""The `math` family: the `aabb/construction` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.aabb.shared import *


def cases():
    yield {"case_id": "aabb-builder/empty", "family": "math", "operation": "aabb_builder",
           "points": []}

    yield {"case_id": "aabb-builder/single", "family": "math", "operation": "aabb_builder",
           "points": [vector_bits(1.25, -2.5, 3.75, "f32")]}

    yield {"case_id": "aabb-builder/multiple", "family": "math", "operation": "aabb_builder",
           "points": [vector_bits(3.5, -1.25, 8.0, "f32"),
                      vector_bits(-4.0, 2.5, -6.75, "f32"),
                      vector_bits(1.0, -3.0, 4.25, "f32")]}

    yield {"case_id": "aabb-builder/float-precision", "family": "math", "operation": "aabb_builder",
           "points": [vector_bits(0.2, -0.3, 123456.78, "f32"),
                      vector_bits(-1.0 / 3.0, 1.0 / 7.0, -0.0, "f32")]}

    yield {"case_id": "aabb-builder/special-values", "family": "math", "operation": "aabb_builder",
           "points": [vector_bits(-0.0, 0.0, -0.0, "f32"),
                      vector_bits(0.0, -0.0, 0.0, "f32"),
                      vector_bits(float("nan"), float("inf"), float("-inf"), "f32")]}

    yield {"case_id": "aabb/constructor/block-pos", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "from_block_pos", "position": block_position(-2, 3, 4)},
                              {"op": "from_block_pos", "position": block_position(2147483647, -2147483648, 0)}]}

    yield {"case_id": "aabb/constructor/vec3", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "from_vec3", "begin": vector_bits(3.0, -2.0, 1.0),
                               "end": vector_bits(-4.0, 5.0, -6.0)},
                              {"op": "from_vec3", "begin": vector_bits(-0.0, 0.0, -0.0),
                               "end": vector_bits(0.0, -0.0, 0.0)},
                              {"op": "from_vec3", "begin": vector_bits(float("inf"), -float("inf"), float("nan")),
                               "end": vector_bits(0.0, 1.0, 2.0)}]}

    yield {"case_id": "aabb/constructor/unit-cube", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "unit_cube_from_lower_corner", "position": vector_bits(1.25, -2.5, 0.125)},
                              {"op": "unit_cube_from_lower_corner", "position": vector_bits(-0.0, 0.0, -0.0)},
                              {"op": "unit_cube_from_lower_corner", "position": vector_bits(float("inf"), -float("inf"), float("nan"))}]}

    yield {"case_id": "aabb/constructor/full-blocks", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "encapsulating_full_blocks", "first": block_position(-3, 4, 8),
                               "second": block_position(2, -1, 5)},
                              {"op": "encapsulating_full_blocks", "first": block_position(10, 20, 30),
                               "second": block_position(-10, -20, -30)}]}

    yield {"case_id": "aabb/constructor/full-blocks-overflow", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "encapsulating_full_blocks",
                               "first": block_position(2147483647, 2147483647, 2147483647),
                               "second": block_position(2147483647, 2147483647, 2147483647)},
                              {"op": "encapsulating_full_blocks",
                               "first": block_position(-2147483648, -2147483648, -2147483648),
                               "second": block_position(-2147483648, -2147483648, -2147483648)}]}

    yield {"case_id": "aabb/constructor/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "from_block_pos", "position": block_position(2, 3, 4)},
                              {"op": "unit_cube_from_lower_corner", "position": vector_bits(-1.5, 0.25, 2.75)},
                              {"op": "from_vec3", "begin": vector_bits(5.0, 6.0, 7.0),
                               "end": vector_bits(1.0, 2.0, 3.0)},
                              {"op": "encapsulating_full_blocks", "first": block_position(-2, -1, 0),
                               "second": block_position(4, 5, 6)}]}

    yield {"case_id": "aabb/expand-deflate/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "expand", "x": float_bits(1.0, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(0.0, "f64")},
                             {"op": "deflate", "amount": float_bits(0.5, "f64")}]}

    yield {"case_id": "aabb/expand-overload/vec3", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "expand_vec3", "delta": vector_bits(1.0, -0.5, 0.0)}]}

    yield {"case_id": "aabb/expand-overload/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "expand_vec3", "delta": vector_bits(-0.0, 0.0, -0.0)}]}

    yield {"case_id": "aabb/expand-overload/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "expand_vec3", "delta": vector_bits(-0.25, 0.5, 1.0)},
                              {"op": "expand_vec3", "delta": vector_bits(0.125, -0.75, -0.5)}]}

    yield {"case_id": "aabb/contract/positive", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(0.25, "f64"),
                              "y": float_bits(0.5, "f64"), "z": float_bits(1.0, "f64")}]}

    yield {"case_id": "aabb/contract/negative", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(-0.25, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(-1.0, "f64")}]}

    yield {"case_id": "aabb/contract/mixed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(-0.25, "f64"),
                              "y": float_bits(0.0, "f64"), "z": float_bits(0.5, "f64")}]}

    yield {"case_id": "aabb/contract/signed-zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(-0.0, "f64"),
                              "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64")}]}

    yield {"case_id": "aabb/contract/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contract", "x": float_bits(0.25, "f64"),
                              "y": float_bits(-0.25, "f64"), "z": float_bits(0.0, "f64")},
                             {"op": "contract", "x": float_bits(-0.125, "f64"),
                              "y": float_bits(0.125, "f64"), "z": float_bits(-0.5, "f64")}]}

    yield {"case_id": "aabb/inflate/scalar-positive", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate", "amount": float_bits(0.25, "f64")}]}

    yield {"case_id": "aabb/inflate/vector-mixed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate_xyz", "x": float_bits(0.25, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(0.75, "f64")}]}

    yield {"case_id": "aabb/inflate/scalar-negative", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate", "amount": float_bits(-0.75, "f64")}]}

    yield {"case_id": "aabb/inflate/signed-zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate_xyz", "x": float_bits(-0.0, "f64"),
                              "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64")}]}

    yield {"case_id": "aabb/inflate/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "inflate_xyz", "x": float_bits(0.25, "f64"),
                              "y": float_bits(0.5, "f64"), "z": float_bits(0.0, "f64")},
                             {"op": "inflate", "amount": float_bits(-0.125, "f64")}]}

    yield {"case_id": "aabb/deflate/vector-mixed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(0.25, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(0.75, "f64")}]}

    yield {"case_id": "aabb/deflate/vector-reversed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(2.0, "f64"),
                              "y": float_bits(2.0, "f64"), "z": float_bits(2.0, "f64")}]}

    yield {"case_id": "aabb/deflate/signed-zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(-0.0, "f64"),
                              "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64")}]}

    yield {"case_id": "aabb/deflate/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(0.25, "f64"),
                              "y": float_bits(0.5, "f64"), "z": float_bits(0.0, "f64")},
                             {"op": "deflate", "amount": float_bits(-0.125, "f64")}]}

    yield {"case_id": "aabb/deflate/special-values", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "deflate_xyz", "x": float_bits(float("nan"), "f64"),
                              "y": float_bits(float("inf"), "f64"),
                              "z": float_bits(float("-inf"), "f64")}]}

    yield {"case_id": "aabb/intersect/contained", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(0.25, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(0.75, "f64"),
               "max_y": float_bits(0.75, "f64"), "max_z": float_bits(0.75, "f64")}}]}

    yield {"case_id": "aabb/intersect/partial", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(-0.5, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(-1.0, "f64"), "max_x": float_bits(0.5, "f64"),
               "max_y": float_bits(1.5, "f64"), "max_z": float_bits(0.5, "f64")}}]}

    yield {"case_id": "aabb/intersect/disjoint-normalizes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(2.0, "f64"), "min_y": float_bits(2.0, "f64"),
               "min_z": float_bits(2.0, "f64"), "max_x": float_bits(3.0, "f64"),
               "max_y": float_bits(3.0, "f64"), "max_z": float_bits(3.0, "f64")}}]}

    yield {"case_id": "aabb/intersect/touching-face", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(1.0, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(2.0, "f64"),
               "max_y": float_bits(0.75, "f64"), "max_z": float_bits(0.75, "f64")}}]}

    yield {"case_id": "aabb/intersect/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(0.0, "f64"), "min_y": float_bits(0.0, "f64"),
               "min_z": float_bits(0.0, "f64"), "max_x": float_bits(0.0, "f64"),
               "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64")}}]}

    yield {"case_id": "aabb/intersect/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersect", "box": {
               "min_x": float_bits(-0.5, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(0.75, "f64"),
               "max_y": float_bits(1.5, "f64"), "max_z": float_bits(0.75, "f64")}},
                             {"op": "intersect", "box": {
                                 "min_x": float_bits(0.5, "f64"), "min_y": float_bits(-1.0, "f64"),
                                 "min_z": float_bits(0.5, "f64"), "max_x": float_bits(2.0, "f64"),
                                 "max_y": float_bits(0.5, "f64"), "max_z": float_bits(2.0, "f64")}}]}

    yield {"case_id": "aabb/minmax/contained", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(0.25, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(0.75, "f64"),
               "max_y": float_bits(0.75, "f64"), "max_z": float_bits(0.75, "f64")}}]}

    yield {"case_id": "aabb/minmax/partial", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(-0.5, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(-1.0, "f64"), "max_x": float_bits(0.5, "f64"),
               "max_y": float_bits(1.5, "f64"), "max_z": float_bits(0.5, "f64")}}]}

    yield {"case_id": "aabb/minmax/disjoint", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(2.0, "f64"), "min_y": float_bits(2.0, "f64"),
               "min_z": float_bits(2.0, "f64"), "max_x": float_bits(3.0, "f64"),
               "max_y": float_bits(3.0, "f64"), "max_z": float_bits(3.0, "f64")}}]}

    yield {"case_id": "aabb/minmax/touching-face", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(1.0, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(2.0, "f64"),
               "max_y": float_bits(0.75, "f64"), "max_z": float_bits(0.75, "f64")}}]}

    yield {"case_id": "aabb/minmax/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(0.0, "f64"), "min_y": float_bits(0.0, "f64"),
               "min_z": float_bits(0.0, "f64"), "max_x": float_bits(0.0, "f64"),
               "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64")}}]}

    yield {"case_id": "aabb/minmax/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "minmax", "box": {
               "min_x": float_bits(-0.5, "f64"), "min_y": float_bits(0.25, "f64"),
               "min_z": float_bits(0.25, "f64"), "max_x": float_bits(0.75, "f64"),
               "max_y": float_bits(1.5, "f64"), "max_z": float_bits(0.75, "f64")}},
                             {"op": "minmax", "box": {
                                 "min_x": float_bits(-1.0, "f64"), "min_y": float_bits(-1.0, "f64"),
                                 "min_z": float_bits(-1.0, "f64"), "max_x": float_bits(0.5, "f64"),
                                 "max_y": float_bits(0.5, "f64"), "max_z": float_bits(2.0, "f64")}}]}

    yield {"case_id": "aabb/move/positive", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(1.0, "f64"),
                              "y": float_bits(2.0, "f64"), "z": float_bits(3.0, "f64")}]}

    yield {"case_id": "aabb/move/negative", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(-1.0, "f64"),
                              "y": float_bits(-2.0, "f64"), "z": float_bits(-3.0, "f64")}]}

    yield {"case_id": "aabb/move/mixed", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(-0.25, "f64"),
                              "y": float_bits(0.5, "f64"), "z": float_bits(1.25, "f64")}]}

    yield {"case_id": "aabb/move/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "move", "x": float_bits(-0.0, "f64"),
                      "y": float_bits(0.0, "f64"), "z": float_bits(-0.0, "f64")}]}

    yield {"case_id": "aabb/move/large-delta", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(9007199254740992.0, "f64"),
                              "y": float_bits(-9007199254740992.0, "f64"),
                              "z": float_bits(0.5, "f64")}]}

    yield {"case_id": "aabb/move/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move", "x": float_bits(0.25, "f64"),
                              "y": float_bits(-0.5, "f64"), "z": float_bits(1.0, "f64")},
                             {"op": "move", "x": float_bits(-0.125, "f64"),
                              "y": float_bits(0.25, "f64"), "z": float_bits(-0.5, "f64")}]}

    yield {"case_id": "aabb/move-overload/basic", "family": "math", "operation": "aabb",
           **box_bits(-1.25, 2.5, -3.75, 4.5, 5.75, 6.25),
           "steps": [{"op": "move_block_pos", "position": block_position(3, -2, 5)},
                      {"op": "move_vec3", "position": vector_bits(0.125, -0.25, 1.5)},
                      {"op": "move_vector3f", "position": vector_bits(0.2, -0.3, 0.4, "f32")}]}

    yield {"case_id": "aabb/move-overload/block-pos-extremes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move_block_pos", "position": block_position(2147483647, -2147483648, -2147483647)},
                              {"op": "move_block_pos", "position": block_position(-2147483648, 2147483647, 2147483647)}]}

    yield {"case_id": "aabb/move-overload/vec3-special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move_vec3", "position": vector_bits(float("inf"), float("-inf"), float("nan"))},
                              {"op": "move_vec3", "position": vector_bits(float("nan"), 0.5, float("inf"))}]}

    yield {"case_id": "aabb/move-overload/vector3f-precision", "family": "math", "operation": "aabb",
           **box_bits(-100.0, -200.0, -300.0, 400.0, 500.0, 600.0),
           "steps": [{"op": "move_vector3f", "position": vector_bits(0.2, -0.3, 123456.78, "f32")},
                      {"op": "move_vector3f", "position": vector_bits(-1.0 / 3.0, 1.0 / 7.0, -0.0, "f32")}]}

    yield {"case_id": "aabb/move-overload/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "move_block_pos", "position": block_position(0, 0, 0)},
                      {"op": "move_vec3", "position": vector_bits(-0.0, 0.0, -0.0)},
                      {"op": "move_vector3f", "position": vector_bits(-0.0, 0.0, -0.0, "f32")}]}

    yield {"case_id": "aabb/move-overload/chain", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "move_vec3", "position": vector_bits(0.25, -0.5, 1.0)},
                              {"op": "move_block_pos", "position": block_position(-2, 3, -4)},
                              {"op": "move_vector3f", "position": vector_bits(0.75, 0.125, -0.25, "f32")}]}

    yield {"case_id": "aabb/of-size/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(0.5, 0.5, 0.5),
                               "size_x": float_bits(1.0, "f64"),
                               "size_y": float_bits(1.0, "f64"),
                               "size_z": float_bits(1.0, "f64")}]}

    yield {"case_id": "aabb/of-size/asymmetric", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(-3.25, 4.5, 0.125),
                               "size_x": float_bits(2.5, "f64"),
                               "size_y": float_bits(7.25, "f64"),
                               "size_z": float_bits(0.75, "f64")}]}

    yield {"case_id": "aabb/of-size/negative-sizes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(1.0, -2.0, 3.0),
                               "size_x": float_bits(-4.0, "f64"),
                               "size_y": float_bits(-6.0, "f64"),
                               "size_z": float_bits(-8.0, "f64")}]}

    yield {"case_id": "aabb/of-size/zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(1.5, -2.25, 3.75),
                               "size_x": float_bits(0.0, "f64"),
                               "size_y": float_bits(0.0, "f64"),
                               "size_z": float_bits(0.0, "f64")}]}

    yield {"case_id": "aabb/of-size/signed-zero", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(-0.0, 0.0, -0.0),
                               "size_x": float_bits(-0.0, "f64"),
                               "size_y": float_bits(0.0, "f64"),
                               "size_z": float_bits(-0.0, "f64")}]}

    yield {"case_id": "aabb/of-size/special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "of_size", "center": point(float("inf"), float("-inf"), float("nan")),
                               "size_x": float_bits(2.0, "f64"),
                               "size_y": float_bits(float("inf"), "f64"),
                               "size_z": float_bits(float("nan"), "f64")}]}

    yield {"case_id": "aabb/set-endpoint/basic", "family": "math", "operation": "aabb",
           **box_bits(-2.0, -3.0, -4.0, 5.0, 6.0, 7.0),
           "steps": [{"op": "set_min_x", "value": float_bits(-1.5, "f64")},
                      {"op": "set_min_y", "value": float_bits(-2.5, "f64")},
                      {"op": "set_min_z", "value": float_bits(-3.5, "f64")},
                      {"op": "set_max_x", "value": float_bits(4.5, "f64")},
                      {"op": "set_max_y", "value": float_bits(5.5, "f64")},
                      {"op": "set_max_z", "value": float_bits(6.5, "f64")}]}

    yield {"case_id": "aabb/set-endpoint/min-chain", "family": "math", "operation": "aabb",
           **box_bits(0.0, 10.0, 20.0, 30.0, 40.0, 50.0),
           "steps": [{"op": "set_min_x", "value": float_bits(1.0, "f64")},
                      {"op": "set_min_y", "value": float_bits(11.0, "f64")},
                      {"op": "set_min_z", "value": float_bits(21.0, "f64")}]}

    yield {"case_id": "aabb/set-endpoint/max-chain", "family": "math", "operation": "aabb",
           **box_bits(-30.0, -40.0, -50.0, 0.0, 10.0, 20.0),
           "steps": [{"op": "set_max_x", "value": float_bits(-1.0, "f64")},
                      {"op": "set_max_y", "value": float_bits(9.0, "f64")},
                      {"op": "set_max_z", "value": float_bits(19.0, "f64")}]}

    yield {"case_id": "aabb/set-endpoint/reversed-normalizes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "set_min_x", "value": float_bits(4.0, "f64")},
                              {"op": "set_min_y", "value": float_bits(5.0, "f64")},
                              {"op": "set_min_z", "value": float_bits(6.0, "f64")},
                              {"op": "set_max_x", "value": float_bits(-4.0, "f64")},
                              {"op": "set_max_y", "value": float_bits(-5.0, "f64")},
                              {"op": "set_max_z", "value": float_bits(-6.0, "f64")}]}

    yield {"case_id": "aabb/set-endpoint/signed-zero", "family": "math", "operation": "aabb",
           **box_bits(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0),
           "steps": [{"op": "set_min_x", "value": float_bits(-0.0, "f64")},
                      {"op": "set_max_x", "value": float_bits(0.0, "f64")},
                      {"op": "set_min_y", "value": float_bits(0.0, "f64")},
                      {"op": "set_max_y", "value": float_bits(-0.0, "f64")},
                      {"op": "set_min_z", "value": float_bits(-0.0, "f64")},
                      {"op": "set_max_z", "value": float_bits(0.0, "f64")}]}

    yield {"case_id": "aabb/set-endpoint/special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "set_min_x", "value": float_bits(float("nan"), "f64")},
                              {"op": "set_max_y", "value": float_bits(float("inf"), "f64")},
                              {"op": "set_min_z", "value": float_bits(float("-inf"), "f64")},
                              {"op": "set_max_x", "value": float_bits(float("nan"), "f64")},
                              {"op": "set_min_y", "value": float_bits(float("-inf"), "f64")},
                              {"op": "set_max_z", "value": float_bits(float("inf"), "f64")}]}
