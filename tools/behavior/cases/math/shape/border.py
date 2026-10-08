"""The `math` family: the `shape/border` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    yield {"case_id": "shape/world-border-collision-shape/static-extents", "family": "math",
           "operation": "shape", "kind": "world_border_collision_shape",
           "inputs": [
            {"center_x": float_bits(0.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "size": float_bits(5.5, "f64"), "absolute_max_size": 100},
            {"center_x": float_bits(10.25, "f64"), "center_z": float_bits(-4.75, "f64"),
             "size": float_bits(9.5, "f64"), "absolute_max_size": 100},
            {"center_x": float_bits(90.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "size": float_bits(50.0, "f64"), "absolute_max_size": 100},
            {"center_x": float_bits(0.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "size": float_bits(300.0, "f64"), "absolute_max_size": 100},
           ]}

    yield {"case_id": "shape/world-border-collision-shape/moving-extents", "family": "math",
           "operation": "shape", "kind": "world_border_collision_shape_moving",
           "inputs": [
            {"center_x": float_bits(1.25, "f64"), "center_z": float_bits(-2.5, "f64"),
             "from": float_bits(20.0, "f64"), "to": float_bits(40.0, "f64"),
             "absolute_max_size": 100, "duration": 4, "tick_count": 4,
             "partial_ticks": [
                 float_bits(0.0, "f32"), float_bits(0.25, "f32"),
                 float_bits(0.5, "f32"), float_bits(1.0, "f32"),
             ]},
            {"center_x": float_bits(-10.25, "f64"), "center_z": float_bits(4.75, "f64"),
             "from": float_bits(40.0, "f64"), "to": float_bits(10.0, "f64"),
             "absolute_max_size": 15, "duration": 3, "tick_count": 3,
             "partial_ticks": [
                 float_bits(0.0, "f32"), float_bits(0.25, "f32"),
                 float_bits(0.5, "f32"), float_bits(1.0, "f32"),
             ]},
            {"center_x": float_bits(90.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "from": float_bits(50.0, "f64"), "to": float_bits(300.0, "f64"),
             "absolute_max_size": 100, "duration": 2, "tick_count": 2,
             "partial_ticks": [
                 float_bits(0.0, "f32"), float_bits(0.5, "f32"),
                 float_bits(1.0, "f32"),
             ]},
           ]}

    yield {"case_id": "world/client-border-state/packet-application", "family": "math",
           "operation": "shape", "kind": "client_world_border_state",
           "inputs": [
            {"events": [
                {"op": "initialize", "center_x": float_bits(2.25, "f64"),
                 "center_z": float_bits(-3.5, "f64"), "old_size": float_bits(20.0, "f64"),
                 "new_size": float_bits(40.0, "f64"), "lerp_time": 4,
                 "absolute_max_size": 30, "warning_blocks": 7, "warning_time": 20},
                {"op": "tick"},
                {"op": "tick"},
                {"op": "center", "center_x": float_bits(4.0, "f64"),
                 "center_z": float_bits(-1.0, "f64")},
                {"op": "lerp", "old_size": float_bits(30.0, "f64"),
                 "new_size": float_bits(10.0, "f64"), "lerp_time": 3},
                {"op": "tick"},
                {"op": "tick"},
                {"op": "tick"},
                {"op": "warning_blocks", "value": 12},
                {"op": "warning_time", "value": 40},
            ]},
            {"events": [
                {"op": "initialize", "center_x": float_bits(-8.0, "f64"),
                 "center_z": float_bits(5.0, "f64"), "old_size": float_bits(1.0, "f64"),
                 "new_size": float_bits(24.0, "f64"), "lerp_time": 0,
                 "absolute_max_size": 12, "warning_blocks": 3, "warning_time": 9},
                {"op": "size", "size": float_bits(8.0, "f64")},
                {"op": "center", "center_x": float_bits(-5.5, "f64"),
                 "center_z": float_bits(2.5, "f64")},
                {"op": "lerp", "old_size": float_bits(8.0, "f64"),
                 "new_size": float_bits(8.0, "f64"), "lerp_time": 5},
                {"op": "warning_blocks", "value": 0},
                {"op": "warning_time", "value": 0},
            ]},
           ]}

    yield {"case_id": "shape/world-border-render/extract-and-quads", "family": "math",
           "operation": "shape", "kind": "world_border_render",
           "inputs": [
            # Stationary, camera close to the +X edge only: one quad (EAST).
            {"center_x": float_bits(0.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "absolute_max_size": 1000, "size": float_bits(100.0, "f64"),
             "partial_tick": float_bits(0.0, "f32"),
             "camera_x": float_bits(45.0, "f64"), "camera_y": float_bits(70.0, "f64"),
             "camera_z": float_bits(0.0, "f64"), "render_distance": float_bits(16.0, "f64"),
             "half_height_y": float_bits(8.0, "f32"), "time_millis": 0},
            # Stationary, camera near the +X/+Z corner: tied distances on
            # SOUTH and EAST (quads on both, in that stable-sort order).
            {"center_x": float_bits(0.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "absolute_max_size": 1000, "size": float_bits(100.0, "f64"),
             "partial_tick": float_bits(0.0, "f32"),
             "camera_x": float_bits(45.0, "f64"), "camera_y": float_bits(70.0, "f64"),
             "camera_z": float_bits(45.0, "f64"), "render_distance": float_bits(16.0, "f64"),
             "half_height_y": float_bits(8.0, "f32"), "time_millis": 0},
            # Far from every edge and outside the padded bounding box: zero
            # alpha, no quads, but `closest` is still reported.
            {"center_x": float_bits(0.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "absolute_max_size": 10000, "size": float_bits(100.0, "f64"),
             "partial_tick": float_bits(0.0, "f32"),
             "camera_x": float_bits(1000.0, "f64"), "camera_y": float_bits(70.0, "f64"),
             "camera_z": float_bits(1000.0, "f64"), "render_distance": float_bits(16.0, "f64"),
             "half_height_y": float_bits(8.0, "f32"), "time_millis": 0},
            # Growing mid-lerp, fractional partial tick and camera Y (so
            # `v0`'s `frac(cameraY * 0.5)` is non-trivial), non-zero and
            # wrapping `time_millis` for the texture scroll.
            {"center_x": float_bits(10.0, "f64"), "center_z": float_bits(-5.0, "f64"),
             "absolute_max_size": 1000, "from": float_bits(20.0, "f64"),
             "to": float_bits(60.0, "f64"), "duration": 8, "ticks": 4,
             "partial_tick": float_bits(0.5, "f32"),
             "camera_x": float_bits(39.0, "f64"), "camera_y": float_bits(71.3, "f64"),
             "camera_z": float_bits(-5.0, "f64"), "render_distance": float_bits(16.0, "f64"),
             "half_height_y": float_bits(10.0, "f32"), "time_millis": 4200},
            # Shrinking mid-lerp, camera on the WEST side, a different
            # `half_height_y` and a `time_millis` under one scroll period.
            {"center_x": float_bits(0.0, "f64"), "center_z": float_bits(0.0, "f64"),
             "absolute_max_size": 1000, "from": float_bits(100.0, "f64"),
             "to": float_bits(40.0, "f64"), "duration": 10, "ticks": 3,
             "partial_tick": float_bits(1.0, "f32"),
             "camera_x": float_bits(-40.0, "f64"), "camera_y": float_bits(70.0, "f64"),
             "camera_z": float_bits(2.0, "f64"), "render_distance": float_bits(16.0, "f64"),
             "half_height_y": float_bits(8.0, "f32"), "time_millis": 900},
           ]}
