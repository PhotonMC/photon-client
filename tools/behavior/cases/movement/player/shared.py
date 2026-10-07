"""The movement family: the player and the world: what several `all` cases share."""

from cases.movement.player.common import *


"""The reference's own client player, ticked through scenes.

Each row is a scene, a start pose and the keys held on every tick. The
expected output is everything the tick leaves behind — position, motion,
the collision flags, the pose — and every packet the player asked to
send, as the bytes the packet's own codec writes. This is the whole
client tick, not its parts: input, `aiStep`, `travel`, `move` and the
position packets, in the order the reference runs them."""

wall = [b for x in range(-3, 4) for b in column(x, 4, 64, 66)]

stream = ([{"x": x, "y": 64, "z": -3, "block": "minecraft:water"} for x in range(-1, 2)]
          + [{"x": x, "y": 64, "z": z, "block": "minecraft:water",
              "properties": {"level": str(z + 3)}}
             for x in range(-1, 2) for z in range(-2, 4)])

bubbles = ([{"x": 0, "y": 63, "z": 0, "block": "minecraft:soul_sand"}]
           + column(0, 0, 64, 70, "minecraft:bubble_column", {"drag": "false"})
           + [{"x": 1, "y": 63, "z": 0, "block": "minecraft:magma_block"}]
           + column(1, 0, 64, 70, "minecraft:bubble_column", {"drag": "true"})
           + [b for x in range(-1, 3) for z in (-1, 1) for b in column(x, z, 64, 70)]
           + column(-1, 0, 64, 70) + column(2, 0, 64, 70))
