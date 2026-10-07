"""The movement family: entities."""

from cases.movement.entities.items import cases as items
from cases.movement.entities.living import cases as living
from cases.movement.entities.pick_table import cases as pick_table
from cases.movement.entities.projectiles import cases as projectiles
from cases.movement.entities.shadows import cases as shadows


def movement_entity_cases():
    yield from items()
    yield from living()
    yield from pick_table()
    yield from projectiles()
    yield from shadows()
