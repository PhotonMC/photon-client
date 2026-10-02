"""GUI family corpus: original synthetic screen scenarios; expected values come from the Java JAR."""

import itertools
import math
import struct


def gui_cases():
    """The screens' drawing as the reference's own GUI code records it: the
    rectangles and the text, in order, with their poses applied. The glyphs
    are not drawn; their advances come from the reference's font providers."""
    def chat(label, messages, tick, width=427, height=240, focused=False, scroll=None,
             extra=(), deletions=(), options=None, rescale=None, lang_after=None, mouse=None):
        row = {"label": label, "width": width, "height": height,
               "messages": [{"text": text, "tick": at} for text, at in messages] + list(extra),
               "tick": tick}
        if deletions:
            row["deletions"] = [{"delete": sig, "tick": at} for sig, at in deletions]
        if focused:
            row["focused"] = True
        if scroll is not None:
            row["scroll"] = scroll
        if options:
            row["options"] = options
        if rescale:
            row["rescale"] = rescale
        if lang_after:
            row["lang_after"] = lang_after
        if mouse is not None:
            row["mouse"] = list(mouse)
        return row

    long_text = ("this is a fairly long chat message that should wrap onto a second line "
                 "when it reaches the edge of the chat box and goes on for a third one too")
    rows = [
        chat("one-line", [("hello world", 0)], 0),
        chat("three-lines", [("one", 0), ("two", 1), ("three", 2)], 3),
        chat("wrapped", [(long_text, 0)], 5),
        chat("fading", [("fading out", 0)], 190),
        chat("faded", [("gone", 0)], 200),
        chat("many", [(f"line {n}", 0) for n in range(14)], 10),
        chat("tall-window", [("hello", 0)], 0, width=640, height=480),
        chat("newline", [("first line\nsecond line", 0)], 0),
        chat("open-unscrolled", [(f"line {n}", 0) for n in range(30)], 400, focused=True),
        chat("open-scrolled", [(f"line {n}", 0) for n in range(30)], 400, focused=True, scroll=3),
        chat("open-scrolled-end", [(f"line {n}", 0) for n in range(30)], 400, focused=True, scroll=100),
        chat("open-short", [("only", 0), ("two", 0)], 400, focused=True),
        chat("tags", [], 5, extra=[
            {"text": "system", "tick": 0, "tag": "system"},
            {"text": "not secure", "tick": 1, "tag": "not_secure", "signature": 1},
            {"text": "modified", "tick": 2, "tag": "modified:the original", "signature": 2},
            {"text": "error", "tick": 3, "tag": "error"},
            {"text": "no tag", "tick": 4, "signature": 3},
        ]),
        chat("modified-wraps-narrower", [], 5, extra=[
            {"text": long_text, "tick": 0, "tag": "modified:x", "signature": 4},
        ]),
        chat("delete-waits", [], 30, extra=[{"text": "will go", "tick": 0, "signature": 7}],
             deletions=[(7, 10)]),
        chat("delete-after-sixty", [], 100, extra=[
            {"text": "before", "tick": 0, "tag": "system"},
            {"text": "will go", "tick": 1, "signature": 7},
            {"text": "after", "tick": 2, "tag": "system"},
        ], deletions=[(7, 5)]),
        chat("delete-unknown", [], 10, extra=[{"text": "kept", "tick": 0, "signature": 8}],
             deletions=[(9, 5)]),
    ]
    # The chat options: each alone and together, closed and open, before the
    # messages and after them (the chat wraps them again), with tags.
    many = [(f"line {n}", 0) for n in range(30)]
    opt = lambda **values: values
    rows += [
        chat("scale-half", [(long_text, 0)], 3, options=opt(scale=0.5)),
        chat("scale-three-quarters", [(long_text, 0)], 3, options=opt(scale=0.75)),
        chat("scale-third", [("hello", 0), ("two", 1)], 3, options=opt(scale=0.33)),
        chat("scale-open", many, 400, focused=True, options=opt(scale=0.6)),
        chat("scale-open-scrolled", many, 400, focused=True, scroll=5, options=opt(scale=0.9)),
        chat("scale-tags", [], 5, options=opt(scale=0.7), extra=[
            {"text": "system", "tick": 0, "tag": "system"},
            {"text": "modified", "tick": 2, "tag": "modified:the original", "signature": 2}]),
        chat("width-narrow", [(long_text, 0)], 3, options=opt(width=0.0)),
        chat("width-half", [(long_text, 0)], 3, options=opt(width=0.5)),
        chat("width-quarter-open", [(long_text, 0)] * 4, 3, focused=True, options=opt(width=0.25)),
        chat("width-and-scale", [(long_text, 0)], 3, options=opt(width=0.4, scale=0.8)),
        chat("opacity-zero", [("hello", 0)], 3, options=opt(opacity=0.0)),
        chat("opacity-low", [("hello", 0), ("fading", 1)], 3, options=opt(opacity=0.3)),
        chat("opacity-fading", [("fading out", 0)], 190, options=opt(opacity=0.5)),
        chat("opacity-open-tags", [], 5, focused=True, options=opt(opacity=0.4), extra=[
            {"text": "system", "tick": 0, "tag": "system"},
            {"text": "error", "tick": 3, "tag": "error"}]),
        chat("background-zero", [("hello", 0)], 3, options=opt(background_opacity=0.0)),
        chat("background-full", [("hello", 0), ("two", 0)], 3, options=opt(background_opacity=1.0)),
        chat("background-fading", [("fading out", 0)], 190, options=opt(background_opacity=0.8)),
        chat("spacing-full", [("one", 0), ("two", 1), ("three", 2)], 3, options=opt(line_spacing=1.0)),
        chat("spacing-quarter", [(f"line {n}", 0) for n in range(14)], 10, options=opt(line_spacing=0.25)),
        chat("spacing-open", many, 400, focused=True, options=opt(line_spacing=0.5)),
        chat("spacing-open-scrolled", many, 400, focused=True, scroll=4, options=opt(line_spacing=0.3)),
        chat("spacing-tags", [], 5, options=opt(line_spacing=0.6), extra=[
            {"text": "system", "tick": 0, "tag": "system"},
            {"text": "no tag", "tick": 4, "signature": 3}]),
        chat("height-unfocused-low", [(f"line {n}", 0) for n in range(14)], 10, options=opt(height_unfocused=0.0)),
        chat("height-unfocused-full", [(f"line {n}", 0) for n in range(30)], 10, options=opt(height_unfocused=1.0)),
        chat("height-focused-low", many, 400, focused=True, options=opt(height_focused=0.2)),
        chat("height-focused-half-scrolled", many, 400, focused=True, scroll=7, options=opt(height_focused=0.5)),
        chat("height-and-spacing", many, 400, focused=True, options=opt(height_focused=0.6, line_spacing=0.4)),
        chat("scroll-end-low-height", many, 400, focused=True, scroll=100, options=opt(height_focused=0.3)),
        chat("everything", many, 400, focused=True, scroll=3, options=opt(
            scale=0.7, width=0.6, opacity=0.6, background_opacity=0.3, line_spacing=0.2,
            height_focused=0.7)),
        chat("rescale-width", [(long_text, 0), ("short", 1)], 3, rescale=opt(width=0.3)),
        chat("rescale-scale", [(long_text, 0)], 3, rescale=opt(scale=0.5)),
        chat("rescale-resets-scroll", many, 400, focused=True, scroll=6, rescale=opt(opacity=0.9)),
        chat("rescale-after-options", [(long_text, 0)], 3, options=opt(width=0.8), rescale=opt(width=0.2)),
        chat("tall-window-scaled", [("hello", 0)], 0, width=640, height=480, options=opt(scale=0.5)),
    ]
    # The mouse over an open chat: the tooltip of the text under it. The last
    # line is at the bottom, its text from x 4 and 192 down.
    import json as _json
    show = lambda text, **style: dict({"text": text}, **style)
    tip = lambda value: {"action": "show_text", "value": value}
    hover = lambda text, value="tooltip text", **style: show(text, hover_event=tip(value), **style)
    comp = lambda component, at=0, **kw: dict({"json": _json.dumps(component), "tick": at}, **kw)
    rows += [
        chat("hover-show-text", [], 5, focused=True, mouse=(10, 196), extra=[comp(hover("hover me"))]),
        chat("hover-show-text-unfocused", [], 5, mouse=(10, 196), extra=[comp(hover("hover me"))]),
        chat("hover-miss-right", [], 5, focused=True, mouse=(300, 196), extra=[comp(hover("hover me"))]),
        chat("hover-miss-above", [], 5, focused=True, mouse=(10, 150), extra=[comp(hover("hover me"))]),
        chat("hover-left-edge", [], 5, focused=True, mouse=(4, 196), extra=[comp(hover("hover me"))]),
        chat("hover-before-left-edge", [], 5, focused=True, mouse=(3, 196), extra=[comp(hover("hover me"))]),
        chat("hover-last-glyph", [], 5, focused=True, mouse=(4 + 6 * 8 - 2, 196), extra=[comp(hover("hover me"))]),
        chat("hover-past-end", [], 5, focused=True, mouse=(4 + 8 * 6 + 1, 196), extra=[comp(hover("hover me"))]),
        chat("hover-glyph-top", [], 5, focused=True, mouse=(10, 192), extra=[comp(hover("hover me"))]),
        chat("hover-glyph-bottom", [], 5, focused=True, mouse=(10, 200), extra=[comp(hover("hover me"))]),
        chat("hover-below-line", [], 5, focused=True, mouse=(10, 201), extra=[comp(hover("hover me"))]),
        chat("hover-sibling", [], 5, focused=True, mouse=(40, 196),
             extra=[comp(show("a", extra=[hover("bold", "bold tip", bold=True)]))]),
        chat("hover-inherited", [], 5, focused=True, mouse=(20, 196),
             extra=[comp(hover("parent ", "parent tip", extra=[show("child")]))]),
        chat("hover-on-child-only", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(show("plain ", extra=[hover("child", "child tip")]))]),
        chat("hover-italic", [], 5, focused=True, mouse=(12, 196), extra=[comp(hover("slanted", italic=True))]),
        chat("hover-bold-edge", [], 5, focused=True, mouse=(11, 196), extra=[comp(hover("bold", bold=True))]),
        chat("hover-coloured-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("hover me", {"text": "red tip", "color": "red"}))]),
        chat("hover-long-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("hover me", long_text * 2))]),
        chat("hover-newline-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("hover me", "first\nsecond\nthird"))]),
        chat("hover-two-lines-upper", [], 5, focused=True, mouse=(10, 187),
             extra=[comp(hover("upper", "upper tip"), 0), comp(hover("lower", "lower tip"), 1)]),
        chat("hover-two-lines-lower", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("upper", "upper tip"), 0), comp(hover("lower", "lower tip"), 1)]),
        chat("hover-wrapped-second-line", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover(long_text, "wrapped tip"))]),
        chat("hover-wrapped-first-line", [], 5, focused=True, mouse=(10, 178),
             extra=[comp(hover(long_text, "wrapped tip"))]),
        chat("hover-with-tag-text", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("tagged"), tag="system")]),
        chat("hover-with-tag-bar", [], 5, focused=True, mouse=(0, 196),
             extra=[comp(hover("tagged"), tag="system")]),
        chat("hover-scaled", [], 5, focused=True, mouse=(5, 198), options=opt(scale=0.5),
             extra=[comp(hover("hover me"))]),
        chat("hover-scaled-miss", [], 5, focused=True, mouse=(5, 150), options=opt(scale=0.5),
             extra=[comp(hover("hover me"))]),
        chat("hover-spaced", [], 5, focused=True, mouse=(10, 196), options=opt(line_spacing=0.5),
             extra=[comp(hover("hover me"))]),
        chat("hover-tall-window", [], 5, focused=True, width=640, height=480, mouse=(10, 436),
             extra=[comp(hover("hover me"))]),
        chat("hover-narrow-window", [], 5, focused=True, width=200, height=150, mouse=(10, 106),
             extra=[comp(hover("hover me", long_text))]),
        chat("hover-space-glyph", [], 5, focused=True, mouse=(4 + 5 * 6 + 1, 196),
             extra=[comp(hover("hello world"))]),
        chat("hover-entity-no-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(show("entity", hover_event={"action": "show_entity", "id": "minecraft:pig",
                                                   "uuid": [1, 2, 3, 4]}))]),
        chat("click-only-no-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(show("link", click_event={"action": "open_url", "url": "https://example.com"}))]),
        chat("hover-last-wins", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("ab", "first", extra=[hover("", "second")]))]),
        chat("hover-no-mouse-rows-draw", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("hover me")), {"text": "x", "tick": 1}]),
    ]
    # A language change words the translatable messages again, and wraps them.
    warning = {"key": "options.languageAccuracyWarning"}
    rows += [
        chat("lang-same", [], 3, extra=[dict(warning, tick=0)], lang_after="en_us"),
        chat("lang-german", [], 3, extra=[dict(warning, tick=0), {"key": "gui.done", "tick": 1}], lang_after="de_de"),
        chat("lang-french", [], 3, extra=[dict(warning, tick=0), {"text": "plain", "tick": 1}], lang_after="fr_fr"),
        chat("lang-portuguese-tags", [], 3, extra=[dict(warning, tick=0, tag="system"),
                                                    {"key": "menu.quit", "tick": 1, "tag": "error"}],
             lang_after="pt_br"),
        chat("lang-spanish-open", [], 400, focused=True,
             extra=[dict(warning, tick=n) for n in range(0, 8)], lang_after="es_es", scroll=3),
        chat("lang-unknown", [], 3, extra=[dict(warning, tick=0)], lang_after="xx_xx"),
    ]
    yield {"case_id": "gui/chat", "family": "gui", "operation": "chat", "rows": rows}


