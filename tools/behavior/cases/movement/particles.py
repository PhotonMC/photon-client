"""The movement family: particles of the reference (`Particle.tick`, the smoke and flame
providers) ticked in a scene, with the random seed they are made with fixed; and the registry of
particle types they are named by."""

from cases.helpers import *
from cases.movement.entities.common import floor

PARTIALS = ["00000000", "3f000000", "3f800000"]

CEILING = [{"x": x, "y": 68, "z": z, "block": "minecraft:stone"}
           for x in range(-8, 9) for z in range(-8, 9)]
WALL = [{"x": 2, "y": y, "z": z, "block": "minecraft:stone"}
        for y in range(63, 70) for z in range(-8, 9)]
SLAB = [{"x": 0, "y": 66, "z": 0, "block": "minecraft:stone_slab"}]
SOLID = [{"x": x, "y": y, "z": z, "block": "minecraft:stone"}
         for x in range(-3, 4) for y in range(60, 68) for z in range(-3, 4)]


def row(label, particle, blocks, at, delta, seed, ticks, sprites=8, engine_seed=7):
    return {
        "label": label, "particle": particle, "blocks": blocks, "chunk_radius": 1,
        "x": float_bits(at[0], "f64"), "y": float_bits(at[1], "f64"), "z": float_bits(at[2], "f64"),
        "xa": float_bits(delta[0], "f64"), "ya": float_bits(delta[1], "f64"),
        "za": float_bits(delta[2], "f64"),
        "seed": seed, "engine_seed": engine_seed, "sprites": sprites, "partials": PARTIALS,
        "tick_count": ticks,
    }


def particle_type_cases():
    yield {"case_id": "movement/particle-types", "family": "movement",
           "operation": "particle_types", "rows": [{"label": "registry"}]}


def particle_tick_cases():
    still = (0.0, 0.0, 0.0)
    rows = [
        # Smoke: what the constructor draws, the sprite that follows the age, and the end of its life.
        row("smoke-in-the-air", "smoke", [], (0.5, 70.0, 0.5), still, 12345, 40),
        row("smoke-another-seed", "smoke", [], (0.5, 70.0, 0.5), still, -987654321, 40),
        row("smoke-with-a-push", "smoke", [], (0.5, 70.0, 0.5), (0.02, 0.05, -0.03), 99, 40),
        row("smoke-one-sprite", "smoke", [], (0.5, 70.0, 0.5), still, 5, 30, sprites=1),
        row("smoke-three-sprites", "smoke", [], (0.5, 70.0, 0.5), still, 6, 30, sprites=3),
        # With physics: a ceiling stops its rise (and speeds the sideways drift up), a wall and a floor stop it.
        row("smoke-under-a-ceiling", "smoke", CEILING, (0.5, 66.0, 0.5), (0.0, 0.1, 0.0), 12345, 40),
        row("smoke-along-a-wall", "smoke", WALL + floor(), (1.5, 65.0, 0.5), (0.1, 0.0, 0.0), 321, 40),
        row("smoke-onto-a-floor", "smoke", floor(), (0.5, 64.5, 0.5), (0.05, -0.3, 0.0), 77, 40),
        row("smoke-beside-a-slab", "smoke", SLAB, (0.9, 66.2, 0.5), (-0.1, 0.0, 0.0), 8, 30),
        # Flame: no physics (it goes through blocks), shrinks, glows, and picks its sprite from the engine's random.
        row("flame-in-the-air", "flame", [], (0.5, 70.0, 0.5), still, 12345, 30, sprites=1),
        row("flame-another-seed", "flame", [], (0.5, 70.0, 0.5), (0.0, 0.02, 0.0), 4242, 30, sprites=1),
        row("flame-through-a-ceiling", "flame", CEILING, (0.5, 66.0, 0.5), (0.0, 0.1, 0.0), 12345, 40,
            sprites=1),
        row("flame-inside-stone", "flame", SOLID, (0.5, 64.0, 0.5), still, 12345, 30, sprites=1),
        row("flame-three-sprites", "flame", [], (0.5, 70.0, 0.5), still, 12345, 10, sprites=3,
            engine_seed=11),
        row("flame-three-sprites-other-engine", "flame", [], (0.5, 70.0, 0.5), still, 12345, 10,
            sprites=3, engine_seed=12),
        row("small-flame", "small_flame", [], (0.5, 70.0, 0.5), still, 12345, 30, sprites=1),
    ]
    yield {"case_id": "movement/particle-ticks", "family": "movement", "operation": "particle_ticks",
           "rows": rows}


