"""The `entity` family: humanoids."""

from cases.helpers import *


def entity_zombie_cases():
    """The zombie family, grown and baby, from the layer table, posed by the
    reference's `HumanoidModel.setupAnim` and `animateZombieArms` (and the
    drowned's swim) and drawn by its part renderer: standing, looking about,
    walking, swinging, aggressive or not, and swimming."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             age=0.0, attack=0.0, aggressive=False, swim=0.0):
        return {"label": f"{layer}-{label}", "layer": layer,
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "age_in_ticks": float_bits(age), "attack_time": float_bits(attack),
                "aggressive": aggressive, "swim_amount": float_bits(swim)}

    rows = []
    for layer in ("minecraft:zombie#main", "minecraft:zombie_baby#main", "minecraft:husk#main",
                  "minecraft:giant#main", "minecraft:drowned#main", "minecraft:drowned_baby#main"):
        rows += [
            pose("standing", layer),
            pose("aggressive", layer, aggressive=True, age=17.5),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0, age=3.25),
            pose("walk-early", layer, walk_position=0.9, walk_speed=0.4, age=40.0),
            pose("walk-fast", layer, walk_position=21.6, walk_speed=1.0, age=123.456),
            pose("swing-early", layer, attack=0.1, age=5.0),
            pose("swing-mid", layer, attack=0.55, aggressive=True, age=9.0),
            pose("swing-late", layer, attack=0.9, walk_position=3.4, walk_speed=0.6),
            pose("walk-aggressive-swing", layer, head_y_rot=12.0, x_rot=-8.0, walk_position=7.3,
                 walk_speed=0.83, attack=0.3, aggressive=True, age=64.0),
        ]
    # The zombie villager's own classes pose as a zombie does; a few rows show it.
    for layer in ("minecraft:zombie_villager#main", "minecraft:zombie_villager_baby#main",
                  "minecraft:zombie_villager_no_hat#main"):
        rows += [
            pose("standing", layer),
            pose("walk-aggressive-swing", layer, head_y_rot=12.0, x_rot=-8.0, walk_position=7.3,
                 walk_speed=0.83, attack=0.3, aggressive=True, age=64.0),
        ]
    for layer in ("minecraft:drowned#main", "minecraft:drowned_baby#main", "minecraft:zombie#main"):
        rows += [pose("swimming-quarter", layer, swim=0.25, age=11.0, walk_position=2.0, walk_speed=0.5),
                 pose("swimming-full", layer, swim=1.0, age=33.3, head_y_rot=20.0, x_rot=15.0)]
    yield {
        "case_id": "entity/zombie-model",
        "family": "entity",
        "operation": "zombie_model",
        "rows": rows,
    }


def entity_skeleton_cases():
    """The skeleton family from the layer table, posed by `SkeletonModel.setupAnim`
    and drawn: the humanoid pose, and the arms an aggressive skeleton with no
    bow swings in front of it."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             age=0.0, attack=0.0, aggressive=False, bow=False, swim=0.0):
        return {"label": f"{layer}-{label}", "layer": layer,
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "age_in_ticks": float_bits(age), "attack_time": float_bits(attack),
                "swim_amount": float_bits(swim), "aggressive": aggressive, "holding_bow": bow}

    rows = []
    for layer in ("minecraft:skeleton#main", "minecraft:stray#main", "minecraft:stray#outer",
                  "minecraft:bogged#main", "minecraft:bogged#outer", "minecraft:parched#main",
                  "minecraft:wither_skeleton#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0, age=3.25),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83, age=40.0),
            pose("aggressive-with-bow", layer, aggressive=True, bow=True, age=17.5, attack=0.4),
            pose("aggressive-no-bow", layer, aggressive=True, age=17.5),
            pose("aggressive-swing-early", layer, aggressive=True, attack=0.15, age=5.0),
            pose("aggressive-swing-mid", layer, aggressive=True, attack=0.55, age=9.0,
                 walk_position=3.4, walk_speed=0.6),
            pose("aggressive-swing-late", layer, aggressive=True, attack=0.9, age=123.456,
                 head_y_rot=12.0, x_rot=-8.0),
            pose("swimming", layer, swim=0.6, age=11.0, walk_position=2.0, walk_speed=0.5),
        ]
    yield {
        "case_id": "entity/skeleton-model",
        "family": "entity",
        "operation": "skeleton_model",
        "rows": rows,
    }


