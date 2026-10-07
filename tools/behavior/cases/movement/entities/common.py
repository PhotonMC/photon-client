"""Helpers of the entity movement inputs."""

from cases.helpers import *


def floor(block="minecraft:stone", y=63, low=-8, high=8, x_low=None):
    start = low if x_low is None else x_low
    return [{"x": x, "y": y, "z": z, "block": block}
            for x in range(start, high + 1) for z in range(low, high + 1)]


def row(label, blocks, position, motion, ticks=60, on_ground=False, entity_id=21,
        events=()):
    return {
        "events": list(events),
        "label": label,
        "blocks": blocks,
        "chunk_radius": 1,
        "entity_id": entity_id,
        "x": float_bits(position[0], "f64"),
        "y": float_bits(position[1], "f64"),
        "z": float_bits(position[2], "f64"),
        "motion_x": float_bits(motion[0], "f64"),
        "motion_y": float_bits(motion[1], "f64"),
        "motion_z": float_bits(motion[2], "f64"),
        "on_ground": on_ground,
        "tick_count": ticks,
    }


# Fluids: the item slows, floats up while it sinks slowly, and the flow of
# a current pushes it, whichever fluid it is in and however deep.
def pool(block, depth, low=-3, high=3, properties=None):
    out = []
    for x in range(low, high + 1):
        for z in range(low, high + 1):
            for y in range(64, 64 + depth):
                placed = {"x": x, "y": y, "z": z, "block": block}
                if properties:
                    placed["properties"] = properties
                out.append(placed)
    return out


def level(x, y, z, depth, block="minecraft:water"):
    return {"x": x, "y": y, "z": z, "block": block, "properties": {"level": str(depth)}}


# The server's packets between ticks. A relative move counts from where
# the last packet put the item, not from where the client has moved it to.
def move(tick, xa, ya, za, on_ground=False):
    return {"tick": tick, "type": "move", "xa": xa, "ya": ya, "za": za,
            "on_ground": on_ground}


def sync(tick, position, motion=(0.0, 0.0, 0.0), on_ground=False):
    return {"tick": tick, "type": "sync", "x": float_bits(position[0], "f64"),
            "y": float_bits(position[1], "f64"), "z": float_bits(position[2], "f64"),
            "motion_x": float_bits(motion[0], "f64"), "motion_y": float_bits(motion[1], "f64"),
            "motion_z": float_bits(motion[2], "f64"), "on_ground": on_ground}


# Projectiles the server moves in coarse steps: the client carries them on between its
# packets. The packet's own values (position, motion and turn) are exact multiples of what
# the wire holds, and the statics (owner, targets) stand where the scene puts them.
def proj(label, kind, blocks, position, motion, ticks=30, y_rot=0.0, x_rot=0.0, others=(),
         owner=None, dangerous=None, entity_id=21):
    out = {
        "label": label, "blocks": blocks, "chunk_radius": 1, "entity_id": entity_id,
        "type": kind, "tick_count": ticks, "y_rot": float_bits(y_rot), "x_rot": float_bits(x_rot),
        "others": [{"id": other_id, "type": other_kind, "x": float_bits(at[0], "f64"),
                    "y": float_bits(at[1], "f64"), "z": float_bits(at[2], "f64")}
                   for other_id, other_kind, at in others],
    }
    for axis, value in zip("xyz", position):
        out[axis] = float_bits(value, "f64")
    for axis, value in zip("xyz", motion):
        out["motion_" + axis] = float_bits(value, "f64")
    if owner is not None:
        out["owner"] = owner
    if dangerous is not None:
        out["dangerous"] = dangerous
    return out


# An entity's shadow: the block squares under it that are lit and whole
# enough, how dark each is, and the quad drawn for each.
def shadow(label, blocks, position=(0.5, 64.0, 0.5), radius=0.7, strength=1.0,
           distance_sq=0.0, invisible=False, option=True, translation=(0.5, 0.0, 3.0)):
    return {"label": label, "blocks": blocks, "chunk_radius": 1,
            "x": float_bits(position[0], "f64"), "y": float_bits(position[1], "f64"),
            "z": float_bits(position[2], "f64"), "radius": float_bits(radius),
            "strength": float_bits(strength), "distance_sq": float_bits(distance_sq, "f64"),
            "invisible": invisible, "shadows_option": option,
            "tx": float_bits(translation[0]), "ty": float_bits(translation[1]),
            "tz": float_bits(translation[2])}


on_top = lambda block, props=None: [dict({"x": 0, "y": 63, "z": 0, "block": block},
                                          **({"properties": props} if props else {}))]


floor_of = lambda block: [{"x": x, "y": 63, "z": z, "block": block}
                          for x in range(-8, 9) for z in range(-8, 9)]


# A living entity hurt and killed by the packets that do it, ticked on.
def living(label, entity, ticks, events, player_x=None, extras=None):
    row = {"label": label, "blocks": [], "chunk_radius": 0, "entity": f"minecraft:{entity}",
            "entity_id": 40, "x": float_bits(0.5, "f64"), "y": float_bits(64.0, "f64"),
            "z": float_bits(0.5, "f64"), "tick_count": ticks, "events": events}
    if player_x is not None:
        row["player_x"] = float_bits(player_x, "f64")
    if extras:
        row["extras"] = extras
    return row


