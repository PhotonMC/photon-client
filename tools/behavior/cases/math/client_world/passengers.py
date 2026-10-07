"""The `math` family: the `client_world/passengers` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.client_world.shared import *


def cases():
    yield {
        "case_id": "client-world/entity-passengers",
        "family": "math",
        "operation": "client_world_entity_passengers",
        "entities": [
            {"entity_id": 37, "type_id": 89},
            {"entity_id": 38, "type_id": 120},
            {"entity_id": 39, "type_id": 89},
            {"entity_id": 40, "type_id": 71},
            {"entity_id": 41, "type_id": 120},
            {"entity_id": 42, "type_id": 156},
        ],
        "initial_links": [
            {"vehicle_id": 39, "passenger_ids": [38]},
            {"vehicle_id": 37, "passenger_ids": [40]},
            {"vehicle_id": 40, "passenger_ids": [41]},
        ],
        "operations": [
            {
                "label": "replacement-reparent-duplicate-unknown",
                "vehicle_id": 37,
                "passenger_ids": [38, 39, 999, 38, 42],
            },
            {
                "label": "cycle-rejected-after-reparent",
                "vehicle_id": 39,
                "passenger_ids": [37],
            },
            {"label": "empty-replacement", "vehicle_id": 37, "passenger_ids": []},
            {
                "label": "unknown-vehicle-ignored",
                "vehicle_id": 999,
                "passenger_ids": [38],
            },
            {
                "label": "nested-relation-order",
                "vehicle_id": 40,
                "passenger_ids": [37, 42],
            },
            {
                "label": "cycle-keeps-existing-links",
                "vehicle_id": 37,
                "passenger_ids": [38, 40],
            },
            {
                "label": "nested-replacement",
                "vehicle_id": 40,
                "passenger_ids": [41],
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-passenger-position",
        "family": "math",
        "operation": "client_world_entity_passenger_position",
        "vehicle": {
            "entity_id": 37,
            "type_id": 89,
            "position": vector_bits(2.0, 2.0, 2.0),
            "rotation": {"y": float_bits(90.0), "x": float_bits(0.0)},
            "no_gravity": False,
        },
        "passengers": [
            {
                "entity_id": 38,
                "type_id": 120,
                "position": vector_bits(2.0, 12.0, 2.0),
                "movement": vector_bits(0.125, 0.0, 0.0),
                "rotation": {"y": float_bits(10.0), "x": float_bits(-15.0)},
                "no_gravity": False,
            },
            {
                "entity_id": 39,
                "type_id": 120,
                "position": vector_bits(2.0, 12.0, 2.0),
                "movement": vector_bits(-0.125, 0.0, 0.0),
                "rotation": {"y": float_bits(-20.0), "x": float_bits(25.0)},
                "no_gravity": False,
            },
        ],
        "initial_passenger_ids": [38],
        "operations": [
            {"action": "tick", "label": "single-seat", "ticks": 1},
            {
                "action": "set-passengers-and-tick",
                "label": "multi-seat",
                "passenger_ids": [38, 39],
                "ticks": 1,
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-passenger-player-position",
        "family": "math",
        "operation": "client_world_entity_passenger_player_position",
        "vehicle": {
            "entity_id": 37,
            "type_id": 89,
            "position": vector_bits(2.0, 2.0, 2.0),
            "rotation": {"y": float_bits(90.0), "x": float_bits(0.0)},
            "no_gravity": False,
        },
        "passenger": {
            "entity_id": 38,
            "type_id": 156,
            "position": vector_bits(2.0, 12.0, 2.0),
            "movement": vector_bits(0.125, 0.0, 0.0),
            "rotation": {"y": float_bits(10.0), "x": float_bits(-15.0)},
            "no_gravity": False,
        },
        "operations": [
            {"action": "tick", "label": "player-seat", "ticks": 1},
        ],
    }

    boat_family_rows = [
        ("oak-boat", 37, 89, 38),
        ("oak-chest-boat", 39, 90, 40),
        ("bamboo-raft", 41, 9, 42),
        ("bamboo-chest-raft", 43, 8, 44),
    ]

    yield {
        "case_id": "client-world/entity-passenger-boat-family-position",
        "family": "math",
        "operation": "client_world_entity_passenger_boat_family_position",
        "rows": [
            {
                "label": label,
                "vehicle": {
                    "entity_id": vehicle_id,
                    "type_id": vehicle_type_id,
                    "position": vector_bits(2.0, 10.0, 2.0),
                    "rotation": {"y": float_bits(90.0), "x": float_bits(0.0)},
                    "no_gravity": False,
                },
                "passenger": {
                    "entity_id": passenger_id,
                    "type_id": 120,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(0.125, 0.0, 0.0),
                    "rotation": {"y": float_bits(10.0), "x": float_bits(-15.0)},
                    "no_gravity": False,
                },
            }
            for label, vehicle_id, vehicle_type_id, passenger_id in boat_family_rows
        ],
        "operations": [
            {"action": "tick", "label": "family-seat", "ticks": 1},
        ],
    }

    rotation_rows = [
        ("snowball-clamp", 45, 120, 46, 170.0),
        ("player-clamp", 47, 156, 48, 140.0),
    ]

    yield {
        "case_id": "client-world/entity-passenger-rotation",
        "family": "math",
        "operation": "client_world_entity_passenger_rotation",
        "rows": [
            {
                "label": label,
                "vehicle": {
                    "entity_id": vehicle_id,
                    "type_id": 89,
                    "position": vector_bits(2.0, 2.0, 2.0),
                    "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
                "passenger": {
                    "entity_id": passenger_id,
                    "type_id": passenger_type_id,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 0.0),
                    "rotation": {"y": float_bits(passenger_yaw), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
            }
            for label, vehicle_id, passenger_type_id, passenger_id, passenger_yaw
            in rotation_rows
        ],
        "operations": [
            {"action": "tick", "label": "clamp", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-passenger-boat-turn-exemption",
        "family": "math",
        "operation": "client_world_entity_passenger_boat_turn_exemption",
        "vehicle": {
            "entity_id": 49,
            "type_id": 89,
            "position": vector_bits(2.0, 2.0, 2.0),
            "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
            "no_gravity": True,
        },
        "passenger": {
            "entity_id": 50,
            "type_id": 17,
            "position": vector_bits(2.0, 12.0, 2.0),
            "movement": vector_bits(0.0, 0.0, 0.0),
            "rotation": {"y": float_bits(170.0), "x": float_bits(0.0)},
            "no_gravity": True,
        },
        "operations": [
            {"action": "tick", "label": "breeze-exemption", "ticks": 1},
        ],
    }

    animal_rotation_rows = [
        ("even-animal", 51, 52, 53),
        ("odd-animal", 54, 55, 56),
    ]

    yield {
        "case_id": "client-world/entity-passenger-animal-rotation",
        "family": "math",
        "operation": "client_world_entity_passenger_animal_rotation",
        "rows": [
            {
                "label": label,
                "vehicle": {
                    "entity_id": vehicle_id,
                    "type_id": 89,
                    "position": vector_bits(2.0, 2.0, 2.0),
                    "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
                "animal": {
                    "entity_id": animal_id,
                    "type_id": 30,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 0.0),
                    "rotation": {"y": float_bits(170.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
                "filler": {
                    "entity_id": filler_id,
                    "type_id": 120,
                    "position": vector_bits(2.0, 12.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 0.0),
                    "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
            }
            for label, vehicle_id, animal_id, filler_id in animal_rotation_rows
        ],
        "operations": [
            {"action": "tick", "label": "animal-capacity-offset", "ticks": 1},
        ],
    }

    yield {
        "case_id": "client-world/entity-passenger-animal-position",
        "family": "math",
        "operation": "client_world_entity_passenger_animal_position",
        "vehicle": {
            "entity_id": 57,
            "type_id": 89,
            "position": vector_bits(2.0, 2.0, 2.0),
            "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
            "no_gravity": True,
        },
        "animal": {
            "entity_id": 58,
            "type_id": 30,
            "position": vector_bits(2.0, 12.0, 2.0),
            "movement": vector_bits(0.0, 0.0, 0.0),
            "rotation": {"y": float_bits(170.0), "x": float_bits(0.0)},
            "no_gravity": True,
        },
        "operations": [
            {"action": "tick", "label": "single-animal-seat", "ticks": 1},
        ],
    }

    turn_rows = [
        ("player-callback", 59, 60, 156),
        ("breeze-callback", 61, 62, 17),
    ]

    yield {
        "case_id": "client-world/entity-passenger-turn",
        "family": "math",
        "operation": "client_world_entity_passenger_turn",
        "rows": [
            {
                "label": label,
                "vehicle": {
                    "entity_id": vehicle_id,
                    "type_id": 89,
                    "position": vector_bits(2.0, 2.0, 2.0),
                    "rotation": {"y": float_bits(0.0), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
                "passenger": {
                    "entity_id": passenger_id,
                    "type_id": passenger_type_id,
                    "position": vector_bits(2.0, 2.0, 2.0),
                    "movement": vector_bits(0.0, 0.0, 0.0),
                    "rotation": {"y": float_bits(168.75), "x": float_bits(0.0)},
                    "no_gravity": True,
                },
            }
            for label, vehicle_id, passenger_id, passenger_type_id in turn_rows
        ],
        "operations": [
            {
                "action": "turn",
                "label": "direct-passenger-turn",
                "mouse_x": -100.0,
                "mouse_y": 0.0,
            },
        ],
    }
