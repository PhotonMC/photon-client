"""The `gui` family: language."""

from cases.helpers import *


def latin_language_codes():
    """The languages whose names are all in Latin letters, from the asset
    pack's own list: the font's other scripts need fallbacks and the right
    to left shaping, which are not ported, so the list shows these."""
    import json
    import pathlib
    import unicodedata
    root = pathlib.Path(__file__).resolve().parents[4] / ".data/minecraft/assets"
    index = json.loads(next((root / "indexes").glob("*.json")).read_text())["objects"]
    digest = index["pack.mcmeta"]["hash"]
    packs = json.loads((root / "objects" / digest[:2] / digest).read_text())["language"]
    plain = lambda text: all(c in " ()-'.," or c.isdigit() or unicodedata.name(c, "").startswith("LATIN")
                             for c in text)
    return [code for code, info in sorted(packs.items()) if plain(info["name"] + " (" + info["region"] + ")")]


def gui_language_cases():
    """The language screen as the reference draws it, in the languages it
    lists: the title over the search box, the list centred on the language
    in use, the warning, the font button and Done; the list scrolled and its
    bar dragged, the search typed into and cleared, the entries pressed
    (the outline white while the list has the focus, the hint when the box
    has not). Done, the font button and a double click open another screen or change
    the language, which the game's reload does: those are not pressed here."""
    def row(label, language="en_us", framebuffer=(854, 480), gui_scale=2, in_world=True,
            mouse=(0, 0), steps=()):
        return {"label": label, "language": language, "framebuffer": list(framebuffer),
                "gui_scale": gui_scale, "in_world": in_world, "mouse": list(mouse), "steps": list(steps),
                "codes": codes}

    codes = latin_language_codes()
    click = lambda x, y, double=False: {"do": "click", "x": x, "y": y, "double": double}
    drag = lambda x, y: {"do": "drag", "x": x, "y": y}
    release = lambda x=0, y=0: {"do": "release", "x": x, "y": y}
    wheel = lambda dy, x=200, y=100: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": False}
    typed = lambda text: [{"do": "char", "char": c} for c in text]
    back = {"do": "key", "key": "backspace"}
    rows = [
        row("english"), row("german", "de_de"), row("turkish", "tr_tr"),
        row("out-of-world", in_world=False), row("small", framebuffer=(640, 480)),
        row("large", framebuffer=(1280, 720)), row("tiny", framebuffer=(640, 480), gui_scale=3),
        row("short", framebuffer=(854, 300)), row("auto-large", framebuffer=(1920, 1080), gui_scale=0),
        row("first-language", "af_za"), row("unknown-language", "xx_xx"),
        row("hover-done", mouse=(300, 225)), row("hover-fonts", mouse=(130, 225)),
        row("hover-entry", mouse=(213, 120)),
        row("scroll-1", steps=[wheel(-1)]), row("scroll-5", steps=[wheel(-5)]),
        row("scroll-end", steps=[wheel(-500)]), row("scroll-top", steps=[wheel(500)]),
        row("scroll-over-header", steps=[wheel(-3, 200, 10)]),
        row("bar-drag", steps=[click(356, 90), drag(356, 120), release(356, 120)]),
        row("bar-drag-end", steps=[click(356, 90), drag(356, 400)]),
        row("bar-drag-top", steps=[click(356, 90), drag(356, 0)]),
        row("press-entry", steps=[click(213, 90)]),
        row("press-entry-then-search", steps=[click(213, 90), click(200, 25)]),
        row("press-entry-twice", steps=[click(213, 90), click(213, 125)]),
        row("press-entry-clipped-bottom", steps=[click(213, 190)]),
        row("press-gap", steps=[click(20, 90)]),
        row("press-search", steps=[click(150, 25)]),
        row("type-tr", steps=typed("tr")), row("type-de", "de_de", steps=typed("de")),
        row("type-german", steps=typed("German")), row("type-capital", steps=typed("TUR")),
        row("type-nothing-matches", steps=typed("zzzz")),
        row("type-region", steps=typed("Deutschland")),
        row("type-then-clear", steps=typed("tr") + [back, back]),
        row("type-one-back", steps=typed("abc") + [back]),
        row("type-then-press-entry", steps=typed("fran") + [click(213, 55)]),
        row("type-then-scroll", steps=typed("e") + [wheel(-2)]),
        row("type-after-list-focus", steps=[click(213, 90)] + typed("x")),
        row("hint-after-list", steps=[click(213, 90)]),
        row("type-selection-lost", "tr_tr", steps=typed("e")),
        row("type-long", steps=typed("abcdefghijklmnopqrstuvwxyz")),
    ]
    key = lambda name: {"do": "key", "key": name}
    tab, shift_tab = key("tab"), key("shift+tab")
    up, down, left, right = key("up"), key("down"), key("left"), key("right")
    rows += [
        row("tab-1", steps=[tab]), row("tab-2", steps=[tab] * 2), row("tab-3", steps=[tab] * 3),
        row("tab-4", steps=[tab] * 4), row("tab-5", steps=[tab] * 5), row("tab-9", steps=[tab] * 9),
        row("back-1", steps=[shift_tab]), row("back-2", steps=[shift_tab] * 2),
        row("back-3", steps=[shift_tab] * 3), row("back-4", steps=[shift_tab] * 4),
        row("back-5", steps=[shift_tab] * 5), row("tab-then-back", steps=[tab, tab, shift_tab]),
        row("tab-1-turkish", "tr_tr", steps=[tab]), row("tab-1-german", "de_de", steps=[tab]),
        row("tab-1-first", "af_za", steps=[tab]), row("tab-1-unknown", "xx_xx", steps=[tab]),
        row("search-down", steps=[down]), row("search-up", steps=[up]),
        row("search-left", steps=[left]), row("search-right", steps=[right]),
        row("typed-left", steps=typed("abc") + [left]), row("typed-left-right", steps=typed("abc") + [left, left, right]),
        row("typed-down", steps=typed("e") + [down]), row("typed-up", steps=typed("e") + [up]),
        row("typed-tab", steps=typed("e") + [tab]), row("typed-tab-tab", steps=typed("e") + [tab, tab]),
        row("typed-back", steps=typed("e") + [shift_tab]), row("typed-back-back", steps=typed("e") + [shift_tab] * 2),
        row("typed-nothing-tab", steps=typed("zzzz") + [tab]), row("typed-nothing-down", steps=typed("zzzz") + [down]),
        row("typed-nothing-back", steps=typed("zzzz") + [shift_tab]),
        row("typed-one-match-down", steps=typed("Deutsch") + [down, down]),
        row("list-down", steps=[tab, down]), row("list-down-3", steps=[tab] + [down] * 3),
        row("list-up", steps=[tab, up]), row("list-up-3", steps=[tab] + [up] * 3),
        row("list-down-20", steps=[tab] + [down] * 20), row("list-up-20", steps=[tab] + [up] * 20),
        row("list-down-end", steps=[tab] + [down] * 200), row("list-up-top", "af_za", steps=[tab, up]),
        row("list-up-top-twice", "af_za", steps=[tab, up, up]),
        row("list-left", steps=[tab, left]), row("list-right", steps=[tab, right]),
        row("list-left-down", steps=[tab, left, down]), row("list-left-tab", steps=[tab, left, tab]),
        row("list-left-back", steps=[tab, left, shift_tab]), row("list-right-up", steps=[tab, right, up]),
        row("list-down-up", steps=[tab, down, up]), row("list-down-tab", steps=[tab, down, tab]),
        row("list-down-back", steps=[tab, down, shift_tab]), row("list-down-back-back", steps=[tab, down, shift_tab, shift_tab]),
        row("list-end-down-fonts", steps=[tab] + [down] * 400), row("list-end-down-down", steps=[tab] + [down] * 401),
        row("list-top-up-search", "af_za", steps=[tab, up]), row("list-top-up-down", "af_za", steps=[tab, up, down]),
        row("fonts-up", steps=[tab, tab, up]), row("fonts-down", steps=[tab, tab, down]),
        row("fonts-left", steps=[tab, tab, left]), row("fonts-right", steps=[tab, tab, right]),
        row("done-up", steps=[tab] * 3 + [up]), row("done-down", steps=[tab] * 3 + [down]),
        row("done-left", steps=[tab] * 3 + [left]), row("done-right", steps=[tab] * 3 + [right]),
        row("done-left-up", steps=[tab] * 3 + [left, up]),
        row("fonts-up-down", steps=[tab, tab, up, down]), row("fonts-up-up", steps=[tab, tab, up, up]),
        row("search-down-down", steps=[down, down]), row("search-down-up", steps=[down, up]),
        row("search-down-right", steps=[down, right]), row("search-down-up-up", steps=[down, up, up]),
        row("search-down-up-down", steps=[down, up, down]),
        row("scroll-tab", "tr_tr", framebuffer=(854, 300), steps=[tab]),
        row("scroll-down-short", framebuffer=(854, 300), steps=[tab] + [down] * 30),
        row("scroll-up-short", framebuffer=(854, 300), steps=[tab] + [up] * 30),
        row("scroll-tiny-down", framebuffer=(640, 480), gui_scale=3, steps=[tab] + [down] * 12),
        row("scroll-wheel-then-tab", steps=[wheel(-5), tab]),
        row("scroll-wheel-then-down", steps=[tab, wheel(-8), down]),
        row("scroll-end-then-up", steps=[tab, wheel(-500), up]),
        row("out-of-world-tab", in_world=False, steps=[tab, down]),
        row("hover-and-focus", mouse=(213, 120), steps=[tab, down]),
        row("hover-fonts-focus-done", mouse=(130, 225), steps=[tab] * 3),
        row("click-down", steps=[click(213, 90), down]), row("click-up", steps=[click(213, 90), up]),
        row("click-left", steps=[click(213, 90), left]), row("click-tab", steps=[click(213, 90), tab]),
        row("click-back", steps=[click(213, 90), shift_tab]),
        row("click-back-back", steps=[click(213, 90), shift_tab, shift_tab]),
        row("click-search-tab", steps=[click(150, 25), tab]), row("click-search-back", steps=[click(150, 25), shift_tab]),
        row("click-bar-tab", steps=[click(356, 90), release(356, 90), tab]),
        row("click-bar-back", steps=[click(356, 90), release(356, 90), shift_tab]),
        row("click-bar-down", steps=[click(356, 90), release(356, 90), down]),
        row("click-bar-up", steps=[click(356, 90), release(356, 90), up]),
        row("click-bar-left", steps=[click(356, 90), release(356, 90), left]),
        row("click-bar-then-entry-down", steps=[click(356, 90), release(356, 90), click(213, 90), down]),
        row("click-gap-down", steps=[click(20, 90), down]),
    ]
    yield {"case_id": "gui/language-screen", "family": "gui", "operation": "language_screen", "rows": rows}
    keys = ["gui.done", "options.language.title", "options.video", "options.language.accuracy",
            "menu.quit", "gui.cancel", "options.on", "options.off", "deathScreen.title", "options.fov",
            "not.a.key", "options.generic_value", "chat.type.text"]
    yield {"case_id": "gui/lang-text", "family": "gui", "operation": "lang_text",
           "rows": [{"label": code, "language": code, "keys": keys}
                    for code in ("en_us", "tr_tr", "de_de", "fr_fr", "ja_jp", "ar_sa", "ru_ru", "xx_xx")]}


