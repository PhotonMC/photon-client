//! Cases for `shape/collect-block-collisions`, `shape/step-up-candidates`, `shape/step-up-selection`.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::case;
use crate::math::values::*;

fn full_and_slabs() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/collect-block-collisions/full-and-slabs",
        "math",
        "shape",
        json!({
            "kind": "collect_block_collisions",
            "box": box_bits(0.25, 0.25, 0.25, 2.75, 1.75, 1.75),
            "placements": [
                {"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"},
                {"x": 1, "y": 0, "z": 0, "state_id": 2, "kind": "slab_bottom"},
                {"x": 1, "y": 1, "z": 0, "state_id": 3, "kind": "slab_top"},
                {"x": 2, "y": 0, "z": 0, "state_id": 4, "kind": "slab_double"},
            ],
            "shape_states": [
                {"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)},
                {"state_id": 2, "shape": box_bits(0, 0, 0, 1, 0.5, 1)},
                {"state_id": 3, "shape": box_bits(0, 0.5, 0, 1, 1, 1)},
                {"state_id": 4, "shape": box_bits(0, 0, 0, 1, 1, 1)},
            ],
        }),
    ));
    out.push(case(
        "shape/collect-block-collisions/touching-faces",
        "math",
        "shape",
        json!({
            "kind": "collect_block_collisions",
            "box": box_bits(0, 0, 0, 1, 1, 1),
            "placements": [
                {"x": 0, "y": 0, "z": 0, "state_id": 2, "kind": "slab_bottom"},
                {"x": 1, "y": 0, "z": 0, "state_id": 1, "kind": "stone"},
                {"x": 0, "y": 1, "z": 0, "state_id": 1, "kind": "stone"},
                {"x": 0, "y": 0, "z": 1, "state_id": 1, "kind": "stone"},
            ],
            "shape_states": [
                {"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)},
                {"state_id": 2, "shape": box_bits(0, 0, 0, 1, 0.5, 1)},
            ],
        }),
    ));
    out
}

fn chunk_boundary() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/collect-block-collisions/chunk-boundary",
        "math",
        "shape",
        json!({
            "kind": "collect_block_collisions",
            "box": box_bits(14.25, 0.25, 0.25, 17.75, 1.25, 1.25),
            "placements": [
                {"x": 15, "y": 0, "z": 0, "state_id": 1, "kind": "stone"},
                {"x": 16, "y": 0, "z": 0, "state_id": 3, "kind": "slab_top"},
            ],
            "shape_states": [
                {"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)},
                {"state_id": 3, "shape": box_bits(0, 0.5, 0, 1, 1, 1)},
            ],
        }),
    ));
    out.push(case(
        "shape/step-up-candidates/empty",
        "math",
        "shape",
        json!({
            "kind": "collect_step_up_heights",
            "moving": box_bits(0, 2, 0, 1, 3, 1),
            "max_step_height": f32_bits(0.6),
            "step_height_to_skip": f32_bits(0.0),
            "shapes": [],
        }),
    ));
    out.push(case(
        "shape/step-up-candidates/sorted-unique",
        "math",
        "shape",
        json!({
            "kind": "collect_step_up_heights",
            "moving": box_bits(0, 2, 0, 1, 3, 1),
            "max_step_height": f32_bits(0.75),
            "step_height_to_skip": f32_bits(-1.0),
            "shapes": [
                box_bits(0, 2.5, 0, 1, 2.75, 1),
                box_bits(0, 2.0, 0, 1, 2.5, 1),
                box_bits(0, 2.25, 0, 1, 2.5, 1),
            ],
        }),
    ));
    out
}

