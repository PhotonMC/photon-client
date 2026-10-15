//! Cases for `shape/world-border-collision-shape`, `world/client-border-state`, `shape/world-border-render`.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;

fn static_extents() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/world-border-collision-shape/static-extents",
        "math",
        "shape",
        json!({
            "kind": "world_border_collision_shape",
            "inputs": [
                {
                    "center_x": f64_bits(0.0),
                    "center_z": f64_bits(0.0),
                    "size": f64_bits(5.5),
                    "absolute_max_size": 100,
                },
                {
                    "center_x": f64_bits(10.25),
                    "center_z": f64_bits(-4.75),
                    "size": f64_bits(9.5),
                    "absolute_max_size": 100,
                },
                {
                    "center_x": f64_bits(90.0),
                    "center_z": f64_bits(0.0),
                    "size": f64_bits(50.0),
                    "absolute_max_size": 100,
                },
                {
                    "center_x": f64_bits(0.0),
                    "center_z": f64_bits(0.0),
                    "size": f64_bits(300.0),
                    "absolute_max_size": 100,
                },
            ],
        }),
    ));
    out
}

fn moving_extents() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/world-border-collision-shape/moving-extents",
        "math",
        "shape",
        json!({
            "kind": "world_border_collision_shape_moving",
            "inputs": [
                {
                    "center_x": f64_bits(1.25),
                    "center_z": f64_bits(-2.5),
                    "from": f64_bits(20.0),
                    "to": f64_bits(40.0),
                    "absolute_max_size": 100,
                    "duration": 4,
                    "tick_count": 4,
                    "partial_ticks": [f32_bits(0.0), f32_bits(0.25), f32_bits(0.5), f32_bits(1.0)],
                },
                {
                    "center_x": f64_bits(-10.25),
                    "center_z": f64_bits(4.75),
                    "from": f64_bits(40.0),
                    "to": f64_bits(10.0),
                    "absolute_max_size": 15,
                    "duration": 3,
                    "tick_count": 3,
                    "partial_ticks": [f32_bits(0.0), f32_bits(0.25), f32_bits(0.5), f32_bits(1.0)],
                },
                {
                    "center_x": f64_bits(90.0),
                    "center_z": f64_bits(0.0),
                    "from": f64_bits(50.0),
                    "to": f64_bits(300.0),
                    "absolute_max_size": 100,
                    "duration": 2,
                    "tick_count": 2,
                    "partial_ticks": [f32_bits(0.0), f32_bits(0.5), f32_bits(1.0)],
                },
            ],
        }),
    ));
    out
}

fn packet_application() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "world/client-border-state/packet-application",
        "math",
        "shape",
        json!({
            "kind": "client_world_border_state",
            "inputs": [
                {
                    "events": [
                        {
                            "op": "initialize",
                            "center_x": f64_bits(2.25),
                            "center_z": f64_bits(-3.5),
                            "old_size": f64_bits(20.0),
                            "new_size": f64_bits(40.0),
                            "lerp_time": 4,
                            "absolute_max_size": 30,
                            "warning_blocks": 7,
                            "warning_time": 20,
                        },
                        {"op": "tick"},
                        {"op": "tick"},
                        {"op": "center", "center_x": f64_bits(4.0), "center_z": f64_bits(-1.0)},
                        {"op": "lerp", "old_size": f64_bits(30.0), "new_size": f64_bits(10.0), "lerp_time": 3},
                        {"op": "tick"},
                        {"op": "tick"},
                        {"op": "tick"},
                        {"op": "warning_blocks", "value": 12},
                        {"op": "warning_time", "value": 40},
                    ],
                },
                {
                    "events": [
                        {
                            "op": "initialize",
                            "center_x": f64_bits(-8.0),
                            "center_z": f64_bits(5.0),
                            "old_size": f64_bits(1.0),
                            "new_size": f64_bits(24.0),
                            "lerp_time": 0,
                            "absolute_max_size": 12,
                            "warning_blocks": 3,
                            "warning_time": 9,
                        },
                        {"op": "size", "size": f64_bits(8.0)},
                        {"op": "center", "center_x": f64_bits(-5.5), "center_z": f64_bits(2.5)},
                        {"op": "lerp", "old_size": f64_bits(8.0), "new_size": f64_bits(8.0), "lerp_time": 5},
                        {"op": "warning_blocks", "value": 0},
                        {"op": "warning_time", "value": 0},
                    ],
                },
            ],
        }),
    ));
    out
}

