"""The `gui` family: commands."""

import struct
from cases.helpers import *


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
    # A player who may not use selectors: the `@` is refused where it stands,
    # and names and coordinates read as before.
    for number, text in enumerate([
            "/give @p", "/give @p ", "/give @a[type=player] minecraft:stone", "/tp @p @a", "/tp @s 1 2 3",
            "/tp Alex 1 2 3", "/tp Alex", "/give Alex minecraft:stone 3", "/execute as @a run say hi",
            "/execute as Alex run say hi", "/scoreboard players set @a obj 5", "/scoreboard players set Alex obj 5",
            "/scoreboard players set * obj 5", "/give @", "/give @x", "/give @e[", "/say @a hello",
            "/give @p minecraft:stone 64 ", "/tp @p 1 2 3 4"]):
        for empty in (True, False):
            rows.append({"label": f"no-selectors-{number}-{text}-{empty}", "text": text,
                         "suggestions_empty": empty, "selectors": False})
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
    # A player who may not use selectors is offered names only, and an `@`
    # is a name that no player has.
    for i, texts in ((6, ["", "@", "@a", "@a[", "@z", "Mi", "mi", "x", "@p[type=player,"]),
                     (7, ["", "@", "Mi"]), (31, ["", "@", "Mi", "*"])):
        for text in texts:
            add(f"/t{i} {text}")
            rows[-1]["selectors"] = False
    add("/single ")
    rows[-1]["selectors"] = False
    add("/single @")
    rows[-1]["selectors"] = False
    for name in ("ask", "sounds", "bogus", "mixed"):
        add(f"/{name} ")
        add(f"/{name} a")
        if name == "sounds":
            # A provider the game lists locally (the sounds); the port asks.
            rows[-1]["registry_backed"] = rows[-2]["registry_backed"] = True
    # The entity in the crosshair is offered by an entity argument that takes
    # more than players, as its string UUID; not by the players-only ones.
    aimed = ["5e8c9a1b-0000-4000-8000-000000000001"]
    for text in ("/t6 ", "/t6 5", "/t6 5e8c", "/t6 Mi", "/t6 @", "/t6 x", "/single ", "/t7 ", "/t7 5",
                 "/t31 ", "/t31 5"):
        add(text)
        rows[-1]["selected_entities"] = aimed
    add("/t6 ")
    rows[-1]["selected_entities"] = aimed
    rows[-1]["selectors"] = False
    add("/t42 creative ")
    add("/t6 Alex ")
    add("/t42 cr", cursor=5)
    add("/t42 creative", cursor=5)
    yield {"case_id": "gui/command-suggest", "family": "gui", "operation": "command_suggest",
           "tree": tree, "rows": rows}
