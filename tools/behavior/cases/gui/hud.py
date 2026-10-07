"""The `gui` family: hud."""

import math
import struct
from cases.helpers import *


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


def gui_hud_hotbar_cases():
    """The crosshair, the attack indicator under it or beside the hotbar, and the hotbar, for a
    real local player: the option's three values, the attack strength from none to full at the
    default attack speed and a slow weapon's (whose delay is long enough to show the full
    indicator), what the crosshair is on, the game mode and the camera, the selected slot, and
    several window sizes."""
    def row(label, width=427, height=240, indicator=1, ticks=0, speed=4.0, target="none",
            spectator=False, third_person=False, slot=0):
        return {"label": label, "width": width, "height": height, "attack_indicator": indicator,
                "attack_ticks": ticks, "attack_speed": speed, "target": target,
                "spectator": spectator, "third_person": third_person, "selected_slot": slot}

    rows = []
    for indicator, name in ((0, "off"), (1, "crosshair"), (2, "hotbar")):
        for ticks in (0, 1, 2, 3, 4, 5, 9):
            rows.append(row(f"{name}-fast-{ticks}", indicator=indicator, ticks=ticks))
        for ticks in (0, 1, 6, 12, 13):
            rows.append(row(f"{name}-sword-{ticks}", indicator=indicator, ticks=ticks, speed=1.6))
        for target in ("living", "dead", "item"):
            for ticks, speed in ((20, 1.6), (6, 1.6), (5, 4.0), (5, 3.9), (6, 3.9)):
                rows.append(row(f"{name}-{target}-{ticks}-{speed}", indicator=indicator,
                                ticks=ticks, speed=speed, target=target))
    rows += [
        row("slot-0"), row("slot-4", slot=4), row("slot-8", slot=8),
        row("small", 320, 240), row("wide", 1000, 500), row("odd", 641, 361),
        row("tiny", 213, 120), row("hotbar-odd", 641, 361, indicator=2, ticks=2, speed=1.6),
        row("third-person", third_person=True, ticks=2, speed=1.6),
        row("third-person-hotbar", third_person=True, indicator=2, ticks=2, speed=1.6),
        row("spectator", spectator=True, ticks=2, speed=1.6),
        row("spectator-hotbar", spectator=True, indicator=2, ticks=2, speed=1.6),
        row("slow-speed", speed=0.5, ticks=20), row("fast-speed", speed=20.0, ticks=0),
        row("hotbar-slow-speed", indicator=2, speed=0.5, ticks=20),
    ]
    yield {"case_id": "gui/hud-hotbar", "family": "gui", "operation": "hud_hotbar", "rows": rows}


