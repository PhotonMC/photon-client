"""The `entity` family: flyers."""

from cases.helpers import *


def entity_airborne_cases():
    """The blaze, ghast and phantom posed by the reference's own `setupAnim` (the blaze's three rings
    of rods turning and bobbing with the age, and its head's look; the ghast's tentacles swaying out
    of step; the phantom's wings and tail beating by the flap time), and what a phantom's renderer
    does to the pose: its size's scale and lift, and its pitch's tip."""
    rows = []
    for age in (0.0, 1.5, 37.25, 211.75):
        rows.append({"label": f"blaze-{age}", "layer": "minecraft:blaze#main",
                     "age_in_ticks": float_bits(age), "y_rot": float_bits(0.0),
                     "x_rot": float_bits(0.0)})
        rows.append({"label": f"ghast-{age}", "layer": "minecraft:ghast#main",
                     "age_in_ticks": float_bits(age)})
        rows.append({"label": f"phantom-{age}", "layer": "minecraft:phantom#main",
                     "flap_time": float_bits(age + 9.0)})
    rows += [{"label": "blaze-looks", "layer": "minecraft:blaze#main",
              "age_in_ticks": float_bits(12.5), "y_rot": float_bits(-47.0),
              "x_rot": float_bits(23.5)},
             {"label": "phantom-flap-offset", "layer": "minecraft:phantom#main",
              "flap_time": float_bits(1234.5)}]
    yield {"case_id": "entity/airborne-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}
    turns = []
    for label, size, x_rot, body_rot in (("small", 0, 0.0, 0.0), ("grown", 3, 25.0, 37.0),
                                         ("full-size", 64, -40.0, -120.0), ("tipped", 10, 80.5, 12.0)):
        turns.append({"label": f"phantom-{label}", "kind": "phantom", "baby": False, "size": size,
                      "x_rot": float_bits(x_rot), "body_rot": float_bits(body_rot),
                      "age_in_ticks": float_bits(10.5)})
    turns.append({"label": "phantom-dying", "kind": "phantom", "baby": False, "size": 4,
                  "x_rot": float_bits(-15.0), "body_rot": float_bits(60.0),
                  "age_in_ticks": float_bits(10.5), "death_time": float_bits(7.5)})
    yield {"case_id": "entity/phantom-rotations", "family": "entity",
           "operation": "setup_rotations", "rows": turns}


def entity_flier_cases():
    """The bat, vex and allay posed by the reference's own `setupAnim`: a bat that hangs turns its head
    and runs one animation, one that flies runs the other; a vex (leaning, arms bobbing, wings
    beating, and charging with its arms out, higher with an item in either hand); an allay (wings
    flapping with the walk, the body tipping as it flies, the arms rising to what it holds, bobbing
    while it hovers, dancing and spinning)."""
    rows = []

    def bat(label, age=37.5, y_rot=0.0, resting=False, **started):
        row = {"label": f"bat-{label}", "layer": "minecraft:bat#main", "age_in_ticks": float_bits(age),
               "y_rot": float_bits(y_rot), "resting": resting}
        row.update(started)
        return row
    rows += [bat("nothing"), bat("flies", fly=5), bat("flies-later", age=90.0, fly=5),
             bat("hangs", resting=True, rest=5, y_rot=40.0), bat("hangs-later", age=64.0, resting=True, rest=5),
             bat("flies-with-a-turned-head", y_rot=-55.0, fly=5),
             bat("rests-without-hanging", y_rot=25.0, rest=5)]

    def vex(label, age=37.5, y_rot=0.0, x_rot=0.0, charging=False, right=False, left=False):
        return {"label": f"vex-{label}", "layer": "minecraft:vex#main", "age_in_ticks": float_bits(age),
                "y_rot": float_bits(y_rot), "x_rot": float_bits(x_rot), "charging": charging,
                "right_item": right, "left_item": left}
    rows += [vex("hovers"), vex("later", age=91.25), vex("looks", y_rot=30.0, x_rot=-15.0),
             vex("charges", charging=True), vex("charges-with-a-right-item", charging=True, right=True),
             vex("charges-with-a-left-item", charging=True, left=True, age=12.0),
             vex("charges-with-both", charging=True, right=True, left=True, age=60.0),
             vex("holds-an-item-without-charging", right=True)]

    def allay(label, age=37.5, y_rot=0.0, x_rot=0.0, walk=(0.0, 0.0), dancing=False, spinning=False,
              spin=0.0, holding=0.0):
        return {"label": f"allay-{label}", "layer": "minecraft:allay#main", "age_in_ticks": float_bits(age),
                "y_rot": float_bits(y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk[0]), "walk_speed": float_bits(walk[1]),
                "dancing": dancing, "spinning": spinning, "spinning_progress": float_bits(spin),
                "holding_progress": float_bits(holding)}
    rows += [allay("hovers"), allay("hovers-later", age=91.25),
             allay("looks", y_rot=-30.0, x_rot=20.0),
             allay("flies", walk=(3.0, 0.5)), allay("flies-slowly", walk=(1.5, 0.12), age=12.0),
             allay("holds", holding=1.0), allay("part-holds", holding=0.4, age=60.0),
             allay("flies-holding", walk=(3.0, 0.5), holding=1.0),
             allay("dances", dancing=True, age=20.0), allay("dances-later", dancing=True, age=75.5),
             allay("spins", dancing=True, spinning=True, spin=0.6),
             allay("dances-flying", dancing=True, walk=(2.0, 0.4), age=44.0),
             allay("dances-holding", dancing=True, holding=1.0, spinning=True, spin=1.0)]
    yield {"case_id": "entity/flier-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_parrot_cases():
    """The parrot posed by the reference's own `ParrotModel.setupAnim`: each pose (flying, standing
    with the legs walking and the wings beating, sitting, party swaying, on a shoulder), with the wings
    at rest and beating, and the texture of each variant (ids past the ends clamp)."""
    def parrot(label, pose, age=37.5, y_rot=0.0, x_rot=0.0, walk=(0.0, 0.0), flap=0.0):
        return {"label": f"parrot-{label}", "layer": "minecraft:parrot#main", "pose": pose,
                "age_in_ticks": float_bits(age), "y_rot": float_bits(y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk[0]), "walk_speed": float_bits(walk[1]),
                "flap_angle": float_bits(flap)}
    rows = []
    for pose in ("FLYING", "STANDING", "SITTING", "PARTY", "ON_SHOULDER"):
        name = pose.lower().replace("_", "-")
        rows += [parrot(f"{name}", pose), parrot(f"{name}-beating", pose, flap=0.8),
                 parrot(f"{name}-looking", pose, y_rot=35.0, x_rot=-20.0, flap=0.3),
                 parrot(f"{name}-walking", pose, walk=(4.5, 0.6), flap=0.5, age=91.25)]
    yield {"case_id": "entity/parrot-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}
    yield {"case_id": "entity/parrot-textures", "family": "entity", "operation": "parrot_textures",
           "rows": [{"label": f"variant-{variant}", "id": variant} for variant in (-3, 0, 1, 2, 3, 4, 5, 9)]}


def entity_breeze_cases():
    """The breeze's three models (the body with its rods and head, the wind, the eyes) posed by the
    reference's own `BreezeModel.setupAnim`: each of the six animations alone (the looping idle,
    the others running out), early and late, and some together."""
    def breeze(layer, label, age=37.5, **started):
        row = {"label": f"breeze-{layer}-{label}", "layer": f"minecraft:breeze#{layer}",
               "age_in_ticks": float_bits(age)}
        row.update(started)
        return row
    rows = []
    for layer in ("main", "wind", "eyes"):
        rows += [breeze(layer, "nothing"), breeze(layer, "idle", idle=3),
                 breeze(layer, "idle-later", age=111.0, idle=3),
                 breeze(layer, "shoot", age=45.0, shoot=30), breeze(layer, "shoot-over", age=120.0, shoot=30),
                 breeze(layer, "slide", age=50.0, slide=40), breeze(layer, "slide-back", age=62.0, slide_back=55),
                 breeze(layer, "inhale", age=41.0, inhale=33), breeze(layer, "jump", age=60.0, long_jump=50),
                 breeze(layer, "idle-shooting", age=45.0, idle=3, shoot=30),
                 breeze(layer, "sliding-then-back", age=70.0, slide=40, slide_back=65, idle=3)]
    yield {"case_id": "entity/breeze-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_happy_ghast_cases():
    """The happy ghast posed by the reference's own `HappyGhastModel.setupAnim` (tentacles swaying out
    of step; the body squeezed while it wears something), grown and baby, and the harness's goggles on
    the brow or, ridden, over the eyes."""
    rows = []
    for layer in ("happy_ghast", "happy_ghast_baby"):
        for age in (0.0, 1.5, 37.25, 211.75):
            rows.append({"label": f"{layer}-{age}", "layer": f"minecraft:{layer}#main",
                         "age_in_ticks": float_bits(age), "body_item": False, "ridden": False})
        rows.append({"label": f"{layer}-harnessed", "layer": f"minecraft:{layer}#main",
                     "age_in_ticks": float_bits(37.25), "body_item": True, "ridden": False})
    for layer in ("happy_ghast_harness", "happy_ghast_baby_harness"):
        for ridden in (False, True):
            rows.append({"label": f"{layer}-{'ridden' if ridden else 'idle'}",
                         "layer": f"minecraft:{layer}#main", "age_in_ticks": float_bits(0.0),
                         "body_item": True, "ridden": ridden})
    yield {"case_id": "entity/happy-ghast-model", "family": "entity", "operation": "airborne_model",
           "rows": rows}


def entity_flier_item_cases():
    """Where a vex's and an allay's `ItemInHandLayer` puts what each hand holds (from an identity
    pose): in the right hand, the left, both, a left-handed mob's, a vex charging (arms out) or not,
    an allay hovering, flying and holding (the arm's tilt carries the item)."""
    def item(kind, label, holds=(True, False), age=37.5, charging=False, walk=(0.0, 0.0), holding=0.0,
             main_arm="right"):
        return {"label": f"{kind}-{label}", "kind": kind, "holds_right": holds[0], "holds_left": holds[1],
                "age_in_ticks": float_bits(age), "charging": charging, "main_arm": main_arm,
                "walk_position": float_bits(walk[0]), "walk_speed": float_bits(walk[1]),
                "holding_progress": float_bits(holding)}
    rows = [item("vex", "right"), item("vex", "left", holds=(False, True)),
            item("vex", "both", holds=(True, True), age=91.25),
            item("vex", "charging-right", charging=True), item("vex", "charging-left", charging=True,
                                                                  holds=(False, True)),
            item("vex", "charging-both", charging=True, holds=(True, True), age=12.0),
            item("allay", "right", holding=1.0), item("allay", "left", holds=(False, True), holding=1.0),
            item("allay", "both", holds=(True, True), holding=1.0, age=60.0),
            item("allay", "flying", holding=1.0, walk=(3.0, 0.5)),
            item("allay", "part-holding", holding=0.4, walk=(1.5, 0.12), age=12.0),
            item("allay", "hovering-empty-handed", holds=(False, False))]
    yield {"case_id": "entity/flier-item-pose", "family": "entity", "operation": "flier_item_pose",
           "rows": rows}
