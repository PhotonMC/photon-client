"""The `entity` family: aquatic."""

from cases.helpers import *


def entity_squid_cases():
    """The squid, grown and baby, in both the squid's and the glow squid's layers, posed by the
    reference's own `SquidModel.setupAnim` (every tentacle leans by the beat's angle); how its
    renderer turns the pose (the body's two turns, lower for a baby, no fall once dead); and the
    light a glow squid gives itself while it is dark."""
    rows = [{"label": f"{layer}-{angle}", "layer": f"minecraft:{layer}#main",
             "tentacle_angle": float_bits(angle)}
            for layer in ("squid", "squid_baby", "glow_squid", "glow_squid_baby")
            for angle in (0.0, 0.2, 0.785398, -0.4)]
    yield {"case_id": "entity/squid-model", "family": "entity", "operation": "squid_model",
           "rows": rows}
    turns = [{"label": f"squid-{'baby-' if baby else ''}{label}", "kind": "squid", "baby": baby,
              "body_rot": float_bits(body_rot), "x_rot": float_bits(0.0),
              "age_in_ticks": float_bits(10.5), "x_body_rot": float_bits(x),
              "z_body_rot": float_bits(z)}
             for baby in (False, True)
             for label, body_rot, x, z in (("still", 0.0, 0.0, 0.0), ("tipped", 37.0, -63.5, 0.0),
                                           ("spinning", -120.0, -89.9, 417.25),
                                           ("head-down", 12.0, -90.0, 1203.5))]
    yield {"case_id": "entity/squid-rotations", "family": "entity",
           "operation": "setup_rotations", "rows": turns}
    yield {"case_id": "entity/glow-squid-light", "family": "entity",
           "operation": "glow_squid_light",
           "rows": [{"label": f"dark-{dark}-light-{light}", "dark_ticks": dark, "block_light": light}
                    for dark in (0, 1, 3, 5, 9, 10, 11, 100) for light in (0, 7, 15)]}


def entity_fish_cases():
    """The fish, posed by the reference's own `setupAnim` of each model (the tail's, the back
    half's or the fins' swing, wider out of water); how their renderers turn the pose (the swing
    about y, the lying on the side out of water, the pufferfish's bob, each also once dead);
    the textures they wear and a tropical fish's by its packed variant."""
    layers = ["cod#main", "salmon#main", "salmon_small#main", "salmon_large#main",
              "tropical_fish_small#main", "tropical_fish_small#pattern",
              "tropical_fish_large#main", "tropical_fish_large#pattern",
              "pufferfish_small#main", "pufferfish_medium#main", "pufferfish_big#main"]
    rows = [{"label": f"{layer}-{age}-{'water' if in_water else 'land'}",
             "layer": f"minecraft:{layer}", "age_in_ticks": float_bits(age), "in_water": in_water}
            for layer in layers for age in (0.0, 3.5, 17.25, 101.0) for in_water in (True, False)]
    yield {"case_id": "entity/fish-model", "family": "entity", "operation": "fish_model",
           "rows": rows}
    turns = [{"label": f"{kind}-{label}", "kind": kind, "baby": False,
              "body_rot": float_bits(body_rot), "x_rot": float_bits(0.0),
              "age_in_ticks": float_bits(age), "in_water": in_water,
              "feet_x": float_bits(1.25), "feet_y": float_bits(64.5), "feet_z": float_bits(-3.75),
              "scale": float_bits(1.0),
              **({"death_time": float_bits(death)} if death else {})}
             for kind in ("cod", "salmon", "tropical_fish", "pufferfish")
             for label, body_rot, age, in_water, death in (
                 ("swimming", 37.0, 10.5, True, 0.0), ("swimming-later", -120.0, 61.25, True, 0.0),
                 ("on-land", 37.0, 10.5, False, 0.0), ("on-land-later", 200.0, 33.75, False, 0.0),
                 ("dying", 12.0, 21.5, True, 9.0), ("dying-on-land", 90.0, 21.5, False, 25.0))]
    yield {"case_id": "entity/fish-rotations", "family": "entity", "operation": "setup_rotations",
           "rows": turns}
    packed = [base | index << 8 for base in (0, 1) for index in range(6)]
    packed += [0x0600, 0x0002, 0x0101, 0xFFFF, 0x10100, 0x0E030500, 0x11C80000, -1]
    yield {"case_id": "entity/fish-textures", "family": "entity", "operation": "fish_textures",
           "rows": [{"label": name} for name in ("cod", "salmon", "pufferfish")]
           + [{"label": f"packed-{value & 0xFFFFFFFF:x}", "packed_id": value} for value in packed]}


