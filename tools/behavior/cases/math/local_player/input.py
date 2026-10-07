"""The `math` family: the `local_player/input` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    local_player_input_cases = [
        {
            "label": "zero-preserves-signed-zero",
            "x": "80000000", "y": "00000000",
        },
        {
            "label": "axis-applies-base-factor",
            "x": float_bits(0.0), "y": float_bits(1.0),
        },
        {
            "label": "diagonal-clamps-to-unit-square",
            "x": float_bits(1.0), "y": float_bits(1.0),
        },
        {
            "label": "short-vector-keeps-scaled-length",
            "x": float_bits(0.25), "y": float_bits(-0.5),
        },
        {
            "label": "item-use-multiplier",
            "x": float_bits(1.0), "y": float_bits(0.0),
            "using_item": True, "item_use_speed_multiplier": float_bits(0.2),
        },
        {
            "label": "item-use-bypassed-in-passenger",
            "x": float_bits(1.0), "y": float_bits(0.0),
            "using_item": True, "passenger": True,
            "item_use_speed_multiplier": float_bits(0.2),
        },
        {
            "label": "sneaking-attribute-factor",
            "x": float_bits(0.0), "y": float_bits(1.0),
            "moving_slowly": True, "sneaking_movement_factor": float_bits(0.3),
        },
        {
            "label": "item-and-sneaking-order",
            "x": float_bits(1.0), "y": float_bits(1.0),
            "using_item": True, "item_use_speed_multiplier": float_bits(0.5),
            "moving_slowly": True, "sneaking_movement_factor": float_bits(0.3),
        },
        {
            "label": "zero-item-factor-reaches-square-zero",
            "x": float_bits(0.25), "y": float_bits(-0.5),
            "using_item": True, "item_use_speed_multiplier": float_bits(0.0),
        },
        {
            "label": "non-unit-vector-clamps",
            "x": float_bits(2.0), "y": float_bits(1.0),
        },
    ]

    local_player_input_defaults = {
        "using_item": False, "passenger": False, "moving_slowly": False,
        "item_use_speed_multiplier": float_bits(0.2),
        "sneaking_movement_factor": float_bits(0.3),
    }

    local_player_input_inputs = []

    for values in local_player_input_cases:
        input_values = {**local_player_input_defaults, **values}
        local_player_input_inputs.append(input_values)

    yield {"case_id": "player-input/modify", "family": "math",
           "operation": "local_player_input", "inputs": local_player_input_inputs}

    local_player_apply_input_cases = [
        {
            "label": "controlled-axis-updates-impulses-and-bob",
            "move_x": float_bits(0.0), "move_y": float_bits(1.0),
            "jump": False, "xxa": float_bits(0.25), "zza": float_bits(-0.5),
            "jumping": True, "x_bob": float_bits(1.0), "y_bob": float_bits(2.0),
            "x_bob_o": float_bits(3.0), "y_bob_o": float_bits(4.0),
            "x_rot": float_bits(10.0), "y_rot": float_bits(20.0),
            "controlled_camera": True,
        },
        {
            "label": "controlled-diagonal-sets-jumping",
            "move_x": float_bits(0.6), "move_y": float_bits(0.8),
            "jump": True, "xxa": float_bits(-1.0), "zza": float_bits(1.0),
            "jumping": False, "x_bob": float_bits(-2.0), "y_bob": float_bits(4.0),
            "x_bob_o": float_bits(-3.0), "y_bob_o": float_bits(5.0),
            "x_rot": float_bits(-30.0), "y_rot": float_bits(60.0),
            "controlled_camera": True,
        },
        {
            "label": "controlled-zero-preserves-signed-zero",
            "move_x": "80000000", "move_y": "00000000",
            "jump": True, "xxa": float_bits(1.0), "zza": float_bits(-1.0),
            "jumping": False, "x_bob": "80000000", "y_bob": "00000000",
            "x_bob_o": "80000000", "y_bob_o": "00000000",
            "x_rot": float_bits(0.0), "y_rot": float_bits(-0.0),
            "controlled_camera": True,
        },
        {
            "label": "uncontrolled-camera-uses-base-input",
            "move_x": float_bits(0.0), "move_y": float_bits(1.0),
            "jump": True, "xxa": float_bits(0.5), "zza": float_bits(-0.25),
            "jumping": True, "x_bob": float_bits(1.0), "y_bob": float_bits(2.0),
            "x_bob_o": float_bits(3.0), "y_bob_o": float_bits(4.0),
            "x_rot": float_bits(10.0), "y_rot": float_bits(20.0),
            "controlled_camera": False,
        },
        {
            "label": "controlled-overwrites-existing-impulses",
            "move_x": float_bits(-1.0), "move_y": float_bits(0.0),
            "jump": False, "xxa": float_bits(2.0), "zza": float_bits(3.0),
            "jumping": True, "x_bob": float_bits(0.0), "y_bob": float_bits(0.0),
            "x_bob_o": float_bits(1.0), "y_bob_o": float_bits(1.0),
            "x_rot": float_bits(0.0), "y_rot": float_bits(0.0),
            "controlled_camera": True,
        },
    ]

    local_player_apply_input_defaults = {
        "move_x": float_bits(0.0), "move_y": float_bits(0.0),
        "jump": False, "xxa": float_bits(0.0), "zza": float_bits(0.0),
        "jumping": False, "x_bob": float_bits(0.0), "y_bob": float_bits(0.0),
        "x_bob_o": float_bits(0.0), "y_bob_o": float_bits(0.0),
        "x_rot": float_bits(0.0), "y_rot": float_bits(0.0),
        "controlled_camera": True,
    }

    local_player_apply_input_inputs = []

    for values in local_player_apply_input_cases:
        input_values = {**local_player_apply_input_defaults, **values}
        local_player_apply_input_inputs.append(input_values)

    yield {"case_id": "player-input/apply", "family": "math",
           "operation": "local_player_apply_input", "inputs": local_player_apply_input_inputs}

    local_player_sprint_gate_cases = [
        {
            "label": "eligible-ground-forward",
            "move_y": float_bits(1.0),
        },
        {
            "label": "already-sprinting-blocks-start",
            "sprinting": True,
        },
        {
            "label": "forward-threshold-rejects-zero",
            "move_y": float_bits(0.0),
        },
        {
            "label": "forward-threshold-rejects-small",
            "move_y": float_bits(0.00001),
        },
        {
            "label": "mobility-restricted-blocks-start",
            "mobility_restricted": True,
        },
        {
            "label": "insufficient-food-blocks-start",
            "enough_food": False,
        },
        {
            "label": "mayfly-allows-insufficient-food",
            "enough_food": False,
            "mayfly": True,
        },
        {
            "label": "shallow-water-blocks-ground-start",
            "shallow_water": True,
        },
        {
            "label": "flying-allows-shallow-water",
            "shallow_water": True,
            "flying": True,
        },
        {
            "label": "item-use-blocks-sprint",
            "using_item": True,
        },
        {
            "label": "fall-flying-blocks-out-of-water",
            "fall_flying": True,
        },
        {
            "label": "fall-flying-allows-underwater",
            "fall_flying": True,
            "underwater": True,
        },
        {
            "label": "moving-slowly-blocks-out-of-water",
            "moving_slowly": True,
        },
        {
            "label": "moving-slowly-allows-underwater",
            "moving_slowly": True,
            "underwater": True,
        },
    ]

    local_player_sprint_gate_defaults = {
        "move_y": float_bits(1.0),
        "sprinting": False,
        "mobility_restricted": False,
        "enough_food": True,
        "mayfly": False,
        "shallow_water": False,
        "flying": False,
        "using_item": False,
        "fall_flying": False,
        "underwater": False,
        "moving_slowly": False,
    }

    local_player_sprint_gate_inputs = []

    for values in local_player_sprint_gate_cases:
        input_values = {**local_player_sprint_gate_defaults, **values}
        local_player_sprint_gate_inputs.append(input_values)

    yield {"case_id": "player-sprint/can-start", "family": "math",
           "operation": "local_player_can_start_sprinting", "inputs": local_player_sprint_gate_inputs}

    local_player_fluid_adjustment_cases = [
        {
            "label": "zero-gravity-preserves-movement",
            "base_gravity": float_bits(0.0, "f64"),
            "is_falling": True,
            "x": float_bits(-1.25, "f64"),
            "y": float_bits(-0.005, "f64"),
            "z": float_bits(2.5, "f64"),
        },
        {
            "label": "sprinting-bypasses-gravity",
            "sprinting": True,
            "y": float_bits(-0.2, "f64"),
        },
        {
            "label": "falling-near-custom-gravity-uses-clamp",
            "base_gravity": float_bits(0.16, "f64"),
            "is_falling": True,
            "y": float_bits(0.01, "f64"),
        },
        {
            "label": "falling-outside-custom-gravity-window-uses-subtraction",
            "base_gravity": float_bits(0.16, "f64"),
            "is_falling": True,
            "y": float_bits(0.014, "f64"),
        },
        {
            "label": "rising-uses-gravity-subtraction",
            "is_falling": False,
            "y": float_bits(0.3, "f64"),
        },
        {
            "label": "default-gravity-exact-zero",
            "is_falling": True,
            "y": float_bits(0.005, "f64"),
        },
        {
            "label": "negative-zero-gravity-preserves-signed-zero",
            "base_gravity": "8000000000000000",
            "is_falling": True,
            "x": "8000000000000000",
            "y": "8000000000000000",
            "z": "0000000000000000",
        },
        {
            "label": "falling-custom-gravity-ordinary-subtraction",
            "base_gravity": float_bits(0.16, "f64"),
            "is_falling": True,
            "y": float_bits(-0.2, "f64"),
        },
    ]

    local_player_fluid_adjustment_defaults = {
        "base_gravity": float_bits(0.08, "f64"),
        "is_falling": False,
        "sprinting": False,
        "x": float_bits(0.125, "f64"),
        "y": float_bits(-0.05, "f64"),
        "z": float_bits(-0.25, "f64"),
    }

    local_player_fluid_adjustment_inputs = []

    for values in local_player_fluid_adjustment_cases:
        input_values = {**local_player_fluid_adjustment_defaults, **values}
        local_player_fluid_adjustment_inputs.append(input_values)

    yield {"case_id": "player-movement/fluid-falling-adjustment", "family": "math",
           "operation": "local_player_fluid_falling_adjustment",
           "inputs": local_player_fluid_adjustment_inputs}

    local_player_climbable_movement_cases = [
        {
            "label": "not-climbable-preserves-movement",
            "on_climbable": False,
        },
        {
            "label": "climbable-clamps-horizontal-and-downward",
            "on_climbable": True,
        },
        {
            "label": "suppressing-player-stops-ladder-slide",
            "on_climbable": True,
            "suppress_sliding_down_ladder": True,
            "x": float_bits(0.1, "f64"),
            "y": float_bits(-0.1, "f64"),
            "z": float_bits(-0.1, "f64"),
        },
        {
            "label": "scaffolding-keeps-clamped-downward-motion",
            "on_climbable": True,
            "scaffolding": True,
            "suppress_sliding_down_ladder": True,
        },
        {
            "label": "suppressing-player-keeps-upward-motion",
            "on_climbable": True,
            "suppress_sliding_down_ladder": True,
            "x": float_bits(-0.1, "f64"),
            "y": float_bits(0.25, "f64"),
            "z": float_bits(0.1, "f64"),
        },
        {
            "label": "climbable-preserves-signed-zero",
            "on_climbable": True,
            "x": "8000000000000000",
            "y": "8000000000000000",
            "z": "0000000000000000",
        },
    ]

    local_player_climbable_movement_defaults = {
        "on_climbable": True,
        "scaffolding": False,
        "suppress_sliding_down_ladder": False,
        "x": float_bits(0.4, "f64"),
        "y": float_bits(-0.4, "f64"),
        "z": float_bits(-0.4, "f64"),
    }

    local_player_climbable_movement_inputs = []

    for values in local_player_climbable_movement_cases:
        input_values = {**local_player_climbable_movement_defaults, **values}
        local_player_climbable_movement_inputs.append(input_values)

    yield {"case_id": "player-movement/climbable-movement", "family": "math",
           "operation": "local_player_climbable_movement",
           "inputs": local_player_climbable_movement_inputs}
