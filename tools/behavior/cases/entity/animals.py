"""The `entity` family: animals."""

from cases.helpers import *


def entity_sheep_cases():
    """A sheep's models from the layer table (the body, the undercoat and the
    wool, grown and baby), posed by the reference's `SheepModel`,
    `BabySheepModel` and `SheepFurModel` `setupAnim`: standing, looking,
    walking, with the head down and tipped while eating."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             eat_position=0.0, eat_angle=None, age_scale=1.0):
        # Not eating, the angle is the pitch.
        angle = x_rot * 0.017453292 if eat_angle is None else eat_angle
        return {"label": f"{layer}-{label}", "layer": layer,
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "eat_position": float_bits(eat_position), "eat_angle": float_bits(angle),
                "age_scale": float_bits(age_scale)}

    rows = []
    for layer, scale in (("minecraft:sheep#main", 1.0), ("minecraft:sheep#wool", 1.0),
                         ("minecraft:sheep#wool_undercoat", 1.0), ("minecraft:sheep_baby#main", 0.5),
                         ("minecraft:sheep_baby#wool", 0.5)):
        rows += [
            pose("standing", layer, age_scale=scale),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0, age_scale=scale),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83, age_scale=scale),
            pose("eating-down", layer, eat_position=1.0, eat_angle=0.62831855, age_scale=scale),
            pose("eating-rising", layer, eat_position=0.375, eat_angle=0.5, x_rot=20.0,
                 walk_position=2.0, walk_speed=0.2, age_scale=scale),
        ]
    yield {
        "case_id": "entity/sheep-model",
        "family": "entity",
        "operation": "sheep_model",
        "rows": rows,
    }


def entity_chicken_cases():
    """A chicken's models from the layer table (grown, cold and baby), posed by
    the reference's `ChickenModel.setupAnim` with the wings' flap, drawn."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             flap=0.0, flap_speed=0.0):
        return {"label": f"{layer}-{label}", "layer": layer,
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "flap": float_bits(flap), "flap_speed": float_bits(flap_speed)}

    rows = []
    for layer in ("minecraft:chicken#main", "minecraft:cold_chicken#main",
                  "minecraft:chicken_baby#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83),
            pose("flapping", layer, flap=3.7, flap_speed=1.0),
            pose("flapping-slowing", layer, flap=11.25, flap_speed=0.4, walk_position=2.0,
                 walk_speed=0.2, head_y_rot=12.0, x_rot=-8.0),
        ]
    yield {
        "case_id": "entity/chicken-model",
        "family": "entity",
        "operation": "chicken_model",
        "rows": rows,
    }


def entity_sheep_colour_cases():
    """What a sheep's wool is multiplied by: the reference's table for each of
    the sixteen dyes, and a `jeb_` sheep's colour along its cycle."""
    rows = [{"label": f"dye-{dye}", "dye": dye} for dye in range(16)]
    rows += [{"label": f"texture-dye-{dye}", "texture_dye": dye} for dye in range(16)]
    for tick in (0.0, 0.5, 12.25, 24.9, 25.0, 31.5, 100.1, 399.0, 400.0, 401.7, 1234.567):
        rows.append({"label": f"jeb-{tick}", "tick": float_bits(tick)})
    yield {
        "case_id": "entity/sheep-colors",
        "family": "entity",
        "operation": "sheep_colors",
        "rows": rows,
    }


def entity_sheep_eat_cases():
    """How far a sheep's head is down and how it is tipped while it eats, from
    the reference's own `Sheep` with its timer and pitch set."""
    rows = []
    for tick in (0, 1, 3, 4, 5, 20, 36, 37, 39, 40):
        for partial in (0.0, 0.25, 1.0):
            rows.append({"label": f"eat-{tick}-{partial}", "eat_tick": tick,
                         "partial": float_bits(partial), "x_rot": float_bits(17.5)})
    rows.append({"label": "pitch-down", "eat_tick": 0, "partial": float_bits(0.5),
                 "x_rot": float_bits(-45.0)})
    yield {
        "case_id": "entity/sheep-eat",
        "family": "entity",
        "operation": "sheep_eat",
        "rows": rows,
    }


