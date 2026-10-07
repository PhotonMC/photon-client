"""The movement family: entities: what several `all` cases share."""

from cases.movement.entities.common import *


"""A dropped item ticked in a scene (`ItemEntity.tick` on the client):
what a broken block's drop does — falls, lands, bounces and slides —
through floors of different friction and speed factor, walls, ceilings,
an edge between two floors, shaped blocks, and the rest an item on the
ground takes (it moves only one tick in four)."""

wall = [{"x": 3, "y": y, "z": z, "block": "minecraft:stone"}
        for y in range(64, 67) for z in range(-8, 9)]

ceiling = [{"x": x, "y": 66, "z": z, "block": "minecraft:stone"}
           for x in range(-8, 9) for z in range(-8, 9)]

# A stream: a source with the water flowing away over a floor, each block
# one level lower than the one before.
stream = [level(0, 64, 0, 0)] + [level(x, 64, 0, x) for x in range(1, 8)]