def entity_piglin_cases():
    """The piglin family from the layer table, posed by `PiglinModel` and
    `ZombifiedPiglinModel.setupAnim` and drawn: the ears flapping with the walk,
    the arms of a zombified piglin held out, and a piglin dancing."""
    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             age=0.0, attack=0.0, aggressive=False, dancing=False, swim=0.0, pose=None, main="right",
             used=0.0, charge=25.0):
        return {"label": f"{layer}-{label}", "layer": layer,
                "arm_pose": pose or ("dancing" if dancing else "default"), "main_arm": main,
                "ticks_using_item": float_bits(used), "max_crossbow_charge": float_bits(charge),
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "age_in_ticks": float_bits(age), "attack_time": float_bits(attack),
                "swim_amount": float_bits(swim), "aggressive": aggressive, "dancing": dancing}

    rows = []
    for layer in ("minecraft:piglin#main", "minecraft:piglin_baby#main", "minecraft:piglin_brute#main",
                  "minecraft:zombified_piglin#main", "minecraft:zombified_piglin_baby#main"):
        rows += [
            pose("standing", layer),
            pose("ears-in-the-wind", layer, age=57.5),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83, age=40.0),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0, age=3.25),
            pose("swing", layer, attack=0.5, age=9.0),
            pose("aggressive", layer, aggressive=True, attack=0.25, age=21.0),
            pose("swimming", layer, swim=0.5, age=4.0, walk_position=2.0, walk_speed=0.4),
        ]
        if "zombified" not in layer:
            for arm in ("right", "left"):
                rows += [
                    pose(f"melee-{arm}", layer, pose="attacking_with_melee_weapon", main=arm, age=11.0,
                         walk_position=2.0, walk_speed=0.3),
                    pose(f"melee-swing-{arm}", layer, pose="attacking_with_melee_weapon", main=arm,
                         attack=0.35, age=11.0, aggressive=True),
                    pose(f"melee-swing-late-{arm}", layer, pose="attacking_with_melee_weapon", main=arm,
                         attack=0.85, age=3.5, head_y_rot=10.0, x_rot=-5.0),
                    pose(f"crossbow-hold-{arm}", layer, pose="crossbow_hold", main=arm, head_y_rot=-14.0,
                         x_rot=20.0),
                    pose(f"admiring-{arm}", layer, pose="admiring_item", main=arm, age=7.0,
                         head_y_rot=30.0, x_rot=-12.0),
                    pose(f"admiring-swing-{arm}", layer, pose="admiring_item", main=arm, attack=0.4),
                ]
                rows += [pose(f"crossbow-charge-{arm}-{used}", layer, pose="crossbow_charge", main=arm,
                              used=used) for used in (0.0, 6.0, 12.5, 25.0, 40.0)]
            rows += [pose("dancing", layer, dancing=True, age=15.0),
                     pose("dancing-late", layer, dancing=True, age=333.3, head_y_rot=10.0)]
    yield {
        "case_id": "entity/piglin-model",
        "family": "entity",
        "operation": "piglin_model",
        "rows": rows,
    }


def entity_spider_cases():
    """The spider and the cave spider from the layer table, posed by the
    reference's `SpiderModel.setupAnim` (eight legs, each pair a quarter of a
    stride apart) and the creeper by `CreeperModel.setupAnim` (the legs
    swinging in diagonal pairs), drawn."""
    def pose(label, layer, operation, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0):
        return {"label": f"{layer}-{label}", "layer": layer,
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed)}

    def rows(layers, operation):
        out = []
        for layer in layers:
            out += [
                pose("standing", layer, operation),
                pose("looking", layer, operation, head_y_rot=-35.5, x_rot=48.0),
                pose("walking-early", layer, operation, walk_position=0.9, walk_speed=0.4),
                pose("walking", layer, operation, walk_position=7.3, walk_speed=0.83),
                pose("running", layer, operation, walk_position=21.6, walk_speed=1.0,
                     head_y_rot=12.0, x_rot=-8.0),
            ]
        return out

    yield {"case_id": "entity/spider-model", "family": "entity", "operation": "spider_model",
           "rows": rows(("minecraft:spider#main", "minecraft:cave_spider#main"), "spider_model")}
    yield {"case_id": "entity/creeper-model", "family": "entity", "operation": "creeper_model",
           "rows": rows(("minecraft:creeper#main", "minecraft:creeper#armor"), "creeper_model")}


