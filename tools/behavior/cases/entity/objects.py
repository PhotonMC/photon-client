"""The `entity` family: objects."""

from cases.helpers import *


def entity_boat_cases():
    """The boat models (a boat, a chest boat, a raft, a chest raft, of some woods) posed by the
    reference's own `setupAnim` (each paddle by the time it has rowed, the right one mirrored, the
    time sin-wrapped and clamped), and where `AbstractBoatRenderer.submit` draws
    a boat: turned, rocked by a hit (by how long since and how hard), shaken by a bubble column
    (not under water) and flipped."""
    def state(label, y_rot=0.0, hurt_dir=1, hurt_time=0.0, damage_time=0.0, bubble=0.0,
              under_water=False, left=0.0, right=0.0, **extra):
        row = {"label": label, "y_rot": float_bits(y_rot), "hurt_dir": hurt_dir,
               "hurt_time": float_bits(hurt_time), "damage_time": float_bits(damage_time),
               "bubble_angle": float_bits(bubble), "under_water": under_water,
               "rowing_left": float_bits(left), "rowing_right": float_bits(right)}
        row.update(extra)
        return row
    rows = []
    for layer in ("boat/oak", "chest_boat/oak", "boat/bamboo", "chest_boat/bamboo", "boat/cherry"):
        name = layer.replace("/", "-")
        for label, left, right in (("still", 0.0, 0.0), ("rowing", 0.7, 1.9), ("rowed-far", 3.2, 5.5),
                                   ("rowing-one-side", 2.4, 0.0)):
            rows.append(state(f"{name}-{label}", left=left, right=right, layer=f"minecraft:{layer}#main"))
    yield {"case_id": "entity/boat-model", "family": "entity", "operation": "boat_model", "rows": rows}
    poses = [state("still"), state("turned", y_rot=37.0), state("turned-far", y_rot=-170.5),
             state("hit", hurt_time=7.5, damage_time=12.0, hurt_dir=1),
             state("hit-the-other-way", hurt_time=3.25, damage_time=30.0, hurt_dir=-1),
             state("hit-just-over", hurt_time=0.0, damage_time=12.0),
             state("shaken-by-bubbles", bubble=6.5), state("shaken-under-water", bubble=6.5, under_water=True),
             state("shaken-a-little", bubble=0.000_005), state("everything", y_rot=90.0, hurt_time=5.0,
                                                               damage_time=20.0, hurt_dir=-1, bubble=-8.0)]
    for row in poses:
        row.update({"feet_x": float_bits(10.5), "feet_y": float_bits(64.25), "feet_z": float_bits(-3.75)})
    yield {"case_id": "entity/boat-pose", "family": "entity", "operation": "boat_pose", "rows": poses}


def entity_minecart_pose_cases():
    """Where the reference's own minecart renderer draws a minecart and the block it carries, from the
    state its old behaviour extracts: off a rail (the turn it was sent), along a rail (flat, along
    each axis, sloping up and down, the two ends differing in height, at a corner), with the
    rocking of a hit, with and without a block, for some ids (the shift that keeps two apart)."""
    def vec(x, y, z):
        return [float_bits(x, "f64"), float_bits(y, "f64"), float_bits(z, "f64")]

    def cart(label, rail=None, at=(10.5, 64.0, -3.5), x_rot=0.0, y_rot=0.0, entity_id=7, hurt=(0, 0.0, 0.0),
             block=False, offset=6):
        seed = entity_id * 493286711
        seed = ((seed * seed * 4392167121) + (seed * 98761))
        seed = ((seed + 2 ** 63) % 2 ** 64) - 2 ** 63
        row = {"label": f"minecart-{label}", "feet_x": float_bits(at[0]), "feet_y": float_bits(at[1]),
               "feet_z": float_bits(at[2]), "x": float_bits(at[0], "f64"), "y": float_bits(at[1], "f64"),
               "z": float_bits(at[2], "f64"), "x_rot": float_bits(x_rot), "y_rot": float_bits(y_rot),
               "offset_seed": str(seed), "hurt_dir": hurt[0], "hurt_time": float_bits(hurt[1]),
               "damage_time": float_bits(hurt[2]), "display_offset": offset, "block": block}
        if rail:
            row["rail"] = {"pos": vec(*rail[0]), "front": vec(*rail[1]), "back": vec(*rail[2])}
        return row
    def tnt(label, fuse, **kwargs):
        row = cart(label, block=True, **kwargs)
        row["tnt_fuse"] = float_bits(fuse)
        return row
    rows = [cart("off-rail"), cart("off-rail-turned", y_rot=123.0, x_rot=11.0),
            cart("flat-along-x", rail=((10.5, 64.0625, -3.5), (10.8, 64.0625, -3.5), (10.2, 64.0625, -3.5))),
            cart("flat-along-z", rail=((10.5, 64.0625, -3.5), (10.5, 64.0625, -3.2), (10.5, 64.0625, -3.8))),
            cart("sloping-up", rail=((10.5, 64.5, -3.5), (10.8, 64.8, -3.5), (10.2, 64.2, -3.5))),
            cart("sloping-down", rail=((10.5, 64.5, -3.5), (10.2, 64.8, -3.5), (10.8, 64.2, -3.5))),
            cart("corner", rail=((10.4, 64.0625, -3.4), (10.7, 64.0625, -3.5), (10.4, 64.0625, -3.1))),
            cart("ends-meet", rail=((10.5, 64.0625, -3.5), (10.5, 64.0625, -3.5), (10.5, 64.0625, -3.5))),
            cart("hit", hurt=(1, 7.5, 12.0)), cart("hit-the-other-way", hurt=(-1, 3.25, 40.0)),
            cart("hit-over", hurt=(1, -0.5, 12.0)), cart("another-id", entity_id=123456),
            cart("with-a-block", block=True), cart("with-a-block-low", block=True, offset=8),
            cart("with-a-block-high", block=True, offset=1),
            tnt("tnt-unlit", -1.0), tnt("tnt-long-fuse", 40.0), tnt("tnt-fuse-ten", 10.0), tnt("tnt-fuse-nine", 9.0),
            tnt("tnt-fuse-five-lit", 5.0), tnt("tnt-fuse-four-lit", 4.99), tnt("tnt-fuse-three", 3.0),
            tnt("tnt-fuse-one", 1.0), tnt("tnt-about-to-blow", 0.25), tnt("tnt-at-zero", 0.0),
            tnt("tnt-lit-on-a-rail", 7.5, rail=((10.5, 64.5, -3.5), (10.8, 64.8, -3.4), (10.2, 64.2, -3.6)), hurt=(1, 4.0, 10.0)),
            cart("everything", rail=((10.5, 64.5, -3.5), (10.8, 64.8, -3.4), (10.2, 64.2, -3.6)), hurt=(-1, 5.0, 20.0),
                 block=True, entity_id=99, x_rot=4.0, y_rot=-50.0, at=(-100.25, 70.5, 33.75))]
    # The new behaviour draws from the turns alone, whatever rail there is; a hit and a block still apply.
    new_render = []
    for row in [cart("new-flat"), cart("new-turned-and-sloped", y_rot=123.0, x_rot=-37.0),
                cart("new-ignores-the-rail", rail=((10.5, 64.5, -3.5), (10.8, 64.8, -3.4), (10.2, 64.2, -3.6)),
                     y_rot=-80.0, x_rot=22.5),
                cart("new-hit-with-a-block", hurt=(-1, 5.0, 20.0), block=True, entity_id=99, y_rot=45.0, x_rot=10.0)]:
        row["new_render"] = True
        new_render.append(row)
    rows += new_render
    yield {"case_id": "entity/minecart-pose", "family": "entity", "operation": "minecart_pose", "rows": rows}