def gui_hud_status_cases():
    """The status bars of a real local player: the hearts (health from half a heart to forty hearts,
    absorption, hardcore, the poisoned, withered and frozen kinds, regeneration's hop, the shake at low
    health), the armor, the food (with the shake at no saturation, and hunger), the air bubbles (under water
    and after, popping and emptied), the experience bar and its level, several window sizes, and frames
    over time: the hearts blinking as health falls or rises, and the displayed health catching up."""
    def frame(tick=0, ms=0, health=20.0, invulnerable=0):
        return {"tick": tick, "ms": ms, "health": float_bits(health), "invulnerable": invulnerable}

    def row(label, frames=None, width=427, height=240, max_health=20.0, absorption=0.0, armor=0.0, food=20,
            saturation=5.0, air=300, eye_in_water=False, effects=(), frozen_ticks=0, hardcore=False,
            xp_level=0, xp_progress=0.0):
        return {"label": label, "width": width, "height": height, "hardcore": hardcore,
                "max_health": max_health, "absorption": float_bits(absorption), "armor": armor,
                "food": food, "saturation": float_bits(saturation), "air": air,
                "eye_in_water": eye_in_water, "effects": list(effects), "frozen_ticks": frozen_ticks,
                "xp_level": xp_level, "xp_progress": float_bits(xp_progress),
                "frames": frames or [frame()]}

    rows = [row("full"), row("small", width=320, height=240), row("wide", width=1000, height=500),
            row("odd", width=641, height=361)]
    for health in (0.0, 0.5, 1.0, 3.0, 4.0, 4.5, 5.0, 10.0, 10.5, 19.0, 19.5, 20.0):
        rows.append(row(f"health-{health}", [frame(7, 0, health)]))
    for max_health in (2.0, 10.0, 30.0, 40.0, 60.0, 100.0, 200.0):
        rows.append(row(f"max-{max_health}", [frame(3, 0, max_health)], max_health=max_health))
        rows.append(row(f"max-{max_health}-half", [frame(3, 0, max_health / 2 + 0.5)], max_health=max_health))
    for absorption in (0.5, 1.0, 4.0, 10.0, 20.0, 25.0, 40.0):
        rows.append(row(f"absorption-{absorption}", [frame(2, 0, 20.0)], absorption=absorption))
        rows.append(row(f"absorption-{absorption}-hurt", [frame(2, 0, 7.0)], absorption=absorption))
    for armor in (0.0, 1.0, 2.0, 7.0, 19.0, 20.0, 30.0):
        rows.append(row(f"armor-{armor}", armor=armor))
    rows.append(row("armor-over-rows", [frame(1, 0, 40.0)], armor=20.0, max_health=60.0))
    for food in (0, 1, 2, 9, 10, 19, 20):
        rows.append(row(f"food-{food}", food=food))
    for tick in (0, 1, 5, 6, 13):
        for food in (0, 4, 20):
            rows.append(row(f"starving-{food}-{tick}", [frame(tick, 0, 20.0)], food=food, saturation=0.0))
    rows.append(row("hunger", effects=["hunger"], food=11))
    rows.append(row("hunger-starving", [frame(0, 0, 20.0)], effects=["hunger"], food=0, saturation=0.0))
    for air in (300, 299, 290, 250, 150, 31, 30, 1, 0, -20):
        rows.append(row(f"air-{air}", [frame(1, 0, 20.0)], air=air, eye_in_water=True))
        rows.append(row(f"air-{air}-out", [frame(1, 0, 20.0)], air=air))
    for tick in (0, 1, 2, 3):
        rows.append(row(f"drowned-{tick}", [frame(tick, 0, 20.0)], air=0, eye_in_water=True))
    rows.append(row("in-water-full", eye_in_water=True))
    rows.append(row("air-with-armor", [frame(1, 0, 40.0)], air=100, eye_in_water=True, armor=10.0,
                    max_health=40.0, food=20))
    for effect in ("poison", "wither", "regeneration"):
        for tick in (0, 3, 9, 24):
            rows.append(row(f"{effect}-{tick}", [frame(tick, 0, 13.0)], effects=[effect]))
        rows.append(row(f"{effect}-absorption", [frame(4, 0, 13.0)], effects=[effect], absorption=6.0))
        rows.append(row(f"{effect}-hardcore", [frame(4, 0, 13.0)], effects=[effect], hardcore=True))
    rows.append(row("regeneration-big", [frame(31, 0, 30.0)], effects=["regeneration"], max_health=40.0))
    rows.append(row("poison-wither", effects=["poison", "wither"]))
    rows.append(row("frozen-short", [frame(0, 0, 13.0)], frozen_ticks=139))
    rows.append(row("frozen", [frame(0, 0, 13.0)], frozen_ticks=140))
    rows.append(row("frozen-poisoned", [frame(0, 0, 13.0)], frozen_ticks=200, effects=["poison"]))
    rows.append(row("hardcore", [frame(0, 0, 13.0)], hardcore=True))
    rows.append(row("hardcore-absorption", [frame(0, 0, 13.0)], hardcore=True, absorption=5.0))
    for level, progress in ((0, 0.0), (0, 0.5), (1, 0.0), (1, 0.01), (5, 0.5), (9, 0.99), (10, 1.0),
                            (30, 0.37), (99, 0.5), (100, 0.25), (1000, 0.9), (0, 0.001), (3, 0.0054)):
        rows.append(row(f"xp-{level}-{progress}", xp_level=level, xp_progress=progress))
    # Over time.
    ms = lambda tick: tick * 50
    rows.append(row("hurt", [frame(0, 0, 20.0), frame(1, 50, 14.0, 20)]
                    + [frame(t, ms(t), 14.0, max(0, 20 - (t - 1))) for t in range(2, 26)]))
    rows.append(row("healed", [frame(0, 0, 10.0), frame(1, 50, 16.0, 20)]
                    + [frame(t, ms(t), 16.0, max(0, 20 - (t - 1))) for t in range(2, 14)]))
    rows.append(row("hurt-twice", [frame(0, 0, 20.0), frame(1, 50, 14.0, 20), frame(5, 250, 11.0, 15),
                                   frame(6, 300, 11.0, 14), frame(14, 700, 11.0, 6), frame(30, 1500, 11.0, 0),
                                   frame(31, 1550, 11.0, 0)]))
    rows.append(row("hurt-no-immunity", [frame(0, 0, 20.0), frame(1, 50, 14.0, 0), frame(2, 100, 14.0, 0)]))
    rows.append(row("catches-up", [frame(0, 0, 20.0), frame(1, 50, 8.0, 20), frame(10, 500, 8.0, 11),
                                   frame(20, 1000, 8.0, 1), frame(21, 1100, 8.0, 0), frame(22, 1200, 8.0, 0),
                                   frame(23, 2300, 8.0, 0)]))
    rows.append(row("hurt-with-absorption", [frame(0, 0, 20.0), frame(1, 50, 14.0, 20), frame(2, 100, 14.0, 19),
                                             frame(3, 150, 14.0, 18), frame(4, 200, 14.0, 17)],
                    absorption=8.0))
    rows.append(row("hurt-big", [frame(0, 0, 40.0), frame(1, 50, 25.0, 20), frame(2, 100, 25.0, 19),
                                 frame(3, 150, 25.0, 18)], max_health=40.0))
    rows.append(row("hurt-low", [frame(0, 0, 6.0), frame(1, 50, 2.0, 20), frame(2, 100, 2.0, 19),
                                 frame(3, 150, 2.0, 18), frame(4, 200, 2.0, 17)]))
    rows.append(row("hurt-regeneration", [frame(0, 0, 20.0), frame(1, 50, 14.0, 20), frame(2, 100, 14.0, 19)],
                    effects=["regeneration"]))
    rows.append(row("hurt-hardcore", [frame(0, 0, 20.0), frame(1, 50, 14.0, 20), frame(2, 100, 14.0, 19)],
                    hardcore=True))
    yield {"case_id": "gui/hud-status", "family": "gui", "operation": "hud_status", "rows": rows}


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