def gui_title_cases():
    """The title, the subtitle and the action bar at points of their times: the
    fade in and out, the hold, the scales and places, and a subtitle with no
    title."""
    def titles(label, elapsed=0, partial=0.0, title="Hello", subtitle=None, overlay=None,
               overlay_elapsed=0, times=(10, 70, 20), width=427, height=240):
        row = {"label": label, "width": width, "height": height, "fade_in": times[0],
               "stay": times[1], "fade_out": times[2], "elapsed": elapsed,
               "overlay_elapsed": overlay_elapsed, "partial": partial}
        if title is not None:
            row["title"] = title
        if subtitle is not None:
            row["subtitle"] = subtitle
        if overlay is not None:
            row["overlay"] = overlay
        return row

    rows = [
        titles("fade-in-start", elapsed=0, partial=0.5),
        titles("fade-in-middle", elapsed=5, partial=0.25),
        titles("hold", elapsed=30, subtitle="below"),
        titles("fade-out", elapsed=90, partial=0.75, subtitle="below"),
        titles("last-tick", elapsed=99),
        titles("title-only-wide", elapsed=30, title="A much longer title to centre", width=640, height=480),
        titles("custom-times", elapsed=3, times=(5, 10, 5), subtitle="x"),
        titles("no-fade-in", elapsed=0, times=(0, 70, 20)),
        titles("subtitle-without-title", title=None, subtitle="lonely"),
        titles("action-bar", title=None, overlay="Action bar text", overlay_elapsed=10),
        titles("action-bar-fading", title=None, overlay="Action bar text", overlay_elapsed=50, partial=0.5),
        titles("both", elapsed=20, subtitle="sub", overlay="bar", overlay_elapsed=5),
    ]
    yield {"case_id": "gui/titles", "family": "gui", "operation": "titles", "rows": rows}


def command_tree_hex(root):
    """A `ClientboundCommandsPacket` payload for a tree of `("lit", name,
    children, exec)` and `("arg", name, (type id, parameter bytes), children,
    exec)` tuples; a child may be `("redirect", node)` on a node for its
    redirect, given as `redirects[id(node)]` through `with_redirect`."""
    def varint(value):
        out = bytearray()
        while True:
            byte = value & 0x7F
            value >>= 7
            out.append(byte | (0x80 if value else 0))
            if not value:
                return bytes(out)

    def utf(text):
        raw = text.encode()
        return varint(len(raw)) + raw

    order = []
    index = {}

    def visit(node):
        if id(node) in index:
            return
        index[id(node)] = len(order)
        order.append(node)
        for child in node["children"]:
            visit(child)
        if node.get("redirect") is not None:
            visit(node["redirect"])

    visit(root)
    out = bytearray(varint(len(order)))
    for node in order:
        flags = {"root": 0, "lit": 1, "arg": 2}[node["kind"]]
        if node.get("exec"):
            flags |= 4
        if node.get("redirect") is not None:
            flags |= 8
        if node.get("provider"):
            flags |= 16
        out.append(flags)
        out += varint(len(node["children"]))
        for child in node["children"]:
            out += varint(index[id(child)])
        if node.get("redirect") is not None:
            out += varint(index[id(node["redirect"])])
        if node["kind"] == "lit":
            out += utf(node["name"])
        elif node["kind"] == "arg":
            out += utf(node["name"]) + varint(node["type"][0]) + node["type"][1]
            if node.get("provider"):
                out += utf(node["provider"])
    out += varint(index[id(root)])
    return out.hex()


def gui_command_parse_cases():
    """Commands typed against a tree like a server's: which node read which
    part, what could not be read and why, the text's colours, and the usage
    and error lines under it. Argument types the port reads by shape are
    only given well-formed text."""
    import struct

    def lit(name, *children, exec=False, redirect=None):
        return {"kind": "lit", "name": name, "children": list(children), "exec": exec, "redirect": redirect}

    def arg(name, type_, *children, exec=False, redirect=None):
        return {"kind": "arg", "name": name, "type": type_, "children": list(children),
                "exec": exec, "redirect": redirect}

    nothing = b""
    boolean = (0, nothing)

    def integer(low=None, high=None):
        flags = (1 if low is not None else 0) | (2 if high is not None else 0)
        data = bytes([flags]) + (struct.pack(">i", low) if low is not None else b"") + (
            struct.pack(">i", high) if high is not None else b"")
        return (3, data)

    def floating(low=None, high=None):
        flags = (1 if low is not None else 0) | (2 if high is not None else 0)
        data = bytes([flags]) + (struct.pack(">f", low) if low is not None else b"") + (
            struct.pack(">f", high) if high is not None else b"")
        return (1, data)

    def double(low=None, high=None):
        flags = (1 if low is not None else 0) | (2 if high is not None else 0)
        data = bytes([flags]) + (struct.pack(">d", low) if low is not None else b"") + (
            struct.pack(">d", high) if high is not None else b"")
        return (2, data)

    def long_(low=None, high=None):
        flags = (1 if low is not None else 0) | (2 if high is not None else 0)
        data = bytes([flags]) + (struct.pack(">q", low) if low is not None else b"") + (
            struct.pack(">q", high) if high is not None else b"")
        return (4, data)

    def string(kind):
        return (5, bytes([kind]))

    entity = (6, bytes([0]))
    players = (6, bytes([1 | 2]))
    block_pos, vec3, vec2 = (8, nothing), (10, nothing), (11, nothing)
    column_pos = (9, nothing)
    item = (14, nothing)
    block_state = (12, nothing)
    message = (20, nothing)
    objective = (24, nothing)
    operation = (26, nothing)
    angle = (28, nothing)
    slot = (30, nothing)
    holder = (31, bytes([1]))
    swizzle = (32, nothing)
    identifier = (36, nothing)
    anchor = (38, nothing)
    gamemode = (42, nothing)
    time = (43, struct.pack(">i", 0))
    uuid = (56, nothing)
    team_color = (16, nothing)
    hex_color = (17, nothing)

    root = {"kind": "root", "children": [], "exec": False, "redirect": None}
    execute = lit("execute", lit("run", redirect=root),
                  lit("as", arg("targets", entity, redirect=None)))
    execute["children"][1]["children"][0]["redirect"] = execute
    root["children"] = [
        lit("gamemode", arg("gamemode", gamemode, exec=True), exec=False),
        lit("give", arg("targets", entity,
                        arg("item", item, exec=True,
                            *[arg("count", integer(1), exec=True)]))),
        lit("tp", arg("destination", entity, exec=True),
            arg("location", vec3, exec=True),
            arg("targets", entity, arg("location", vec3, exec=True))),
        lit("say", arg("message", message, exec=True)),
        lit("time", lit("set", lit("day", exec=True), lit("night", exec=True), arg("time", time, exec=True)),
            lit("add", arg("time", time, exec=True))),
        execute,
        lit("scoreboard", lit("players", lit("set", arg("targets", holder, arg("objective", objective,
            arg("score", integer(), exec=True)))))),
        lit("setblock", arg("pos", block_pos, arg("block", block_state, exec=True), exec=False)),
        lit("weather", lit("clear", arg("duration", integer(1, 1000000), exec=True), exec=True),
            lit("rain", exec=True), lit("thunder", exec=True)),
        lit("num", arg("f", floating(0.0, 1.0), arg("d", double(-5.0), arg("l", long_(0, 100),
            arg("b", boolean, exec=True), exec=True)))),
        lit("str", arg("word", string(0), arg("phrase", string(1), arg("rest", string(2), exec=True)))),
        lit("misc", lit("a", arg("angle", angle, exec=True)), lit("c", arg("col", column_pos, exec=True)),
            lit("v", arg("v2", vec2, exec=True)), lit("i", arg("id", identifier, exec=True)),
            lit("s", arg("slot", slot, exec=True)), lit("w", arg("swizzle", swizzle, exec=True)),
            lit("o", arg("op", operation, exec=True)), lit("e", arg("anchor", anchor, exec=True)),
            lit("u", arg("uuid", uuid, exec=True)), lit("t", arg("color", team_color, exec=True)),
            lit("h", arg("hex", hex_color, exec=True))),
        lit("alias", exec=True), lit("alias2", exec=True),
    ]
    tree = command_tree_hex(root)
    texts = [
        "/", "/g", "/gamemode", "/gamemode ", "/gamemode creative", "/gamemode creative ",
        "/gamemode nope", "/gamemode creative extra", "/unknown", "/unknown arg",
        "/give @p", "/give @p ", "/give @p minecraft:stone", "/give @p minecraft:stone 64",
        "/give @p minecraft:stone 0", "/give @e[type=pig,distance=..5] minecraft:stone 3 junk",
        "/tp @p", "/tp @p @a", "/tp 1 2 3", "/tp 1 2", "/tp ~ ~1 ~-2", "/tp @p 1 2 3", "/tp ^ ^ ^1",
        "/tp ^ 1 ^", "/say hello world", "/say", "/say ",
        "/time set day", "/time set 10s", "/time set 5x", "/time set night extra", "/time add 1d", "/time ",
        "/execute run gamemode survival", "/execute as @a run say hi", "/execute as @a ",
        "/execute run ", "/scoreboard players set @a obj 5", "/scoreboard players set @a obj x",
        "/scoreboard players ", "/setblock 1 2 3", "/setblock 1 2 3 minecraft:oak_log[axis=x]",
        "/setblock 1 2", "/setblock ~ ~ ~ minecraft:chest{Items:[{id:\"x\"}]} tail",
        "/weather clear", "/weather clear 100", "/weather clear 0", "/weather clear 2000000",
        "/weather snow", "/num 0.5 1.5 10 true", "/num 2 1 1 1", "/num 0.5 -9 1 1", "/num 0.5 1 101 true",
        "/num 0.5 1 1 maybe", "/num x", "/num 0.5 1e3", "/str word \"a phrase\" the rest",
        "/str \"unclosed", "/misc a ~5", "/misc a nan", "/misc c 1 2", "/misc c ~ ~1", "/misc v 1.5 2",
        "/misc i minecraft:stone", "/misc i Bad:Id", "/misc s weapon.mainhand", "/misc w xz", "/misc w xx",
        "/misc w q", "/misc o +=", "/misc o ><", "/misc o ?", "/misc e feet", "/misc e head",
        "/misc u 123e4567-e89b-12d3-a456-426614174000", "/misc u nope", "/misc t red", "/misc t pink",
        "/misc h 1a2", "/misc h abcd", "/alias", "/alias x", "/alias2 ", "no slash", "/gamemode  creative",
        "/give @p minecraft:stone 64 ", "/tp @p 1 2 3 4",
    ]
    rows = []
    for number, text in enumerate(texts):
        for empty in (True, False):
            rows.append({"label": f"{number}-{text}-{empty}", "text": text, "suggestions_empty": empty})
    yield {"case_id": "gui/command-parse", "family": "gui", "operation": "command_parse",
           "tree": tree, "rows": rows}


