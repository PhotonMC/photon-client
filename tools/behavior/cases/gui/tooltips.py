"""The `gui` family: item tooltip lines (`ItemStack.getTooltipLines`)."""

from cases.movement.player.containers import CUSTOM_NAME, DAMAGE, ITEM_IDS, LORE, bundle, nbt, patch, varint

MAP_ID, MAP_POST_PROCESSING, WRITTEN_BOOK_CONTENT, BANNER_PATTERNS, CONTAINER = 46, 48, 55, 72, 75
LOCK, SCALE = 0, 1


def text(value):
    """`ByteBufCodecs.STRING_UTF8`."""
    encoded = value.encode("utf-8")
    return varint(len(encoded)) + encoded.hex()


def container(items):
    """`ItemContainerContents`: an optional stack per slot, (item, count, patch hex) or None."""
    return varint(len(items)) + "".join(
        "00" if item is None else "01" + varint(ITEM_IDS[item[0]]) + varint(item[1]) + item[2]
        for item in items)


def book(author, generation):
    """`WrittenBookContent`: a title with no filtered form, no pages, resolved."""
    return text("Title") + "00" + text(author) + varint(generation) + varint(0) + "01"


def gui_tooltip_cases():
    """Tooltip lines of stacks whose items or components add lines: maps with and without the
    client's map data and post-processing, container contents up to the cut-off, written books,
    smithing templates, the advanced (F3+H) lines, and the order of patterns against lore."""
    def row(label, item, added=(), count=1, creative=False, advanced=False, map_data=None):
        made = {"label": label, "item": f"minecraft:{item}", "count": count, "creative": creative,
                "advanced": advanced}
        if added:
            made["patch"] = patch(added)
        if map_data is not None:
            made["map"] = {"scale": map_data[0], "locked": map_data[1]}
        return made

    named = patch([(CUSTOM_NAME, nbt("Rock"))])
    map_id = (MAP_ID, varint(3))
    rows = [
        row("plain", "stone"),
        row("plain advanced", "stone", advanced=True),
        row("named advanced", "stone", [(CUSTOM_NAME, nbt("Rock"))], advanced=True),
        row("damaged", "iron_sword", [(DAMAGE, varint(10))]),
        row("damaged advanced", "iron_sword", [(DAMAGE, varint(10))], advanced=True),
        row("map unknown", "filled_map", [map_id]),
        row("map", "filled_map", [map_id], map_data=(2, False)),
        row("map locked advanced", "filled_map", [map_id], advanced=True, map_data=(4, True)),
        row("map scaling advanced", "filled_map", [map_id, (MAP_POST_PROCESSING, varint(SCALE))],
            advanced=True, map_data=(1, False)),
        row("map locking named", "filled_map",
            [map_id, (MAP_POST_PROCESSING, varint(LOCK)), (CUSTOM_NAME, nbt("Atlas"))], map_data=(0, False)),
        row("shulker contents", "shulker_box",
            [(CONTAINER, container([("stone", 10, patch()), None, ("dirt", 3, patch()), ("stone", 1, named)]))]),
        row("shulker many", "shulker_box",
            [(CONTAINER, container([("stone", n, patch()) for n in range(1, 8)]))]),
        row("shulker advanced", "shulker_box",
            [(CONTAINER, container([("ender_pearl", 16, patch())]))], advanced=True),
        row("written book", "written_book", [(WRITTEN_BOOK_CONTENT, book("Alex", 1))]),
        row("written book no author", "written_book", [(WRITTEN_BOOK_CONTENT, book(" ", 3))]),
        row("netherite template", "netherite_upgrade_smithing_template"),
        row("trim template advanced", "sentry_armor_trim_smithing_template", advanced=True),
        row("banner and lore", "white_banner",
            [(BANNER_PATTERNS, varint(1) + varint(1) + varint(14)), (LORE, varint(1) + nbt("Note"))]),
    ]
    yield {"case_id": "gui/tooltip-lines", "family": "gui", "operation": "tooltip_lines", "rows": rows}


def gui_bundle_tooltip_cases():
    """A bundle's tooltip in a container screen: empty, some stacks with and without a selected
    one (a named one, a damaged one), full, more than twelve stacks, and placed by the screen's
    edges."""
    empty = patch()

    def row(label, contents=None, selected=None, mouse=(120, 100), width=427, height=240):
        made = {"label": label, "item": "minecraft:bundle", "count": 1, "width": width, "height": height,
                "mouse_x": mouse[0], "mouse_y": mouse[1]}
        if contents is not None:
            made["patch"] = bundle(contents)
        if selected is not None:
            made["selected"] = selected
        return made

    three = [("dirt", 5, empty), ("ender_pearl", 2, empty), ("stone", 3, empty)]
    named = patch([(CUSTOM_NAME, nbt("Rock"))])
    damaged = patch([(DAMAGE, varint(100))])
    rows = [
        row("empty"),
        row("three", three),
        row("three, second selected", three, selected=1),
        row("named selected", [("stone", 2, named)] + three, selected=0),
        row("damaged sword", [("iron_sword", 1, damaged), ("stone", 9, empty)], selected=1),
        row("full", [("stone", 64, empty)]),
        row("nine", [("stone", 1, empty)] * 9, selected=8),
        row("thirteen", [("stone", n, empty) for n in range(1, 14)], selected=2),
        row("by the right edge", three, selected=2, mouse=(400, 60)),
        row("by the bottom", [("stone", 1, empty)] * 12, mouse=(50, 230), width=320, height=240),
    ]
    yield {"case_id": "gui/bundle-tooltip", "family": "gui", "operation": "bundle_tooltip", "rows": rows}