def entity_item_frame_pose_cases():
    """Where the reference's own item frame renderer draws a frame's model and the item in it, for
    each way it faces (the four around, and up and down, which turn by an axis), with and without
    the frame (an invisible one hangs the item further out) and the item turned by each eighth."""
    def frame(label, direction, invisible=False, rotation=0, item=True, glow=False, map_=False,
              light=15728880, feet=(10.5, 64.25, -3.5)):
        return {"label": f"item-frame-{label}", "direction": direction, "invisible": invisible,
                "rotation": rotation, "item": item, "glow": glow, "map": map_, "light": light,
                "feet_x": float_bits(feet[0]), "feet_y": float_bits(feet[1]), "feet_z": float_bits(feet[2])}
    rows = [frame(direction, direction) for direction in ["north", "south", "west", "east", "up", "down"]]
    rows += [frame(f"turned-{turn}", "east", rotation=turn) for turn in range(8)]
    rows += [frame("empty", "north", item=False), frame("invisible", "south", invisible=True),
             frame("invisible-up", "up", invisible=True, rotation=3),
             frame("far", "down", rotation=5, feet=(-100.25, 70.5, 33.75))]
    # A frame that holds a map draws the map in the item's place, turned by quarter turns, and lit
    # as the map is (the glow frame's way when it is one).
    rows += [frame(f"map-{turn}", "north", rotation=turn, map_=True, light=0x00A000C0) for turn in (0, 1, 2, 3, 5)]
    rows += [frame("map-glow", "up", map_=True, glow=True, light=0x00A000C0, invisible=True),
             frame("map-south", "south", map_=True, rotation=2)]
    yield {"case_id": "entity/item-frame-pose", "family": "entity", "operation": "item_frame_pose", "rows": rows}


