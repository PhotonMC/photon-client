"""The movement family: entities: the `shadows` cases."""

from cases.movement.entities.common import *
from cases.movement.entities.shared import *


def cases():
    ground = floor()

    roof = [{"x": x, "y": 67, "z": z, "block": "minecraft:stone"} for x in range(-8, 9) for z in range(-8, 9)]

    shadow_rows = [
        shadow("plain-floor", ground),
        shadow("small-radius", ground, radius=0.15, strength=0.75),
        shadow("wide-radius", ground, radius=2.6),
        shadow("player", ground, radius=0.5),
        shadow("at-a-block-corner", ground, position=(0.02, 64.0, 0.98)),
        shadow("on-the-border", ground, position=(0.0, 64.0, 0.0), radius=1.0),
        shadow("raised-half", ground, position=(0.5, 64.5, 0.5)),
        shadow("raised-one", ground, position=(0.5, 65.0, 0.5)),
        shadow("raised-two", ground, position=(0.5, 66.0, 0.5)),
        shadow("raised-too-high", ground, position=(0.5, 70.0, 0.5)),
        shadow("raised-but-weak", ground, position=(0.5, 65.0, 0.5), strength=0.6),
        shadow("far-from-the-camera-64", ground, distance_sq=64.0),
        shadow("far-from-the-camera-200", ground, distance_sq=200.0),
        shadow("far-from-the-camera-255", ground, distance_sq=255.0),
        shadow("far-from-the-camera-256", ground, distance_sq=256.0),
        shadow("far-from-the-camera-300", ground, distance_sq=300.0),
        shadow("invisible", ground, invisible=True),
        shadow("option-off", ground, option=False),
        shadow("under-a-roof", ground + roof, position=(0.5, 64.0, 0.5)),
        shadow("under-a-high-roof", ground + [dict(b, y=80) for b in roof]),
        shadow("by-a-torch", ground + [{"x": 1, "y": 64, "z": 0, "block": "minecraft:torch"}]),
        shadow("by-a-lantern", ground + [{"x": 0, "y": 64, "z": 1, "block": "minecraft:lantern"}]),
        shadow("over-nothing", [], position=(0.5, 64.0, 0.5)),
        shadow("far-below-the-ground", ground, position=(0.5, 40.0, 0.5)),
    ]

    for block, props in [
        ("minecraft:stone_slab", {"type": "bottom"}), ("minecraft:stone_slab", {"type": "top"}),
        ("minecraft:stone_slab", {"type": "double"}), ("minecraft:oak_stairs", None),
        ("minecraft:white_carpet", None), ("minecraft:oak_fence", None), ("minecraft:glass", None),
        ("minecraft:ice", None), ("minecraft:soul_sand", None), ("minecraft:dirt_path", None),
        ("minecraft:farmland", None), ("minecraft:honey_block", None), ("minecraft:barrier", None),
        ("minecraft:water", None), ("minecraft:lava", None), ("minecraft:snow", {"layers": "1"}),
        ("minecraft:snow", {"layers": "8"}), ("minecraft:cauldron", None), ("minecraft:cactus", None),
        ("minecraft:chest", None), ("minecraft:oak_leaves", None), ("minecraft:light", None),
        ("minecraft:oak_trapdoor", {"open": "false", "half": "bottom"}), ("minecraft:oak_door", None),
        ("minecraft:piston", None), ("minecraft:iron_bars", None), ("minecraft:bedrock", None),
    ]:
        label = block.split(":")[1] + ("-" + "-".join(props.values()) if props else "")
        shadow_rows.append(shadow(f"on-{label}", ground + [
            dict({"x": 0, "y": 63, "z": 0, "block": block}, **({"properties": props} if props else {}))]))

    yield {
        "case_id": "movement/entity-shadows",
        "family": "movement",
        "operation": "entity_shadows",
        "rows": shadow_rows,
    }