def entity_rabbit_cases():
    """The rabbit, grown and baby, posed by the reference's own `RabbitModel.setupAnim` with its
    hop and idle head tilt animation states started at different ticks, and the rabbit's
    own tick: the jump event, the hop and idle states, the jump completion."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, age=0.0, hop=None, idle=None):
        row = {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
               "x_rot": float_bits(x_rot), "age_in_ticks": float_bits(age)}
        if hop is not None:
            row["hop_start"] = hop
        if idle is not None:
            row["idle_start"] = idle
        return row

    rows = []
    for layer in ("minecraft:rabbit#main", "minecraft:rabbit_baby#main"):
        rows += [
            pose("standing", layer, head_y_rot=-35.5, x_rot=48.0, age=3.25),
            pose("hopping-early", layer, head_y_rot=12.0, x_rot=-8.0, age=14.5, hop=10),
            pose("hopping-late", layer, age=40.0, hop=10),
            pose("idle-tilting", layer, head_y_rot=-35.5, x_rot=48.0, age=60.25, idle=20),
            pose("idle-tilting-late", layer, age=120.0, idle=20),
            pose("both", layer, head_y_rot=12.0, x_rot=-8.0, age=50.0, hop=30, idle=20),
        ]
    yield {"case_id": "entity/rabbit-model", "family": "entity", "operation": "rabbit_model", "rows": rows}


def entity_equine_cases():
    """The horse family from the layer table, posed by the reference's own
    `AbstractEquineModel.setupAnim` (and the baby and donkey overrides): standing,
    walking, galloping in water, grazing, rearing, the mouth open, the tail swishing,
    the donkey's chests; and the animal's own client tick (grazing, rearing and mouth
    animations from the flags the server sends, the tail from the animal's random)."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, age=0.0,
             eat=0.0, stand=0.0, feeding=0.0, tail=False, water=False, chest=False):
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "age_in_ticks": float_bits(age),
                "eat": float_bits(eat), "stand": float_bits(stand), "feeding": float_bits(feeding),
                "animate_tail": tail, "in_water": water, "has_chest": chest}

    rows = []
    layers = ("minecraft:horse#main", "minecraft:horse_baby#main", "minecraft:donkey#main",
              "minecraft:donkey_baby#main", "minecraft:mule#main", "minecraft:mule_baby#main",
              "minecraft:zombie_horse#main", "minecraft:skeleton_horse_baby#main")
    for layer in layers:
        rows += [
            pose("standing", layer, head_y_rot=-35.5, x_rot=48.0, age=3.25),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83, age=40.0, tail=True),
            pose("slow-walk", layer, walk_position=2.1, walk_speed=0.15, age=12.0),
            pose("swimming", layer, walk_position=7.3, walk_speed=0.6, age=40.0, water=True),
            pose("grazing-half", layer, eat=0.5, age=21.5),
            pose("grazing", layer, eat=1.0, head_y_rot=12.0, x_rot=-8.0, age=33.0),
            pose("rearing-half", layer, stand=0.4, walk_speed=0.2, age=9.0),
            pose("rearing", layer, stand=1.0, age=64.0, chest=True),
            pose("mouth-open", layer, feeding=1.0, age=17.0, chest=True),
            pose("rearing-and-grazing", layer, stand=0.5, eat=0.5, feeding=0.7, age=50.0),
        ]
    yield {"case_id": "entity/equine-model", "family": "entity", "operation": "equine_model",
           "rows": rows}


