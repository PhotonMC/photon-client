"""The `entity` family: bosses."""

import math
from cases.helpers import *


def entity_cube_mob_cases():
    """The magma cube posed by the reference's own `MagmaCubeModel.setupAnim` (its slices lift apart
    by the squish, none while it is squashed), and the pose both cube mobs' renderers leave: their
    size and squish scale it (the slime's drawn a hair smaller first)."""
    squishes = (-0.5, -0.1, 0.0, 0.3, 1.0, 1.7)
    yield {"case_id": "entity/magma-cube-model", "family": "entity", "operation": "magma_cube_model",
           "rows": [{"label": f"magma-cube-{squish}", "layer": "minecraft:magma_cube#main",
                     "squish": float_bits(squish)} for squish in squishes]}
    rows = [{"label": f"{kind}-size-{size}-squish-{squish}", "kind": kind, "baby": False,
             "body_rot": float_bits(37.0), "x_rot": float_bits(0.0),
             "age_in_ticks": float_bits(10.5), "size": size, "squish": float_bits(squish),
             "feet_x": float_bits(3.5), "feet_y": float_bits(-1.25), "feet_z": float_bits(0.75),
             "scale": float_bits(1.0)}
            for kind in ("slime", "magma_cube") for size in (1, 2, 4, 127)
            for squish in (-0.5, 0.0, 0.35, 1.0)]
    rows += [{"label": f"sulfur-cube-{'small-' if baby else ''}size-{size}-fuse-{fuse}",
              "kind": "sulfur_cube", "baby": baby, "body_rot": float_bits(37.0),
              "x_rot": float_bits(0.0), "age_in_ticks": float_bits(10.5), "size": size,
              "squish": float_bits(squish), "fuse_remaining": float_bits(fuse),
              "feet_x": float_bits(3.5), "feet_y": float_bits(-1.25), "feet_z": float_bits(0.75),
              "scale": float_bits(1.0)}
             for baby, size in ((False, 2), (True, 1), (False, 3))
             for squish in (-0.5, 0.4) for fuse in (0.0, 1.5, 6.25, 9.9, 10.0, 30.0)]
    # A cube that holds a block: no squish, and the block put where `SulfurCubeInnerLayer` draws it.
    rows += [{"label": f"sulfur-block-{'baby' if baby else 'grown'}-size-{size}-fuse-{fuse}",
              "kind": "sulfur_cube", "baby": baby, "body_rot": float_bits(37.0),
              "x_rot": float_bits(0.0), "age_in_ticks": float_bits(10.5), "size": size,
              "squish": float_bits(0.4), "fuse_remaining": float_bits(fuse),
              "contained_block": True, "feet_x": float_bits(3.5), "feet_y": float_bits(-1.25),
              "feet_z": float_bits(0.75), "scale": float_bits(1.0)}
             for baby, size in ((False, 2), (True, 1)) for fuse in (0.0, 6.25)]
    # The overlay the block is submitted with: hurt, and a lit fuse (which wins over hurt).
    rows += [{"label": f"sulfur-block-overlay-{name}", "kind": "sulfur_cube", "baby": False,
              "body_rot": float_bits(37.0), "x_rot": float_bits(0.0),
              "age_in_ticks": float_bits(10.5), "size": 2, "squish": float_bits(0.4),
              "fuse_remaining": float_bits(fuse), "contained_block": True, **extra}
             for name, fuse, extra in (("hurt", 0.0, {"red_overlay": True}),
                                       ("lit", 1.5, {}),
                                       ("lit-and-hurt", 1.5, {"red_overlay": True}),
                                       ("fuse-between-flashes", 6.25, {"red_overlay": True}))]
    rows.append({"label": "slime-dying", "kind": "slime", "baby": False,
                 "body_rot": float_bits(-80.0), "x_rot": float_bits(0.0),
                 "age_in_ticks": float_bits(3.0), "size": 3, "squish": float_bits(0.2),
                 "death_time": float_bits(7.5)})
    yield {"case_id": "entity/cube-mob-rotations", "family": "entity",
           "operation": "setup_rotations", "rows": rows}