def entity_sea_animal_cases():
    """The dolphin, grown and baby, posed by the reference's own `DolphinModel.setupAnim` (the body
    takes the turn and pitch; while it moves it undulates), and where its layer carries an item;
    the turtle, grown and baby, posed by `TurtleModel.setupAnim` (rowing on land, faster when
    laying; paddling in water; the egg on an adult's belly)."""
    rows = [{"label": f"{layer}-{label}", "layer": f"minecraft:{layer}#main",
             "y_rot": float_bits(y_rot), "x_rot": float_bits(x_rot),
             "age_in_ticks": float_bits(age), "is_moving": moving}
            for layer in ("dolphin", "dolphin_baby")
            for label, y_rot, x_rot, age, moving in (
                ("still", 0.0, 0.0, 12.5, False), ("turned", 33.5, -17.25, 12.5, False),
                ("swimming", 33.5, -17.25, 12.5, True), ("swimming-later", -80.0, 41.0, 77.75, True))]
    yield {"case_id": "entity/dolphin-model", "family": "entity", "operation": "sea_animal_model",
           "rows": rows}
    yield {"case_id": "entity/dolphin-item-pose", "family": "entity",
           "operation": "dolphin_item_pose",
           "rows": [{"label": f"pitch-{x_rot}", "x_rot": float_bits(x_rot)}
                    for x_rot in (0.0, -0.0, 10.0, -10.0, 60.0, -60.0, 90.0, -90.0, 33.3)]}
    rows = [{"label": f"{layer}-{label}", "layer": f"minecraft:{layer}#main",
             "head_y_rot": float_bits(25.0), "x_rot": float_bits(-12.5),
             "walk_position": float_bits(position), "walk_speed": float_bits(speed),
             "on_land": on_land, "laying_egg": laying, "has_egg": egg}
            for layer in ("turtle", "turtle_baby")
            for label, position, speed, on_land, laying, egg in (
                ("standing", 0.0, 0.0, True, False, False),
                ("walking", 3.7, 0.6, True, False, False),
                ("laying", 3.7, 0.6, True, True, False),
                ("swimming", 3.7, 0.6, False, False, False),
                ("swimming-faster", 11.2, 1.0, False, False, False),
                ("with-an-egg", 3.7, 0.6, True, False, True),
                ("with-an-egg-swimming", 5.1, 0.3, False, False, True))]
    yield {"case_id": "entity/turtle-model", "family": "entity", "operation": "sea_animal_model",
           "rows": rows}


def entity_axolotl_cases():
    """The axolotl posed by the reference's own `setupAnim`: an adult from the eased factors of
    swimming, hovering, crawling, lying still and playing dead (alone and blended, with the legs
    mirrored by what is left); a baby from the keyframe animations whose states have started
    (alone, and the walk that also runs the underwater walk)."""
    def adult(label, age=37.5, y_rot=0.0, x_rot=0.0, **factors):
        row = {"label": f"adult-{label}", "layer": "minecraft:axolotl#main", "y_rot": float_bits(y_rot),
               "x_rot": float_bits(x_rot), "walk_position": float_bits(0.0),
               "walk_speed": float_bits(0.0), "age_in_ticks": float_bits(age)}
        for name in ("playing_dead", "moving", "in_water", "on_ground"):
            row[name] = float_bits(factors.get(name, 0.0))
        return row
    rows = [adult("still"),
            adult("swimming", in_water=1.0, moving=1.0),
            adult("swimming-turned", y_rot=33.0, x_rot=-25.0, in_water=1.0, moving=1.0),
            adult("hovering", in_water=1.0),
            adult("crawling", on_ground=1.0, moving=1.0),
            adult("lying-still", on_ground=1.0),
            adult("playing-dead", playing_dead=1.0),
            adult("swimming-half", age=91.25, in_water=0.5, moving=0.5),
            adult("hovering-into-lying", age=12.0, in_water=0.3, on_ground=0.7),
            adult("crawling-into-swimming", age=55.5, in_water=0.4, on_ground=0.6, moving=0.8),
            adult("dead-while-swimming", age=20.0, in_water=0.6, moving=1.0, playing_dead=0.4),
            adult("a-sliver-of-a-factor", in_water=0.000_005, moving=0.9)]

    def baby(label, age=37.5, walk_position=0.0, walk_speed=0.0, **started):
        row = {"label": f"baby-{label}", "layer": "minecraft:axolotl_baby#main",
               "y_rot": float_bits(0.0), "x_rot": float_bits(0.0),
               "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
               "age_in_ticks": float_bits(age)}
        for name in ("playing_dead", "moving", "in_water", "on_ground"):
            row[name] = float_bits(0.0)
        row.update(started)
        return row
    rows += [baby("nothing"),
             baby("swim", swim=3), baby("swim-later", age=90.0, swim=3),
             baby("walk", walk_position=4.5, walk_speed=0.4, walk=5),
             baby("walk-fast", age=71.0, walk_position=9.0, walk_speed=1.0, walk=5),
             baby("idle-on-ground", idle_on_ground=2), baby("idle-underwater", idle_under_water=2),
             baby("idle-underwater-on-ground", idle_under_water_on_ground=2),
             baby("play-dead", play_dead=1), baby("play-dead-later", age=44.0, play_dead=1)]
    yield {"case_id": "entity/axolotl-model", "family": "entity", "operation": "axolotl_model",
           "rows": rows}


