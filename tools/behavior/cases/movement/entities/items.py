"""The movement family: entities: the `items` cases."""

from cases.movement.entities.common import *
from cases.movement.entities.shared import *


def cases():
    step = [{"x": x, "y": 64, "z": z, "block": "minecraft:stone"}
            for x in range(2, 9) for z in range(-8, 9)]

    rows = [
        # A block broken a little above the floor: the drop's own random motion.
        row("block-drop", floor(), (0.5, 65.2, 0.5), (0.05, 0.2, -0.03), 80),
        row("block-drop-other-way", floor(), (0.5, 65.2, 0.5), (-0.08, 0.2, 0.09), 80),
        row("dropped-high", floor(), (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), 80),
        row("dropped-high-sideways", floor(), (0.5, 70.0, 0.5), (0.2, 0.0, -0.15), 100),
        # Resting on the floor, still: one tick in four moves it, by the id.
    ] + [
        row(f"resting-id-{entity_id}", floor(), (0.5, 64.0, 0.5), (0.0, 0.0, 0.0), 24,
            on_ground=True, entity_id=entity_id)
        for entity_id in (20, 21, 22, 23, -3)
    ] + [
        row("sliding-on-stone", floor(), (0.5, 64.0, 0.5), (0.2, 0.0, 0.0), 60, on_ground=True),
        row("sliding-on-ice", floor("minecraft:ice"), (0.5, 64.0, 0.5), (0.2, 0.0, 0.1), 80,
            on_ground=True),
        row("sliding-on-packed-ice", floor("minecraft:packed_ice"), (0.5, 64.0, 0.5),
            (0.2, 0.0, 0.0), 80, on_ground=True),
        row("sliding-on-slime", floor("minecraft:slime_block"), (0.5, 64.0, 0.5),
            (0.2, 0.0, 0.0), 60, on_ground=True),
        row("falling-onto-slime", floor("minecraft:slime_block"), (0.5, 68.0, 0.5),
            (0.0, 0.0, 0.0), 80),
        row("sliding-on-soul-sand", floor("minecraft:soul_sand"), (0.5, 64.0, 0.5),
            (0.2, 0.0, 0.0), 60, on_ground=True),
        row("falling-onto-soul-sand", floor("minecraft:soul_sand"), (0.5, 66.0, 0.5),
            (0.1, 0.0, 0.0), 60),
        row("sliding-on-honey", floor("minecraft:honey_block"), (0.5, 64.0, 0.5),
            (0.2, 0.0, 0.0), 60, on_ground=True),
        # Obstacles.
        row("into-a-wall", floor() + wall, (0.5, 64.0, 0.5), (0.4, 0.0, 0.0), 40,
            on_ground=True),
        row("falling-along-a-wall", floor() + wall, (2.8, 68.0, 0.5), (0.3, 0.0, 0.0), 60),
        row("into-a-ceiling", floor() + ceiling, (0.5, 64.0, 0.5), (0.0, 0.6, 0.0), 40),
        row("no-step-up", floor() + step, (0.5, 64.0, 0.5), (0.3, 0.0, 0.0), 40, on_ground=True),
        # The edge between two floors, where the block below is picked by
        # the supporting block, and not the item's own column.
        row("edge-of-ice", floor() + floor("minecraft:ice", x_low=1), (0.9, 64.0, 0.5),
            (0.0, 0.0, 0.0), 40, on_ground=True),
        row("landing-on-the-edge-of-ice", floor() + floor("minecraft:ice", x_low=1),
            (0.9, 66.0, 0.5), (0.05, 0.0, 0.0), 60),
        row("sliding-off-ice-onto-stone", floor() + floor("minecraft:ice", x_low=-8, high=0),
            (-0.5, 64.0, 0.5), (0.3, 0.0, 0.0), 60, on_ground=True),
        # Shapes that are not whole blocks.
        row("landing-on-a-slab", [{"x": 0, "y": 64, "z": 0, "block": "minecraft:stone_slab"}]
            + floor(), (0.5, 67.0, 0.5), (0.0, 0.0, 0.0), 60),
        row("landing-on-a-fence", [{"x": 0, "y": 64, "z": 0, "block": "minecraft:oak_fence"}]
            + floor(), (0.5, 67.0, 0.5), (0.0, 0.0, 0.0), 60),
        row("landing-on-a-carpet", [{"x": 0, "y": 64, "z": 0, "block": "minecraft:white_carpet"}]
            + floor(), (0.5, 67.0, 0.5), (0.0, 0.0, 0.0), 60),
        # A box the floor does not reach.
        row("falling-out-of-the-world", [], (0.5, 10.0, 0.5), (0.0, 0.0, 0.0), 30),
    ]

    waterfall = [level(0, 64 + y, 0, 0) for y in range(3, 4)] + [
        level(0, 64 + y, 0, 8) for y in range(0, 3)]

    rows += [
        row("falling-into-water", floor() + pool("minecraft:water", 3), (0.5, 70.0, 0.5),
            (0.0, 0.0, 0.0), 120),
        row("falling-into-water-sideways", floor() + pool("minecraft:water", 3), (-2.5, 70.0, 0.5),
            (0.15, 0.0, 0.05), 120),
        row("sunk-in-deep-water", floor() + pool("minecraft:water", 5), (0.5, 64.2, 0.5),
            (0.0, 0.0, 0.0), 100),
        row("lying-in-shallow-water", floor() + pool("minecraft:water", 1), (0.5, 64.0, 0.5),
            (0.0, 0.0, 0.0), 80, on_ground=True),
        row("thrown-up-through-water", floor() + pool("minecraft:water", 3), (0.5, 64.5, 0.5),
            (0.0, 0.5, 0.0), 100),
        row("water-over-ice", floor("minecraft:ice") + pool("minecraft:water", 2), (0.5, 66.0, 0.5),
            (0.1, 0.0, 0.0), 80),
        row("water-over-slime", floor("minecraft:slime_block") + pool("minecraft:water", 2),
            (0.5, 66.0, 0.5), (0.0, 0.0, 0.0), 80),
        row("water-surface-eighth", floor() + pool("minecraft:water", 1, properties={"level": "7"}),
            (0.5, 64.0, 0.5), (0.0, 0.0, 0.0), 60, on_ground=True),
        row("water-surface-sixth", floor() + pool("minecraft:water", 1, properties={"level": "6"}),
            (0.5, 64.0, 0.5), (0.0, 0.0, 0.0), 60, on_ground=True),
        row("in-a-stream", floor() + stream, (1.5, 64.0, 0.5), (0.0, 0.0, 0.0), 100, on_ground=True),
        row("dropped-into-a-stream", floor() + stream, (2.5, 67.0, 0.5), (0.0, 0.0, 0.0), 100),
        row("at-the-stream-end", floor() + stream, (7.5, 64.0, 0.5), (0.0, 0.0, 0.0), 80,
            on_ground=True),
        row("in-a-waterfall", floor() + waterfall, (0.5, 66.8, 0.5), (0.0, 0.0, 0.0), 80),
        row("falling-into-lava", floor() + pool("minecraft:lava", 3), (0.5, 70.0, 0.5),
            (0.0, 0.0, 0.0), 120),
        row("lying-in-shallow-lava", floor() + pool("minecraft:lava", 1), (0.5, 64.0, 0.5),
            (0.0, 0.0, 0.0), 80, on_ground=True),
        row("lava-flow", floor() + [level(0, 64, 0, 0, "minecraft:lava")]
            + [level(x, 64, 0, x, "minecraft:lava") for x in range(1, 4)],
            (1.5, 64.0, 0.5), (0.0, 0.0, 0.0), 80, on_ground=True),
        row("between-water-and-lava", floor() + pool("minecraft:water", 3, -3, 0)
            + pool("minecraft:lava", 3, 1, 3), (0.5, 66.0, 0.5), (0.0, 0.0, 0.0), 100),
        row("into-a-wall-under-water", floor() + pool("minecraft:water", 3) + wall,
            (0.5, 65.0, 0.5), (0.3, 0.0, 0.0), 60),
    ]

    def motion(tick, velocity):
        return {"tick": tick, "type": "motion", "motion_x": float_bits(velocity[0], "f64"),
                "motion_y": float_bits(velocity[1], "f64"),
                "motion_z": float_bits(velocity[2], "f64")}

    rows += [
        row("moves-count-from-the-last-packet", floor(), (0.5, 66.0, 0.5), (0.1, 0.1, 0.0), 40,
            events=[move(5, 400, 1200, 0), move(10, 800, -300, 40), move(15, 0, 0, 0, True),
                    move(25, -200, 0, 100, True)]),
        row("sync-then-moves", floor(), (0.5, 66.0, 0.5), (0.1, 0.1, 0.0), 50,
            events=[sync(8, (1.25, 64.0, 0.75), on_ground=True), move(12, 100, 0, -100, True),
                    move(30, 0, 8, 0, True)]),
        row("motion-packets", floor(), (0.5, 65.0, 0.5), (0.0, 0.0, 0.0), 60, on_ground=True,
            events=[motion(3, (0.15, 0.3, 0.0)), move(10, 1000, 2000, 0),
                    motion(20, (0.0, 0.2, 0.1)), sync(40, (3.0, 64.0, 0.5), on_ground=True)]),
        row("sync-while-resting", floor(), (0.5, 64.0, 0.5), (0.0, 0.0, 0.0), 40, on_ground=True,
            events=[sync(10, (0.5, 64.0, 0.5), on_ground=True),
                    sync(30, (0.62, 64.0, 0.5), on_ground=True)]),
    ]

    yield {
        "case_id": "movement/item-entity-ticks",
        "family": "movement",
        "operation": "item_entity_ticks",
        "rows": rows,
    }