def gui_command_suggest_cases():
    """What a command tree suggests for text at a cursor: literals, the
    values of each argument type, selectors and names, coordinates, and
    whether the server would be asked (`ask_server`, unknown providers)."""
    import struct

    def parameters(type_id, variant=0):
        if type_id in (1, 2, 3, 4):
            return bytes([0])
        if type_id == 5:
            return bytes([variant])
        if type_id == 6:
            return bytes([variant])
        if type_id == 31:
            return bytes([variant])
        if type_id == 43:
            return struct.pack(">i", 0)
        if type_id in (44, 45, 46, 47, 48):
            name = b"minecraft:item"
            return bytes([len(name)]) + name
        return b""

    def lit(name, *children, exec=False):
        return {"kind": "lit", "name": name, "children": list(children), "exec": exec, "redirect": None}

    def arg(name, type_, *children, exec=False, provider=None):
        return {"kind": "arg", "name": name, "type": type_, "children": list(children),
                "exec": exec, "redirect": None, "provider": provider}

    branches = [lit(f"t{i}", arg("x", (i, parameters(i)), exec=True)) for i in range(57)]
    branches += [
        lit("single", arg("x", (6, parameters(6, 3)), exec=True)),
        lit("ask", arg("x", (5, parameters(5)), exec=True, provider="minecraft:ask_server")),
        lit("sounds", arg("x", (5, parameters(5)), exec=True, provider="minecraft:available_sounds")),
        lit("bogus", arg("x", (5, parameters(5)), exec=True, provider="minecraft:bogus")),
        lit("mixed", lit("literal", exec=True), arg("x", (5, parameters(5)), exec=True, provider="minecraft:ask_server")),
        lit("alpha", exec=True), lit("Alphabet", exec=True), lit("beta", exec=True), lit("gamma", exec=True),
    ]
    root = {"kind": "root", "children": branches, "exec": False, "redirect": None}
    tree = command_tree_hex(root)
    rows = []

    def add(text, cursor=None):
        row = {"label": f"{len(rows)}-{text}", "text": text, "players": ["Alex", "Miner", "miner2"],
               "teams": ["red", "blue"]}
        if cursor is not None:
            row["cursor"] = cursor
        rows.append(row)

    for text in ["/", "/t", "/T1", "/a", "/al", "/AL", "/alpha", "/alpha ", "/nope", "/be", "/zzz", ""]:
        add(text)
    # The game lists these from its registries; the port asks the server.
    registry = {12, 13, 14, 15, 25, 27, 34, 35, 41, 44, 45, 46, 47, 48, 55}
    for i in range(57):
        add(f"/t{i} ")
        if i in registry:
            rows[-1]["registry_backed"] = True
    samples = {
        0: ["tr", "F", "x"], 42: ["cre", "ive", "s", "x"], 38: ["f", "e", "ey"],
        26: ["=", "+", "<", "x"], 16: ["re", "ed", "dark_", "_blue", "x"], 30: ["side", "sidebar.t", "team.r", "x"],
        33: ["r", "b", "x"], 6: ["@", "@a", "@z", "@a[", "@a[t", "Mi", "mi", "x", "@a ", "@p[type=player,"],
        7: ["@", "Mi"], 31: ["@", "Mi", "*"],
        8: ["1", "1 2", "~", "^", "1 2 3"], 10: ["1", "1 2", "~", "^", "1 2 3"], 9: ["1", "1 2"], 11: ["1", "1 2"],
        29: ["1"], 43: ["1", "1d", "x"], 5: ["a"],
    }
    for i, texts in samples.items():
        for text in texts:
            add(f"/t{i} {text}")
            if text.startswith(("@a[", "@p[")):
                rows[-1]["registry_backed"] = True
    add("/single ")
    add("/single @")
    for name in ("ask", "sounds", "bogus", "mixed"):
        add(f"/{name} ")
        add(f"/{name} a")
        if name == "sounds":
            # A provider the game lists locally (the sounds); the port asks.
            rows[-1]["registry_backed"] = rows[-2]["registry_backed"] = True
    add("/t42 creative ")
    add("/t6 Alex ")
    add("/t42 cr", cursor=5)
    add("/t42 creative", cursor=5)
    yield {"case_id": "gui/command-suggest", "family": "gui", "operation": "command_suggest",
           "tree": tree, "rows": rows}


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


def gui_boss_bar_cases():
    """The boss bars: every colour and notch, progress at its ends and in the
    middle (the discrete width), several bars down the screen and the cut-off
    at a third of its height."""
    def bar(name, progress=1.0, color=0, overlay=0):
        return {"name": name, "progress": progress, "color": color, "overlay": overlay}

    def boss(label, bars, width=427, height=240):
        return {"label": label, "width": width, "height": height, "bars": bars}

    rows = [
        boss("full", [bar("Ender Dragon")]),
        boss("empty", [bar("Wither", 0.0, 4)]),
        boss("a-sliver", [bar("Wither", 0.001, 2)]),
        boss("half", [bar("Half", 0.5, 3)]),
        boss("almost-full", [bar("Almost", 0.999, 5)]),
    ]
    rows += [boss("colour-%d" % c, [bar("Colour", 0.7, c)]) for c in range(7)]
    rows += [boss("notches-%d" % o, [bar("Notched", 0.6, 1, o)]) for o in range(1, 5)]
    rows.append(boss("three", [bar("One", 0.3), bar("Two", 0.6, 1, 2), bar("Three", 0.9, 2, 4)]))
    rows.append(boss("cut-off", [bar("Bar %d" % i, 0.5, i % 7) for i in range(8)], height=240))
    rows.append(boss("cut-off-tall", [bar("Bar %d" % i, 0.5, i % 7) for i in range(8)], height=480))
    rows.append(boss("narrow-screen", [bar("A long boss name to centre", 0.4)], width=200, height=150))
    yield {"case_id": "gui/boss_bar", "family": "gui", "operation": "boss_bar", "rows": rows}


def gui_sidebar_cases():
    """The scoreboard's sidebar: ordering (value, then name without case),
    hidden `#` owners, the 15 line limit, number formats (blank, styled,
    fixed, per score), display names, team prefixes, suffixes and colours, and
    a team colour's own sidebar over the general one."""
    def objective(name="o", title="Title", fmt=None):
        described = {"name": name, "title": title}
        if fmt is not None:
            described["format"] = fmt
        return described

    def score(owner, value, name="o", display=None, fmt=None):
        described = {"owner": owner, "objective": name, "value": value}
        if display is not None:
            described["display"] = display
        if fmt is not None:
            described["format"] = fmt
        return described

    def board(label, objectives, scores, slots=None, teams=None, local="me", width=427, height=240):
        row = {"label": label, "width": width, "height": height, "local": local,
               "objectives": objectives, "scores": scores,
               "slots": slots if slots is not None else {"1": objectives[0]["name"]}}
        if teams:
            row["teams"] = teams
        return row

    rows = [
        board("basic", [objective()], [score("Alice", 5), score("Bob", 12), score("Cy", 1)]),
        board("ties-by-name", [objective()], [score("bob", 5), score("Al", 5), score("cy", 5), score("AL2", 5)]),
        board("negative", [objective()], [score("a", -3), score("b", 0), score("c", 7)]),
        board("hidden", [objective()], [score("#total", 99), score("visible", 1)]),
        board("fifteen-limit", [objective()], [score("p%02d" % i, i) for i in range(20)]),
        board("wide-title", [objective(title="A rather long scoreboard title")], [score("a", 1)]),
        board("long-name", [objective(title="T")], [score("a_name_that_is_quite_long", 123456)]),
        board("blank-format", [objective(fmt="blank")], [score("a", 1), score("b", 2)]),
        board("fixed-format", [objective(fmt="fixed:many")], [score("a", 1)]),
        board("styled-format", [objective(fmt="style:green")], [score("a", 1)]),
        board("per-score-format", [objective()], [score("a", 1, fmt="fixed:x"), score("b", 2)]),
        board("display-name", [objective()], [score("a", 1, display="Shown")]),
        board("team-dress", [objective()], [score("al", 3), score("bo", 2)],
              teams=[{"name": "t", "prefix": "[T] ", "suffix": "!", "color": 12, "players": ["al"]}]),
        board("team-colour-sidebar", [objective("all"), objective("mine", "Team only")],
              [score("al", 1, "all"), score("x", 9, "mine")],
              slots={"1": "all", "15": "mine"},
              teams=[{"name": "t", "color": 12, "players": ["me"]}]),
        board("team-without-slot", [objective("all")], [score("al", 1, "all")],
              teams=[{"name": "t", "color": 12, "players": ["me"]}]),
        board("no-sidebar", [objective()], [score("a", 1)], slots={"0": "o"}),
        board("small-screen", [objective()], [score("a", 1), score("b", 2)], width=200, height=150),
    ]
    yield {"case_id": "gui/sidebar", "family": "gui", "operation": "sidebar", "rows": rows}


