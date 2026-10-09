"""The `gui` family: the credits screen."""


def gui_credits_screen_cases():
    """The credits roll and the end poem: scrolled, the speed keys (Space, Ctrl,
    Up) pressed and released, and a roll that has run past its end."""
    def key(code, down=True):
        return {"code": code, "down": down}

    space, ctrl_left, ctrl_right, up = 32, 341, 345, 265

    def row(label, poem=False, width=427, height=240, scroll=0.0, keys=(), partial=0.0, player="Steve"):
        return {"label": label, "poem": poem, "width": width, "height": height, "scroll": scroll,
                "keys": list(keys), "partial": partial, "player": player}

    rows = []
    for poem in (False, True):
        tag = "poem" if poem else "credits"
        rows += [
            row(f"{tag}-start", poem), row(f"{tag}-early", poem, scroll=60.0),
            row(f"{tag}-middle", poem, scroll=900.5), row(f"{tag}-deep", poem, scroll=2500.0),
            row(f"{tag}-small", poem, 320, 240, scroll=700.0), row(f"{tag}-wide", poem, 854, 480, scroll=700.0),
            row(f"{tag}-space", poem, scroll=300.0, keys=[key(space)], partial=1.0),
            row(f"{tag}-space-ctrl", poem, scroll=300.0, keys=[key(space), key(ctrl_left)], partial=1.0),
            row(f"{tag}-both-ctrl", poem, scroll=300.0, keys=[key(space), key(ctrl_left), key(ctrl_right)],
                partial=1.0),
            row(f"{tag}-ctrl-alone", poem, scroll=300.0, keys=[key(ctrl_left)], partial=1.0),
            row(f"{tag}-up", poem, scroll=300.0, keys=[key(up)], partial=1.0),
            row(f"{tag}-up-space", poem, scroll=300.0, keys=[key(up), key(space)], partial=2.0),
            row(f"{tag}-released", poem, scroll=300.0,
                keys=[key(space), key(ctrl_left), key(space, False), key(ctrl_left, False)], partial=1.0),
            row(f"{tag}-up-released", poem, scroll=300.0, keys=[key(up), key(up, False)], partial=1.0),
            row(f"{tag}-up-at-top", poem, scroll=0.0, keys=[key(up)], partial=1.0),
            row(f"{tag}-fraction", poem, scroll=10.0, partial=0.37),
            row(f"{tag}-past-end", poem, scroll=1.0e6),
        ]
    rows.append(row("poem-name", True, scroll=400.0, player="Alexandra_the_Great"))
    yield {"case_id": "gui/credits-screen", "family": "gui", "operation": "credits_screen", "rows": rows}
