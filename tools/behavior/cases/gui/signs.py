"""The sign text scenarios."""

DYES = ["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
        "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black"]


def gui_sign_text_cases():
    """What the reference's own sign renderers submit for a sign's text: standing, wall and
    hanging signs at each turn and wall, each dye, glowing text near and far, filtered lines,
    lines too wide for the sign and styled ones."""
    plain = ["Hello", "", "sign", "x"]

    def face(lines, color="black", glowing=False, filtered=None):
        return {"messages": lines, "filtered": filtered, "color": color, "glowing": glowing}

    def row(label, kind="standing", wall=None, rotation=0, front=None, back=None, filtering=False,
            draw_outline=False, light=0xF000F0 - 17):
        return {"label": label, "kind": kind, "wall": wall, "rotation": rotation,
                "front": front or face(plain), "back": back or face(["", "", "", ""]),
                "filtering": filtering, "draw_outline": draw_outline, "light": light}

    rows = []
    for kind in ("standing", "hanging"):
        for rotation in range(16):
            rows.append(row(f"{kind} floor {rotation}", kind, rotation=rotation,
                            back=face(["back", "of", "the", "sign"], "red")))
        for wall in ("north", "south", "east", "west"):
            rows.append(row(f"{kind} wall {wall}", kind, wall=wall, light=240))
    for index, dye in enumerate(DYES):
        kind = "standing" if index % 2 == 0 else "hanging"
        rows.append(row(f"dye {dye}", kind, front=face(plain, dye)))
        rows.append(row(f"dye {dye} glowing far", kind, front=face(plain, dye, True)))
        rows.append(row(f"dye {dye} glowing near", kind, front=face(plain, dye, True),
                        draw_outline=True))
    filtered = ["####", "", "clean", "x"]
    rows.append(row("written lines", front=face(["rude", "", "clean", "x"], filtered=filtered)))
    rows.append(row("filtered lines", front=face(["rude", "", "clean", "x"], filtered=filtered),
                    filtering=True))
    wide = ["The quick brown fox jumps over the lazy dog", "WWWWWWWWWWWWWWWWWWWWWW",
            "iiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii", "nospacesatallinthislongline"]
    rows.append(row("too wide", front=face(wide), back=face(wide, "blue", True)))
    rows.append(row("too wide hanging", "hanging", front=face(wide)))
    styled = [{"text": "Red", "color": "red"},
              {"text": "Bold ", "bold": True, "extra": [{"text": "and", "color": "#123456"}]},
              ["", {"text": "list", "italic": True}, " tail"],
              {"text": "Glow", "color": "gold"}]
    rows.append(row("styled", front=face(styled), back=face(styled, "white", True), draw_outline=True))
    rows.append(row("styled hanging", "hanging", wall="east", front=face(styled, "cyan", True)))
    yield {"case_id": "gui/sign-texts", "family": "gui", "operation": "sign_texts", "rows": rows}


def gui_sign_edit_screen_cases():
    """The sign editing screens: each wood, standing, on a wall and hanging, front and back; the
    lines typed until the sign is full, moved between with the arrows and Enter, the cursor in the
    text and at its end, its blink, selections by Shift and Control, words erased, the Done button
    under the mouse and focused by Tab, dyes and glowing text."""
    woods = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "pale_oak", "mangrove",
             "cherry", "bamboo", "crimson", "warped"]

    def face(lines=("", "", "", ""), color="black", glowing=False):
        return {"messages": list(lines), "filtered": None, "color": color, "glowing": glowing}

    def row(label, kind="standing", wood="oak", front=True, lines=("", "", "", ""), color="black",
            glowing=False, width=427, height=240, mouse=(0, 0), steps=(), ms=0):
        return {"label": label, "kind": kind, "wood": wood, "front": front,
                "face": face(lines, color, glowing), "width": width, "height": height,
                "mouse": list(mouse), "steps": list(steps), "ms": ms}

    type_ = lambda text: {"do": "type", "text": text}
    key = lambda name: {"do": "key", "key": name}
    written = ("Hello", "", "sign", "x")
    rows = []
    for index, wood in enumerate(woods):
        kind = ["standing", "wall", "hanging", "wall_hanging"][index % 4]
        rows.append(row(f"{wood} {kind}", kind, wood, lines=written))
    for kind in ("standing", "wall", "hanging", "wall_hanging"):
        rows.append(row(f"empty {kind}", kind))
        rows.append(row(f"back {kind}", kind, front=False, lines=written))
        rows.append(row(f"typed {kind}", kind, steps=[type_("Hi there"), key("down"), type_("two")]))
        rows.append(row(f"full {kind}", kind, steps=[type_("The quick brown fox jumps over it")]))
        rows.append(row(f"blink {kind}", kind, lines=written, ms=300))
    rows += [
        row("small", width=320, lines=written), row("wide", width=640, height=360, lines=written),
        row("tall hanging", "hanging", height=480, lines=written),
        row("up wraps", lines=written, steps=[key("up")]),
        row("enter twice", lines=written, steps=[key("enter"), key("enter")]),
        row("down to empty", lines=written, steps=[key("down")]),
        row("cursor inside", lines=written, steps=[key("left"), key("left")]),
        row("cursor home", lines=written, steps=[key("home")]),
        row("cursor inside blink", lines=written, steps=[key("left")], ms=900),
        row("select left", lines=written, steps=[key("shift+left"), key("shift+left")]),
        row("select home", lines=written, steps=[key("left"), key("shift+home")]),
        row("select all", lines=("one two three", "", "", ""), steps=[key("ctrl+a")]),
        row("select all typed", lines=("one two three", "", "", ""), steps=[key("ctrl+a"), type_("new")]),
        row("word left", lines=("one two three", "", "", ""), steps=[key("ctrl+left"), type_("X")]),
        row("word right", lines=("one two three", "", "", ""),
            steps=[key("home"), key("ctrl+right"), type_("X")]),
        row("word erased", lines=("one two three", "", "", ""), steps=[key("ctrl+backspace")]),
        row("backspace", lines=written, steps=[key("backspace"), key("backspace")]),
        row("delete inside", lines=written, steps=[key("home"), key("delete"), key("end")]),
        row("selection erased", lines=written, steps=[key("shift+left"), key("shift+left"), key("backspace")]),
        row("hover done", lines=written, mouse=(213, 215)),
        row("tab to done", lines=written, steps=[key("tab")]),
        row("red", lines=written, color="red"), row("red glowing", lines=written, color="red", glowing=True),
        row("black glowing", lines=written, glowing=True),
        row("white hanging", "hanging", lines=written, color="white"),
        row("section sign refused", steps=[type_("a§b")]),
    ]
    yield {"case_id": "gui/sign-edit-screen", "family": "gui", "operation": "sign_edit_screen",
           "rows": rows}