def entity_villager_cases():
    """The villagers: the built-in registries' order of the types and the
    professions with the hat each texture's metadata asks for; the models
    (the villager and the wandering trader, grown and baby, hatted or not)
    posed by the reference's `VillagerModel.setupAnim`; and the passes of the
    reference's own `VillagerProfessionLayer` (the model, the texture, the
    colour) for each type and profession, grown and baby, at each level."""
    yield {"case_id": "entity/villager-registries", "family": "entity",
           "operation": "villager_registries", "rows": [{"label": "all"}]}

    def pose(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             age=0.0, unhappy=False):
        return {"label": f"{layer}-{label}", "layer": layer,
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "age_in_ticks": float_bits(age), "unhappy": unhappy}

    rows = []
    for layer in ("minecraft:villager#main", "minecraft:villager_baby#main",
                  "minecraft:villager_no_hat#main", "minecraft:villager_baby_no_hat#main",
                  "minecraft:wandering_trader#main"):
        rows += [
            pose("standing", layer),
            pose("looking", layer, head_y_rot=-35.5, x_rot=48.0, age=3.25),
            pose("walking", layer, walk_position=7.3, walk_speed=0.83, age=40.0),
            pose("unhappy", layer, unhappy=True, age=17.5),
            pose("unhappy-walking", layer, unhappy=True, walk_position=2.0, walk_speed=0.4,
                 head_y_rot=12.0, x_rot=-8.0, age=63.0),
        ]
    yield {"case_id": "entity/villager-model", "family": "entity",
           "operation": "villager_model", "rows": rows}

    types = ("desert", "jungle", "plains", "savanna", "snow", "swamp", "taiga")
    professions = ("none", "armorer", "butcher", "cartographer", "cleric", "farmer", "fisherman",
                   "fletcher", "leatherworker", "librarian", "mason", "nitwit", "shepherd",
                   "toolsmith", "weaponsmith")
    layer_rows = []
    for path in ("villager", "zombie_villager"):
        for baby in (False, True):
            for type_name in types:
                for profession in professions:
                    layer_rows.append({"label": f"{path}-{'baby-' if baby else ''}{type_name}-{profession}-3",
                                       "path": path, "baby": baby, "type": type_name,
                                       "profession": profession, "level": 3})
        for level in (0, 1, 2, 4, 5, 9):
            layer_rows.append({"label": f"{path}-plains-farmer-level-{level}", "path": path,
                               "baby": False, "type": "plains", "profession": "farmer", "level": level})
    yield {"case_id": "entity/villager-layers", "family": "entity",
           "operation": "villager_layers", "rows": layer_rows}