fn extract_and_quads() -> Value {
    case(
        "shape/world-border-render/extract-and-quads",
        "math",
        "shape",
        json!({
            "kind": "world_border_render",
            "inputs": extract_and_quads_inputs(),
        }),
    )
}

fn extract_and_quads_inputs_center_x() -> Value {
    json!({
        "center_x": f64_bits(0.0),
        "center_z": f64_bits(0.0),
        "absolute_max_size": 1000,
        "from": f64_bits(100.0),
        "to": f64_bits(40.0),
        "duration": 10,
        "ticks": 3,
        "partial_tick": f32_bits(1.0),
        "camera_x": f64_bits(-40.0),
        "camera_y": f64_bits(70.0),
        "camera_z": f64_bits(2.0),
        "render_distance": f64_bits(16.0),
        "half_height_y": f32_bits(8.0),
        "time_millis": 900,
    })
}

fn extract_and_quads_inputs_center_z() -> Value {
    json!({
        "center_x": f64_bits(10.0),
        "center_z": f64_bits(-5.0),
        "absolute_max_size": 1000,
        "from": f64_bits(20.0),
        "to": f64_bits(60.0),
        "duration": 8,
        "ticks": 4,
        "partial_tick": f32_bits(0.5),
        "camera_x": f64_bits(39.0),
        "camera_y": f64_bits(71.3),
        "camera_z": f64_bits(-5.0),
        "render_distance": f64_bits(16.0),
        "half_height_y": f32_bits(10.0),
        "time_millis": 4200,
    })
}

fn extract_and_quads_inputs() -> Value {
    json!([
        {
            "center_x": f64_bits(0.0),
            "center_z": f64_bits(0.0),
            "absolute_max_size": 1000,
            "size": f64_bits(100.0),
            "partial_tick": f32_bits(0.0),
            "camera_x": f64_bits(45.0),
            "camera_y": f64_bits(70.0),
            "camera_z": f64_bits(0.0),
            "render_distance": f64_bits(16.0),
            "half_height_y": f32_bits(8.0),
            "time_millis": 0,
        },
        {
            "center_x": f64_bits(0.0),
            "center_z": f64_bits(0.0),
            "absolute_max_size": 1000,
            "size": f64_bits(100.0),
            "partial_tick": f32_bits(0.0),
            "camera_x": f64_bits(45.0),
            "camera_y": f64_bits(70.0),
            "camera_z": f64_bits(45.0),
            "render_distance": f64_bits(16.0),
            "half_height_y": f32_bits(8.0),
            "time_millis": 0,
        },
        {
            "center_x": f64_bits(0.0),
            "center_z": f64_bits(0.0),
            "absolute_max_size": 10000,
            "size": f64_bits(100.0),
            "partial_tick": f32_bits(0.0),
            "camera_x": f64_bits(1000.0),
            "camera_y": f64_bits(70.0),
            "camera_z": f64_bits(1000.0),
            "render_distance": f64_bits(16.0),
            "half_height_y": f32_bits(8.0),
            "time_millis": 0,
        },
        extract_and_quads_inputs_center_z(),
        extract_and_quads_inputs_center_x(),
    ])
}

pub fn cases() -> Vec<Value> {
    [static_extents(), moving_extents(), packet_application(), vec![extract_and_quads()]].concat()
}
