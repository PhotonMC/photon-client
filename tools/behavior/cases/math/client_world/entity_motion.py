"""The `math` family: the `client_world/entity_motion` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.client_world.shared import *


def cases():
    remote_player_head_defaults = {
        "action": "rotate_head",
        "head_target": None,
        "ticks": 0,
    }

    remote_player_head_operations = [
        {
            "label": "packet-target-wraps-across-zero",
            "head_target": float_bits(10.0),
        },
        {
            "label": "first-tick-uses-shortest-path",
            "ticks": 1,
        },
        {
            "label": "second-tick-before-replacement",
            "ticks": 1,
        },
        {
            "label": "new-packet-restarts-three-step-target",
            "head_target": float_bits(-180.0),
        },
        {
            "label": "replacement-first-tick",
            "ticks": 1,
        },
        {
            "label": "replacement-reaches-after-two-more-ticks",
            "ticks": 2,
        },
        {
            "label": "negative-packet-target-starts-new-interpolation",
            "head_target": float_bits(-90.0),
        },
        {
            "label": "negative-target-reaches-after-three-ticks",
            "ticks": 3,
        },
    ]

    yield {
        "case_id": "client-world/entity-player-head-rotation",
        "family": "math",
        "operation": "client_world_entity_player_head_rotation",
        "entity_id": 77,
        "entity_type_id": 156,
        "initial_position": vector_bits(10.0, 64.0, -2.0),
        "initial_rotation": {
            "y": float_bits(15.0),
            "x": float_bits(-10.0),
        },
        "initial_head": float_bits(350.0),
        "operations": [
            {**remote_player_head_defaults, **operation}
            for operation in remote_player_head_operations
        ],
    }

    yield {
        "case_id": "client-world/entity-player-interpolation",
        "family": "math",
        "operation": "client_world_entity_player_interpolation",
        "entity_id": 37,
        "initial_position": vector_bits(1.0, 80.0, -35.5),
        "initial_rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(350.0),
        "operations": [
            {
                "action": "move",
                "label": "packet-target-starts-interpolation",
                "delta": {"x": 12288, "y": 8192, "z": 4096},
                "target_position": vector_bits(4.0, 82.0, -34.5),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
            },
            {"action": "tick", "label": "first-interpolation-tick", "ticks": 1},
            {
                "action": "move",
                "label": "same-target-keeps-remaining-steps",
                "delta": {"x": 0, "y": 0, "z": 0},
                "target_position": vector_bits(4.0, 82.0, -34.5),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
            },
            {"action": "tick", "label": "second-interpolation-tick", "ticks": 1},
            {
                "action": "move",
                "label": "changed-target-restarts-interpolation",
                "delta": {"x": 4096, "y": -4096, "z": 4096},
                "target_position": vector_bits(5.0, 81.0, -33.5),
                "rotation": {
                    "y": float_bits(190.0),
                    "x": float_bits(-10.0),
                },
            },
            {"action": "tick", "label": "replacement-first-tick", "ticks": 1},
            {"action": "tick", "label": "replacement-final-two-ticks", "ticks": 2},
        ],
    }

    yield {
        "case_id": "client-world/entity-player-snap",
        "family": "math",
        "operation": "client_world_entity_player_snap",
        "entity_id": 37,
        "initial_position": vector_bits(1.0, 80.0, -35.5),
        "initial_rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(350.0),
        "operations": [
            {
                "action": "position_sync",
                "label": "exact-threshold-interpolates",
                "target_position": vector_bits(65.0, 80.0, -35.5),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
            },
            {"action": "tick", "label": "exact-threshold-first-tick", "ticks": 1},
            {
                "action": "position_sync",
                "label": "over-threshold-snaps",
                "target_position": vector_bits(90.0, 80.0, -35.5),
                "rotation": {
                    "y": float_bits(20.0),
                    "x": float_bits(10.0),
                },
            },
            {
                "action": "position_sync",
                "label": "exact-threshold-after-snap",
                "target_position": vector_bits(154.0, 80.0, -35.5),
                "rotation": {
                    "y": float_bits(30.0),
                    "x": float_bits(5.0),
                },
            },
            {"action": "tick", "label": "boundary-first-tick", "ticks": 1},
            {
                "action": "position_sync",
                "label": "second-over-threshold-snaps",
                "target_position": vector_bits(219.0, 80.0, -35.5),
                "rotation": {
                    "y": float_bits(40.0),
                    "x": float_bits(-5.0),
                },
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-player-motion",
        "family": "math",
        "operation": "client_world_entity_player_motion",
        "entity_id": 37,
        "initial_position": vector_bits(1.0, 80.0, -35.5),
        "initial_rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(350.0),
        "initial_motion": vector_bits(0.125, 0.25, -0.375),
        "operations": [
            {
                "action": "motion",
                "label": "packet-target-keeps-current-motion",
                "target_motion": vector_bits(0.875, -0.5, 0.625),
            },
            {"action": "tick", "label": "first-motion-tick", "ticks": 1},
            {"action": "tick", "label": "second-motion-tick", "ticks": 1},
            {
                "action": "motion",
                "label": "replacement-target-resets-three-steps",
                "target_motion": vector_bits(-0.375, 0.75, 1.125),
            },
            {"action": "tick", "label": "replacement-first-tick", "ticks": 1},
            {"action": "tick", "label": "replacement-final-two-ticks", "ticks": 2},
        ],
    }

    yield {
        "case_id": "client-world/entity-teleport",
        "family": "math",
        "operation": "client_world_entity_teleport",
        "entity_id": 37,
        "initial_position": vector_bits(1.0, 80.0, -35.5),
        "initial_rotation": {
            "y": float_bits(30.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(40.0),
        "initial_motion": vector_bits(1.0, 2.0, 3.0),
        "operations": [
            {
                "action": "teleport",
                "label": "relative-all-interpolates",
                "position": vector_bits(2.0, 3.0, 4.0),
                "delta_motion": vector_bits(0.25, -0.5, 0.75),
                "rotation": {
                    "y": float_bits(15.0),
                    "x": float_bits(10.0),
                },
                "relative_flags": 511,
            },
            {"action": "tick", "label": "relative-first-tick", "ticks": 1},
            {
                "action": "teleport",
                "label": "absolute-over-threshold-snaps",
                "position": vector_bits(100.0, 80.0, -35.5),
                "delta_motion": vector_bits(0.5, 0.25, -0.75),
                "rotation": {
                    "y": float_bits(-170.0),
                    "x": float_bits(80.0),
                },
                "relative_flags": 0,
            },
            {
                "action": "teleport",
                "label": "relative-after-snap",
                "position": vector_bits(1.0, 1.0, 1.0),
                "delta_motion": vector_bits(-0.125, 0.25, 0.5),
                "rotation": {
                    "y": float_bits(5.0),
                    "x": float_bits(-5.0),
                },
                "relative_flags": 255,
            },
            {"action": "tick", "label": "relative-after-snap-first-tick", "ticks": 1},
        ],
    }
