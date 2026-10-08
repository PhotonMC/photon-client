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