def gui_tab_list_cases():
    """The player list: one column and several, the header and the footer (and
    their wrapping), spectators, display names, list order and the ping icons."""
    def player(name, mode=0, latency=0, order=0, display=None):
        described = {"name": name, "mode": mode, "latency": latency, "order": order}
        if display is not None:
            described["display"] = display
        return described

    def tab(label, players, header=None, footer=None, width=427, height=240):
        row = {"label": label, "width": width, "height": height, "players": players}
        if header is not None:
            row["header"] = header
        if footer is not None:
            row["footer"] = footer
        return row

    twenty = [player(f"player{n:02}", latency=n * 40) for n in range(20)]
    many = [player(f"player{n:02}", latency=n * 20 - 10) for n in range(45)]
    rows = [
        tab("alone", [player("Miner", latency=23)]),
        tab("pings", [player("a", latency=-1), player("b", latency=100), player("c", latency=200),
                      player("d", latency=400), player("e", latency=800), player("f", latency=2000)]),
        tab("header-footer", [player("Miner")], header="Welcome to the server", footer="Have fun"),
        tab("footer-only", [player("Miner")], footer="Bottom line"),
        tab("header-only", [player("Miner")], header="Top line"),
        tab("wrapped-header", [player("Miner")],
            header="a header long enough that the tab list has to break it over lines " * 2),
        tab("sorted", [player("zed"), player("Amy", mode=3), player("bob"), player("Cat", order=5),
                       player("dan", mode=3, order=5)]),
        tab("display-names", [player("a", display="Alice the Great"), player("b")]),
        tab("twenty", twenty),
        tab("columns", many),
        tab("narrow", many, width=320),
        tab("eighty-cap", [player(f"p{n:03}") for n in range(95)]),
    ]

    def scored(label, players, scores, hearts=False, fmt=None, teams=None, width=427, height=240):
        objective = {"name": "o", "title": "T"}
        if hearts:
            objective["hearts"] = True
        if fmt is not None:
            objective["format"] = fmt
        row = tab(label, players, width=width, height=height)
        row["objectives"] = [objective]
        row["scores"] = [{"owner": name, "objective": "o", "value": value} for name, value in scores.items()]
        row["slots"] = {"0": "o"}
        if teams:
            row["teams"] = teams
        return row

    trio = [player("Amy"), player("Bob"), player("Cy")]
    rows += [
        scored("scores", trio, {"Amy": 3, "Bob": 120, "Cy": 7}),
        scored("score-missing", trio, {"Amy": 3}),
        scored("score-spectator", [player("Amy"), player("Bob", mode=3)], {"Amy": 3, "Bob": 5}),
        scored("score-blank", trio, {"Amy": 3, "Bob": 4}, fmt="blank"),
        scored("score-fixed", trio, {"Amy": 3}, fmt="fixed:lots"),
        scored("score-styled", trio, {"Amy": 3}, fmt="style:aqua"),
        scored("score-columns", [player(f"player{n:02}") for n in range(30)],
               {f"player{n:02}": n * 13 for n in range(30)}),
        scored("hearts", trio, {"Amy": 20, "Bob": 7, "Cy": 1}, hearts=True),
        scored("hearts-over-twenty", trio, {"Amy": 30, "Bob": 22}, hearts=True),
        scored("hearts-text", trio, {"Amy": 80, "Bob": 100}, hearts=True),
        scored("hearts-zero", trio, {"Amy": 0}, hearts=True),
        scored("hearts-narrow", trio, {"Amy": 20}, hearts=True, width=160),
        scored("teams", [player("zed"), player("amy"), player("bob")], {},
               teams=[{"name": "b", "prefix": "[B] ", "color": 12, "players": ["amy"]},
                      {"name": "a", "suffix": " (A)", "color": 10, "players": ["zed", "bob"]}]),
        scored("team-with-display", [player("zed", display="Shown")], {},
               teams=[{"name": "a", "prefix": "[A] ", "color": 12, "players": ["zed"]}]),
    ]
    yield {"case_id": "gui/tab-list", "family": "gui", "operation": "tab_list", "rows": rows}


def gui_container_screen_cases():
    """The screens of the menus that change their items, over the reference's
    own menu: labels, textures, sprites, name box and what each shows of its
    data. Slots hold items, which are not recorded."""
    def screen(label, menu, title_key, slots=(), data=None, xp=None, keys=(), sync_slot=None,
               mouse=(0, 0), width=427, height=240, stonecutter=()):
        row = {"label": label, "width": width, "height": height, "mouse_x": mouse[0],
               "mouse_y": mouse[1], "menu": f"minecraft:{menu}", "title_key": title_key,
               "container_id": 4, "game_mode": "survival", "selected": 0, "state_id": 0,
               "blocks": [], "chunk_radius": 0, "clicks": [], "keys": list(keys),
               "slots": [{"slot": slot, "item": f"minecraft:{item}", "count": count}
                         for slot, item, count in slots]}
        if data is not None:
            row["data"] = data
        if xp is not None:
            row["xp_level"] = xp
        if sync_slot is not None:
            row["sync_slot"] = sync_slot
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
    yield {"case_id": "gui/container-screen", "family": "gui", "operation": "container_screen",
           "rows": rows}


def gui_suggestion_cases():
    """The chat box's list of matches: where it sits, which one is chosen, what
    Tab, Shift+Tab, the arrow keys and Escape do to the list and the text, and
    the grey text after the cursor."""
    def row(label, text, start, length, matches, keys, width=427, height=240):
        return {"label": label, "width": width, "height": height, "text": text, "start": start,
                "length": length, "matches": matches, "keys": keys}

    many = [f"entry{n}" for n in range(14)]
    rows = [
        row("first-chosen", "/gam", 1, 3, ["gamemode", "gamerule"], []),
        row("tab-twice", "/gam", 1, 3, ["gamemode", "gamerule"], ["tab", "tab", "tab"]),
        row("shift-tab", "/gam", 1, 3, ["gamemode", "gamerule"], ["tab", "shift+tab", "tab"]),
        row("arrows", "/gam", 1, 3, ["gamemode", "gamerule", "gametest"], ["down", "down", "up", "up", "up"]),
        row("escape-hides", "/gam", 1, 3, ["gamemode", "gamerule"], ["escape"]),
        row("prefix-first", "/ga", 1, 2, ["zgame", "gamemode", "minecraft:gamerule"], []),
        row("mid-text", "/say hel", 5, 3, ["help", "hello", "hex"], ["tab"]),
        row("scrolling-window", "/e", 1, 1, many, ["down"] * 11 + ["up"] * 3),
        row("wraps-round", "/e", 1, 1, many, ["up", "up", "down"]),
        row("narrow-screen", "/gam", 1, 3, ["gamemode", "gamerule"], ["tab"], width=320, height=240),
    ]
    yield {"case_id": "gui/suggestions", "family": "gui", "operation": "suggestions", "rows": rows}


def gui_confirm_screen_cases():
    """The confirmation screen (and the death screen's own, over its red
    gradient): the message wrapped and clipped to fifteen lines, the delay
    that keeps the buttons asleep and Escape shut, Tab, Enter and clicks, and
    what the callback was told."""
    def row(label, width=427, height=240, mouse=(0, 0), death=False, title="Are you sure?",
            message="This cannot be undone.", yes="Yes", no="No", delay=0, ticks=0, keys=(), click=False,
            in_world=False, link=None):
        row = {"label": label, "width": width, "height": height, "mouse": list(mouse), "death": death,
               "title": title, "message": message, "yes": yes, "no": no, "delay": delay, "ticks": ticks,
               "keys": list(keys), "click": click, "in_world": in_world}
        if link:
            row["link"] = link
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