def entity_shulker_cases():
    """The shulker posed by the reference's own `ShulkerModel.setupAnim` (the lid's rise, its twist
    past a third open, its bob once the lid is more than open, and the head's turn from the body),
    and the turns its renderer leaves for each face it can be attached to."""
    rows = [{"label": f"shulker-peek-{peek}-head-{head}", "layer": "minecraft:shulker#main",
             "peek_amount": float_bits(peek), "age_in_ticks": float_bits(37.25),
             "x_rot": float_bits(-12.5), "y_head_rot": float_bits(head),
             "y_body_rot": float_bits(0.0)}
            for peek in (0.0, 0.15, 0.3, 0.31, 0.5, 0.75, 1.0) for head in (0.0, 90.0, -137.5)]
    yield {"case_id": "entity/shulker-model", "family": "entity", "operation": "shulker_model",
           "rows": rows}
    turns = [{"label": f"shulker-face-{face}-body-{body_rot}", "kind": "shulker", "baby": False,
              "body_rot": float_bits(body_rot), "x_rot": float_bits(0.0),
              "age_in_ticks": float_bits(10.5), "attach_face": face,
              "feet_x": float_bits(3.5), "feet_y": float_bits(-1.25), "feet_z": float_bits(0.75),
              "scale": float_bits(1.0)}
             for face in range(6) for body_rot in (0.0, 37.0)]
    turns.append({"label": "shulker-dying", "kind": "shulker", "baby": False,
                  "body_rot": float_bits(20.0), "x_rot": float_bits(0.0),
                  "age_in_ticks": float_bits(3.0), "attach_face": 4,
                  "death_time": float_bits(7.5), "feet_x": float_bits(3.5),
                  "feet_y": float_bits(-1.25), "feet_z": float_bits(0.75),
                  "scale": float_bits(1.0)})
    yield {"case_id": "entity/shulker-rotations", "family": "entity",
           "operation": "setup_rotations", "rows": turns}


