//! Cases for `shape/collide`, `shape/collide-many`, `shape/collide-movement`, `shape/entity-movement-state` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn positive_x() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/collide/positive-x",
        "math",
        "shape",
        json!({
            "kind": "collide",
            "axis": "x",
            "shape": box_bits(0, 0, 0, 1, 1, 1),
            "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75),
            "distance": f64_bits(4),
        }),
    ));
    out.push(case(
        "shape/collide/negative-x",
        "math",
        "shape",
        json!({
            "kind": "collide",
            "axis": "x",
            "shape": box_bits(0, 0, 0, 1, 1, 1),
            "moving": box_bits(2, 0.25, 0.25, 3, 0.75, 0.75),
            "distance": f64_bits(-4),
        }),
    ));
    out.push(case(
        "shape/collide/positive-y",
        "math",
        "shape",
        json!({
            "kind": "collide",
            "axis": "y",
            "shape": box_bits(0, 0, 0, 1, 1, 1),
            "moving": box_bits(0.25, -2, 0.25, 0.75, -1, 0.75),
            "distance": f64_bits(4),
        }),
    ));
    out
}

fn negative_z() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/collide/negative-z",
        "math",
        "shape",
        json!({
            "kind": "collide",
            "axis": "z",
            "shape": box_bits(0, 0, 0, 1, 1, 1),
            "moving": box_bits(0.25, 0.25, 2, 0.75, 0.75, 3),
            "distance": f64_bits(-4),
        }),
    ));
    out.push(case(
        "shape/collide/orthogonal-gap",
        "math",
        "shape",
        json!({
            "kind": "collide",
            "axis": "x",
            "shape": box_bits(0, 0, 0, 1, 1, 1),
            "moving": box_bits(-2, 2, 0.25, -1, 3, 0.75),
            "distance": f64_bits(4),
        }),
    ));
    out.push(case(
        "shape/collide/zero-distance",
        "math",
        "shape",
        json!({
            "kind": "collide",
            "axis": "x",
            "shape": box_bits(0, 0, 0, 1, 1, 1),
            "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75),
            "distance": f64_bits(0),
        }),
    ));
    out
}

fn empty_shape() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/collide/empty-shape",
        "math",
        "shape",
        json!({
            "kind": "collide",
            "axis": "x",
            "shape": box_bits(0, 0, 0, 5e-08, 1, 1),
            "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75),
            "distance": f64_bits(4),
        }),
    ));
    out.push(case(
        "shape/collide/limited-distance",
        "math",
        "shape",
        json!({
            "kind": "collide",
            "axis": "x",
            "shape": box_bits(0, 0, 0, 1, 1, 1),
            "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75),
            "distance": f64_bits(0.5),
        }),
    ));
    out.push(case(
        "shape/collide-many/positive-nearest",
        "math",
        "shape",
        json!({
            "kind": "collide_many",
            "axis": "x",
            "shapes": [box_bits(2, 0, 0, 3, 1, 1), box_bits(0, 0, 0, 1, 1, 1)],
            "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75),
            "distance": f64_bits(4),
        }),
    ));
    out
}

fn negative_nearest() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/collide-many/negative-nearest",
        "math",
        "shape",
        json!({
            "kind": "collide_many",
            "axis": "x",
            "shapes": [box_bits(-2, 0, 0, -1, 1, 1), box_bits(0, 0, 0, 1, 1, 1)],
            "moving": box_bits(2, 0.25, 0.25, 3, 0.75, 0.75),
            "distance": f64_bits(-4),
        }),
    ));
    out.push(case(
        "shape/collide-many/empty-first",
        "math",
        "shape",
        json!({
            "kind": "collide_many",
            "axis": "x",
            "shapes": [box_bits(0, 0, 0, 5e-08, 1, 1), box_bits(0, 0, 0, 1, 1, 1)],
            "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75),
            "distance": f64_bits(4),
        }),
    ));
    out.push(json!({
        "case_id": "shape/collide-many/zero-distance",
        "family": "math",
        "kind": "collide_many",
        "operation": "shape",
        "axis": "x",
        "shapes": [box_bits(0, 0, 0, 1, 1, 1)],
        "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75),
        "distance": f64_bits(0),
    }));
    out
}