def particle_cases():
    yield from particle_type_cases()
    yield from particle_tick_cases()
    yield from particle_engine_cases()
    yield from particle_quad_cases()
    yield from particle_terrain_cases()


def engine_row(label, events, status="ALL", ticks=3, seeds=8, camera=(0.0, 64.0, 0.0), level_seed=101,
               listener_seed=202, engine_seed=303, describe=4, sprites=8):
    return {
        "label": label, "blocks": [], "chunk_radius": 1, "events": events, "status": status,
        "tick_count": ticks, "particle_seeds": seeds, "particle_seed_base": 1000,
        "particle_seed_step": 7919, "camera_x": float_bits(camera[0], "f64"),
        "camera_y": float_bits(camera[1], "f64"), "camera_z": float_bits(camera[2], "f64"),
        "level_seed": level_seed, "listener_seed": listener_seed, "engine_seed": engine_seed,
        "sprites": sprites, "describe": describe, "partials": PARTIALS,
    }


def add(tick, particle, at, push=(0.0, 0.0, 0.0), override=False, always_show=False):
    return {"tick": tick, "type": "add", "particle": particle, "override": override,
            "always_show": always_show, "x": float_bits(at[0], "f64"), "y": float_bits(at[1], "f64"),
            "z": float_bits(at[2], "f64"), "xa": float_bits(push[0], "f64"),
            "ya": float_bits(push[1], "f64"), "za": float_bits(push[2], "f64")}


def packet(tick, particle, at, spread, speed, count, override=False, always_show=False):
    return {"tick": tick, "type": "packet", "particle": particle, "override": override,
            "always_show": always_show, "x": float_bits(at[0], "f64"), "y": float_bits(at[1], "f64"),
            "z": float_bits(at[2], "f64"), "x_dist": float_bits(spread[0]),
            "y_dist": float_bits(spread[1]), "z_dist": float_bits(spread[2]),
            "max_speed": float_bits(speed), "count": count}


def particle_engine_cases():
    near = (5.0, 64.0, 5.0)
    far = (40.0, 64.0, 0.0)
    rows = [
        # What the level lets in: near the camera, far from it, and far but let in by the packet's flag.
        engine_row("level-add-near-and-far", [
            add(0, "smoke", near), add(0, "smoke", far), add(0, "flame", far, override=True),
            add(1, "small_flame", near, (0.0, 0.1, 0.0)), add(1, "soul_fire_flame", (0.0, 64.0, 31.9)),
            add(1, "copper_fire_flame", (0.0, 64.0, 32.1))]),
        # The option: some are dropped by the level's own random at DECREASED, all at MINIMAL unless
        # shown always.
        engine_row("decreased", [add(0, "smoke", near) for _ in range(14)], status="DECREASED", seeds=20),
        engine_row("minimal", [add(0, "smoke", near) for _ in range(6)], status="MINIMAL"),
        engine_row("minimal-always-shown", [add(0, "smoke", near, always_show=True) for _ in range(30)],
                   status="MINIMAL", seeds=30),
        engine_row("decreased-always-shown", [add(0, "flame", near, always_show=True) for _ in range(20)],
                   status="DECREASED", seeds=20),
        # The packet: one particle moving as told, and a count, spread and sped by the listener's random.
        engine_row("packet-without-a-count", [packet(0, "smoke", near, (0.5, 0.25, -0.125), 2.0, 0)]),
        engine_row("packet-count", [packet(0, "flame", (0.0, 64.0, 0.0), (1.0, 0.5, 1.0), 0.05, 20)],
                   seeds=20, describe=20),
        engine_row("packet-count-decreased", [packet(1, "smoke", (0.0, 64.0, 0.0), (2.0, 1.0, 2.0), 0.02, 30)],
                   status="DECREASED", seeds=30),
        engine_row("packet-far", [packet(0, "smoke", far, (1.0, 1.0, 1.0), 0.01, 10),
                                  packet(1, "smoke", far, (1.0, 1.0, 1.0), 0.01, 10, override=True)],
                   seeds=10),
        # Past the reservoir's start a particle gets in by a falling chance, and a full group lets none in.
        engine_row("reservoir", [packet(0, "smoke", (0.0, 64.0, 0.0), (3.0, 1.0, 3.0), 0.02, 14000),
                                 packet(1, "smoke", (0.0, 64.0, 0.0), (3.0, 1.0, 3.0), 0.02, 14000)],
                   ticks=3, seeds=28000, describe=3),
    ]
    yield {"case_id": "movement/particle-engine", "family": "movement", "operation": "particle_engine",
           "rows": rows}