def gui_settings_key_cases():
    """Tab, Shift+Tab and the arrows on the settings lists: the first and the
    last widget taken from no focus, the walk through rows (two to a row,
    headers skipped, an inactive widget passed over), the column kept going
    up and down, the end of the list wrapping round through Done, the list
    scrolled to a focused entry (with the window short enough to need it),
    Enter and the arrows on a focused slider and cycle button, and the
    keyboard focus tooltip.  Enter on Done or on a button that opens another
    screen is not pressed: the reference closes or replaces the screen."""
    key = lambda name: {"do": "key", "key": name}
    tab, back = key("tab"), key("shift+tab")
    up, down, left, right = key("up"), key("down"), key("left"), key("right")
    enter, space = key("enter"), key("space")
    frame = lambda ms: {"do": "frame", "ms": ms}

    def keyed(label, steps, framebuffer=(854, 480), gui_scale=2, in_world=True, mouse=(0, 0), ms=0):
        return {"label": label, "framebuffer": list(framebuffer), "gui_scale": gui_scale,
                "in_world": in_world, "mouse": list(mouse), "steps": list(steps), "ms": ms}

    walks = [
        ("tab-1", [tab]), ("tab-2", [tab] * 2), ("tab-3", [tab] * 3), ("tab-6", [tab] * 6),
        ("tab-12", [tab] * 12), ("tab-30", [tab] * 30),
        ("back-1", [back]), ("back-2", [back] * 2), ("back-3", [back] * 3), ("back-8", [back] * 8),
        ("tab-back", [tab, tab, back]), ("tab-3-back-3", [tab] * 3 + [back] * 3),
        ("back-tab", [back, tab]), ("tab-past-end", [tab] * 40), ("back-past-start", [back] * 40),
        ("down-1", [down]), ("down-2", [down] * 2), ("down-4", [down] * 4), ("down-20", [down] * 20),
        ("up-1", [up]), ("up-2", [up] * 2), ("up-5", [up] * 5),
        ("left-1", [left]), ("right-1", [right]),
        ("tab-right", [tab, right]), ("tab-left", [tab, left]), ("tab-right-right", [tab, right, right]),
        ("tab-right-left", [tab, right, left]), ("tab-down-up", [tab, down, up]),
        ("tab-up", [tab, up]), ("tab-down", [tab, down]),
        ("down-right-down", [down, right, down]), ("down-right-down-down", [down, right, down, down]),
        ("right-down-up", [right, down, up]), ("down-down-right-up-up", [down, down, right, up, up]),
        ("down-to-done", [down] * 30 + [down]), ("done-up", [down] * 40 + [up]),
        ("done-up-up", [down] * 40 + [up, up]), ("done-left", [down] * 40 + [left]),
        ("done-right", [down] * 40 + [right]), ("done-tab", [back, tab]), ("done-back", [back, back]),
        ("up-up-down", [up, up, down]), ("tab-tab-left-down", [tab, tab, left, down]),
        ("hover-and-focus", [tab, tab]),
    ]
    frames = [("focus-tooltip", [tab, frame(50), frame(700)]),
              ("focus-tooltip-second", [tab, tab, frame(50), frame(700)]),
              ("focus-tooltip-moved", [tab, frame(50), frame(700), tab, frame(900), frame(1700)]),
              ("focus-tooltip-late", [tab, frame(100), frame(2000)])]
    rows = [keyed(label, steps) for label, steps in walks + frames]
    rows += [keyed("hover-and-focus-mouse", [tab, tab], mouse=(100, 60)),
             keyed("hover-done-focus-list", [tab], mouse=(213, 223)),
             keyed("focus-done-hover-other", [back], mouse=(100, 60))]
    # The list scrolls to what it focuses.
    for name, framebuffer, scale in (("short", (854, 300), 2), ("very-short", (640, 400), 4),
                                     ("tiny", (640, 480), 3), ("large", (1280, 720), 2),
                                     ("auto-large", (1920, 1080), 0)):
        rows += [keyed("scroll-tab-6-" + name, [tab] * 6, framebuffer, scale),
                 keyed("scroll-tab-12-" + name, [tab] * 12, framebuffer, scale),
                 keyed("scroll-down-8-" + name, [down] * 8, framebuffer, scale),
                 keyed("scroll-back-3-" + name, [back] * 3, framebuffer, scale),
                 keyed("scroll-up-3-" + name, [down] * 20 + [up] * 3, framebuffer, scale)]
    rows += [keyed("out-of-world-tab", [tab] * 3, in_world=False),
             keyed("scrolled-then-tab", [{"do": "scroll", "x": 200, "y": 100, "dy": -3, "ctrl": False}, tab]),
             keyed("scrolled-then-down", [{"do": "scroll", "x": 200, "y": 100, "dy": -5, "ctrl": False}, down]),
             keyed("clicked-then-tab", [{"do": "click", "x": 100, "y": 60, "shift": False},
                                        {"do": "release", "x": 100, "y": 60}, tab]),
             keyed("clicked-then-down", [{"do": "click", "x": 100, "y": 60, "shift": False},
                                         {"do": "release", "x": 100, "y": 60}, down]),
             keyed("clicked-then-up", [{"do": "click", "x": 100, "y": 60, "shift": False},
                                       {"do": "release", "x": 100, "y": 60}, up])]
    return rows


