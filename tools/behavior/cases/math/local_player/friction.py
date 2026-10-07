"""The `math` family: the `local_player/friction` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    local_player_friction_influenced_speed_cases = [
        {
            "label": "ground-at-friction-threshold-keeps-speed",
            "on_ground": True,
            "block_friction": float_bits(0.6, "f32"),
        },
        {
            "label": "ground-below-friction-threshold-keeps-speed",
            "on_ground": True,
            "block_friction": float_bits(0.5, "f32"),
        },
        {
            "label": "ground-above-friction-threshold-scales-speed",
            "on_ground": True,
            "block_friction": float_bits(0.8, "f32"),
        },
        {
            "label": "ground-just-above-friction-threshold-scales-speed",
            "on_ground": True,
            "block_friction": float_bits(0.6000001, "f32"),
            "speed": float_bits(0.125, "f32"),
        },
        {
            "label": "air-without-controlling-player-uses-default-flying-speed",
            "on_ground": False,
            "flying": False,
            "sprinting": False,
            "passenger": False,
        },
        {
            "label": "air-sprinting-without-flight-uses-sprint-speed",
            "on_ground": False,
            "flying": False,
            "sprinting": True,
            "passenger": False,
        },
        {
            "label": "air-flying-uses-ability-speed",
            "on_ground": False,
            "flying": True,
            "flying_speed": float_bits(0.07, "f32"),
            "sprinting": False,
            "passenger": False,
        },
        {
            "label": "air-flying-sprint-doubles-ability-speed",
            "on_ground": False,
            "flying": True,
            "flying_speed": float_bits(0.07, "f32"),
            "sprinting": True,
            "passenger": False,
        },
        {
            "label": "air-flying-passenger-uses-grounded-fallback",
            "on_ground": False,
            "flying": True,
            "sprinting": True,
            "passenger": True,
        },
        {
            "label": "ground-scaled-speed-preserves-negative-zero",
            "on_ground": True,
            "block_friction": float_bits(0.8, "f32"),
            "speed": "80000000",
        },
        {
            "label": "nan-friction-fails-strict-comparison-and-keeps-speed",
            "on_ground": True,
            "block_friction": "7fc00001",
            "speed": float_bits(0.25, "f32"),
        },
    ]

    local_player_friction_influenced_speed_defaults = {
        "on_ground": True,
        "block_friction": float_bits(0.6, "f32"),
        "speed": float_bits(0.1, "f32"),
        "flying": False,
        "flying_speed": float_bits(0.05, "f32"),
        "sprinting": False,
        "passenger": False,
    }

    local_player_friction_influenced_speed_inputs = []

    for values in local_player_friction_influenced_speed_cases:
        input_values = {**local_player_friction_influenced_speed_defaults, **values}
        local_player_friction_influenced_speed_inputs.append(input_values)

    yield {"case_id": "player-movement/friction-influenced-speed", "family": "math",
           "operation": "local_player_friction_influenced_speed",
           "inputs": local_player_friction_influenced_speed_inputs}

    local_player_modified_friction_cases = [
        {
            "label": "unit-modifier-keeps-block-friction",
            "friction": float_bits(0.8, "f32"),
            "modifier": float_bits(1.0, "f32"),
        },
        {
            "label": "zero-modifier-reaches-one",
            "friction": float_bits(0.8, "f32"),
            "modifier": float_bits(0.0, "f32"),
        },
        {
            "label": "double-modifier-reduces-friction",
            "friction": float_bits(0.8, "f32"),
            "modifier": float_bits(2.0, "f32"),
        },
        {
            "label": "negative-modifier-clamps-high",
            "friction": float_bits(0.8, "f32"),
            "modifier": float_bits(-1.0, "f32"),
        },
        {
            "label": "large-modifier-clamps-low",
            "friction": float_bits(0.2, "f32"),
            "modifier": float_bits(2.0, "f32"),
        },
        {
            "label": "zero-friction-with-unit-modifier",
            "friction": float_bits(0.0, "f32"),
            "modifier": float_bits(1.0, "f32"),
        },
        {
            "label": "full-friction-ignores-modifier",
            "friction": float_bits(1.0, "f32"),
            "modifier": float_bits(3.0, "f32"),
        },
        {
            "label": "nan-friction-propagates-through-clamp",
            "friction": "7fc01234",
            "modifier": float_bits(1.0, "f32"),
        },
    ]

    local_player_modified_friction_defaults = {
        "friction": float_bits(0.6, "f32"),
        "modifier": float_bits(1.0, "f32"),
    }

    local_player_modified_friction_inputs = []

    for values in local_player_modified_friction_cases:
        input_values = {**local_player_modified_friction_defaults, **values}
        local_player_modified_friction_inputs.append(input_values)

    yield {"case_id": "player-movement/modified-friction", "family": "math",
           "operation": "local_player_modified_friction",
           "inputs": local_player_modified_friction_inputs}

    local_player_handle_relative_friction_cases = [
        {
            "label": "air-relative-movement-uses-player-flying-speed",
        },
        {
            "label": "ground-friction-speed-enters-relative-movement",
            "on_ground": True,
            "block_friction": float_bits(0.8, "f32"),
            "speed": float_bits(0.12, "f32"),
        },
        {
            "label": "climbable-clamps-delta-before-return",
            "input": vector_bits(0.0, 0.0, 0.0),
            "delta": vector_bits(0.4, -0.4, -0.4),
            "on_climbable": True,
        },
        {
            "label": "horizontal-climbable-collision-sets-step-height",
            "on_climbable": True,
            "horizontal_collision": True,
        },
        {
            "label": "jumping-climbable-sets-step-height",
            "on_climbable": True,
            "jumping": True,
        },
        {
            "label": "jumping-without-climbable-keeps-motion",
            "jumping": True,
        },
    ]

    local_player_handle_relative_friction_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "on_ground": False,
        "block_friction": float_bits(0.6, "f32"),
        "speed": float_bits(0.1, "f32"),
        "flying_speed": float_bits(0.05, "f32"),
        "flying": False,
        "sprinting": False,
        "passenger": False,
        "on_climbable": False,
        "scaffolding": False,
        "suppress_sliding_down_ladder": False,
        "horizontal_collision": False,
        "jumping": False,
    }

    local_player_handle_relative_friction_inputs = []

    for values in local_player_handle_relative_friction_cases:
        input_values = {**local_player_handle_relative_friction_defaults, **values}
        local_player_handle_relative_friction_inputs.append(input_values)

    yield {"case_id": "player-movement/handle-relative-friction", "family": "math",
           "operation": "local_player_handle_relative_friction",
           "inputs": local_player_handle_relative_friction_inputs}
