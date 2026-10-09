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
            mouse=(0, 0), reporting=True, local="Steve", frames=2):
        person = lambda name: {"name": name, "id": offline_id(name)}
        return {"label": label, "players": [person(name) for name in [local] + list(players)],
                "gone": [person(name) for name in gone],
                "hidden": [offline_id(name) for name in hidden], "frames": frames,
                "local": offline_id(local), "page": page, "search": search, "width": width,
                "height": height, "mouse": list(mouse), "reporting": reporting,
                "server": "Test Server"}

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
    yield {"case_id": "gui/social-interactions-screen", "family": "gui",
           "operation": "social_interactions_screen", "rows": rows}