def entity_warden_cases():
    """The warden's five models (the body, the bioluminescent parts, the pulsating spots, the
    tendrils, the heart) posed by the reference's own `WardenModel.setupAnim`: the look, the walk
    (slow, fast, past the speed it stops growing at), the tendrils' pulse, each of the six animations
    alone (early, late and run out), and some together. The layers differ only in which parts they
    keep, so all but the body run a shorter list."""
    def warden(layer, label, age=37.5, look=(0.0, 0.0), walk=(0.0, 0.0), tendril=0.0, **started):
        row = {"label": f"warden-{layer}-{label}", "layer": f"minecraft:warden#{layer}",
               "age_in_ticks": float_bits(age), "y_rot": float_bits(look[0]),
               "x_rot": float_bits(look[1]), "walk_position": float_bits(walk[0]),
               "walk_speed": float_bits(walk[1]), "tendril": float_bits(tendril)}
        row.update(started)
        return row
    rows = []
    for layer in ("main", "bioluminescent", "pulsating_spots", "tendrils", "heart"):
        rows += [warden(layer, "nothing"),
                 warden(layer, "look-walk-tendril", age=91.25, look=(23.5, -14.0), walk=(6.3, 0.12),
                        tendril=0.6),
                 warden(layer, "emerge", age=60.0, emerge=20)]
        if layer != "main":
            continue
        rows += [warden(layer, "look", look=(-31.0, 17.5)),
                 warden(layer, "slow-walk", walk=(2.7, 0.04)),
                 warden(layer, "fast-walk", walk=(11.9, 0.31)),
                 warden(layer, "fastest-walk", age=12.0, walk=(30.1, 1.0)),
                 warden(layer, "tendril", age=12.5, tendril=1.0),
                 warden(layer, "attack", age=45.0, attack=40),
                 warden(layer, "attack-over", age=120.0, attack=40),
                 warden(layer, "sonic-boom", age=70.0, sonic_boom=50),
                 warden(layer, "sonic-boom-late", age=105.0, sonic_boom=50),
                 warden(layer, "dig", age=64.0, digging=40),
                 warden(layer, "dig-late", age=140.0, digging=40),
                 warden(layer, "emerge-late", age=150.0, emerge=20),
                 warden(layer, "roar", age=55.0, roar=40),
                 warden(layer, "roar-late", age=100.0, roar=40),
                 warden(layer, "sniff", age=50.0, sniff=40),
                 warden(layer, "sniff-late", age=80.0, sniff=40),
                 warden(layer, "roar-then-attack", age=70.0, look=(10.0, 5.0), roar=40, attack=65),
                 warden(layer, "walking-roar", age=55.0, walk=(8.1, 0.2), roar=40)]
    yield {"case_id": "entity/warden-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_creaking_cases():
    """The creaking's two models (the body, the eyes) posed by the reference's own
    `CreakingModel.setupAnim`: the look, the walk (shown only while it can move, so with it
    frozen as well), each of the three animations alone (early, late and run out), and some
    together."""
    def creaking(layer, label, age=37.5, look=(0.0, 0.0), walk=(0.0, 0.0), can_move=True, **started):
        row = {"label": f"creaking-{layer}-{label}", "layer": f"minecraft:creaking#{layer}",
               "age_in_ticks": float_bits(age), "y_rot": float_bits(look[0]),
               "x_rot": float_bits(look[1]), "walk_position": float_bits(walk[0]),
               "walk_speed": float_bits(walk[1]), "can_move": can_move}
        row.update(started)
        return row
    rows = []
    for layer in ("main", "eyes"):
        rows += [creaking(layer, "nothing"),
                 creaking(layer, "look-walk", age=91.25, look=(23.5, -14.0), walk=(6.3, 0.62)),
                 creaking(layer, "attack", age=45.0, attack=40)]
        if layer != "main":
            continue
        rows += [creaking(layer, "look", look=(-31.0, 17.5)),
                 creaking(layer, "slow-walk", walk=(2.7, 0.1)),
                 creaking(layer, "fast-walk", walk=(11.9, 0.9)),
                 creaking(layer, "fastest-walk", walk=(30.1, 3.0)),
                 creaking(layer, "frozen-walk", walk=(11.9, 0.9), can_move=False),
                 creaking(layer, "attack-over", age=120.0, attack=40),
                 creaking(layer, "invulnerable", age=62.0, invulnerable=60),
                 creaking(layer, "invulnerable-over", age=90.0, invulnerable=60),
                 creaking(layer, "death", age=70.0, death=50),
                 creaking(layer, "death-late", age=150.0, death=50),
                 creaking(layer, "walking-attack", age=55.0, walk=(8.1, 0.5), attack=50),
                 creaking(layer, "frozen-death", age=80.0, can_move=False, death=50, attack=70)]
    yield {"case_id": "entity/creaking-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_armadillo_cases():
    """The armadillo's two models (adult, baby) posed by the reference's own
    `ArmadilloModel.setupAnim`: the look inside its limits, the walk (and not while it hides in its
    shell), the ball and the body it hides behind, and each of the three animations alone and with
    others."""
    def armadillo(layer, label, age=37.5, look=(0.0, 0.0), walk=(0.0, 0.0), hiding=False, **started):
        row = {"label": f"armadillo-{layer}-{label}", "layer": f"minecraft:{layer}#main",
               "age_in_ticks": float_bits(age), "y_rot": float_bits(look[0]),
               "x_rot": float_bits(look[1]), "walk_position": float_bits(walk[0]),
               "walk_speed": float_bits(walk[1]), "hiding": hiding}
        row.update(started)
        return row
    rows = []
    for layer in ("armadillo", "armadillo_baby"):
        rows += [armadillo(layer, "nothing"),
                 armadillo(layer, "look", look=(20.0, -10.0)),
                 armadillo(layer, "look-past-limits", look=(-70.0, 60.0)),
                 armadillo(layer, "look-past-other-limits", look=(70.0, -60.0)),
                 armadillo(layer, "walk", walk=(6.3, 0.2)),
                 armadillo(layer, "fast-walk", walk=(11.9, 0.9)),
                 armadillo(layer, "hiding", look=(20.0, -10.0), walk=(6.3, 0.2), hiding=True),
                 armadillo(layer, "roll-up", age=40.0, roll_up=36, hiding=True),
                 armadillo(layer, "roll-up-over", age=70.0, roll_up=36, hiding=True),
                 armadillo(layer, "peek", age=80.0, peek=50, hiding=True),
                 armadillo(layer, "peek-late", age=160.0, peek=50, hiding=True),
                 armadillo(layer, "roll-out", age=75.0, roll_out=60),
                 armadillo(layer, "roll-out-late", age=100.0, roll_out=60),
                 armadillo(layer, "rolling-out-walking", age=70.0, walk=(8.1, 0.5), roll_out=60)]
    yield {"case_id": "entity/armadillo-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_sniffer_cases():
    """The sniffer's two models (adult, snifflet) posed by the reference's own `SnifferModel.setupAnim`:
    the look, the walk and the searching walk, and each of the five animations alone (early, late and
    run out) and with the walk."""
    def sniffer(layer, label, age=37.5, look=(0.0, 0.0), walk=(0.0, 0.0), searching=False, **started):
        row = {"label": f"sniffer-{layer}-{label}", "layer": f"minecraft:{layer}#main",
               "age_in_ticks": float_bits(age), "y_rot": float_bits(look[0]),
               "x_rot": float_bits(look[1]), "walk_position": float_bits(walk[0]),
               "walk_speed": float_bits(walk[1]), "searching": searching}
        row.update(started)
        return row
    rows = []
    for layer in ("sniffer", "sniffer_baby"):
        rows += [sniffer(layer, "nothing"),
                 sniffer(layer, "look", look=(20.0, -10.0)),
                 sniffer(layer, "walk", walk=(6.3, 0.02)),
                 sniffer(layer, "fast-walk", walk=(11.9, 0.3)),
                 sniffer(layer, "searching", walk=(6.3, 0.02), searching=True),
                 sniffer(layer, "searching-still", age=44.0, searching=True),
                 sniffer(layer, "dig", age=60.0, digging=40),
                 sniffer(layer, "dig-late", age=240.0, digging=40),
                 sniffer(layer, "sniff", age=52.0, sniffing=40),
                 sniffer(layer, "sniff-over", age=100.0, sniffing=40),
                 sniffer(layer, "rise", age=70.0, rising=40),
                 sniffer(layer, "rise-over", age=140.0, rising=40),
                 sniffer(layer, "happy", age=70.0, happy=40),
                 sniffer(layer, "scenting", age=70.0, scenting=40),
                 sniffer(layer, "scenting-late", age=260.0, scenting=40),
                 sniffer(layer, "digging-walking", age=70.0, walk=(8.1, 0.05), digging=40)]
    yield {"case_id": "entity/sniffer-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_wither_cases():
    """The wither's two models (the body, the armor) posed by the reference's own
    `WitherBossModel.setupAnim`: the middle head's look, the side heads' turns relative to the body,
    and the ribcage's and the tail's sway over the age."""
    def wither(layer, label, age=37.5, look=(0.0, 0.0), body=0.0, heads=((0.0, 0.0), (0.0, 0.0))):
        return {"label": f"wither-{layer}-{label}", "layer": f"minecraft:wither#{layer}",
                "age_in_ticks": float_bits(age), "y_rot": float_bits(look[0]),
                "x_rot": float_bits(look[1]), "body_rot": float_bits(body),
                "x_head_0": float_bits(heads[0][0]), "y_head_0": float_bits(heads[0][1]),
                "x_head_1": float_bits(heads[1][0]), "y_head_1": float_bits(heads[1][1])}
    rows = []
    for layer in ("main", "armor"):
        rows += [wither(layer, "nothing"),
                 wither(layer, "later", age=201.75),
                 wither(layer, "look", look=(23.5, -14.0)),
                 wither(layer, "heads", body=40.0, heads=((12.0, 70.0), (-25.0, 10.0))),
                 wither(layer, "everything", age=91.25, look=(-31.0, 17.5), body=-120.0,
                        heads=((30.0, -100.0), (-5.0, 45.5)))]
    yield {"case_id": "entity/wither-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_ender_dragon_cases():
    """The ender dragon's model posed by the reference's own `EnderDragonModel.setupAnim`: from a
    flight history of few and of many samples (the ring wraps after sixty-four), climbing or
    descending and turning one way or the other, at some points of its wing beat and between two
    ticks, flying, landing or taking off (the neck follows the distance to the egg), and sitting."""
    import math

    def history(count, climb, turn, start_y=70.0, start_rot=-170.0):
        samples = []
        for i in range(count):
            y = start_y + climb * i + 0.3 * math.sin(i * 0.4)
            rot = ((start_rot + turn * i + 180.0) % 360.0) - 180.0
            samples.append({"y": float_bits(y, "f64"), "y_rot": float_bits(rot)})
        return samples

    def dragon(label, samples, flap=0.37, partial=0.0, landing=False, sitting=False, egg=2500.0):
        return {"label": f"dragon-{label}", "layer": "minecraft:ender_dragon#main",
                "flap_time": float_bits(flap), "partial_ticks": float_bits(partial),
                "landing_or_taking_off": landing, "sitting": sitting,
                "distance_to_egg": float_bits(egg, "f64"), "history": samples}
    rows = [dragon("one-sample", history(1, 0.0, 0.0)),
            dragon("straight", history(30, 0.0, 0.0)),
            dragon("climbs-and-turns-left", history(40, 0.35, 6.5), flap=1.62, partial=0.5),
            dragon("descends-and-turns-right", history(50, -0.5, -9.25, start_y=95.0), flap=2.9, partial=0.25),
            dragon("wraps-the-ring", history(150, 0.05, 3.1), flap=7.123, partial=1.0),
            dragon("wraps-the-ring-between-ticks", history(130, -0.12, -2.7), flap=0.0, partial=0.7),
            dragon("flaps-up", history(45, 0.2, 4.0), flap=0.2),
            dragon("flaps-down", history(45, 0.2, 4.0), flap=0.7),
            dragon("lands-far-from-the-egg", history(45, -0.4, 2.0), landing=True, egg=3600.0),
            dragon("lands-near-the-egg", history(45, -0.4, 2.0), landing=True, egg=9.0),
            dragon("sits", history(45, 0.0, 0.0), sitting=True, flap=0.9),
            dragon("sits-and-turns", history(45, 0.0, 5.0), sitting=True)]
    yield {"case_id": "entity/ender-dragon-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_dragon_rays_cases():
    """The rays of a dying ender dragon as the reference's own `submitRays` builds them: few rays and
    many (the count grows with the square of the death time), through the last fifth of it (the
    rays lengthen and widen, the middle fades) and past the end, from an offset pose."""
    def rays(label, death_time, at=(0.0, -1.0, -2.0)):
        return {"label": label, "x": float_bits(at[0]), "y": float_bits(at[1]),
                "z": float_bits(at[2]), "death_time": float_bits(death_time)}
    rows = [rays("none", 0.0), rays("barely", 0.02), rays("few", 0.1), rays("some", 0.4),
            rays("half", 0.5), rays("three-quarters", 0.75), rays("at-the-edge", 0.8),
            rays("overdriving", 0.9), rays("nearly-done", 0.99), rays("done", 1.0),
            rays("over", 1.04), rays("from-an-offset", 0.33, at=(10.5, 64.9, -3.25))]
    yield {"case_id": "entity/dragon-rays", "family": "entity", "operation": "dragon_rays",
           "rows": rows}


def entity_end_crystal_cases():
    """The end crystal's model posed by the reference's own `EndCrystalModel.setupAnim`: at ages
    through the first beats and far on (the glass bobs, the three cubes turn about a tilted axis),
    with its base and without."""
    rows = []
    for age in (0.0, 0.5, 3.25, 10.0, 17.7, 31.4, 100.125, 1234.5, 98765.25):
        for bottom in (True, False):
            rows.append({"label": f"crystal-{age}-{'base' if bottom else 'no-base'}",
                         "layer": "minecraft:end_crystal#main", "age_in_ticks": float_bits(age),
                         "shows_bottom": bottom})
    yield {"case_id": "entity/end-crystal-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_crystal_beam_cases():
    """An end crystal's beam as the reference's own `submitCrystalBeams` builds it from the pose it is
    submitted at: along each axis and slantwise, short and long (the texture running along it),
    over the ticks (the texture scrolling) and from an offset pose, with some light."""
    def beam(label, delta, time=12.5, at=(0.0, 0.0, 0.0), light=15728880):
        return {"label": label, "x": float_bits(at[0]), "y": float_bits(at[1]), "z": float_bits(at[2]),
                "dx": float_bits(delta[0]), "dy": float_bits(delta[1]), "dz": float_bits(delta[2]),
                "time": float_bits(time), "light": light}
    rows = [beam("along-z", (0.0, 0.0, 6.0)), beam("along-x", (5.0, 0.0, 0.0)),
            beam("straight-up", (0.0, 9.0, 0.0)), beam("straight-down", (0.0, -4.0, 0.0)),
            beam("slantwise", (3.0, -1.5, 4.5)), beam("behind", (-2.0, 0.5, -7.0)),
            beam("short", (0.2, 0.1, 0.3)), beam("at-the-start", (0.0, 0.0, 0.0)),
            beam("long", (20.0, 30.0, -40.0), time=3.0), beam("just-started", (4.0, 8.0, 1.0), time=0.0),
            beam("later", (4.0, 8.0, 1.0), time=1234.5), beam("lit", (4.0, 8.0, 1.0), light=240),
            beam("from-an-offset", (2.0, 5.0, -3.0), at=(10.5, 64.9, -3.25))]
    yield {"case_id": "entity/crystal-beam", "family": "entity", "operation": "crystal_beam",
           "rows": rows}
