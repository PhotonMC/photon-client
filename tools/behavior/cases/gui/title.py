"""The title screen scenarios."""


def gui_title_screen_cases():
    """The title screen in screens of several sizes, with the splash at several moments of its
    beat, the mouse over each button, the icons' tooltips and the keyboard focus."""
    def row(label, width=427, height=240, mouse=(0, 0), steps=(), ms=0, splash="Hello world!", hide=False):
        return {"label": label, "width": width, "height": height, "mouse": list(mouse),
                "steps": list(steps), "ms": ms, "splash": splash, "hide_splash": hide}

    frame = lambda ms: {"do": "frame", "ms": ms}
    key = lambda name: {"do": "key", "key": name}
    rows = [row("default"), row("small", 320, 240), row("wide", 640, 360), row("tall", 427, 480),
            row("splash-hidden", hide=True), row("long-splash", splash="Absolutely no memes!"),
            row("beat-quarter", ms=250), row("beat-half", ms=500), row("beat-later", ms=12_345)]
    buttons = {"singleplayer": (213, 115), "multiplayer": (213, 140), "realms": (213, 165),
               "friends": (189, 190), "language": (213, 190), "accessibility": (237, 190),
               "options": (160, 210), "quit": (265, 210), "copyright": (400, 233)}
    for name, point in buttons.items():
        rows.append(row("hover-" + name, mouse=point))
    for name in ("friends", "language", "accessibility"):
        rows.append(row("tip-" + name, mouse=buttons[name], steps=[frame(0)], ms=10))
    rows += [row("tab-%d" % count, steps=[key("tab")] * count) for count in (1, 4, 9)]
    rows += [
        row("tab-icon-tip", steps=[key("tab")] * 5 + [frame(0)], ms=10),
        row("shift-tab", steps=[key("shift+tab")] * 2),
        row("arrow-down", steps=[key("down")] * 3),
        row("arrow-icons", steps=[key("down")] * 4 + [key("right")]),
        row("arrow-to-quit", steps=[key("down")] * 5 + [key("right")]),
    ]
    yield {"case_id": "gui/title-screen", "family": "gui", "operation": "title_screen", "rows": rows}


def gui_join_multiplayer_screen_cases():
    """The server list: saved servers being asked about (the bars turn with the clock), the local
    network's header, the mouse over a server and its join and move arrows, the status tooltip,
    clicks that select or move a server, scrolling a long list, and several screen sizes."""
    three = ["Alpha", "Beta Server", "Gamma"]

    def row(label, servers=three, width=427, height=240, mouse=(0, 0), steps=(), ms=0):
        return {"label": label, "servers": list(servers), "width": width, "height": height,
                "mouse": list(mouse), "steps": list(steps), "ms": ms}

    click = lambda x, y: {"do": "click", "x": x, "y": y}
    scroll = lambda dy: {"do": "scroll", "x": 213, "y": 100, "dy": dy}
    many = ["Server %d" % index for index in range(1, 7)]
    rows = [row("three"), row("none", servers=[]), row("one", servers=["Only"]),
            row("small", width=320), row("wide", width=640, height=360), row("tall", height=480),
            row("long-name", servers=["A server with a rather long name for its row"])]
    rows += [row("bars-%d" % ms, ms=ms) for ms in (100, 250, 450, 700, 1234)]
    rows += [
        row("hover-text", mouse=(200, 80)), row("hover-join", mouse=(90, 80)),
        row("hover-move-up", mouse=(70, 75)), row("hover-move-down", mouse=(70, 95)),
        row("hover-first-no-up", mouse=(70, 40)), row("hover-last-no-down", mouse=(70, 125)),
        row("hover-lan-header", mouse=(213, 150)), row("tip-status", mouse=(352, 40)),
        row("hover-back", mouse=(323, 222)),
        row("select-second", steps=[click(200, 80)]),
        row("select-then-hover-delete", mouse=(161, 222), steps=[click(200, 80)]),
        row("select-lan-header", steps=[click(213, 150)]),
        row("move-first-down", steps=[click(70, 57)]),
        row("move-last-up", steps=[click(70, 110)]),
        row("many", servers=many),
        row("many-scrolled", servers=many, steps=[scroll(-1)]),
        row("many-scrolled-end", servers=many, steps=[scroll(-10)]),
        row("many-scrolled-back", servers=many, steps=[scroll(-3), scroll(1)]),
    ]
    yield {"case_id": "gui/join-multiplayer-screen", "family": "gui",
           "operation": "join_multiplayer_screen", "rows": rows}


