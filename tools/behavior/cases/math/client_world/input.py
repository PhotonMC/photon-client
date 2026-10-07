"""The `math` family: the `client_world/input` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.client_world.shared import *


def cases():
    player_sneaking_input_defaults = {
        "action": "apply_input",
        "player_id": 42,
        "move_x": float_bits(0.0),
        "move_y": float_bits(1.0),
        "jump": False,
        "xxa": float_bits(0.0),
        "zza": float_bits(0.0),
        "jumping": False,
        "x_bob": float_bits(0.0),
        "y_bob": float_bits(0.0),
        "x_bob_o": float_bits(0.0),
        "y_bob_o": float_bits(0.0),
        "x_rot": float_bits(0.0),
        "y_rot": float_bits(0.0),
        "controlled_camera": True,
        "using_item": False,
        "passenger": False,
        "moving_slowly": True,
        "item_use_speed_multiplier": float_bits(0.2),
    }

    player_sneaking_input_operations = [
        {
            "label": "default-sneaking-speed",
            "attributes": [],
        },
        {
            "label": "custom-sneaking-speed",
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.5, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:sneaking_total", 0.5, 2),
                    ],
                },
            ],
        },
        {
            "label": "item-and-sneaking-attribute-order",
            "move_x": float_bits(1.0),
            "move_y": float_bits(1.0),
            "using_item": True,
            "item_use_speed_multiplier": float_bits(0.5),
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.4, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:sneaking_add", 0.1, 0),
                    ],
                },
            ],
        },
        {
            "label": "non-slow-input-ignores-sneaking-speed",
            "moving_slowly": False,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.9, "f64"),
                    "modifiers": [],
                },
            ],
        },
        {
            "label": "wrong-player-preserves-sneaking-speed",
            "player_id": 99,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.8, "f64"),
                    "modifiers": [],
                },
            ],
        },
    ]

    yield {
        "case_id": "client-world/player-sneaking-speed-input-binding",
        "family": "math",
        "operation": "client_world_player_sneaking_input",
        "player_id": 42,
        "operations": [
            {**player_sneaking_input_defaults, **operation}
            for operation in player_sneaking_input_operations
        ],
    }

    player_keyboard_input_defaults = {
        "action": "apply_keyboard_input",
        "player_id": 42,
        "up": False,
        "down": False,
        "left": False,
        "right": False,
        "jump": False,
        "shift": False,
        "sprint": False,
        "controlled_camera": True,
        "using_item": False,
        "passenger": False,
        "moving_slowly": False,
        "item_use_speed_multiplier": float_bits(0.2),
        "attributes": [],
    }

    player_keyboard_input_operations = [
        {
            "label": "forward-key-reaches-local-input",
            "up": True,
        },
        {
            "label": "diagonal-keys-use-sneaking-attribute",
            "up": True,
            "left": True,
            "jump": True,
            "moving_slowly": True,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.4, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:keyboard_sneaking_total", 0.5, 2),
                    ],
                },
            ],
        },
        {
            "label": "opposing-keys-zero-movement",
            "up": True,
            "down": True,
            "left": True,
            "right": True,
            "jump": True,
            "shift": True,
            "sprint": True,
            "moving_slowly": True,
        },
        {
            "label": "item-and-sneaking-key-order",
            "up": True,
            "right": True,
            "using_item": True,
            "item_use_speed_multiplier": float_bits(0.5),
            "moving_slowly": True,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.4, "f64"),
                    "modifiers": [
                        attribute_modifier("p10:keyboard_sneaking_add", 0.1, 0),
                    ],
                },
            ],
        },
        {
            "label": "wrong-player-preserves-key-attribute",
            "player_id": 99,
            "up": True,
            "moving_slowly": True,
            "attributes": [
                {
                    "attribute_id": 31,
                    "base": float_bits(0.9, "f64"),
                    "modifiers": [],
                },
            ],
        },
    ]

    yield {
        "case_id": "client-world/player-keyboard-input-binding",
        "family": "math",
        "operation": "client_world_player_keyboard_input",
        "player_id": 42,
        "operations": [
            {**player_keyboard_input_defaults, **operation}
            for operation in player_keyboard_input_operations
        ],
    }

    player_tick_defaults = {
        "action": "tick",
        "position": vector_bits(10.0, 64.0, -2.0),
        "rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "ticks": 1,
        "head_target": None,
        "effects": [],
    }

    player_tick_operations = [
        {
            "label": "head-target-and-effect-start",
            "head_target": float_bits(135.0),
        },
        {
            "label": "position-and-rotation-update-before-tick",
            "position": vector_bits(14.0, 68.0, 6.0),
            "rotation": {
                "y": float_bits(10.0),
                "x": float_bits(20.0),
            },
            "ticks": 0,
        },
        {
            "label": "head-target-reached-after-position-tick",
            "position": vector_bits(14.0, 68.0, 6.0),
            "rotation": {
                "y": float_bits(10.0),
                "x": float_bits(20.0),
            },
            "ticks": 2,
        },
        {
            "label": "effect-replacement-before-tick",
            "effects": [
                {
                    "effect_id": 18,
                    "amplifier": 1,
                    "duration_ticks": 2,
                    "flags": 15,
                },
            ],
        },
    ]

    yield {
        "case_id": "client-world/player-tick-state",
        "family": "math",
        "operation": "client_world_player_tick",
        "player_id": 42,
        "initial_position": vector_bits(10.0, 64.0, -2.0),
        "initial_rotation": {
            "y": float_bits(350.0),
            "x": float_bits(-20.0),
        },
        "initial_head": float_bits(350.0),
        "initial_effects": [
            {
                "effect_id": 18,
                "amplifier": 0,
                "duration_ticks": 3,
                "flags": 8,
            },
            {
                "effect_id": 19,
                "amplifier": 1,
                "duration_ticks": -1,
                "flags": 0,
            },
        ],
        "operations": [
            {**player_tick_defaults, **operation}
            for operation in player_tick_operations
        ],
    }

    def sprint_key_presses(
        forward=False, backward=False, left=False, right=False,
        jump=False, shift=False, sprint=False,
    ):
        return {
            "forward": forward,
            "backward": backward,
            "left": left,
            "right": right,
            "jump": jump,
            "shift": shift,
            "sprint": sprint,
        }

    sprint_trigger_conditions = {
        "mobility_restricted": False,
        "enough_food": True,
        "mayfly": False,
        "shallow_water": False,
        "flying": False,
        "slow_due_to_using_item": False,
        "fall_flying": False,
        "underwater": False,
        "moving_slowly": False,
    }

    sprint_trigger_sequences = [
        {
            "label": "double-tap-within-window",
            "operations": [
                {
                    "label": "first-forward-opens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "release-keeps-window",
                    "key_presses": sprint_key_presses(),
                    "move_vector": vector2_bits(0.0, 0.0),
                    "ticks": 1,
                },
                {
                    "label": "second-forward-starts-sprint",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
            ],
        },
        {
            "label": "window-expiry-and-sprint-key",
            "operations": [
                {
                    "label": "first-forward-opens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "release-expires-window",
                    "key_presses": sprint_key_presses(),
                    "move_vector": vector2_bits(0.0, 0.0),
                    "ticks": 7,
                },
                {
                    "label": "forward-after-expiry-reopens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "sprint-key-starts-immediately",
                    "key_presses": sprint_key_presses(forward=True, sprint=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
            ],
        },
        {
            "label": "shift-resets-window",
            "operations": [
                {
                    "label": "first-forward-opens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "shift-press-is-current-only",
                    "key_presses": sprint_key_presses(shift=True),
                    "move_vector": vector2_bits(0.0, 0.0),
                    "ticks": 1,
                },
                {
                    "label": "previous-shift-clears-window",
                    "key_presses": sprint_key_presses(),
                    "move_vector": vector2_bits(0.0, 0.0),
                    "ticks": 1,
                },
                {
                    "label": "forward-after-shift-opens-new-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
            ],
        },
        {
            "label": "backward-resets-window",
            "operations": [
                {
                    "label": "first-forward-opens-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
                {
                    "label": "backward-clears-window",
                    "key_presses": sprint_key_presses(backward=True),
                    "move_vector": vector2_bits(0.0, -1.0),
                    "ticks": 1,
                },
                {
                    "label": "forward-after-backward-opens-new-window",
                    "key_presses": sprint_key_presses(forward=True),
                    "move_vector": vector2_bits(0.0, 1.0),
                    "ticks": 1,
                },
            ],
        },
    ]

    yield {
        "case_id": "client-world/player-sprint-trigger-window",
        "family": "math",
        "operation": "client_world_player_sprint_trigger",
        "player_id": 42,
        "sprint_window": 7,
        "conditions": sprint_trigger_conditions,
        "sequences": sprint_trigger_sequences,
    }