fn empty_list() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/collide-many/empty-list",
        "math",
        "shape",
        json!({
            "kind": "collide_many",
            "axis": "z",
            "shapes": [],
            "moving": box_bits(0, 0, 2, 1, 1, 3),
            "distance": f64_bits(-4),
        }),
    ));
    out.push(case(
        "shape/collide-movement/empty",
        "math",
        "shape",
        json!({
            "kind": "collide_movement",
            "movement": vector_bits(1.25, -2.5, 3.75),
            "moving": box_bits(0, 0, 0, 1, 1, 1),
            "shapes": [],
        }),
    ));
    out.push(case(
        "shape/collide-movement/x-then-z",
        "math",
        "shape",
        json!({
            "kind": "collide_movement",
            "movement": vector_bits(2, 0, 1),
            "moving": box_bits(0, 0, 0, 1, 1, 1),
            "shapes": [box_bits(2, 0, 1.5, 3, 1, 2.5), box_bits(1.5, 0, 2, 2.5, 1, 3)],
        }),
    ));
    out.push(case(
        "shape/collide-movement/z-then-x",
        "math",
        "shape",
        json!({
            "kind": "collide_movement",
            "movement": vector_bits(1, 0, 2),
            "moving": box_bits(0, 0, 0, 1, 1, 1),
            "shapes": [box_bits(2, 0, 1.5, 3, 1, 2.5), box_bits(1.5, 0, 2, 2.5, 1, 3)],
        }),
    ));
    out
}

fn y_first() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/collide-movement/y-first",
        "math",
        "shape",
        json!({
            "kind": "collide_movement",
            "movement": vector_bits(2, -2, 0),
            "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
            "shapes": [box_bits(-1, 0, -1, 2, 1, 2), box_bits(1, 1, -1, 2, 4, 2)],
        }),
    ));
    out
}

fn base_flags() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/entity-movement-state/base-flags",
        "math",
        "shape",
        json!({
            "kind": "entity_movement_state",
            "inputs": [
                {
                    "label": "empty",
                    "movement": vector_bits(1.25, -2.5, 3.75),
                    "moving": box_bits(0, 0, 0, 1, 1, 1),
                    "shapes": [],
                },
                {
                    "label": "horizontal-x",
                    "movement": vector_bits(2, 0, 0),
                    "moving": box_bits(0, 0, 0, 1, 1, 1),
                    "shapes": [box_bits(1, -1, -1, 2, 4, 2)],
                },
                {
                    "label": "horizontal-within-epsilon",
                    "movement": vector_bits(2, 0, 0),
                    "moving": box_bits(0, 0, 0, 1, 1, 1),
                    "shapes": [box_bits(2.999995, -1, -1, 4, 4, 2)],
                },
                {
                    "label": "falling-onto-floor",
                    "movement": vector_bits(0, -2, 0),
                    "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                    "shapes": [box_bits(-1, 0, -1, 2, 1, 2)],
                },
                {
                    "label": "rising-into-ceiling",
                    "movement": vector_bits(0, 2, 0),
                    "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                    "shapes": [box_bits(-1, 4, -1, 2, 5, 2)],
                },
                {
                    "label": "horizontal-and-falling",
                    "movement": vector_bits(2, -2, 0),
                    "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                    "shapes": [box_bits(-1, 0, -1, 2, 1, 2), box_bits(1, 1, -1, 2, 4, 2)],
                },
            ],
        }),
    ));
    out
}