def gui_settings_key_press_cases(slider_tabs, toggle_tabs):
    """Enter and the arrows on a widget the keys reached: a slider takes
    Enter to let the arrows move it, then steps by them; a cycle button takes
    Enter and Space.  The counts are the Tab presses that reach a slider and
    a cycle button that is safe to press."""
    key = lambda name: {"do": "key", "key": name}
    tab, back = key("tab"), key("shift+tab")
    left, right, enter, space = key("left"), key("right"), key("enter"), key("space")
    keyed = lambda label, steps: {"label": label, "framebuffer": [854, 480], "gui_scale": 2,
                                  "in_world": True, "mouse": [0, 0], "steps": list(steps), "ms": 0}
    slider = [tab] * slider_tabs
    toggle = [tab] * toggle_tabs
    return [
        keyed("slider-arrows-before-enter", slider + [right]),
        keyed("slider-enter-right", slider + [enter, right]),
        keyed("slider-enter-right-right", slider + [enter, right, right]),
        keyed("slider-enter-left", slider + [enter, left]),
        keyed("slider-enter-left-left-left", slider + [enter, left, left, left]),
        keyed("slider-enter-twice-right", slider + [enter, enter, right]),
        keyed("slider-space", slider + [space, right]),
        keyed("slider-enter-right-tab", slider + [enter, right, tab]),
        keyed("slider-enter-down", slider + [enter, key("down")]),
        keyed("slider-enter-many", slider + [enter] + [right] * 12),
        keyed("slider-enter-many-left", slider + [enter] + [left] * 12),
        keyed("toggle-enter", toggle + [enter]), keyed("toggle-space", toggle + [space]),
        keyed("toggle-enter-twice", toggle + [enter, enter]),
        keyed("toggle-enter-right", toggle + [enter, right]),
        keyed("toggle-left", toggle + [left]),
    ]


def gui_settings_key_generators():
    plain = gui_settings_key_cases()
    for case_id, operation, slider_tabs, toggle_tabs in (
            ("video", "video_settings_screen", 2, 3), ("chat-options", "chat_options_screen", 5, 2),
            ("mouse-settings", "mouse_settings_screen", 1, 3), ("controls", "controls_screen", 8, 3)):
        yield {"case_id": "gui/%s-keys" % case_id, "family": "gui", "operation": operation,
               "rows": plain + gui_settings_key_press_cases(slider_tabs, toggle_tabs)}