def gui_name_tag_cases():
    """A remote player's name tag as the reference's own renderer shows and
    submits it (`AvatarRenderer`, `SubmitNodeCollection.submitNameTag`): the
    rules (distance, crouching, invisible, spectating, riding, the HUD) and
    the pose of the text for cameras that look in different directions."""

    def row(label, name="Steve", position=(10.5, 64.0, 5.5), y_rot=0.0, crouching=False,
            shift=False, invisible=False, vehicle=False, camera=(0.5, 65.62, 0.5),
            camera_rot=(0.0, 0.0), spectator=False, hud_hidden=False, light=15728880,
            chat_only=True, opacity=0.25):
        return {
            "label": label, "blocks": [], "chunk_radius": 0, "name": name,
            "x": float_bits(position[0], "f64"), "y": float_bits(position[1], "f64"),
            "z": float_bits(position[2], "f64"), "y_rot": float_bits(y_rot),
            "crouching": crouching, "shift_key_down": shift, "invisible": invisible,
            "vehicle": vehicle, "camera_x": float_bits(camera[0], "f64"),
            "camera_y": float_bits(camera[1], "f64"), "camera_z": float_bits(camera[2], "f64"),
            "camera_y_rot": float_bits(camera_rot[0]), "camera_x_rot": float_bits(camera_rot[1]),
            "local_spectator": spectator, "hud_hidden": hud_hidden, "light": light,
            "background_for_chat_only": chat_only, "background_opacity": float_bits(opacity),
        }

    rows = [row("standing")]
    for label, yaw, pitch in [("look-south", 0.0, 0.0), ("look-east", -90.0, 0.0),
                              ("look-north-down", 180.0, 45.0), ("look-up-left", 37.5, -60.0),
                              ("odd-angles", 123.456, 17.25), ("past-a-turn", 400.0, -10.0)]:
        rows.append(row(label, camera_rot=(yaw, pitch)))
    for distance in (1.0, 20.0, 31.9, 32.1, 40.0, 63.9, 64.1):
        at = (0.5 + distance, 64.0, 0.5)
        rows.append(row(f"standing-{distance}", position=at))
        rows.append(row(f"sneaking-{distance}", position=at, shift=True))
    rows += [
        row("crouching-pose", crouching=True, shift=True),
        row("crouching-pose-no-key", crouching=True),
        row("invisible", invisible=True),
        row("invisible-spectating", invisible=True, spectator=True),
        row("spectating", spectator=True),
        row("riding", vehicle=True),
        row("hud-hidden", hud_hidden=True),
        row("deadmau5", name="deadmau5"),
        row("wide-name", name="WWWWWWWWWWWWWWWW"),
        row("short-name", name="I"),
        row("dark-cave", light=0),
        row("lit-by-torch", light=15 << 4),
        row("sky-lit", light=15 << 20),
        row("opacity-from-options", chat_only=False, opacity=0.5),
        row("opacity-zero", chat_only=False, opacity=0.0),
        row("opacity-full", chat_only=False, opacity=1.0),
        row("facing-away", y_rot=180.0),
        row("turned", y_rot=-73.5),
        row("below-the-camera", position=(10.5, 40.0, 5.5)),
        row("far-below-spectating-sneak", position=(30.0, 50.0, 30.0), shift=True, spectator=True),
    ]
    yield {
        "case_id": "gui/name-tags",
        "family": "gui",
        "operation": "name_tags",
        "rows": rows,
    }
