"""The `gui` family: screens."""

from cases.helpers import *


def gui_pause_screen_cases():
    """The pause menu in screens of several sizes, with the mouse away and
    over each of its buttons."""
    def row(label, width, height, mouse=(0, 0), steps=(), ms=0):
        return {"label": label, "width": width, "height": height, "mouse": list(mouse),
                "steps": list(steps), "ms": ms}

    rows = [row("default", 427, 240), row("small", 320, 240), row("wide", 640, 360), row("tiny", 200, 150)]
    for name, point in {"return": (213, 60), "advancements": (160, 90), "statistics": (265, 90),
                        "report": (170, 120), "feedback": (193, 120), "friends": (215, 120),
                        "reporting": (237, 120), "options": (213, 150), "disconnect": (213, 180)}.items():
        rows.append(row("hover-" + name, 427, 240, point))
    # Tooltips: shown once the widget has been hovered a moment (a frame
    # earlier), and not on the first frame of it.
    frame = lambda ms: {"do": "frame", "ms": ms}
    key = lambda name: {"do": "key", "key": name}
    icons = {"report": (170, 120), "feedback": (193, 120), "friends": (215, 120), "reporting": (237, 120)}
    for name, point in icons.items():
        rows.append(row("tip-" + name, 427, 240, point, [frame(0)], 10))
        rows.append(row("tip-first-frame-" + name, 427, 240, point, [], 0))
    rows += [
        row("tip-late", 427, 240, (170, 120), [frame(0), frame(500)], 1000),
        row("tip-small-screen", 320, 240, (150, 120), [frame(0)], 10),
        row("tip-tiny-screen", 200, 150, (100, 95), [frame(0)], 10),
        row("tip-wide-screen", 640, 360, (270, 175), [frame(0)], 10),
        row("tip-edge-left", 427, 240, (166, 112), [frame(0)], 10),
        row("tip-edge-bottom", 427, 240, (170, 138), [frame(0)], 10),
        row("tip-edge-top", 427, 240, (170, 101), [frame(0)], 10),
        row("tip-moves-between", 427, 240, (193, 120), [frame(0), frame(5)], 10),
        row("tip-not-on-button", 427, 240, (160, 90), [frame(0)], 10),
        row("tip-on-wide-button", 427, 240, (213, 60), [frame(0)], 10),
    ]
    # Keyboard focus: Tab and the arrows, then the tooltip below the widget.
    tab = key("tab")
    rows += [
        row("tab-1", 427, 240, steps=[tab]), row("tab-2", 427, 240, steps=[tab, tab]),
        row("tab-4-icon", 427, 240, steps=[tab] * 4, ms=0),
        row("tab-4-icon-tip", 427, 240, steps=[tab] * 4 + [frame(0)], ms=10),
        row("tab-5-icon-tip", 427, 240, steps=[tab] * 5 + [frame(0)], ms=10),
        row("tab-6-icon-tip", 427, 240, steps=[tab] * 6 + [frame(0)], ms=10),
        row("tab-7-icon-tip", 427, 240, steps=[tab] * 7 + [frame(0)], ms=10),
        row("tab-then-hover-tip", 427, 240, (237, 120), [tab] * 4 + [frame(0)], 10),
        row("tab-icon-small-tip", 320, 240, steps=[tab] * 4 + [frame(0)], ms=10),
        row("tab-icon-tiny-tip", 200, 150, steps=[tab] * 4 + [frame(0)], ms=10),
        row("shift-tab-tip", 427, 240, steps=[key("shift+tab")] * 4 + [frame(0)], ms=10),
        row("arrow-down", 427, 240, steps=[key("down")]),
        row("arrow-up", 427, 240, steps=[key("up")]),
        row("arrow-down-twice", 427, 240, steps=[key("down")] * 2),
        row("arrow-down-thrice", 427, 240, steps=[key("down")] * 3),
        row("arrow-down-icons-tip", 427, 240, steps=[key("down")] * 3 + [frame(0)], ms=10),
        row("arrow-right-from-first", 427, 240, steps=[key("down"), key("right")]),
        row("arrow-grid", 427, 240, steps=[key("down"), key("down"), key("right"), key("down")]),
        row("arrow-icons-across", 427, 240, steps=[key("down")] * 3 + [key("right"), key("right"), frame(0)], ms=10),
        row("arrow-icons-left", 427, 240, steps=[key("down")] * 3 + [key("right"), key("left"), frame(0)], ms=10),
        row("arrow-down-from-icons", 427, 240, steps=[key("down")] * 4),
        row("arrow-down-to-disconnect", 427, 240, steps=[key("down")] * 6),
        row("arrow-down-past", 427, 240, steps=[key("down")] * 8),
        row("arrow-up-from-disconnect", 427, 240, steps=[key("down")] * 6 + [key("up")]),
        row("arrow-up-to-return", 427, 240, steps=[key("down")] * 3 + [key("up")] * 3),
        row("arrow-left-from-nothing", 427, 240, steps=[key("left")]),
        row("arrow-right-from-nothing", 427, 240, steps=[key("right")]),
        row("arrow-small", 320, 240, steps=[key("down")] * 3 + [key("right")]),
        row("arrow-wide", 640, 360, steps=[key("down"), key("right"), key("down")]),
        row("arrow-tiny", 200, 150, steps=[key("down")] * 3 + [key("right"), key("down")]),
    ]
    yield {"case_id": "gui/pause-screen", "family": "gui", "operation": "pause_screen", "rows": rows}


