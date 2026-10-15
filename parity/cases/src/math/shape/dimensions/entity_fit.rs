//! Cases for `shape/entity-fit`, `shape/living-entity-dimensions`, `shape/entity-dimensions-refresh`, `shape/player-pose-dimensions`.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::case;
use crate::math::values::*;

fn empty_default() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/entity-fit/empty-default",
        "math",
        "shape",
        json!({
            "kind": "entity_fit",
            "position": vector_bits(0.5, 0.0, 0.5),
            "dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.8)},
            "placements": [],
            "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}],
        }),
    ));
    out.push(case(
        "shape/entity-fit/scaled-and-eye-height",
        "math",
        "shape",
        json!({
            "kind": "entity_fit",
            "position": vector_bits(-1.25, 2.0, 3.75),
            "dimensions": {
                "width": f32_bits(0.6),
                "height": f32_bits(1.8),
                "width_scale": f32_bits(0.5),
                "height_scale": f32_bits(2.0),
                "eye_height": f32_bits(2.75),
            },
            "placements": [],
            "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}],
        }),
    ));
    out
}

fn fixed_ignores_scale() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/entity-fit/fixed-ignores-scale",
        "math",
        "shape",
        json!({
            "kind": "entity_fit",
            "position": vector_bits(2.0, 0.0, -2.0),
            "dimensions": {
                "width": f32_bits(1.0),
                "height": f32_bits(2.0),
                "fixed": true,
                "width_scale": f32_bits(0.5),
                "height_scale": f32_bits(0.5),
            },
            "placements": [],
            "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}],
        }),
    ));
    out.push(case(
        "shape/entity-fit/overlap",
        "math",
        "shape",
        json!({
            "kind": "entity_fit",
            "position": vector_bits(0.5, 0.25, 0.5),
            "dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.0)},
            "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"}],
            "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}],
        }),
    ));
    out.push(case(
        "shape/entity-fit/touching-face-after-deflate",
        "math",
        "shape",
        json!({
            "kind": "entity_fit",
            "position": vector_bits(1.5, 0.25, 0.5),
            "dimensions": {"width": f32_bits(1.0), "height": f32_bits(1.0)},
            "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"}],
            "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}],
        }),
    ));
    out
}

fn epsilon_intrusion() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/entity-fit/epsilon-intrusion",
        "math",
        "shape",
        json!({
            "kind": "entity_fit",
            "position": vector_bits(1.4999997, 0.25, 0.5),
            "dimensions": {"width": f32_bits(1.0), "height": f32_bits(1.0)},
            "placements": [{"x": 0, "y": 0, "z": 0, "state_id": 1, "kind": "stone"}],
            "shape_states": [{"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)}],
        }),
    ));
    out
}

fn scale_branches() -> Value {
    case(
        "shape/living-entity-dimensions/scale-branches",
        "math",
        "shape",
        json!({
            "kind": "living_entity_dimensions_scale",
            "inputs": [
                {
                    "label": "regular-age-then-entity-scale",
                    "pose_id": 0,
                    "position": vector_bits(-1.25, 2.0, 3.75),
                    "default_dimensions": {
                        "width": f32_bits(0.6),
                        "height": f32_bits(1.8),
                        "eye_height": f32_bits(1.62),
                    },
                    "age_scale": f32_bits(0.5),
                    "entity_scale": f32_bits(1.5),
                },
                {
                    "label": "sleeping-ignores-both-scales",
                    "pose_id": 2,
                    "position": vector_bits(4.0, -2.0, -3.0),
                    "default_dimensions": {
                        "width": f32_bits(0.6),
                        "height": f32_bits(1.8),
                        "eye_height": f32_bits(1.62),
                    },
                    "age_scale": f32_bits(0.5),
                    "entity_scale": f32_bits(2.0),
                },
                fixed_default_ignores_entity_scale(),
                {
                    "label": "regular-height-only-rounding",
                    "pose_id": 3,
                    "position": vector_bits(2.5, 0.0, 1.25),
                    "default_dimensions": {
                        "width": f32_bits(0.75),
                        "height": f32_bits(1.25),
                        "eye_height": f32_bits(0.9),
                    },
                    "age_scale": f32_bits(0.7),
                    "entity_scale": f32_bits(1.3),
                },
            ],
        }),
    )
}