def gui_options_screen_cases():
    """The options screen: the field of view, the header's second button
    (online outside a world, world options in one), the grid of settings
    screens (telemetry asleep: the player is offline) and Done, in several
    screen sizes; hover, Tab through every widget and round, the slider
    moved with the mouse and the keys (an arrow key that the slider does not
    take moves the focus, which is not built). The buttons that open other
    screens are not pressed."""
    def row(label, width=427, height=240, mouse=(0, 0), in_world=True, fov=70, steps=()):
        return {"label": label, "width": width, "height": height, "mouse": list(mouse), "in_world": in_world,
                "fov": fov, "steps": list(steps)}

    def key(name):
        return {"do": "key", "key": name}

    click = lambda x, y=38: {"do": "click", "x": x, "y": y}
    drag = lambda x, y=38: {"do": "drag", "x": x, "y": y}
    release = lambda x, y=38: {"do": "release", "x": x, "y": y}
    tab = key("tab")
    rows = [
        row("in-world"), row("out-of-world", in_world=False), row("small", 320, 240), row("large", 640, 360),
        row("tiny", 200, 150), row("wide", 854, 480, in_world=False), row("short", 427, 200),
        row("fov-min", fov=30), row("fov-max", fov=110), row("fov-90", fov=90),
        row("hover-fov", mouse=(100, 38)), row("hover-online", mouse=(300, 38), in_world=False),
        row("hover-world-options", mouse=(300, 38)), row("hover-skin", mouse=(100, 95)),
        row("hover-controls", mouse=(300, 119)), row("hover-telemetry-asleep", mouse=(100, 190)),
        row("hover-done", mouse=(200, 220)), row("hover-gap", mouse=(212, 95)),
        row("hover-elsewhere", mouse=(5, 5)),
        row("tab-1", steps=[tab]), row("tab-2", steps=[tab] * 2), row("tab-3", steps=[tab] * 3),
        row("tab-9", steps=[tab] * 9), row("tab-10-past-telemetry", steps=[tab] * 10),
        row("tab-12-done", steps=[tab] * 12), row("tab-13-round", steps=[tab] * 13),
        row("shift-tab", steps=[key("shift+tab")]), row("shift-tab-2", steps=[key("shift+tab")] * 2),
        row("tab-then-back", steps=[tab, tab, key("shift+tab")]),
        row("tab-back-round", steps=[tab, key("shift+tab")]),
        row("keys-right", steps=[tab, key("right"), key("right"), key("right")]),
        row("keys-left", steps=[tab, key("left")] + [key("left")] * 2, fov=40),
        row("keys-at-max", fov=110, steps=[tab, key("right")]),
        row("keys-enter-twice", steps=[tab, key("enter"), key("enter"), key("right")]),
        row("keys-enter-locks-shown", steps=[tab, key("enter")]),
        row("enter-unfocused", steps=[key("enter")]),
        row("click-fov", steps=[click(130)]), row("click-fov-release", steps=[click(130), release(130)]),
        row("click-fov-left", steps=[click(59), release(59)]),
        row("click-fov-right", steps=[click(208), release(208)]),
        row("drag-fov", steps=[click(70), drag(110), drag(150)]),
        row("drag-fov-release", steps=[click(70), drag(110), drag(150), release(150)]),
        row("drag-past-end", steps=[click(70), drag(900), release(900)]),
        row("drag-without-click", steps=[drag(150), release(150)]),
        row("click-empty", steps=[click(5, 5), release(5, 5)]),
        row("click-telemetry-asleep", steps=[click(100, 190), release(100, 190)]),
        row("click-fov-then-keys", steps=[click(130), release(130), key("right")]),
        row("click-fov-then-tab", steps=[click(130), release(130), tab]),
        row("tab-then-click-fov", steps=[tab, tab, click(130), release(130)]),
        # Arrows the slider does not take move the focus to the neighbour.
        row("arrow-down-from-nothing", steps=[key("down")]),
        row("arrow-up-from-nothing", steps=[key("up")]),
        row("arrow-right-from-nothing", steps=[key("right")]),
        row("arrow-left-from-nothing", steps=[key("left")]),
        row("arrow-down-from-fov", steps=[tab, key("down")]),
        row("arrow-down-twice", steps=[tab, key("down"), key("down")]),
        row("arrow-down-thrice", steps=[tab, key("down"), key("down"), key("down")]),
        row("arrow-down-to-done", steps=[tab] + [key("down")] * 7),
        row("arrow-down-past-done", steps=[tab] + [key("down")] * 9),
        row("arrow-up-from-done", steps=[tab, key("shift+tab"), key("up")]),
        row("arrow-up-from-fov", steps=[tab, key("up")]),
        row("arrow-right-from-button", steps=[tab, key("down"), key("right")]),
        row("arrow-left-from-button", steps=[tab, key("down"), key("right"), key("left")]),
        row("arrow-grid-down", steps=[tab, key("down"), key("down"), key("down"), key("up")]),
        row("arrow-grid-across", steps=[tab, key("down"), key("down"), key("right"), key("down"), key("left")]),
        row("arrow-over-asleep", steps=[tab] + [key("down")] * 6 + [key("right"), key("down")]),
        row("arrow-slider-takes-right", steps=[tab, key("enter"), key("right"), key("right")]),
        row("arrow-slider-left-not-locked", steps=[tab, key("enter"), key("enter"), key("left")]),
        row("arrow-after-click", steps=[click(130), release(130), key("down")]),
        row("arrow-wide-small", 200, 150, steps=[tab, key("down"), key("down"), key("right")]),
        row("arrow-wide-large", 640, 360, steps=[tab, key("down"), key("right"), key("down")]),
        row("arrow-out-of-world", in_world=False, steps=[key("down"), key("down"), key("right")]),
    ]
    yield {"case_id": "gui/options-screen", "family": "gui", "operation": "options_screen", "rows": rows}


def gui_video_settings_cases():
    """The video settings screen: the list of display, quality and preference
    options in several window sizes and GUI scales, scrolled with the wheel and
    the scroll bar, hovered, pressed (a cycle button, with and without Shift,
    a slider by a press and a drag), Control and the wheel through the GUI
    scales, and the preset turning to Custom when a quality option changes.
    Nothing presses the options whose reference effect needs a window (full
    screen) or a pipeline (the presets): those are drawn and not pressed."""
    def row(label, framebuffer=(854, 480), gui_scale=2, in_world=True, mouse=(0, 0), steps=(), ms=0):
        return {"label": label, "framebuffer": list(framebuffer), "gui_scale": gui_scale,
                "in_world": in_world, "mouse": list(mouse), "steps": list(steps), "ms": ms}

    click = lambda x, y, shift=False: {"do": "click", "x": x, "y": y, "shift": shift}
    drag = lambda x, y: {"do": "drag", "x": x, "y": y}
    release = lambda x=0, y=0: {"do": "release", "x": x, "y": y}
    wheel = lambda dy, x=200, y=100, ctrl=False: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": ctrl}
    rows = [
        row("in-world"), row("out-of-world", in_world=False),
        row("small", (640, 480)), row("large", (1280, 720)), row("tiny", (640, 480), 3),
        row("auto-large", (1920, 1080), 0), row("short", (854, 300)), row("scale-1", gui_scale=1),
        row("very-short", (640, 400), 4), row("wide", (1708, 480), in_world=False),
        row("hover-gui-scale", mouse=(250, 110)), row("hover-fullscreen-resolution", mouse=(200, 60)),
        row("hover-gamma", mouse=(100, 160)), row("hover-done", mouse=(213, 223)),
        row("hover-gap", mouse=(212, 110)), row("hover-first-pixel-of-last", mouse=(100, 206)),
        row("hover-under-header", mouse=(100, 30), steps=[wheel(-2)]),
        row("hover-header-line", mouse=(100, 40), steps=[wheel(-1)]),
        row("scroll-1", steps=[wheel(-1)]), row("scroll-5", steps=[wheel(-1)] * 5),
        row("scroll-10", steps=[wheel(-1)] * 10), row("scroll-end", steps=[wheel(-100)]),
        row("scroll-up-past-top", steps=[wheel(-3), wheel(10)]),
        row("scroll-fraction", steps=[wheel(-0.5)]), row("scroll-fraction-up", steps=[wheel(-4), wheel(1.5)]),
        row("scroll-over-header", steps=[wheel(-3, 200, 10)]),
        row("scroll-over-widget", steps=[wheel(-2, 250, 110)]),
        row("scroll-end-short", (854, 300), steps=[wheel(-100)]),
        row("scroll-end-very-short", (640, 400), 4, steps=[wheel(-100)]),
        row("scroll-end-large", (1280, 720), steps=[wheel(-100)]),
        row("scroll-end-out-of-world", in_world=False, steps=[wheel(-100)]),
        row("bar-drag", steps=[click(379, 60), drag(379, 90), release(379, 90)]),
        row("bar-drag-held", steps=[click(379, 60), drag(379, 90)]),
        row("bar-drag-past-bottom", steps=[click(379, 60), drag(379, 230)]),
        row("bar-drag-above", steps=[wheel(-5), click(379, 100), drag(379, 20)]),
        row("bar-drag-in-steps", steps=[click(379, 50), drag(379, 70), drag(379, 100), drag(379, 130)]),
        row("bar-click-only", steps=[click(379, 100), release(379, 100)]),
        row("bar-drag-short", (854, 300), steps=[click(379, 50), drag(379, 80)]),
        row("bar-drag-very-short", (640, 400), 4, steps=[click(100, 40), drag(100, 60)]),
        row("bar-edge-left", steps=[click(376, 60), drag(376, 100)]),
        row("bar-edge-right", steps=[click(382, 60), drag(382, 100)]),
        row("bar-beside", steps=[click(383, 60), drag(383, 100)]),
        row("gui-scale-press", steps=[click(250, 110)]),
        row("gui-scale-press-hover", mouse=(250, 110), steps=[click(250, 110)]),
        row("gui-scale-press-twice", steps=[click(250, 110), click(250, 110)]),
        row("gui-scale-press-thrice", steps=[click(250, 110)] * 3),
        row("gui-scale-shift", steps=[click(250, 110, True)]),
        row("gui-scale-from-auto", (1920, 1080), 0, steps=[click(300, 110)]),
        row("gui-scale-large-auto-twice", (1920, 1080), 0, steps=[click(300, 110), click(300, 110)]),
        row("gui-scale-then-scroll", steps=[click(250, 110), wheel(-3, 300, 100)]),
        row("gui-scale-scrolled", steps=[wheel(-5), click(250, 50)]),
        row("gui-scale-then-press-vsync", steps=[click(250, 110), click(250, 85)]),
        row("gui-scale-short", (854, 300), steps=[click(250, 110)]),
        row("gui-scale-shift-wraps", (1920, 1080), 0, steps=[click(300, 110, True)]),
        row("press-vsync", steps=[click(250, 85)]), row("press-vsync-away", mouse=(5, 5), steps=[click(250, 85)]),
        row("press-vsync-twice", steps=[click(250, 85), click(250, 85)]),
        row("press-vsync-shift", steps=[click(250, 85, True)]),
        row("press-inactivity", steps=[click(100, 110)]),
        row("press-exclusive", steps=[click(250, 135)]),
        row("press-backend", steps=[click(250, 160)]), row("press-backend-3", steps=[click(250, 160)] * 3),
        row("press-gap", steps=[click(212, 85)]), row("press-header-line", steps=[click(100, 40)]),
        row("press-outside-row", steps=[click(30, 85)]), row("press-under-header", steps=[click(100, 20)]),
        row("press-two-then-move-focus", steps=[click(250, 85), click(100, 110)]),
        row("press-slider-framerate", steps=[click(100, 85)]),
        row("press-slider-framerate-release", steps=[click(100, 85), release(100, 85)]),
        row("drag-slider-framerate", steps=[click(100, 85), drag(150, 85), release(150, 85)]),
        row("drag-slider-framerate-held", steps=[click(100, 85), drag(190, 85)]),
        row("drag-slider-framerate-end", steps=[click(100, 85), drag(300, 85), release(300, 85)]),
        row("drag-slider-framerate-start", steps=[click(100, 85), drag(0, 85), release(0, 85)]),
        row("press-slider-gamma", steps=[click(70, 160)]), row("drag-slider-gamma", steps=[click(70, 160), drag(60, 160)]),
        row("drag-slider-gamma-100", steps=[click(70, 160), drag(300, 160), release(300, 160)]),
        row("drag-slider-gamma-0", steps=[click(70, 160), drag(10, 160), release(10, 160)]),
        row("press-resolution-slider", steps=[click(100, 60)]),
        row("press-resolution-slider-drag", steps=[click(100, 60), drag(200, 60), release(200, 60)]),
        row("press-preset-last-pixel", steps=[click(170, 206)]),
        row("press-preset-last-pixel-release", steps=[click(170, 206), release(170, 206)]),
        row("press-preset-scrolled-in", steps=[wheel(-9), click(170, 100)]),
        row("press-ao-custom", steps=[wheel(-10), click(100, 170)]),
        row("press-ao-custom-twice", steps=[wheel(-10), click(100, 170), click(100, 170)]),
        row("press-shadows-custom", steps=[wheel(-10), click(100, 180)]),
        row("press-particles-custom", steps=[wheel(-10), click(300, 170)]),
        row("press-vsync-keeps-preset", steps=[wheel(-10), click(250, 20)]),
        row("press-weather-slider", steps=[wheel(-100), click(300, 110)]),
        row("press-weather-slider-drag", steps=[wheel(-100), click(300, 110), drag(250, 110), release(250, 110)]),
        row("press-cloud-range-clipped", steps=[wheel(-100), click(300, 40)]),
        row("press-cutout-leaves", steps=[wheel(-100), click(100, 60)]),
        row("press-transparency", steps=[wheel(-100), click(300, 60)]),
        row("anisotropy-asleep", steps=[wheel(-100)]),
        row("anisotropy-asleep-hover", mouse=(300, 86), steps=[wheel(-100)]),
        row("anisotropy-asleep-press", steps=[wheel(-100), click(300, 86)]),
        row("filtering-rgss", steps=[wheel(-100), click(100, 86)]),
        row("filtering-anisotropic", steps=[wheel(-100), click(100, 86), click(100, 86)]),
        row("filtering-anisotropic-press-slider", steps=[wheel(-100), click(100, 86)] * 2 + [click(300, 86)]),
        row("filtering-anisotropic-drag-slider",
            steps=[wheel(-100), click(100, 86), click(100, 86), click(250, 86), drag(340, 86), release(340, 86)]),
        row("filtering-back-to-none", steps=[wheel(-100)] + [click(100, 86)] * 3),
        row("press-autosave", steps=[wheel(-100), click(100, 165)]),
        row("press-vignette", steps=[wheel(-100), click(300, 165)]),
        row("press-attack", steps=[wheel(-100), click(100, 190)]),
        row("press-attack-shift", steps=[wheel(-100), click(100, 190, True)]),
        row("press-chunk-fade", steps=[wheel(-100), click(300, 190)]),
        row("drag-chunk-fade", steps=[wheel(-100), click(300, 190), drag(240, 190), release(240, 190)]),
        row("drag-chunk-fade-zero", steps=[wheel(-100), click(300, 190), drag(0, 190), release(0, 190)]),
        row("ctrl-wheel-down", steps=[wheel(-1, ctrl=True)]), row("ctrl-wheel-up-at-top", steps=[wheel(1, ctrl=True)]),
        row("ctrl-wheel-auto-down", (1920, 1080), 0, steps=[wheel(-1, ctrl=True)]),
        row("ctrl-wheel-auto-up", (1920, 1080), 0, steps=[wheel(1, ctrl=True)]),
        row("ctrl-wheel-down-twice", (1920, 1080), 3, steps=[wheel(-1, ctrl=True)] * 2),
        row("ctrl-wheel-down-thrice", (1920, 1080), 3, steps=[wheel(-1, ctrl=True)] * 3),
        row("ctrl-wheel-up", (1920, 1080), 2, steps=[wheel(1, ctrl=True)]),
        row("ctrl-wheel-scrolled", steps=[wheel(-5), wheel(-1, ctrl=True)]),
        row("ctrl-wheel-outside-list", steps=[wheel(-1, 200, 10, True)]),
        row("ctrl-wheel-fraction", (1920, 1080), 3, steps=[wheel(-0.5, ctrl=True)]),
        row("ctrl-wheel-focused", steps=[click(250, 85), wheel(-1, ctrl=True)]),
        row("ctrl-wheel-then-wheel", steps=[wheel(-1, ctrl=True), wheel(-3)]),
        row("ctrl-wheel-at-1", gui_scale=1, steps=[wheel(-1, ctrl=True)]),
    ]
    # Tooltips: a moment after the mouse is on the widget (a frame at 0, one at 10).
    frame = lambda ms: {"do": "frame", "ms": ms}
    tip = lambda label, mouse, steps=(), **kw: row("tip-" + label, mouse=mouse, steps=[frame(0)] + list(steps), ms=10, **kw)
    rows += [
        row("tip-first-frame", mouse=(100, 110)),
        tip("inactivity", (100, 110)), tip("exclusive", (250, 135)), tip("backend", (250, 160)),
        tip("gui-scale-none", (250, 110)), tip("resolution-none", (200, 60)),
        tip("preset", (150, 108), [wheel(-9)]), tip("preset-last-pixel", (150, 206)),
        tip("prioritize", (100, 146), [wheel(-10)]),
        tip("ao-none", (100, 170), [wheel(-10)]),
        tip("blurriness", (100, 40), [wheel(-100)]), tip("cutout-leaves", (100, 60), [wheel(-100)]),
        tip("transparency", (300, 60), [wheel(-100)]), tip("filtering", (100, 86), [wheel(-100)]),
        tip("anisotropy-asleep", (300, 86), [wheel(-100)]), tip("weather", (100, 110), [wheel(-100)]),
        tip("vignette", (300, 167), [wheel(-100)]), tip("chunk-fade", (300, 192), [wheel(-100)]),
        tip("attack-none", (100, 192), [wheel(-100)]),
        tip("exclusive-restart", (250, 135), [click(250, 135)]),
        tip("exclusive-back", (250, 135), [click(250, 135), click(250, 135)]),
        tip("backend-restart", (250, 160), [click(250, 160)]),
        tip("backend-vulkan", (250, 160), [click(250, 160), click(250, 160)]),
        tip("backend-default", (250, 160), [click(250, 160)] * 3),
        tip("inactivity-minimized", (100, 110), [click(100, 110)]),
        tip("filtering-rgss", (100, 86), [wheel(-100), click(100, 86)]),
        tip("filtering-anisotropic", (100, 86), [wheel(-100), click(100, 86), click(100, 86)]),
        tip("anisotropy-awake", (300, 86), [wheel(-100), click(100, 86), click(100, 86)]),
        tip("prioritize-by-player", (100, 146), [wheel(-10), click(100, 146)]),
        tip("prioritize-nearby", (100, 146), [wheel(-10), click(100, 146), click(100, 146)]),
        tip("preset-after-custom", (150, 108), [wheel(-10), click(100, 170)]),
        tip("small-screen", (100, 110), [], framebuffer=(640, 480)),
        tip("tiny-screen", (150, 70), [], framebuffer=(640, 480), gui_scale=3),
        tip("out-of-world", (100, 110), [], in_world=False),
        row("tip-late", mouse=(100, 110), steps=[frame(0), frame(300)], ms=1000),
        row("tip-moved-away", mouse=(5, 5), steps=[frame(0)], ms=10),
        row("tip-clipped-hover", mouse=(100, 30), steps=[frame(0), wheel(-2)], ms=10),
        row("tip-marquee-time", mouse=(100, 110), steps=[frame(0), wheel(-100)], ms=1234),
    ]
    yield {"case_id": "gui/video-settings-screen", "family": "gui", "operation": "video_settings_screen",
           "rows": rows}


