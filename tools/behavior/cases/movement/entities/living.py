"""The movement family: entities: the `living` cases."""

from cases.movement.entities.common import *
from cases.movement.entities.shared import *


def cases():
    from cases.movement.entities.animation import animation_rows_parts
    living_rows = [
        living("quiet", "pig", 6, []),
        living("damage", "pig", 16, [{"tick": 2, "type": "damage"}]),
        living("hurt-animation", "cow", 16, [{"tick": 2, "type": "hurt"}]),
        living("hurt-twice", "pig", 24, [{"tick": 1, "type": "damage"}, {"tick": 6, "type": "hurt"}]),
        living("death", "pig", 30, [{"tick": 2, "type": "death"}]),
        living("damage-then-death", "zombie", 36,
               [{"tick": 2, "type": "damage"}, {"tick": 3, "type": "death"}]),
        living("death-then-damage", "cow", 30,
               [{"tick": 2, "type": "death"}, {"tick": 5, "type": "damage"}]),
        living("damage-while-walking", "pig", 20,
               [{"tick": 1, "type": "move", "xa": 800, "ya": 0, "za": 0},
                {"tick": 3, "type": "damage"},
                {"tick": 4, "type": "move", "xa": 800, "ya": 0, "za": 0}]),
        living("spider-dies", "spider", 30, [{"tick": 1, "type": "death"}]),
        living("death-with-the-player-far-off", "pig", 30, [{"tick": 2, "type": "death"}], player_x=200.5),
        living("death-at-the-edge-of-simulation", "pig", 30, [{"tick": 2, "type": "death"}], player_x=136.5),
        living("death-just-past-simulation", "pig", 30, [{"tick": 2, "type": "death"}], player_x=144.5),
    ]

    yield {
        "case_id": "movement/living-entity-ticks",
        "family": "movement",
        "operation": "living_entity_ticks",
        "rows": living_rows,
    }

    animation_rows = animation_rows_parts()

    yield {
        "case_id": "movement/living-entity-animations",
        "family": "movement",
        "operation": "living_entity_ticks",
        "rows": animation_rows,
    }
