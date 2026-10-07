"""The movement family: entities: the `pick_table` cases."""

from cases.movement.entities.common import *
from cases.movement.entities.shared import *


def cases():
    # What the crosshair needs of every entity type, as a new entity of it
    # answers: whether it can be aimed at, how much its box grows for the aim,
    # and its size (a baby's too, for the types that have babies).
    yield {
        "case_id": "movement/entity-pick-table",
        "family": "movement",
        "operation": "entity_pick_table",
        "rows": [{"label": "all", "blocks": [], "chunk_radius": 0}],
    }
