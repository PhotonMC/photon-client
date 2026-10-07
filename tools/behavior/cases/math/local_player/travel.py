"""The `math` family: the `local_player/travel` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    local_player_travel_flying_cases = [
        {
            "label": "air-branch-rotates-and-drags",
        },
        {
            "label": "water-branch-uses-water-speed",
            "in_water": True,
            "water_speed": float_bits(0.04, "f32"),
        },
        {
            "label": "lava-branch-uses-lava-speed",
            "in_lava": True,
            "lava_speed": float_bits(0.06, "f32"),
        },
        {
            "label": "water-wins-over-lava",
            "in_water": True,
            "in_lava": True,
            "water_speed": float_bits(0.04, "f32"),
            "lava_speed": float_bits(0.06, "f32"),
        },
        {
            "label": "long-input-normalizes-before-speed",
            "input": vector_bits(1.0, 2.0, 2.0),
            "delta": vector_bits(-0.25, 0.125, 0.5),
            "y_rot": float_bits(-120.0, "f32"),
            "air_speed": float_bits(0.07, "f32"),
        },
        {
            "label": "subthreshold-input-skips-relative-movement",
            "input": vector_bits(0.0001, 0.0, 0.0),
            "delta": vector_bits(0.1, -0.2, 0.3),
        },
        {
            "label": "zero-input-preserves-signed-motion",
            "input": vector_bits(-0.0, 0.0, -0.0),
            "delta": vector_bits(-0.0, 0.125, -0.25),
            "y_rot": float_bits(90.0, "f32"),
        },
        {
            "label": "nonfinite-relative-delta-is-rejected",
            "input": {"x": "7ff8000000000001", "y": float_bits(0.0, "f64"),
                      "z": float_bits(0.0, "f64")},
            "delta": vector_bits(0.1, -0.2, 0.3),
        },
    ]

    local_player_travel_flying_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "in_water": False,
        "in_lava": False,
        "water_speed": float_bits(0.02, "f32"),
        "lava_speed": float_bits(0.03, "f32"),
        "air_speed": float_bits(0.04, "f32"),
    }

    local_player_travel_flying_inputs = []

    for values in local_player_travel_flying_cases:
        input_values = {**local_player_travel_flying_defaults, **values}
        local_player_travel_flying_inputs.append(input_values)

    yield {"case_id": "player-movement/travel-flying", "family": "math",
           "operation": "local_player_travel_flying",
           "inputs": local_player_travel_flying_inputs}

    local_player_travel_in_water_cases = [
        {
            "label": "plain-water-applies-drag-and-fluid-gravity",
        },
        {
            "label": "sprinting-uses-sprint-water-slowdown-and-bypasses-gravity",
            "sprinting": True,
        },
        {
            "label": "ground-water-efficiency-blends-speed-and-drag",
            "water_efficiency": float_bits(0.6, "f32"),
            "speed": float_bits(0.12, "f32"),
        },
        {
            "label": "airborne-water-efficiency-is-halved",
            "water_efficiency": float_bits(0.6, "f32"),
            "speed": float_bits(0.12, "f32"),
            "on_ground": False,
        },
        {
            "label": "dolphins-grace-overrides-computed-drag",
            "water_efficiency": float_bits(0.6, "f32"),
            "speed": float_bits(0.12, "f32"),
            "sprinting": True,
            "dolphins_grace": True,
        },
        {
            "label": "horizontal-climbable-sets-fluid-step",
            "horizontal_collision": True,
            "on_climbable": True,
            "base_gravity": float_bits(0.08, "f64"),
            "is_falling": True,
            "free_after_jump": False,
        },
        {
            "label": "free-horizontal-fluid-collision-jumps-out",
            "horizontal_collision": True,
            "base_gravity": float_bits(0.08, "f64"),
            "is_falling": True,
            "free_after_jump": True,
        },
        {
            "label": "negative-water-efficiency-is-ignored",
            "water_efficiency": float_bits(-0.5, "f32"),
            "speed": float_bits(0.12, "f32"),
        },
    ]

    local_player_travel_in_water_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "sprinting": False,
        "water_slow_down": float_bits(0.8, "f32"),
        "water_efficiency": float_bits(0.0, "f32"),
        "on_ground": True,
        "speed": float_bits(0.1, "f32"),
        "dolphins_grace": False,
        "horizontal_collision": False,
        "on_climbable": False,
        "free_after_jump": False,
        "base_gravity": float_bits(0.08, "f64"),
        "is_falling": False,
        "old_y": float_bits(10.0, "f64"),
        "current_y": float_bits(10.0, "f64"),
    }

    local_player_travel_in_water_inputs = []

    for values in local_player_travel_in_water_cases:
        input_values = {**local_player_travel_in_water_defaults, **values}
        local_player_travel_in_water_inputs.append(input_values)

    yield {"case_id": "player-movement/travel-in-water", "family": "math",
           "operation": "local_player_travel_in_water",
           "inputs": local_player_travel_in_water_inputs}

    local_player_travel_in_lava_cases = [
        {
            "label": "shallow-lava-applies-fluid-drag-falling-and-gravity",
        },
        {
            "label": "deep-lava-uses-half-scale-before-gravity",
            "shallow_lava": False,
        },
        {
            "label": "zero-gravity-skips-lava-gravity-step",
            "base_gravity": float_bits(0.0, "f64"),
        },
        {
            "label": "sprinting-bypasses-falling-adjustment-but-keeps-gravity",
            "sprinting": True,
        },
        {
            "label": "horizontal-fluid-collision-without-free-space-keeps-motion",
            "horizontal_collision": True,
            "free_after_jump": False,
        },
        {
            "label": "horizontal-fluid-collision-with-free-space-jumps-out",
            "horizontal_collision": True,
            "free_after_jump": True,
        },
    ]

    local_player_travel_in_lava_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "sprinting": False,
        "shallow_lava": True,
        "horizontal_collision": False,
        "free_after_jump": False,
        "base_gravity": float_bits(0.08, "f64"),
        "is_falling": False,
        "old_y": float_bits(10.0, "f64"),
        "current_y": float_bits(10.0, "f64"),
    }

    local_player_travel_in_lava_inputs = []

    for values in local_player_travel_in_lava_cases:
        input_values = {**local_player_travel_in_lava_defaults, **values}
        local_player_travel_in_lava_inputs.append(input_values)

    yield {"case_id": "player-movement/travel-in-lava", "family": "math",
           "operation": "local_player_travel_in_lava",
           "inputs": local_player_travel_in_lava_inputs}

    local_player_travel_in_air_cases = [
        {
            "label": "air-travel-applies-server-gravity-and-drag",
        },
        {
            "label": "ground-travel-uses-below-block-friction",
            "on_ground": True,
            "below_block": "ice",
            "block_friction": float_bits(0.98, "f32"),
            "speed": float_bits(0.12, "f32"),
        },
        {
            "label": "levitation-replaces-gravity-with-effect-target",
            "levitation_amplifier": 2,
        },
        {
            "label": "discard-friction-keeps-relative-horizontal-motion",
            "should_discard_friction": True,
            "air_drag_modifier": float_bits(0.2, "f32"),
        },
        {
            "label": "omnidirectional-air-mover-uses-horizontal-drag-vertically",
            "omnidirectional_air_mover": True,
            "air_drag_modifier": float_bits(0.2, "f32"),
        },
        {
            "label": "air-drag-attribute-modifies-horizontal-and-vertical",
            "air_drag_modifier": float_bits(0.2, "f32"),
        },
        {
            "label": "client-unloaded-above-min-y-uses-fallback-vertical-speed",
            "client_side": True,
            "has_chunk_at": False,
        },
        {
            "label": "client-unloaded-at-min-y-zeroes-vertical-speed",
            "client_side": True,
            "has_chunk_at": False,
            "current_y": float_bits(-64.0, "f64"),
        },
    ]

    local_player_travel_in_air_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "on_ground": False,
        "below_block": "air",
        "block_friction": float_bits(0.6, "f32"),
        "friction_modifier": float_bits(1.0, "f32"),
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
        "levitation_amplifier": -1,
        "gravity": float_bits(0.08, "f64"),
        "should_discard_friction": False,
        "air_drag_modifier": float_bits(0.0, "f32"),
        "omnidirectional_air_mover": False,
        "client_side": False,
        "has_chunk_at": True,
        "min_y": -64,
        "current_y": float_bits(10.0, "f64"),
    }

    local_player_travel_in_air_inputs = []

    for values in local_player_travel_in_air_cases:
        input_values = {**local_player_travel_in_air_defaults, **values}
        local_player_travel_in_air_inputs.append(input_values)

    yield {"case_id": "player-movement/travel-in-air", "family": "math",
           "operation": "local_player_travel_in_air",
           "inputs": local_player_travel_in_air_inputs}

    local_player_update_fall_flying_cases = [
        {
            "label": "neutral-glide-applies-lift-and-drag",
        },
        {
            "label": "nose-down-converts-descending-speed",
            "look": vector_bits(0.0, -0.5, 0.8660254037844386),
            "x_rot": float_bits(30.0, "f32"),
            "delta": vector_bits(0.2, -0.4, 0.3),
        },
        {
            "label": "nose-up-adds-lift",
            "look": vector_bits(0.0, 0.5, 0.8660254037844386),
            "x_rot": float_bits(-30.0, "f32"),
            "delta": vector_bits(0.3, 0.1, 0.2),
        },
        {
            "label": "vertical-look-skips-horizontal-conversion",
            "look": vector_bits(0.0, 1.0, 0.0),
            "x_rot": float_bits(15.0, "f32"),
        },
        {
            "label": "zero-gravity-keeps-lift-free-motion",
            "gravity": float_bits(0.0, "f64"),
            "look": vector_bits(-0.6, 0.0, 0.8),
            "x_rot": float_bits(0.0, "f32"),
        },
        {
            "label": "negative-look-direction-aligns-horizontal-motion",
            "look": vector_bits(-0.6, 0.0, 0.8),
            "x_rot": float_bits(0.0, "f32"),
            "delta": vector_bits(0.35, -0.1, -0.25),
        },
    ]

    local_player_update_fall_flying_defaults = {
        "look": vector_bits(0.0, 0.0, 1.0),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "x_rot": float_bits(0.0, "f32"),
        "gravity": float_bits(0.08, "f64"),
    }

    local_player_update_fall_flying_inputs = []

    for values in local_player_update_fall_flying_cases:
        input_values = {**local_player_update_fall_flying_defaults, **values}
        local_player_update_fall_flying_inputs.append(input_values)

    yield {"case_id": "player-movement/update-fall-flying-movement", "family": "math",
           "operation": "local_player_update_fall_flying_movement",
           "inputs": local_player_update_fall_flying_inputs}

    local_player_travel_dispatch_cases = [
        {
            "label": "air-is-default-when-no-fluid-or-glide",
        },
        {
            "label": "water-fluid-branch",
            "in_water": True,
        },
        {
            "label": "lava-fluid-branch",
            "in_lava": True,
            "shallow_lava": False,
        },
        {
            "label": "water-wins-over-lava-in-fluid-branch",
            "in_water": True,
            "in_lava": True,
            "water_efficiency": float_bits(0.4, "f32"),
        },
        {
            "label": "unaffected-water-falls-through-to-air",
            "in_water": True,
            "affected_by_fluids": False,
        },
        {
            "label": "standing-on-lava-falls-through-to-air",
            "in_lava": True,
            "can_stand_on_fluid": True,
        },
        {
            "label": "fall-flying-uses-glide-branch",
            "fall_flying": True,
            "look": vector_bits(0.0, -0.35, 0.9367496997597597),
            "x_rot": float_bits(20.0, "f32"),
        },
        {
            "label": "fall-flying-on-climbable-falls-back-to-air",
            "fall_flying": True,
            "on_climbable": True,
            "on_ground": True,
        },
        {
            "label": "fluid-precedes-fall-flying",
            "in_water": True,
            "fall_flying": True,
        },
    ]

    local_player_travel_dispatch_defaults = {
        "input": vector_bits(0.3, -0.4, 0.8),
        "delta": vector_bits(0.1, -0.2, 0.3),
        "y_rot": float_bits(37.5, "f32"),
        "x_rot": float_bits(0.0, "f32"),
        "look": vector_bits(0.0, 0.0, 1.0),
        "in_water": False,
        "in_lava": False,
        "affected_by_fluids": True,
        "can_stand_on_fluid": False,
        "fall_flying": False,
        "on_climbable": False,
        "sprinting": False,
        "water_slow_down": float_bits(0.8, "f32"),
        "water_efficiency": float_bits(0.0, "f32"),
        "on_ground": False,
        "block_friction": float_bits(0.6, "f32"),
        "speed": float_bits(0.1, "f32"),
        "dolphins_grace": False,
        "horizontal_collision": False,
        "free_after_jump": False,
        "shallow_lava": True,
        "flying_speed": float_bits(0.05, "f32"),
        "flying": False,
        "passenger": False,
        "scaffolding": False,
        "suppress_sliding_down_ladder": False,
        "jumping": False,
        "levitation_amplifier": -1,
        "gravity": float_bits(0.08, "f64"),
        "client_side": True,
        "has_chunk_at": True,
        "current_y": float_bits(10.0, "f64"),
        "min_y": -64,
        "should_discard_friction": False,
        "friction_modifier": float_bits(1.0, "f32"),
        "air_drag_modifier": float_bits(0.0, "f32"),
        "omnidirectional_air_mover": False,
        "below_block": "air",
    }

    local_player_travel_dispatch_inputs = []

    for values in local_player_travel_dispatch_cases:
        input_values = {**local_player_travel_dispatch_defaults, **values}
        local_player_travel_dispatch_inputs.append(input_values)

    yield {"case_id": "player-movement/travel-dispatch", "family": "math",
           "operation": "local_player_travel_dispatch",
           "inputs": local_player_travel_dispatch_inputs}