def gui_chat_options_cases():
    """The chat settings screen: the eighteen options in a list, with the
    options' own values shown (every slider's label), the ones that hold a
    value on their widget, the narrator asleep (there is none), the list
    scrolled and its bar dragged, and sliders and cycle buttons pressed."""
    def row(label, framebuffer=(854, 480), gui_scale=2, in_world=True, mouse=(0, 0), steps=(),
            options=None, ms=0):
        row = {"label": label, "framebuffer": list(framebuffer), "gui_scale": gui_scale,
               "in_world": in_world, "mouse": list(mouse), "steps": list(steps), "ms": ms}
        if options:
            row["options"] = options
        return row

    click = lambda x, y, shift=False: {"do": "click", "x": x, "y": y, "shift": shift}
    drag = lambda x, y: {"do": "drag", "x": x, "y": y}
    release = lambda x=0, y=0: {"do": "release", "x": x, "y": y}
    wheel = lambda dy, x=200, y=100: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": False}
    opt = lambda **values: values
    rows = [
        row("in-world"), row("out-of-world", in_world=False), row("small", (640, 480)),
        row("large", (1280, 720)), row("tiny", (640, 480), 3), row("short", (854, 300)),
        row("auto-large", (1920, 1080), 0),
        row("hover-visibility", mouse=(100, 45)), row("hover-narrator-asleep", mouse=(100, 195)),
        row("hover-slider", mouse=(100, 95)), row("hover-done", mouse=(213, 223)),
        row("scroll-1", steps=[wheel(-1)]), row("scroll-5", steps=[wheel(-5)]),
        row("scroll-end", steps=[wheel(-100)]), row("scroll-end-short", (854, 300), steps=[wheel(-100)]),
        row("bar-drag", steps=[click(379, 60), drag(379, 90), release(379, 90)]),
        row("bar-drag-end", steps=[click(379, 60), drag(379, 230)]),
        row("opacity-zero", options=opt(opacity=0.0)), row("opacity-full", options=opt(opacity=1.0)),
        row("opacity-mid", options=opt(opacity=0.37)),
        row("background-zero", options=opt(background_opacity=0.0)),
        row("background-mid", options=opt(background_opacity=0.73)),
        row("scale-zero", options=opt(scale=0.0)), row("scale-mid", options=opt(scale=0.55)),
        row("spacing-full", options=opt(line_spacing=1.0)), row("spacing-mid", options=opt(line_spacing=0.25)),
        row("width-zero", options=opt(width=0.0)), row("width-mid", options=opt(width=0.5)),
        row("width-full", options=opt(width=1.0)),
        row("height-focused-low", options=opt(height_focused=0.0)),
        row("height-focused-mid", options=opt(height_focused=0.33)),
        row("height-unfocused-high", options=opt(height_unfocused=1.0)),
        row("everything", options=opt(opacity=0.2, background_opacity=0.9, scale=0.7, line_spacing=0.6,
                                       width=0.3, height_focused=0.8, height_unfocused=0.1)),
        row("press-visibility", steps=[click(100, 45)]),
        row("press-visibility-twice", steps=[click(100, 45)] * 2),
        row("press-visibility-thrice", steps=[click(100, 45)] * 3),
        row("press-visibility-shift", steps=[click(100, 45, True)]),
        row("press-colors", steps=[click(250, 45)]), row("press-links", steps=[click(100, 70)]),
        row("press-links-prompt", steps=[click(250, 70)]),
        row("press-narrator-asleep", steps=[click(100, 195)]),
        row("press-gap", steps=[click(212, 45)]),
        row("press-opacity", steps=[click(100, 95)]), row("press-opacity-end", steps=[click(205, 95)]),
        row("press-opacity-start", steps=[click(60, 95)]),
        row("drag-opacity", steps=[click(80, 95), drag(130, 95), release(130, 95)]),
        row("drag-opacity-held", steps=[click(80, 95), drag(180, 95)]),
        row("drag-opacity-past", steps=[click(80, 95), drag(400, 95), release(400, 95)]),
        row("press-background", steps=[click(300, 95)]),
        row("drag-background", steps=[click(230, 95), drag(300, 95), release(300, 95)]),
        row("press-scale", steps=[click(100, 120)]),
        row("drag-scale-zero", steps=[click(100, 120), drag(0, 120), release(0, 120)]),
        row("drag-scale-full", steps=[click(100, 120), drag(300, 120), release(300, 120)]),
        row("press-spacing", steps=[click(300, 120)]),
        row("press-delay", steps=[click(100, 145)]),
        row("drag-delay", steps=[click(70, 145), drag(120, 145), drag(160, 145), release(160, 145)]),
        row("drag-delay-end", steps=[click(70, 145), drag(300, 145), release(300, 145)]),
        row("press-width", steps=[click(300, 145)]),
        row("drag-width", steps=[click(230, 145), drag(300, 145), release(300, 145)]),
        row("drag-width-zero", steps=[click(300, 145), drag(0, 145), release(0, 145)]),
        row("press-height-focused", steps=[click(100, 170)]),
        row("drag-height-focused", steps=[click(70, 170), drag(150, 170), release(150, 170)]),
        row("press-height-unfocused", steps=[click(300, 170)]),
        row("drag-height-unfocused-zero", steps=[click(300, 170), drag(0, 170), release(0, 170)]),
        row("press-autosuggest", steps=[click(250, 195)]),
        row("scrolled-press-hide-matched", steps=[wheel(-100), click(100, 100)]),
        row("scrolled-press-reduced-debug", steps=[wheel(-100), click(250, 100)]),
        row("scrolled-press-secure", steps=[wheel(-100), click(100, 125)]),
        row("scrolled-press-drafts", steps=[wheel(-100), click(250, 125)]),
        row("scrolled-press-twice", steps=[wheel(-100), click(250, 125), click(250, 125)]),
        row("press-clipped-bottom", steps=[click(100, 206)]),
        row("press-two-in-turn", steps=[click(100, 45), click(250, 70), click(100, 95)]),
        # Tooltips of the widgets that have one.
        row("tip-hide-matched", mouse=(100, 167), steps=[{"do": "frame", "ms": 0}, wheel(-100)], ms=10),
        row("tip-hide-matched-seen", mouse=(100, 167), steps=[wheel(-100), {"do": "frame", "ms": 0}], ms=10),
        row("tip-secure", mouse=(100, 192), steps=[wheel(-100), {"do": "frame", "ms": 0}], ms=10),
        row("tip-drafts", mouse=(300, 192), steps=[wheel(-100), {"do": "frame", "ms": 0}], ms=10),
        row("tip-none-on-opacity", mouse=(100, 95), steps=[{"do": "frame", "ms": 0}], ms=10),
        row("tip-drafts-pressed", mouse=(300, 192), steps=[wheel(-100), {"do": "frame", "ms": 0}, click(300, 192)], ms=10),
    ]
    yield {"case_id": "gui/chat-options-screen", "family": "gui", "operation": "chat_options_screen",
           "rows": rows}