def entity_camel_cases():
    """The camel, grown and baby, posed by the reference's own `CamelModel.setupAnim`: the head
    within its limits, dipping while the dash cools down, the walk and each animation state; and
    the camel's own client tick through the pose, dash and last-pose data and the game time."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, age=0.0,
             cooldown=0.0, **started):
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "age_in_ticks": float_bits(age),
                "jump_cooldown": float_bits(cooldown), **started}

    rows = []
    for layer in ("minecraft:camel#main", "minecraft:camel_baby#main"):
        rows += [
            pose("standing", layer, head_y_rot=-50.0, x_rot=60.0, age=3.25),
            pose("looking", layer, head_y_rot=12.0, x_rot=-40.0, age=3.25),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83, age=40.0),
            pose("walking-fast", layer, walk_position=21.6, walk_speed=1.0, age=63.0),
            pose("dash-cooldown", layer, x_rot=10.0, cooldown=30.0, age=9.0),
            pose("dash-cooldown-long", layer, x_rot=60.0, cooldown=55.0, age=9.0),
            pose("sitting-down", layer, age=30.0, sit=10),
            pose("sitting", layer, age=130.0, sit_pose=10, walk_position=3.0, walk_speed=0.4),
            pose("standing-up", layer, age=45.0, sit_up=20),
            pose("idle", layer, age=77.5, idle=30, head_y_rot=20.0),
            pose("dashing", layer, age=33.0, dash=12, walk_position=4.0, walk_speed=0.9),
            pose("dashing-and-idle", layer, age=61.25, dash=12, idle=44),
        ]
    yield {"case_id": "entity/camel-model", "family": "entity", "operation": "camel_model",
           "rows": rows}


def entity_wolf_cases():
    """The wolf, grown and baby, posed by the reference's own `WolfModel.setupAnim`: the walk, the
    tail (angry, by health), sitting, the head roll and the body rolled by a shake."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             angry=False, sitting=False, tail=0.6283185, roll=0.0, shake=0.0):
        scale = 0.5 if layer.endswith("wolf_baby#main") else 1.0
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "angry": angry, "sitting": sitting,
                "tail_angle": float_bits(tail), "head_roll_angle": float_bits(roll),
                "shake_anim": float_bits(shake), "age_scale": float_bits(scale)}

    rows = []
    for layer in ("minecraft:wolf#main", "minecraft:wolf_baby#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83),
            pose("angry", layer, walk_position=7.3, walk_speed=0.83, angry=True, tail=1.5393804),
            pose("tame-hurt", layer, tail=1.2),
            pose("sitting", layer, sitting=True, walk_position=7.3, walk_speed=0.83),
            pose("sitting-looking", layer, sitting=True, head_y_rot=14.0, x_rot=-20.0, tail=1.7),
            pose("interested", layer, roll=0.3, head_y_rot=10.0),
            pose("shaking", layer, shake=0.9, roll=0.1),
            pose("shaking-late", layer, shake=1.65),
            pose("shaking-sitting", layer, shake=0.5, sitting=True),
        ]
    yield {"case_id": "entity/wolf-model", "family": "entity", "operation": "wolf_model",
           "rows": rows}