fn fixed_default_ignores_entity_scale() -> Value {
    json!({
        "label": "fixed-default-ignores-entity-scale",
        "pose_id": 5,
        "position": vector_bits(0.25, 1.0, -0.75),
        "default_dimensions": {
            "width": f32_bits(1.0),
            "height": f32_bits(2.0),
            "eye_height": f32_bits(1.7),
            "fixed": true,
        },
        "age_scale": f32_bits(0.5),
        "entity_scale": f32_bits(2.0),
    })
}

fn client_reapply() -> Value {
    case(
        "shape/entity-dimensions-refresh/client-reapply",
        "math",
        "shape",
        json!({
            "kind": "entity_dimensions_refresh",
            "inputs": client_reapply_inputs(),
        }),
    )
}

fn client_shrink_to_sleeping() -> Value {
    json!({
        "label": "client-shrink-to-sleeping",
        "position": vector_bits(4.0, -2.0, -3.0),
        "old_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.8), "eye_height": f32_bits(1.62)},
        "new_dimensions": {
            "width": f32_bits(0.2),
            "height": f32_bits(0.2),
            "eye_height": f32_bits(0.2),
            "fixed": true,
        },
        "level_client_side": true,
        "first_tick": false,
        "no_physics": false,
        "is_player": false,
    })
}

fn server_large_dimensions_gate() -> Value {
    json!({
        "label": "server-large-dimensions-gate",
        "position": vector_bits(0.0, 0.0, 0.0),
        "old_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.5), "eye_height": f32_bits(1.27)},
        "new_dimensions": {"width": f32_bits(4.1), "height": f32_bits(1.8), "eye_height": f32_bits(1.62)},
        "level_client_side": false,
        "first_tick": false,
        "no_physics": false,
        "is_player": false,
    })
}

fn server_no_physics_gate() -> Value {
    json!({
        "label": "server-no-physics-gate",
        "position": vector_bits(1.5, -1.0, 2.25),
        "old_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.5), "eye_height": f32_bits(1.27)},
        "new_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.8), "eye_height": f32_bits(1.62)},
        "level_client_side": false,
        "first_tick": false,
        "no_physics": true,
        "is_player": false,
    })
}

fn client_reapply_inputs() -> Value {
    json!([
        {
            "label": "client-growth-reapplies-box",
            "position": vector_bits(-1.25, 2.0, 3.75),
            "old_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.5), "eye_height": f32_bits(1.27)},
            "new_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.8), "eye_height": f32_bits(1.62)},
            "level_client_side": true,
            "first_tick": false,
            "no_physics": false,
            "is_player": false,
        },
        client_shrink_to_sleeping(),
        {
            "label": "server-growth-eligible",
            "position": vector_bits(0.25, 1.0, -0.75),
            "old_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.5), "eye_height": f32_bits(1.27)},
            "new_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.8), "eye_height": f32_bits(1.62)},
            "level_client_side": false,
            "first_tick": false,
            "no_physics": false,
            "is_player": false,
        },
        {
            "label": "server-player-excluded",
            "position": vector_bits(2.5, 0.0, 1.25),
            "old_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.5), "eye_height": f32_bits(1.27)},
            "new_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.8), "eye_height": f32_bits(1.62)},
            "level_client_side": false,
            "first_tick": false,
            "no_physics": false,
            "is_player": true,
        },
        {
            "label": "server-first-tick-gate",
            "position": vector_bits(-2.0, 3.0, 0.5),
            "old_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.5), "eye_height": f32_bits(1.27)},
            "new_dimensions": {"width": f32_bits(0.6), "height": f32_bits(1.8), "eye_height": f32_bits(1.62)},
            "level_client_side": false,
            "first_tick": true,
            "no_physics": false,
            "is_player": false,
        },
        server_no_physics_gate(),
        server_large_dimensions_gate(),
    ])
}

fn all_poses_and_fallback() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/player-pose-dimensions/all-poses-and-fallback",
        "math",
        "shape",
        json!({
            "kind": "player_pose_dimensions",
            "position": vector_bits(-2.25, 4.0, 1.75),
            "pose_ids": ([(0_i64..18).collect::<Vec<_>>(), vec![-1, 18]].concat()),
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        empty_default(),
        fixed_ignores_scale(),
        epsilon_intrusion(),
        vec![scale_branches()],
        vec![client_reapply()],
        all_poses_and_fallback(),
    ]
    .concat()
}