def entity_display_pose_cases():
    """Where the reference's own display renderers draw a block or an item display, from the render
    state an entity makes: each billboard constraint (the entity's turns, the camera's, or a mix),
    a transformation that keeps normals (a turn and a uniform scale), one that does not (a scale
    that differs by axis, with the turns before and after it), and the item's turn about."""
    def bits(values):
        return [float_bits(value) for value in values]

    quarter = [0.0, 0.7071068, 0.0, 0.7071068]
    tilted = [0.2418448, 0.0, 0.0, 0.9703133]
    def display(label, kind="block", billboard=0, translation=(0.0, 0.0, 0.0), scale=(1.0, 1.0, 1.0),
                left=(0.0, 0.0, 0.0, 1.0), right=(0.0, 0.0, 0.0, 1.0), entity=(33.0, -12.5),
                camera=(110.0, 25.0), feet=(10.5, 64.25, -3.5)):
        return {"label": f"display-{label}", "kind": kind, "billboard": billboard,
                "translation": bits(translation), "scale": bits(scale), "left_rotation": bits(left),
                "right_rotation": bits(right), "entity_y_rot": float_bits(entity[0]),
                "entity_x_rot": float_bits(entity[1]), "camera_y_rot": float_bits(camera[0]),
                "camera_x_rot": float_bits(camera[1]), "feet_x": float_bits(feet[0]),
                "feet_y": float_bits(feet[1]), "feet_z": float_bits(feet[2])}
    rows = [display(f"billboard-{name}", billboard=index, translation=(0.25, -0.5, 1.0), left=quarter)
            for index, name in enumerate(["fixed", "vertical", "horizontal", "center"])]
    rows += [display("uniform-scale", scale=(2.0, 2.0, 2.0), left=tilted, translation=(-1.0, 0.5, 0.0)),
             display("squashed", scale=(2.0, 0.5, 1.0), left=quarter, right=tilted, translation=(0.0, 1.0, 0.0)),
             display("squashed-center", billboard=3, scale=(1.0, 3.0, 0.25), left=tilted, right=quarter),
             display("mirrored", scale=(-1.0, 1.0, 1.0), left=tilted),
             display("item", kind="item", billboard=3, translation=(0.0, 0.5, 0.0), scale=(0.5, 0.5, 0.5), left=tilted),
             display("item-fixed", kind="item", right=quarter),
             display("no-turns", entity=(0.0, 0.0), camera=(0.0, 0.0)),
             display("far", scale=(1.5, 1.5, 1.5), left=quarter, feet=(-100.25, 70.5, 33.75))]
    yield {"case_id": "entity/display-pose", "family": "entity", "operation": "display_pose", "rows": rows}


def entity_display_light_cases():
    """The light the reference's own display renderer uses when the server set a brightness: its
    block and sky levels, whatever else the number holds."""
    rows = [{"label": f"display-light-{label}", "brightness": value} for label, value in
            [("both", (15 << 4) | (10 << 20)), ("dark", 0), ("block-only", 3 << 4), ("sky-only", 7 << 20),
             ("stray-bits", 0x7FFFFFFF), ("negative", -2)]]
    yield {"case_id": "entity/display-light", "family": "entity", "operation": "display_light", "rows": rows}


def entity_chest_cases():
    """The chest's three models with the lid shut and at a few openings, and where the real
    `ChestRenderer` draws a chest of each type, material and facing, at each openness."""
    models = []
    for layer in ("minecraft:chest#main", "minecraft:double_chest_left#main",
                  "minecraft:double_chest_right#main"):
        for open_ in (0.0, 0.25, 0.7, 1.0):
            models.append({"label": f"{layer} {open_}", "layer": layer, "open": float_bits(open_)})
    yield {"case_id": "entity/chest-model", "family": "entity", "operation": "chest_model",
           "rows": models}
    poses = []
    for facing in ("north", "east", "south", "west"):
        for kind in ("single", "left", "right"):
            poses.append({"label": f"{facing} {kind}", "type": kind, "facing": facing,
                          "material": "REGULAR", "open": float_bits(0.4), "light": 0xF000F0 - 17,
                          "x": float_bits(3.0), "y": float_bits(-1.25), "z": float_bits(7.5)})
    for repeat in range(3):
        poses.append({"label": f"repeat {repeat}", "type": "single", "facing": "south",
                      "material": "REGULAR", "open": float_bits(0.4), "light": 240,
                      "x": float_bits(1.0), "y": float_bits(1.0), "z": float_bits(1.0)})
    for material in ("ENDER_CHEST", "CHRISTMAS", "TRAPPED", "COPPER_UNAFFECTED", "COPPER_EXPOSED",
                     "COPPER_WEATHERED", "COPPER_OXIDIZED"):
        for kind in ("single", "left", "right"):
            poses.append({"label": f"{material} {kind}", "type": kind, "facing": "south",
                          "material": material, "open": float_bits(0.0), "light": 15728880,
                          "x": float_bits(0.0), "y": float_bits(0.0), "z": float_bits(0.0)})
    for index, open_ in enumerate((0.0, 0.1, 0.5, 0.9, 1.0)):
        poses.append({"label": f"open {index}", "type": "single", "facing": "north", "material": "TRAPPED",
                      "open": float_bits(open_), "light": 240, "x": float_bits(-4.5),
                      "y": float_bits(2.0), "z": float_bits(-0.5)})
    yield {"case_id": "entity/chest-pose", "family": "entity", "operation": "chest_pose", "rows": poses}


def entity_shulker_box_cases():
    """Where the real `ShulkerBoxRenderer` draws a box of each colour, facing and lid progress, and
    the model with the lid set for it."""
    colors = ["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
              "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black"]
    rows = []
    for facing in ("down", "up", "north", "south", "west", "east"):
        for progress in (0.0, 0.3, 1.0):
            rows.append({"label": f"{facing} {progress}", "color": None, "facing": facing,
                         "progress": float_bits(progress), "light": 0xF000F0 - 17,
                         "x": float_bits(3.0), "y": float_bits(-1.25), "z": float_bits(7.5)})
    for color in colors:
        rows.append({"label": color, "color": color, "facing": "up", "progress": float_bits(0.5),
                     "light": 15728880, "x": float_bits(0.0), "y": float_bits(0.0),
                     "z": float_bits(0.0)})
    yield {"case_id": "entity/shulker-box-pose", "family": "entity", "operation": "shulker_box_pose",
           "rows": rows}


