"""The `math` family: the `client_world/player` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.client_world.shared import *


def cases():
    yield {
        "case_id": "client-world/player-attribute-application-and-effective-value",
        "family": "math",
        "operation": "client_world_player_attributes",
        "player_id": 42,
        "operations": [
            {
                "action": "apply",
                "label": "initial-snapshot",
                "player_id": 42,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(0.25, "f64"),
                        "modifiers": [
                            attribute_modifier("p08:oracle_add", 0.1, 0),
                            attribute_modifier("p08:oracle_base", 0.2, 1),
                            attribute_modifier("p08:oracle_total", 0.5, 2),
                        ],
                    },
                    {
                        "attribute_id": 23,
                        "base": float_bits(40.0, "f64"),
                        "modifiers": [
                            attribute_modifier("p08:health_add", 5.0, 0),
                        ],
                    },
                ],
                "queries": [
                    {
                        "label": "movement-speed-effective",
                        "attribute_id": 26,
                        "min": float_bits(0.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                    {
                        "label": "max-health-effective",
                        "attribute_id": 23,
                        "min": float_bits(1.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                ],
            },
            {
                "action": "apply",
                "label": "movement-speed-replacement",
                "player_id": 42,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(0.5, "f64"),
                        "modifiers": [
                            attribute_modifier("p08:replacement", -0.1, 0),
                        ],
                    },
                ],
                "queries": [
                    {
                        "label": "replacement-clears-old-modifiers",
                        "attribute_id": 26,
                        "min": float_bits(0.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                    {
                        "label": "unmentioned-attribute-preserved",
                        "attribute_id": 23,
                        "min": float_bits(1.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                ],
            },
            {
                "action": "apply",
                "label": "wrong-player-ignored",
                "player_id": 99,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(2.0, "f64"),
                        "modifiers": [
                            attribute_modifier("p08:ignored", 1.0, 0),
                        ],
                    },
                ],
                "queries": [
                    {
                        "label": "state-after-wrong-player",
                        "attribute_id": 26,
                        "min": float_bits(0.0, "f64"),
                        "max": float_bits(1024.0, "f64"),
                    },
                ],
            },
        ],
    }

    yield {
        "case_id": "client-world/player-attribute-movement-speed-binding",
        "family": "math",
        "operation": "client_world_player_movement_speed",
        "player_id": 42,
        "operations": [
            {
                "label": "default-player-speed",
                "player_id": 42,
                "attributes": [],
                "on_ground": True,
                "block_friction": float_bits(0.6, "f32"),
            },
            {
                "label": "modified-attribute-ground-speed",
                "player_id": 42,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(0.25, "f64"),
                        "modifiers": [
                            attribute_modifier("p10:movement_add", 0.1, 0),
                            attribute_modifier("p10:movement_base", 0.2, 1),
                            attribute_modifier("p10:movement_total", 0.5, 2),
                        ],
                    },
                ],
                "on_ground": True,
                "block_friction": float_bits(0.8, "f32"),
            },
            {
                "label": "replacement-attribute-ground-speed",
                "player_id": 42,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(0.5, "f64"),
                        "modifiers": [
                            attribute_modifier("p10:movement_replacement", -0.1, 0),
                        ],
                    },
                ],
                "on_ground": True,
                "block_friction": float_bits(0.6, "f32"),
            },
            {
                "label": "wrong-player-preserves-speed",
                "player_id": 99,
                "attributes": [
                    {
                        "attribute_id": 26,
                        "base": float_bits(2.0, "f64"),
                        "modifiers": [],
                    },
                ],
                "on_ground": True,
                "block_friction": float_bits(0.6, "f32"),
            },
        ],
    }

    player_attribute_water_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "sprinting": False,
        "water_slow_down": float_bits(0.8, "f32"),
        "water_efficiency": float_bits(0.6, "f32"),
        "on_ground": True,
        "dolphins_grace": False,
        "horizontal_collision": False,
        "on_climbable": False,
        "free_after_jump": False,
        "base_gravity": float_bits(0.08, "f64"),
        "is_falling": False,
        "old_y": float_bits(10.0, "f64"),
        "current_y": float_bits(10.0, "f64"),
    }

    player_attribute_water_operations = [
        {
            "label": "default-speed-enters-water-efficiency",
            "player_id": 42,
            "attributes": [],
        },
        {
            "label": "modified-speed-enters-water-efficiency",
            "player_id": 42,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(0.25, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:water_add", 0.1, 0),
                        attribute_modifier("p10:water_base", 0.2, 1),
                        attribute_modifier("p10:water_total", 0.5, 2),
                    ],
                },
            ],
        },
        {
            "label": "replacement-speed-enters-water-efficiency",
            "player_id": 42,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(0.5, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:water_replacement", -0.1, 0),
                    ],
                },
            ],
        },
        {
            "label": "wrong-player-preserves-water-speed",
            "player_id": 99,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(2.0, "f64"),
                    "modifiers": [],
                },
            ],
        },
    ]

    yield {
        "case_id": "client-world/player-attribute-water-travel-binding",
        "family": "math",
        "operation": "client_world_player_attribute_water",
        "player_id": 42,
        "operations": [
            {**player_attribute_water_defaults, **operation}
            for operation in player_attribute_water_operations
        ],
    }

    player_sprint_attribute_operations = [
        {
            "label": "default-start-sprinting",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": True,
            "attributes": [],
        },
        {
            "label": "repeated-start-sprinting",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": True,
            "attributes": [],
        },
        {
            "label": "custom-speed-start-sprinting",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": True,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(0.25, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:sprint_custom", 0.2, 2),
                    ],
                },
            ],
        },
        {
            "label": "stop-sprinting-removes-modifier",
            "player_id": 42,
            "action": "set_sprinting",
            "sprinting": False,
            "attributes": [],
        },
        {
            "label": "wrong-player-preserves-stopped-state",
            "player_id": 99,
            "action": "set_sprinting",
            "sprinting": True,
            "attributes": [
                {
                    "attribute_id": 26,
                    "base": float_bits(2.0, "f64"),
                    "modifiers": [],
                },
            ],
        },
    ]

    yield {
        "case_id": "client-world/player-sprint-transition-attribute-binding",
        "family": "math",
        "operation": "client_world_player_sprint_attribute",
        "player_id": 42,
        "operations": player_sprint_attribute_operations,
    }
