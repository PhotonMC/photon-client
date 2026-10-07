"""The `math` family: the `aabb/queries` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.aabb.shared import *


def cases():
    inside = {"op": "contains", "x": float_bits(0.5, "f64"),
              "y": float_bits(0.5, "f64"), "z": float_bits(0.5, "f64")}

    outside = {"op": "contains", "x": float_bits(2.0, "f64"),
               "y": float_bits(0.0, "f64"), "z": float_bits(0.0, "f64")}

    yield {"case_id": "aabb/contains/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [inside, outside]}

    yield {"case_id": "aabb/intersects/overlap", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersects", "box": dict(unit)},
                             {"op": "intersects", "box": {
                                 "min_x": float_bits(2.0, "f64"), "min_y": float_bits(2.0, "f64"),
                                 "min_z": float_bits(2.0, "f64"), "max_x": float_bits(3.0, "f64"),
                                 "max_y": float_bits(3.0, "f64"), "max_z": float_bits(3.0, "f64")}}]}

    yield {"case_id": "aabb/predicate-overload/basic", "family": "math", "operation": "aabb",
           **box_bits(-2.0, -2.0, -2.0, 2.0, 2.0, 2.0),
           "steps": [{"op": "contains_vec3", "position": vector_bits(0.5, -0.5, 1.5)},
                      {"op": "intersects_coords", **box_bits(-1.0, -1.0, -1.0, 0.0, 0.0, 0.0)},
                      {"op": "intersects_vec3", "min": vector_bits(1.0, 1.0, 1.0),
                       "max": vector_bits(-1.0, -1.0, -1.0)},
                      {"op": "intersects_block_pos", "position": block_position(-1, -1, -1)}]}

    yield {"case_id": "aabb/predicate-overload/contains-boundaries", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "contains_vec3", "position": vector_bits(0.0, 0.0, 0.0)},
                              {"op": "contains_vec3", "position": vector_bits(1.0, 0.5, 0.5)},
                              {"op": "contains_vec3", "position": vector_bits(float("nan"), 0.5, 0.5)},
                              {"op": "contains_vec3", "position": vector_bits(float("inf"), 0.5, 0.5)}]}

    yield {"case_id": "aabb/predicate-overload/raw-coordinates", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersects_coords", **box_bits(-0.5, -0.5, -0.5, 0.5, 0.5, 0.5)},
                              {"op": "intersects_coords", **box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)},
                              {"op": "intersects_coords", **box_bits(0.5, 0.5, 0.5, -0.5, -0.5, -0.5)},
                              {"op": "intersects_coords", **box_bits(float("nan"), 0.0, 0.0, 1.0, 1.0, 1.0)}]}

    yield {"case_id": "aabb/predicate-overload/vec3-normalizes", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "intersects_vec3", "min": vector_bits(1.0, 1.0, 1.0),
                               "max": vector_bits(-1.0, -1.0, -1.0)},
                              {"op": "intersects_vec3", "min": vector_bits(0.0, -0.0, 0.0),
                               "max": vector_bits(-0.0, 0.0, -0.0)},
                              {"op": "intersects_vec3", "min": vector_bits(float("nan"), 0.0, 0.0),
                               "max": vector_bits(1.0, 1.0, 1.0)}]}

    yield {"case_id": "aabb/predicate-overload/block-pos-boundaries", "family": "math", "operation": "aabb",
           "min_x": float_bits(-2147483648.0, "f64"), "min_y": float_bits(0.0, "f64"),
           "min_z": float_bits(0.0, "f64"), "max_x": float_bits(-2147483647.0, "f64"),
           "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64"),
           "steps": [{"op": "intersects_block_pos", "position": block_position(-2147483648, 0, 0)},
                      {"op": "intersects_block_pos", "position": block_position(2147483647, 0, 0)}]}

    yield {"case_id": "aabb/predicate-overload/special-chain", "family": "math", "operation": "aabb",
           **box_bits(-1.0, -1.0, -1.0, 1.0, 1.0, 1.0),
           "steps": [{"op": "intersects_coords", **box_bits(float("inf"), -1.0, -1.0, float("inf"), 1.0, 1.0)},
                      {"op": "intersects_vec3", "min": vector_bits(-2.0, -2.0, -2.0),
                       "max": vector_bits(2.0, 2.0, 2.0)},
                      {"op": "contains_vec3", "position": vector_bits(-1.0, 0.0, 0.0)},
                      {"op": "intersects_block_pos", "position": block_position(0, 0, 0)}]}

    yield {"case_id": "aabb/collided-along-vector/empty", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(1.0, 0.0, 0.0),
                              "boxes": []}]}

    yield {"case_id": "aabb/collided-along-vector/from-contained", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(0.0, 0.0, 0.0),
                              "boxes": [box_bits(0.25, 0.25, 0.25, 0.75, 0.75, 0.75)]}]}

    yield {"case_id": "aabb/collided-along-vector/to-contained", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(2.0, 0.0, 0.0),
                              "boxes": [box_bits(2.5, 0.0, 0.0, 3.5, 1.0, 1.0)]}]}

    yield {"case_id": "aabb/collided-along-vector/crossing", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(3.0, 0.0, 0.0),
                              "boxes": [box_bits(1.5, 0.0, 0.0, 2.0, 1.0, 1.0)]}]}

    yield {"case_id": "aabb/collided-along-vector/miss", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(3.0, 0.0, 0.0),
                              "boxes": [box_bits(1.5, 2.0, 0.0, 2.0, 3.0, 1.0)]}]}

    yield {"case_id": "aabb/collided-along-vector/many", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(3.0, 0.0, 0.0),
                              "boxes": [box_bits(1.5, 2.0, 0.0, 2.0, 3.0, 1.0),
                                        box_bits(1.5, 0.0, 0.0, 2.0, 1.0, 1.0)]}]}

    yield {"case_id": "aabb/collided-along-vector/epsilon-special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "collided_along_vector", "vector": vector_bits(0.0, 0.0, 0.0),
                              "boxes": [box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)]},
                             {"op": "collided_along_vector", "vector": vector_bits(0.0000002, 0.0, 0.0),
                              "boxes": [box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)]},
                             {"op": "collided_along_vector", "vector": vector_bits(float("nan"), 0.0, 0.0),
                              "boxes": [box_bits(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)]}]}

    yield {"case_id": "aabb/collided-along-vector/tiny-moving-box", "family": "math", "operation": "aabb",
           **box_bits(0.0, 0.0, 0.0, 0.00000001, 0.00000001, 0.00000001),
           "steps": [{"op": "collided_along_vector", "vector": vector_bits(1.0, 0.0, 0.0),
                      "boxes": [box_bits(0.5, 0.0, 0.0, 0.6, 1.0, 1.0)]}]}

    yield {"case_id": "aabb/equality/identical", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "equals", "box": box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)},
                              {"op": "hash_code"}]}

    yield {"case_id": "aabb/equality/different-endpoint", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "equals", "box": box_bits(0.0, 0.0, 0.0, 2.0, 1.0, 1.0)},
                              {"op": "hash_code"}]}

    yield {"case_id": "aabb/equality/signed-zero", "family": "math", "operation": "aabb",
           **raw_box(float_bits(-0.0, "f64"), float_bits(0.0, "f64"), float_bits(0.0, "f64"),
                     float_bits(1.0, "f64"), float_bits(1.0, "f64"), float_bits(1.0, "f64")),
           "steps": [{"op": "equals", "box": raw_box(
               float_bits(0.0, "f64"), float_bits(0.0, "f64"), float_bits(0.0, "f64"),
               float_bits(1.0, "f64"), float_bits(1.0, "f64"), float_bits(1.0, "f64"))},
                     {"op": "hash_code"}]}

    yield {"case_id": "aabb/equality/nan-payload", "family": "math", "operation": "aabb",
           **raw_box("7ff8000000000000", float_bits(0.0, "f64"), float_bits(0.0, "f64"),
                     float_bits(float("inf"), "f64"), float_bits(1.0, "f64"), float_bits(1.0, "f64")),
           "steps": [{"op": "equals", "box": raw_box(
               "7ff8000000000001", float_bits(0.0, "f64"), float_bits(0.0, "f64"),
               float_bits(float("inf"), "f64"), float_bits(1.0, "f64"), float_bits(1.0, "f64"))},
                     {"op": "hash_code"}]}

    yield {"case_id": "aabb/equality/infinities", "family": "math", "operation": "aabb",
           **box_bits(float("-inf"), float("-inf"), float("-inf"),
                      float("inf"), float("inf"), float("inf")),
           "steps": [{"op": "equals", "box": box_bits(
               float("-inf"), float("-inf"), float("-inf"),
               float("inf"), float("inf"), float("inf"))},
                     {"op": "equals", "box": box_bits(-1.0, -1.0, -1.0, 1.0, 1.0, 1.0)},
                     {"op": "hash_code"}]}

    yield {"case_id": "aabb/to-string/basic", "family": "math", "operation": "aabb",
           **box_bits(0.125, 0.25, 0.375, 0.625, 0.75, 0.875),
           "steps": [{"op": "to_string"}]}

    yield {"case_id": "aabb/to-string/scientific", "family": "math", "operation": "aabb",
           **box_bits(0.0001, 10000000.0, -0.0, 0.0002, 10000001.0, 2.0),
           "steps": [{"op": "to_string"}]}

    yield {"case_id": "aabb/to-string/signed-zero", "family": "math", "operation": "aabb",
           **box_bits(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0),
           "steps": [{"op": "to_string"}]}

    yield {"case_id": "aabb/to-string/infinity", "family": "math", "operation": "aabb",
           **box_bits(float("-inf"), float("-inf"), float("-inf"),
                      float("inf"), float("inf"), float("inf")),
           "steps": [{"op": "to_string"}]}

    yield {"case_id": "aabb/to-string/nan", "family": "math", "operation": "aabb",
           **box_bits(float("nan"), 0.0, 0.0, 1.0, 1.0, 1.0),
           "steps": [{"op": "to_string"}]}

    yield {"case_id": "aabb/sizes/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "sizes"}]}

    yield {"case_id": "aabb/sizes/rectangular", "family": "math", "operation": "aabb",
           "min_x": float_bits(-2.0, "f64"), "min_y": float_bits(0.5, "f64"),
           "min_z": float_bits(10.0, "f64"), "max_x": float_bits(1.0, "f64"),
           "max_y": float_bits(4.5, "f64"), "max_z": float_bits(12.5, "f64"),
           "steps": [{"op": "sizes"}]}

    yield {"case_id": "aabb/sizes/reversed-normalizes", "family": "math", "operation": "aabb",
           "min_x": float_bits(3.0, "f64"), "min_y": float_bits(2.0, "f64"),
           "min_z": float_bits(1.0, "f64"), "max_x": float_bits(-1.0, "f64"),
           "max_y": float_bits(-2.0, "f64"), "max_z": float_bits(-3.0, "f64"),
           "steps": [{"op": "sizes"}]}

    yield {"case_id": "aabb/sizes/zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(1.5, "f64"), "min_y": float_bits(-2.0, "f64"),
           "min_z": float_bits(4.0, "f64"), "max_x": float_bits(1.5, "f64"),
           "max_y": float_bits(-2.0, "f64"), "max_z": float_bits(4.0, "f64"),
           "steps": [{"op": "sizes"}]}

    yield {"case_id": "aabb/sizes/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(0.0, "f64"),
           "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64"),
           "steps": [{"op": "sizes"}]}

    yield {"case_id": "aabb/sizes/large-sum-rounding", "family": "math", "operation": "aabb",
           **unit, "max_z": float_bits(9007199254740992.0, "f64"),
           "steps": [{"op": "sizes"}]}

    yield {"case_id": "aabb/has-nan/finite", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "has_nan"}]}

    yield {"case_id": "aabb/has-nan/x-min", "family": "math", "operation": "aabb",
           **unit, "min_x": float_bits(float("nan"), "f64"),
           "steps": [{"op": "has_nan"}]}

    yield {"case_id": "aabb/has-nan/y-max", "family": "math", "operation": "aabb",
           **unit, "max_y": float_bits(float("nan"), "f64"),
           "steps": [{"op": "has_nan"}]}

    yield {"case_id": "aabb/has-nan/z-both", "family": "math", "operation": "aabb",
           **unit, "min_z": float_bits(float("nan"), "f64"),
           "max_z": float_bits(float("nan"), "f64"),
           "steps": [{"op": "has_nan"}]}

    yield {"case_id": "aabb/has-nan/positive-infinity", "family": "math", "operation": "aabb",
           **unit, "min_x": float_bits(float("inf"), "f64"),
           "max_x": float_bits(float("inf"), "f64"),
           "steps": [{"op": "has_nan"}]}

    yield {"case_id": "aabb/has-nan/negative-infinity", "family": "math", "operation": "aabb",
           **unit, "min_y": float_bits(float("-inf"), "f64"),
           "max_y": float_bits(float("-inf"), "f64"),
           "steps": [{"op": "has_nan"}]}

    yield {"case_id": "aabb/positions/unit", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "positions"}]}

    yield {"case_id": "aabb/positions/reversed-normalizes", "family": "math", "operation": "aabb",
           "min_x": float_bits(3.0, "f64"), "min_y": float_bits(2.0, "f64"),
           "min_z": float_bits(1.0, "f64"), "max_x": float_bits(-1.0, "f64"),
           "max_y": float_bits(-2.0, "f64"), "max_z": float_bits(-3.0, "f64"),
           "steps": [{"op": "positions"}]}

    yield {"case_id": "aabb/positions/asymmetric", "family": "math", "operation": "aabb",
           "min_x": float_bits(-7.25, "f64"), "min_y": float_bits(0.125, "f64"),
           "min_z": float_bits(10.5, "f64"), "max_x": float_bits(2.75, "f64"),
           "max_y": float_bits(8.875, "f64"), "max_z": float_bits(11.25, "f64"),
           "steps": [{"op": "positions"}]}

    yield {"case_id": "aabb/positions/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(0.0, "f64"),
           "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64"),
           "steps": [{"op": "positions"}]}

    yield {"case_id": "aabb/positions/infinite", "family": "math", "operation": "aabb",
           "min_x": float_bits(float("-inf"), "f64"), "min_y": float_bits(float("-inf"), "f64"),
           "min_z": float_bits(float("-inf"), "f64"), "max_x": float_bits(float("inf"), "f64"),
           "max_y": float_bits(float("inf"), "f64"), "max_z": float_bits(float("inf"), "f64"),
           "steps": [{"op": "positions"}]}

    yield {"case_id": "aabb/positions/after-move", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "positions"},
                              {"op": "move", "x": float_bits(1.25, "f64"),
                               "y": float_bits(-0.5, "f64"), "z": float_bits(2.0, "f64")},
                              {"op": "positions"}]}

    axis_box = box_bits(-4.0, -3.0, -2.0, 4.0, 3.0, 2.0)

    yield {"case_id": "aabb/axis-limit/min-all", "family": "math", "operation": "aabb",
           **axis_box, "steps": [{"op": "min", "axis": "x"},
                                  {"op": "min", "axis": "y"},
                                  {"op": "min", "axis": "z"}]}

    yield {"case_id": "aabb/axis-limit/max-all", "family": "math", "operation": "aabb",
           **axis_box, "steps": [{"op": "max", "axis": "x"},
                                  {"op": "max", "axis": "y"},
                                  {"op": "max", "axis": "z"}]}

    yield {"case_id": "aabb/axis-limit/mixed", "family": "math", "operation": "aabb",
           **axis_box, "steps": [{"op": "min", "axis": "z"},
                                  {"op": "max", "axis": "x"},
                                  {"op": "min", "axis": "y"},
                                  {"op": "max", "axis": "z"}]}

    yield {"case_id": "aabb/axis-limit/reversed-normalizes", "family": "math", "operation": "aabb",
           "min_x": float_bits(4.0, "f64"), "min_y": float_bits(3.0, "f64"),
           "min_z": float_bits(2.0, "f64"), "max_x": float_bits(-4.0, "f64"),
           "max_y": float_bits(-3.0, "f64"), "max_z": float_bits(-2.0, "f64"),
           "steps": [{"op": "min", "axis": "x"}, {"op": "max", "axis": "y"}]}

    yield {"case_id": "aabb/axis-limit/signed-zero", "family": "math", "operation": "aabb",
           "min_x": float_bits(-0.0, "f64"), "min_y": float_bits(-0.0, "f64"),
           "min_z": float_bits(-0.0, "f64"), "max_x": float_bits(0.0, "f64"),
           "max_y": float_bits(0.0, "f64"), "max_z": float_bits(0.0, "f64"),
           "steps": [{"op": "min", "axis": "x"}, {"op": "max", "axis": "y"},
                      {"op": "min", "axis": "z"}, {"op": "max", "axis": "x"}]}

    yield {"case_id": "aabb/axis-limit/nan", "family": "math", "operation": "aabb",
           **unit, "min_x": float_bits(float("nan"), "f64"),
           "steps": [{"op": "min", "axis": "x"}, {"op": "max", "axis": "x"},
                      {"op": "min", "axis": "y"}]}

    yield {"case_id": "aabb/distance/point-inside-boundary", "family": "math", "operation": "aabb",
           **box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           "steps": [{"op": "distance_point", "point": point(0.5, 0.5, 0.5)},
                      {"op": "distance_point", "point": point(0.0, 0.0, 0.0)},
                      {"op": "distance_point", "point": point(1.0, 1.0, 1.0)}]}

    yield {"case_id": "aabb/distance/point-outside", "family": "math", "operation": "aabb",
           **box_bits(-2.0, -1.0, 0.0, 3.0, 4.0, 5.0),
           "steps": [{"op": "distance_point", "point": point(5.0, -3.0, 7.0)},
                      {"op": "distance_point", "point": point(-4.0, 2.0, 1.0)}]}

    yield {"case_id": "aabb/distance/point-special", "family": "math", "operation": "aabb",
           **unit, "steps": [{"op": "distance_point", "point": point(float("inf"), 0.5, 0.5)},
                              {"op": "distance_point", "point": point(float("-inf"), 0.5, 0.5)},
                              {"op": "distance_point", "point": point(float("nan"), 0.5, 0.5)}]}

    yield {"case_id": "aabb/distance/box-overlap-touch", "family": "math", "operation": "aabb",
           **box_bits(0.0, 0.0, 0.0, 2.0, 2.0, 2.0),
           "steps": [{"op": "distance_box", "box": box_bits(1.0, 1.0, 1.0, 3.0, 3.0, 3.0)},
                      {"op": "distance_box", "box": box_bits(2.0, 2.0, 2.0, 4.0, 4.0, 4.0)}]}

    yield {"case_id": "aabb/distance/box-separated-diagonal", "family": "math", "operation": "aabb",
           **box_bits(-2.0, -1.0, 0.0, -1.0, 1.0, 2.0),
           "steps": [{"op": "distance_box", "box": box_bits(2.0, 3.0, 5.0, 4.0, 6.0, 7.0)},
                      {"op": "distance_box", "box": box_bits(-5.0, -4.0, -3.0, -3.0, -2.0, -1.0)}]}

    yield {"case_id": "aabb/distance/box-reversed-and-special", "family": "math", "operation": "aabb",
           "min_x": float_bits(4.0, "f64"), "min_y": float_bits(3.0, "f64"),
           "min_z": float_bits(2.0, "f64"), "max_x": float_bits(-4.0, "f64"),
           "max_y": float_bits(-3.0, "f64"), "max_z": float_bits(-2.0, "f64"),
           "steps": [{"op": "distance_box", "box": box_bits(10.0, 10.0, 10.0, 11.0, 11.0, 11.0)},
                      {"op": "distance_box", "box": box_bits(float("nan"), 0.0, 0.0, 1.0, 1.0, 1.0)}]}

    clip_rays = {
        "west-to-east": ((-1.0, 0.5, 0.5), (2.0, 0.5, 0.5)),
        "east-to-west": ((2.0, 0.5, 0.5), (-1.0, 0.5, 0.5)),
        "inside-to-east": ((0.5, 0.5, 0.5), (2.0, 0.5, 0.5)),
        "diagonal": ((-1.0, -1.0, -1.0), (2.0, 2.0, 2.0)),
        "parallel-miss": ((-1.0, 2.0, 0.5), (2.0, 2.0, 0.5)),
        "below-threshold": ((-1.0, 0.5, 0.5), (-1.0 + 1.0e-8, 0.5, 0.5)),
    }

    for label, (source, target) in clip_rays.items():
        yield {"case_id": f"aabb/clip/{label}", "family": "math", "operation": "aabb",
               **unit, "steps": [{"op": "clip", "from": point(*source), "to": point(*target)}]}

    clip_coordinate_cases = {
        "west-to-east": {
            "bounds": (0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "source": (-1.0, 0.5, 0.5),
            "target": (2.0, 0.5, 0.5),
        },
        "east-to-west": {
            "bounds": (0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "source": (2.0, 0.5, 0.5),
            "target": (-1.0, 0.5, 0.5),
        },
        "diagonal": {
            "bounds": (-2.0, -1.0, 0.0, 3.0, 2.0, 4.0),
            "source": (-3.0, -2.0, -1.0),
            "target": (4.0, 3.0, 5.0),
        },
        "parallel-miss": {
            "bounds": (0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "source": (-1.0, 2.0, 0.5),
            "target": (2.0, 2.0, 0.5),
        },
        "reversed-bounds": {
            "bounds": (1.0, 1.0, 1.0, 0.0, 0.0, 0.0),
            "source": (-1.0, 0.5, 0.5),
            "target": (2.0, 0.5, 0.5),
        },
        "threshold-special": {
            "bounds": (0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            "source": (-1.0, 0.5, 0.5),
            "target": (-1.0 + 1.0e-8, 0.5, 0.5),
            "extra": [
                {"from": point(0.5, 0.5, 0.5), "to": point(0.5, 0.5, 0.5)},
                {"from": point(float("nan"), 0.5, 0.5), "to": point(2.0, 0.5, 0.5)},
                {"from": point(float("-inf"), 0.5, 0.5), "to": point(float("inf"), 0.5, 0.5)},
            ],
        },
    }

    for label, details in clip_coordinate_cases.items():
        min_x, min_y, min_z, max_x, max_y, max_z = details["bounds"]
        steps = [{"op": "clip_coords", "min_x": float_bits(min_x, "f64"),
                  "min_y": float_bits(min_y, "f64"), "min_z": float_bits(min_z, "f64"),
                  "max_x": float_bits(max_x, "f64"), "max_y": float_bits(max_y, "f64"),
                  "max_z": float_bits(max_z, "f64"), "from": point(*details["source"]),
                  "to": point(*details["target"])}]
        for extra in details.get("extra", []):
            steps.append({"op": "clip_coords", "min_x": float_bits(min_x, "f64"),
                          "min_y": float_bits(min_y, "f64"), "min_z": float_bits(min_z, "f64"),
                          "max_x": float_bits(max_x, "f64"), "max_y": float_bits(max_y, "f64"),
                          "max_z": float_bits(max_z, "f64"), **extra})
        yield {"case_id": f"aabb/clip-coords/{label}", "family": "math", "operation": "aabb",
               **unit, "steps": steps}

    def clip_many_step(boxes, source, target, position):
        return {"op": "clip_many", "boxes": boxes, "from": point(*source),
                "to": point(*target), "pos": block_position(*position)}

    clip_many_unit = [box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)]

    yield {"case_id": "aabb/clip-many/west-hit", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step(clip_many_unit, (-1.0, 0.5, 0.5),
                                              (2.0, 0.5, 0.5), (0, 0, 0))]}

    yield {"case_id": "aabb/clip-many/east-hit-offset", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step(clip_many_unit, (5.0, -1.5, 5.5),
                                              (2.0, -1.5, 5.5), (3, -2, 5))]}

    yield {"case_id": "aabb/clip-many/vertical-and-depth", "family": "math", "operation": "aabb",
           **unit, "steps": [
               clip_many_step(clip_many_unit, (0.5, -1.0, 0.5), (0.5, 2.0, 0.5), (0, 0, 0)),
               clip_many_step(clip_many_unit, (0.5, 2.0, 0.5), (0.5, -1.0, 0.5), (0, 0, 0)),
               clip_many_step(clip_many_unit, (0.5, 0.5, -1.0), (0.5, 0.5, 2.0), (0, 0, 0)),
               clip_many_step(clip_many_unit, (0.5, 0.5, 2.0), (0.5, 0.5, -1.0), (0, 0, 0)),
           ]}

    yield {"case_id": "aabb/clip-many/diagonal-priority", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step(clip_many_unit, (-1.0, -1.0, -1.0),
                                              (2.0, 2.0, 2.0), (0, 0, 0))]}

    yield {"case_id": "aabb/clip-many/nearest-far-first", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step([
               box_bits(3.0, 0.0, 0.0, 4.0, 1.0, 1.0),
               box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
           ], (9.0, 0.5, 0.5), (16.0, 0.5, 0.5), (10, 0, 0))]}

    yield {"case_id": "aabb/clip-many/nearest-near-first", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step([
               box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
               box_bits(3.0, 0.0, 0.0, 4.0, 1.0, 1.0),
           ], (-1.0, 0.5, 0.5), (6.0, 0.5, 0.5), (0, 0, 0))]}

    yield {"case_id": "aabb/clip-many/empty", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step([], (-1.0, 0.5, 0.5),
                                              (2.0, 0.5, 0.5), (0, 0, 0))]}

    yield {"case_id": "aabb/clip-many/translated-miss", "family": "math", "operation": "aabb",
           **unit, "steps": [clip_many_step(clip_many_unit, (-1.0, 0.5, 0.5),
                                              (2.0, 0.5, 0.5), (0, 3, 0))]}

    yield {"case_id": "aabb/clip-many/interior-and-threshold", "family": "math", "operation": "aabb",
           **unit, "steps": [
               clip_many_step(clip_many_unit, (0.5, 0.5, 0.5), (2.0, 0.5, 0.5), (0, 0, 0)),
               clip_many_step(clip_many_unit, (-1.0, 0.5, 0.5), (-1.0 + 1.0e-8, 0.5, 0.5), (0, 0, 0)),
           ]}

    def bounding_box(min_x, min_y, min_z, max_x, max_y, max_z):
        return {"min_x": str(min_x), "min_y": str(min_y), "min_z": str(min_z),
                "max_x": str(max_x), "max_y": str(max_y), "max_z": str(max_z)}

    bounding_box_cases = {
        "normal": [bounding_box(-4, -2, 3, 5, 6, 9)],
        "inverted-all": [bounding_box(4, 5, 6, -1, -2, -3)],
        "inverted-axes": [bounding_box(4, -2, 8, -1, 6, 3)],
        "integer-maximum-overflow": [bounding_box(-2, -3, -4, 2147483647, 2147483647, 2147483647)],
        "integer-limits": [bounding_box(-2147483648, -2147483648, -2147483648,
                                            2147483647, 2147483647, 2147483647)],
        "chain": [bounding_box(10, 20, 30, 12, 24, 36),
                   bounding_box(-8, -6, -4, -2, 0, 2)],
    }

    for label, boxes in bounding_box_cases.items():
        steps = [{"op": "of_bounding_box", "box": box} for box in boxes]
        yield {"case_id": f"aabb/of-bounding-box/{label}", "family": "math", "operation": "aabb",
               **unit, "steps": steps}
