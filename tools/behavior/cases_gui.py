"""GUI family corpus: original synthetic screen scenarios; expected values come from the Java JAR."""

import itertools
import struct


def gui_cases():
    """The screens' drawing as the reference's own GUI code records it: the
    rectangles and the text, in order, with their poses applied. The glyphs
    are not drawn; their advances come from the reference's font providers."""
    def chat(label, messages, tick, width=427, height=240, focused=False, scroll=None,
             extra=(), deletions=()):
        row = {"label": label, "width": width, "height": height,
               "messages": [{"text": text, "tick": at} for text, at in messages] + list(extra),
               "tick": tick}
        if deletions:
            row["deletions"] = [{"delete": sig, "tick": at} for sig, at in deletions]
        if focused:
            row["focused"] = True
        if scroll is not None:
            row["scroll"] = scroll
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
    def row(label, width, height, mouse=(0, 0)):
        return {"label": label, "width": width, "height": height, "mouse": list(mouse)}

    rows = [row("default", 427, 240), row("small", 320, 240), row("wide", 640, 360), row("tiny", 200, 150)]
    for name, point in {"return": (213, 60), "advancements": (160, 90), "statistics": (265, 90),
                        "report": (170, 120), "feedback": (193, 120), "friends": (215, 120),
                        "reporting": (237, 120), "options": (213, 150), "disconnect": (213, 180)}.items():
        rows.append(row("hover-" + name, 427, 240, point))
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


def gui_generators():
    return itertools.chain(
        gui_cases(), gui_suggestion_cases(), gui_title_cases(), gui_boss_bar_cases(),
        gui_sidebar_cases(), gui_command_parse_cases(), gui_command_suggest_cases(),
        gui_pause_screen_cases(), gui_death_screen_cases(), gui_disconnected_screen_cases(),
        gui_tab_list_cases(), gui_container_screen_cases())