def entity_frog_cases():
    """The frog and the tadpole posed by the reference's own `setupAnim`: a frog walking (or
    swimming) by its stride, the speed capped at one and the animations whose states have started
    (the jump, croak and tongue end, the idle float loops; the throat shows while it croaks); a
    tadpole's tail swing, wider out of water."""
    def frog(label, age=37.5, walk_position=0.0, walk_speed=0.0, swimming=False, **started):
        row = {"label": f"frog-{label}", "layer": "minecraft:frog#main",
               "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
               "age_in_ticks": float_bits(age), "swimming": swimming}
        row.update(started)
        return row
    rows = [frog("still"),
            frog("walking", walk_position=4.5, walk_speed=0.3),
            frog("walking-fast", walk_position=9.0, walk_speed=1.0),
            frog("swimming", walk_position=3.0, walk_speed=0.3, swimming=True),
            frog("swimming-fast", walk_position=7.5, walk_speed=0.9, swimming=True),
            frog("jump", age=12.5, jump=10), frog("jump-over", age=40.0, jump=10),
            frog("croak", age=40.0, croak=10), frog("croak-over", age=100.0, croak=10),
            frog("tongue", age=14.5, tongue=10), frog("tongue-over", age=40.0, tongue=10),
            frog("floating", age=33.0, swim_idle=10), frog("floating-looped", age=100.0, swim_idle=10),
            frog("jump-walking", age=12.5, walk_position=4.5, walk_speed=0.3, jump=10),
            frog("croak-swimming", age=40.0, walk_position=3.0, walk_speed=0.3, swimming=True,
                 croak=10),
            frog("floating-swimming", age=33.0, walk_position=3.0, walk_speed=0.0, swimming=True,
                 swim_idle=10)]
    for label, in_water in (("in-water", True), ("out-of-water", False)):
        rows.append({"label": f"tadpole-{label}", "layer": "minecraft:tadpole#main",
                     "age_in_ticks": float_bits(37.5), "in_water": in_water})
    yield {"case_id": "entity/frog-model", "family": "entity", "operation": "frog_model",
           "rows": rows}