def gui_mouse_cases():
    """The mouse settings screen and the controls screen it opens from: their
    lists in several sizes, the sensitivity and wheel sliders at their ends
    and middle (with their special labels), the inversions, and presses and
    drags on every kind of widget. The buttons that open another screen are
    drawn and hovered, not pressed."""
    def row(label, framebuffer=(854, 480), gui_scale=2, in_world=True, mouse=(0, 0), steps=(),
            options=None, ms=0):
        row = {"label": label, "framebuffer": list(framebuffer), "gui_scale": gui_scale,
               "in_world": in_world, "mouse": list(mouse), "steps": list(steps), "ms": ms}
        if options:
            row["options"] = options
        return row

    click = lambda x, y, shift=False: {"do": "click", "x": x, "y": y, "shift": shift}
    drag = lambda x, y: {"do": "drag", "x": x, "y": y}
    release = lambda x=0, y=0: {"do": "release", "x": x, "y": y}
    wheel = lambda dy, x=200, y=100: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": False}
    opt = lambda **values: values
    mouse = [
        row("in-world"), row("out-of-world", in_world=False), row("small", (640, 480)),
        row("large", (1280, 720)), row("tiny", (640, 480), 3), row("short", (854, 240)),
        row("sensitivity-zero", options=opt(sensitivity=0.0)),
        row("sensitivity-full", options=opt(sensitivity=1.0)),
        row("sensitivity-mid", options=opt(sensitivity=0.37)),
        row("wheel-low", options=opt(wheel=0.01)), row("wheel-high", options=opt(wheel=10.0)),
        row("wheel-odd", options=opt(wheel=2.5)),
        row("inverted", options=opt(invert_x=True, invert_y=True)),
        row("inverted-x", options=opt(invert_x=True)),
        row("hover-sensitivity", mouse=(100, 45)), row("hover-done", mouse=(213, 223)),
        row("hover-gap", mouse=(212, 45)),
        row("press-sensitivity", steps=[click(100, 45)]),
        row("press-sensitivity-start", steps=[click(60, 45)]),
        row("press-sensitivity-end", steps=[click(205, 45)]),
        row("drag-sensitivity", steps=[click(80, 45), drag(150, 45), release(150, 45)]),
        row("drag-sensitivity-held", steps=[click(80, 45), drag(190, 45)]),
        row("drag-sensitivity-zero", steps=[click(100, 45), drag(0, 45), release(0, 45)]),
        row("drag-sensitivity-full", steps=[click(100, 45), drag(400, 45), release(400, 45)]),
        row("press-wheel", steps=[click(300, 45)]),
        row("drag-wheel", steps=[click(230, 45), drag(260, 45), release(260, 45)]),
        row("drag-wheel-low", steps=[click(300, 45), drag(0, 45), release(0, 45)]),
        row("drag-wheel-high", steps=[click(300, 45), drag(500, 45), release(500, 45)]),
        row("press-discrete", steps=[click(100, 70)]), row("press-discrete-twice", steps=[click(100, 70)] * 2),
        row("press-invert-x", steps=[click(250, 70)]), row("press-invert-y", steps=[click(100, 95)]),
        row("press-cursor", steps=[click(250, 95)]), row("press-shift", steps=[click(250, 70, True)]),
        row("press-gap", steps=[click(212, 70)]),
        row("scroll-short", (854, 240), steps=[wheel(-100)]),
        row("press-in-turn", steps=[click(250, 70), click(100, 95), click(100, 45)]),
        row("tip-cursor-changes", mouse=(250, 97), steps=[{"do": "frame", "ms": 0}], ms=10),
        row("tip-cursor-first", mouse=(250, 97), ms=0),
        row("tip-none-on-sensitivity", mouse=(100, 45), steps=[{"do": "frame", "ms": 0}], ms=10),
    ]
    controls = [
        row("in-world"), row("out-of-world", in_world=False), row("small", (640, 480)),
        row("large", (1280, 720)), row("short", (854, 240)), row("tiny", (640, 480), 3),
        row("hover-mouse-settings", mouse=(100, 45)), row("hover-key-binds", mouse=(300, 45)),
        row("hover-toggle", mouse=(100, 70)), row("hover-done", mouse=(213, 223)),
        row("press-toggle-sneak", steps=[click(100, 70)]),
        row("press-toggle-sneak-twice", steps=[click(100, 70)] * 2),
        row("press-toggle-sprint", steps=[click(250, 70)]),
        row("press-toggle-attack", steps=[click(100, 95)]),
        row("press-toggle-use", steps=[click(250, 95)]),
        row("press-shift-toggle", steps=[click(250, 95, True)]),
        row("press-auto-jump", steps=[click(100, 120)]),
        row("press-sprint-window", steps=[click(250, 120)]),
        row("drag-sprint-window", steps=[click(230, 120), drag(300, 120), release(300, 120)]),
        row("drag-sprint-window-zero", steps=[click(230, 120), drag(0, 120), release(0, 120)]),
        row("drag-sprint-window-end", steps=[click(230, 120), drag(400, 120), release(400, 120)]),
        row("press-operator-tab", steps=[click(100, 145)]),
        row("press-gap", steps=[click(212, 70)]),
        row("scroll-short", (854, 240), steps=[wheel(-100)]),
        row("press-in-turn", steps=[click(100, 70), click(250, 95), click(100, 145)]),
        row("tip-sprint-window", mouse=(250, 120), steps=[{"do": "frame", "ms": 0}], ms=10),
        row("tip-sprint-window-dragged", mouse=(250, 120),
            steps=[{"do": "frame", "ms": 0}, click(230, 120), drag(300, 120), release(300, 120)], ms=10),
        row("tip-none-on-buttons", mouse=(100, 45), steps=[{"do": "frame", "ms": 0}], ms=10),
    ]
    yield {"case_id": "gui/mouse-settings-screen", "family": "gui", "operation": "mouse_settings_screen",
           "rows": mouse}
    yield {"case_id": "gui/controls-screen", "family": "gui", "operation": "controls_screen",
           "rows": controls}


def latin_language_codes():
    """The languages whose names are all in Latin letters, from the asset
    pack's own list: the font's other scripts need fallbacks and the right
    to left shaping, which are not ported, so the list shows these."""
    import json
    import pathlib
    import unicodedata
    root = pathlib.Path(__file__).resolve().parents[2] / ".data/minecraft/assets"
    index = json.loads(next((root / "indexes").glob("*.json")).read_text())["objects"]
    digest = index["pack.mcmeta"]["hash"]
    packs = json.loads((root / "objects" / digest[:2] / digest).read_text())["language"]
    plain = lambda text: all(c in " ()-'.," or c.isdigit() or unicodedata.name(c, "").startswith("LATIN")
                             for c in text)
    return [code for code, info in sorted(packs.items()) if plain(info["name"] + " (" + info["region"] + ")")]