def gui_death_screen_cases():
    """The death screen: before and after its buttons wake (20 ticks), with
    and without a cause, hardcore, scores of several widths, and the mouse
    over each button."""
    def row(label, width=427, height=240, mouse=(0, 0), ticks=0, hardcore=False, cause="Steve was slain by Zombie",
            score=0):
        described = {"label": label, "width": width, "height": height, "mouse": list(mouse), "ticks": ticks,
                     "hardcore": hardcore, "score": score}
        if cause is not None:
            described["cause"] = cause
        return described

    rows = [row("fresh"), row("awake", ticks=20), row("one-short", ticks=19), row("late", ticks=80),
            row("no-cause", ticks=20, cause=None), row("hardcore", ticks=20, hardcore=True),
            row("big-score", ticks=20, score=123456), row("small-screen", 320, 240, ticks=20),
            row("wide", 854, 480, ticks=20), row("long-cause", ticks=20, cause="Alex was blown up by Creeper"
                                                                              " whilst fighting Skeleton and more"),
            row("hover-respawn-asleep", mouse=(213, 132)), row("hover-respawn", mouse=(213, 132), ticks=20),
            row("hover-title", mouse=(213, 156), ticks=20), row("hover-elsewhere", mouse=(10, 10), ticks=20)]
    yield {"case_id": "gui/death-screen", "family": "gui", "operation": "death_screen", "rows": rows}


def gui_disconnected_screen_cases():
    """The disconnected screen for short and long reasons (wrapped at the
    screen's width less 50), in several screen sizes, and with the mouse over
    the button."""
    def row(label, reason, width=427, height=240, mouse=(0, 0)):
        return {"label": label, "width": width, "height": height, "mouse": list(mouse), "reason": reason}

    long_reason = "You are not whitelisted on this server and the reason is long enough to wrap " * 3
    rows = [row("short", "Disconnected"), row("kicked", "Kicked for flying"), row("long", long_reason),
            row("small", "Timed out", 320, 240), row("wide", long_reason, 854, 480),
            row("tiny", long_reason, 200, 150), row("empty", ""), row("lines", "one\ntwo\nthree"),
            row("hover", "Disconnected", mouse=(213, 140))]
    yield {"case_id": "gui/disconnected-screen", "family": "gui", "operation": "disconnected_screen", "rows": rows}


def trader(level, xp, progress=True, restock=True):
    return {"level": level, "xp": xp, "progress": progress, "restock": restock}


def trades():
    """The offers of the trading screens: a plain one, one the demand has raised, one that asks
    for two items, and one that is used up."""
    def offer(buy, sell, buy_b=None, uses=0, max_uses=12, xp=2, special=0, demand=0, multiplier=0.05):
        made = {"buy": [f"minecraft:{buy[0]}", buy[1]], "sell": [f"minecraft:{sell[0]}", sell[1]],
                "uses": uses, "max_uses": max_uses, "xp": xp, "special": special, "demand": demand,
                "multiplier": multiplier}
        if buy_b:
            made["buy_b"] = [f"minecraft:{buy_b[0]}", buy_b[1]]
        return made
    return [offer(("emerald", 1), ("bread", 6)),
            offer(("wheat", 20), ("emerald", 1), max_uses=16, demand=4),
            offer(("emerald", 3), ("bookshelf", 1), buy_b=("paper", 2), special=-1),
            offer(("diamond", 2), ("cake", 1), uses=12)]