def sprite_uvs(count):
    return [[float_bits((i * 16 + 1) / 128), float_bits((i * 16 + 15) / 128), float_bits(3 / 64),
             float_bits(11 / 64)] for i in range(count)]


def quad_row(label, particle, seed, ticks, camera, rotation, partial, roll=None, sprites=8):
    row = {
        "label": label, "particle": particle, "blocks": [], "chunk_radius": 1,
        "x": float_bits(0.5, "f64"), "y": float_bits(70.0, "f64"), "z": float_bits(0.5, "f64"),
        "xa": float_bits(0.01, "f64"), "ya": float_bits(0.02, "f64"), "za": float_bits(-0.01, "f64"),
        "seed": seed, "engine_seed": 7, "sprite_uvs": sprite_uvs(sprites), "tick_count": ticks,
        "camera_x": float_bits(camera[0], "f64"), "camera_y": float_bits(camera[1], "f64"),
        "camera_z": float_bits(camera[2], "f64"), "y_rot": float_bits(rotation[0]),
        "x_rot": float_bits(rotation[1]), "partial": float_bits(partial),
    }
    if roll is not None:
        row["o_roll"] = float_bits(roll[0])
        row["roll"] = float_bits(roll[1])
    return row


def particle_quad_cases():
    rows = [
        # The quad faces the camera whichever way it looks, at each partial tick.
        quad_row("smoke-facing-south", "smoke", 12345, 4, (0.5, 70.0, 5.5), (0.0, 0.0), 0.0),
        quad_row("smoke-turned", "smoke", 12345, 4, (3.25, 71.5, -2.5), (90.0, 30.0), 0.5),
        quad_row("smoke-behind-and-above", "smoke", 77, 6, (-4.0, 80.0, 6.0), (-135.0, -60.0), 0.37),
        quad_row("smoke-looking-up", "smoke", 77, 6, (0.5, 60.0, 0.5), (200.0, 80.0), 1.0),
        quad_row("smoke-far-from-the-origin", "smoke", 5, 3, (1000.25, 70.0, -30000.5), (10.0, 5.0), 0.25),
        quad_row("flame-facing-south", "flame", 12345, 3, (0.5, 70.0, 5.5), (0.0, 0.0), 0.0, sprites=1),
        quad_row("flame-turned", "flame", 4242, 5, (-3.0, 69.0, 2.0), (45.0, -20.0), 0.75, sprites=1),
        quad_row("small-flame", "small_flame", 4242, 5, (-3.0, 69.0, 2.0), (45.0, -20.0), 0.75, sprites=1),
        # A turn of the quad about its own axis: the particle's roll, between its last and its new.
        quad_row("smoke-rolled", "smoke", 12345, 4, (3.25, 71.5, -2.5), (90.0, 30.0), 0.5, roll=(0.25, 0.75)),
        quad_row("smoke-rolled-at-the-end", "smoke", 12345, 4, (0.5, 70.0, 5.5), (0.0, 0.0), 1.0,
                 roll=(-1.5, 2.0)),
    ]
    yield {"case_id": "movement/particle-quads", "family": "movement", "operation": "particle_quads",
           "rows": rows}