def entity_feline_cases():
    """The cat and the ocelot, grown and baby, posed by the reference's own `setupAnim`: the walk,
    crouching, sprinting, sitting, lying down (the tail on its own progress) and relaxing."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             crouching=False, sprinting=False, sitting=False, lie=0.0, lie_tail=0.0, relax=0.0):
        scale = 0.5 if "_baby" in layer else 1.0
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "crouching": crouching,
                "sprinting": sprinting, "sitting": sitting,
                "lie_down_amount": float_bits(lie), "lie_down_amount_tail": float_bits(lie_tail),
                "relax_state_one_amount": float_bits(relax), "age_scale": float_bits(scale)}

    rows = []
    for layer in ("minecraft:cat#main", "minecraft:cat_baby#main",
                  "minecraft:ocelot#main", "minecraft:ocelot_baby#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83),
            pose("crouching", layer, crouching=True, walk_position=7.3, walk_speed=0.83),
            pose("sprinting", layer, sprinting=True, walk_position=7.3, walk_speed=0.83),
            pose("sitting", layer, sitting=True, walk_position=7.3, walk_speed=0.83),
            pose("sitting-looking", layer, sitting=True, head_y_rot=14.0, x_rot=-20.0),
            pose("crouching-sitting", layer, crouching=True, sitting=True),
            pose("lying", layer, lie=0.6, lie_tail=0.32, walk_position=3.1, walk_speed=0.4),
            pose("lying-down", layer, lie=1.0, lie_tail=1.0, head_y_rot=9.0, x_rot=-12.0),
            pose("relaxed", layer, relax=0.7, x_rot=20.0),
            pose("lying-relaxed", layer, lie=0.45, lie_tail=0.16, relax=0.3),
        ]
    yield {"case_id": "entity/feline-model", "family": "entity", "operation": "feline_model",
           "rows": rows}


def entity_polar_bear_cases():
    """The polar bear, grown and baby, posed by the reference's own `PolarBearModel.setupAnim`: the
    quadruped's walk and look, and how far it has stood up."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, stand=0.0):
        scale = 0.5 if "_baby" in layer else 1.0
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "stand_scale": float_bits(stand),
                "age_scale": float_bits(scale)}

    rows = []
    for layer in ("minecraft:polar_bear#main", "minecraft:polar_bear_baby#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83),
            pose("rising", layer, stand=0.5, walk_position=2.1, walk_speed=0.3),
            pose("risen", layer, stand=1.0, head_y_rot=12.0, x_rot=-20.0),
        ]
    yield {"case_id": "entity/polar-bear-model", "family": "entity", "operation": "polar_bear_model",
           "rows": rows}


def entity_hoglin_cases():
    """The hoglin and the zoglin, grown and baby, posed by the reference's own `setupAnim`: the ears
    and legs by the walk, the head by the look and the attack swing."""
    def pose(label, layer, head_y_rot=0.0, walk_position=0.0, walk_speed=0.0, attack=0):
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "attack_ticks": attack}

    rows = []
    for layer in ("minecraft:hoglin#main", "minecraft:hoglin_baby#main",
                  "minecraft:zoglin#main", "minecraft:zoglin_baby#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83),
            pose("swing-start", layer, attack=10, head_y_rot=8.0),
            pose("swing-mid", layer, attack=7, walk_position=2.2, walk_speed=0.4),
            pose("swing-peak", layer, attack=5),
            pose("swing-end", layer, attack=1),
        ]
    yield {"case_id": "entity/hoglin-model", "family": "entity", "operation": "hoglin_model",
           "rows": rows}