def display_data(tick, **fields):
    """A display's data the server sent: floats and vectors in their bits, ints as they are."""
    event = {"tick": tick, "type": "display_data"}
    for key, value in fields.items():
        if key in ("translation", "scale", "left_rotation", "right_rotation"):
            event[key] = [float_bits(part) for part in value]
        elif key in ("view_range", "shadow_radius", "shadow_strength", "width", "height"):
            event[key] = float_bits(value)
        else:
            event[key] = value
    return event


def cart_step(x, y, z, mx, my, mz, y_rot, x_rot, weight):
    """A step along the track the server sent (the turns in the byte the wire packs them in)."""
    return {"x": float_bits(x, "f64"), "y": float_bits(y, "f64"), "z": float_bits(z, "f64"),
            "mx": float_bits(mx, "f64"), "my": float_bits(my, "f64"), "mz": float_bits(mz, "f64"),
            "y_rot": y_rot, "x_rot": x_rot, "weight": float_bits(weight)}


# What the animation packets and events do to a living entity: the swing of
# an arm (`Animate`), a sheep told to eat, a chicken's wings.
animate = lambda tick, action: {"tick": tick, "type": "animate", "action": action}


on_ground = lambda tick, ground: {"tick": tick, "type": "move", "xa": 0, "ya": 0, "za": 0,
                                   "on_ground": ground}


swing = ["swing"]


def squid(label, ticks, events, water="deep", entity="squid", speed=0.15):
    """A squid at the middle of the scene's only loaded chunk, in a pool of water, a layer of
    it that reaches only part of its box, or none, its tentacle speed (drawn unseeded by the
    constructor) fixed."""
    layers = {"deep": range(60, 70), "shallow": range(64, 65), "none": range(0)}[water]
    row = living(label, entity, ticks,
                 [{"tick": 0, "type": "squid_speed", "speed": float_bits(speed)}] + events,
                 extras=["squid"])
    row["x"] = float_bits(8.5, "f64")
    row["z"] = float_bits(8.5, "f64")
    row["blocks"] = [{"x": x, "y": y, "z": z, "block": "minecraft:water"}
                     for y in layers for x in range(4, 13) for z in range(4, 13)]
    return row


def fish(label, entity, ticks, events, water="deep", y=64.0, extras="fish"):
    """A swimmer at the middle of the scene's only loaded chunk, in a pool of water, a layer of it
    that reaches only part of its box, or none; `y` is its feet."""
    layers = {"deep": range(60, 70), "shallow": range(64, 65), "none": range(0)}[water]
    row = living(label, entity, ticks, events, extras=[extras])
    row["x"] = float_bits(8.5, "f64")
    row["y"] = float_bits(y, "f64")
    row["z"] = float_bits(8.5, "f64")
    row["blocks"] = [{"x": x, "y": y, "z": z, "block": "minecraft:water"}
                     for y in layers for x in range(4, 13) for z in range(4, 13)]
    return row


def beside(row, entity, x, y, z):
    """The row with another entity, which stays where it is."""
    row["second"] = {"entity": f"minecraft:{entity}", "entity_id": 41,
                     "x": float_bits(x, "f64"), "y": float_bits(y, "f64"),
                     "z": float_bits(z, "f64")}
    return row


guardian_random = lambda tail, seed: {"tick": 0, "type": "guardian_random",
                                      "tail": float_bits(tail), "seed": seed}


fish_data = lambda tick, fish, value: {"tick": tick, "type": "fish_data", "fish": fish,
                                       "value": value}


cube_size = lambda tick, size: {"tick": tick, "type": "cube_size", "size": size}


head_turn = lambda tick, angle: {"tick": tick, "type": "head_rotation", "angle": angle}


shulker_data = lambda tick, **fields: {"tick": tick, "type": "shulker_data", **fields}


move_by = lambda tick, xa, ya, za: {"tick": tick, "type": "move", "xa": xa, "ya": ya, "za": za}


sync_to = lambda tick, x, y, z: {"tick": tick, "type": "sync", "x": float_bits(x, "f64"),
                                 "y": float_bits(y, "f64"), "z": float_bits(z, "f64"),
                                 "motion_x": float_bits(0.0, "f64"),
                                 "motion_y": float_bits(0.0, "f64"),
                                 "motion_z": float_bits(0.0, "f64"), "on_ground": False}


passengers = lambda tick, vehicle, riders: {"tick": tick, "type": "passengers",
                                            "vehicle": vehicle, "riders": riders}


def shulker_at(label, ticks, events, x, y, z, vehicle=False):
    """A shulker put where the packet says, not yet held to a block; a boat beside it to ride."""
    row = living(label, "shulker", ticks, events, extras=["shulker"])
    row["x"], row["y"], row["z"] = (float_bits(x, "f64"), float_bits(y, "f64"),
                                    float_bits(z, "f64"))
    if vehicle:
        row = beside(row, "oak_boat", 3.5, 64.0, 5.5)
    return row


hops = [on_ground(2, True), on_ground(9, False), on_ground(21, True), on_ground(30, False),
        on_ground(33, True)]


motion = lambda tick, x, y, z: {"tick": tick, "type": "entity_motion", "x": float_bits(x, "f64"),
                                "y": float_bits(y, "f64"), "z": float_bits(z, "f64")}