def entity_illager_cases():
    """The illagers' and the witch's models posed by the reference's own
    `IllagerModel.setupAnim` and `WitchModel.setupAnim`: every arm pose the
    illager model branches on (with and without a held item, either main arm,
    a crossbow charged from nothing to past its duration), riding, walking and
    looking; the witch's nose by the entity's number and by what it holds."""
    def pose(label, layer="minecraft:evoker#main", arm_pose="NEUTRAL", main_arm="right",
             head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, age=0.0,
             riding=False, attack_time=0.0, has_item=False, max_charge=0, ticks_using=0.0,
             hat=False):
        return {"label": f"{layer.split(':')[1].split('#')[0]}-{label}", "layer": layer,
                "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "age_in_ticks": float_bits(age), "riding": riding, "arm_pose": arm_pose,
                "main_arm": main_arm, "attack_time": float_bits(attack_time),
                "has_item": has_item, "max_crossbow_charge": max_charge,
                "ticks_using_item": float_bits(ticks_using), "hat": hat}

    rows = []
    for layer in ("minecraft:evoker#main", "minecraft:vindicator#main",
                  "minecraft:pillager#main", "minecraft:illusioner#main"):
        hat = layer.endswith("illusioner#main")
        rows += [
            pose("crossed", layer, "CROSSED", hat=hat),
            pose("neutral-walking", layer, walk_position=7.3, walk_speed=0.83, age=40.0,
                 head_y_rot=-35.5, x_rot=48.0, hat=hat),
        ]
    layer = "minecraft:evoker#main"
    rows += [
        pose("riding", arm_pose="NEUTRAL", riding=True, head_y_rot=12.0, x_rot=-8.0),
        pose("attacking-empty", arm_pose="ATTACKING", attack_time=0.35, age=21.5),
        pose("attacking-item-right", arm_pose="ATTACKING", has_item=True, attack_time=0.35, age=21.5),
        pose("attacking-item-left", arm_pose="ATTACKING", main_arm="left", has_item=True,
             attack_time=0.7, age=9.25),
        pose("spellcasting", arm_pose="SPELLCASTING", age=13.75),
        pose("bow", arm_pose="BOW_AND_ARROW", head_y_rot=20.0, x_rot=-15.0),
        pose("crossbow-hold", arm_pose="CROSSBOW_HOLD", head_y_rot=-20.0, x_rot=10.0),
        pose("celebrating", arm_pose="CELEBRATING", age=33.0),
    ]
    for ticks in (0.0, 12.5, 40.0):
        rows.append(pose(f"crossbow-charge-{ticks}", arm_pose="CROSSBOW_CHARGE", max_charge=25,
                         ticks_using=ticks))
    yield {"case_id": "entity/illager-model", "family": "entity", "operation": "illager_model",
           "rows": rows}

    def witch(label, entity_id=0, holding=False, age=0.0, head_y_rot=0.0, x_rot=0.0,
              walk_position=0.0, walk_speed=0.0):
        return {"label": label, "entity_id": entity_id, "holding_item": holding,
                "age_in_ticks": float_bits(age), "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed)}

    yield {"case_id": "entity/witch-model", "family": "entity", "operation": "witch_model",
           "rows": [
               witch("standing"),
               witch("sniffing-id-7", entity_id=7, age=31.5, head_y_rot=-35.5, x_rot=48.0),
               witch("sniffing-id-13", entity_id=13, age=9.0),
               witch("walking", entity_id=3, age=40.0, walk_position=7.3, walk_speed=0.83),
               witch("holding", entity_id=7, holding=True, age=31.5),
           ]}


def entity_golem_cases():
    """The golems and the enderman posed by the reference's own `setupAnim`
    (`IronGolemModel`, `SnowGolemModel`, `EndermanModel`); the pose the iron
    golem's renderer adds while it walks and the crackiness level a health
    fraction gives, by the reference's own methods."""
    def look(label, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0, age=0.0, **more):
        return {"label": label, "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
                "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
                "age_in_ticks": float_bits(age), **more}

    def iron(label, attack_ticks=0.0, offer=0, **kwargs):
        return look(label, attack_ticks=float_bits(attack_ticks), offer_flower_tick=offer, **kwargs)

    yield {"case_id": "entity/iron-golem-model", "family": "entity", "operation": "iron_golem_model",
           "rows": [
               iron("standing"),
               iron("looking", head_y_rot=-35.5, x_rot=48.0),
               iron("walking", walk_position=7.3, walk_speed=0.83),
               iron("walking-fast", walk_position=21.6, walk_speed=1.0, head_y_rot=12.0, x_rot=-8.0),
               iron("attacking-early", attack_ticks=9.5, walk_position=7.3, walk_speed=0.83),
               iron("attacking-late", attack_ticks=2.25),
               iron("offering-a-flower", offer=400, walk_position=7.3, walk_speed=0.83),
               iron("offering-a-flower-late", offer=33),
               iron("attacking-while-offering", attack_ticks=4.0, offer=100),
           ]}
    yield {"case_id": "entity/snow-golem-model", "family": "entity", "operation": "snow_golem_model",
           "rows": [
               look("standing"),
               look("looking", head_y_rot=-35.5, x_rot=48.0),
               look("looking-the-other-way", head_y_rot=120.0, x_rot=-20.0),
           ]}
    yield {"case_id": "entity/enderman-model", "family": "entity", "operation": "enderman_model",
           "rows": [
               look("standing", attack_time=float_bits(0.0), swim_amount=float_bits(0.0),
                    creepy=False, carrying=False),
               look("walking", walk_position=7.3, walk_speed=0.83, age=40.0,
                    attack_time=float_bits(0.0), swim_amount=float_bits(0.0),
                    creepy=False, carrying=False),
               look("walking-fast-looking", walk_position=21.6, walk_speed=1.0, head_y_rot=12.0,
                    x_rot=-8.0, attack_time=float_bits(0.0), swim_amount=float_bits(0.0),
                    creepy=False, carrying=False),
               look("creepy", head_y_rot=-35.5, x_rot=48.0, attack_time=float_bits(0.0),
                    swim_amount=float_bits(0.0), creepy=True, carrying=False),
               look("carrying", walk_position=7.3, walk_speed=0.83, attack_time=float_bits(0.0),
                    swim_amount=float_bits(0.0), creepy=False, carrying=True),
               look("carrying-and-creepy", attack_time=float_bits(0.0), swim_amount=float_bits(0.0),
                    creepy=True, carrying=True),
           ]}

    def render(label, walk_position, walk_speed, body_rot, fraction):
        return {"label": label, "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "body_rot": float_bits(body_rot),
                "fraction": float_bits(fraction)}

    rows = [
        render("still", 3.0, 0.0, 30.0, 1.0),
        render("barely-moving", 3.0, 0.009, 30.0, 1.0),
        render("walk-start", 0.0, 0.5, 90.0, 0.8),
        render("walk-mid", 7.3, 0.83, 90.0, 0.75),
        render("walk-late", 21.6, 1.0, -45.0, 0.74),
        render("walk-far", 1234.5, 0.6, 200.0, 0.5),
    ]
    rows += [render(f"health-{fraction}", 5.0, 0.7, 10.0, fraction)
             for fraction in (0.0, 0.1, 0.2499, 0.25, 0.4999, 0.5, 0.7499, 0.75, 0.9, 1.0)]
    # What the charged creeper's swirl layer submits, at several ages, charged and not.
    yield {"case_id": "entity/swirl-layers", "family": "entity", "operation": "swirl_layers",
           "rows": [{"label": f"age-{age}-{'charged' if powered else 'plain'}",
                     "age_in_ticks": float_bits(age), "powered": powered}
                    for age in (0.0, 7.5, 123.25, 1000.5, 99999.75) for powered in (True, False)]}
    # What the glowing-eyes layers submit: the draw order, the texture, the pipeline's
    # shader flags, blend and depth state, the light and the overlay.
    yield {"case_id": "entity/eyes-layers", "family": "entity", "operation": "eyes_layers",
           "rows": [{"label": "spider", "layer": "spider"}, {"label": "enderman", "layer": "enderman"}]}

    def block_layer(label, layer, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
                    attack_ticks=0.0, offer=0):
        return {"label": label, "layer": layer, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "walk_position": float_bits(walk_position),
                "walk_speed": float_bits(walk_speed), "attack_ticks": float_bits(attack_ticks),
                "offer_flower_tick": offer}

    # Where the layers that draw a block on an entity put it: the enderman's carried
    # block, the snow golem's pumpkin (on the head's pose) and the iron golem's flower
    # (on the right arm's pose).
    yield {"case_id": "entity/block-layers", "family": "entity", "operation": "entity_block_layers",
           "rows": [
               block_layer("carried", "carried"),
               block_layer("mushrooms-straight", "mushrooms"),
               block_layer("mushrooms-looking", "mushrooms", head_y_rot=-35.5, x_rot=48.0),
               block_layer("pumpkin-straight", "snow_head"),
               block_layer("pumpkin-looking", "snow_head", head_y_rot=-35.5, x_rot=48.0),
               block_layer("pumpkin-turned", "snow_head", head_y_rot=120.0, x_rot=-20.0),
               block_layer("flower-offered", "flower", offer=400),
               block_layer("flower-offered-late", "flower", offer=33, head_y_rot=12.0, x_rot=-8.0),
               block_layer("flower-while-striking", "flower", attack_ticks=4.0, offer=100),
               block_layer("flower-while-walking", "flower", offer=70, walk_position=7.3,
                           walk_speed=0.83),
           ]}
    yield {"case_id": "entity/iron-golem-render", "family": "entity",
           "operation": "iron_golem_render", "rows": rows}


def entity_villager_held_cases():
    """Where the layers that hold something on a villager, a wandering trader and a witch
    put it: the crossed arms' item, the witch's item and potion, and the head item (with the
    villager's transforms and the default's), from the reference's own layer classes."""
    def row(label, kind, baby=False, head_y_rot=0.0, x_rot=0.0, age=0.0, unhappy=False,
            entity_id=0, villager_transforms=True):
        return {"label": label, "kind": kind, "baby": baby, "head_y_rot": float_bits(head_y_rot),
                "x_rot": float_bits(x_rot), "age_in_ticks": float_bits(age), "unhappy": unhappy,
                "entity_id": entity_id, "villager_transforms": villager_transforms}

    yield {"case_id": "entity/villager-held-layers", "family": "entity",
           "operation": "villager_held_layers", "rows": [
               row("crossed-arms", "crossed"),
               row("crossed-arms-baby", "crossed", baby=True),
               row("witch-item", "witch_item", head_y_rot=-35.5, x_rot=48.0, age=31.5, entity_id=7),
               row("witch-potion", "witch_potion", age=31.5, entity_id=7),
               row("witch-potion-looking", "witch_potion", head_y_rot=12.0, x_rot=-8.0, age=9.0,
                   entity_id=13),
               row("head-villager", "villager_head", head_y_rot=-35.5, x_rot=48.0, age=3.25),
               row("head-villager-unhappy", "villager_head", unhappy=True, age=17.5),
               row("head-baby-villager", "villager_head", baby=True, head_y_rot=12.0, x_rot=-8.0),
               row("head-trader", "villager_head", villager_transforms=False, head_y_rot=-35.5,
                   x_rot=48.0),
           ]}


def entity_copper_golem_cases():
    """The copper golem posed by the reference's own `CopperGolemModel.setupAnim`: the look, the
    walk (with arms raised while an item is held, the arms never lower than that), the idle spin and
    the four chest animations, each started at a tick and read at a later age."""
    def pose(label, age=0.0, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             holds_item=False, **starts):
        row = {"label": label, "head_y_rot": float_bits(head_y_rot), "x_rot": float_bits(x_rot),
               "walk_position": float_bits(walk_position), "walk_speed": float_bits(walk_speed),
               "age_in_ticks": float_bits(age), "holds_item": holds_item}
        row.update(starts)
        return row

    rows = [
        pose("standing", head_y_rot=-35.5, x_rot=48.0),
        pose("walking", walk_position=7.3, walk_speed=0.31, age=20.5),
        pose("running", walk_position=19.75, walk_speed=0.9, head_y_rot=12.0, x_rot=-8.0),
        pose("holding-still", holds_item=True),
        pose("holding-looking", holds_item=True, head_y_rot=20.0, x_rot=15.0),
        pose("holding-walking-slowly", holds_item=True, walk_position=3.1, walk_speed=0.12),
        pose("holding-walking", holds_item=True, walk_position=8.9, walk_speed=0.45),
        pose("holding-running", holds_item=True, walk_position=21.4, walk_speed=1.0),
    ]
    for age in (10.0, 11.5, 14.25, 19.0, 30.0, 55.5, 80.0):
        rows.append(pose(f"idle-{age}", age=age, idle=10))
    for key in ("get_item", "get_no_item", "drop_item", "drop_no_item"):
        for age in (5.0, 12.5, 25.0, 41.75, 70.0, 110.0, 190.0):
            rows.append(pose(f"{key}-{age}", age=age, **{key: 5}))
    rows += [
        pose("walking-and-idle", age=24.0, walk_position=6.0, walk_speed=0.5, idle=10),
        pose("holding-and-dropping", age=33.0, holds_item=True, walk_position=2.0, walk_speed=0.2,
             drop_item=5, head_y_rot=-14.0, x_rot=6.0),
    ]
    yield {"case_id": "entity/copper-golem-model", "family": "entity",
           "operation": "copper_golem_model", "rows": rows}


def entity_copper_golem_layer_cases():
    """Where the layers that carry things on a copper golem put them, from the reference's own
    `ItemInHandLayer`, `CustomHeadLayer` and `BlockDecorationLayer` on the model as posed, idle and
    at a chest (the hand's transform differs), for the look and walk."""
    rows = []
    for layer in ("hands", "head", "antenna"):
        for action in ("IDLE", "GETTING_ITEM", "DROPPING_NO_ITEM"):
            for label, look, walk in (
                ("standing", (0.0, 0.0), (0.0, 0.0)),
                ("looking", (-35.5, 48.0), (0.0, 0.0)),
                ("walking", (12.0, -8.0), (7.3, 0.6)),
            ):
                rows.append({
                    "label": f"{layer}-{action.lower()}-{label}", "layer": layer, "action": action,
                    "head_y_rot": float_bits(look[0]), "x_rot": float_bits(look[1]),
                    "walk_position": float_bits(walk[0]), "walk_speed": float_bits(walk[1]),
                    "age_in_ticks": float_bits(21.5)})
    yield {"case_id": "entity/copper-golem-layers", "family": "entity",
           "operation": "copper_golem_layers", "rows": rows}


def entity_insect_cases():
    """The silverfish, the endermite and the bee, posed by the reference's own `setupAnim` (the
    segments' wave, the bee's wings, legs and bobbing, its roll and its stinger) and the bee's own
    tick: its roll amount follows the rolling flag, its anger the game time."""
    def age(label, layer, age):
        return {"label": f"{layer}-{label}", "layer": layer, "age_in_ticks": float_bits(age)}

    rows = []
    for layer in ("minecraft:silverfish#main", "minecraft:endermite#main"):
        rows += [age("start", layer, 0.0), age("early", layer, 3.5), age("late", layer, 47.25),
                 age("long", layer, 1234.5)]
    yield {"case_id": "entity/segmented-model", "family": "entity",
           "operation": "segmented_model", "rows": rows}

    def bee(label, layer, age=0.0, roll=0.0, stinger=True, ground=False, angry=False):
        return {"label": f"{layer}-{label}", "layer": layer, "age_in_ticks": float_bits(age),
                "roll_amount": float_bits(roll), "has_stinger": stinger, "on_ground": ground,
                "angry": angry}

    rows = []
    for layer in ("minecraft:bee#main", "minecraft:bee_baby#main"):
        rows += [
            bee("flying", layer, age=12.5),
            bee("flying-later", layer, age=70.25),
            bee("angry", layer, age=33.0, angry=True),
            bee("on-the-ground", layer, age=33.0, ground=True),
            bee("stung", layer, age=33.0, stinger=False),
            bee("rolling-half", layer, age=21.0, roll=0.5, angry=True),
            bee("rolling-full", layer, age=21.0, roll=1.0, angry=True, stinger=False),
        ]
    yield {"case_id": "entity/bee-model", "family": "entity", "operation": "bee_model", "rows": rows}


def entity_creeper_render_cases():
    """What the creeper's renderer does to a swelling creeper: the scale it
    applies to the pose (wobbling as the fuse nears its end) and how far it
    flashes white."""
    rows = [{"label": f"swelling-{value}", "swelling": float_bits(value)}
            for value in (0.0, 0.05, 0.1, 0.19, 0.25, 0.37, 0.5, 0.64, 0.8, 0.95, 1.0, 1.0714286)]
    yield {"case_id": "entity/creeper-render", "family": "entity",
           "operation": "creeper_render", "rows": rows}
