"""The `entity` family: items."""

from cases.helpers import *


def entity_item_pose_cases():
    """A dropped item's poses (`ItemEntityRenderer.submit`): the bob and spin
    through the age, each copy of the stack (one to five) with the offsets its
    seed gives, for a thick model (a block) and a flat one (a sprite), and
    models that sit off the origin or have no thickness at all."""
    def row(label, age, bob, count, seed, box):
        low, high = box
        data = {"label": label, "age": float_bits(age), "bob": float_bits(bob), "count": count, "seed": seed}
        for axis, value in zip("xyz", low):
            data["min_" + axis] = float_bits(value)
        for axis, value in zip("xyz", high):
            data["max_" + axis] = float_bits(value)
        return data

    block = ((-0.25, 0.0, -0.25), (0.25, 0.5, 0.25))
    sprite = ((-0.25, -0.25, -0.015625), (0.25, 0.25, 0.015625))
    thick_sprite = ((-0.25, -0.25, -0.05), (0.25, 0.25, 0.05))
    off_centre = ((0.0, 0.125, 0.0), (0.5, 0.375, 0.2))
    flat_plane = ((-0.25, -0.25, 0.0), (0.25, 0.25, 0.0))
    rows = []
    for count in (1, 2, 3, 4, 5):
        rows.append(row(f"block-{count}", 12.5, 1.25, count, 1 + count, block))
        rows.append(row(f"sprite-{count}", 12.5, 1.25, count, 262 + count, sprite))
    for age in (0.0, 0.5, 7.0, 31.25, 100.75, 1200.5, 20000.0):
        rows.append(row(f"age-{age}", age, 0.7, 1, 5, block))
    for bob in (0.0, 1.0, 3.1415927, 6.2831855):
        rows.append(row(f"bob-{bob}", 40.0, bob, 2, 5, sprite))
    for seed in (0, 1, 187, 2025, -5, 2147483647):
        rows.append(row(f"seed-{seed}", 3.0, 2.0, 5, seed, block))
        rows.append(row(f"flat-seed-{seed}", 3.0, 2.0, 5, seed, sprite))
    rows += [
        row("thick-sprite", 9.0, 0.3, 3, 17, thick_sprite),
        row("off-centre", 9.0, 0.3, 4, 17, off_centre),
        row("flat-plane", 9.0, 0.3, 3, 17, flat_plane),
        row("depth-at-threshold", 9.0, 0.3, 3, 17, ((-0.25, -0.25, 0.0), (0.25, 0.25, 0.0625))),
        row("depth-over-threshold", 9.0, 0.3, 3, 17, ((-0.25, -0.25, 0.0), (0.25, 0.25, 0.0626))),
    ]
    yield {
        "case_id": "entity/item-entity-pose",
        "family": "entity",
        "operation": "item_entity_pose",
        "rows": rows,
    }


