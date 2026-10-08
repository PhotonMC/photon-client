"""The movement family: the player and the world: the `containers` cases."""

from cases.movement.player.common import *
from cases.movement.player.shared import *

import struct


def varint(value):
    value &= 0xFFFFFFFF
    out = bytearray()
    while value & ~0x7F:
        out.append((value & 0x7F) | 0x80)
        value >>= 7
    out.append(value)
    return out.hex()


def nbt(value):
    """A network tag (its type, then its payload) of a Python value: str, a
    (type, value) pair for numbers and arrays, a list or a dict."""
    if isinstance(value, str):
        encoded = value.encode("utf-8")
        return "08" + len(encoded).to_bytes(2, "big").hex() + encoded.hex()
    if isinstance(value, dict):
        payload = ""
        for key, inner in value.items():
            tag = nbt(inner)
            encoded = key.encode("utf-8")
            payload += tag[:2] + len(encoded).to_bytes(2, "big").hex() + encoded.hex() + tag[2:]
        return "0a" + payload + "00"
    if isinstance(value, list):
        element = nbt(value[0])[:2] if value else "00"
        if any(nbt(item)[:2] != element for item in value):
            # A list of mixed types: each element wrapped in a compound under "".
            value = [item if isinstance(item, dict) else {"": item} for item in value]
            element = "0a"
        return "09" + element + len(value).to_bytes(4, "big").hex() + "".join(nbt(item)[2:] for item in value)
    kind, number = value
    sizes = {"b": ("01", 1), "s": ("02", 2), "i": ("03", 4), "l": ("04", 8)}
    if kind in sizes:
        tag, size = sizes[kind]
        return tag + number.to_bytes(size, "big", signed=True).hex()
    if kind == "f":
        return "05" + struct.pack(">f", number).hex()
    if kind == "d":
        return "06" + struct.pack(">d", number).hex()
    arrays = {"B": ("07", 1), "I": ("0b", 4), "L": ("0c", 8)}
    tag, size = arrays[kind]
    return tag + len(number).to_bytes(4, "big").hex() + "".join(
        item.to_bytes(size, "big", signed=True).hex() for item in number)


# Data component type ids, and the vanilla enchantments' network ids.
CUSTOM_DATA, MAX_STACK_SIZE, MAX_DAMAGE, DAMAGE, UNBREAKABLE = 0, 1, 2, 3, 4
CUSTOM_NAME, ITEM_NAME, LORE, RARITY, ENCHANTMENTS = 6, 9, 11, 12, 13
REPAIR_COST, GLINT, STORED_ENCHANTMENTS, DYED_COLOR = 19, 21, 42, 44
EFFICIENCY, MENDING, PROTECTION, SHARPNESS, UNBREAKING = 8, 23, 28, 33, 40
BINDING_CURSE, BUNDLE_CONTENTS, VANISHING_CURSE = 2, 50, 41
# Item registry ids of the items a bundle holds here.
ITEM_IDS = {"stone": 1, "dirt": 55, "ender_pearl": 1144, "bundle": 1065, "iron_sword": 959}


def bundle(items):
    """A `bundle_contents` patch: (item, count, patch hex) per stack, first first."""
    value = varint(len(items)) + "".join(
        varint(ITEM_IDS[item]) + varint(count) + inner for item, count, inner in items)
    return patch([(BUNDLE_CONTENTS, value)])


def patch(added=(), removed=()):
    """`DataComponentPatch.STREAM_CODEC`: (type id, value hex) pairs, then removed type ids."""
    return (varint(len(added)) + varint(len(removed))
            + "".join(varint(kind) + value for kind, value in added)
            + "".join(varint(kind) for kind in removed))


def enchantments(levels):
    return varint(len(levels)) + "".join(varint(kind) + varint(level) for kind, level in levels)


def with_patch(stack, hex_patch):
    stack["patch"] = hex_patch
    return stack