fn negative_skip() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/step-up-candidates/negative-skip",
        "math",
        "shape",
        json!({
            "kind": "collect_step_up_heights",
            "moving": box_bits(0, 10, 0, 1, 11, 1),
            "max_step_height": f32_bits(0.5),
            "step_height_to_skip": f32_bits(0.25),
            "shapes": [
                box_bits(0, 9, 0, 1, 10.25, 1),
                box_bits(0, 10.125, 0, 1, 10.5, 1),
                box_bits(0, 10.375, 0, 1, 11, 1),
            ],
        }),
    ));
    out.push(case(
        "shape/step-up-candidates/zero-skipped-and-max-break",
        "math",
        "shape",
        json!({
            "kind": "collect_step_up_heights",
            "moving": box_bits(0, 0, 0, 1, 1, 1),
            "max_step_height": f32_bits(0.5),
            "step_height_to_skip": f32_bits(0.0),
            "shapes": [box_bits(0, 0, 0, 1, 0.75, 1), box_bits(0, 0.25, 0, 1, 1, 1)],
        }),
    ));
    out.push(case(
        "shape/step-up-candidates/signed-zero-sort",
        "math",
        "shape",
        json!({
            "kind": "collect_step_up_heights",
            "moving": box_bits(0, 0, 0, 1, 3, 1),
            "max_step_height": f32_bits(2.0),
            "step_height_to_skip": f32_bits(3.0),
            "shapes": [box_bits(0, -0.0, 0, 1, 2, 1), box_bits(0, 0.0, 0, 1, 2, 1)],
        }),
    ));
    out
}

fn low_obstacle() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/step-up-selection/low-obstacle",
        "math",
        "shape",
        json!({
            "kind": "select_step_up_movement",
            "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
            "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
            "movement": vector_bits(2, 0, 0),
            "movement_step": vector_bits(0, 0, 0),
            "max_step_height": f32_bits(0.6),
            "step_height_to_skip": f32_bits(0.0),
            "shapes": [box_bits(1, 0, 0, 2, 0.5, 1)],
        }),
    ));
    out.push(case(
        "shape/step-up-selection/first-better-candidate-wins",
        "math",
        "shape",
        json!({
            "kind": "select_step_up_movement",
            "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
            "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
            "movement": vector_bits(3, 0, 0),
            "movement_step": vector_bits(0, 0, 0),
            "max_step_height": f32_bits(0.75),
            "step_height_to_skip": f32_bits(0.0),
            "shapes": [box_bits(1, 0, 0, 2, 0.25, 1), box_bits(2.5, 0, 0, 3.5, 0.5, 1)],
        }),
    ));
    out.push(case(
        "shape/step-up-selection/equal-horizontal-keeps-step",
        "math",
        "shape",
        json!({
            "kind": "select_step_up_movement",
            "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
            "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
            "movement": vector_bits(2, 0, 0),
            "movement_step": vector_bits(2, 0, 0),
            "max_step_height": f32_bits(0.6),
            "step_height_to_skip": f32_bits(0.0),
            "shapes": [box_bits(1, 0, 0, 2, 0.5, 1)],
        }),
    ));
    out
}

fn ground_offset() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/step-up-selection/ground-offset",
        "math",
        "shape",
        json!({
            "kind": "select_step_up_movement",
            "bounding_box": box_bits(0, 1, 0, 1, 2, 1),
            "grounded_box": box_bits(0, 0.5, 0, 1, 1.5, 1),
            "movement": vector_bits(2, -1, 0),
            "movement_step": vector_bits(0, -0.5, 0),
            "max_step_height": f32_bits(0.6),
            "step_height_to_skip": f32_bits(-0.5),
            "shapes": [box_bits(1, 0.5, 0, 2, 1, 1)],
        }),
    ));
    out.push(case(
        "shape/step-up-selection/no-candidate-falls-back",
        "math",
        "shape",
        json!({
            "kind": "select_step_up_movement",
            "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
            "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
            "movement": vector_bits(2, 0, 0),
            "movement_step": vector_bits(0, 0, 0),
            "max_step_height": f32_bits(0.6),
            "step_height_to_skip": f32_bits(0.0),
            "shapes": [box_bits(1, 0, 0, 2, 1, 1)],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [full_and_slabs(), chunk_boundary(), negative_skip(), low_obstacle(), ground_offset()].concat()
}
