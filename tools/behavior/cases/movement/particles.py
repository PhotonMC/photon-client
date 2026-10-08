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
POOL = [{"x": x, "y": y, "z": z, "block": "minecraft:water"}
        for x in range(-3, 4) for y in range(60, 67) for z in range(-3, 4)]
POOL_FLOOR = [{"x": x, "y": 59, "z": z, "block": "minecraft:stone"} for x in range(-3, 4) for z in range(-3, 4)]
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
        # The rest of the smoke family: two and a half times as big, tinted, ash that falls (white ash
        # pushed by the provider's random), a dust plume that slows each tick.
        row("large-smoke", "large_smoke", [], (0.5, 70.0, 0.5), (0.0, 0.02, 0.0), 12345, 40),
        row("large-smoke-under-a-ceiling", "large_smoke", CEILING, (0.5, 66.0, 0.5), (0.0, 0.1, 0.0), 77, 40),
        row("white-smoke", "white_smoke", [], (0.5, 70.0, 0.5), still, 4242, 40),
        row("ash", "ash", [], (0.5, 70.0, 0.5), (0.3, 0.3, 0.3), 12345, 60),
        row("ash-onto-a-floor", "ash", floor(), (0.5, 64.3, 0.5), still, 99, 80),
        row("white-ash", "white_ash", [], (0.5, 70.0, 0.5), still, 12345, 60, engine_seed=13),
        row("white-ash-other-engine", "white_ash", [], (0.5, 70.0, 0.5), still, 12345, 30, engine_seed=14),
        row("dust-plume", "dust_plume", [], (0.5, 70.0, 0.5), (0.02, 0.0, -0.01), 12345, 30),
        row("dust-plume-onto-a-floor", "dust_plume", floor(), (0.5, 64.2, 0.5), still, 8, 30),
        # Soul flames: they rise with physics (a ceiling stops them), and the sculk soul glows.
        row("soul", "soul", [], (0.5, 70.0, 0.5), (0.0, 0.03, 0.0), 12345, 30),
        row("soul-under-a-ceiling", "soul", CEILING, (0.5, 67.3, 0.5), (0.0, 0.08, 0.0), 321, 30),
        row("sculk-soul", "sculk_soul", [], (0.5, 70.0, 0.5), (0.0, 0.03, 0.0), 12345, 30),
        # Campfire smoke wanders up and fades out at its end; the signal fire's lives longer.
        row("campfire-smoke", "campfire_cosy_smoke", [], (0.5, 70.0, 0.5), (0.0, 0.07, 0.0), 12345, 140,
            sprites=3, engine_seed=21),
        row("campfire-smoke-under-a-ceiling", "campfire_cosy_smoke", CEILING, (0.5, 66.5, 0.5),
            (0.0, 0.07, 0.0), 5, 90, sprites=3),
        row("signal-smoke", "campfire_signal_smoke", [], (0.5, 70.0, 0.5), (0.0, 0.07, 0.0), 12345, 60,
            sprites=3, engine_seed=22),
        # A player's cloud grows in like smoke and goes through blocks; a sneeze is green and faint.
        row("cloud", "cloud", [], (0.5, 70.0, 0.5), (0.01, 0.05, 0.0), 12345, 40),
        row("cloud-inside-stone", "cloud", SOLID, (0.5, 64.0, 0.5), still, 77, 30),
        row("sneeze", "sneeze", [], (0.5, 70.0, 0.5), (0.0, 0.0, 0.05), 12345, 30),
        # Bubbles rise in water and go out of it; the column's rise faster and live longer.
        row("bubble-in-water", "bubble", POOL + POOL_FLOOR, (0.5, 63.0, 0.5), (0.0, 0.1, 0.0), 12345, 30),
        row("bubble-to-the-surface", "bubble", POOL, (0.5, 66.8, 0.5), (0.0, 0.3, 0.0), 77, 20),
        row("bubble-in-air", "bubble", [], (0.5, 70.0, 0.5), still, 5, 5),
        row("bubble-column-up", "bubble_column_up", POOL + POOL_FLOOR, (0.5, 60.2, 0.5), (0.0, 0.0, 0.0), 12345, 60),
        row("bubble-column-up-sideways", "bubble_column_up", POOL, (2.5, 63.0, 0.5), (0.4, 0.1, 0.0), 99, 40),
        row("bubble-pop", "bubble_pop", [], (0.5, 70.0, 0.5), (0.01, 0.02, 0.0), 12345, 8),
        # Rain: a speck thrown up that falls and goes on landing, in a fluid or inside a block's shape.
        row("rain-onto-a-floor", "rain", floor(), (0.5, 64.5, 0.5), still, 12345, 30),
        row("rain-into-water", "rain", POOL, (0.5, 67.2, 0.5), still, 77, 30),
        row("rain-onto-a-slab", "rain", SLAB, (0.3, 67.0, 0.3), still, 8, 30),
        row("rain-other-seed", "rain", floor(), (0.5, 64.5, 0.5), still, -4242, 30),
        # A splash falls slower, and a push only sideways throws it so.
        row("splash", "splash", floor(), (0.5, 64.5, 0.5), still, 12345, 30),
        row("splash-sideways", "splash", floor(), (0.5, 64.5, 0.5), (0.1, 0.0, -0.05), 12345, 30),
        row("splash-pushed-up", "splash", floor(), (0.5, 64.5, 0.5), (0.1, 0.2, 0.0), 12345, 30),
        # A fishing bobber's wake grows and turns through four pictures.
        row("fishing-wake", "fishing", POOL, (0.5, 67.0, 0.5), (0.02, 0.0, 0.01), 12345, 30, sprites=4),
        # A downward current spirals through the water, and goes on the floor or out of it.
        row("current-down", "current_down", POOL + POOL_FLOOR, (0.5, 65.0, 0.5), still, 12345, 100),
        row("current-down-out-of-water", "current_down", POOL, (0.5, 60.1, 0.5), still, 7, 30),
        # Squid ink fades in its second half and sinks in the air; the glow squid's is teal.
        row("squid-ink-in-air", "squid_ink", [], (0.5, 70.0, 0.5), (0.05, 0.0, 0.0), 12345, 30),
        row("squid-ink-in-water", "squid_ink", POOL, (0.5, 63.0, 0.5), (0.0, 0.05, 0.05), 99, 30),
        row("glow-squid-ink", "glow_squid_ink", POOL, (0.5, 63.0, 0.5), (0.05, 0.0, 0.0), 12345, 30),
        # Dust: a colour darkened a little at random, a size and a life by the scale (held to 0.01–4);
        # a transition draws between its two colours.
        dict(row("dust-red", "dust", [], (0.5, 70.0, 0.5), still, 12345, 30), color=0xFF0000,
             scale=float_bits(1.0)),
        dict(row("dust-big-blue", "dust", floor(), (0.5, 64.2, 0.5), (0.0, -0.2, 0.0), 77, 40), color=0x3366FF,
             scale=float_bits(3.5)),
        dict(row("dust-too-small", "dust", [], (0.5, 70.0, 0.5), still, 5, 10), color=0x00FF00,
             scale=float_bits(0.001)),
        dict(row("dust-transition", "dust_color_transition", [], (0.5, 70.0, 0.5), still, 12345, 30),
             color=0xFF8800, to_color=0x0088FF, scale=float_bits(1.5)),
        # Spells: a sideways push from the class's own random, then rising; the instant effect's colour
        # and power, the mob effect's colour and opacity (eased back each tick), the witch's purple.
        dict(row("spell-infested", "infested", [], (0.5, 70.0, 0.5), still, 12345, 30), spell_seed=31),
        dict(row("spell-instant", "effect", [], (0.5, 70.0, 0.5), (0.0, 0.1, 0.0), 12345, 30), spell_seed=32,
             color=0xFFAA22, power=float_bits(1.5)),
        dict(row("spell-mob-effect", "entity_effect", [], (0.5, 70.0, 0.5), (0.2, 0.1, 0.0), 12345, 30),
             spell_seed=33, color=0x8044AAFF),
        dict(row("spell-witch", "witch", [], (0.5, 70.0, 0.5), still, 12345, 30, engine_seed=34), spell_seed=35),
        # Totem: green or yellow, falling fast and fading out; glows of every kind.
        row("totem", "totem_of_undying", [], (0.5, 70.0, 0.5), (0.1, 0.5, -0.1), 12345, 75),
        row("totem-other-seed", "totem_of_undying", [], (0.5, 70.0, 0.5), (0.0, 0.6, 0.0), 2, 20),
        row("glow", "glow", [], (0.5, 70.0, 0.5), (0.0, 0.05, 0.0), 12345, 30, engine_seed=41),
        row("glow-pushed", "glow", [], (0.5, 70.0, 0.5), (0.1, 0.05, 0.0), 12345, 30, engine_seed=42),
        row("electric-spark", "electric_spark", [], (0.5, 70.0, 0.5), (0.4, 0.4, 0.0), 12345, 6, engine_seed=43),
        row("scrape", "scrape", [], (0.5, 70.0, 0.5), (1.0, 2.0, 0.0), 12345, 45, engine_seed=44),
        row("wax-on", "wax_on", [], (0.5, 70.0, 0.5), (1.0, 2.0, 3.0), 12345, 45, engine_seed=45),
        row("wax-off", "wax_off", [], (0.5, 70.0, 0.5), (1.0, 2.0, 3.0), 12345, 45, engine_seed=46),
        # Portals spiral in from where they were pushed to, the reverse ones drift out faster; the
        # enchanting table's runes and the nautilus' fall back to where they started, the vault's glow
        # and fade in; the ominous spawn goes straight, blue to white.
        row("portal", "portal", [], (0.5, 70.0, 0.5), (0.8, -0.4, 0.3), 12345, 55),
        row("reverse-portal", "reverse_portal", [], (0.5, 70.0, 0.5), (0.05, 0.02, -0.03), 12345, 65),
        row("enchant", "enchant", [], (0.5, 70.0, 0.5), (1.5, 1.0, -0.5), 12345, 45),
        row("nautilus", "nautilus", [], (0.5, 70.0, 0.5), (-0.5, 0.5, 0.5), 77, 45),
        row("vault-connection", "vault_connection", [], (0.5, 70.0, 0.5), (2.0, 0.0, 1.0), 12345, 45),
        dict(row("ominous-spawning", "ominous_spawning", [], (0.5, 70.0, 0.5), (1.0, 2.0, 0.5), 12345, 35),
             level_seed=51),
        # Hanging specks: underwater, warped spores, the spore blossom's air (slow to fall, long-lived);
        # town specks drift through blocks: mycelium, a happy villager, a cracked egg, a composter (its life
        # from the level's random) and a dolphin's (faint at random, half the life).
        row("underwater", "underwater", POOL, (0.5, 63.0, 0.5), still, 12345, 40),
        row("warped-spore", "warped_spore", [], (0.5, 70.0, 0.5), still, 12345, 40, engine_seed=52),
        row("spore-blossom-air", "spore_blossom_air", floor(), (0.5, 66.0, 0.5), still, 12345, 60, engine_seed=53),
        row("mycelium", "mycelium", SOLID, (0.5, 64.0, 0.5), (0.2, 0.3, -0.1), 12345, 40),
        row("happy-villager", "happy_villager", [], (0.5, 70.0, 0.5), (0.1, 0.1, 0.1), 12345, 40),
        row("egg-crack", "egg_crack", [], (0.5, 70.0, 0.5), (0.0, 0.5, 0.0), 77, 30),
        dict(row("composter", "composter", [], (0.5, 70.0, 0.5), (0.0, 0.2, 0.0), 12345, 10), level_seed=54),
        row("dolphin", "dolphin", POOL, (0.5, 63.0, 0.5), (0.3, 0.0, 0.3), 12345, 30, engine_seed=55),
        # Hits: a crit ticks once as it is made (a damage indicator rises and lasts twenty, the enchanted
        # one is bluer); a sweep, an explosion and a sonic boom stand still, full bright, sized by the
        # first push; a poof rises; hearts and notes bob up (a note's colour from its push, under a
        # ceiling that speeds it sideways); gusts stand still; sculk charges turn by their roll and glow.
        row("crit", "crit", [], (0.5, 70.0, 0.5), (0.4, 0.2, -0.3), 12345, 12, engine_seed=61),
        row("damage-indicator", "damage_indicator", [], (0.5, 70.0, 0.5), (0.0, 0.1, 0.0), 77, 22),
        row("enchanted-hit", "enchanted_hit", [], (0.5, 70.0, 0.5), (-0.2, 0.3, 0.1), 12345, 12),
        row("sweep-attack", "sweep_attack", [], (0.5, 70.0, 0.5), (0.3, 0.0, 0.0), 12345, 6),
        row("poof", "poof", [], (0.5, 70.0, 0.5), (0.02, 0.05, -0.01), 12345, 50),
        row("explosion", "explosion", [], (0.5, 70.0, 0.5), (0.5, 0.0, 0.0), 12345, 12),
        row("sonic-boom", "sonic_boom", [], (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), 12345, 18),
        row("heart", "heart", [], (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), 12345, 18, engine_seed=62),
        row("angry-villager", "angry_villager", [], (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), 77, 18),
        row("note", "note", CEILING, (0.5, 67.5, 0.5), (0.375, 0.0, 0.0), 12345, 8, engine_seed=63),
        row("gust", "gust", [], (0.5, 70.0, 0.5), (0.25, 0.0, 0.0), 12345, 17),
        row("small-gust", "small_gust", [], (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), 77, 17),
        dict(row("sculk-charge", "sculk_charge", [], (0.5, 70.0, 0.5), (0.01, 0.02, 0.0), 12345, 22,
                 engine_seed=64), roll=float_bits(0.7)),
        row("sculk-charge-pop", "sculk_charge_pop", [], (0.5, 70.0, 0.5), (0.0, 0.03, 0.0), 12345, 12,
            engine_seed=65),
    ]
    yield {"case_id": "movement/particle-ticks", "family": "movement", "operation": "particle_ticks",
           "rows": rows}