def gui_container_screen_cases():
    """The screens of the menus that change their items, over the reference's
    own menu: labels, textures, sprites, name box and what each shows of its
    data. Slots hold items, which are not recorded."""
    def screen(label, menu, title_key, slots=(), data=None, xp=None, keys=(), sync_slot=None,
               mouse=(0, 0), width=427, height=240, stonecutter=(), data_after=None, tooltips=True, game_mode="survival", books=None, offers=None,
               merchant=None, patches=None, cooldowns=None, cooldown_ticks=0):
        row = {"label": label, "width": width, "height": height, "mouse_x": mouse[0],
               "mouse_y": mouse[1], "menu": f"minecraft:{menu}", "title_key": title_key,
               "container_id": 4, "game_mode": game_mode, "selected": 0, "state_id": 0,
               "blocks": [], "chunk_radius": 0, "clicks": [], "keys": list(keys),
               "slots": [dict({"slot": slot, "item": f"minecraft:{item}", "count": count},
                              **({"book_pages": books[slot]} if books and slot in books else {}),
                              **({"patch": patches[slot]} if patches and slot in patches else {}))
                         for slot, item, count in slots]}
        if data is not None:
            row["data"] = data
        if xp is not None:
            row["xp_level"] = xp
        if sync_slot is not None:
            row["sync_slot"] = sync_slot
        if data_after is not None:
            row["data_after"] = data_after
        if offers is not None:
            row["offers"] = offers
        if merchant is not None:
            row["merchant"] = merchant
        if cooldowns is not None:
            row["cooldowns"] = [{"group": group, "duration": duration} for group, duration in cooldowns]
            row["cooldown_ticks"] = cooldown_ticks
        if not tooltips:
            row["tooltips"] = False
        if stonecutter:
            row["stonecutter_inputs"] = list(stonecutter)
            # Its entries show no item the reference can name here.
            row["tooltips"] = False
        return row

    rows = [
        screen("anvil-empty", "anvil", "container.repair"),
        screen("anvil-item", "anvil", "container.repair", slots=[(0, "iron_sword", 1)], sync_slot=0),
        screen("anvil-no-result", "anvil", "container.repair", slots=[(0, "iron_sword", 1)],
               data=[0], sync_slot=0),
        screen("grindstone-empty", "grindstone", "container.grindstone_title"),
        screen("grindstone-error", "grindstone", "container.grindstone_title",
               slots=[(0, "iron_sword", 1)]),
        screen("stonecutter-empty", "stonecutter", "container.stonecutter"),
        screen("stonecutter-entries", "stonecutter", "container.stonecutter",
               slots=[(0, "iron_ingot", 1)], stonecutter=["minecraft:iron_ingot"] * 5, data=[2],
               sync_slot=0),
        screen("stonecutter-hover", "stonecutter", "container.stonecutter",
               slots=[(0, "iron_ingot", 1)], stonecutter=["minecraft:iron_ingot"] * 5, data=[-1],
               mouse=(198, 56), sync_slot=0),
        screen("stonecutter-scrollable", "stonecutter", "container.stonecutter",
               slots=[(0, "iron_ingot", 1)], stonecutter=["minecraft:iron_ingot"] * 14, data=[-1],
               sync_slot=0),
        screen("loom-empty", "loom", "container.loom"),
        screen("cartography-empty", "cartography_table", "container.cartography_table"),
        screen("cartography-paper", "cartography_table", "container.cartography_table",
               slots=[(1, "paper", 1)], sync_slot=1),
        screen("cartography-glass", "cartography_table", "container.cartography_table",
               slots=[(1, "glass_pane", 1)], sync_slot=1),
        screen("brewing-empty", "brewing_stand", "container.brewing"),
        screen("brewing-fuel", "brewing_stand", "container.brewing", data=[0, 20]),
        screen("brewing-partial-fuel", "brewing_stand", "container.brewing", data=[0, 7]),
        screen("brewing-start", "brewing_stand", "container.brewing", data=[399, 20]),
        screen("brewing-half", "brewing_stand", "container.brewing", data=[201, 3]),
        screen("brewing-end", "brewing_stand", "container.brewing", data=[1, 0]),
        screen("brewing-items", "brewing_stand", "container.brewing",
               slots=[(0, "potion", 1), (3, "nether_wart", 2)], data=[120, 11], sync_slot=3),
        screen("beacon-empty", "beacon", "container.beacon", data=[0, 0, 0]),
        screen("beacon-level-one", "beacon", "container.beacon", data=[1, 0, 0]),
        screen("beacon-level-three", "beacon", "container.beacon", data=[3, 0, 0]),
        screen("beacon-level-four", "beacon", "container.beacon", data=[4, 0, 0]),
        screen("beacon-primary", "beacon", "container.beacon", data=[4, 0, 0], data_after=[[1, 1]]),
        screen("beacon-secondary", "beacon", "container.beacon", data=[4, 0, 0],
               data_after=[[1, 11], [2, 10]]),
        screen("beacon-upgrade", "beacon", "container.beacon", data=[4, 0, 0],
               data_after=[[1, 10], [2, 10]]),
        screen("beacon-payment", "beacon", "container.beacon", slots=[(0, "emerald", 1)],
               data=[4, 0, 0], data_after=[[1, 1]], sync_slot=0),
        screen("beacon-hover-power", "beacon", "container.beacon", data=[4, 0, 0], mouse=(160, 40), tooltips=False),
        screen("beacon-hover-confirm", "beacon", "container.beacon", slots=[(0, "emerald", 1)],
               data=[4, 0, 0], data_after=[[1, 1]], sync_slot=0, mouse=(270, 125), tooltips=False),
        screen("crafter-empty", "crafter_3x3", "container.crafter", data=[0] * 10),
        screen("crafter-disabled", "crafter_3x3", "container.crafter",
               data=[0, 1, 0, 0, 1, 0, 1, 0, 0, 0]),
        screen("crafter-powered", "crafter_3x3", "container.crafter", data=[1, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        screen("crafter-items", "crafter_3x3", "container.crafter",
               slots=[(0, "stone", 3), (1, "stick", 1), (4, "dirt", 2)], data=[0, 1, 0, 0, 1, 0, 0, 0, 0, 0],
               sync_slot=0),
        screen("crafter-hover-slot", "crafter_3x3", "container.crafter", data=[0] * 10,
               mouse=(155, 58), tooltips=False),
        screen("crafter-hover-disabled", "crafter_3x3", "container.crafter",
               data=[1, 0, 0, 0, 0, 0, 0, 0, 0, 0], mouse=(155, 58), tooltips=False),
        screen("lectern-empty", "lectern", "container.lectern"),
        screen("lectern-book", "lectern", "container.lectern", slots=[(0, "written_book", 1)],
               sync_slot=0, books={0: ["Hello, lectern.", "A second page", "The third"]}),
        screen("lectern-middle-page", "lectern", "container.lectern", slots=[(0, "written_book", 1)],
               sync_slot=0, data_after=[[0, 1]],
               books={0: ["Hello, lectern.", "A second page", "The third"]}),
        screen("lectern-last-page", "lectern", "container.lectern", slots=[(0, "written_book", 1)],
               sync_slot=0, data_after=[[0, 2]],
               books={0: ["Hello, lectern.", "A second page", "The third"]}),
        screen("lectern-long-page", "lectern", "container.lectern", slots=[(0, "written_book", 1)],
               sync_slot=0,
               books={0: ["The quick brown fox jumps over the lazy dog, again and again, until the "
                          "page is full of words that wrap at the width of the book's text column "
                          "and run on and on and on and on and on and on and on and on and on and "
                          "on and on and on and on and on and on and on and on and on and on and on "
                          "and on and on and on and on and on and on and on and on and on and on"]}),
        screen("lectern-hover-forward", "lectern", "container.lectern", slots=[(0, "written_book", 1)],
               sync_slot=0, mouse=(240, 165), books={0: ["One", "Two"]}),
        screen("lectern-hover-done", "lectern", "container.lectern", mouse=(150, 205)),
        screen("lectern-adventure", "lectern", "container.lectern", slots=[(0, "written_book", 1)],
               sync_slot=0, game_mode="adventure", books={0: ["One", "Two"]}),
        screen("merchant-empty", "merchant", "entity.minecraft.villager", offers=[], keys=["frame"],
               merchant=trader(1, 0)),
        screen("merchant-offers", "merchant", "entity.minecraft.villager", offers=trades(),
               keys=["frame"], merchant=trader(2, 15)),
        screen("merchant-scroll-bar", "merchant", "entity.minecraft.villager", offers=trades() * 3,
               keys=["frame"], merchant=trader(3, 100)),
        screen("merchant-payment", "merchant", "entity.minecraft.villager", offers=trades(),
               slots=[(0, "emerald", 1)], keys=["frame"], merchant=trader(2, 15)),
        screen("merchant-priced", "merchant", "entity.minecraft.villager", offers=trades(),
               slots=[(0, "wheat", 30)], keys=["frame"], merchant=trader(2, 40)),
        screen("merchant-out-of-stock", "merchant", "entity.minecraft.villager",
               offers=trades()[3:] + trades()[:3], keys=["frame"], merchant=trader(4, 180)),
        screen("merchant-last-level", "merchant", "entity.minecraft.villager", offers=trades(),
               keys=["frame"], merchant=trader(5, 400)),
        screen("merchant-no-bar", "merchant", "entity.minecraft.villager", offers=trades(),
               keys=["frame"], merchant=trader(2, 15, progress=False)),
        screen("merchant-hover-trade", "merchant", "entity.minecraft.villager", offers=trades(),
               keys=["frame"], mouse=(160, 70), tooltips=False, merchant=trader(2, 15)),
        screen("smithing-empty", "smithing", "container.upgrade"),
        screen("smithing-template", "smithing", "container.upgrade",
               slots=[(0, "netherite_upgrade_smithing_template", 1)]),
        screen("smithing-trim", "smithing", "container.upgrade",
               slots=[(0, "coast_armor_trim_smithing_template", 1), (1, "iron_chestplate", 1)]),
        screen("enchanting-empty", "enchantment", "container.enchant"),
        screen("enchanting-rows", "enchantment", "container.enchant",
               slots=[(0, "diamond_sword", 1), (1, "lapis_lazuli", 3)], xp=30,
               data=[10, 20, 30, 12345, -1, -1, -1, -1, -1, -1]),
        screen("enchanting-poor", "enchantment", "container.enchant",
               slots=[(0, "diamond_sword", 1), (1, "lapis_lazuli", 1)], xp=15,
               data=[10, 20, 30, 777, -1, -1, -1, -1, -1, -1]),
        screen("enchanting-hover", "enchantment", "container.enchant",
               slots=[(0, "diamond_sword", 1), (1, "lapis_lazuli", 3)], xp=30,
               data=[10, 20, 30, 12345, -1, -1, -1, -1, -1, -1], mouse=(195, 56)),
        screen("narrow", "anvil", "container.repair", width=320, height=240),
        screen("tall", "grindstone", "container.grindstone_title", width=640, height=480),
    ]

    # `itemDecorations`' bar: durability, and a bundle's fullness.
    def varint(value):
        out = bytearray()
        while value & ~0x7F:
            out.append((value & 0x7F) | 0x80)
            value >>= 7
        out.append(value)
        return out.hex()

    def damaged(value):
        return "0100" + "03" + varint(value)

    def bundle(*items):
        # (item registry id, count) per stack, each with no patch.
        value = varint(len(items)) + "".join(varint(item) + varint(count) + "0000" for item, count in items)
        return "0100" + "32" + value

    worn = [(3, 0), (4, 1), (5, 30), (6, 125), (7, 200), (8, 249), (9, 250)]
    rows += [
        screen("decorations-durability", "grindstone", "container.grindstone_title",
               slots=[(slot, "iron_sword", 1) for slot, _ in worn] + [(10, "diamond_pickaxe", 1),
                                                                     (11, "elytra", 1)],
               patches=dict([(slot, damaged(value)) for slot, value in worn]
                            + [(10, damaged(1000)), (11, damaged(431))]), tooltips=False),
        screen("decorations-bundle", "grindstone", "container.grindstone_title",
               slots=[(12, "bundle", 1), (13, "bundle", 1), (14, "bundle", 1), (15, "bundle", 1),
                      (16, "bundle", 1)],
               patches={13: bundle((1, 32)), 14: bundle((1, 64)), 15: bundle((1144, 3)),
                        16: bundle((1, 63))}, tooltips=False),
    ]

    def use_cooldown(group):
        # `UseCooldown.STREAM_CODEC`: the seconds (a float), then the group if any.
        encoded = group.encode()
        return "0100" + "1a" + "3f800000" + "01" + varint(len(encoded)) + encoded.hex()

    for ticks in (0, 5, 7, 19, 20):
        rows.append(screen(f"decorations-cooldown-{ticks}", "grindstone", "container.grindstone_title",
                           slots=[(20, "ender_pearl", 16), (21, "ender_pearl", 1), (22, "chorus_fruit", 3),
                                  (23, "stick", 1), (24, "stick", 1), (25, "wind_charge", 2)],
                           patches={23: use_cooldown("minecraft:charge"), 24: use_cooldown("pearls")},
                           cooldowns=[("minecraft:ender_pearl", 20), ("minecraft:chorus_fruit", 30),
                                      ("minecraft:charge", 13), ("minecraft:pearls", 7),
                                      ("minecraft:wind_charge", 3)],
                           cooldown_ticks=ticks, tooltips=False))
    def mount_screen(label, entity, title_key, columns=0, tame=True, baby=False, slots=(), mouse=(0, 0)):
        row = screen(label, "mount", title_key, slots=slots, mouse=mouse)
        row["mount"] = {"entity": f"minecraft:{entity}", "columns": columns, "baby": baby, "tame": tame}
        return row

    # `HorseInventoryScreen`, `NautilusInventoryScreen`: the texture, the chest's slots sprite, the
    # saddle and armour frames where the entity wears them, the empty icons and the title. The
    # mount's model is not recorded.
    rows += [
        mount_screen("mount-horse", "horse", "entity.minecraft.horse"),
        mount_screen("mount-horse-saddled", "horse", "entity.minecraft.horse",
                     slots=[(0, "saddle", 1), (1, "iron_horse_armor", 1)], mouse=(220, 100)),
        mount_screen("mount-donkey-chest", "donkey", "entity.minecraft.donkey", columns=5,
                     slots=[(2, "stone", 3), (9, "bread", 2)]),
        mount_screen("mount-donkey-untamed", "donkey", "entity.minecraft.donkey", tame=False),
        mount_screen("mount-donkey-foal", "donkey", "entity.minecraft.donkey", baby=True),
        mount_screen("mount-mule-chest", "mule", "entity.minecraft.mule", columns=3),
        mount_screen("mount-llama", "llama", "entity.minecraft.llama", columns=2, tame=False),
        mount_screen("mount-camel", "camel", "entity.minecraft.camel"),
        mount_screen("mount-skeleton-horse", "skeleton_horse", "entity.minecraft.skeleton_horse"),
        mount_screen("mount-nautilus", "nautilus", "entity.minecraft.nautilus"),
        mount_screen("mount-nautilus-untamed", "nautilus", "entity.minecraft.nautilus", tame=False),
        mount_screen("mount-zombie-nautilus", "zombie_nautilus", "entity.minecraft.zombie_nautilus"),
    ]
    yield {"case_id": "gui/container-screen", "family": "gui", "operation": "container_screen",
           "rows": rows}


def gui_confirm_screen_cases():
    """The confirmation screen (and the death screen's own, over its red
    gradient): the message wrapped and clipped to fifteen lines, the delay
    that keeps the buttons asleep and Escape shut, Tab, Enter and clicks, and
    what the callback was told."""
    def row(label, width=427, height=240, mouse=(0, 0), death=False, title="Are you sure?",
            message="This cannot be undone.", yes="Yes", no="No", delay=0, ticks=0, keys=(), click=False,
            in_world=False, link=None, pack=None):
        row = {"label": label, "width": width, "height": height, "mouse": list(mouse), "death": death,
               "title": title, "message": message, "yes": yes, "no": no, "delay": delay, "ticks": ticks,
               "keys": list(keys), "click": click, "in_world": in_world}
        if link:
            row["link"] = link
        if pack:
            row["pack"] = pack
        return row

    long_message = ("This message is long enough that it has to wrap onto several lines of the screen "
                    "before the buttons, and it keeps going so that the third line is needed too.")
    lines = lambda count: "\n".join(f"line {n}" for n in range(count))
    quit_row = dict(death=True, title="Are you sure you want to quit?", message="", yes="Title Screen",
                    no="Respawn", delay=20)
    rows = [
        row("plain"), row("plain-in-world", in_world=True), row("empty-message", message="", mouse=(0, 0)),
        row("wrapped", message=long_message), row("wrapped-narrow", 320, 240, message=long_message),
        row("wrapped-wide", 854, 480, message=long_message), row("newlines", message="one\ntwo\n\nfour"),
        row("fifteen-lines", message=lines(15)), row("sixteen-lines", message=lines(16)),
        row("many-lines", message=lines(40)), row("custom-buttons", yes="Open", no="Cancel"),
        row("hover-yes", mouse=(100, 145)), row("hover-no", mouse=(300, 145)),
        row("hover-gap", mouse=(213, 145)), row("hover-elsewhere", mouse=(10, 10)),
        row("quit", **quit_row), row("quit-awake", ticks=20, **quit_row),
        row("quit-hover-asleep", mouse=(100, 140), **quit_row),
        row("quit-hover-awake", mouse=(100, 140), ticks=20, **quit_row),
        row("quit-small", 320, 240, **quit_row),
        row("delay-one-short", delay=20, ticks=19), row("delay-done", delay=20, ticks=20),
        row("delay-long", delay=20, ticks=80),
        row("tab", keys=["tab"]), row("tab-twice", keys=["tab", "tab"]), row("tab-thrice", keys=["tab"] * 3),
        row("shift-tab", keys=["shift+tab"]), row("shift-tab-twice", keys=["shift+tab", "shift+tab"]),
        row("tab-asleep", delay=20, ticks=5, keys=["tab"]),
        row("tab-awake", delay=20, ticks=20, keys=["tab"]),
        row("enter-unfocused", keys=["enter"]), row("enter-yes", keys=["tab", "enter"]),
        row("space-no", keys=["tab", "tab", "space"]),
        row("enter-asleep", delay=20, ticks=5, keys=["tab", "enter"]),
        row("escape", keys=["escape"]), row("escape-asleep", delay=20, ticks=5, keys=["escape"]),
        row("escape-last-tick", delay=20, ticks=19, keys=["escape"]),
        row("escape-awake", delay=20, ticks=20, keys=["escape"]),
        row("escape-quit", ticks=25, keys=["escape"], **quit_row),
        row("click-yes", mouse=(100, 145), click=True), row("click-no", mouse=(300, 145), click=True),
        row("click-gap", mouse=(213, 145), click=True),
        row("click-asleep", mouse=(100, 145), delay=20, ticks=5, click=True),
        row("click-awake", mouse=(300, 145), delay=20, ticks=20, click=True),
    ]
    # `ConfirmLinkScreen`: three buttons of 100 and, for a link that is not
    # trusted, a warning. Copy to Clipboard is not pressed (it needs the keyboard).
    url = "https://example.com/a/path?query=1"
    untrusted = {"url": url, "trusted": False}
    trusted = {"url": url, "trusted": True}
    long_url = "https://example.com/" + "segment/" * 12 + "end"
    rows += [
        row("link", link=untrusted), row("link-trusted", link=trusted),
        row("link-in-world", link=untrusted, in_world=True), row("link-small", 320, 240, link=untrusted),
        row("link-wide", 854, 480, link=untrusted), row("link-tiny", 200, 150, link=untrusted),
        row("link-long-url", link={"url": long_url, "trusted": False}),
        row("link-long-url-trusted", link={"url": long_url, "trusted": True}),
        row("link-empty-url", link={"url": "", "trusted": False}),
        row("link-hover-yes", mouse=(110, 150), link=untrusted), row("link-hover-copy", mouse=(213, 150), link=untrusted),
        row("link-hover-no", mouse=(320, 150), link=untrusted), row("link-hover-trusted", mouse=(110, 145), link=trusted),
        row("link-tab", keys=["tab"], link=untrusted), row("link-tab-2", keys=["tab"] * 2, link=untrusted),
        row("link-tab-3", keys=["tab"] * 3, link=untrusted), row("link-tab-4-round", keys=["tab"] * 4, link=untrusted),
        row("link-shift-tab", keys=["shift+tab"], link=untrusted),
        row("link-enter-yes", keys=["tab", "enter"], link=untrusted),
        row("link-enter-no", keys=["tab", "tab", "tab", "enter"], link=untrusted),
        row("link-escape", keys=["escape"], link=untrusted),
        row("link-click-yes", mouse=(110, 150), click=True, link=untrusted),
        row("link-click-no", mouse=(320, 150), click=True, link=untrusted),
        row("link-click-gap", mouse=(161, 150), click=True, link=untrusted),
    ]
    # The server resource pack question: wording of an optional and a required pack, with and
    # without the server's own prompt, and the answers' keys.
    optional, required = {"required": False}, {"required": True}
    with_prompt = lambda base: {**base, "prompt": "Get the server textures to see custom items."}
    rows += [
        row("pack", pack=optional), row("pack-prompt", pack=with_prompt(optional)),
        row("pack-required", pack=required), row("pack-required-prompt", pack=with_prompt(required)),
        row("pack-required-small", 320, 240, pack=with_prompt(required)),
        row("pack-in-world", in_world=True, pack=with_prompt(optional)),
        row("pack-hover-no", mouse=(300, 145), pack=required),
        row("pack-click-yes", mouse=(100, 145), click=True, pack=required),
        row("pack-click-no", mouse=(300, 145), click=True, pack=optional),
    ]
    yield {"case_id": "gui/confirm-screen", "family": "gui", "operation": "confirm_screen", "rows": rows}


def gui_widgets_cases():
    """The widgets an option makes: the slider of a range (field of view), of
    a share (sensitivity, chat opacity), and the cycle button of a boolean and
    of a choice. Each is set to a value, focused, clicked, dragged, stepped
    with the keys or turned with the wheel; the drawing and the option's value
    are what is compared."""
    def row(label, kind, value, width=150, active=True, mouse=(0, 0), steps=()):
        return {"label": label, "kind": kind, "value": value, "width": width, "active": active,
                "mouse": list(mouse), "steps": list(steps)}

    def focus(input="MOUSE"):
        return {"do": "focus", "input": input}

    def click(x, shift=False):
        return {"do": "click", "x": x, "y": 60, "shift": shift}

    def key(name, shift=False):
        return {"do": "key", "key": name, "shift": shift}

    drag = lambda x: {"do": "drag", "x": x}
    release = lambda x: {"do": "release", "x": x}
    scroll = lambda dy: {"do": "scroll", "dy": dy}
    rows = [
        row("fov-normal", "fov", 70), row("fov-quake-pro", "fov", 110), row("fov-lowest", "fov", 30),
        row("fov-90", "fov", 90), row("fov-wide-bar", "fov", 90, width=200),
        row("fov-big-bar", "fov", 45, width=310), row("fov-inactive", "fov", 90, active=False),
        row("fov-hover-bar", "fov", 90, mouse=(120, 55)), row("fov-hover-handle", "fov", 90, mouse=(144, 55)),
        row("fov-click", "fov", 70, steps=[click(175)]),
        row("fov-click-release", "fov", 70, steps=[click(175), release(175)]),
        row("fov-click-left-edge", "fov", 70, steps=[click(100), release(100)]),
        row("fov-click-right-edge", "fov", 70, steps=[click(249), release(249)]),
        row("fov-click-outside", "fov", 70, steps=[click(300)]),
        row("fov-click-inactive", "fov", 70, active=False, steps=[click(175)]),
        row("fov-drag", "fov", 70, steps=[click(120), drag(160), drag(200)]),
        row("fov-drag-release", "fov", 70, steps=[click(120), drag(160), drag(200), release(200)]),
        row("fov-drag-past-end", "fov", 70, steps=[click(120), drag(900), release(900)]),
        row("fov-drag-before-start", "fov", 90, steps=[click(120), drag(-50), release(-50)]),
        row("fov-focus-mouse", "fov", 70, steps=[focus("MOUSE")]),
        row("fov-focus-tab", "fov", 70, steps=[focus("KEYBOARD_TAB")]),
        row("fov-focus-arrow", "fov", 70, steps=[focus("KEYBOARD_ARROW")]),
        row("fov-focus-none", "fov", 70, steps=[focus("NONE")]),
        row("fov-keys-right", "fov", 70, steps=[focus(), key("right"), key("right"), key("right")]),
        row("fov-keys-left", "fov", 70, steps=[focus(), key("left"), key("left")]),
        row("fov-keys-at-end", "fov", 110, steps=[focus(), key("right")]),
        row("fov-keys-at-start", "fov", 30, steps=[focus(), key("left")]),
        row("fov-keys-locked", "fov", 70, steps=[focus("KEYBOARD_ARROW"), key("right")]),
        row("fov-enter-unlocks", "fov", 70, steps=[focus("KEYBOARD_ARROW"), key("enter"), key("right")]),
        row("fov-space-toggles", "fov", 70, steps=[focus(), key("space"), key("right")]),
        row("fov-unfocus", "fov", 70, steps=[focus(), {"do": "unfocus"}]),
        row("sensitivity-half", "sensitivity", 0.5), row("sensitivity-yawn", "sensitivity", 0.0),
        row("sensitivity-hyperspeed", "sensitivity", 1.0), row("sensitivity-third", "sensitivity", 0.33),
        row("sensitivity-click", "sensitivity", 0.5, steps=[click(120), release(120)]),
        row("sensitivity-drag", "sensitivity", 0.5, steps=[click(120), drag(130)]),
        row("sensitivity-drag-to-ends", "sensitivity", 0.5, steps=[click(120), drag(-100), release(-100)]),
        row("sensitivity-keys", "sensitivity", 0.5, steps=[focus(), key("right"), key("right"), key("left")]),
        row("sensitivity-keys-at-end", "sensitivity", 1.0, steps=[focus(), key("right")]),
        row("sensitivity-wide-keys", "sensitivity", 0.5, width=310, steps=[focus(), key("right")]),
        row("chat-opacity-full", "chat_opacity", 1.0), row("chat-opacity-none", "chat_opacity", 0.0),
        row("chat-opacity-third", "chat_opacity", 0.33),
        row("chat-opacity-click", "chat_opacity", 1.0, steps=[click(130), release(130)]),
        row("invert-off", "invert_y", False), row("invert-on", "invert_y", True),
        row("invert-inactive", "invert_y", True, active=False),
        row("invert-hover", "invert_y", False, mouse=(150, 55)),
        row("invert-click", "invert_y", False, steps=[click(150)]),
        row("invert-click-twice", "invert_y", False, steps=[click(150), click(150)]),
        row("invert-shift-click", "invert_y", False, steps=[click(150, True)]),
        row("invert-click-outside", "invert_y", False, steps=[click(300)]),
        row("invert-click-inactive", "invert_y", False, active=False, steps=[click(150)]),
        row("invert-scroll-up", "invert_y", False, steps=[scroll(1.0)]),
        row("invert-scroll-down", "invert_y", False, steps=[scroll(-1.0)]),
        row("invert-scroll-none", "invert_y", False, steps=[scroll(0.0)]),
        row("invert-enter-unfocused", "invert_y", False, steps=[key("enter")]),
        row("invert-enter-focused", "invert_y", False, steps=[focus(), key("enter")]),
        row("invert-space-shift", "invert_y", False, steps=[focus(), key("space", True)]),
        row("invert-focused", "invert_y", False, steps=[focus()]),
        row("invert-wide", "invert_y", False, width=310),
        row("visibility-shown", "chat_visibility", 0), row("visibility-commands", "chat_visibility", 1),
        row("visibility-hidden", "chat_visibility", 2),
        row("visibility-click", "chat_visibility", 0, steps=[click(150)]),
        row("visibility-click-wraps", "chat_visibility", 2, steps=[click(150)]),
        row("visibility-shift-click-wraps", "chat_visibility", 0, steps=[click(150, True)]),
        row("visibility-scroll", "chat_visibility", 1, steps=[scroll(1.0), scroll(1.0)]),
        row("visibility-scroll-down", "chat_visibility", 1, steps=[scroll(-1.0), scroll(-1.0)]),
    ]
    yield {"case_id": "gui/widgets", "family": "gui", "operation": "widgets", "rows": rows}


def gui_game_mode_switcher_cases():
    """`GameModeSwitcherScreen` (F3+F4) over the game's own client: where it starts, the switch key,
    and the mouse picking a slot once it has moved."""
    def row(label, game_mode, steps, previous=None, width=427, height=240):
        made = {"label": label, "width": width, "height": height, "game_mode": game_mode,
                "selected": 0, "state_id": 0, "blocks": [], "chunk_radius": 0, "clicks": [],
                "slots": [], "steps": [{"mouse": list(mouse), **({"key": True} if key else {})}
                                        for mouse, key in steps]}
        if previous is not None:
            made["previous"] = previous
        return made

    away = (10, 10)
    rows = [
        row("from-creative", "creative", [(away, False)]),
        row("from-survival", "survival", [(away, False)]),
        row("previous-spectator", "survival", [(away, False)], previous="spectator"),
        row("previous-adventure", "creative", [(away, False)], previous="adventure"),
        row("switch-key-cycles", "survival", [(away, False), (away, True), (away, True), (away, True),
                                              (away, True)]),
        # The slots lie at x 154, 185, 216 and 247, 26 wide, from y 89.
        row("mouse-picks-once-moved", "survival", [((170, 100), False), ((170, 100), False),
                                                   ((200, 100), False), ((260, 100), False),
                                                   (away, False)]),
        row("mouse-waits-for-motion", "creative", [((230, 100), False), ((230, 100), False),
                                                   ((231, 100), False)]),
        row("switch-key-then-mouse", "creative", [((200, 100), False), ((200, 100), True),
                                                   ((200, 100), False), ((170, 95), False)]),
        row("small-window", "survival", [(away, False), ((20, 30), True)], width=200, height=120),
    ]
    yield {"case_id": "gui/game-mode-switcher", "family": "gui", "operation": "game_mode_switcher",
           "rows": rows}
