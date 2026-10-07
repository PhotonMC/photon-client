"""The movement family: entities: the `projectiles` cases."""

from cases.movement.entities.common import *
from cases.movement.entities.shared import *


def cases():
    hurting = ["minecraft:fireball", "minecraft:small_fireball", "minecraft:dragon_fireball",
               "minecraft:wither_skull", "minecraft:wind_charge", "minecraft:breeze_wind_charge"]

    # The blocks around: the wall at x = 3, and a deep pool.
    lake = floor() + pool("minecraft:water", 4, -6, 6)

    projectile_rows = []

    for kind in hurting:
        name = kind.split(":")[1]
        projectile_rows.append(proj(name + "-open-air", kind, floor(), (0.5, 66.0, 0.5),
                                    (0.25, 0.0625, -0.125), 30, y_rot=-90.0, x_rot=0.0))

    projectile_rows += [
        proj("wither-skull-blue", "minecraft:wither_skull", floor(), (0.5, 66.0, 0.5),
             (0.25, 0.0625, -0.125), 30, dangerous=True),
        # Turns: straight up and down, backwards across the yaw seam, and a standstill.
        proj("fireball-up", "minecraft:fireball", floor(), (0.5, 66.0, 0.5), (0.0, 0.5, 0.0), 20),
        proj("fireball-down", "minecraft:fireball", floor(), (0.5, 80.0, 0.5), (0.0, -0.5, 0.0), 20),
        proj("fireball-across-the-seam", "minecraft:fireball", floor(), (0.5, 66.0, 0.5),
             (-0.3, 0.0, -0.01), 20, y_rot=170.0),
        proj("fireball-standing", "minecraft:fireball", floor(), (0.5, 66.0, 0.5), (0.0, 0.0, 0.0), 10),
        proj("wind-charge-standing", "minecraft:wind_charge", floor(), (0.5, 66.0, 0.5),
             (0.0, 0.0, 0.0), 10),
        # Blocks: into a wall, and past a ledge.
        proj("fireball-into-the-wall", "minecraft:fireball", floor() + wall, (0.5, 65.0, 0.5),
             (0.5, 0.0, 0.0), 20, y_rot=-90.0),
        proj("small-fireball-into-the-wall", "minecraft:small_fireball", floor() + wall,
             (0.5, 65.0, 0.5), (0.5, 0.0625, 0.03125), 20, y_rot=-90.0),
        proj("wind-charge-into-the-wall", "minecraft:wind_charge", floor() + wall, (0.5, 65.0, 0.5),
             (0.5, 0.0, 0.0), 20, y_rot=-90.0),
        proj("fireball-into-the-floor", "minecraft:fireball", floor(), (0.5, 65.0, 0.5),
             (0.25, -0.5, 0.0), 20),
        proj("fireball-under-the-ceiling", "minecraft:fireball", floor() + ceiling, (0.5, 64.5, 0.5),
             (0.0, 0.5, 0.0), 20),
        # Water: it slows (but a wind charge does not), and a stream pushes it.
        proj("fireball-into-water", "minecraft:fireball", lake, (0.5, 70.0, 0.5),
             (0.25, -0.5, 0.0), 40),
        proj("small-fireball-in-water", "minecraft:small_fireball", lake, (0.5, 65.0, 0.5),
             (0.25, 0.0, 0.0), 40),
        proj("wind-charge-in-water", "minecraft:wind_charge", lake, (0.5, 65.0, 0.5),
             (0.25, 0.0, 0.0), 40),
        proj("wither-skull-in-water", "minecraft:wither_skull", lake, (0.5, 65.0, 0.5),
             (0.25, 0.0, 0.0), 40),
        proj("fireball-in-a-stream", "minecraft:fireball", floor() + stream, (1.5, 64.5, 0.5),
             (0.0, 0.0, 0.0), 40),
        proj("wind-charge-in-a-stream", "minecraft:wind_charge", floor() + stream, (1.5, 64.5, 0.5),
             (0.0, 0.0, 0.0), 40),
        proj("fireball-in-lava", "minecraft:fireball", floor() + pool("minecraft:lava", 3),
             (0.5, 65.5, 0.5), (0.125, 0.0, 0.0), 30),
        # Entities: a target in the way, the owner it leaves, one it starts inside.
        proj("fireball-at-a-pig", "minecraft:fireball", floor(), (0.5, 64.5, 0.5),
             (0.5, 0.0, 0.0), 20, y_rot=-90.0, others=[(30, "minecraft:pig", (4.5, 64.0, 0.5))]),
        proj("fireball-leaves-its-owner", "minecraft:fireball", floor(), (0.5, 65.0, 0.5),
             (0.25, 0.0, 0.0), 30, y_rot=-90.0, owner=30,
             others=[(30, "minecraft:blaze", (0.5, 64.0, 0.5)),
                     (31, "minecraft:pig", (5.5, 64.0, 0.5))]),
        proj("fireball-from-inside-a-pig", "minecraft:fireball", floor(), (0.5, 64.5, 0.5),
             (0.25, 0.0, 0.0), 20, y_rot=-90.0, others=[(30, "minecraft:pig", (0.5, 64.0, 0.5))]),
        proj("fireball-grazes-a-pig", "minecraft:fireball", floor(), (0.5, 64.5, 0.5),
             (0.5, 0.0, 0.0), 20, y_rot=-90.0, others=[(30, "minecraft:pig", (4.5, 64.0, 0.9))]),
        # The shulker bullet drifts on the motion it was sent, and turns half the way.
        proj("shulker-bullet", "minecraft:shulker_bullet", floor(), (0.5, 66.0, 0.5),
             (0.125, 0.0625, -0.25), 20),
        proj("shulker-bullet-turning", "minecraft:shulker_bullet", floor(), (0.5, 66.0, 0.5),
             (-0.25, 0.0, 0.125), 20, y_rot=120.0, x_rot=30.0),
        proj("shulker-bullet-in-water", "minecraft:shulker_bullet", lake, (0.5, 65.0, 0.5),
             (0.125, 0.0, 0.0), 20),
        proj("shulker-bullet-in-a-stream", "minecraft:shulker_bullet", floor() + stream,
             (1.5, 64.5, 0.5), (0.0, 0.0, 0.0), 20),
        # The llama's spit: a drag and a fall, and it dies in blocks and in water.
        proj("llama-spit", "minecraft:llama_spit", floor(), (0.5, 66.0, 0.5),
             (0.5, 0.25, 0.0), 40, y_rot=-90.0),
        proj("llama-spit-into-the-floor", "minecraft:llama_spit", floor(), (0.5, 65.0, 0.5),
             (0.25, -0.5, 0.0), 20),
        proj("llama-spit-into-the-wall", "minecraft:llama_spit", floor() + wall, (0.5, 65.0, 0.5),
             (0.5, 0.0, 0.0), 20, y_rot=-90.0),
        proj("llama-spit-into-water", "minecraft:llama_spit", lake, (0.5, 70.0, 0.5),
             (0.0, -0.5, 0.0), 20),
        proj("llama-spit-in-a-stream", "minecraft:llama_spit", floor() + stream, (1.5, 64.5, 0.5),
             (0.0, 0.0, 0.0), 20),
        proj("llama-spit-past-a-pig", "minecraft:llama_spit", floor(), (0.5, 64.5, 0.5),
             (0.5, 0.0, 0.0), 20, y_rot=-90.0, others=[(30, "minecraft:pig", (4.5, 64.0, 0.5))]),
    ]

    yield {
        "case_id": "movement/projectile-ticks",
        "family": "movement",
        "operation": "projectile_ticks",
        "rows": projectile_rows,
    }