def gui_server_form_cases():
    """The server list's forms: adding (an empty name shows its hint once the box loses the
    focus), editing, joining directly; text typed and erased, Tab through the widgets, the
    resource pack choice cycled, Done asleep for an address the resolver refuses."""
    def row(label, kind="add", name="", ip="", pack="PROMPT", width=427, height=240, mouse=(0, 0),
            steps=()):
        return {"label": label, "kind": kind, "name": name, "ip": ip, "pack": pack, "width": width,
                "height": height, "mouse": list(mouse), "steps": list(steps)}

    type_ = lambda text: {"do": "type", "text": text}
    key = lambda name: {"do": "key", "key": name}
    click = lambda x, y: {"do": "click", "x": x, "y": y}
    rows = [
        row("add-empty"), row("add-small", width=320), row("add-wide", width=640, height=360),
        row("add-typed-name", steps=[type_("My Server")]),
        row("add-tab-to-address", steps=[key("tab")]),
        row("add-typed-both", steps=[type_("Home"), key("tab"), type_("localhost:25600")]),
        row("add-bad-port", steps=[key("tab"), type_("host:99999")]),
        row("add-erased", steps=[type_("abc"), key("backspace"), key("backspace")]),
        row("add-tab-to-pack", steps=[key("tab"), key("tab")]),
        row("add-pack-cycled", steps=[key("tab"), key("tab"), key("space")]),
        row("add-pack-clicked", steps=[click(213, 135)]),
        row("add-click-address", steps=[click(250, 115)]),
        row("add-hover-cancel", mouse=(213, 205)),
        row("edit", kind="edit", name="Old Name", ip="old.example:25565", pack="ENABLED"),
        row("edit-long-address", kind="edit", name="N",
            ip="a.very.long.server.address.that.does.not.fit.in.the.box.example.com"),
        row("edit-tab-to-done", kind="edit", name="N", ip="x.example",
            steps=[key("tab"), key("tab"), key("tab")]),
        row("direct-empty", kind="direct"),
        row("direct-last", kind="direct", ip="127.0.0.1:25600"),
        row("direct-typed", kind="direct", steps=[type_("example.org")]),
        row("direct-tab", kind="direct", ip="example.org", steps=[key("tab")]),
        row("direct-small", kind="direct", width=320, ip="example.org"),
    ]
    yield {"case_id": "gui/server-form", "family": "gui", "operation": "server_form", "rows": rows}


def gui_connect_screen_cases():
    """The screens of a join: connecting with each status the handshake shows, the mouse on
    Cancel, and loading the terrain, in several screen sizes."""
    def row(label, screen="connect", status="connect.connecting", width=427, height=240, mouse=(0, 0)):
        return {"label": label, "screen": screen, "status": status, "width": width, "height": height,
                "mouse": list(mouse)}

    rows = [row("connecting"), row("joining", status="connect.joining"),
            row("negotiating", status="connect.negotiating"), row("connecting-small", width=320),
            row("connecting-wide", width=640, height=360), row("hover-cancel", mouse=(213, 200)),
            row("loading", screen="loading"), row("loading-small", screen="loading", width=320),
            row("loading-tall", screen="loading", height=480)]
    yield {"case_id": "gui/connect-screen", "family": "gui", "operation": "connect_screen", "rows": rows}


def gui_server_list_ping_cases():
    """The server list after its pings: the answer's description in up to two lines, the player
    count or the red version of an incompatible server, the bars of the ping's time, a server the
    answer leaves fields out of, servers that could not be reached, and the tooltips of the bars
    and of the player count."""
    def answer(description, online=2, max_=20, sample=(), version="26.2", protocol=776, players=True):
        status = {"description": description, "version": {"name": version, "protocol": protocol}}
        if players:
            status["players"] = {"max": max_, "online": online,
                                 "sample": [{"name": name, "id": "00000000-0000-0000-0000-00000000000%d" % i}
                                            for i, name in enumerate(sample, 1)]}
        return status

    def ping(status, ms=40):
        import json
        return {"json": json.dumps(status), "ping": ms}

    def row(label, pings, mouse=(0, 0)):
        return {"label": label, "servers": ["Alpha", "Beta Server", "Gamma"][:len(pings)],
                "pings": pings, "width": 427, "height": 240, "mouse": list(mouse), "steps": [], "ms": 0}

    ok = ping(answer("A Minecraft Server", sample=["Alex", "Steve"]))
    rows = [
        row("answered", [ok]),
        row("ping-times", [ping(answer("a"), 149), ping(answer("b"), 299), ping(answer("c"), 1000)]),
        row("ping-mid", [ping(answer("a"), 450), ping(answer("b"), 999)]),
        row("styled-motd", [ping(answer({"text": "Red ", "color": "red", "extra": [{"text": "bold", "bold": True}]}))]),
        row("two-line-motd", [ping(answer("A rather long description that goes on and on past the width of one line, "
                                          "and then a third line that is not shown at all, no matter how long"))]),
        row("incompatible", [ping(answer("old", version="1.8.9", protocol=47))]),
        row("no-players", [ping(answer("quiet", players=False))]),
        row("no-version", [ping({"description": "x", "players": {"max": 1, "online": 0}})]),
        row("unreachable", [{"unreachable": True, "unknown_host": False},
                            {"unreachable": True, "unknown_host": True}]),
        row("mixed", [ok, None, {"unreachable": True, "unknown_host": False}]),
        row("tip-ping", [ok], mouse=(352, 40)),
        row("tip-players", [ping(answer("x", online=5, sample=["Alex", "Steve"]))], mouse=(320, 40)),
        row("tip-incompatible", [ping(answer("old", version="1.8.9", protocol=47))], mouse=(352, 40)),
        row("tip-unreachable", [{"unreachable": True, "unknown_host": False}], mouse=(352, 40)),
    ]
    yield {"case_id": "gui/server-list-pings", "family": "gui", "operation": "join_multiplayer_screen",
           "rows": rows}