def cases():
    P, Q, SW, CL, TH, QC, PA = "PICKUP", "QUICK_MOVE", "SWAP", "CLONE", "THROW", "QUICK_CRAFT", "PICKUP_ALL"

    merchant_offers = [
        offer(("emerald", 1), ("bread", 6)),
        offer(("wheat", 20), ("emerald", 1), max_uses=16, demand=4),
        offer(("emerald", 3), ("bookshelf", 1), buy_b=("paper", 2), special=-1),
        offer(("diamond", 2), ("cake", 1), uses=12),
    ]

    click_rows = [
        clicks("pick up and put down", [st(36, "stone", 64)], [cl(36, 0, P), cl(10, 0, P)], state_id=5),
        clicks("halves and single placements", [st(36, "stone", 63)],
               [cl(36, 1, P), cl(11, 1, P), cl(12, 1, P), cl(11, 0, P), cl(36, 0, P)]),
        clicks("swap unlike stacks", [st(36, "stone", 12)], [cl(36, 0, P), cl(37, 0, P)],
               carried=("dirt", 10)),
        clicks("merge over the limit", [st(20, "stone", 50)], [cl(20, 0, P), cl(21, 1, P)],
               carried=("stone", 40)),
        clicks("sixteen stack limit", [st(20, "ender_pearl", 10)], [cl(20, 0, P)],
               carried=("ender_pearl", 10)),
        clicks("unstackable swap", [st(20, "diamond_sword", 1)], [cl(20, 0, P), cl(21, 0, P)],
               carried=("iron_sword", 1)),
        clicks("outside the window", [], [cl(-999, 1, P), cl(-999, 0, P)], carried=("stone", 5)),
        clicks("quick move rows", [st(36, "stone", 64), st(10, "dirt", 30), st(37, "dirt", 50)],
               [cl(36, 0, Q), cl(10, 0, Q), cl(9, 0, Q)]),
        clicks("quick move merges widely",
               [st(i, "stone", 60) for i in range(9, 36)] + [st(36, "stone", 64), st(40, "stone", 1)],
               [cl(36, 0, Q)]),
        clicks("quick move armour and shield",
               [st(20, "diamond_helmet", 1), st(21, "iron_boots", 1), st(22, "shield", 1),
                st(23, "carved_pumpkin", 1), st(24, "white_carpet", 3)],
               [cl(20, 0, Q), cl(21, 0, Q), cl(22, 0, Q), cl(23, 0, Q), cl(24, 0, Q), cl(5, 0, Q)]),
        clicks("armour slots refuse and cap",
               [st(36, "stone", 64), st(37, "diamond_chestplate", 1), st(38, "leather_leggings", 1)],
               [cl(36, 0, P), cl(6, 0, P), cl(36, 0, P), cl(37, 0, P), cl(6, 0, P), cl(38, 0, P), cl(6, 0, P)]),
        clicks("number keys", [st(15, "stone", 20), st(38, "dirt", 5), st(45, "torch", 3)],
               [cl(15, 0, SW), cl(15, 2, SW), cl(16, 5, SW), cl(20, 40, SW), cl(36, 40, SW)]),
        clicks("number key into armour",
               [st(36, "diamond_helmet", 1), st(37, "stone", 64), st(5, "iron_helmet", 1)],
               [cl(5, 0, SW), cl(5, 1, SW), cl(8, 1, SW)]),
        clicks("throw one and all", [st(36, "stone", 5), st(20, "dirt", 3)],
               [cl(36, 0, TH), cl(36, 1, TH), cl(20, 0, TH), cl(21, 1, TH)]),
        clicks("double click gathers",
               [st(10, "stone", 30), st(11, "stone", 64), st(12, "stone", 10), st(40, "stone", 2),
                st(13, "dirt", 5)],
               [cl(20, 0, PA)], carried=("stone", 5)),
        clicks("double click from the end",
               [st(10, "stone", 30), st(44, "stone", 40), st(12, "stone", 10)],
               [cl(20, 1, PA)], carried=("stone", 3)),
        clicks("left drag splits", [],
               [cl(-999, 0, QC), cl(10, 1, QC), cl(11, 1, QC), cl(12, 1, QC), cl(11, 1, QC), cl(-999, 2, QC)],
               carried=("stone", 64)),
        clicks("right drag places one each", [st(11, "stone", 63)],
               [cl(-999, 4, QC), cl(10, 5, QC), cl(11, 5, QC), cl(12, 5, QC), cl(-999, 6, QC)],
               carried=("stone", 10)),
        clicks("drag over one slot is a click", [],
               [cl(-999, 0, QC), cl(10, 1, QC), cl(-999, 2, QC)], carried=("stone", 7)),
        clicks("middle drag refused in survival", [],
               [cl(-999, 8, QC), cl(10, 9, QC), cl(-999, 10, QC), cl(10, 0, P)], carried=("stone", 7)),
        clicks("broken drag resets", [],
               [cl(-999, 0, QC), cl(10, 1, QC), cl(11, 0, P)], carried=("stone", 7)),
        clicks("take the crafting result",
               [st(0, "oak_planks", 4), st(1, "oak_log", 3), st(2, "water_bucket", 1)],
               [cl(0, 0, P), cl(0, 0, P), cl(20, 0, P)]),
        clicks("quick move the crafting result",
               [st(0, "oak_planks", 4), st(1, "oak_log", 2), st(36, "oak_planks", 62)],
               [cl(0, 0, Q)]),
        clicks("crafting grid remainders",
               [st(0, "cake", 1), st(1, "milk_bucket", 1), st(2, "sugar", 2), st(3, "egg", 1),
                st(4, "honey_bottle", 3)],
               [cl(0, 0, P)]),
        clicks("result refuses placing", [st(0, "stick", 4)], [cl(0, 0, P)], carried=("stick", 2)),
        clicks("number key from the result", [st(0, "oak_planks", 4), st(1, "oak_log", 2)],
               [cl(0, 3, SW)]),
        clicks("clone needs creative", [st(10, "stone", 3)], [cl(10, 2, CL)]),
        clicks("right drag over thirty slots grows the changed map", [],
               [cl(-999, 4, QC)] + [cl(i, 5, QC) for i in range(9, 45)] + [cl(-999, 6, QC)],
               carried=("stone", 64)),
        clicks("left drag with the zero slot key", [st(0, "stick", 1)],
               [cl(0, 0, TH), cl(-999, 0, QC), cl(1, 1, QC), cl(2, 1, QC), cl(-999, 2, QC)],
               carried=("stone", 9)),
    ]

    # Menus the server opens: the container's own slots first, then the 27 of
    # the main inventory and the 9 of the hotbar.
    chest = "generic_9x3"

    click_rows += [
        clicks("chest pick up and put down", [st(0, "stone", 64)],
               [cl(0, 0, P), cl(5, 0, P), cl(5, 1, P), cl(30, 0, P)], menu=chest, state_id=2),
        clicks("chest quick move out and in",
               [st(0, "stone", 40), st(1, "diamond_sword", 1), st(30, "dirt", 3), st(57, "stone", 50)],
               [cl(0, 0, Q), cl(1, 0, Q), cl(30, 0, Q), cl(57, 0, Q)], menu=chest),
        clicks("chest quick move merges and fills",
               [st(i, "stone", 64) for i in range(0, 26)] + [st(26, "stone", 60), st(40, "stone", 30)],
               [cl(40, 0, Q)], menu=chest),
        clicks("chest number keys", [st(3, "stone", 20), st(55, "dirt", 5)],
               [cl(3, 1, SW), cl(4, 1, SW), cl(3, 8, SW), cl(28, 0, SW), cl(3, 40, SW)], menu=chest),
        clicks("chest throw", [st(2, "stone", 5)], [cl(2, 0, TH), cl(2, 1, TH)], menu=chest),
        clicks("chest left drag over both parts", [],
               [cl(-999, 0, QC), cl(3, 1, QC), cl(4, 1, QC), cl(40, 1, QC), cl(58, 1, QC), cl(-999, 2, QC)],
               carried=("stone", 64), menu=chest),
        clicks("chest right drag", [st(4, "stone", 63)],
               [cl(-999, 4, QC), cl(3, 5, QC), cl(4, 5, QC), cl(62, 5, QC), cl(-999, 6, QC)],
               carried=("stone", 10), menu=chest),
        clicks("chest double click gathers across both parts",
               [st(0, "stone", 30), st(40, "stone", 64), st(41, "stone", 10), st(62, "stone", 2)],
               [cl(5, 0, PA), cl(5, 1, PA)], carried=("stone", 5), menu=chest),
        clicks("clone in a chest", [st(1, "stone", 3)], [cl(1, 2, CL)], menu=chest),
        clicks("one row chest quick move", [st(0, "stone", 5), st(9, "dirt", 7)],
               [cl(0, 0, Q), cl(9, 0, Q)], menu="generic_9x1"),
        clicks("six row chest quick move", [st(0, "stone", 5), st(54, "dirt", 7)],
               [cl(0, 0, Q), cl(54, 0, Q)], menu="generic_9x6"),
        clicks("hopper quick move", [st(0, "stone", 5), st(1, "stone", 64), st(5, "dirt", 7)],
               [cl(0, 0, Q), cl(5, 0, Q), cl(1, 0, Q)], menu="hopper"),
        clicks("dispenser quick move gives up on no change",
               [st(0, "stone", 5), st(9, "dirt", 7), st(10, "stone", 64)] +
               [st(i, "cobblestone", 64) for i in range(1, 9)] + [st(11, "diamond_sword", 1)],
               [cl(0, 0, Q), cl(9, 0, Q), cl(11, 0, Q)], menu="generic_3x3"),
        clicks("shulker box refuses shulker boxes",
               [st(27, "shulker_box", 1), st(28, "red_shulker_box", 1), st(29, "stone", 4), st(0, "dirt", 2)],
               [cl(27, 0, Q), cl(28, 0, P), cl(1, 0, P), cl(29, 0, Q), cl(0, 0, Q)], menu="shulker_box"),
        clicks("shulker box pick up a shulker box", [st(27, "shulker_box", 1)],
               [cl(27, 0, P), cl(3, 0, P), cl(40, 0, P)], menu="shulker_box"),
    ]

    # The crafting table: the result at 0, the grid at 1 to 9, then the player.
    crafting = "crafting"

    click_rows += [
        clicks("table take the result", [st(0, "oak_planks", 4), st(1, "oak_log", 3), st(2, "water_bucket", 1)],
               [cl(0, 0, P), cl(0, 0, P), cl(20, 0, P)], menu=crafting),
        clicks("table quick move the result",
               [st(0, "oak_planks", 4), st(1, "oak_log", 2), st(36, "oak_planks", 62)],
               [cl(0, 0, Q)], menu=crafting),
        clicks("table grid remainders",
               [st(0, "cake", 1), st(1, "milk_bucket", 1), st(9, "sugar", 2), st(5, "egg", 1),
                st(8, "honey_bottle", 3)],
               [cl(0, 0, P)], menu=crafting),
        clicks("table quick move into the grid", [st(20, "stone", 30), st(40, "dirt", 3), st(3, "stone", 60)],
               [cl(20, 0, Q), cl(40, 0, Q)], menu=crafting),
        clicks("table quick move out of the grid", [st(4, "stone", 5), st(9, "dirt", 7)],
               [cl(4, 0, Q), cl(9, 0, Q)], menu=crafting),
        clicks("table result refuses placing", [st(0, "stick", 4)], [cl(0, 0, P)],
               carried=("stick", 2), menu=crafting),
        clicks("table number key from the result", [st(0, "oak_planks", 4), st(1, "oak_log", 2)],
               [cl(0, 3, SW)], menu=crafting),
        clicks("table double click skips the result",
               [st(0, "stone", 30), st(10, "stone", 10), st(45, "stone", 4)],
               [cl(11, 0, PA)], carried=("stone", 5), menu=crafting),
        clicks("table drag over the grid", [],
               [cl(-999, 0, QC), cl(1, 1, QC), cl(2, 1, QC), cl(30, 1, QC), cl(-999, 2, QC)],
               carried=("stone", 9), menu=crafting),
    ]

    # The furnaces: the ingredient at 0, the fuel at 1, the result at 2.
    smelts = ["raw_iron", "beef", "cobblestone", "sand"]

    click_rows += [
        clicks("furnace quick move sorts by use",
               [st(3, "raw_iron", 10), st(4, "coal", 20), st(5, "dirt", 7), st(30, "beef", 3),
                st(31, "charcoal", 5), st(6, "lava_bucket", 1)],
               [cl(3, 0, Q), cl(4, 0, Q), cl(5, 0, Q), cl(30, 0, Q), cl(31, 0, Q), cl(6, 0, Q)],
               menu="furnace", furnace_inputs=smelts),
        clicks("furnace quick move out", [st(0, "raw_iron", 5), st(1, "coal", 3), st(2, "iron_ingot", 7)],
               [cl(0, 0, Q), cl(1, 0, Q), cl(2, 0, Q)], menu="furnace", furnace_inputs=smelts),
        clicks("furnace fuel slot takes fuel and buckets only",
               [st(3, "coal", 10), st(4, "dirt", 5), st(5, "bucket", 3), st(6, "water_bucket", 1)],
               [cl(3, 0, P), cl(1, 0, P), cl(3, 0, P), cl(4, 0, P), cl(1, 0, P), cl(4, 0, P),
                cl(5, 0, P), cl(1, 1, P), cl(5, 0, P), cl(6, 0, P), cl(1, 0, P)],
               menu="furnace", furnace_inputs=smelts),
        clicks("furnace ingredient slot takes anything", [st(3, "dirt", 10)],
               [cl(3, 0, P), cl(0, 0, P), cl(0, 1, P)], menu="furnace", furnace_inputs=smelts),
        clicks("furnace result only comes out", [st(2, "iron_ingot", 7), st(3, "iron_ingot", 4)],
               [cl(2, 1, P), cl(3, 0, P), cl(2, 0, P), cl(2, 0, P), cl(10, 0, P)],
               menu="furnace", furnace_inputs=smelts),
        clicks("furnace double click gathers the result too",
               [st(2, "iron_ingot", 9), st(3, "iron_ingot", 10), st(4, "iron_ingot", 20)],
               [cl(12, 0, PA)], carried=("iron_ingot", 5), menu="furnace", furnace_inputs=smelts),
        clicks("blast furnace quick move", [st(3, "raw_iron", 10), st(4, "coal", 20), st(5, "beef", 4)],
               [cl(3, 0, Q), cl(4, 0, Q), cl(5, 0, Q)], menu="blast_furnace", furnace_inputs=smelts),
        clicks("smoker quick move and number key", [st(3, "beef", 10), st(4, "stick", 20), st(2, "cooked_beef", 4)],
               [cl(3, 0, Q), cl(4, 0, Q), cl(2, 2, SW), cl(2, 0, TH)], menu="smoker", furnace_inputs=smelts),
    ]

    # The menus that turn their items into something: the client runs the
    # slots' rules, what taking a result uses up and quickMoveStack.
    smith_sets = {"recipe_sets": {
        "minecraft:smithing_template": ["netherite_upgrade_smithing_template"],
        "minecraft:smithing_base": ["diamond_sword", "diamond_chestplate"],
        "minecraft:smithing_addition": ["netherite_ingot"],
    }}

    click_rows += [
        clicks("grindstone takes damageable and enchanted only",
               [st(30, "diamond_sword", 1), st(31, "stone", 4), st(32, "iron_pickaxe", 1)],
               [cl(30, 0, P), cl(0, 0, P), cl(31, 0, P), cl(1, 0, P), cl(1, 1, P), cl(32, 0, P), cl(1, 0, P)],
               menu="grindstone"),
        clicks("grindstone quick move",
               [st(5, "diamond_sword", 1), st(6, "iron_pickaxe", 1), st(7, "golden_axe", 1), st(8, "stone", 3)],
               [cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(8, 0, Q), cl(0, 0, Q)], menu="grindstone"),
        clicks("grindstone take the result",
               [st(0, "diamond_sword", 1), st(1, "diamond_sword", 1), st(2, "diamond_sword", 1)],
               [cl(2, 0, P), cl(20, 0, P)], menu="grindstone"),
        clicks("grindstone quick move the result", [st(0, "iron_sword", 1), st(2, "iron_sword", 1)],
               [cl(2, 0, Q)], menu="grindstone"),
        clicks("cartography slots", [st(30, "paper", 8), st(31, "map", 1), st(32, "glass_pane", 2),
                                      st(33, "stone", 5)],
               [cl(30, 0, P), cl(1, 0, P), cl(31, 0, P), cl(0, 0, P), cl(0, 0, P), cl(32, 0, P), cl(1, 0, P),
                cl(33, 0, P), cl(1, 0, P)], menu="cartography_table"),
        clicks("cartography quick move", [st(5, "paper", 8), st(6, "map", 1), st(7, "stone", 3),
                                           st(31, "glass_pane", 4)],
               [cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(31, 0, Q)], menu="cartography_table"),
        clicks("cartography take the result", [st(0, "filled_map", 2), st(1, "paper", 3), st(2, "filled_map", 2)],
               [cl(2, 0, P), cl(20, 0, P)], menu="cartography_table"),
        clicks("smithing slots by set",
               [st(30, "netherite_upgrade_smithing_template", 1), st(31, "diamond_sword", 1),
                st(32, "netherite_ingot", 2), st(33, "stone", 3), st(34, "diamond_sword", 1)],
               [cl(30, 0, P), cl(0, 0, P), cl(31, 0, P), cl(1, 0, P), cl(32, 0, P), cl(2, 0, P),
                cl(33, 0, P), cl(2, 0, P), cl(34, 0, P), cl(0, 0, P)],
               menu="smithing", extra=smith_sets),
        clicks("smithing quick move",
               [st(5, "netherite_upgrade_smithing_template", 1), st(6, "diamond_sword", 1),
                st(7, "netherite_ingot", 2), st(8, "stone", 3), st(9, "diamond_chestplate", 1)],
               [cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(8, 0, Q), cl(9, 0, Q), cl(0, 0, Q)],
               menu="smithing", extra=smith_sets),
        clicks("smithing take the result",
               [st(0, "netherite_upgrade_smithing_template", 1), st(1, "diamond_sword", 1),
                st(2, "netherite_ingot", 3), st(3, "netherite_sword", 1)],
               [cl(3, 0, P), cl(20, 0, P)], menu="smithing", extra=smith_sets),
        clicks("smithing result leaves by quick move",
               [st(0, "netherite_upgrade_smithing_template", 1), st(1, "diamond_sword", 1),
                st(3, "netherite_sword", 1)],
               [cl(3, 0, Q)], menu="smithing", extra=smith_sets),
        clicks("stonecutter quick move", [st(2, "stone", 10), st(3, "dirt", 4), st(30, "cobblestone", 8),
                                           st(1, "stone_slab", 3)],
               [cl(2, 0, Q), cl(3, 0, Q), cl(30, 0, Q), cl(1, 0, Q)],
               menu="stonecutter", extra={"stonecutter_inputs": ["stone", "cobblestone"]}),
        clicks("stonecutter take the result", [st(0, "stone", 5), st(1, "stone_slab", 2)],
               [cl(1, 0, P), cl(20, 0, P), cl(0, 0, P)],
               menu="stonecutter", extra={"stonecutter_inputs": ["stone"]}),
        clicks("stonecutter double click skips the result",
               [st(1, "stone_slab", 2), st(10, "stone_slab", 5), st(30, "stone_slab", 9)],
               [cl(12, 0, PA)], carried=("stone_slab", 3),
               menu="stonecutter", extra={"stonecutter_inputs": ["stone"]}),
        clicks("loom slots by tag and component",
               [st(31, "white_banner", 1), st(32, "red_dye", 4), st(33, "creeper_banner_pattern", 1),
                st(34, "stone", 2), st(35, "sugar", 2)],
               [cl(31, 0, P), cl(0, 0, P), cl(32, 0, P), cl(0, 0, P), cl(1, 0, P), cl(34, 0, P), cl(1, 0, P),
                cl(35, 0, P), cl(2, 0, P)], menu="loom"),
        clicks("loom pattern with the banner",
               [st(31, "white_banner", 1), st(33, "creeper_banner_pattern", 1), st(34, "flower_banner_pattern", 1)],
               [cl(31, 0, P), cl(0, 0, P), cl(33, 0, P), cl(2, 0, P), cl(34, 0, P), cl(2, 0, P)], menu="loom"),
        clicks("loom quick move",
               [st(5, "white_banner", 1), st(6, "blue_dye", 3), st(7, "flower_banner_pattern", 1),
                st(8, "stone", 3), st(9, "black_banner", 1)],
               [cl(5, 0, Q), cl(8, 0, Q), cl(9, 0, Q), cl(3, 0, Q), cl(6, 0, Q)], menu="loom"),
        clicks("loom pattern quick move", [st(7, "flower_banner_pattern", 1), st(8, "stone", 3)],
               [cl(7, 0, Q), cl(8, 0, Q)], menu="loom"),
        clicks("loom take the result",
               [st(0, "white_banner", 2), st(1, "red_dye", 1), st(3, "white_banner", 1)],
               [cl(3, 0, P), cl(20, 0, P)], menu="loom"),
        clicks("loom result keeps splitting",
               [st(0, "white_banner", 1), st(1, "red_dye", 4), st(3, "white_banner", 3)],
               [cl(3, 1, P), cl(20, 0, P), cl(3, 0, P)], menu="loom"),
        clicks("anvil takes anything in its inputs", [st(30, "stone", 4), st(31, "diamond_sword", 1)],
               [cl(30, 0, P), cl(0, 0, P), cl(31, 0, P), cl(1, 0, P)], menu="anvil"),
        clicks("anvil result needs the levels",
               [st(0, "iron_sword", 1), st(2, "iron_sword", 1)],
               [cl(2, 0, P), cl(20, 0, P), cl(2, 0, P)], menu="anvil",
               extra={"data": [5], "xp_level": 3}),
        clicks("anvil result with the levels",
               [st(0, "iron_sword", 1), st(1, "iron_ingot", 2), st(2, "iron_sword", 1)],
               [cl(2, 0, P), cl(20, 0, P)], menu="anvil", extra={"data": [5], "xp_level": 12}),
        clicks("anvil result costs nothing in creative",
               [st(0, "iron_sword", 1), st(2, "iron_sword", 1)],
               [cl(2, 0, P), cl(20, 0, P)], menu="anvil", game_mode="creative", extra={"data": [39]}),
        clicks("anvil quick move", [st(0, "iron_sword", 1), st(2, "iron_sword", 1), st(10, "stone", 2),
                                     st(30, "diamond_sword", 1)],
               [cl(0, 0, Q), cl(2, 0, Q), cl(10, 0, Q), cl(30, 0, Q)], menu="anvil",
               extra={"data": [1], "xp_level": 30}),
        clicks("enchanting table item slot holds one",
               [st(30, "diamond_sword", 3), st(31, "lapis_lazuli", 64), st(32, "stone", 9)],
               [cl(30, 0, P), cl(0, 0, P), cl(0, 0, P), cl(31, 0, P), cl(1, 0, P), cl(32, 0, P), cl(1, 0, P)],
               menu="enchantment"),
        clicks("enchanting table quick move",
               [st(5, "diamond_sword", 4), st(6, "lapis_lazuli", 20), st(7, "stone", 3), st(0, "iron_axe", 1)],
               [cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(0, 0, Q), cl(1, 0, Q), cl(5, 0, Q), cl(5, 0, Q)],
               menu="enchantment"),
        clicks("enchanting table number keys", [st(0, "iron_sword", 1), st(1, "lapis_lazuli", 10)],
               [cl(0, 2, SW), cl(1, 3, SW)], menu="enchantment"),
        clicks("brewing stand slots take potions, one ingredient kind and fuel",
               [st(30, "potion", 3), st(31, "glass_bottle", 5), st(32, "blaze_powder", 20),
                st(33, "nether_wart", 8), st(34, "stone", 4), st(35, "water_bucket", 1)],
               [cl(30, 0, P), cl(0, 0, P), cl(1, 0, P), cl(31, 0, P), cl(2, 0, P), cl(31, 0, P),
                cl(32, 0, P), cl(4, 0, P), cl(32, 0, P), cl(3, 0, P), cl(4, 0, P), cl(33, 1, P),
                cl(3, 0, P), cl(34, 0, P), cl(3, 0, P), cl(4, 0, P), cl(35, 0, P), cl(1, 0, P)],
               menu="brewing_stand"),
        clicks("brewing stand quick move",
               [st(5, "blaze_powder", 4), st(6, "nether_wart", 6), st(7, "potion", 1),
                st(8, "splash_potion", 1), st(9, "glass_bottle", 2), st(10, "stone", 3),
                st(36, "glass_bottle", 1), st(0, "potion", 1), st(3, "nether_wart", 2),
                st(4, "blaze_powder", 5)],
               [cl(5, 0, Q), cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(8, 0, Q), cl(9, 0, Q),
                cl(10, 0, Q), cl(36, 0, Q), cl(0, 0, Q), cl(3, 0, Q), cl(4, 0, Q)],
               menu="brewing_stand"),
        clicks("beacon payment slot holds one payment item",
               [st(5, "iron_ingot", 3), st(6, "emerald", 1), st(7, "stone", 2), st(30, "diamond", 4)],
               [cl(5, 0, P), cl(0, 0, P), cl(0, 0, P), cl(6, 0, P), cl(0, 0, P), cl(7, 0, P), cl(0, 0, P),
                cl(30, 1, P), cl(0, 0, P), cl(0, 0, P)],
               menu="beacon"),
        clicks("beacon quick move",
               [st(5, "iron_ingot", 3), st(6, "emerald", 1), st(7, "stone", 2), st(30, "diamond", 4),
                st(8, "netherite_ingot", 1), st(0, "gold_ingot", 1)],
               [cl(5, 0, Q), cl(6, 0, Q), cl(7, 0, Q), cl(30, 0, Q), cl(8, 0, Q), cl(0, 0, Q), cl(6, 0, Q),
                cl(8, 0, Q)],
               menu="beacon"),
        clicks("crafter disabled slot takes nothing",
               [st(20, "stone", 10), st(21, "stick", 3), st(2, "stone", 2)],
               [cl(20, 0, P), cl(1, 0, P), cl(0, 0, P), cl(1, 1, P), cl(21, 0, P), cl(1, 0, P),
                cl(4, 0, P), cl(2, 0, P)],
               menu="crafter_3x3", extra={"data": [0, 1, 0, 0, 1, 0, 0, 0, 0, 0]}),
        clicks("crafter quick move",
               [st(20, "stone", 10), st(21, "stick", 3), st(2, "stone", 2), st(3, "dirt", 5),
                st(44, "stone", 8)],
               [cl(20, 0, Q), cl(21, 0, Q), cl(2, 0, Q), cl(3, 0, Q), cl(44, 0, Q)],
               menu="crafter_3x3", extra={"data": [1, 0, 0, 1, 0, 0, 0, 0, 0, 0]}),
        clicks("crafter result slot gives nothing",
               [st(45, "stick", 4), st(20, "stone", 3), st(21, "stick", 5)],
               [cl(45, 0, P), cl(45, 1, P), cl(45, 0, Q), cl(20, 0, P), cl(45, 0, P), cl(21, 0, PA),
                cl(45, 0, TH), cl(0, 2, SW), cl(45, 2, SW)],
               menu="crafter_3x3"),
        clicks("merchant payment makes the result and taking it pays the price",
               [st(3, "emerald", 5), st(4, "wheat", 40)],
               [cl(3, 0, P), cl(0, 0, P), cl(2, 0, P), cl(10, 0, P), cl(0, 0, P), cl(4, 0, P), cl(1, 0, P),
                cl(0, 0, P)],
               menu="merchant", extra={"offers": merchant_offers}),
        clicks("merchant demand raises the price",
               [st(0, "wheat", 30), st(1, "paper", 2), st(10, "emerald", 3)],
               [cl(2, 0, P), cl(11, 0, P), cl(0, 0, P), cl(10, 0, P), cl(0, 0, P)],
               menu="merchant", extra={"offers": merchant_offers}),
        clicks("merchant takes two payments and an out of stock offer gives nothing",
               [st(0, "emerald", 7), st(1, "paper", 5), st(5, "diamond", 2)],
               [cl(2, 0, P), cl(12, 0, P), cl(5, 0, P), cl(0, 0, P), cl(0, 0, P), cl(2, 0, P)],
               menu="merchant", extra={"offers": merchant_offers}),
        clicks("merchant quick move",
               [st(0, "emerald", 3), st(1, "paper", 2), st(4, "emerald", 7), st(33, "wheat", 40),
                st(10, "bread", 12)],
               [cl(2, 0, Q), cl(4, 0, Q), cl(33, 0, Q), cl(10, 0, Q), cl(0, 0, Q), cl(1, 0, Q)],
               menu="merchant", extra={"offers": merchant_offers}),
        clicks("merchant chosen trade moves its payments in",
               [st(10, "emerald", 5), st(11, "wheat", 40), st(30, "wheat", 30)],
               [cl(2, 0, Q), cl(0, 0, P)],
               menu="merchant", extra={"offers": merchant_offers, "select": 1}),
        clicks("merchant chosen trade gives the payments back first",
               [st(0, "emerald", 3), st(1, "paper", 2), st(11, "wheat", 25), st(12, "emerald", 9)],
               [cl(2, 0, Q)],
               menu="merchant", extra={"offers": merchant_offers, "select": 1}),
        clicks("merchant double click gathers from no slot",
               [st(3, "emerald", 5), st(4, "emerald", 5), st(0, "emerald", 3)],
               [cl(3, 0, P), cl(4, 0, PA)], menu="merchant", extra={"offers": merchant_offers}),
        clicks("brewing stand number keys", [st(0, "potion", 1), st(4, "blaze_powder", 6),
                                              st(30, "glass_bottle", 1), st(31, "stone", 1)],
               [cl(0, 2, SW), cl(4, 3, SW), cl(2, 5, SW), cl(3, 6, SW)], menu="brewing_stand"),
    ]

    named = patch([(CUSTOM_NAME, nbt("Excalibur"))])
    click_rows += [
        clicks("a named sword is hashed with its name",
               [with_patch(st(20, "diamond_sword", 1), named)], [cl(20, 0, P), cl(21, 0, P)]),
        clicks("named stacks of one name merge",
               [with_patch(st(10, "stone", 5), patch([(CUSTOM_NAME, nbt("Rock"))]))], [cl(10, 0, P)],
               carried=("stone", 3)),
        clicks("an enchanted pickaxe swaps with a damaged sword",
               [with_patch(st(20, "iron_sword", 1), patch([(DAMAGE, varint(37))]))], [cl(20, 0, P)],
               carried=("iron_pickaxe", 1)),
        clicks("a stack with a removed component",
               [with_patch(st(36, "diamond_sword", 1), patch(removed=[DAMAGE]))], [cl(36, 0, P), cl(9, 0, P)]),
    ]
    click_rows[-3]["carried"]["patch"] = patch([(CUSTOM_NAME, nbt("Rock"))])
    click_rows[-2]["carried"]["patch"] = patch([(ENCHANTMENTS, enchantments([(EFFICIENCY, 3), (UNBREAKING, 2)]))])
    empty = patch()
    click_rows += [
        clicks("a carried bundle takes a stack in and puts it out",
               [st(20, "stone", 64)], [cl(20, 0, P), cl(21, 1, P), cl(22, 1, P)], carried=("bundle", 1)),
        clicks("a carried bundle takes what fits and then fails",
               [st(20, "stone", 64), st(21, "dirt", 5)], [cl(20, 0, P), cl(21, 0, P)], carried=("bundle", 1)),
        clicks("a stack carried onto a bundle merges and comes back out",
               [with_patch(st(20, "bundle", 1), bundle([("dirt", 5, empty), ("ender_pearl", 2, empty)]))],
               [cl(20, 0, P), cl(20, 1, P), cl(20, 1, P), cl(20, 0, P)], carried=("dirt", 30)),
        clicks("a sword and a bundle go into a bundle",
               [st(20, "iron_sword", 1), with_patch(st(21, "bundle", 1), bundle([("stone", 3, empty)]))],
               [cl(20, 0, P), cl(21, 0, P), cl(22, 1, P)], carried=("bundle", 1)),
        clicks("a shulker box does not go into a bundle",
               [st(20, "bundle", 1)], [cl(20, 0, P), cl(21, 1, P)], carried=("shulker_box", 1)),
        clicks("a secondary click on an empty bundle takes nothing",
               [st(20, "stone", 1), st(21, "bundle", 1)], [cl(21, 1, P), cl(20, 1, P), cl(-999, 0, P)]),
        clicks("cursed armour stays on in survival",
               [with_patch(st(5, "iron_helmet", 1), patch([(ENCHANTMENTS, enchantments([(BINDING_CURSE, 1)]))])),
                st(36, "stone", 1)],
               [cl(5, 0, P), cl(5, 0, SW), cl(5, 0, Q)]),
        clicks("cursed armour comes off in creative",
               [with_patch(st(5, "iron_helmet", 1), patch([(ENCHANTMENTS, enchantments([(BINDING_CURSE, 1)]))]))],
               [cl(5, 0, P)], game_mode="creative"),
    ]
    click_rows[-7]["carried"]["patch"] = bundle([("ender_pearl", 10, empty)])
    # The screen's bundle actions: the wheel over a bundle selects one of its items (against the
    # wheel, wrapping), a secondary click takes the selected one, leaving the slot selects none
    # and tells the server even when nothing was selected, a stack put in keeps the index.
    S, U = "SCROLL", "UNHOVER"
    three = bundle([("dirt", 5, empty), ("ender_pearl", 2, empty), ("stone", 3, empty)])
    click_rows += [
        clicks("the wheel selects a bundle's item and a secondary click takes it",
               [with_patch(st(20, "bundle", 1), three)],
               [cl(20, 1, S), cl(20, -1, S), cl(20, -1, S), cl(20, 1, P), cl(20, 1, P)]),
        clicks("leaving a bundle selects none and says so each time",
               [with_patch(st(20, "bundle", 1), three), st(21, "bundle", 1)],
               [cl(20, 1, S), cl(20, 0, U), cl(20, 0, U), cl(21, 1, S), cl(21, 0, U), cl(22, 0, U)]),
        clicks("a stack put into a bundle keeps the selected index",
               [with_patch(st(20, "bundle", 1), three)],
               [cl(20, -1, S), cl(20, 0, P), cl(20, 1, P)], carried=("ender_pearl", 4)),
        clicks("a primary click with nothing carried selects none",
               [with_patch(st(20, "bundle", 1), three)],
               [cl(20, 1, S), cl(20, 0, P), cl(21, 0, P), cl(20, 1, P)]),
        clicks("thirteen stacks show eleven and the wheel wraps there",
               [with_patch(st(20, "bundle", 1), bundle([("stone", 1, empty)] * 13))],
               [cl(20, 1, S), cl(20, -1, S), cl(20, -1, S), cl(20, 1, P)]),
    ]

    def enchanted(slot, item, levels, kind=ENCHANTMENTS):
        return with_patch(st(slot, item, 1), patch([(kind, enchantments(levels))]))

    def worn(slot, item, damage):
        return with_patch(st(slot, item, 1), patch([(DAMAGE, varint(damage))]))

    damaged_stick = patch([(MAX_STACK_SIZE, varint(4)), (MAX_DAMAGE, varint(10)), (DAMAGE, varint(3))])
    click_rows += [
        clicks("grindstone strips all but curses",
               [enchanted(30, "diamond_sword", [(SHARPNESS, 3), (VANISHING_CURSE, 1)])],
               [cl(30, 0, P), cl(0, 0, P), cl(2, 0, P), cl(20, 0, P)], menu="grindstone"),
        clicks("grindstone merges two worn pickaxes",
               [worn(0, "iron_pickaxe", 100), worn(30, "iron_pickaxe", 200)],
               [cl(30, 0, P), cl(1, 0, P), cl(2, 0, Q)], menu="grindstone"),
        clicks("grindstone merges enchantments and keeps curses",
               [enchanted(0, "diamond_sword", [(BINDING_CURSE, 1), (SHARPNESS, 2)]),
                enchanted(30, "diamond_sword", [(VANISHING_CURSE, 1), (SHARPNESS, 5), (UNBREAKING, 3)])],
               [cl(30, 0, P), cl(1, 0, P)], menu="grindstone"),
        clicks("grindstone turns a book of no curse into a book",
               [enchanted(30, "enchanted_book", [(SHARPNESS, 2)], STORED_ENCHANTMENTS),
                enchanted(31, "enchanted_book", [(VANISHING_CURSE, 1), (MENDING, 1)], STORED_ENCHANTMENTS)],
               [cl(30, 0, P), cl(0, 0, P), cl(2, 0, Q), cl(31, 0, P), cl(1, 0, P), cl(1, 0, Q)],
               menu="grindstone"),
        clicks("grindstone refuses two books and two items of a kind",
               [enchanted(0, "enchanted_book", [(SHARPNESS, 2)], STORED_ENCHANTMENTS),
                enchanted(30, "enchanted_book", [(UNBREAKING, 1)], STORED_ENCHANTMENTS),
                enchanted(31, "iron_sword", [(SHARPNESS, 1)])],
               [cl(30, 0, P), cl(1, 0, P), cl(1, 0, P), cl(31, 0, P), cl(0, 0, SW)], menu="grindstone"),
        clicks("damaged stacks do not merge on a quick move",
               [with_patch(st(9, "stick", 1), damaged_stick), with_patch(st(36, "stick", 1), damaged_stick),
                with_patch(st(10, "stick", 1), patch([(MAX_STACK_SIZE, varint(4))])),
                with_patch(st(37, "stick", 1), patch([(MAX_STACK_SIZE, varint(4))]))],
               [cl(9, 0, Q), cl(10, 0, Q)]),
    ]

    yield {
        "case_id": "movement/container-clicks",
        "family": "movement",
        "operation": "container_clicks",
        "rows": click_rows,
    }

    def hashed(label, item, hex_patch):
        return {"label": label, "item": f"minecraft:{item}", "count": 1, "patch": hex_patch}

    yield {
        "case_id": "movement/component-hashes",
        "family": "movement",
        "operation": "component_hashes",
        "rows": [
            hashed("damage and repair cost", "iron_sword", patch([(DAMAGE, varint(12)), (REPAIR_COST, varint(3))])),
            hashed("max stack size and max damage", "stick", patch([(MAX_STACK_SIZE, varint(16)),
                                                                    (MAX_DAMAGE, varint(250))])),
            hashed("unbreakable", "iron_pickaxe", patch([(UNBREAKABLE, "")])),
            hashed("plain custom name", "stone", patch([(CUSTOM_NAME, nbt("Hello, Dünya"))])),
            hashed("styled item name", "stone", patch([(ITEM_NAME, nbt(
                {"text": "Bold", "bold": ("b", 1), "italic": ("b", 0), "color": "red"}))])),
            hashed("name with siblings", "stone", patch([(CUSTOM_NAME, nbt(
                {"text": "", "extra": ["a", {"text": "b", "underlined": ("b", 1)}]}))])),
            hashed("translated name", "stone", patch([(CUSTOM_NAME, nbt(
                {"translate": "item.minecraft.stone", "with": ["x"]}))])),
            hashed("lore lines", "paper", patch([(LORE, varint(2) + nbt("first")
                                                   + nbt({"text": "second", "italic": ("b", 0)}))])),
            hashed("rarity", "stick", patch([(RARITY, varint(3))])),
            hashed("enchantments", "diamond_sword", patch([(ENCHANTMENTS, enchantments(
                [(SHARPNESS, 5), (UNBREAKING, 3), (MENDING, 1)]))])),
            hashed("no enchantments on a book", "book", patch([(STORED_ENCHANTMENTS, enchantments([]))])),
            hashed("stored enchantments", "enchanted_book", patch([(STORED_ENCHANTMENTS, enchantments(
                [(PROTECTION, 4)]))])),
            hashed("glint override", "stick", patch([(GLINT, "00")])),
            hashed("dyed colour", "leather_helmet", patch([(DYED_COLOR, (0xA06540).to_bytes(4, "big").hex())])),
            hashed("custom data", "stick", patch([(CUSTOM_DATA, nbt({
                "flag": ("b", 1), "short": ("s", -2), "int": ("i", 70000), "long": ("l", -5),
                "float": ("f", 0.5), "double": ("d", -1.25), "text": "ş", "bytes": ("B", [1, -1]),
                "ints": ("I", [3, 4]), "longs": ("L", [5]), "list": [("i", 1), ("i", 2)],
                "nested": {"empty": []}}))])),
            hashed("bundle contents", "bundle", bundle([
                ("stone", 1, patch()),
                ("dirt", 5, patch([(CUSTOM_NAME, nbt("x"))], removed=[MAX_STACK_SIZE])),
                ("bundle", 1, bundle([("ender_pearl", 3, patch())]))])),
            hashed("added and removed together", "diamond_sword", patch(
                [(CUSTOM_NAME, nbt("Mixed")), (DAMAGE, varint(1))], removed=[MAX_STACK_SIZE])),
        ],
    }