def entity_bell_cases():
    """The real `BellRenderer` model for a bell at rest and swinging from each side, through the
    fifty ticks of a ring."""
    rows = []
    for shake in (None, "north", "south", "east", "west", "up"):
        for ticks in (0.0, 0.5, 3.25, 10.0, 24.75, 49.9):
            rows.append({"label": f"{shake} {ticks}", "ticks": float_bits(ticks), "shake": shake,
                         "light": 0xF000F0 - 17})
    yield {"case_id": "entity/bell-pose", "family": "entity", "operation": "bell_pose", "rows": rows}


def entity_skull_block_cases():
    """The real `SkullBlockRenderer` for each skull standing at each of the sixteen turns and
    hanging on each wall, with the jaw and ears at several animation times."""
    rows = []
    kinds = ("SKELETON", "WITHER_SKELETON", "PLAYER", "ZOMBIE", "CREEPER", "PIGLIN", "DRAGON")
    for kind in kinds:
        for rotation in range(16):
            rows.append({"label": f"{kind} floor {rotation}", "kind": kind, "wall": None,
                         "rotation": rotation, "animation": float_bits(0.0),
                         "light": 0xF000F0 - 17})
        for wall in ("north", "south", "east", "west"):
            rows.append({"label": f"{kind} wall {wall}", "kind": kind, "wall": wall, "rotation": 0,
                         "animation": float_bits(0.0), "light": 240})
    for kind in ("PIGLIN", "DRAGON"):
        for animation in (0.5, 7.25, 33.0, 120.5):
            rows.append({"label": f"{kind} animated {animation}", "kind": kind, "wall": None,
                         "rotation": 4, "animation": float_bits(animation), "light": 15728880})
    yield {"case_id": "entity/skull-block-pose", "family": "entity", "operation": "skull_block_pose",
           "rows": rows}


def entity_elder_guardian_particle_cases():
    """The pose and colour of the elder guardian particle over the whole of its life, the camera
    looking every way."""
    rows = []
    for age in (0, 1, 7, 15, 23, 29, 30):
        for yaw, pitch in ((0.0, 0.0), (90.0, 0.0), (-135.5, 30.25), (41.0, -77.5)):
            rows.append({"label": f"age {age} yaw {yaw} pitch {pitch}", "age": age, "lifetime": 30,
                         "yaw": float_bits(yaw), "pitch": float_bits(pitch),
                         "partial": float_bits(0.4)})
    for partial in (0.0, 0.25, 0.999):
        rows.append({"label": f"partial {partial}", "age": 10, "lifetime": 30,
                     "yaw": float_bits(12.0), "pitch": float_bits(-3.0), "partial": float_bits(partial)})
    yield {"case_id": "entity/elder-guardian-particle-pose", "family": "entity",
           "operation": "elder_guardian_particle_pose", "rows": rows}


def entity_banner_cases():
    """Every model the real `BannerRenderer` submits for banners standing at each turn and hanging on
    each wall, of several colours, with no patterns, a few and the most (sixteen), the flag at
    several phases."""
    patterns = ["stripe_bottom", "stripe_top", "stripe_left", "stripe_right", "stripe_center",
                "stripe_middle", "stripe_downright", "stripe_downleft", "small_stripes", "cross",
                "straight_cross", "diagonal_left", "diagonal_right", "diagonal_up_left",
                "diagonal_up_right", "half_vertical", "half_vertical_right", "half_horizontal",
                "half_horizontal_bottom", "square_bottom_left", "triangle_bottom", "circle",
                "rhombus", "border", "curly_border", "gradient", "gradient_up", "bricks", "creeper",
                "skull", "flower", "mojang", "globe", "piglin", "flow", "guster"]
    dyes = ["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
            "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black"]

    def layers(count, offset):
        return [{"pattern": "minecraft:" + patterns[(offset + index) % len(patterns)],
                 "color": dyes[(offset * 3 + index * 5) % 16]} for index in range(count)]

    rows = []
    for rotation in range(16):
        rows.append({"label": f"floor {rotation}", "wall": None, "rotation": rotation,
                     "color": dyes[rotation], "patterns": layers(rotation % 4, rotation),
                     "phase": float_bits(rotation / 16.0), "light": 0xF000F0 - 17})
    for index, wall in enumerate(("north", "south", "east", "west")):
        rows.append({"label": f"wall {wall}", "wall": wall, "rotation": 0, "color": dyes[index * 4],
                     "patterns": layers(3, index), "phase": float_bits(0.37 * index), "light": 240})
    for offset in range(0, len(patterns), 6):
        rows.append({"label": f"every pattern from {offset}", "wall": None, "rotation": 3,
                     "color": "white", "patterns": layers(6, offset),
                     "phase": float_bits(0.5), "light": 15728880})
    rows.append({"label": "the most", "wall": None, "rotation": 9, "color": "red",
                 "patterns": layers(16, 5), "phase": float_bits(0.99), "light": 15728880})
    rows.append({"label": "past the most", "wall": "south", "rotation": 0, "color": "blue",
                 "patterns": layers(20, 7), "phase": float_bits(0.01), "light": 15728880})
    yield {"case_id": "entity/banner-pose", "family": "entity", "operation": "banner_pose",
           "rows": rows}


