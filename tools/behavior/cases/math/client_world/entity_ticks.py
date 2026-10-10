"""The `math` family: the `client_world/entity_ticks` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.client_world.shared import *


def cases():
    yield {
        "case_id": "client-world/entity-snowball-tick",
        "family": "math",
        "operation": "client_world_entity_snowball_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 1.0, 2.0),
                "movement": vector_bits(0.25, 0.5, 0.25),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(-20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 1.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(355.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-arrow-tick",
        "family": "math",
        "operation": "client_world_entity_arrow_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 1.0, 2.0),
                "movement": vector_bits(0.25, 0.5, 0.25),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(-20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 1.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(355.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-experience-orb-tick",
        "family": "math",
        "operation": "client_world_entity_experience_orb_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 10.0, 2.0),
                "movement": vector_bits(0.125, 0.25, -0.125),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 10.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(45.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-client-tick", "ticks": 1},
            {"action": "tick", "label": "second-client-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-primed-tnt-tick",
        "family": "math",
        "operation": "client_world_entity_primed_tnt_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 10.0, 2.0),
                "movement": vector_bits(0.125, 0.25, -0.125),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 10.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(45.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-falling-block-tick",
        "family": "math",
        "operation": "client_world_entity_falling_block_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 10.0, 2.0),
                "movement": vector_bits(0.125, 0.25, -0.125),
                "rotation": {
                    "y": float_bits(10.0),
                    "x": float_bits(20.0),
                },
                "no_gravity": False,
            },
            {
                "entity_id": 38,
                "position": vector_bits(4.0, 10.0, 2.0),
                "movement": vector_bits(0.0, 0.0, 1.0),
                "rotation": {
                    "y": float_bits(45.0),
                    "x": float_bits(30.0),
                },
                "no_gravity": True,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-firework-tick",
        "family": "math",
        "operation": "client_world_entity_firework_tick",
        "entities": [
            {
                "entity_id": 37,
                "position": vector_bits(2.0, 10.0, 2.0),
                "movement": vector_bits(0.25, 0.5, 0.125),
                "rotation": {
                    "y": float_bits(355.0),
                    "x": float_bits(-20.0),
                },
                "no_gravity": False,
            },
        ],
        "operations": [
            {"action": "tick", "label": "first-air-tick", "ticks": 1},
            {"action": "tick", "label": "second-air-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-boat-air-tick",
        "family": "math",
        "operation": "client_world_entity_boat_air_tick",
        "player_id": 38,
        "boat": {
            "entity_id": 37,
            "position": vector_bits(2.0, 10.0, 2.0),
            "movement": vector_bits(0.25, 0.5, -0.75),
            "rotation": {
                "y": float_bits(10.0),
                "x": float_bits(-20.0),
            },
            "no_gravity": False,
        },
        "input": {
            "left": False,
            "right": True,
            "forward": True,
            "backward": False,
        },
        "operations": [
            {"action": "tick", "label": "air-gravity-control", "ticks": 1},
            {"action": "set_no_gravity", "label": "enable-no-gravity", "value": True},
            {"action": "tick", "label": "air-no-gravity-control", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-boat-remote-tick",
        "family": "math",
        "operation": "client_world_entity_boat_remote_tick",
        "passenger": {
            "entity_id": 38,
            "type_id": 151,
        },
        "boat": {
            "entity_id": 37,
            "position": vector_bits(2.0, 12.0, 2.0),
            "movement": vector_bits(0.25, 0.5, -0.75),
            "rotation": {
                "y": float_bits(10.0),
                "x": float_bits(-20.0),
            },
            "no_gravity": False,
        },
        "input": {
            "left": False,
            "right": True,
            "forward": True,
            "backward": False,
        },
        "operations": [
            {"action": "tick", "label": "remote-first-tick", "ticks": 1},
            {"action": "tick", "label": "remote-second-tick", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-ticking-eligibility",
        "family": "math",
        "operation": "client_world_entity_ticking_eligibility",
        "player": {
            "entity_id": 42,
            "position": vector_bits(10.0, 64.0, -2.0),
        },
        "remote": {
            "entity_id": 37,
            "type_id": 71,
            "position": vector_bits(1.25, 80.0, -35.5),
        },
        "operations": [
            {"action": "tick", "label": "loaded-first-tick", "ticks": 1},
            {
                "action": "unload_chunk",
                "label": "remote-chunk-unloaded",
                "chunk_x": 0,
                "chunk_z": -3,
            },
            {"action": "tick", "label": "unloaded-tick", "ticks": 1},
            {
                "action": "load_chunk",
                "label": "remote-chunk-reloaded",
                "chunk_x": 0,
                "chunk_z": -3,
            },
            {"action": "tick", "label": "reloaded-tick", "ticks": 1},
        ],
    }
