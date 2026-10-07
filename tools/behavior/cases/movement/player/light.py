"""The movement family: the player and the world: the `light` cases."""

from cases.movement.player.common import *
from cases.movement.player.shared import *


def cases():
    ground = slab(-8, 8, -8, 8, 61, 63)

    walls = ([b for b in slab(-3, 3, -3, 3, 64, 67)
              if abs(b["x"]) == 3 or abs(b["z"]) == 3 or b["y"] == 67])

    light_rows = [
        lit("dig-to-the-void", ground,
            [put(0, 63, 0, "minecraft:air"), put(0, 62, 0, "minecraft:air"),
             put(0, 61, 0, "minecraft:air"), put(0, 61, 0, "minecraft:stone")],
            (-3, 40, -3, 7, 28, 7)),
        lit("closed-room-glowstone", ground + walls,
            [put(0, 64, 0, "minecraft:glowstone"), put(0, 64, 0, "minecraft:air"),
             put(0, 67, 0, "minecraft:air"), put(0, 67, 0, "minecraft:stone"),
             put(2, 65, 2, "minecraft:glowstone")],
            (-4, 60, -4, 9, 10, 9)),
        lit("materials-under-sky", ground,
            [put(0, 64, 0, "minecraft:torch"),
             put(2, 64, 0, "minecraft:oak_slab", {"type": "bottom"}),
             put(2, 65, 0, "minecraft:oak_slab", {"type": "top"}),
             put(4, 64, 0, "minecraft:oak_leaves", {"persistent": "true"}),
             put(4, 65, 0, "minecraft:oak_leaves", {"persistent": "true"}),
             put(-2, 64, 0, "minecraft:water"),
             put(-4, 64, 0, "minecraft:glass"),
             put(-4, 65, 0, "minecraft:stone_stairs", {"facing": "north", "half": "bottom"}),
             put(0, 66, 2, "minecraft:stone"), put(0, 64, 0, "minecraft:air")],
            (-6, 60, -3, 13, 9, 7)),
        lit("chunk-edge", slab(8, 24, -4, 4, 61, 63),
            [put(15, 63, 0, "minecraft:air"), put(16, 63, 0, "minecraft:air"),
             put(16, 62, 0, "minecraft:glowstone"), put(16, 62, 0, "minecraft:air")],
            (12, 58, -2, 8, 9, 5)),
    ]

    yield {
        "case_id": "movement/light-changes",
        "family": "movement",
        "operation": "light_changes",
        "rows": light_rows,
    }
