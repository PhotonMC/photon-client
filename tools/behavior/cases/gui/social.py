"""The `gui` family: the social interactions screen."""

import hashlib


def offline_id(name):
    """`UUID.nameUUIDFromBytes`: the MD5 of the name as a version 3 id."""
    digest = bytearray(hashlib.md5(name.encode()).digest())
    digest[6] = (digest[6] & 0x0F) | 0x30
    digest[8] = (digest[8] & 0x3F) | 0x80
    text = digest.hex()
    return f"{text[:8]}-{text[8:12]}-{text[12:16]}-{text[16:20]}-{text[20:]}"


def gui_social_interactions_screen_cases():
    """The player list on its three pages: online players beside hidden and departed ones, the
    search over them (a hit, a miss), the empty pages, and several screen sizes."""
    crowd = ["Alex", "bob", "_Zed", "Carol"]

    def row(label, players=crowd, gone=(), hidden=(), page="all", search="", width=427, height=240,
            mouse=(0, 0), reporting=True, local="Steve", frames=2, steps=()):
        person = lambda name: {"name": name, "id": offline_id(name)}
        return {"label": label, "players": [person(name) for name in [local] + list(players)],
                "gone": [person(name) for name in gone],
                "hidden": [offline_id(name) for name in hidden], "frames": frames,
                "local": offline_id(local), "page": page, "search": search, "width": width,
                "height": height, "mouse": list(mouse), "reporting": reporting,
                "server": "Test Server", "steps": list(steps)}

    rows = [
        row("alone", players=[]), row("crowd"), row("small", width=320), row("wide", width=854, height=480),
        row("short", height=160), row("no-reporting", reporting=False),
        row("gone", gone=["Dave", "erin"]), row("hidden-online", hidden=["Alex"]),
        row("hidden-gone", gone=["Dave"], hidden=["Dave"]),
        row("page-hidden", page="hidden", hidden=["Alex", "bob"]), row("page-hidden-empty", page="hidden"),
        row("page-blocked-empty", page="blocked"),
        row("search-hit", search="a"), row("search-miss", search="zzz"),
        row("search-miss-hidden", page="hidden", hidden=["Alex"], search="q"),
        row("hover-hide", mouse=(283, 144)), row("hover-report", mouse=(307, 144)),
        row("first-frame", frames=1), row("blocked-first-frame", page="blocked", frames=1),
        row("page-blocked-second-frame", page="blocked", frames=2),
    ]
    click = lambda x, y: {"do": "click", "x": x, "y": y}
    scroll = lambda dy: {"do": "scroll", "x": 200, "y": 120, "dy": dy}
    key = lambda code: {"do": "key", "key": code}
    typed = lambda text: {"do": "type", "text": text}
    hide_zed, hide_alex_scrolled = click(283, 144), click(283, 144)
    rows += [
        row("click-hide", steps=[hide_zed]),
        row("click-hide-show", steps=[hide_zed, hide_zed]),
        row("click-hide-hover", steps=[hide_zed], mouse=(283, 144)),
        row("click-show-page-hidden", page="hidden", hidden=["Alex", "bob"], steps=[click(283, 108)]),
        row("click-hide-scrolled", steps=[scroll(-2), hide_alex_scrolled]),
        row("click-local-row", steps=[click(283, 108)]),
        row("click-gone-row", gone=["Dave"], steps=[scroll(-6), click(283, 200)]),
        row("tab-hidden", hidden=["Alex"], steps=[click(200, 55)]),
        row("tab-blocked", steps=[click(280, 55)]),
        row("tab-back-to-all", hidden=["Alex"], page="hidden", steps=[click(120, 55)]),
        row("tab-keeps-search", search="a", hidden=["Alex"], steps=[click(200, 55)]),
        row("scroll-down", steps=[scroll(-1)]),
        row("scroll-past-bottom", steps=[scroll(-20)]), row("scroll-past-top", steps=[scroll(3)]),
        row("scroll-outside-list", steps=[{"do": "scroll", "x": 200, "y": 30, "dy": -1}]),
        row("scroll-short-list", players=[], steps=[scroll(-1)]),
        row("drag-bar", steps=[click(333, 100), {"do": "drag", "x": 333, "y": 120, "dy": 20}]),
        row("drag-bar-released", steps=[click(333, 100), {"do": "release", "x": 333, "y": 100},
                                        {"do": "drag", "x": 333, "y": 140, "dy": 20}]),
        row("drag-bar-past-end", steps=[click(333, 100), {"do": "drag", "x": 333, "y": 200, "dy": 5}]),
        row("drag-without-bar", steps=[click(200, 100), {"do": "drag", "x": 333, "y": 140, "dy": 20}]),
        row("search-unfocus", steps=[click(120, 55)]),
        row("search-unfocus-typed", steps=[click(120, 55), typed("zz")]),
        row("search-refocus", steps=[click(120, 55), click(200, 80)]),
        row("search-click-cursor", search="Carol", steps=[click(140, 80)]),
        row("search-type", steps=[typed("a")]), row("search-type-backspace", search="al", steps=[key(259)]),
        row("search-type-scrolled", steps=[scroll(-3), typed("a")]),
        row("search-type-miss", steps=[typed("zz"), key(259), key(259)]),
        row("done", steps=[click(200, 223)]),
        row("key-escape", steps=[key(256)]),
        row("key-social-in-search", steps=[key(80)]),
        row("key-social-unfocused", steps=[click(120, 55), key(80)]),
        row("key-social-list-focus", steps=[click(283, 108), key(80)]),
        row("hint-first-frame", page="blocked", frames=1, steps=[click(200, 165)]),
        row("hint-hidden-later", steps=[click(200, 165)]),
        row("click-nothing", steps=[click(5, 5)]),
    ]
    yield {"case_id": "gui/social-interactions-screen", "family": "gui",
           "operation": "social_interactions_screen", "rows": rows}