def entity_text_display_cases():
    """What the reference's own text display renderer submits for the lines a text split into (each
    of a given width): the background box (none when clear, the default one by the option's opacity,
    and one seen through blocks), each line's place by the alignment, the shadow, the opacity in the
    colour, and the light."""
    def text(label, widths, flags=0, opacity=-1, background=0x40000000, light=15728880, feet=(10.5, 64.25, -3.5)):
        return {"label": f"text-{label}", "widths": widths, "flags": flags, "opacity": opacity,
                "background": background, "light": light, "feet_x": float_bits(feet[0]),
                "feet_y": float_bits(feet[1]), "feet_z": float_bits(feet[2])}
    rows = [text("one-line", [37]), text("three-lines-centred", [40, 95, 17]),
            text("left", [40, 95, 17], flags=8), text("right", [40, 95, 17], flags=16),
            text("both-align-flags", [40, 95, 17], flags=24), text("shadow-and-see-through", [20, 30], flags=3),
            text("default-background", [20, 30], flags=4, background=0x12345678),
            text("clear-background", [20, 30], background=0), text("half-opacity", [20, 30], opacity=-128),
            text("dim-opacity", [20, 30], opacity=40, background=0x80102030 - (1 << 32), light=240),
            text("no-lines", []), text("far", [64], feet=(-100.25, 70.5, 33.75))]
    yield {"case_id": "entity/text-display-submit", "family": "entity", "operation": "text_display_submit",
           "rows": rows}


def entity_map_render_cases():
    """What the reference's own map renderer draws: the map's quad and each decoration (all of
    them, or only those a frame shows), turned by sixteenths, one in front of the next."""
    def decoration(x, y, rot, on_frame, sprite=(0.25, 0.5, 0.375, 0.625)):
        return {"x": x, "y": y, "rot": rot, "on_frame": on_frame,
                "sprite": {"u0": float_bits(sprite[0]), "v0": float_bits(sprite[1]),
                           "u1": float_bits(sprite[2]), "v1": float_bits(sprite[3])}}

    def map_(label, decorations, only_frame=False, light=15728850, feet=(10.5, 64.25, -3.5)):
        return {"label": f"map-{label}", "decorations": decorations, "only_frame": only_frame, "light": light,
                "feet_x": float_bits(feet[0]), "feet_y": float_bits(feet[1]), "feet_z": float_bits(feet[2])}
    many = [decoration(0, 0, 0, True), decoration(-128, 127, 4, False, (0.5, 0.0, 0.625, 0.125)),
            decoration(63, -64, 15, True, (0.0, 0.25, 0.125, 0.375)), decoration(-20, 33, 8, False)]
    rows = [map_("bare", []), map_("decorated", many), map_("only-a-frames", many, only_frame=True),
            map_("none-shown", [decoration(1, 2, 3, False)], only_frame=True),
            map_("turned-far", many, light=0x00F000F0, feet=(-100.25, 70.5, 33.75))]
    yield {"case_id": "entity/map-render", "family": "entity", "operation": "map_render", "rows": rows}


def entity_arrow_cases():
    """The arrow's model posed by the reference's own `ArrowModel.setupAnim` (still, and shaking
    at ticks through the seven it lasts, between two ticks) and where the reference's own arrow
    renderer draws it from the turns it was sent (along each axis, slantwise, pitched up and down)."""
    model_rows = [{"label": f"arrow-shake-{shake}", "layer": "minecraft:arrow#main",
                   "shake": float_bits(shake)} for shake in (0.0, 0.5, 1.0, 2.5, 4.0, 6.5, 7.0, -0.25)]
    yield {"case_id": "entity/arrow-model", "family": "entity", "operation": "airborne_model",
           "rows": model_rows}

    def pose(label, y_rot, x_rot, shake=0.0, at=(10.5, 64.0, -3.5)):
        return {"label": f"arrow-{label}", "feet_x": float_bits(at[0]), "feet_y": float_bits(at[1]),
                "feet_z": float_bits(at[2]), "y_rot": float_bits(y_rot), "x_rot": float_bits(x_rot),
                "shake": float_bits(shake)}
    pose_rows = [pose("still", 0.0, 0.0), pose("south-east", 45.0, 0.0), pose("west", -90.0, 0.0),
                 pose("up", 30.0, 60.0), pose("down", 150.0, -75.0), pose("wrapped", 400.0, 10.0),
                 pose("offset", 12.5, -33.3, at=(-100.25, 70.5, 33.75))]
    yield {"case_id": "entity/arrow-pose", "family": "entity", "operation": "arrow_pose", "rows": pose_rows}
    # The trident: its model (nothing moves it) and where its renderer draws it from the turns it was sent.
    yield {"case_id": "entity/trident-model", "family": "entity", "operation": "airborne_model",
           "rows": [{"label": "trident", "layer": "minecraft:trident#main"}]}
    trident_rows = []
    for row in pose_rows:
        copy = dict(row)
        copy["label"] = row["label"].replace("arrow-", "trident-", 1)
        copy["trident"] = True
        trident_rows.append(copy)
    yield {"case_id": "entity/trident-pose", "family": "entity", "operation": "arrow_pose", "rows": trident_rows}