def entity_fox_cases():
    """The fox, grown and baby, posed by the reference's own `FoxModel.setupAnim` (the walk, the
    crouching, sleeping, sitting and pouncing poses, the faceplant) and the matrix its held item is
    carried with, and the textures it wears."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, roll=0.0,
             crouch=0.0, crouching=False, sleeping=False, sitting=False, faceplanted=False,
             pouncing=False, age=37.5):
        scale = 0.5 if "_baby" in layer else 1.0
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "head_roll_angle": float_bits(roll),
                "crouch_amount": float_bits(crouch), "crouching": crouching, "sleeping": sleeping,
                "sitting": sitting, "faceplanted": faceplanted, "pouncing": pouncing,
                "age_in_ticks": float_bits(age), "age_scale": float_bits(scale)}

    rows = []
    for layer in ("minecraft:fox#main", "minecraft:fox_baby#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83),
            pose("walking-slow", layer, walk_position=2.1, walk_speed=0.2, roll=0.2),
            pose("interested", layer, roll=0.34, head_y_rot=10.0),
            pose("crouching", layer, crouching=True, crouch=3.4, walk_position=1.0, walk_speed=0.3),
            pose("crouched", layer, crouching=True, crouch=5.0, head_y_rot=20.0),
            pose("sleeping", layer, sleeping=True, roll=0.2, head_y_rot=15.0),
            pose("sitting", layer, sitting=True, head_y_rot=14.0, x_rot=-20.0),
            pose("sleeping-and-sitting", layer, sleeping=True, sitting=True),
            pose("pouncing", layer, pouncing=True, crouch=4.0, x_rot=-30.0),
            pose("pouncing-crouched", layer, pouncing=True, crouching=True, crouch=5.0),
            pose("faceplanted", layer, faceplanted=True, x_rot=60.0, head_y_rot=5.0),
        ]
    yield {"case_id": "entity/fox-model", "family": "entity", "operation": "fox_model", "rows": rows}
    yield {"case_id": "entity/fox-textures", "family": "entity", "operation": "fox_textures",
           "rows": [{"label": "all"}]}


def entity_panda_cases():
    """The panda, grown and baby, posed by the reference's own `PandaModel.setupAnim` (the unhappy
    shake, the sneeze, sitting with eating or fright, lying on the back, the roll), what its renderer
    turns the pose by (and a cat's and a fox's), and the textures it wears."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, age=37.5,
             unhappy=False, sneezing=False, sneeze_time=0, eating=False, scared=False, sit=0.0,
             back=0.0, roll=0.0):
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "age_in_ticks": float_bits(age),
                "unhappy": unhappy, "sneezing": sneezing, "sneeze_time": sneeze_time,
                "eating": eating, "scared": scared, "sit_amount": float_bits(sit),
                "lie_on_back_amount": float_bits(back), "roll_amount": float_bits(roll)}

    rows = []
    for layer in ("minecraft:panda#main", "minecraft:panda_baby#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83),
            pose("unhappy", layer, unhappy=True, walk_position=2.0, walk_speed=0.3),
            pose("sneeze-start", layer, sneezing=True, sneeze_time=1),
            pose("sneeze-wind-up", layer, sneezing=True, sneeze_time=14),
            pose("sneeze-hold", layer, sneezing=True, sneeze_time=15),
            pose("sneeze-release", layer, sneezing=True, sneeze_time=17),
            pose("sneeze-release-late", layer, sneezing=True, sneeze_time=19),
            pose("sneeze-done", layer, sneezing=True, sneeze_time=20),
            pose("sitting-part", layer, sit=0.45, head_y_rot=12.0, x_rot=-9.0),
            pose("sitting", layer, sit=1.0, walk_position=3.3, walk_speed=0.5),
            pose("eating", layer, sit=1.0, eating=True),
            pose("scared", layer, sit=1.0, scared=True),
            pose("eating-and-scared", layer, sit=0.8, eating=True, scared=True),
            pose("lying-on-back", layer, back=0.7, walk_position=1.1, walk_speed=0.4),
            pose("lying-on-back-fully", layer, back=1.0, head_y_rot=8.0),
            pose("rolling", layer, roll=0.6, walk_position=1.9, walk_speed=0.6),
            pose("rolling-sitting", layer, roll=1.0, sit=0.5),
            pose("unhappy-and-lying", layer, unhappy=True, back=0.5, sit=0.3),
        ]
    yield {"case_id": "entity/panda-model", "family": "entity", "operation": "panda_model",
           "rows": rows}
    yield {"case_id": "entity/panda-textures", "family": "entity", "operation": "panda_textures",
           "rows": [{"label": "all"}]}

    def turn(kind, label, baby=False, body_rot=37.0, x_rot=0.0, age=37.5, **fields):
        row = {"label": f"{kind}-{'baby-' if baby else ''}{label}", "kind": kind, "baby": baby,
               "body_rot": float_bits(body_rot), "x_rot": float_bits(x_rot),
               "age_in_ticks": float_bits(age)}
        row.update({"roll_time": float_bits(0.0), "sit_amount": float_bits(0.0),
                    "lie_on_back_amount": float_bits(0.0), "scared": False, "sitting": False,
                    "eating": False, "lie_down_amount": float_bits(0.0),
                    "on_sleeping_player": False, "pouncing": False, "faceplanted": False})
        for key, value in fields.items():
            row[key] = float_bits(value) if isinstance(value, float) else value
        return row

    turns = []
    for baby in (False, True):
        for roll_time in (0.0, 0.4, 3.5, 7.0, 7.6, 8.0, 11.25, 15.9, 16.0, 20.5, 23.99, 24.0, 28.7,
                          31.5, 32.0, 40.0):
            turns.append(turn("panda", f"roll-{roll_time}", baby, roll_time=roll_time, x_rot=-12.0))
        turns += [
            turn("panda", "sit-part", baby, sit_amount=0.4, x_rot=18.0, sitting=True),
            turn("panda", "sitting", baby, sit_amount=1.0, sitting=True, eating=True),
            turn("panda", "eating-bob", baby, sit_amount=1.0, sitting=True, eating=True, age=91.7),
            turn("panda", "scared", baby, sit_amount=1.0, scared=True, sitting=True, age=12.3),
            turn("panda", "lying-on-back", baby, lie_on_back_amount=0.65, x_rot=-20.0),
            turn("panda", "lying-fully", baby, lie_on_back_amount=1.0),
            turn("panda", "rolling-while-sitting", baby, roll_time=5.5, sit_amount=0.7,
                 lie_on_back_amount=0.2, x_rot=9.0),
            turn("cat", "standing", baby),
            turn("cat", "lying-part", baby, lie_down_amount=0.45),
            turn("cat", "lying", baby, lie_down_amount=1.0),
            turn("cat", "lying-on-a-sleeper", baby, lie_down_amount=0.8, on_sleeping_player=True),
            turn("fox", "standing", baby, x_rot=33.0),
            turn("fox", "pouncing", baby, x_rot=33.0, pouncing=True),
            turn("fox", "faceplanted", baby, x_rot=-47.5, faceplanted=True),
        ]
    yield {"case_id": "entity/setup-rotations", "family": "entity", "operation": "setup_rotations",
           "rows": turns}