def entity_nautilus_cases():
    """The nautilus models posed by the reference's own `setupAnim`: the body following the look
    within ten degrees (inside, past, either side), the swim animation by the stride and the age
    (a stride of nothing still swims, a fast one is capped), a zombie nautilus alike, and its corals
    shown unless body armor is worn."""
    def nautilus(layer, label, age=37.5, y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
                 armored=False):
        return {"label": f"{layer.split(':')[1].split('#')[0]}-{label}", "layer": layer,
                "y_rot": float_bits(y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "age_in_ticks": float_bits(age), "armored": armored}
    rows = []
    for layer in ("minecraft:nautilus#main", "minecraft:nautilus_baby#main"):
        rows += [nautilus(layer, "still"), nautilus(layer, "later", age=91.25),
                 nautilus(layer, "looks-a-little", y_rot=6.5, x_rot=-4.0),
                 nautilus(layer, "looks-past-the-limit", y_rot=-48.0, x_rot=35.0),
                 nautilus(layer, "swims", walk_position=4.5, walk_speed=0.3),
                 nautilus(layer, "swims-fast", age=12.0, walk_position=9.0, walk_speed=1.0)]
    for layer in ("minecraft:nautilus#saddle", "minecraft:nautilus_armor#main"):
        rows += [nautilus(layer, "still"),
                 nautilus(layer, "looks-and-swims", y_rot=-7.0, x_rot=12.0, walk_position=3.0,
                          walk_speed=0.5, age=91.25)]
    for layer in ("minecraft:zombie_nautilus#main", "minecraft:zombie_nautilus_coral#main"):
        rows += [nautilus(layer, "still"),
                 nautilus(layer, "looks-and-swims", y_rot=-7.0, x_rot=12.0, walk_position=3.0,
                          walk_speed=0.5),
                 nautilus(layer, "armored", armored=True)]
    yield {"case_id": "entity/nautilus-model", "family": "entity", "operation": "nautilus_model",
           "rows": rows}


def entity_guardian_cases():
    """The guardian posed by the reference's own `GuardianModel.setupAnim`: the head's look, the
    spikes drawn out as they breathe (and withdrawn), the tail swaying with its animation, and the
    eye following what it looks at (above or below it, to either side, straight on, or at its own
    place), for a guardian and an elder (whose scaled layer the spikes' places ignore)."""
    def guardian(layer, label, age=37.5, y_rot=0.0, x_rot=0.0, spikes=1.0, tail=0.0, eye=None):
        row = {"label": f"{layer.split(':')[1].split('#')[0]}-{label}", "layer": layer,
               "y_rot": float_bits(y_rot), "x_rot": float_bits(x_rot),
               "age_in_ticks": float_bits(age), "spikes": float_bits(spikes),
               "tail": float_bits(tail)}
        if eye:
            row["eye"] = {key: [float_bits(part, "f64") for part in value]
                          for key, value in eye.items()}
        return row

    def looking(at, direction=(0.0, 0.0, 1.0), eye=(0.0, 1.0, 0.0)):
        return {"eye_position": eye, "look_direction": direction, "look_at_position": at}
    rows = []
    for layer in ("minecraft:guardian#main", "minecraft:elder_guardian#main"):
        rows += [guardian(layer, "rests"), guardian(layer, "later", age=91.25),
                 guardian(layer, "withdraws-its-spikes", spikes=0.0),
                 guardian(layer, "half-withdrawn", age=12.0, spikes=0.4),
                 guardian(layer, "looks", y_rot=30.0, x_rot=-15.0),
                 guardian(layer, "tail-sways", tail=1.3), guardian(layer, "tail-sways-back", tail=7.7),
                 guardian(layer, "eye-above", eye=looking((3.0, 4.0, 5.0))),
                 guardian(layer, "eye-below", eye=looking((-2.0, -1.5, 4.0))),
                 guardian(layer, "eye-level", eye=looking((1.0, 1.0, 6.0))),
                 guardian(layer, "eye-to-the-left", eye=looking((6.0, 2.0, 1.0), (1.0, 0.0, 0.5))),
                 guardian(layer, "eye-straight-on", eye=looking((0.0, 2.0, 5.0), (0.0, 0.0, 1.0))),
                 guardian(layer, "eye-at-its-own-place", eye=looking((0.0, 3.0, 0.0)))]
    yield {"case_id": "entity/guardian-model", "family": "entity", "operation": "guardian_model",
           "rows": rows}


def entity_guardian_beam_cases():
    """A guardian's beam as the reference's own `renderBeam` builds it from the pose its eye leaves:
    along each axis and slantwise, short and long (the texture repeating along it), across the
    attack charging (the colour, the spin and the cap's texture swapping on even and odd ticks) and
    from an offset eye."""
    def beam(label, vector, time=12.5, scale=0.3, at=(0.0, 0.0, 0.0)):
        return {"label": label, "x": float_bits(at[0]), "y": float_bits(at[1]), "z": float_bits(at[2]),
                "vector": [float_bits(part, "f64") for part in vector], "time": float_bits(time),
                "scale": float_bits(scale), "tex_v_off": float_bits(time * 0.5 % 1.0)}
    rows = [beam("along-z", (0.0, 0.0, 6.0)), beam("along-x", (5.0, 0.0, 0.0)),
            beam("straight-up", (0.0, 4.0, 0.0)), beam("straight-down", (0.0, -4.0, 0.0)),
            beam("slantwise", (3.0, -1.5, 4.5)), beam("behind", (-2.0, 0.5, -7.0)),
            beam("short", (0.2, 0.1, 0.3)), beam("on-the-eye", (0.0, 0.0, 0.0)),
            beam("just-started", (0.0, 0.0, 6.0), time=0.5, scale=0.006),
            beam("odd-tick", (1.0, 2.0, 3.0), time=7.25, scale=0.09),
            beam("charged", (1.0, 2.0, 3.0), time=80.0, scale=1.0),
            beam("elder-charged", (1.0, 2.0, 3.0), time=60.0, scale=1.0),
            beam("from-the-eye", (2.0, -1.0, 3.0), at=(10.5, 64.9, -3.25)),
            beam("far", (20.0, 6.0, -30.0), time=33.75, scale=0.17)]
    yield {"case_id": "entity/guardian-beam", "family": "entity", "operation": "guardian_beam",
           "rows": rows}
