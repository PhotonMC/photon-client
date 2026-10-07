"""Helpers of the movement inputs."""

from cases.helpers import *


def floor(y=63, block="minecraft:stone", low=-16, high=31):
    return [{"x": x, "y": y, "z": z, "block": block}
            for x in range(low, high + 1) for z in range(low, high + 1)]


def column(x, z, low, high, block="minecraft:stone", properties=None):
    out = []
    for y in range(low, high + 1):
        placed = {"x": x, "y": y, "z": z, "block": block}
        if properties:
            placed["properties"] = properties
        out.append(placed)
    return out


def keys(count, **held):
    tick = {name: held.get(name, False)
            for name in ("forward", "backward", "left", "right", "jump", "sneak", "sprint")}
    return [dict(tick) for _ in range(count)]


def turned(ticks, y_rot=None, x_rot=None):
    for tick in ticks:
        if y_rot is not None:
            tick["y_rot"] = float_bits(y_rot)
        if x_rot is not None:
            tick["x_rot"] = float_bits(x_rot)
    return ticks


def scene(label, blocks, ticks, start=(0.5, 64.0, 0.5), y_rot=0.0, x_rot=0.0,
          game_mode="survival", auto_jump=False, border=None):
    row = {
        "label": label,
        "blocks": blocks,
        "chunk_radius": 1,
        "entity_id": 7,
        "x": float_bits(start[0], "f64"),
        "y": float_bits(start[1], "f64"),
        "z": float_bits(start[2], "f64"),
        "y_rot": float_bits(y_rot),
        "x_rot": float_bits(x_rot),
        "game_mode": game_mode,
        "auto_jump": auto_jump,
        "ticks": ticks,
    }
    if border is not None:
        row["border"] = {"x": float_bits(border[0], "f64"), "z": float_bits(border[1], "f64"),
                         "size": float_bits(border[2], "f64")}
    return row


# Riding a boat: the player sits in it as the server's passengers packet puts it there, and the
# level ticks the boat (its own `tick`, then the rider through `rideTick`). Each tick reports the
# player and the boat, and the packets sent: the input, the rotation, the vehicle's position and
# the paddles.
def ride(label, blocks, ticks, boat=(0.5, 64.0, 0.5), y_rot=0.0, motion=None, kind="minecraft:oak_boat"):
    row = scene(label, blocks, ticks, start=boat)
    row["ride"] = {"type": kind, "id": 8, "x": float_bits(boat[0], "f64"),
                   "y": float_bits(boat[1], "f64"), "z": float_bits(boat[2], "f64"),
                   "y_rot": float_bits(y_rot)}
    if motion is not None:
        row["ride"].update({"motion_x": float_bits(motion[0], "f64"),
                            "motion_y": float_bits(motion[1], "f64"),
                            "motion_z": float_bits(motion[2], "f64")})
    return row


# Riding a horse: a tamed horse in a saddle, steered by the player's keys and look, jumping
# when the jump key is let go of (`AbstractHorse.tickRidden`, `LocalPlayer.jumpableVehicle`).
def horse(label, blocks, ticks, start=(0.5, 64.0, 0.5), y_rot=0.0, kind="minecraft:horse"):
    return ride(label, blocks, ticks, boat=start, y_rot=y_rot, kind=kind)