def terrain_row(label, mode, block, properties=None, **extra):
    state = {"block": block}
    if properties:
        state["properties"] = properties
    row = {
        "label": label, "mode": mode, "state": state, "pos_x": 2, "pos_y": 64, "pos_z": 3,
        "blocks": [dict(x=2, y=64, z=3, **state)], "chunk_radius": 1, "partials": PARTIALS,
        "sprite_uv": [float_bits(0.25), float_bits(0.3125), float_bits(0.5), float_bits(0.5625)],
    }
    row.update(extra)
    return row


def single(label, block, provider="block", seed=12345, push=(0.1, 0.2, -0.1), properties=None, ticks=24,
           engine_seed=9):
    return terrain_row(label, "single", block, properties, provider=provider, seed=seed,
                       engine_seed=engine_seed, tick_count=ticks,
                       x=float_bits(2.5, "f64"), y=float_bits(64.5, "f64"), z=float_bits(3.5, "f64"),
                       xa=float_bits(push[0], "f64"), ya=float_bits(push[1], "f64"),
                       za=float_bits(push[2], "f64"))


def particle_terrain_cases():
    rows = [
        # What a provider makes: the draws of the constructor, the tint, the window of the texture.
        single("stone", "minecraft:stone"),
        single("stone-another-seed", "minecraft:stone", seed=-77, push=(-0.4, 0.0, 0.3)),
        single("spruce-leaves-tinted", "minecraft:spruce_leaves"),
        single("birch-leaves-tinted", "minecraft:birch_leaves", seed=5),
        single("grass-block-not-tinted", "minecraft:grass_block"),
        # Types that make nothing.
        single("air", "minecraft:air"),
        single("moving-piston", "minecraft:moving_piston"),
        single("barrier", "minecraft:barrier"),
        single("structure-void", "minecraft:structure_void"),
        # The two other providers of the class: still, short-lived; and rising, long-lived.
        single("crumbling", "minecraft:stone", provider="crumbling", engine_seed=3, ticks=12),
        single("crumbling-another", "minecraft:stone", provider="crumbling", engine_seed=4, ticks=12),
        single("dust-pillar", "minecraft:stone", provider="dust_pillar", engine_seed=3, ticks=30,
               push=(0.0, 0.3, 0.0)),
        # A block breaking: one particle in each box of its shape, as many as its size asks.
        terrain_row("destroy-stone", "destroy", "minecraft:stone"),
        terrain_row("destroy-slab", "destroy", "minecraft:stone_slab", {"type": "bottom", "waterlogged": "false"}),
        terrain_row("destroy-stairs", "destroy", "minecraft:oak_stairs",
                    {"facing": "north", "half": "bottom", "shape": "straight", "waterlogged": "false"}),
        terrain_row("destroy-inner-stairs", "destroy", "minecraft:oak_stairs",
                    {"facing": "east", "half": "top", "shape": "inner_left", "waterlogged": "false"}),
        terrain_row("destroy-fence", "destroy", "minecraft:oak_fence",
                    {"north": "true", "east": "false", "south": "true", "west": "false", "waterlogged": "false"}),
        terrain_row("destroy-chest", "destroy", "minecraft:chest",
                    {"facing": "north", "type": "single", "waterlogged": "false"}),
        terrain_row("destroy-torch", "destroy", "minecraft:torch"),
        terrain_row("destroy-barrier", "destroy", "minecraft:barrier"),
        terrain_row("destroy-air", "destroy", "minecraft:air"),
    ]
    for direction in ("DOWN", "UP", "NORTH", "SOUTH", "WEST", "EAST"):
        rows.append(terrain_row(f"breaking-stone-{direction.lower()}", "breaking", "minecraft:stone",
                                direction=direction, level_seed=31))
    rows += [
        terrain_row("breaking-slab-up", "breaking", "minecraft:stone_slab",
                    {"type": "bottom", "waterlogged": "false"}, direction="UP", level_seed=8),
        terrain_row("breaking-slab-side", "breaking", "minecraft:stone_slab",
                    {"type": "top", "waterlogged": "false"}, direction="EAST", level_seed=9),
        terrain_row("breaking-barrier", "breaking", "minecraft:barrier", direction="UP", level_seed=1),
    ]
    yield {"case_id": "movement/particle-terrain", "family": "movement", "operation": "particle_terrain",
           "rows": rows}
