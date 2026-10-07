"""The movement family: the player and the world: the `rides` cases."""

from cases.movement.player.common import *
from cases.movement.player.shared import *


def cases():
    lake = [{"x": x, "y": 64, "z": z, "block": "minecraft:water"}
            for x in range(-12, 13) for z in range(-12, 13)]

    deep_lake = lake + [{"x": x, "y": 65, "z": z, "block": "minecraft:water"}
                        for x in range(-12, 13) for z in range(-12, 13)]

    shore = [{"x": x, "y": 64, "z": z, "block": "minecraft:water"}
             for x in range(-12, 13) for z in range(-12, 0)]

    ride_rows = [
        ride("boat-water-idle", floor() + lake, keys(30), boat=(0.5, 64.2, 0.5)),
        ride("boat-water-forward", floor() + lake, keys(40, forward=True), boat=(0.5, 64.2, 0.5)),
        ride("boat-water-backward", floor() + lake, keys(30, backward=True), boat=(0.5, 64.2, 0.5)),
        ride("boat-water-turn-left", floor() + lake, keys(30, left=True), boat=(0.5, 64.2, 0.5), y_rot=30.0),
        ride("boat-water-turn-right-forward", floor() + lake, keys(40, right=True, forward=True),
             boat=(0.5, 64.2, 0.5), y_rot=-45.0),
        ride("boat-water-both-turns", floor() + lake, keys(20, left=True, right=True),
             boat=(0.5, 64.2, 0.5)),
        ride("boat-water-input-changes", floor() + lake,
             keys(8, forward=True) + keys(8, forward=True, left=True) + keys(8) + keys(8, backward=True),
             boat=(0.5, 64.2, 0.5)),
        ride("boat-water-motion", floor() + lake, keys(30), boat=(0.5, 64.2, 0.5), motion=(0.2, 0.0, -0.1)),
        ride("boat-drops-into-water", floor() + lake, keys(40), boat=(0.5, 68.0, 0.5)),
        ride("boat-drops-onto-land", floor(), keys(30), boat=(0.5, 66.0, 0.5)),
        ride("boat-land-forward", floor(), keys(40, forward=True), boat=(0.5, 64.0, 0.5)),
        ride("boat-land-turn", floor(), keys(30, right=True, forward=True), boat=(0.5, 64.0, 0.5)),
        ride("boat-land-ice", floor(block="minecraft:ice"), keys(40, forward=True), boat=(0.5, 64.0, 0.5)),
        ride("boat-land-blue-ice", floor(block="minecraft:blue_ice"), keys(40, forward=True) + keys(10),
             boat=(0.5, 64.0, 0.5)),
        ride("boat-land-slime", floor(block="minecraft:slime_block"), keys(30, forward=True),
             boat=(0.5, 64.0, 0.5)),
        ride("boat-land-honey", floor(block="minecraft:honey_block"), keys(20, forward=True),
             boat=(0.5, 64.0, 0.5)),
        ride("boat-land-soul-sand", floor(block="minecraft:soul_sand"), keys(20, forward=True),
             boat=(0.5, 64.0, 0.5)),
        ride("boat-land-lily-pad", floor() + [{"x": 0, "y": 64, "z": 0, "block": "minecraft:lily_pad"}],
             keys(10), boat=(0.5, 64.0, 0.5)),
        ride("boat-land-fence", floor() + [{"x": x, "y": 64, "z": 3, "block": "minecraft:oak_fence"}
                                           for x in range(-3, 4)], keys(30, forward=True),
             boat=(0.5, 64.0, 0.5)),
        ride("boat-land-stair", floor() + [{"x": x, "y": 64, "z": z, "block": "minecraft:stone_stairs",
                                            "properties": {"facing": "north"}}
                                           for x in range(-2, 3) for z in range(2, 4)],
             keys(30, forward=True), boat=(0.5, 64.0, 0.5)),
        ride("boat-drops-onto-slime", floor(block="minecraft:slime_block"), keys(60),
             boat=(0.5, 70.0, 0.5)),
        ride("boat-in-cobweb", floor() + [{"x": 0, "y": y, "z": 0, "block": "minecraft:cobweb"}
                                          for y in (64, 65)], keys(30), boat=(0.5, 68.0, 0.5)),
        ride("boat-in-powder-snow", floor() + [{"x": 0, "y": 64, "z": 0, "block": "minecraft:powder_snow"}],
             keys(30), boat=(0.5, 66.0, 0.5)),
        ride("boat-into-wall", floor() + lake + wall, keys(40, forward=True), boat=(0.5, 64.2, 0.5)),
        ride("boat-shore", floor() + shore, keys(50, forward=True), boat=(0.5, 64.2, -6.5), y_rot=0.0),
        ride("boat-under-water", floor() + deep_lake, keys(30), boat=(0.5, 64.0, 0.5)),
        ride("boat-surface", floor() + deep_lake, keys(30), boat=(0.5, 65.5, 0.5)),
        ride("boat-surface-forward", floor() + deep_lake, keys(40, forward=True) + keys(20, left=True, forward=True),
             boat=(0.5, 65.5, 0.5), y_rot=20.0),
        ride("boat-drops-into-deep-water", floor() + deep_lake, keys(50), boat=(0.5, 72.0, 0.5)),
        ride("boat-sprint-sneak-jump", floor() + deep_lake,
             keys(8, forward=True, sprint=True) + keys(8, sneak=True) + keys(8, jump=True)
             + keys(8, forward=True, sneak=True, jump=True), boat=(0.5, 65.5, 0.5)),
        ride("boat-turn-then-keep-going", floor() + deep_lake,
             keys(30, forward=True, left=True) + keys(30, forward=True), boat=(0.5, 65.5, 0.5)),
        ride("boat-under-flowing-water", floor() + stream, keys(30), boat=(0.5, 64.1, -0.5)),
        ride("boat-bubble-column", floor() + bubbles, keys(40), boat=(0.5, 64.0, 0.5)),
        ride("boat-chest", floor() + lake, keys(20, forward=True), boat=(0.5, 64.2, 0.5),
             kind="minecraft:oak_chest_boat"),
        ride("raft", floor() + lake, keys(20, forward=True), boat=(0.5, 64.2, 0.5),
             kind="minecraft:bamboo_raft"),
    ]

    yield {
        "case_id": "movement/ride-boat",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": ride_rows,
    }

    step = [{"x": x, "y": 64, "z": 3, "block": "minecraft:stone"} for x in range(-3, 4)]

    fence = [{"x": x, "y": 64, "z": 3, "block": "minecraft:oak_fence"} for x in range(-3, 4)]

    stairs = [{"x": x, "y": 64, "z": z, "block": "minecraft:stone_stairs", "properties": {"facing": "north"}}
              for x in range(-2, 3) for z in (3, 4)]

    ladder_wall = column(0, 3, 64, 68)

    horse_rows = [
        horse("horse-idle", floor(), keys(30)),
        horse("horse-forward", floor(), keys(50, forward=True)),
        horse("horse-backward", floor(), keys(30, backward=True)),
        horse("horse-strafe", floor(), keys(30, left=True) + keys(30, right=True)),
        horse("horse-diagonal", floor(), keys(40, forward=True, left=True)),
        horse("horse-sprint-key", floor(), keys(30, forward=True, sprint=True)),
        horse("horse-sneak", floor(), keys(20, forward=True, sneak=True)),
        horse("horse-turn", floor(), turned(keys(40, forward=True), y_rot=-90.0)),
        horse("horse-look", floor(), looking(keys(40, forward=True), (0.0, 0.0), (35.0, 40.0), (-80.0, -60.0))),
        horse("horse-small-jump", floor(), keys(10) + keys(3, jump=True) + keys(30)),
        horse("horse-full-jump", floor(), keys(10) + keys(15, jump=True) + keys(40)),
        horse("horse-jump-forward", floor(), keys(8, forward=True) + keys(10, forward=True, jump=True)
              + keys(40, forward=True)),
        horse("horse-two-jumps", floor(), keys(10) + keys(5, jump=True) + keys(25) + keys(8, jump=True) + keys(30)),
        horse("horse-step-up", floor() + step, keys(30, forward=True)),
        horse("horse-fence", floor() + fence, keys(30, forward=True)),
        horse("horse-stairs", floor() + stairs, keys(30, forward=True)),
        horse("horse-wall", floor() + ladder_wall, keys(30, forward=True)),
        horse("horse-water", floor() + lake, keys(40, forward=True), start=(0.5, 64.2, -8.5)),
        horse("horse-deep-water", floor() + deep_lake, keys(40, forward=True) + keys(20), start=(0.5, 65.0, 0.5)),
        horse("horse-swim-jump", floor() + deep_lake, keys(10) + keys(10, jump=True) + keys(30), start=(0.5, 65.0, 0.5)),
        horse("horse-ice", floor(block="minecraft:ice"), keys(30, forward=True) + keys(30)),
        horse("horse-slime", floor(block="minecraft:slime_block"), keys(10) + keys(10, jump=True) + keys(50)),
        horse("horse-honey", floor(block="minecraft:honey_block"), keys(30, forward=True)),
        horse("horse-soul-sand", floor(block="minecraft:soul_sand"), keys(30, forward=True)),
        horse("horse-cobweb", floor() + [{"x": 0, "y": 64, "z": z, "block": "minecraft:cobweb"} for z in (1, 2, 3)],
              keys(30, forward=True)),
        horse("horse-falls", floor(), keys(40), start=(0.5, 75.0, 0.5)),
        horse("horse-bubbles", floor() + bubbles, keys(40)),
        horse("horse-ladder", floor() + column(0, 2, 64, 70, "minecraft:ladder", {"facing": "north"}) + ladder_wall[:0],
              keys(30, forward=True)),
        horse("donkey", floor(), keys(30, forward=True), kind="minecraft:donkey"),
        horse("mule-jump", floor(), keys(5) + keys(10, jump=True) + keys(30), kind="minecraft:mule"),
        horse("skeleton-horse", floor(), keys(30, forward=True), kind="minecraft:skeleton_horse"),
        horse("zombie-horse", floor(), keys(30, forward=True), kind="minecraft:zombie_horse"),
    ]

    horse_rows.append(horse("horse-unsaddled", floor(), keys(30, forward=True) + keys(10, jump=True)))

    horse_rows[-1]["ride"]["unsaddled"] = True

    yield {
        "case_id": "movement/ride-horse",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": horse_rows,
    }

    camel_rows = [
        camel("camel-idle", floor(), keys(30)),
        camel("camel-forward", floor(), keys(50, forward=True)),
        camel("camel-sprint", floor(), keys(10, forward=True) + keys(40, forward=True, sprint=True)),
        camel("camel-strafe-back", floor(), keys(20, left=True) + keys(20, backward=True)),
        camel("camel-look", floor(), looking(keys(50, forward=True), (0.0, 0.0), (50.0, 30.0), (-80.0, -40.0))),
        camel("camel-dash", floor(), keys(5, forward=True) + keys(12, forward=True, jump=True) + keys(70, forward=True)),
        camel("camel-small-dash", floor(), keys(5, forward=True) + keys(3, jump=True) + keys(60, forward=True)),
        camel("camel-dash-twice", floor(), keys(12, jump=True) + keys(80) + keys(12, jump=True) + keys(30)),
        camel("camel-dash-looking-up", floor(), looking(keys(5) + keys(12, jump=True) + keys(50), (0.0, -90.0))),
        camel("camel-dash-sprint", floor(), keys(5, forward=True, sprint=True) + keys(10, forward=True, sprint=True, jump=True)
              + keys(60, forward=True, sprint=True)),
        camel("camel-sitting", floor(), keys(30), sit_age=100),
        camel("camel-sitting-forward", floor(), keys(60, forward=True), sit_age=100),
        camel("camel-sitting-dash", floor(), keys(10, jump=True) + keys(30), sit_age=100),
        camel("camel-sitting-down", floor(), keys(60, forward=True), sit_age=10),
        camel("camel-standing-up", floor(), keys(60, forward=True), sit_age=-30 + 52 - 20 + 0),
        camel("camel-step", floor() + [{"x": x, "y": 64, "z": 3, "block": "minecraft:stone"} for x in range(-3, 4)],
              keys(40, forward=True)),
        camel("camel-fence", floor() + [{"x": x, "y": 64, "z": 3, "block": "minecraft:oak_fence"} for x in range(-3, 4)],
              keys(40, forward=True)),
        camel("camel-water", floor() + lake, keys(40, forward=True), start=(0.5, 64.2, -8.5)),
        camel("camel-deep-water", floor() + deep_lake, keys(40, forward=True) + keys(20), start=(0.5, 65.0, 0.5)),
        camel("camel-sitting-in-water", floor() + lake, keys(30), start=(0.5, 64.2, 0.5), sit_age=100),
        camel("camel-ice", floor(block="minecraft:ice"), keys(30, forward=True, sprint=True) + keys(30)),
        camel("camel-slime-dash", floor(block="minecraft:slime_block"), keys(10, jump=True) + keys(60)),
        camel("camel-falls", floor(), keys(40), start=(0.5, 75.0, 0.5)),
        camel("camel-husk", floor(), keys(30, forward=True) + keys(10, jump=True) + keys(40), kind="minecraft:camel_husk"),
    ]

    yield {
        "case_id": "movement/ride-camel",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": camel_rows,
    }

    pig_rows = [
        pig("pig-straight", floor(), keys(50)),
        pig("pig-keys-ignored", floor(), keys(20, backward=True) + keys(20, left=True) + keys(10, jump=True)),
        pig("pig-steer", floor(), looking(keys(60), (0.0, 0.0), (60.0, 20.0), (-120.0, -40.0))),
        pig("pig-turn", floor(), turned(keys(40), y_rot=-90.0)),
        pig("pig-no-stick", floor(), keys(30, forward=True), stick=None),
        pig("pig-other-item", floor(), keys(30), stick="minecraft:stick"),
        pig("pig-unsaddled", floor(), keys(30)),
        pig("pig-slab", floor() + [{"x": x, "y": 64, "z": 3, "block": "minecraft:stone_slab"} for x in range(-3, 4)], keys(30)),
        pig("pig-wall", floor() + ladder_wall, keys(30)),
        pig("pig-ladder", floor() + column(0, 2, 64, 70, "minecraft:ladder", {"facing": "north"}), keys(30)),
        pig("pig-water", floor() + lake, keys(40), start=(0.5, 64.2, -8.5)),
        pig("pig-deep-water", floor() + deep_lake, keys(40) + keys(20), start=(0.5, 65.0, 0.5)),
        pig("pig-ice", floor(block="minecraft:ice"), keys(40)),
        pig("pig-honey", floor(block="minecraft:honey_block"), keys(30)),
        pig("pig-cobweb", floor() + [{"x": 0, "y": 64, "z": z, "block": "minecraft:cobweb"} for z in (1, 2, 3)], keys(30)),
        pig("pig-falls", floor(), keys(30), start=(0.5, 75.0, 0.5)),
    ]

    yield {
        "case_id": "movement/ride-pig",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": pig_rows,
    }

    strider_rows = [
        strider("strider-cold-land", floor(), keys(40)),
        strider("strider-no-stick", floor(), keys(20), stick=None),
        strider("strider-unsaddled", floor(), keys(20), saddled=False),
        strider("strider-steer", floor(), looking(keys(60), (0.0, 0.0), (70.0, 15.0), (-110.0, -30.0))),
        strider("strider-wall", floor() + ladder_wall, keys(30)),
        strider("strider-step", floor() + [{"x": x, "y": 64, "z": 3, "block": "minecraft:stone_slab"}
                                          for x in range(-3, 4)], keys(30)),
        strider("strider-lava-walk", floor() + lava_pool(), keys(50), start=(0.5, 64.5, -3.5)),
        strider("strider-onto-lava", floor() + lava_pool(), keys(60), start=(0.5, 64.0, -9.5)),
        strider("strider-lava-deep", floor() + lava_pool(3), keys(50), start=(0.5, 66.6, -3.5)),
        strider("strider-sinks-in", floor() + lava_pool(2), keys(40), start=(0.5, 64.2, -3.5)),
        strider("strider-falls-into-lava", floor() + lava_pool(), keys(40), start=(0.5, 75.0, 0.5)),
        strider("strider-lava-roof", floor() + lava_pool(2) + [{"x": x, "y": 67, "z": z, "block": "minecraft:stone"}
                                                         for x in range(-5, 6) for z in range(-5, 6)],
                keys(40), start=(0.5, 65.0, -3.5)),
        strider("strider-lava-edge", floor() + lava_pool(1, 0, 5), keys(60), start=(-3.5, 64.0, 2.5), y_rot=-90.0),
        strider("strider-warm-floor", floor(block="minecraft:magma_block"), keys(40)),
        strider("strider-water", floor() + lake, keys(40), start=(0.5, 64.2, -8.5)),
        strider("strider-ice", floor(block="minecraft:ice"), keys(40)),
        strider("strider-cobweb", floor() + [{"x": 0, "y": 64, "z": z, "block": "minecraft:cobweb"} for z in (1, 2, 3)],
                keys(30)),
    ]

    yield {
        "case_id": "movement/ride-strider",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": strider_rows,
    }

    # Riding a minecart, which the server moves: the rider sits at its seat and keeps its own
    # keys to itself, with the minecart standing, rolled to a heading or off the ground.
    minecart_rows = [
        ride("minecart-idle", floor(), keys(30), kind="minecraft:minecart"),
        ride("minecart-keys", floor(), keys(20, forward=True, left=True) + keys(10, jump=True, sneak=True),
             kind="minecraft:minecart"),
        ride("minecart-look", floor(), looking(keys(40), (0.0, 0.0), (90.0, 20.0), (-45.0, -60.0)),
             kind="minecraft:minecart", y_rot=70.0),
        ride("minecart-chest", floor(), keys(20), kind="minecraft:chest_minecart", y_rot=-30.0),
        ride("minecart-motion", floor(), keys(30), kind="minecraft:minecart", motion=(0.2, 0.0, 0.1)),
        ride("minecart-air", floor(), keys(40), kind="minecraft:minecart", boat=(0.5, 70.0, 0.5)),
    ]

    yield {
        "case_id": "movement/ride-minecart",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": minecart_rows,
    }