def entity_thrown_item_cases():
    """Where the reference's own thrown item renderer (and its firework rocket's) puts an item: scaled
    by the type's (the fireballs' three, the small one's three quarters), turned to face a camera that
    looks along each way and slantwise, and a rocket shot at an angle laid on its side."""
    def seen(label, camera=(0.0, 0.0), scale=1.0, firework=False, angled=False, at=(10.5, 64.0, -3.5)):
        return {"label": f"thrown-{label}", "feet_x": float_bits(at[0]), "feet_y": float_bits(at[1]),
                "feet_z": float_bits(at[2]), "camera_y_rot": float_bits(camera[0]),
                "camera_x_rot": float_bits(camera[1]), "scale": float_bits(scale), "firework": firework,
                "shot_at_angle": angled}
    rows = [seen("south", (0.0, 0.0)), seen("west", (90.0, 0.0)), seen("north", (180.0, 0.0)),
            seen("down-on-it", (0.0, 90.0)), seen("up-at-it", (45.0, -60.0)), seen("slantwise", (-130.0, 33.5)),
            seen("fireball", (20.0, 10.0), scale=3.0), seen("small-fireball", (20.0, 10.0), scale=0.75),
            seen("far-off", (30.0, 5.0), at=(-100.25, 70.5, 33.75)),
            seen("rocket", (20.0, 10.0), firework=True), seen("rocket-at-an-angle", (20.0, 10.0), firework=True, angled=True),
            seen("rocket-at-an-angle-looked-down-on", (-70.0, 80.0), firework=True, angled=True)]
    yield {"case_id": "entity/thrown-item-pose", "family": "entity", "operation": "thrown_item_pose", "rows": rows}


def entity_charge_cases():
    """The wind charge's model posed by the reference's own `WindChargeModel.setupAnim` (the charge and
    the wind spinning opposite ways with the age) and the quad the reference's own dragon fireball
    renderer submits, seen by a camera that looks along each way and slantwise."""
    yield {"case_id": "entity/wind-charge-model", "family": "entity", "operation": "airborne_model",
           "rows": [{"label": f"wind-charge-age-{age}", "layer": "minecraft:wind_charge#main",
                     "age_in_ticks": float_bits(age)} for age in (0.0, 0.5, 1.0, 3.25, 11.75, 22.5, 100.1)]}

    def seen(label, camera=(0.0, 0.0), at=(10.5, 64.0, -3.5), light=0x00F000F0):
        return {"label": f"fireball-{label}", "feet_x": float_bits(at[0]), "feet_y": float_bits(at[1]),
                "feet_z": float_bits(at[2]), "camera_y_rot": float_bits(camera[0]),
                "camera_x_rot": float_bits(camera[1]), "light": light}
    rows = [seen("south"), seen("west", (90.0, 0.0)), seen("north", (180.0, 0.0)),
            seen("down-on-it", (0.0, 90.0)), seen("up-at-it", (45.0, -60.0)), seen("slantwise", (-130.0, 33.5)),
            seen("far-off", (30.0, 5.0), at=(-100.25, 70.5, 33.75), light=0x00F00000)]
    yield {"case_id": "entity/dragon-fireball-quad", "family": "entity", "operation": "dragon_fireball_quad",
           "rows": rows}


def entity_small_projectile_cases():
    """The shulker bullet's, llama spit's and evoker fangs' models posed by the reference's own model
    code (the bullet turned the way it flies, the fangs through their bite and its end) and where
    their own renderers draw them from (the bullet tumbling with its age, in its core and its shell;
    fangs not at all before they bite)."""
    yield {"case_id": "entity/shulker-bullet-model", "family": "entity", "operation": "airborne_model",
           "rows": [{"label": f"bullet-{label}", "layer": "minecraft:shulker_bullet#main",
                     "y_rot": float_bits(y), "x_rot": float_bits(x)}
                    for label, y, x in (("still", 0.0, 0.0), ("turned", 45.0, 30.0), ("wrapped", 400.0, -75.5),
                                        ("down", -120.0, 90.0))]}
    yield {"case_id": "entity/llama-spit-model", "family": "entity", "operation": "airborne_model",
           "rows": [{"label": "spit", "layer": "minecraft:llama_spit#main"}]}
    progresses = (0.0, 0.05, 0.25, 0.5, 0.6, 0.85, 0.9, 0.93, 0.99, 1.0)
    yield {"case_id": "entity/evoker-fangs-model", "family": "entity", "operation": "airborne_model",
           "rows": [{"label": f"fangs-{progress}", "layer": "minecraft:evoker_fangs#main",
                     "bite_progress": float_bits(progress)} for progress in progresses]}

    def pose(kind, label, at=(10.5, 64.0, -3.5), **values):
        row = {"label": f"{kind}-{label}", "kind": kind, "feet_x": float_bits(at[0]), "feet_y": float_bits(at[1]),
               "feet_z": float_bits(at[2])}
        row.update({key: float_bits(value) for key, value in values.items()})
        return row
    rows = [pose("shulker_bullet", f"age-{age}", age_in_ticks=age) for age in (0.0, 1.5, 7.25, 31.0, 99.75)]
    rows.append(pose("shulker_bullet", "far-off", at=(-100.25, 70.5, 33.75), age_in_ticks=12.5))
    rows += [pose("llama_spit", label, y_rot=y, x_rot=x)
             for label, y, x in (("south", 0.0, 0.0), ("west", -90.0, 0.0), ("up", 30.0, 60.0), ("wrapped", 400.0, 10.0))]
    rows += [pose("evoker_fangs", f"{label}", y_rot=y, bite_progress=progress)
             for label, y, progress in (("waiting", 40.0, 0.0), ("biting", 40.0, 0.4), ("late", -130.0, 0.95),
                                        ("done", 0.0, 1.0), ("turned", 400.0, 0.3))]
    rows.append(pose("leash_knot", "fence"))
    rows.append(pose("leash_knot", "far-off", at=(-100.25, 70.5, 33.75)))
    yield {"case_id": "entity/leash-knot-model", "family": "entity", "operation": "airborne_model",
           "rows": [{"label": "knot", "layer": "minecraft:leash_knot#main"}]}
    yield {"case_id": "entity/small-projectile-pose", "family": "entity", "operation": "small_projectile_pose",
           "rows": rows}