def entity_shaking_cases():
    """What `setupRotations` leaves a mob at while it shakes: frozen through (any mob), a hoglin
    that is converting (not immune, awake, in a dimension where piglins zombify: read from the
    registry's own dimension types) and a strider that suffocates, over several ticks."""
    def row(kind, label, ticks=5, partial=0.0, dimension="minecraft:overworld", ticks_frozen=0,
            immune=False, no_ai=False, suffocating=False):
        return {"label": f"{kind}-{label}-{ticks}", "kind": kind, "ticks": ticks,
                "partial": float_bits(partial), "body_rot": float_bits(37.0),
                "dimension": dimension, "ticks_frozen": ticks_frozen, "immune": immune,
                "no_ai": no_ai, "suffocating": suffocating}

    nether = "minecraft:the_nether"
    rows = []
    for ticks, partial in ((0, 0.0), (5, 0.0), (6, 0.75), (123, 0.4), (4001, 0.9)):
        rows += [
            row("cow", "warm", ticks, partial),
            row("cow", "almost-frozen", ticks, partial, ticks_frozen=139),
            row("cow", "frozen", ticks, partial, ticks_frozen=140),
            row("hoglin", "overworld", ticks, partial),
            row("hoglin", "overworld-immune", ticks, partial, immune=True),
            row("hoglin", "overworld-no-ai", ticks, partial, no_ai=True),
            row("hoglin", "nether", ticks, partial, dimension=nether),
            row("hoglin", "nether-frozen", ticks, partial, dimension=nether, ticks_frozen=200),
            row("strider", "lava", ticks, partial),
            row("strider", "suffocating", ticks, partial, suffocating=True),
            row("strider", "frozen", ticks, partial, ticks_frozen=140),
        ]
    yield {"case_id": "entity/shaking-rotations", "family": "entity",
           "operation": "shaking_rotations", "rows": rows}


