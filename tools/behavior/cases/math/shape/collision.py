"""The `math` family: the `shape/collision` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    yield {"case_id": "shape/collide/positive-x", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(4, "f64")}

    yield {"case_id": "shape/collide/negative-x", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(2, 0.25, 0.25, 3, 0.75, 0.75), "distance": float_bits(-4, "f64")}

    yield {"case_id": "shape/collide/positive-y", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "y", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(0.25, -2, 0.25, 0.75, -1, 0.75), "distance": float_bits(4, "f64")}

    yield {"case_id": "shape/collide/negative-z", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "z", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(0.25, 0.25, 2, 0.75, 0.75, 3), "distance": float_bits(-4, "f64")}

    yield {"case_id": "shape/collide/orthogonal-gap", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(-2, 2, 0.25, -1, 3, 0.75), "distance": float_bits(4, "f64")}

    yield {"case_id": "shape/collide/zero-distance", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(0, "f64")}

    yield {"case_id": "shape/collide/empty-shape", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 0.00000005, 1, 1),
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(4, "f64")}

    yield {"case_id": "shape/collide/limited-distance", "family": "math", "operation": "shape", "kind": "collide",
           "axis": "x", "shape": box_bits(0, 0, 0, 1, 1, 1),
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(0.5, "f64")}

    yield {"case_id": "shape/collide-many/positive-nearest", "family": "math", "operation": "shape", "kind": "collide_many",
           "axis": "x", "shapes": [box_bits(2, 0, 0, 3, 1, 1), box_bits(0, 0, 0, 1, 1, 1)],
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(4, "f64")}

    yield {"case_id": "shape/collide-many/negative-nearest", "family": "math", "operation": "shape", "kind": "collide_many",
           "axis": "x", "shapes": [box_bits(-2, 0, 0, -1, 1, 1), box_bits(0, 0, 0, 1, 1, 1)],
           "moving": box_bits(2, 0.25, 0.25, 3, 0.75, 0.75), "distance": float_bits(-4, "f64")}

    yield {"case_id": "shape/collide-many/empty-first", "family": "math", "operation": "shape", "kind": "collide_many",
           "axis": "x", "shapes": [box_bits(0, 0, 0, 0.00000005, 1, 1), box_bits(0, 0, 0, 1, 1, 1)],
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(4, "f64")}

    yield {"case_id": "shape/collide-many/zero-distance", "family": "math", "kind": "collide_many", "operation": "shape",
           "axis": "x", "shapes": [box_bits(0, 0, 0, 1, 1, 1)],
           "moving": box_bits(-2, 0.25, 0.25, -1, 0.75, 0.75), "distance": float_bits(0, "f64")}

    yield {"case_id": "shape/collide-many/empty-list", "family": "math", "operation": "shape", "kind": "collide_many",
           "axis": "z", "shapes": [], "moving": box_bits(0, 0, 2, 1, 1, 3), "distance": float_bits(-4, "f64")}

    yield {"case_id": "shape/collide-movement/empty", "family": "math", "operation": "shape",
           "kind": "collide_movement", "movement": vector_bits(1.25, -2.5, 3.75),
           "moving": box_bits(0, 0, 0, 1, 1, 1), "shapes": []}

    yield {"case_id": "shape/collide-movement/x-then-z", "family": "math", "operation": "shape",
           "kind": "collide_movement", "movement": vector_bits(2, 0, 1),
           "moving": box_bits(0, 0, 0, 1, 1, 1),
           "shapes": [box_bits(2, 0, 1.5, 3, 1, 2.5), box_bits(1.5, 0, 2, 2.5, 1, 3)]}

    yield {"case_id": "shape/collide-movement/z-then-x", "family": "math", "operation": "shape",
           "kind": "collide_movement", "movement": vector_bits(1, 0, 2),
           "moving": box_bits(0, 0, 0, 1, 1, 1),
           "shapes": [box_bits(2, 0, 1.5, 3, 1, 2.5), box_bits(1.5, 0, 2, 2.5, 1, 3)]}

    yield {"case_id": "shape/collide-movement/y-first", "family": "math", "operation": "shape",
           "kind": "collide_movement", "movement": vector_bits(2, -2, 0),
           "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
           "shapes": [box_bits(-1, 0, -1, 2, 1, 2), box_bits(1, 1, -1, 2, 4, 2)]}

    yield {"case_id": "shape/entity-movement-state/base-flags", "family": "math",
           "operation": "shape", "kind": "entity_movement_state", "inputs": [
               {"label": "empty", "movement": vector_bits(1.25, -2.5, 3.75),
                "moving": box_bits(0, 0, 0, 1, 1, 1), "shapes": []},
               {"label": "horizontal-x", "movement": vector_bits(2, 0, 0),
                "moving": box_bits(0, 0, 0, 1, 1, 1),
                "shapes": [box_bits(1, -1, -1, 2, 4, 2)]},
               {"label": "horizontal-within-epsilon", "movement": vector_bits(2, 0, 0),
                "moving": box_bits(0, 0, 0, 1, 1, 1),
                "shapes": [box_bits(2.999995, -1, -1, 4, 4, 2)]},
               {"label": "falling-onto-floor", "movement": vector_bits(0, -2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 0, -1, 2, 1, 2)]},
               {"label": "rising-into-ceiling", "movement": vector_bits(0, 2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 4, -1, 2, 5, 2)]},
               {"label": "horizontal-and-falling", "movement": vector_bits(2, -2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 0, -1, 2, 1, 2),
                           box_bits(1, 1, -1, 2, 4, 2)]},
           ]}

    yield {"case_id": "shape/entity-movement-state/zero-y-retention", "family": "math",
           "operation": "shape", "kind": "entity_movement_state_retention", "inputs": [
               {"label": "zero-y-remote-retains-ground", "delta": vector_bits(2, 0, 0),
                "moving": box_bits(0, 0, 0, 1, 1, 1),
                "shapes": [box_bits(1, -1, -1, 2, 4, 2)],
                "previous": {"vertical_collision": True, "vertical_collision_below": True,
                              "on_ground": True}, "authoritative": False},
               {"label": "zero-y-authoritative-recomputes", "delta": vector_bits(2, 0, 0),
                "moving": box_bits(0, 0, 0, 1, 1, 1),
                "shapes": [box_bits(1, -1, -1, 2, 4, 2)],
                "previous": {"vertical_collision": True, "vertical_collision_below": True,
                              "on_ground": True}, "authoritative": True},
               {"label": "nonzero-y-remote-recomputes-falling", "delta": vector_bits(0, -2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 0, -1, 2, 1, 2)],
                "previous": {"vertical_collision": False, "vertical_collision_below": False,
                              "on_ground": False}, "authoritative": False},
               {"label": "nonzero-y-remote-recomputes-rising", "delta": vector_bits(0, 2, 0),
                "moving": box_bits(0.25, 2, 0.25, 0.75, 3, 0.75),
                "shapes": [box_bits(-1, 4, -1, 2, 5, 2)],
                "previous": {"vertical_collision": True, "vertical_collision_below": True,
                              "on_ground": True}, "authoritative": False},
           ]}

    yield {"case_id": "shape/entity-collision-shapes/empty-test-area", "family": "math",
           "operation": "shape", "kind": "entity_collision_shapes",
           "test_area": box_bits(0, 0, 0, 0.00000001, 0.00000001, 0.00000001),
           "source_present": False,
           "candidates": [{"box": box_bits(0, 0, 0, 1, 1, 1),
                            "can_be_collided_with": True}]}

    yield {"case_id": "shape/entity-collision-shapes/source-free-filter-order", "family": "math",
           "operation": "shape", "kind": "entity_collision_shapes",
           "test_area": box_bits(0, 0, 0, 2, 1, 1), "source_present": False,
           "candidates": [
               {"is_source": True, "box": box_bits(0.25, 0, 0, 0.75, 1, 1),
                "can_be_collided_with": True},
               {"box": box_bits(0.5, 0, 0, 1.5, 1, 1), "spectator": True,
                "can_be_collided_with": True},
               {"box": box_bits(0.75, 0, 0, 1.75, 1, 1),
                "can_be_collided_with": False},
               {"box": box_bits(1, 0, 0, 1.5, 1, 1),
                "can_be_collided_with": True, "same_vehicle": True},
               {"box": box_bits(2.00000005, 0, 0, 2.5, 1, 1),
                "can_be_collided_with": True},
           ]}

    yield {"case_id": "shape/entity-collision-shapes/source-filter-and-except", "family": "math",
           "operation": "shape", "kind": "entity_collision_shapes",
           "test_area": box_bits(0, 0, 0, 2, 1, 1), "source_present": True,
           "source_box": box_bits(0.25, 0, 0, 0.75, 1, 1),
           "candidates": [
               {"is_source": True, "box": box_bits(0.25, 0, 0, 0.75, 1, 1),
                "can_be_collided_with": True},
               {"box": box_bits(0.5, 0, 0, 1.5, 1, 1),
                "can_be_collided_with": True},
               {"box": box_bits(0.75, 0, 0, 1.75, 1, 1),
                "can_be_collided_with": True, "same_vehicle": True},
               {"box": box_bits(1, 0, 0, 1.5, 1, 1), "spectator": True,
                "can_be_collided_with": True},
               {"box": box_bits(1.25, 0, 0, 1.75, 1, 1),
                "can_be_collided_with": False},
               {"box": box_bits(2.00000005, 0, 0, 2.5, 1, 1),
                "can_be_collided_with": True},
           ]}

    yield {"case_id": "shape/collect-block-collisions/empty", "family": "math",
           "operation": "shape", "kind": "collect_block_collisions",
           "box": box_bits(-0.25, -0.25, -0.25, 0.25, 0.25, 0.25),
           "placements": [], "shape_states": []}

    yield {"case_id": "shape/collect-block-collisions/full-and-slabs", "family": "math",
           "operation": "shape", "kind": "collect_block_collisions",
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
           ]}

    yield {"case_id": "shape/collect-block-collisions/touching-faces", "family": "math",
           "operation": "shape", "kind": "collect_block_collisions",
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
           ]}

    yield {"case_id": "shape/collect-block-collisions/chunk-boundary", "family": "math",
           "operation": "shape", "kind": "collect_block_collisions",
           "box": box_bits(14.25, 0.25, 0.25, 17.75, 1.25, 1.25),
           "placements": [
               {"x": 15, "y": 0, "z": 0, "state_id": 1, "kind": "stone"},
               {"x": 16, "y": 0, "z": 0, "state_id": 3, "kind": "slab_top"},
           ],
           "shape_states": [
               {"state_id": 1, "shape": box_bits(0, 0, 0, 1, 1, 1)},
               {"state_id": 3, "shape": box_bits(0, 0.5, 0, 1, 1, 1)},
           ]}

    yield {"case_id": "shape/step-up-candidates/empty", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 2, 0, 1, 3, 1), "max_step_height": float_bits(0.6),
           "step_height_to_skip": float_bits(0.0), "shapes": []}

    yield {"case_id": "shape/step-up-candidates/sorted-unique", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 2, 0, 1, 3, 1), "max_step_height": float_bits(0.75),
           "step_height_to_skip": float_bits(-1.0),
           "shapes": [box_bits(0, 2.5, 0, 1, 2.75, 1),
                      box_bits(0, 2.0, 0, 1, 2.5, 1),
                      box_bits(0, 2.25, 0, 1, 2.5, 1)]}

    yield {"case_id": "shape/step-up-candidates/negative-skip", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 10, 0, 1, 11, 1), "max_step_height": float_bits(0.5),
           "step_height_to_skip": float_bits(0.25),
           "shapes": [box_bits(0, 9, 0, 1, 10.25, 1),
                      box_bits(0, 10.125, 0, 1, 10.5, 1),
                      box_bits(0, 10.375, 0, 1, 11, 1)]}

    yield {"case_id": "shape/step-up-candidates/zero-skipped-and-max-break", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 0, 0, 1, 1, 1), "max_step_height": float_bits(0.5),
           "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(0, 0, 0, 1, 0.75, 1),
                      box_bits(0, 0.25, 0, 1, 1, 1)]}

    yield {"case_id": "shape/step-up-candidates/signed-zero-sort", "family": "math",
           "operation": "shape", "kind": "collect_step_up_heights",
           "moving": box_bits(0, 0, 0, 1, 3, 1), "max_step_height": float_bits(2.0),
           "step_height_to_skip": float_bits(3.0),
           "shapes": [box_bits(0, -0.0, 0, 1, 2, 1),
                      box_bits(0, 0.0, 0, 1, 2, 1)]}

    yield {"case_id": "shape/step-up-selection/low-obstacle", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
           "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
           "movement": vector_bits(2, 0, 0), "movement_step": vector_bits(0, 0, 0),
           "max_step_height": float_bits(0.6), "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(1, 0, 0, 2, 0.5, 1)]}

    yield {"case_id": "shape/step-up-selection/first-better-candidate-wins", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
           "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
           "movement": vector_bits(3, 0, 0), "movement_step": vector_bits(0, 0, 0),
           "max_step_height": float_bits(0.75), "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(1, 0, 0, 2, 0.25, 1),
                      box_bits(2.5, 0, 0, 3.5, 0.5, 1)]}

    yield {"case_id": "shape/step-up-selection/equal-horizontal-keeps-step", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
           "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
           "movement": vector_bits(2, 0, 0), "movement_step": vector_bits(2, 0, 0),
           "max_step_height": float_bits(0.6), "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(1, 0, 0, 2, 0.5, 1)]}

    yield {"case_id": "shape/step-up-selection/ground-offset", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 1, 0, 1, 2, 1),
           "grounded_box": box_bits(0, 0.5, 0, 1, 1.5, 1),
           "movement": vector_bits(2, -1, 0), "movement_step": vector_bits(0, -0.5, 0),
           "max_step_height": float_bits(0.6), "step_height_to_skip": float_bits(-0.5),
           "shapes": [box_bits(1, 0.5, 0, 2, 1, 1)]}

    yield {"case_id": "shape/step-up-selection/no-candidate-falls-back", "family": "math",
           "operation": "shape", "kind": "select_step_up_movement",
           "bounding_box": box_bits(0, 0, 0, 1, 1, 1),
           "grounded_box": box_bits(0, 0, 0, 1, 1, 1),
           "movement": vector_bits(2, 0, 0), "movement_step": vector_bits(0, 0, 0),
           "max_step_height": float_bits(0.6), "step_height_to_skip": float_bits(0.0),
           "shapes": [box_bits(1, 0, 0, 2, 1, 1)]}
