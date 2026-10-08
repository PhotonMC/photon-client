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
