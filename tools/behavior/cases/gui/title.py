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
