"""The movement family: the player and the world: the `ticks` cases."""

from cases.movement.player.common import *
from cases.movement.player.shared import *


def cases():
    slab_step = [{"x": x, "y": 64, "z": z, "block": "minecraft:stone_slab"}
                 for x in range(-3, 4) for z in range(3, 8)]

    block_step = [b for x in range(-3, 4) for z in range(3, 8) for b in column(x, z, 64, 64)]

    ledge = [{"x": x, "y": 63, "z": z, "block": "minecraft:stone"}
             for x in range(-3, 4) for z in range(-3, 2)]

    ladder = column(0, 1, 64, 70, "minecraft:ladder", {"facing": "north"})

    ladder_wall = column(0, 2, 64, 70)

    pool = [{"x": x, "y": y, "z": z, "block": "minecraft:water"}
            for x in range(-6, 7) for z in range(-6, 7) for y in range(64, 67)]

    ceiling = [{"x": x, "y": 65, "z": z, "block": "minecraft:stone"}
               for x in range(-4, 5) for z in range(-4, 9)]

    low_roof = [{"x": x, "y": 66, "z": z, "block": "minecraft:stone"}
                for x in range(-2, 3) for z in range(-2, 3)]

    honey_wall = column(1, 0, 64, 72, "minecraft:honey_block")

    lava = [{"x": x, "y": 64, "z": z, "block": "minecraft:lava"}
            for x in range(-3, 4) for z in range(-3, 8)]

    trapdoor_ladder = (column(0, 1, 64, 65, "minecraft:ladder", {"facing": "north"})
                       + [{"x": 0, "y": 66, "z": 1, "block": "minecraft:oak_trapdoor",
                           "properties": {"facing": "north", "open": "true"}}])

    side_wall = [b for x in range(-16, 4) for b in column(x, 1, 64, 65)]

    # A waterfall two wide against a wall, and a stream flowing past fences.
    waterfall = ([{"x": x, "y": y, "z": 0, "block": "minecraft:water", "properties": {"level": "8"}}
                  for x in range(0, 2) for y in range(64, 72)]
                 + [b for x in range(-1, 3) for b in column(x, 1, 64, 72)])

    fenced_stream = stream + [{"x": 2, "y": 64, "z": z, "block": "minecraft:oak_fence"}
                              for z in range(-2, 4)]

    edge_ladder = column(0, 2, 56, 62, "minecraft:ladder", {"facing": "north"}) + column(0, 3, 56, 63)

    plant_names = ["minecraft:short_grass", "minecraft:fern", "minecraft:dandelion",
                   "minecraft:sweet_berry_bush", "minecraft:oak_sapling"]

    plants = [{"x": x, "y": 64, "z": z, "block": plant_names[(x + z) % len(plant_names)]}
              for x in range(-2, 3) for z in range(-2, 12)]

    plants += [{"x": x, "y": y, "z": 12, "block": "minecraft:tall_grass",
                "properties": {"half": "lower" if y == 64 else "upper"}}
               for x in range(-2, 3) for y in (64, 65)]

    # Blocks that hold a player back from inside them.
    powder_pit = [{"x": x, "y": y, "z": z, "block": "minecraft:powder_snow"}
                  for x in range(-3, 4) for z in range(-3, 8) for y in range(61, 64)]

    powder_ahead = [{"x": x, "y": 64, "z": z, "block": "minecraft:powder_snow"}
                    for x in range(-3, 4) for z in range(2, 5)]

    webs = [{"x": x, "y": y, "z": 2, "block": "minecraft:cobweb"}
            for x in range(-1, 2) for y in range(64, 66)]

    berries = [{"x": x, "y": 64, "z": z, "block": "minecraft:sweet_berry_bush",
                "properties": {"age": "1"}} for x in range(-1, 2) for z in range(2, 4)]

    rows = [
        scene("stand-still", floor(), keys(6)),
        scene("walk-forward", floor(), keys(20, forward=True)),
        scene("walk-backward", floor(), keys(12, backward=True)),
        scene("strafe-left", floor(), keys(12, left=True)),
        scene("walk-diagonal", floor(), keys(12, forward=True, right=True)),
        scene("walk-turned-45", floor(), keys(12, forward=True), y_rot=45.0),
        scene("walk-then-turn", floor(),
              keys(6, forward=True) + turned(keys(8, forward=True), y_rot=90.0)),
        scene("walk-then-stop", floor(), keys(8, forward=True) + keys(10)),
        scene("sprint", floor(), keys(20, forward=True, sprint=True)),
        scene("sprint-then-release", floor(),
              keys(10, forward=True, sprint=True) + keys(6, forward=True)),
        scene("sneak-walk", floor(), keys(14, forward=True, sneak=True)),
        scene("jump-once", floor(), keys(1, jump=True) + keys(14)),
        scene("jump-held", floor(), keys(30, jump=True)),
        scene("walk-jump", floor(), keys(20, forward=True, jump=True)),
        scene("sprint-jump", floor(), keys(24, forward=True, sprint=True, jump=True)),
        scene("walk-into-wall", floor() + wall, keys(20, forward=True)),
        scene("step-onto-slab", floor() + slab_step, keys(16, forward=True)),
        scene("walk-into-block-step", floor() + block_step, keys(16, forward=True)),
        scene("jump-onto-block", floor() + block_step,
              keys(4, forward=True) + keys(10, forward=True, jump=True)),
        scene("fall-onto-floor", floor(), keys(30), start=(0.5, 72.0, 0.5)),
        scene("walk-off-ledge", ledge, keys(20, forward=True)),
        scene("sneak-at-ledge", ledge, keys(24, forward=True, sneak=True)),
        scene("walk-on-ice", floor(block="minecraft:ice"), keys(10, forward=True) + keys(12)),
        scene("walk-on-soul-sand", floor(block="minecraft:soul_sand"), keys(16, forward=True)),
        scene("land-on-slime", floor(block="minecraft:slime_block"), keys(40),
              start=(0.5, 72.0, 0.5)),
        scene("walk-on-honey", floor(block="minecraft:honey_block"), keys(16, forward=True)),
        scene("climb-ladder", floor() + ladder + ladder_wall,
              keys(30, forward=True), y_rot=0.0),
        scene("swim", floor(block="minecraft:stone") + pool, keys(20, forward=True),
              start=(0.5, 65.0, 0.5)),
        scene("swim-up", floor(block="minecraft:stone") + pool, keys(20, jump=True),
              start=(0.5, 64.2, 0.5)),
        scene("auto-jump", floor() + block_step, keys(20, forward=True), auto_jump=True),
        scene("look-down-walk", floor(), turned(keys(10, forward=True), x_rot=90.0)),
        scene("creative-double-jump-fly", floor(),
              keys(1, jump=True) + keys(2) + keys(1, jump=True) + keys(10, jump=True)
              + keys(10, forward=True), game_mode="creative"),
        scene("spectator-float", floor(), keys(10, forward=True, jump=True),
              game_mode="spectator"),
        scene("sprint-swim", floor() + pool, keys(24, forward=True, sprint=True)),
        scene("sneak-sink", floor() + pool, keys(16, sneak=True), start=(0.5, 65.5, 0.5)),
        scene("fall-into-water", floor() + pool, keys(40), start=(0.5, 90.0, 0.5)),
        scene("fall-far", floor(), keys(40), start=(0.5, 95.0, 0.5)),
        scene("crawl-under-ceiling", floor() + ceiling, keys(16, forward=True)),
        scene("jump-under-ceiling", floor() + low_roof, keys(14, jump=True)),
        scene("honey-wall-slide", floor() + honey_wall,
              keys(30, forward=True), start=(0.5, 72.0, 0.5), y_rot=-90.0),
        scene("flowing-water-push", floor() + stream, keys(20)),
        scene("waterfall-swim", floor() + waterfall,
              keys(10, forward=True) + keys(20, forward=True, jump=True),
              start=(0.5, 66.0, 0.5)),
        scene("stream-past-fence", floor() + fenced_stream, keys(20, left=True)),
        scene("lava-wade", floor() + lava, keys(16, forward=True)),
        scene("trapdoor-ladder", floor() + trapdoor_ladder + ladder_wall,
              keys(40, forward=True)),
        scene("sprint-along-wall", floor() + side_wall,
              keys(16, forward=True, sprint=True), y_rot=85.0),
        scene("sneak-onto-slime", floor(block="minecraft:slime_block"),
              keys(24, sneak=True), start=(0.5, 68.0, 0.5)),
        scene("creative-fly-sprint-land", floor(),
              keys(1, jump=True) + keys(1) + keys(1, jump=True)
              + keys(8, forward=True, sprint=True) + keys(10, sneak=True) + keys(4),
              game_mode="creative"),
        scene("auto-jump-sprint", floor() + block_step,
              keys(20, forward=True, sprint=True), auto_jump=True),
        scene("walk-off-ledge-into-ladder", ledge + edge_ladder, keys(30, forward=True)),
        scene("sink-in-powder-snow", [b for b in floor() if b["x"] < -3 or b["x"] > 3
                                      or b["z"] < -3 or b["z"] > 7] + powder_pit + floor(y=60),
              keys(10) + keys(20, forward=True) + keys(20, forward=True, jump=True)),
        scene("walk-into-powder-snow", floor() + powder_ahead, keys(30, forward=True)),
        scene("walk-through-cobweb", floor() + webs,
              keys(30, forward=True) + keys(6, forward=True, jump=True)),
        scene("walk-through-berry-bush", floor() + berries, keys(30, forward=True)),
        scene("sneak-in-plants", floor() + plants,
              keys(4, forward=True) + keys(10, forward=True, sneak=True) + keys(10, forward=True)
              + keys(6, sneak=True) + keys(6)),
        scene("hurt-and-die", floor(), [dict(tick, **extra) for tick, extra in zip(keys(9), [
            {"damage": "minecraft:generic"}, {"damage": "minecraft:drown"},
            {"damage": "minecraft:on_fire"}, {"damage": "minecraft:sweet_berry_bush"},
            {"damage": "minecraft:freeze"}, {"damage": "minecraft:thorns"},
            {"damage": "minecraft:lava"}, {"death": True}, {}])]),
        scene("ride-bubble-column", floor() + bubbles, keys(40), start=(0.5, 64.0, 0.5)),
        scene("sink-in-bubble-column", floor() + bubbles, keys(30), start=(1.5, 69.0, 0.5)),
        scene("walk-into-border", floor(), keys(30, forward=True), border=(0.5, 0.5, 6.0)),
        scene("sprint-jump-into-border", floor(),
              keys(10, forward=True, sprint=True) + keys(20, forward=True, sprint=True, jump=True),
              border=(0.5, 0.5, 6.0)),
        scene("slide-along-border-corner", floor(),
              keys(24, forward=True, right=True) if False else keys(24, forward=True, left=True),
              border=(0.5, 0.5, 5.0), y_rot=45.0),
        scene("start-outside-border", floor(), keys(10, forward=True), border=(40.5, 40.5, 6.0)),
        scene("crouch-in-border", floor(), keys(20, forward=True, sneak=True), border=(0.5, 0.5, 4.0)),
    ]

    yield {
        "case_id": "movement/local-player-ticks",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": rows,
    }