def entity_leash_cases():
    """A leash's strip as the reference's own `LeashFeatureRenderer` builds it: taut and slack, a hand
    above and below the entity, along each axis and slantwise, near and far, with the light the same at
    both ends and different, and the line tied at an offset."""
    def strip(label, start, end, offset=(0.0, 1.0, 0.4), slack=True, lights=(15, 15, 15, 15),
              at=(10.5, 64.0, -3.5)):
        return {"label": f"leash-{label}", "feet_x": float_bits(at[0]), "feet_y": float_bits(at[1]),
                "feet_z": float_bits(at[2]),
                "offset": [float_bits(value, "f64") for value in offset],
                "start": [float_bits(value, "f64") for value in start],
                "end": [float_bits(value, "f64") for value in end],
                "start_block": lights[0], "end_block": lights[1], "start_sky": lights[2],
                "end_sky": lights[3], "slack": slack}
    rows = [
        strip("up-slack", (10.5, 65.0, -3.1), (12.5, 66.2, -1.0)),
        strip("down-slack", (10.5, 65.0, -3.1), (12.5, 63.5, -1.0)),
        strip("up-taut", (10.5, 65.0, -3.1), (12.5, 66.2, -1.0), slack=False),
        strip("down-taut", (10.5, 65.0, -3.1), (12.5, 63.5, -1.0), slack=False),
        strip("level-slack", (10.5, 65.0, -3.1), (13.5, 65.0, -3.1)),
        strip("east", (0.0, 1.0, 0.0), (4.0, 1.5, 0.0)),
        strip("west", (0.0, 1.0, 0.0), (-4.0, 0.5, 0.0)),
        strip("south", (0.0, 1.0, 0.0), (0.0, 1.5, 4.0)),
        strip("north", (0.0, 1.0, 0.0), (0.0, 0.5, -4.0)),
        strip("slantwise", (0.0, 1.0, 0.0), (-3.25, 2.75, 5.5)),
        strip("straight-up", (0.0, 1.0, 0.0), (0.0, 4.0, 0.0)),
        strip("short", (0.0, 1.0, 0.0), (0.25, 1.1, 0.25)),
        strip("far", (0.0, 1.0, 0.0), (20.0, 3.0, -12.0), at=(-100.25, 70.5, 33.75)),
        strip("lit-differently", (0.0, 1.0, 0.0), (4.0, 2.0, 1.0), lights=(0, 15, 15, 0)),
        strip("dark-to-bright", (0.0, 1.0, 0.0), (4.0, 2.0, 1.0), lights=(3, 12, 1, 14)),
        strip("no-offset", (0.0, 1.0, 0.0), (4.0, 2.0, 1.0), offset=(0.0, 0.0, 0.0)),
        strip("odd-offset", (0.3, 1.7, -0.2), (4.0, 2.0, 1.0), offset=(0.3, 1.7, -0.2)),
        strip("on-the-spot", (0.0, 1.0, 0.0), (0.0, 1.0, 0.0)),
    ]
    yield {"case_id": "entity/leash-strip", "family": "entity", "operation": "leash_strip", "rows": rows}


def entity_fishing_hook_cases():
    """A fishing bobber as the reference's own renderer submits it: the quad facing a camera that looks
    along each way and slantwise, and the line's sixteen segments (from its own `stringVertex`) toward
    a hand in each direction, near and far."""
    def seen(label, camera=(0.0, 0.0), offset=(0.0, 1.0, 2.0), at=(10.5, 64.0, -3.5), light=0x00F000F0):
        return {"label": f"hook-{label}", "feet_x": float_bits(at[0]), "feet_y": float_bits(at[1]),
                "feet_z": float_bits(at[2]), "camera_y_rot": float_bits(camera[0]),
                "camera_x_rot": float_bits(camera[1]), "offset_x": float_bits(offset[0]),
                "offset_y": float_bits(offset[1]), "offset_z": float_bits(offset[2]), "light": light}
    rows = [seen("south"), seen("west", (90.0, 0.0), offset=(-3.5, 0.5, 0.25)),
            seen("north", (180.0, 0.0), offset=(2.0, -0.3, -4.75)),
            seen("down-on-it", (0.0, 90.0), offset=(0.1, 3.0, 0.1)),
            seen("up-at-it", (45.0, -60.0), offset=(-1.25, 2.5, 7.0)),
            seen("slantwise", (-130.0, 33.5), offset=(5.5, -2.25, -6.125)),
            seen("far-off", (30.0, 5.0), offset=(0.0, 0.0, 0.5), at=(-100.25, 70.5, 33.75), light=0x00F00000),
            seen("hand-on-the-bobber", offset=(0.0, 0.0, 0.0))]
    yield {"case_id": "entity/fishing-hook", "family": "entity", "operation": "fishing_hook", "rows": rows}


