"""The movement family: the player and the world: the `interaction` cases."""

from cases.movement.player.common import *
from cases.movement.player.shared import *


def cases():
    # Breaking: the attack key as `handleKeybinds` spends it, into the
    # reference's own game mode. The target is named, not picked. Each tick
    # reports the game mode's destroy state, the crack stage the level keeps,
    # the prediction sequence and the watched block states.
    target = {"x": 0, "y": 64, "z": 2, "face": "north"}

    second = {"x": 1, "y": 64, "z": 2, "face": "north"}

    watch = [target, second]

    def hits(count, aim=target, down=True, clicks=0, server=None, **held):
        ticks = keys(count, **held)
        for index, tick in enumerate(ticks):
            attack = {"clicks": clicks if index == 0 else 0, "down": down,
                      "watch": [{"x": w["x"], "y": w["y"], "z": w["z"]} for w in watch]}
            if aim is not None:
                attack["target"] = aim
            if server and index == 0:
                attack["server"] = server
            tick["attack"] = attack
        return ticks

    air = {"x": 0, "y": 64, "z": 2, "block": "minecraft:air"}

    dirt = [at(target, "minecraft:dirt")]

    stone = [at(target, "minecraft:stone")]

    pool = [{"x": x, "y": y, "z": z, "block": "minecraft:water"}
            for x in range(-3, 4) for z in range(-3, 2) for y in range(64, 67)]

    breaking_rows = [
        breaking("dirt-bare-hand", dirt,
                 hits(1, clicks=1) + hits(19) + hits(1, server=[air, {"ack": 2}]) + hits(3)),
        breaking("stone-wooden-pickaxe", stone, hits(1, clicks=1) + hits(26),
                 main_hand="minecraft:wooden_pickaxe"),
        breaking("stone-wooden-shovel", stone, hits(1, clicks=1) + hits(12),
                 main_hand="minecraft:wooden_shovel"),
        breaking("stone-release", stone, hits(1, clicks=1) + hits(9) + hits(3, down=False)),
        breaking("switch-target", dirt + [at(second, "minecraft:dirt")],
                 hits(1, clicks=1) + hits(5) + hits(20, aim=second)),
        breaking("dirt-rejected", dirt,
                 hits(1, clicks=1) + hits(16)
                 + hits(1, server=[at(target, "minecraft:dirt"), {"ack": 2}]) + hits(4)),
        breaking("dirt-haste", dirt, hits(1, clicks=1) + hits(12),
                 effects=[{"effect": "minecraft:haste", "amplifier": 1}]),
        breaking("dirt-mining-fatigue", dirt, hits(1, clicks=1) + hits(12),
                 effects=[{"effect": "minecraft:mining_fatigue", "amplifier": 0}]),
        breaking("dirt-jumping", dirt, hits(1, clicks=1, jump=True) + hits(20, jump=True)),
        breaking("dirt-underwater", dirt + pool, hits(1, clicks=1) + hits(20),
                 start=(0.5, 64.0, 0.5)),
        breaking("waterlogged-slab-axe",
                 [at(target, "minecraft:oak_slab", {"waterlogged": "true"})],
                 hits(1, clicks=1) + hits(34), main_hand="minecraft:wooden_axe"),
        breaking("miss-then-hold", dirt,
                 hits(1, aim=None, clicks=1) + hits(3, aim=None, clicks=1)
                 + hits(8, clicks=1) + hits(2, down=False) + hits(1, clicks=1) + hits(3)),
        breaking("creative-hold", dirt + [at(second, "minecraft:dirt")],
                 hits(1, clicks=1) + hits(3) + hits(8, aim=second), game_mode="creative"),
        breaking("creative-sword", dirt, hits(1, clicks=1) + hits(6),
                 main_hand="minecraft:wooden_sword", game_mode="creative"),
        breaking("adventure-bare-hand", dirt, hits(1, clicks=1) + hits(4), game_mode="adventure"),
    ]

    top_of = aim_at(0, 63, 2, "up", (0.5, 64.0, 2.5))

    around = [(0, 64, 2), (0, 65, 2), (1, 64, 2), (-1, 64, 2), (0, 64, 3)]

    side_high = aim_at(0, 64, 3, "north", (0.5, 64.75, 3.0))

    side_low = aim_at(0, 64, 3, "north", (0.5, 64.25, 3.0))

    pool_under = [put(x, 64, z, "minecraft:water") for x in range(-2, 3) for z in range(1, 5)]

    using_rows = [
        using("stone-on-floor", [], uses(1, top_of, clicks=1, watch_at=around) + uses(12, top_of, down=True, watch_at=around),
              main_hand="minecraft:stone", count=4),
        using("empty-hand-on-stone", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around)),
        using("use-in-air", [], uses(1, None, clicks=1, watch_at=around) + uses(6, None, down=True, watch_at=around),
              main_hand="minecraft:stone", count=4),
        using("slab-bottom-then-double", [],
              uses(1, top_of, clicks=1, watch_at=around) + uses(5, top_of, watch_at=around)
              + uses(1, aim_at(0, 64, 2, "up", (0.5, 64.5, 2.5)), clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:oak_slab", count=4),
        using("slab-on-side-halves", [put(0, 64, 3, "minecraft:stone")],
              uses(1, side_high, clicks=1, watch_at=around) + uses(5, side_low, watch_at=around)
              + uses(1, aim_at(0, 64, 3, "west", (0.0, 64.25, 3.5)), clicks=1, watch_at=around + [(-1, 64, 3)]),
              main_hand="minecraft:stone_slab", count=4),
        using("log-axis-from-face", [put(0, 64, 3, "minecraft:stone")],
              uses(1, side_high, clicks=1, watch_at=around) + uses(5, top_of, watch_at=around)
              + uses(1, aim_at(0, 64, 3, "east", (1.0, 64.5, 3.5)), clicks=1, watch_at=around + [(1, 64, 3)]),
              main_hand="minecraft:oak_log", count=4),
        using("stairs-meet-stairs",
              [put(1, 64, 2, "minecraft:oak_stairs", {"facing": "east", "half": "bottom", "shape": "straight"})],
              uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:oak_stairs", count=4, y_rot=90.0),
        using("stairs-top-half-facing", [put(0, 64, 3, "minecraft:stone")],
              uses(1, side_high, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone_stairs", count=4, y_rot=-45.0),
        using("slab-into-water", pool_under,
              uses(1, aim_at(0, 63, 2, "up", (0.5, 64.0, 2.5)), clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:oak_slab", count=4),
        using("place-where-standing", [],
              uses(1, aim_at(0, 63, 0, "up", (0.5, 64.0, 0.5)), clicks=1, watch_at=[(0, 64, 0)])
              + uses(3, top_of, watch_at=[(0, 64, 0)]),
              main_hand="minecraft:stone", count=4),
        using("creative-keeps-count", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone", count=4, game_mode="creative"),
        using("crafting-table-use", [put(0, 64, 2, "minecraft:crafting_table")],
              uses(1, aim_at(0, 64, 2, "up", (0.5, 65.0, 2.5)), clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone", count=4),
        using("sneak-place-on-crafting-table", [put(0, 64, 2, "minecraft:crafting_table")],
              uses(1, aim_at(0, 64, 2, "up", (0.5, 65.0, 2.5)), clicks=1, watch_at=around, sneak=True)
              + uses(3, top_of, watch_at=around, sneak=True),
              main_hand="minecraft:stone", count=4),
        using("off-hand-places", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              off_hand="minecraft:cobblestone"),
        using("adventure-cannot-place", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone", count=4, game_mode="adventure"),
        using("place-last-block", [], uses(1, top_of, clicks=1, watch_at=around) + uses(3, top_of, watch_at=around),
              main_hand="minecraft:stone", count=1),
    ]

    yield {
        "case_id": "movement/using",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": using_rows,
    }

    yield {
        "case_id": "movement/breaking",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": breaking_rows,
    }

    zombie = mob(20, "zombie", 0.5, 64.0, 2.5)

    entity_rows = [
        facing("attack-zombie", [zombie], aims(2, clicks=1) + aims(3)),
        facing("attack-twice", [zombie], aims(1, clicks=2) + aims(3)),
        facing("attack-held", [zombie], aims(12, down=True)),
        facing("use-on-zombie-empty-hands", [zombie], aims(1, use_clicks=1) + aims(3)),
        facing("use-on-zombie-with-stone", [zombie], aims(1, use_clicks=1) + aims(3),
               main_hand="minecraft:stone"),
        # The swing lasts as long as haste (or the conduit's power) leaves of it, or mining fatigue adds to it
        # (a spear's own length is read the same way for a remote player), and the off hand's arm swings
        # when it is the one that acts.
        facing("attack-with-haste", [zombie], aims(2, clicks=1) + aims(8),
               effects=[{"effect": "minecraft:haste", "amplifier": 1}]),
        facing("attack-with-conduit-power-and-haste", [zombie], aims(2, clicks=1) + aims(8),
               effects=[{"effect": "minecraft:haste", "amplifier": 0}, {"effect": "minecraft:conduit_power", "amplifier": 2}]),
        facing("attack-with-mining-fatigue", [zombie], aims(2, clicks=1) + aims(14),
               effects=[{"effect": "minecraft:mining_fatigue", "amplifier": 1}]),
        facing("off-hand-use-with-haste", [zombie], aims(1, use_clicks=1) + aims(6), off_hand="minecraft:cobblestone",
               effects=[{"effect": "minecraft:haste", "amplifier": 0}]),
        facing("use-on-zombie-both-hands", [zombie], aims(1, use_clicks=1) + aims(3),
               main_hand="minecraft:stone", off_hand="minecraft:cobblestone"),
        facing("use-held-on-zombie", [zombie], aims(10, use_down=True)),
        facing("use-on-zombie-sneaking", [zombie], aims(1, use_clicks=1, sneak=True) + aims(3, sneak=True)),
        facing("pig-from-above", [mob(21, "pig", 0.5, 64.0, 2.5)], aims(1, clicks=1) + aims(2), x_rot=30.0),
        facing("cow-below-the-aim", [mob(22, "cow", 0.5, 64.0, 2.5)], aims(1, clicks=1) + aims(2)),
        facing("armor-stand", [mob(23, "armor_stand", 0.5, 64.0, 2.5)], aims(1, clicks=1) + aims(2)),
        facing("nearest-of-two", [mob(24, "skeleton", 0.5, 64.0, 2.6), mob(25, "zombie", 0.5, 64.0, 1.8)],
               aims(1, clicks=1) + aims(2)),
        facing("nearest-of-two-reversed", [mob(25, "zombie", 0.5, 64.0, 1.8), mob(24, "skeleton", 0.5, 64.0, 2.6)],
               aims(1, clicks=1) + aims(2)),
        facing("inside-the-entity", [mob(26, "zombie", 0.5, 64.0, 0.5)], aims(1, clicks=1) + aims(2)),
        facing("block-in-front", [zombie], aims(1, clicks=1) + aims(6, down=True), blocks=[put(0, 65, 1, "minecraft:dirt")]),
        facing("entity-in-front-of-block", [mob(20, "zombie", 0.5, 64.0, 1.8)],
               aims(1, clicks=1) + aims(2), blocks=[put(0, 65, 4, "minecraft:stone")]),
        facing("out-of-reach", [mob(20, "zombie", 0.5, 64.0, 4.0)], aims(1, clicks=1) + aims(3)),
        facing("out-of-reach-then-block", [mob(20, "zombie", 0.5, 64.0, 4.0)],
               aims(1, clicks=1) + aims(3), blocks=[put(0, 65, 6, "minecraft:stone")]),
        facing("edge-of-reach", [mob(20, "zombie", 0.5, 64.0, 3.45)], aims(1, clicks=1) + aims(3)),
        facing("not-pickable-arrow", [mob(27, "arrow", 0.5, 65.4, 2.5)], aims(1, clicks=1) + aims(3)),
        facing("fireball-grows-the-aim", [mob(28, "fireball", 1.8, 65.1, 2.5)],
               aims(1, clicks=1) + aims(3)),
        facing("fireball-too-far-aside", [mob(28, "fireball", 2.6, 65.1, 2.5)],
               aims(1, clicks=1) + aims(3)),
        facing("boat-ahead", [mob(29, "oak_boat", 0.5, 64.0, 2.5)], aims(1, clicks=1) + aims(3), x_rot=20.0),
        facing("turned-away", [zombie], aims(1, clicks=1) + aims(3), y_rot=180.0),
        facing("creative-attack", [zombie], aims(1, clicks=1) + aims(3), game_mode="creative"),
        facing("adventure-attack", [zombie], aims(1, clicks=1) + aims(3), game_mode="adventure"),
        facing("spectator-attack", [zombie], aims(1, clicks=1) + aims(3), game_mode="spectator"),
        facing("sword-in-hand", [zombie], aims(1, clicks=1) + aims(3), main_hand="minecraft:iron_sword"),
        # The pick item key (the middle button): the aimed entity's or block's item.
        facing("pick-zombie", [zombie], aims(1, pick_clicks=1) + aims(3)),
        facing("pick-zombie-twice", [zombie], aims(1, pick_clicks=2) + aims(2)),
        facing("pick-block", [], aims(1, pick_clicks=1) + aims(3), blocks=[put(0, 65, 2, "minecraft:dirt")]),
        facing("pick-block-behind-entity", [zombie], aims(1, pick_clicks=1) + aims(2),
               blocks=[put(0, 65, 5, "minecraft:stone")]),
        facing("pick-nothing", [], aims(1, pick_clicks=1) + aims(3)),
        facing("pick-out-of-reach", [mob(20, "zombie", 0.5, 64.0, 4.0)], aims(1, pick_clicks=1) + aims(2)),
        facing("pick-after-use-and-attack", [zombie], aims(1, clicks=1, use_clicks=1, pick_clicks=1) + aims(2)),
        facing("pick-in-creative", [zombie], aims(1, pick_clicks=1) + aims(2), game_mode="creative"),
        facing("pick-in-spectator", [zombie], aims(1, pick_clicks=1) + aims(2), game_mode="spectator"),
    ]

    yield {
        "case_id": "movement/entity-aim",
        "family": "movement",
        "operation": "local_player_ticks",
        "rows": entity_rows,
    }
