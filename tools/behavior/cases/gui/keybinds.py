"""The key binds screen (`KeyBindsScreen`): its list of categories and key mappings."""


def gui_key_binds_cases():
    """The key binds screen in several sizes, scrolled, with keys bound away from their defaults
    and onto each other, a mapping waiting for its key and given one (a key, Escape, a mouse
    button), the row and footer resets, hovers with the tooltip of a clash, and the keyboard."""
    def row(label, framebuffer=(854, 480), gui_scale=2, in_world=True, mouse=(0, 0), steps=(),
            bindings=None, ms=0):
        row = {"label": label, "framebuffer": list(framebuffer), "gui_scale": gui_scale,
               "in_world": in_world, "mouse": list(mouse), "steps": list(steps), "ms": ms}
        if bindings:
            row["bindings"] = bindings
        return row

    click = lambda x, y, button=0: {"do": "click", "x": x, "y": y, "button": button}
    key = lambda name: {"do": "key", "key": name}
    wheel = lambda dy, x=200, y=100: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": False}
    frame = {"do": "frame", "ms": 0}
    # At 854x480 and scale 2 the second entry's change button is at 251..326, its reset at
    # 331..381, both 55..75; the footer's Reset Keys at 59..209 and Done at 217..367, 213..233.
    clash = {"key.jump": "key.keyboard.w"}
    rows = [
        row("in-world"), row("out-of-world", in_world=False), row("small", (640, 480)),
        row("large", (1280, 720)), row("short", (854, 240)), row("tiny", (640, 480), 3),
        row("scrolled", steps=[wheel(-3)]), row("scrolled-end", steps=[wheel(-100)]),
        row("rebound", bindings={"key.forward": "key.keyboard.up", "key.attack": "key.mouse.4"}),
        row("unbound", bindings={"key.forward": "key.keyboard.unknown"}),
        row("clash", bindings=clash),
        row("clash-three", bindings={"key.jump": "key.keyboard.w", "key.sneak": "key.keyboard.w"}),
        row("clash-mouse", bindings={"key.pickItem": "key.mouse.left"}),
        row("hover-change", mouse=(280, 65)), row("hover-reset", mouse=(350, 65)),
        row("hover-reset-active", mouse=(350, 65), bindings={"key.forward": "key.keyboard.up"}),
        row("hover-category", mouse=(213, 40)), row("hover-reset-keys", mouse=(130, 222)),
        row("tip-clash", mouse=(280, 65), bindings=clash, steps=[frame], ms=10),
        row("select", steps=[click(280, 65)]),
        row("select-twice", steps=[click(280, 65), click(280, 65)]),
        row("assign-key", steps=[click(280, 65), key("code:70")]),
        row("assign-escape", steps=[click(280, 65), key("escape")]),
        row("assign-mouse", steps=[click(280, 65), click(10, 10, 1)]),
        row("assign-clash", steps=[click(280, 65), key("code:65")]),
        row("assign-same", steps=[click(280, 65), key("code:87")]),
        row("reset-row", bindings={"key.forward": "key.keyboard.up"}, steps=[click(350, 65)]),
        row("reset-all", bindings={"key.forward": "key.keyboard.up", "key.jump": "key.keyboard.w"},
            steps=[click(130, 222)]),
        row("tab-first", steps=[key("tab")]), row("tab-second", steps=[key("tab"), key("tab")]),
        row("tab-back", steps=[key("shift+tab")]),
        row("arrow-down", steps=[key("tab"), key("down"), key("down")]),
        row("arrow-right", steps=[key("tab"), key("down"), key("right")]),
        row("arrow-into-footer", (854, 240), steps=[key("shift+tab"), key("up")]),
        row("tab-select", steps=[key("tab"), key("tab"), key("enter"), key("code:71")]),
    ]
    yield {"case_id": "gui/key-binds-screen", "family": "gui", "operation": "key_binds_screen",
           "rows": rows}