fn zero_y_retention() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/entity-movement-state/zero-y-retention",
        "math",
        "shape",
        json!({
            "kind": "entity_movement_state_retention",
            "inputs": [
                {
                    "label": "zero-y-remote-retains-ground",
                    "delta": vector_bits(2, 0, 0),
                    "moving": box_bits(0, 0, 0, 1, 1, 1),
                    "shapes": [box_bits(1, -1, -1, 2, 4, 2)],
                    "previous": {"vertical_collision": true, "vertical_collision_below": true, "on_ground": true},
                    "authoritative": false,
                },
                {
                    "label": "zero-y-authoritative-recomputes",
                    "delta": vector_bits(2, 0, 0),
                    "moving": box_bits(0, 0, 0, 1, 1, 1),
                    "shapes": [box_bits(1, -1, -1, 2, 4, 2)],
                    "previous": {"vertical_collision": true, "vertical_collision_below": true, "on_ground": true},
                    "authoritative": true,
                },
                {
                    "label": "nonzero-y-remote-recomputes-falling",
                    "delta": vector_bits(0, -2, 0),
                    "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                    "shapes": [box_bits(-1, 0, -1, 2, 1, 2)],
                    "previous": {"vertical_collision": false, "vertical_collision_below": false, "on_ground": false},
                    "authoritative": false,
                },
                {
                    "label": "nonzero-y-remote-recomputes-rising",
                    "delta": vector_bits(0, 2, 0),
                    "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                    "shapes": [box_bits(-1, 4, -1, 2, 5, 2)],
                    "previous": {"vertical_collision": true, "vertical_collision_below": true, "on_ground": true},
                    "authoritative": false,
                },
            ],
        }),
    ));
    out
}

fn empty_test_area() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/entity-collision-shapes/empty-test-area",
        "math",
        "shape",
        json!({
            "kind": "entity_collision_shapes",
            "test_area": box_bits(0, 0, 0, 1e-08, 1e-08, 1e-08),
            "source_present": false,
            "candidates": [{"box": box_bits(0, 0, 0, 1, 1, 1), "can_be_collided_with": true}],
        }),
    ));
    out.push(case(
        "shape/entity-collision-shapes/source-free-filter-order",
        "math",
        "shape",
        json!({
            "kind": "entity_collision_shapes",
            "test_area": box_bits(0, 0, 0, 2, 1, 1),
            "source_present": false,
            "candidates": [
                {"is_source": true, "box": box_bits(0.25, 0, 0, 0.75, 1, 1), "can_be_collided_with": true},
                {"box": box_bits(0.5, 0, 0, 1.5, 1, 1), "spectator": true, "can_be_collided_with": true},
                {"box": box_bits(0.75, 0, 0, 1.75, 1, 1), "can_be_collided_with": false},
                {"box": box_bits(1, 0, 0, 1.5, 1, 1), "can_be_collided_with": true, "same_vehicle": true},
                {"box": box_bits(2.00000005, 0, 0, 2.5, 1, 1), "can_be_collided_with": true},
            ],
        }),
    ));
    out
}

fn source_filter_and_except() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/entity-collision-shapes/source-filter-and-except",
        "math",
        "shape",
        json!({
            "kind": "entity_collision_shapes",
            "test_area": box_bits(0, 0, 0, 2, 1, 1),
            "source_present": true,
            "source_box": box_bits(0.25, 0, 0, 0.75, 1, 1),
            "candidates": [
                {"is_source": true, "box": box_bits(0.25, 0, 0, 0.75, 1, 1), "can_be_collided_with": true},
                {"box": box_bits(0.5, 0, 0, 1.5, 1, 1), "can_be_collided_with": true},
                {"box": box_bits(0.75, 0, 0, 1.75, 1, 1), "can_be_collided_with": true, "same_vehicle": true},
                {"box": box_bits(1, 0, 0, 1.5, 1, 1), "spectator": true, "can_be_collided_with": true},
                {"box": box_bits(1.25, 0, 0, 1.75, 1, 1), "can_be_collided_with": false},
                {"box": box_bits(2.00000005, 0, 0, 2.5, 1, 1), "can_be_collided_with": true},
            ],
        }),
    ));
    out.push(case(
        "shape/collect-block-collisions/empty",
        "math",
        "shape",
        json!({
            "kind": "collect_block_collisions",
            "box": box_bits(-0.25, -0.25, -0.25, 0.25, 0.25, 0.25),
            "placements": [],
            "shape_states": [],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        positive_x(),
        negative_z(),
        empty_shape(),
        negative_nearest(),
        empty_list(),
        y_first(),
        base_flags(),
        zero_y_retention(),
        empty_test_area(),
        source_filter_and_except(),
    ]
    .concat()
}