def entity_quadruped_cases():
    """The pig and the cow, grown and baby, from the layer table, posed by the
    reference's `QuadrupedModel.setupAnim` and drawn by its part renderer:
    standing, looking about, walking at several points of the stride and
    speeds."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0):
        return {"label": f"{layer}-{label}", "layer": layer,
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed)}

    rows = []
    for layer in ("minecraft:pig#main", "minecraft:pig_baby#main", "minecraft:cow#main",
                  "minecraft:cow_baby#main", "minecraft:cold_pig#main", "minecraft:mooshroom#main",
                  "minecraft:mooshroom_baby#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0),
            pose("looking-up", layer, head_y_rot=70.0, x_rot=-60.0),
            pose("walk-early", layer, walk_position=0.9, walk_speed=0.4),
            pose("walk-mid", layer, walk_position=7.3, walk_speed=0.83),
            pose("walk-fast", layer, walk_position=21.6, walk_speed=1.0),
            pose("walk-and-look", layer, head_y_rot=12.0, x_rot=-8.0, walk_position=3.4, walk_speed=0.6),
        ]
    yield {
        "case_id": "entity/quadruped-model",
        "family": "entity",
        "operation": "quadruped_model",
        "rows": rows,
    }


def entity_held_item_cases():
    """What a humanoid holds, as the reference's `ItemInHandLayer` places it
    on the posed model: the pose each held stack is submitted with (arm,
    root and slim offset, the half turn, the baby's closer hand) and the arms'
    poses the held items ask for (`ITEM`, a skeleton's `BOW_AND_ARROW`),
    posed by the reference's `HumanoidModel.setupAnim`, for the player, the
    zombie, the skeleton and the piglin, grown and baby, either hand."""
    def row(label, kind, baby=False, right="empty", left="empty", holds=(False, False),
            main="right", head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
            age=0.0, attack=0.0, swim=0.0, crouching=False, aggressive=False, bow=False,
            head=None, used=0.0, charge=0.0, using=False, hand="main", passenger=False, gliding=False, speed=1.0, skull=None, skull_anim=0.0, eye=False,
            items=None, swing="whack", attack_arm="right", hit=0.0):
        out = {"label": f"{kind}{'-baby' if baby else ''}-{label}", "kind": kind, "baby": baby,
                "ticks_using_item": float_bits(used), "max_crossbow_charge": float_bits(charge),
                "using_item": using, "use_hand": hand, "passenger": passenger, "fall_flying": gliding,
                "speed_value": float_bits(speed),
                "right_pose": right, "left_pose": left, "main_arm": main,
                "holds_right": holds[0], "holds_left": holds[1],
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "age_in_ticks": float_bits(age), "attack_time": float_bits(attack),
                "swim_amount": float_bits(swim), "crouching": crouching,
                "aggressive": aggressive, "holding_bow": bow}
        if head is not None:
            out.update({"head_y_offset": float_bits(head[0]), "head_skull_y": float_bits(head[1]),
                        "head_scale": float_bits(head[2])})
        if skull is not None:
            out.update({"skull": skull, "skull_animation": float_bits(skull_anim)})
        if eye:
            out["eye_item"] = True
        if items is not None:
            # Real stacks in the hands, and how the hand that swings does.
            for side, name in zip(("right", "left"), items):
                if name is not None:
                    out[f"{side}_item"] = f"minecraft:{name}"
            out.update({"swing": swing, "attack_arm": attack_arm, "hit_feedback": float_bits(hit)})
        return out

    rows = []
    for kind in ("player", "player_slim"):
        rows += [
            row("empty-hands", kind),
            row("item-right", kind, right="item", holds=(True, False), age=3.0),
            row("item-left-hand", kind, left="item", holds=(False, True), age=3.0),
            row("both", kind, right="item", left="item", holds=(True, True), walk_position=7.3,
                walk_speed=0.83, age=40.0),
            row("looking", kind, right="item", holds=(True, False), head_y_rot=-35.5, x_rot=48.0),
            row("crouching", kind, right="item", left="item", holds=(True, True), crouching=True),
            row("swing", kind, right="item", holds=(True, False), attack=0.4, age=9.0),
            row("swimming", kind, right="item", holds=(True, False), swim=0.6, age=11.0,
                walk_position=2.0, walk_speed=0.5),
            row("left-handed", kind, right="item", left="item", holds=(True, True), main="left",
                age=17.5, attack=0.3),
            row("bow-drawn-left-handed", kind, right="bow_and_arrow", left="item",
                holds=(True, True), main="left", head_y_rot=12.0, x_rot=-8.0),
            row("offhand-two-handed", kind, right="item", left="bow_and_arrow", holds=(True, True)),
        ]
        # The remaining arm poses, either arm, looking about and crouching.
        for pose in ("block", "throw_trident", "crossbow_hold", "spyglass", "toot_horn", "brush"):
            for arm, other in (("right", "left"), ("left", "right")):
                sides = {arm: pose, other: "item"}
                for look, (yaw, pitch, crouch) in {"level": (0.0, 0.0, False), "up": (-35.5, -48.0, False),
                                                   "down": (40.0, 70.0, True), "wide": (95.0, 12.0, False)}.items():
                    rows.append(row(f"{pose}-{arm}-{look}", kind, right=sides.get("right", "empty"),
                                    left=sides.get("left", "empty"), holds=(True, True),
                                    head_y_rot=yaw, x_rot=pitch, crouching=crouch, age=5.5))
        for arm, other in (("right", "left"), ("left", "right")):
            for used in (0.0, 3.0, 12.5, 25.0, 40.0):
                sides = {arm: "crossbow_charge", other: "item"}
                rows.append(row(f"crossbow-charge-{arm}-{used}", kind, right=sides.get("right", "empty"),
                                left=sides.get("left", "empty"), holds=(True, True), used=used,
                                charge=25.0, head_y_rot=10.0, x_rot=-20.0))
            rows.append(row(f"crossbow-hold-both-{arm}", kind, right="crossbow_hold", left="crossbow_hold",
                            holds=(True, True), head_y_rot=-15.0, x_rot=25.0))
        # An item in use: the arm of the hand using it is posed first, for either main arm.
        for pose in ("bow_and_arrow", "block", "spyglass", "toot_horn", "brush", "crossbow_charge",
                     "throw_trident"):
            for main, hand in (("right", "main"), ("right", "off"), ("left", "main"), ("left", "off")):
                right_arm = (main == "right") == (hand == "main")
                sides = {"right": pose if right_arm else "item", "left": "item" if right_arm else pose}
                rows.append(row(f"using-{pose}-{main}-{hand}", kind, right=sides["right"], left=sides["left"],
                                holds=(True, True), main=main, using=True, hand=hand, used=9.0,
                                charge=25.0, head_y_rot=14.0, x_rot=-18.0, age=7.25))
        rows += [
            row("using-swimming", kind, right="bow_and_arrow", left="item", holds=(True, True), using=True,
                swim=0.6, walk_position=3.0, walk_speed=0.4, age=2.5, head_y_rot=5.0, x_rot=10.0),
            row("not-using-swimming", kind, right="bow_and_arrow", left="item", holds=(True, True),
                swim=0.6, walk_position=3.0, walk_speed=0.4, age=2.5, head_y_rot=5.0, x_rot=10.0),
            row("spyglass-swimming", kind, right="spyglass", left="item", holds=(True, True), using=True,
                swim=0.4, walk_position=15.0, walk_speed=0.4, age=2.5),
            row("passenger", kind, right="item", left="item", holds=(True, True), passenger=True,
                walk_position=2.0, walk_speed=0.3, age=11.0),
            row("passenger-bow", kind, right="bow_and_arrow", left="item", holds=(True, True), passenger=True,
                using=True, head_y_rot=-10.0, x_rot=15.0),
            row("passenger-crouching", kind, right="item", holds=(True, False), passenger=True,
                crouching=True),
            row("gliding", kind, right="item", left="item", holds=(True, True), gliding=True,
                head_y_rot=12.0, x_rot=33.0),
            row("gliding-fast", kind, right="item", left="item", holds=(True, True), gliding=True,
                walk_position=4.0, walk_speed=0.5, speed=3.5, x_rot=-12.0),
            row("slowed-limbs", kind, right="item", left="item", holds=(True, True),
                walk_position=4.0, walk_speed=0.5, speed=3.5, x_rot=-12.0),
            row("gliding-walking", kind, right="item", left="item", holds=(True, True), gliding=True,
                walk_position=4.0, walk_speed=0.5, x_rot=-12.0),
            row("gliding-swim", kind, right="item", holds=(True, False), gliding=True, swim=0.5,
                walk_position=3.0, walk_speed=0.4, x_rot=20.0)]
    for kind in ("zombie", "piglin"):
        for baby in (False, True):
            rows += [
                row("standing", kind, baby, holds=(True, True)),
                row("walking", kind, baby, holds=(True, False), walk_position=7.3,
                    walk_speed=0.83, age=40.0),
                row("swing", kind, baby, holds=(False, True), attack=0.5, age=9.0),
            ]
    for baby in (False, True):
        rows.append(row("aggressive", "zombie", baby, holds=(True, False), aggressive=True,
                        age=17.5))
        rows.append(row("left-handed", "zombie", baby, holds=(True, True), main="left"))
    rows += [
        row("standing", "skeleton", holds=(True, False), bow=True),
        row("bow-drawn", "skeleton", right="bow_and_arrow", holds=(True, True), aggressive=True,
            bow=True, age=17.5, attack=0.4),
        row("bow-drawn-left-handed", "skeleton", left="bow_and_arrow", holds=(True, True),
            main="left", aggressive=True, bow=True, head_y_rot=-35.5, x_rot=48.0, age=3.25),
        row("sword-aggressive", "skeleton", holds=(True, False), aggressive=True, age=17.5,
            attack=0.55),
    ]
    # The item on the head (`CustomHeadLayer`), with each renderer's transforms.
    default = (0.0, 0.0, 1.0)
    for kind, transforms in (("player", default), ("player_slim", default), ("zombie", default),
                             ("skeleton", default), ("piglin", (0.0, 0.0, 1.0019531))):
        for baby in ((False, True) if kind in ("zombie", "piglin") else (False,)):
            rows += [
                row("head-item", kind, baby, head=transforms),
                row("head-item-looking", kind, baby, head=transforms, head_y_rot=-35.5, x_rot=48.0),
                row("head-item-crouching", kind, baby, head=transforms, crouching=True, age=3.0),
            ]
    rows.append(row("head-item-villager-transforms", "zombie", head=(-0.1171875, -0.07421875, 1.0),
                    walk_position=7.3, walk_speed=0.83))
    # A skull on the head: each kind, the walk's swing of a piglin's ears and a dragon's jaw, the head
    # turned, sneaking, and the renderers' transforms.
    skulls = ("skeleton", "wither_skeleton", "player", "zombie", "creeper", "piglin", "dragon")
    for kind, transforms in (("player", default), ("player_slim", default), ("zombie", default),
                             ("skeleton", default), ("piglin", (0.0, 0.0, 1.0019531))):
        for baby in ((False, True) if kind in ("zombie", "piglin") else (False,)):
            for skull in skulls:
                rows.append(row(f"skull-{skull}", kind, baby, head=transforms, skull=skull, skull_anim=3.7))
            rows.append(row("skull-piglin-walking", kind, baby, head=transforms, skull="piglin",
                            skull_anim=12.25, head_y_rot=-35.5, x_rot=48.0))
            rows.append(row("skull-dragon-walking", kind, baby, head=transforms, skull="dragon",
                            skull_anim=21.5, crouching=True, age=3.0))
    # A spyglass in use is held to the eye (a player's only), from either hand, looking about, and when
    # it is not (swinging, another hand, not in use).
    for kind in ("player", "player_slim"):
        for main in ("right", "left"):
            for hand in ("main", "off"):
                for label, yaw, pitch in (("level", 0.0, 0.0), ("up", -35.5, -60.0), ("down", 20.0, 80.0),
                                          ("far-down", 5.0, 120.0)):
                    right_arm = (main == "right") == (hand == "main")
                    sides = {"right": "spyglass" if right_arm else "item",
                             "left": "item" if right_arm else "spyglass"}
                    rows.append(row(f"eye-{main}-{hand}-{label}", kind, right=sides["right"], left=sides["left"],
                                    holds=(True, True), main=main, using=True, hand=hand, eye=True,
                                    head_y_rot=yaw, x_rot=pitch, age=4.5))
        rows += [
            row("eye-swinging", kind, right="spyglass", left="item", holds=(True, True), using=True,
                eye=True, attack=0.3, age=4.5),
            row("eye-not-in-use", kind, right="spyglass", left="item", holds=(True, True), eye=True),
            row("eye-crouching", kind, right="spyglass", left="item", holds=(True, True), using=True,
                eye=True, crouching=True, head_y_rot=10.0, x_rot=-20.0),
            row("eye-no-eye-item", kind, right="spyglass", left="item", holds=(True, True), using=True,
                head_y_rot=10.0, x_rot=-20.0)]
    rows.append(row("skull-villager-transforms", "zombie", head=(-0.1171875, -0.07421875, 1.0), skull="zombie",
                    walk_position=7.3, walk_speed=0.83))
    # A spear (`SpearAnimations`): the arm raises it, sways it and lowers it as the use goes on, for each
    # spear's own timings; the stab of its swing thrusts arm and item; the hit's kick; and what each
    # model state changes (the other hand, the main arm, gliding, swimming).
    spears = ("wooden", "stone", "copper", "iron", "golden", "diamond", "netherite")
    for name in spears:
        for used in (0.5, 9.5, 17.5, 45.5, 95.5, 160.5, 230.5, 300.5):
            rows.append(row(f"spear-{name}-use-{used}", "player", right="spear", left="item", holds=(True, True),
                            items=(f"{name}_spear", None), using=True, used=used, head_y_rot=8.0, x_rot=-14.0,
                            age=6.5))
    for kind, baby in (("player_slim", False), ("zombie", False), ("zombie", True), ("skeleton", False),
                       ("piglin", False), ("piglin", True)):
        rows.append(row("spear-use", kind, baby, right="spear", left="spear", holds=(True, True),
                        items=("iron_spear", "wooden_spear"), using=True, used=33.5, head_y_rot=-12.0, x_rot=20.0,
                        age=2.5, walk_position=3.0, walk_speed=0.3))
    for main, hand in (("right", "main"), ("right", "off"), ("left", "main"), ("left", "off")):
        rows.append(row(f"spear-use-{main}-{hand}", "player", right="spear", left="spear", holds=(True, True),
                        items=("stone_spear", "diamond_spear"), main=main, using=True, hand=hand, used=21.5,
                        head_y_rot=14.0, x_rot=-18.0, age=7.25))
    rows += [
        row("spear-held", "player", right="spear", left="item", holds=(True, True), items=("iron_spear", None),
            head_y_rot=5.0, x_rot=10.0),
        row("spear-held-left", "player", right="item", left="spear", holds=(True, True), main="left",
            items=(None, "iron_spear"), head_y_rot=5.0, x_rot=10.0),
        row("spear-use-gliding", "player", right="spear", holds=(True, False), items=("iron_spear", None),
            using=True, used=14.5, gliding=True, x_rot=30.0),
        row("spear-use-swimming", "player", right="spear", left="item", holds=(True, True),
            items=("iron_spear", None), using=True, used=14.5, swim=0.6, walk_position=3.0, walk_speed=0.4),
        row("spear-held-swimming", "player", right="spear", left="item", holds=(True, True),
            items=("iron_spear", None), swim=0.6, walk_position=15.0, walk_speed=0.4),
        row("spear-use-passenger", "player", right="spear", left="item", holds=(True, True),
            items=("iron_spear", None), using=True, used=14.5, passenger=True),
        row("spear-use-crouching", "player", right="spear", left="item", holds=(True, True),
            items=("iron_spear", None), using=True, used=40.5, crouching=True),
    ]
    for hit in (0.0, 1.5, 2.5, 3.5, 6.5, 9.5, 12.0):
        rows.append(row(f"spear-hit-feedback-{hit}", "player", right="spear", left="item", holds=(True, True),
                        items=("iron_spear", None), using=True, used=30.5, hit=hit))
    for arm in ("right", "left"):
        main = arm
        sides = {arm: "spear", "left" if arm == "right" else "right": "item"}
        for attack in (0.02, 0.06, 0.12, 0.2, 0.3, 0.45, 0.6, 0.8, 0.95):
            for kind, baby in (("player", False), ("zombie", False), ("piglin", True)):
                spear_items = (("iron_spear", None) if arm == "right" else (None, "iron_spear"))
                rows.append(row(f"spear-stab-{arm}-{attack}", kind, baby, right=sides["right"], left=sides["left"],
                                holds=(True, True), items=spear_items, swing="stab", attack_arm=arm, main=main,
                                attack=attack, head_y_rot=6.0, x_rot=-9.0, age=3.5))
    rows += [
        row("spear-stab-in-use", "player", right="spear", left="item", holds=(True, True),
            items=("netherite_spear", None), swing="stab", attack=0.3, using=True, used=12.5, hit=4.0),
        row("spear-stab-offhand-attack", "player", right="item", left="spear", holds=(True, True),
            items=(None, "iron_spear"), swing="stab", attack_arm="left", attack=0.3),
        row("spear-whack-with-spear-held", "player", right="spear", left="item", holds=(True, True),
            items=("iron_spear", None), swing="whack", attack=0.3),
        row("spear-stab-swimming", "player", right="spear", left="item", holds=(True, True),
            items=("iron_spear", None), swing="stab", attack=0.3, swim=0.6, walk_position=3.0, walk_speed=0.4),
    ]
    yield {
        "case_id": "entity/held-item-pose",
        "family": "entity",
        "operation": "held_item_pose",
        "rows": rows,
    }


def entity_animation_cases():
    """The reference's keyframe animations: every definition (the table), a rabbit model
    posed by the reference's own `KeyframeAnimation.apply` at points of the hop and of the
    idle head tilt (both interpolations, all three targets, a looping and a clamped
    definition, a target scale below one), and `AnimationState`'s timing."""
    yield {"case_id": "entity/animation-definitions", "family": "entity",
           "operation": "animation_definitions", "rows": [{"label": "all"}]}

    def pose(layer, definition, millis, scale=1.0):
        return {"label": f"{layer}-{definition}-{millis}-{scale}", "layer": layer,
                "definition": definition, "millis": millis, "scale": float_bits(scale)}

    rows = []
    for layer, hop, tilt in (("minecraft:rabbit#main", "Rabbit.HOP", "Rabbit.IDLE_HEAD_TILT"),
                             ("minecraft:rabbit_baby#main", "BabyRabbit.HOP", "BabyRabbit.IDLE_HEAD_TILT")):
        rows += [pose(layer, hop, millis) for millis in (0, 62, 125, 208, 250, 291, 375, 500, 583, 749, 750, 1100)]
        rows += [pose(layer, hop, 300, 0.5), pose(layer, hop, 300, 0.0)]
        rows += [pose(layer, tilt, millis) for millis in (0, 400, 1000, 2000, 3500, 3999, 4000, 6000)]
    yield {"case_id": "entity/keyframe-animation", "family": "entity",
           "operation": "keyframe_animation", "rows": rows}

    def steps(*calls):
        return list(calls)

    def call(name, age, **more):
        return {"call": name, "age": float_bits(age), **more}

    yield {"case_id": "entity/animation-states", "family": "entity", "operation": "animation_states",
           "rows": [
               {"label": "start-and-run", "steps": steps(
                   call("start", 10.0, tick=10), call("start", 10.5, tick=10), call("start", 25.25, tick=10))},
               {"label": "start-if-stopped", "steps": steps(
                   call("start_if_stopped", 3.0, tick=3), call("start_if_stopped", 9.5, tick=8),
                   call("stop", 12.0), call("start_if_stopped", 15.0, tick=14))},
               {"label": "animate-when", "steps": steps(
                   call("animate_when", 5.0, condition=True, tick=5),
                   call("animate_when", 9.0, condition=True, tick=9),
                   call("animate_when", 11.0, condition=False, tick=11),
                   call("animate_when", 20.0, condition=True, tick=20))},
               {"label": "fast-forward", "steps": steps(
                   call("fast_forward", 30.0, ticks=5, scale=float_bits(1.5)),
                   call("start", 30.0, tick=30),
                   call("fast_forward", 30.0, ticks=5, scale=float_bits(1.5)),
                   call("fast_forward", 30.5, ticks=7, scale=float_bits(0.4)))},
           ]}


def entity_glint_cases():
    """What decides an enchantment glint and how the reference draws it. `hasFoil` of real stacks:
    every item's default (the items with a glint override), a stack with enchantments, one with
    an override either way, one whose override is removed, and empty enchantments. The armor's
    glint render type: its pipeline and texture, and the matrix that moves the texture."""
    # A component patch as the wire carries it: how many are added, how many removed, then each added
    # as its id and value (enchantments: 13, a count and the enchantment ids and levels; the glint
    # override: 21 and a bool), then each removed id.
    foil = [{"label": "foil-defaults", "all": True},
            {"label": "foil-plain-armor", "item": "minecraft:diamond_chestplate", "patch": "0000"},
            {"label": "foil-enchanted-armor", "item": "minecraft:diamond_chestplate",
             "patch": "0100" + "0d" + "02" + "0003" + "0101"},
            {"label": "foil-enchanted-but-overridden-off", "item": "minecraft:diamond_chestplate",
             "patch": "0200" + "0d" + "01" + "0001" + "15" + "00"},
            {"label": "foil-overridden-on", "item": "minecraft:leather_boots", "patch": "0100" + "15" + "01"},
            {"label": "foil-removed-override", "item": "minecraft:enchanted_golden_apple",
             "patch": "0001" + "15"},
            {"label": "foil-no-enchantments", "item": "minecraft:iron_helmet", "patch": "0100" + "0d" + "00"},
            {"label": "foil-enchanted-book-overridden-off", "item": "minecraft:enchanted_book",
             "patch": "0100" + "15" + "00"}]
    yield {"case_id": "entity/item-foil", "family": "entity", "operation": "item_foil", "rows": foil}
    matrices = [{"label": f"glint-matrix-{millis}-{speed}", "millis": millis, "speed": speed}
                for millis, speed in [(0, 0.5), (1234, 0.5), (27500, 0.5), (55000, 0.5), (1_000_000, 0.5),
                                      (987654321, 0.5), (5000, 0.0), (5000, 1.0), (123456, 0.25)]]
    yield {"case_id": "entity/glint-layers", "family": "entity", "operation": "glint_layers", "rows": matrices}


def entity_equippable_cases():
    """The `equippable` component of real stacks: the item's own (from its definition), a patch that
    gives it another asset, a slot and an asset to an item that is not worn, no asset at all, a sound
    the patch names whole, and a patch that removes it. The component's wire form: the slot's id, the
    equip sound (a registry id plus one, or zero and the event), the asset (a flag and its name), the
    camera overlay and the allowed entities (each a flag), five flags and the shearing sound."""
    def name(text):
        return format(len(text), "02x") + text.encode().hex()

    def equippable(slot, asset, sound="01", shearing="01"):
        body = format(slot, "02x") + sound + ("01" + name(asset) if asset else "00") + "00" + "00" + "01" + "01" + "00" + "00" + "00" + shearing
        return "0100" + "20" + body

    rows = [{"label": "equippable-default-chestplate", "item": "minecraft:diamond_chestplate", "patch": "0000"},
            {"label": "equippable-default-stick", "item": "minecraft:stick", "patch": "0000"},
            {"label": "equippable-other-asset", "item": "minecraft:leather_helmet",
             "patch": equippable(4, "minecraft:netherite")},
            {"label": "equippable-on-a-stick", "item": "minecraft:stick", "patch": equippable(3, "minecraft:gold")},
            {"label": "equippable-no-asset", "item": "minecraft:iron_boots", "patch": equippable(1, None)},
            {"label": "equippable-whole-sound", "item": "minecraft:stick",
             "patch": equippable(2, "minecraft:chainmail", sound="00" + name("minecraft:entity.pig.ambient") + "00")},
            {"label": "equippable-whole-sound-range", "item": "minecraft:stick",
             "patch": equippable(7, "minecraft:saddle", sound="00" + name("minecraft:x") + "013f800000")},
            {"label": "equippable-removed", "item": "minecraft:diamond_chestplate", "patch": "0001" + "20"}]
    yield {"case_id": "entity/item-equippable", "family": "entity", "operation": "item_equippable", "rows": rows}


def entity_armor_stand_cases():
    """An armor stand's model and the models of the armor it wears (grown and
    small), posed by the reference's `ArmorStandModel`/`ArmorStandArmorModel`
    `setupAnim`: the default pose, a custom one, arms shown or not, the base
    plate and a turn."""
    default = {"head_pose": (0.0, 0.0, 0.0), "body_pose": (0.0, 0.0, 0.0),
               "left_arm_pose": (-10.0, 0.0, -10.0), "right_arm_pose": (-15.0, 0.0, 10.0),
               "left_leg_pose": (-1.0, 0.0, -1.0), "right_leg_pose": (1.0, 0.0, 1.0)}
    custom = {"head_pose": (12.5, -35.0, 7.25), "body_pose": (-8.0, 20.0, 3.0),
              "left_arm_pose": (-80.0, 15.0, -45.0), "right_arm_pose": (30.0, -60.0, 100.0),
              "left_leg_pose": (25.0, 0.0, -12.0), "right_leg_pose": (-35.0, 10.0, 12.0)}

    def pose(label, layer, poses, arms=False, plate=True, y_rot=0.0):
        row = {"label": f"{layer}-{label}", "layer": layer, "y_rot": float_bits(y_rot),
               "show_arms": arms, "show_base_plate": plate}
        for key, angles in poses.items():
            row[key] = [float_bits(angle) for angle in angles]
        return row

    layers = []
    for size in ("armor_stand", "armor_stand_small"):
        layers += [f"minecraft:{size}#{part}" for part in
                   ("main", "helmet", "chestplate", "leggings", "boots")]
    rows = []
    for layer in layers:
        rows += [
            pose("default", layer, default),
            pose("with-arms", layer, default, arms=True),
            pose("custom", layer, custom, arms=True, y_rot=47.5),
            pose("no-plate", layer, custom, plate=False, y_rot=-120.0),
        ]
    yield {"case_id": "entity/armor-stand-model", "family": "entity",
           "operation": "armor_stand_model", "rows": rows}


def entity_equipment_layer_cases():
    """Every equipment asset of the game's resources: the layers each names for
    the humanoid layer types (the texture and what it is multiplied by,
    undyed and dyed), through the reference's own codec."""
    yield {"case_id": "entity/equipment-layers", "family": "entity",
           "operation": "equipment_layers", "rows": [{"label": "all"}]}


def entity_xp_orb_cases():
    """An experience orb's quad as `ExperienceOrbRenderer.submit` writes it:
    each icon's texels, the colour swaying with its age, and the quad turned
    to the camera for several ways of looking, at several places."""
    def row(label, icon=3, age=0.0, yaw=0.0, pitch=0.0, place=(0.0, 0.0, 3.0), light=15728880):
        return {"label": label, "icon": icon, "age": float_bits(age), "yaw": float_bits(yaw),
                "pitch": float_bits(pitch), "px": float_bits(place[0]), "py": float_bits(place[1]),
                "pz": float_bits(place[2]), "light": light}

    rows = [row(f"icon-{icon}", icon=icon, age=12.5) for icon in range(11)]
    rows += [row(f"age-{age}", age=age) for age in (0.0, 1.5, 7.3, 18.85, 40.25, 99.9, 300.0, 5000.5)]
    for yaw, pitch in [(0.0, 0.0), (90.0, 0.0), (-135.5, 20.0), (37.5, -60.0), (180.0, 89.0), (400.0, 12.0)]:
        rows.append(row(f"looking-{yaw}-{pitch}", yaw=yaw, pitch=pitch, age=4.0))
    rows += [row("left-below", place=(-2.5, -1.25, 1.75), yaw=20.0, pitch=10.0),
             row("far", place=(30.5, 4.0, -22.25), yaw=-70.0, pitch=-5.0, age=77.7),
             row("dark", light=0), row("sky-lit", light=15 << 20), row("torch-lit", light=15 << 4)]
    yield {
        "case_id": "entity/xp-orb",
        "family": "entity",
        "operation": "experience_orb",
        "rows": rows,
    }