def gui_language_cases():
    """The language screen as the reference draws it, in the languages it
    lists: the title over the search box, the list centred on the language
    in use, the warning, the font button and Done; the list scrolled and its
    bar dragged, the search typed into and cleared, the entries pressed
    (the outline white while the list has the focus, the hint when the box
    has not). Done, the font button and a double click open another screen or change
    the language, which the game's reload does: those are not pressed here."""
    def row(label, language="en_us", framebuffer=(854, 480), gui_scale=2, in_world=True,
            mouse=(0, 0), steps=()):
        return {"label": label, "language": language, "framebuffer": list(framebuffer),
                "gui_scale": gui_scale, "in_world": in_world, "mouse": list(mouse), "steps": list(steps),
                "codes": codes}

    codes = latin_language_codes()
    click = lambda x, y, double=False: {"do": "click", "x": x, "y": y, "double": double}
    drag = lambda x, y: {"do": "drag", "x": x, "y": y}
    release = lambda x=0, y=0: {"do": "release", "x": x, "y": y}
    wheel = lambda dy, x=200, y=100: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": False}
    typed = lambda text: [{"do": "char", "char": c} for c in text]
    back = {"do": "key", "key": "backspace"}
    rows = [
        row("english"), row("german", "de_de"), row("turkish", "tr_tr"),
        row("out-of-world", in_world=False), row("small", framebuffer=(640, 480)),
        row("large", framebuffer=(1280, 720)), row("tiny", framebuffer=(640, 480), gui_scale=3),
        row("short", framebuffer=(854, 300)), row("auto-large", framebuffer=(1920, 1080), gui_scale=0),
        row("first-language", "af_za"), row("unknown-language", "xx_xx"),
        row("hover-done", mouse=(300, 225)), row("hover-fonts", mouse=(130, 225)),
        row("hover-entry", mouse=(213, 120)),
        row("scroll-1", steps=[wheel(-1)]), row("scroll-5", steps=[wheel(-5)]),
        row("scroll-end", steps=[wheel(-500)]), row("scroll-top", steps=[wheel(500)]),
        row("scroll-over-header", steps=[wheel(-3, 200, 10)]),
        row("bar-drag", steps=[click(356, 90), drag(356, 120), release(356, 120)]),
        row("bar-drag-end", steps=[click(356, 90), drag(356, 400)]),
        row("bar-drag-top", steps=[click(356, 90), drag(356, 0)]),
        row("press-entry", steps=[click(213, 90)]),
        row("press-entry-then-search", steps=[click(213, 90), click(200, 25)]),
        row("press-entry-twice", steps=[click(213, 90), click(213, 125)]),
        row("press-entry-clipped-bottom", steps=[click(213, 190)]),
        row("press-gap", steps=[click(20, 90)]),
        row("press-search", steps=[click(150, 25)]),
        row("type-tr", steps=typed("tr")), row("type-de", "de_de", steps=typed("de")),
        row("type-german", steps=typed("German")), row("type-capital", steps=typed("TUR")),
        row("type-nothing-matches", steps=typed("zzzz")),
        row("type-region", steps=typed("Deutschland")),
        row("type-then-clear", steps=typed("tr") + [back, back]),
        row("type-one-back", steps=typed("abc") + [back]),
        row("type-then-press-entry", steps=typed("fran") + [click(213, 55)]),
        row("type-then-scroll", steps=typed("e") + [wheel(-2)]),
        row("type-after-list-focus", steps=[click(213, 90)] + typed("x")),
        row("hint-after-list", steps=[click(213, 90)]),
        row("type-selection-lost", "tr_tr", steps=typed("e")),
        row("type-long", steps=typed("abcdefghijklmnopqrstuvwxyz")),
    ]
    yield {"case_id": "gui/language-screen", "family": "gui", "operation": "language_screen", "rows": rows}
    keys = ["gui.done", "options.language.title", "options.video", "options.language.accuracy",
            "menu.quit", "gui.cancel", "options.on", "options.off", "deathScreen.title", "options.fov",
            "not.a.key", "options.generic_value", "chat.type.text"]
    yield {"case_id": "gui/lang-text", "family": "gui", "operation": "lang_text",
           "rows": [{"label": code, "language": code, "keys": keys}
                    for code in ("en_us", "tr_tr", "de_de", "fr_fr", "ja_jp", "ar_sa", "ru_ru", "xx_xx")]}


def _waypoint_bytes(operation, ident, kind, contents, style="minecraft:default", color=None):
    """A `ClientboundTrackedWaypointPacket` body as hex: the operation, the
    identifier (a name or a UUID given as two longs), the icon, the kind
    (0 empty, 1 position, 2 chunk, 3 azimuth) and its contents."""
    import struct

    def varint(value):
        value &= 0xFFFFFFFF
        out = b""
        while True:
            byte = value & 0x7F
            value >>= 7
            out += bytes([byte | (0x80 if value else 0)])
            if not value:
                return out

    def utf(text):
        raw = text.encode()
        return varint(len(raw)) + raw

    body = varint(operation)
    if isinstance(ident, tuple):
        body += b"\x01" + struct.pack(">QQ", ident[0] & (2**64 - 1), ident[1] & (2**64 - 1))
    else:
        body += b"\x00" + utf(ident)
    body += utf(style)
    body += b"\x00" if color is None else b"\x01" + bytes(color)
    body += varint(kind)
    if kind == 1:
        body += b"".join(varint(v) for v in contents)
    elif kind == 2:
        body += b"".join(varint(v) for v in contents)
    elif kind == 3:
        body += struct.pack(">f", contents)
    return body.hex()


def gui_locator_bar_cases():
    """The locator bar: where a dot sits for each heading off the camera's
    (and where it is cut off), the sprite each distance picks, the arrows for
    a waypoint above or below, colours (set, hashed from a name, from a UUID),
    chunk and azimuth waypoints, the viewer's own waypoint left out, the
    farthest drawn first, a turned and a pitched camera, and the operations
    that move and remove waypoints.  Names identify the block waypoints and
    UUIDs the others, as the game has no entity for a UUID to name."""
    me = (0x1234567890ABCDEF, -0x1234567890ABCDEF)

    def block(name, x, y, z, **icon):
        return _waypoint_bytes(0, name, 1, (x, y, z), **icon)

    def row(label, packets, yaw=0.0, pitch=0.0, width=427, height=240, eye=(0.5, 65.62, 0.5),
            fov=70.0, aspect=16 / 9, far=768.0):
        return {"label": label, "width": width, "height": height, "packets": packets,
                "camera": {"yaw": yaw, "pitch": pitch, "pos": list(eye), "fov": fov, "aspect": aspect,
                           "far": far},
                "player": {"pos": [eye[0], eye[1] - 1.62, eye[2]], "uuid": list(me)}}

    rows = [row("empty", [])]
    # Ahead is +z at yaw 0; the bar spans sixty degrees either way.
    for blocks in (10, 60, 127, 128, 160, 200, 250, 331, 332, 400, 3000):
        rows.append(row("distance-%d" % blocks, [block("ahead", 0, 64, blocks)]))
    for degrees in (-70, -61, -60, -59, -45, -30, -10, -1, 0, 1, 10, 30, 45, 59, 60, 61, 90, 135, 180):
        radians = math.radians(degrees)
        x, z = -math.sin(radians) * 50, math.cos(radians) * 50
        rows.append(row("angle-%d" % degrees, [block("w", round(x), 64, round(z))]))
    rows.append(row("above", [block("up", 0, 160, 20)]))
    rows.append(row("below", [block("down", 0, -60, 20)]))
    rows.append(row("level", [block("level", 0, 66, 20)]))
    rows.append(row("behind-above", [block("up", 0, 160, -20)]))
    rows.append(row("behind-below", [block("down", 0, -60, -20)]))
    rows.append(row("behind-level", [block("level", 0, 66, -20)]))
    rows.append(row("overhead", [block("up", 0, 300, 1)]))
    rows.append(row("colours", [
        block("red", -20, 64, 50, color=(255, 0, 0)), block("green", 0, 64, 50, color=(0, 255, 0)),
        block("hashed-a", 20, 64, 50), block("hashed-b", -40, 64, 50), block("hashed-c", 40, 64, 50)]))
    rows.append(row("grey-and-dark", [
        block("grey", -20, 64, 50, color=(128, 128, 128)), block("black", 0, 64, 50, color=(0, 0, 0)),
        block("white", 20, 64, 50, color=(255, 255, 255))]))
    rows.append(row("bowtie", [
        block("near", -20, 64, 30, style="minecraft:bowtie"), block("mid", 0, 64, 80, style="minecraft:bowtie"),
        block("far", 20, 64, 300, style="minecraft:bowtie"), block("end", 40, 64, 500, style="minecraft:bowtie")]))
    rows.append(row("chunks", [
        _waypoint_bytes(0, (1, 1), 2, (0, 4)), _waypoint_bytes(0, (2, 2), 2, (-3, 6)),
        _waypoint_bytes(0, (3, 3), 2, (3, 6)), _waypoint_bytes(0, (4, 4), 2, (0, -5))]))
    rows.append(row("azimuths", [
        _waypoint_bytes(0, (5, 5), 3, 0.0), _waypoint_bytes(0, (6, 6), 3, 0.5),
        _waypoint_bytes(0, (7, 7), 3, -0.7), _waypoint_bytes(0, (8, 8), 3, 3.0)]))
    rows.append(row("azimuth-looking-up", [_waypoint_bytes(0, (5, 5), 3, 0.3)], pitch=-60.0))
    rows.append(row("azimuth-looking-down", [_waypoint_bytes(0, (5, 5), 3, 0.3)], pitch=60.0))
    rows.append(row("chunk-looking-up", [_waypoint_bytes(0, (1, 1), 2, (0, 4))], pitch=-80.0))
    rows.append(row("empty-waypoint", [_waypoint_bytes(0, (9, 9), 0, None)]))
    rows.append(row("own-waypoint", [_waypoint_bytes(0, me, 3, 0.0), block("other", 5, 64, 50)]))
    rows.append(row("overlap", [block("far", 0, 64, 300), block("mid", 1, 64, 150), block("near", 2, 64, 20)]))
    for yaw in (90.0, -90.0, 180.0, 370.0, -190.0, 45.5):
        rows.append(row("yaw-%g" % yaw, [block("north", 0, 64, -50), block("east", 50, 64, 0),
                                         block("west", -50, 64, 0), block("south", 0, 64, 50)], yaw=yaw))
    for pitch in (-90.0, -45.0, 45.0, 90.0):
        rows.append(row("pitch-%g" % pitch, [block("ahead", 0, 70, 30), block("low", 0, 55, 30)], pitch=pitch))
    rows.append(row("wide-fov", [block("up", 0, 100, 30)], fov=110.0))
    rows.append(row("narrow-fov", [block("up", 0, 80, 30)], fov=30.0))
    rows.append(row("far-corner", [block("a", -30000, 64, 30000)], eye=(-29990.5, 70.0, 29900.5)))
    rows.append(row("updated", [block("a", 0, 64, 50), block("b", 20, 64, 50),
                                _waypoint_bytes(2, "a", 1, (-30, 64, 50))]))
    rows.append(row("untracked", [block("a", 0, 64, 50), block("b", 20, 64, 50),
                                  _waypoint_bytes(1, "a", 1, (0, 0, 0))]))
    rows.append(row("retracked", [block("a", 0, 64, 50), block("a", 20, 64, 80)]))
    rows.append(row("small-screen", [block("a", 0, 64, 50), block("b", 30, 90, 50)], width=320, height=180))
    rows.append(row("tall-screen", [block("a", 0, 64, 50)], width=213, height=240))
    yield {"case_id": "gui/locator_bar", "family": "gui", "operation": "locator_bar", "rows": rows}


def gui_generators():
    return itertools.chain(
        gui_cases(), gui_suggestion_cases(), gui_title_cases(), gui_boss_bar_cases(),
        gui_sidebar_cases(), gui_command_parse_cases(), gui_command_suggest_cases(),
        gui_pause_screen_cases(), gui_death_screen_cases(), gui_disconnected_screen_cases(),
        gui_tab_list_cases(), gui_container_screen_cases(),
        gui_confirm_screen_cases(), gui_widgets_cases(),
        gui_options_screen_cases(), gui_video_settings_cases(), gui_chat_options_cases(),
        gui_mouse_cases(), gui_language_cases(), gui_locator_bar_cases())