def looking(ticks, *angles):
    """Each tick's look as `(y_rot, x_rot)`, the last held on."""
    for index, tick in enumerate(ticks):
        yaw, pitch = angles[min(index * len(angles) // max(len(ticks), 1), len(angles) - 1)]
        tick["y_rot"] = float_bits(yaw)
        tick["x_rot"] = float_bits(pitch)
    return ticks


# Riding a camel (a horse that dashes and sits): a jump charge is a dash, a sitting camel rises when
# pushed forward, and one that sits or moves between poses stays put.
def camel(label, blocks, ticks, start=(0.5, 64.0, 0.5), y_rot=0.0, sit_age=None, kind="minecraft:camel"):
    row = ride(label, blocks, ticks, boat=start, y_rot=y_rot, kind=kind)
    if sit_age is not None:
        row["ride"]["sit_age"] = sit_age
    return row


# Riding a pig: saddled and steered with a carrot on a stick it goes straight ahead where the
# rider looks (`Pig.getRiddenInput`); otherwise the rider only sits in it.
def pig(label, blocks, ticks, start=(0.5, 64.0, 0.5), y_rot=0.0, saddled=True, stick="minecraft:carrot_on_a_stick"):
    row = ride(label, blocks, ticks, boat=start, y_rot=y_rot, kind="minecraft:pig")
    row["ride"]["saddled"] = saddled
    if stick:
        row["ride"]["holding"] = stick
    return row


# Riding a strider: steered with a warped fungus on a stick, it goes straight ahead where the
# rider looks, warm (fast) in or on warm blocks and in lava, cold (slow) elsewhere, and stands on
# the surface of lava (`Strider.canStandOnFluid`, `floatStrider`).
def strider(label, blocks, ticks, start=(0.5, 64.0, 0.5), y_rot=0.0, saddled=True,
            stick="minecraft:warped_fungus_on_a_stick"):
    row = ride(label, blocks, ticks, boat=start, y_rot=y_rot, kind="minecraft:strider")
    row["ride"]["saddled"] = saddled
    if stick:
        row["ride"]["holding"] = stick
    return row


def lava_pool(depth=1, low=-5, high=5):
    return [{"x": x, "y": 64 + level, "z": z, "block": "minecraft:lava"}
            for x in range(low, high + 1) for z in range(low, high + 1) for level in range(depth)]


def breaking(label, blocks, ticks, main_hand=None, effects=None, **kwargs):
    row = scene(label, floor() + blocks, ticks, **kwargs)
    row["breaking"] = {}
    if main_hand:
        row["breaking"]["main_hand"] = main_hand
    if effects:
        row["breaking"]["effects"] = effects
    return row


def at(place, block, properties=None):
    placed = {"x": place["x"], "y": place["y"], "z": place["z"], "block": block}
    if properties:
        placed["properties"] = properties
    return placed


# Light: the client level lights itself from nothing, then relights as
# block updates arrive. Each row reports a box's block and sky light
# after the start and after every update.
def slab(low_x, high_x, low_z, high_z, low_y, high_y, block="minecraft:stone"):
    return [{"x": x, "y": y, "z": z, "block": block}
            for x in range(low_x, high_x + 1) for z in range(low_z, high_z + 1)
            for y in range(low_y, high_y + 1)]


def lit(label, blocks, events, box):
    return {
        "label": label,
        "blocks": blocks,
        "chunk_radius": 1,
        "events": events,
        "box": {"x": box[0], "y": box[1], "z": box[2],
                "size_x": box[3], "size_y": box[4], "size_z": box[5]},
    }


def put(x, y, z, block, properties=None):
    placed = {"x": x, "y": y, "z": z, "block": block}
    if properties:
        placed["properties"] = properties
    return placed


# Using: the use key as `handleKeybinds` spends it, into the reference's
# own game mode and block items. The aim is named, with where on the
# face it lands.
def aim_at(x, y, z, face, at):
    return {"x": x, "y": y, "z": z, "face": face,
            "location": {"x": float_bits(at[0], "f64"), "y": float_bits(at[1], "f64"),
                         "z": float_bits(at[2], "f64")}}


def uses(count, aim, clicks=0, down=False, watch_at=(), **held):
    ticks = keys(count, **held)
    for index, tick in enumerate(ticks):
        attack = {"clicks": 0, "down": False,
                  "use": {"clicks": clicks if index == 0 else 0, "down": down},
                  "watch": [{"x": w[0], "y": w[1], "z": w[2]} for w in watch_at]}
        if aim is not None:
            attack["target"] = aim
        tick["attack"] = attack
    return ticks


def using(label, blocks, ticks, main_hand=None, count=1, off_hand=None, **kwargs):
    row = scene(label, floor() + blocks, ticks, **kwargs)
    row["breaking"] = {}
    if main_hand:
        row["breaking"]["main_hand"] = main_hand
        row["breaking"]["main_hand_count"] = count
    if off_hand:
        row["breaking"]["off_hand"] = off_hand
    return row


# Entities in the crosshair: the player picks as `LocalPlayer.raycastHitResult`
# does, among the blocks and the entities the server told of, and the
# attack and use keys act on what it lands on. Everything the player sent
# is compared as the bytes the packets' own codecs write.
def aims(count, clicks=0, use_clicks=0, down=False, use_down=False, pick_clicks=0, **held):
    ticks = keys(count, **held)
    for index, tick in enumerate(ticks):
        tick["attack"] = {"clicks": clicks if index == 0 else 0, "down": down, "pick": True,
                          "use": {"clicks": use_clicks if index == 0 else 0, "down": use_down},
                          "watch": []}
        if pick_clicks:
            tick["attack"]["pick_item"] = {"clicks": pick_clicks if index == 0 else 0}
    return ticks


def mob(entity_id, kind, x, y, z):
    return {"id": entity_id, "type": f"minecraft:{kind}", "x": float_bits(x, "f64"),
            "y": float_bits(y, "f64"), "z": float_bits(z, "f64")}


def facing(label, entities, ticks, blocks=(), main_hand=None, off_hand=None, effects=None, **kwargs):
    row = scene(label, floor() + list(blocks), ticks, **kwargs)
    row["breaking"] = {}
    if main_hand:
        row["breaking"]["main_hand"] = main_hand
    if off_hand:
        row["breaking"]["off_hand"] = off_hand
    if effects:
        row["breaking"]["effects"] = effects
    row["entities"] = entities
    return row


# Container clicks: the inventory screen's clicks into the reference's
# own inventory menu and game mode, which predicts each click and sends
# it. Stacks are named by item and count.
def st(slot, item, count):
    return {"slot": slot, "item": f"minecraft:{item}", "count": count}


def cl(slot, button, kind):
    return {"slot": slot, "button": button, "type": kind}


def clicks(label, slots, click_list, carried=None, game_mode="survival", selected=0, state_id=0,
           menu=None, furnace_inputs=None, extra=None):
    row = {"label": label, "blocks": [], "chunk_radius": 0, "game_mode": game_mode,
           "selected": selected, "state_id": state_id, "slots": slots, "clicks": click_list}
    if menu is not None:
        row["menu"] = f"minecraft:{menu}"
        row["container_id"] = 4
    if extra is not None:
        row.update(extra)
    if furnace_inputs is not None:
        row["furnace_inputs"] = [f"minecraft:{name}" for name in furnace_inputs]
    if carried is not None:
        row["carried"] = {"item": f"minecraft:{carried[0]}", "count": carried[1]}
    return row


def offer(buy, sell, buy_b=None, uses=0, max_uses=12, xp=2, special=0, demand=0, multiplier=0.05):
    made = {"buy": [f"minecraft:{buy[0]}", buy[1]], "sell": [f"minecraft:{sell[0]}", sell[1]],
            "uses": uses, "max_uses": max_uses, "xp": xp, "special": special, "demand": demand,
            "multiplier": multiplier}
    if buy_b:
        made["buy_b"] = [f"minecraft:{buy_b[0]}", buy_b[1]]
    return made
