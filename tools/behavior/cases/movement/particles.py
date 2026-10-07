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