def entity_primed_tnt_cases():
    """Where the reference's own tnt renderer draws the block of a primed tnt from the fuse it has left (long, at the
    swelling's start and end, either side of each flash, spent), and the overlay it flashes with."""
    def at(label, fuse, place=(10.5, 64.0, -3.5)):
        return {"label": f"tnt-{label}", "feet_x": float_bits(place[0]), "feet_y": float_bits(place[1]),
                "feet_z": float_bits(place[2]), "fuse": float_bits(fuse)}
    rows = [at(f"fuse-{fuse}", fuse) for fuse in (81.0, 80.5, 40.0, 11.0, 10.0, 9.5, 9.0, 7.25, 5.0, 4.99, 3.0, 1.5, 1.0, 0.5, 0.0, -0.5)]
    rows.append(at("far-off", 6.0, place=(-100.25, 70.5, 33.75)))
    yield {"case_id": "entity/primed-tnt-pose", "family": "entity", "operation": "primed_tnt_pose", "rows": rows}


def entity_lightning_bolt_cases():
    """The vertices the reference's own lightning renderer hands the vertex consumer for several seeds (zero,
    negative, large, a few ordinary ones) and places."""
    def bolt(label, seed, at=(10.5, 64.0, -3.5)):
        return {"label": f"bolt-{label}", "seed": str(seed), "feet_x": float_bits(at[0]), "feet_y": float_bits(at[1]),
                "feet_z": float_bits(at[2])}
    rows = [bolt("zero", 0), bolt("one", 1), bolt("ordinary", 123456789), bolt("negative", -987654321),
            bolt("large", 9223372036854775807), bolt("small", -9223372036854775808), bolt("far-off", 42, at=(-100.25, 70.5, 33.75))]
    yield {"case_id": "entity/lightning-bolt", "family": "entity", "operation": "lightning_bolt", "rows": rows}


def entity_painting_geometry_cases():
    """The vertices the reference's own painting renderer hands the vertex consumer: paintings of several sizes
    (one block, a wide one, a tall one, a big one) facing each way, each block lit differently."""
    def painting(label, size, direction, at=(10.5, 64.5, -3.5)):
        width, height = size
        sprites = {"front_u0": 0.125, "front_u1": 0.875, "front_v0": 0.0625, "front_v1": 0.5625,
                   "back_u0": 0.5, "back_u1": 0.75, "back_v0": 0.25, "back_v1": 0.375}
        row = {"label": f"painting-{label}", "width": width, "height": height, "direction_2d": direction,
               "feet_x": float_bits(at[0]), "feet_y": float_bits(at[1]), "feet_z": float_bits(at[2]),
               "lights": [(0x00F00000 | (index * 16 + 3) << 4) & 0x00F000F0 for index in range(width * height)]}
        row.update({key: float_bits(value) for key, value in sprites.items()})
        return row
    rows = [painting("one-south", (1, 1), 0), painting("one-west", (1, 1), 1), painting("one-north", (1, 1), 2),
            painting("one-east", (1, 1), 3), painting("wide", (3, 1), 0), painting("tall", (1, 2), 1),
            painting("two-by-two", (2, 2), 2), painting("big", (4, 3), 3), painting("backyard", (3, 4), 0),
            painting("far-off", (2, 1), 1, at=(-100.25, 70.5, 33.75))]
    yield {"case_id": "entity/painting-geometry", "family": "entity", "operation": "painting_geometry", "rows": rows}


def entity_screen_effect_cases():
    """The quads the screen effects lay over the view, as the reference's own `ScreenEffectRenderer` builds them:
    the block the eyes are in and the fire from stand-in sprite corners, and the water from where the player
    looks and how bright it is at the eyes."""
    def row(label, kind, **fields):
        return {"label": label, "kind": kind, **fields}
    sprite = dict(u0=float_bits(0.25), u1=float_bits(0.3125), v0=float_bits(0.5), v1=float_bits(0.5625))
    other = dict(u0=float_bits(0.0), u1=float_bits(1.0), v0=float_bits(0.0), v1=float_bits(1.0))
    rows = [row("block", "block", color=-15132391, **sprite), row("block-whole-texture", "block", color=-15132391, **other),
            row("fire", "fire", **sprite), row("fire-whole-texture", "fire", **other)]
    for label, y_rot, x_rot, brightness in (("ahead", 0.0, 0.0, 1.0), ("turned", 123.5, -30.25, 0.75),
                                            ("looking-down", -200.0, 90.0, 0.2), ("dark", 10.0, 5.0, 0.0)):
        rows.append(row(f"water-{label}", "water", y_rot=float_bits(y_rot), x_rot=float_bits(x_rot),
                        brightness=float_bits(brightness)))
    yield {"case_id": "entity/screen-effect-quads", "family": "entity", "operation": "screen_effect_quads",
           "rows": rows}