def particle_cases():
    yield from particle_type_cases()
    yield from particle_tick_cases()
    yield from particle_engine_cases()
    yield from particle_quad_cases()
    yield from particle_terrain_cases()
    yield from item_pickup_cases()


def engine_row(label, events, status="ALL", ticks=3, seeds=8, camera=(0.0, 64.0, 0.0), level_seed=101,
               listener_seed=202, engine_seed=303, describe=4, sprites=8, blocks=()):
    return {
        "label": label, "blocks": list(blocks), "chunk_radius": 1, "events": events, "status": status,
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


def level_event(tick, event, pos, data=0):
    return {"tick": tick, "type": "level_event", "event": event, "x": pos[0], "y": pos[1], "z": pos[2],
            "data": data}


def block_event(tick, pos, b0=0, b1=0):
    return {"tick": tick, "type": "block_event", "x": pos[0], "y": pos[1], "z": pos[2], "b0": b0, "b1": b1}


def note_block(pos, instrument, note):
    return {"x": pos[0], "y": pos[1], "z": pos[2], "block": "minecraft:note_block",
            "properties": {"instrument": instrument, "note": str(note), "powered": "false"}}


NOTE_BLOCKS = [((0, 64, 0), "harp", 0), ((1, 64, 0), "harp", 12), ((2, 64, 0), "harp", 24),
               ((3, 64, 0), "basedrum", 5), ((4, 64, 0), "trumpet_oxidized", 19), ((5, 64, 0), "zombie", 7),
               ((6, 64, 0), "dragon", 3), ((7, 64, 0), "custom_head", 9)]

SOUND_ONLY_EVENTS = [1000, 1001, 1002, 1004, 1015, 1016, 1017, 1018, 1019, 1020, 1021, 1022, 1024, 1025, 1026,
                     1027, 1029, 1030, 1031, 1033, 1034, 1035, 1039, 1040, 1041, 1042, 1043, 1044, 1045, 1046,
                     1047, 1048, 1049, 1050, 1051, 1052]


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
        # Lava leaves smoke through the level (with its own random and the particles option), and the
        # rest of the rising and ash families come in from the level as the providers make them.
        engine_row("lava-leaves-smoke", [add(0, "lava", near), add(0, "lava", (5.5, 64.0, 4.5))],
                   ticks=40, seeds=120, describe=6),
        engine_row("lava-decreased", [add(0, "lava", near)], status="DECREASED", ticks=30, seeds=60),
        engine_row("lava-far", [add(0, "lava", far, override=True)], ticks=20, seeds=40),
        dict(engine_row("coloured", [
            dict(add(0, "dust", near), color=0xFF0000, scale=float_bits(1.0)),
            dict(add(0, "dust_color_transition", near), color=0xFF0000, to_color=0x0000FF, scale=float_bits(2.0)),
            dict(add(0, "effect", near), color=0x11FF22, power=float_bits(0.5)),
            dict(add(0, "instant_effect", near), color=0xFFFFFF, power=float_bits(1.0)),
            dict(add(1, "entity_effect", near), color=0x40FF00FF),
            add(1, "witch", near), add(1, "infested", near), add(1, "raid_omen", near),
            add(2, "totem_of_undying", near), add(2, "glow", near), add(2, "electric_spark", near),
            add(2, "scrape", near), add(2, "wax_on", near), add(2, "wax_off", near)],
                   ticks=10, seeds=14, describe=14), spell_seed=77),
        engine_row("rising-and-ash", [add(0, name, near, (0.0, 0.05, 0.0)) for name in (
            "large_smoke", "white_smoke", "ash", "white_ash", "dust_plume", "soul", "sculk_soul",
            "campfire_cosy_smoke", "campfire_signal_smoke", "cloud", "sneeze")],
                   ticks=12, seeds=11, describe=11),
        # The emitters leave explosions and gusts about them through the level; they are a group of their own.
        engine_row("emitters", [add(0, "explosion_emitter", (5.0, 64.0, 5.0)),
                                add(0, "gust_emitter_large", (-5.0, 64.0, 5.0)),
                                add(1, "gust_emitter_small", (5.0, 64.0, -5.0))],
                   ticks=12, seeds=100, describe=10),
        # Level events: the sounds alone (their pitch and seed from the level's random, in order), then
        # the ones that puff smoke, a cloud or an explosion about the block, and the dispenser's ten puffs
        # out of each face; one the client does not know plays nothing.
        engine_row("level-event-sounds", [level_event(index % 3, event, (index - 17, 64 + index % 5, 3 - index))
                                          for index, event in enumerate(SOUND_ONLY_EVENTS)]
                   + [level_event(0, 1009, (1, 64, 1), 0), level_event(1, 1009, (1, 64, 1), 1),
                      level_event(1, 1009, (1, 64, 1), 2), level_event(2, 1999, (0, 64, 0), 5)]),
        # A note block struck plays its instrument: a tuned one pitched by its note with a note rising from it,
        # a mob head's at its own pitch, a custom head's not at all (no head above); any other block nothing.
        engine_row("note-blocks", [block_event(index % 2, pos, 0, 0) for index, (pos, _, _) in enumerate(NOTE_BLOCKS)]
                   + [block_event(2, (0, 64, 0)), block_event(2, (0, 64, 0)), block_event(2, (9, 64, 0))],
                   blocks=[note_block(*block) for block in NOTE_BLOCKS]
                   + [{"x": 9, "y": 64, "z": 0, "block": "minecraft:stone"}],
                   ticks=5, seeds=12, describe=8),
        engine_row("level-event-smoke", [level_event(0, 1501, (2, 63, 2)), level_event(0, 1502, (-3, 64, 1)),
                                         level_event(1, 1503, (0, 64, -4)), level_event(1, 2008, (4, 64, 4)),
                                         level_event(2, 2009, (-2, 64, -2))],
                   ticks=6, seeds=45, describe=12),
        engine_row("level-event-decreased", [level_event(0, 1501, (2, 63, 2)), level_event(0, 1502, (-3, 64, 1))],
                   status="DECREASED", ticks=4, seeds=15, describe=6),
        engine_row("level-event-shoot", [level_event(0, 2000, (1, 64, 1), face) for face in range(6)]
                   + [level_event(1, 2010, (-1, 64, -1), 9), level_event(1, 2000, (0, 64, 3), -4)],
                   ticks=4, seeds=80, describe=16),
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


def pickup_row(label, item, targets, camera=(0.0, 65.0, 0.0)):
    return {
        "label": label, "blocks": [], "chunk_radius": 1,
        "x": float_bits(item[0], "f64"), "y": float_bits(item[1], "f64"), "z": float_bits(item[2], "f64"),
        "camera_x": float_bits(camera[0], "f64"), "camera_y": float_bits(camera[1], "f64"),
        "camera_z": float_bits(camera[2], "f64"),
        "targets": [{"x": float_bits(x, "f64"), "y": float_bits(y, "f64"), "z": float_bits(z, "f64"),
                     "eye": float_bits(eye)} for (x, y, z, eye) in targets],
        "partials": PARTIALS + [float_bits(0.3), float_bits(0.875)],
    }


def item_pickup_cases():
    """An entity taken flies to its taker over three ticks (the square of the time), following where
    the taker is: one standing, one walking off, one crouching and turning back; ticked past its end."""
    standing = [(0.5, 64.0, 0.5, 1.62)] * 5
    walking = [(2.3 + 0.21 * tick, 64.0, -1.7 + 0.13 * tick, 1.62) for tick in range(5)]
    crouching = [(-3.1, 63.5, 2.9, 1.62), (-3.0, 63.5, 2.8, 1.27), (-3.2, 63.25, 2.7, 1.27),
                 (-3.3, 63.0, 2.75, 1.27), (-3.3, 63.0, 2.75, 1.62)]
    rows = [
        pickup_row("standing", (1.25, 64.0, 0.75), standing),
        pickup_row("walking-off", (3.0, 64.5, -2.0), walking, camera=(1.5, 65.62, -1.0)),
        pickup_row("crouching", (-1.6, 63.0, 1.9), crouching, camera=(-3.2, 64.8, 2.8)),
    ]
    yield {"case_id": "movement/item-pickup", "family": "movement", "operation": "item_pickup", "rows": rows}