def entity_ravager_cases():
    """The ravager posed by the reference's own `RavagerModel.setupAnim`: the walk and look, the
    attack swing (the neck's thrust and the mouth's snap), the stun's sway and the roar."""
    def pose(label, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, stunned=0.0,
             attack=0.0, roar=0.0):
        return {"label": label, "layer": "minecraft:ravager#main", "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "stunned": float_bits(stunned),
                "attack": float_bits(attack), "roar": float_bits(roar)}

    rows = [
        pose("standing"),
        pose("looking", head_y_rot=-35.5, x_rot=48.0),
        pose("walking", walk_position=7.3, walk_speed=0.83),
    ]
    rows += [pose(f"attacking-{ticks}", attack=ticks, head_y_rot=6.0, walk_position=2.2,
                  walk_speed=0.3) for ticks in (9.5, 8.0, 6.5, 5.0, 4.2, 2.5, 1.0, 0.3)]
    rows += [pose(f"stunned-{ticks}", stunned=ticks, x_rot=-8.0) for ticks in (39.5, 30.0, 12.25, 1.5)]
    rows += [pose(f"roaring-{amount}", roar=amount, head_y_rot=-12.0)
             for amount in (0.05, 0.3, 0.5, 0.75, 0.95)]
    rows.append(pose("attacking-while-stunned", attack=7.0, stunned=20.0))
    yield {"case_id": "entity/ravager-model", "family": "entity", "operation": "ravager_model",
           "rows": rows}


def entity_strider_cases():
    """The strider (grown, baby and the saddle's model) posed by the reference's own `setupAnim`: the
    body turned by the look unless ridden, rocking with the walk, the legs, the flowing and stirring
    bristles; and the textures it wears."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, age=37.5,
             ridden=False):
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "age_in_ticks": float_bits(age),
                "ridden": ridden}

    rows = []
    for layer in ("minecraft:strider#main", "minecraft:strider_baby#main",
                  "minecraft:strider#saddle"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0),
            pose("walking", layer, walk_position=7.3, walk_speed=0.2),
            pose("running", layer, walk_position=11.9, walk_speed=0.83, age=91.3),
            pose("ridden", layer, head_y_rot=20.0, x_rot=-15.0, walk_position=3.1, walk_speed=0.1,
                 ridden=True),
            pose("ridden-and-fast", layer, walk_position=5.5, walk_speed=0.9, ridden=True, age=5.0),
        ]
    yield {"case_id": "entity/strider-model", "family": "entity", "operation": "strider_model",
           "rows": rows}
    yield {"case_id": "entity/strider-textures", "family": "entity", "operation": "strider_textures",
           "rows": [{"label": "all"}]}


def entity_llama_cases():
    """The llama, grown and baby, posed by the reference's own `LlamaModel.setupAnim`: the head, the
    diagonal stride, the chests."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, chest=False):
        return {"label": f"{layer}-{label}", "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "has_chest": chest}

    rows = []
    for layer in ("minecraft:llama#main", "minecraft:llama_baby#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83),
            pose("walking-fast-with-a-chest", layer, walk_position=21.6, walk_speed=1.0, chest=True),
            pose("chest", layer, head_y_rot=12.0, x_rot=-8.0, chest=True),
        ]
    yield {"case_id": "entity/llama-model", "family": "entity", "operation": "llama_model",
           "rows": rows}


def entity_goat_cases():
    """The goat, grown and baby, posed by the reference's own `GoatModel.setupAnim`
    (the quadruped pose, the horns it still has, the head lowered to ram) and drawn."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             left=True, right=True, ramming=0.0):
        return {"label": f"{layer}-{label}", "layer": layer,
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "left_horn": left, "right_horn": right, "ramming": float_bits(ramming)}

    rows = []
    for layer in ("minecraft:goat#main", "minecraft:goat_baby#main"):
        rows += [
            pose("standing", layer),
            pose("walking-and-looking", layer, head_y_rot=12.0, x_rot=-8.0, walk_position=7.3,
                 walk_speed=0.83),
            pose("no-left-horn", layer, left=False),
            pose("no-right-horn", layer, right=False),
            pose("hornless", layer, left=False, right=False),
            pose("ramming-half", layer, head_y_rot=-35.5, x_rot=48.0, ramming=0.2617994),
            pose("ramming-full", layer, walk_position=3.4, walk_speed=0.6, ramming=0.5235988),
        ]
    yield {"case_id": "entity/goat-model", "family": "entity", "operation": "goat_model",
           "rows": rows}
