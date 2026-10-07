"""The `gui` family: chat."""

from cases.helpers import *


def gui_cases():
    """The screens' drawing as the reference's own GUI code records it: the
    rectangles and the text, in order, with their poses applied. The glyphs
    are not drawn; their advances come from the reference's font providers."""
    def chat(label, messages, tick, width=427, height=240, focused=False, scroll=None,
             extra=(), deletions=(), options=None, rescale=None, lang_after=None, mouse=None):
        row = {"label": label, "width": width, "height": height,
               "messages": [{"text": text, "tick": at} for text, at in messages] + list(extra),
               "tick": tick}
        if deletions:
            row["deletions"] = [{"delete": sig, "tick": at} for sig, at in deletions]
        if focused:
            row["focused"] = True
        if scroll is not None:
            row["scroll"] = scroll
        if options:
            row["options"] = options
        if rescale:
            row["rescale"] = rescale
        if lang_after:
            row["lang_after"] = lang_after
        if mouse is not None:
            row["mouse"] = list(mouse)
        return row

    long_text = ("this is a fairly long chat message that should wrap onto a second line "
                 "when it reaches the edge of the chat box and goes on for a third one too")
    rows = [
        chat("one-line", [("hello world", 0)], 0),
        chat("three-lines", [("one", 0), ("two", 1), ("three", 2)], 3),
        chat("wrapped", [(long_text, 0)], 5),
        chat("fading", [("fading out", 0)], 190),
        chat("faded", [("gone", 0)], 200),
        chat("many", [(f"line {n}", 0) for n in range(14)], 10),
        chat("tall-window", [("hello", 0)], 0, width=640, height=480),
        chat("newline", [("first line\nsecond line", 0)], 0),
        chat("open-unscrolled", [(f"line {n}", 0) for n in range(30)], 400, focused=True),
        chat("open-scrolled", [(f"line {n}", 0) for n in range(30)], 400, focused=True, scroll=3),
        chat("open-scrolled-end", [(f"line {n}", 0) for n in range(30)], 400, focused=True, scroll=100),
        chat("open-short", [("only", 0), ("two", 0)], 400, focused=True),
        chat("tags", [], 5, extra=[
            {"text": "system", "tick": 0, "tag": "system"},
            {"text": "not secure", "tick": 1, "tag": "not_secure", "signature": 1},
            {"text": "modified", "tick": 2, "tag": "modified:the original", "signature": 2},
            {"text": "error", "tick": 3, "tag": "error"},
            {"text": "no tag", "tick": 4, "signature": 3},
        ]),
        chat("modified-wraps-narrower", [], 5, extra=[
            {"text": long_text, "tick": 0, "tag": "modified:x", "signature": 4},
        ]),
        chat("delete-waits", [], 30, extra=[{"text": "will go", "tick": 0, "signature": 7}],
             deletions=[(7, 10)]),
        chat("delete-after-sixty", [], 100, extra=[
            {"text": "before", "tick": 0, "tag": "system"},
            {"text": "will go", "tick": 1, "signature": 7},
            {"text": "after", "tick": 2, "tag": "system"},
        ], deletions=[(7, 5)]),
        chat("delete-unknown", [], 10, extra=[{"text": "kept", "tick": 0, "signature": 8}],
             deletions=[(9, 5)]),
    ]
    # The chat options: each alone and together, closed and open, before the
    # messages and after them (the chat wraps them again), with tags.
    many = [(f"line {n}", 0) for n in range(30)]
    opt = lambda **values: values
    rows += [
        chat("scale-half", [(long_text, 0)], 3, options=opt(scale=0.5)),
        chat("scale-three-quarters", [(long_text, 0)], 3, options=opt(scale=0.75)),
        chat("scale-third", [("hello", 0), ("two", 1)], 3, options=opt(scale=0.33)),
        chat("scale-open", many, 400, focused=True, options=opt(scale=0.6)),
        chat("scale-open-scrolled", many, 400, focused=True, scroll=5, options=opt(scale=0.9)),
        chat("scale-tags", [], 5, options=opt(scale=0.7), extra=[
            {"text": "system", "tick": 0, "tag": "system"},
            {"text": "modified", "tick": 2, "tag": "modified:the original", "signature": 2}]),
        chat("width-narrow", [(long_text, 0)], 3, options=opt(width=0.0)),
        chat("width-half", [(long_text, 0)], 3, options=opt(width=0.5)),
        chat("width-quarter-open", [(long_text, 0)] * 4, 3, focused=True, options=opt(width=0.25)),
        chat("width-and-scale", [(long_text, 0)], 3, options=opt(width=0.4, scale=0.8)),
        chat("opacity-zero", [("hello", 0)], 3, options=opt(opacity=0.0)),
        chat("opacity-low", [("hello", 0), ("fading", 1)], 3, options=opt(opacity=0.3)),
        chat("opacity-fading", [("fading out", 0)], 190, options=opt(opacity=0.5)),
        chat("opacity-open-tags", [], 5, focused=True, options=opt(opacity=0.4), extra=[
            {"text": "system", "tick": 0, "tag": "system"},
            {"text": "error", "tick": 3, "tag": "error"}]),
        chat("background-zero", [("hello", 0)], 3, options=opt(background_opacity=0.0)),
        chat("background-full", [("hello", 0), ("two", 0)], 3, options=opt(background_opacity=1.0)),
        chat("background-fading", [("fading out", 0)], 190, options=opt(background_opacity=0.8)),
        chat("spacing-full", [("one", 0), ("two", 1), ("three", 2)], 3, options=opt(line_spacing=1.0)),
        chat("spacing-quarter", [(f"line {n}", 0) for n in range(14)], 10, options=opt(line_spacing=0.25)),
        chat("spacing-open", many, 400, focused=True, options=opt(line_spacing=0.5)),
        chat("spacing-open-scrolled", many, 400, focused=True, scroll=4, options=opt(line_spacing=0.3)),
        chat("spacing-tags", [], 5, options=opt(line_spacing=0.6), extra=[
            {"text": "system", "tick": 0, "tag": "system"},
            {"text": "no tag", "tick": 4, "signature": 3}]),
        chat("height-unfocused-low", [(f"line {n}", 0) for n in range(14)], 10, options=opt(height_unfocused=0.0)),
        chat("height-unfocused-full", [(f"line {n}", 0) for n in range(30)], 10, options=opt(height_unfocused=1.0)),
        chat("height-focused-low", many, 400, focused=True, options=opt(height_focused=0.2)),
        chat("height-focused-half-scrolled", many, 400, focused=True, scroll=7, options=opt(height_focused=0.5)),
        chat("height-and-spacing", many, 400, focused=True, options=opt(height_focused=0.6, line_spacing=0.4)),
        chat("scroll-end-low-height", many, 400, focused=True, scroll=100, options=opt(height_focused=0.3)),
        chat("everything", many, 400, focused=True, scroll=3, options=opt(
            scale=0.7, width=0.6, opacity=0.6, background_opacity=0.3, line_spacing=0.2,
            height_focused=0.7)),
        chat("rescale-width", [(long_text, 0), ("short", 1)], 3, rescale=opt(width=0.3)),
        chat("rescale-scale", [(long_text, 0)], 3, rescale=opt(scale=0.5)),
        chat("rescale-resets-scroll", many, 400, focused=True, scroll=6, rescale=opt(opacity=0.9)),
        chat("rescale-after-options", [(long_text, 0)], 3, options=opt(width=0.8), rescale=opt(width=0.2)),
        chat("tall-window-scaled", [("hello", 0)], 0, width=640, height=480, options=opt(scale=0.5)),
    ]
    # The mouse over an open chat: the tooltip of the text under it. The last
    # line is at the bottom, its text from x 4 and 192 down.
    import json as _json
    show = lambda text, **style: dict({"text": text}, **style)
    tip = lambda value: {"action": "show_text", "value": value}
    hover = lambda text, value="tooltip text", **style: show(text, hover_event=tip(value), **style)
    comp = lambda component, at=0, **kw: dict({"json": _json.dumps(component), "tick": at}, **kw)
    rows += [
        chat("hover-show-text", [], 5, focused=True, mouse=(10, 196), extra=[comp(hover("hover me"))]),
        chat("hover-show-text-unfocused", [], 5, mouse=(10, 196), extra=[comp(hover("hover me"))]),
        chat("hover-miss-right", [], 5, focused=True, mouse=(300, 196), extra=[comp(hover("hover me"))]),
        chat("hover-miss-above", [], 5, focused=True, mouse=(10, 150), extra=[comp(hover("hover me"))]),
        chat("hover-left-edge", [], 5, focused=True, mouse=(4, 196), extra=[comp(hover("hover me"))]),
        chat("hover-before-left-edge", [], 5, focused=True, mouse=(3, 196), extra=[comp(hover("hover me"))]),
        chat("hover-last-glyph", [], 5, focused=True, mouse=(4 + 6 * 8 - 2, 196), extra=[comp(hover("hover me"))]),
        chat("hover-past-end", [], 5, focused=True, mouse=(4 + 8 * 6 + 1, 196), extra=[comp(hover("hover me"))]),
        chat("hover-glyph-top", [], 5, focused=True, mouse=(10, 192), extra=[comp(hover("hover me"))]),
        chat("hover-glyph-bottom", [], 5, focused=True, mouse=(10, 200), extra=[comp(hover("hover me"))]),
        chat("hover-below-line", [], 5, focused=True, mouse=(10, 201), extra=[comp(hover("hover me"))]),
        chat("hover-sibling", [], 5, focused=True, mouse=(40, 196),
             extra=[comp(show("a", extra=[hover("bold", "bold tip", bold=True)]))]),
        chat("hover-inherited", [], 5, focused=True, mouse=(20, 196),
             extra=[comp(hover("parent ", "parent tip", extra=[show("child")]))]),
        chat("hover-on-child-only", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(show("plain ", extra=[hover("child", "child tip")]))]),
        chat("hover-italic", [], 5, focused=True, mouse=(12, 196), extra=[comp(hover("slanted", italic=True))]),
        chat("hover-bold-edge", [], 5, focused=True, mouse=(11, 196), extra=[comp(hover("bold", bold=True))]),
        chat("hover-coloured-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("hover me", {"text": "red tip", "color": "red"}))]),
        chat("hover-long-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("hover me", long_text * 2))]),
        chat("hover-newline-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("hover me", "first\nsecond\nthird"))]),
        chat("hover-two-lines-upper", [], 5, focused=True, mouse=(10, 187),
             extra=[comp(hover("upper", "upper tip"), 0), comp(hover("lower", "lower tip"), 1)]),
        chat("hover-two-lines-lower", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("upper", "upper tip"), 0), comp(hover("lower", "lower tip"), 1)]),
        chat("hover-wrapped-second-line", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover(long_text, "wrapped tip"))]),
        chat("hover-wrapped-first-line", [], 5, focused=True, mouse=(10, 178),
             extra=[comp(hover(long_text, "wrapped tip"))]),
        chat("hover-with-tag-text", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("tagged"), tag="system")]),
        chat("hover-with-tag-bar", [], 5, focused=True, mouse=(0, 196),
             extra=[comp(hover("tagged"), tag="system")]),
        chat("hover-scaled", [], 5, focused=True, mouse=(5, 198), options=opt(scale=0.5),
             extra=[comp(hover("hover me"))]),
        chat("hover-scaled-miss", [], 5, focused=True, mouse=(5, 150), options=opt(scale=0.5),
             extra=[comp(hover("hover me"))]),
        chat("hover-spaced", [], 5, focused=True, mouse=(10, 196), options=opt(line_spacing=0.5),
             extra=[comp(hover("hover me"))]),
        chat("hover-tall-window", [], 5, focused=True, width=640, height=480, mouse=(10, 436),
             extra=[comp(hover("hover me"))]),
        chat("hover-narrow-window", [], 5, focused=True, width=200, height=150, mouse=(10, 106),
             extra=[comp(hover("hover me", long_text))]),
        chat("hover-space-glyph", [], 5, focused=True, mouse=(4 + 5 * 6 + 1, 196),
             extra=[comp(hover("hello world"))]),
        chat("hover-entity-no-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(show("entity", hover_event={"action": "show_entity", "id": "minecraft:pig",
                                                   "uuid": [1, 2, 3, 4]}))]),
        chat("click-only-no-tip", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(show("link", click_event={"action": "open_url", "url": "https://example.com"}))]),
        chat("hover-last-wins", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("ab", "first", extra=[hover("", "second")]))]),
        chat("hover-no-mouse-rows-draw", [], 5, focused=True, mouse=(10, 196),
             extra=[comp(hover("hover me")), {"text": "x", "tick": 1}]),
    ]
    # A language change words the translatable messages again, and wraps them.
    warning = {"key": "options.languageAccuracyWarning"}
    rows += [
        chat("lang-same", [], 3, extra=[dict(warning, tick=0)], lang_after="en_us"),
        chat("lang-german", [], 3, extra=[dict(warning, tick=0), {"key": "gui.done", "tick": 1}], lang_after="de_de"),
        chat("lang-french", [], 3, extra=[dict(warning, tick=0), {"text": "plain", "tick": 1}], lang_after="fr_fr"),
        chat("lang-portuguese-tags", [], 3, extra=[dict(warning, tick=0, tag="system"),
                                                    {"key": "menu.quit", "tick": 1, "tag": "error"}],
             lang_after="pt_br"),
        chat("lang-spanish-open", [], 400, focused=True,
             extra=[dict(warning, tick=n) for n in range(0, 8)], lang_after="es_es", scroll=3),
        chat("lang-unknown", [], 3, extra=[dict(warning, tick=0)], lang_after="xx_xx"),
    ]
    yield {"case_id": "gui/chat", "family": "gui", "operation": "chat", "rows": rows}


def gui_suggestion_cases():
    """The chat box's list of matches: where it sits, which one is chosen, what
    Tab, Shift+Tab, the arrow keys and Escape do to the list and the text, and
    the grey text after the cursor."""
    def row(label, text, start, length, matches, keys, width=427, height=240):
        return {"label": label, "width": width, "height": height, "text": text, "start": start,
                "length": length, "matches": matches, "keys": keys}

    many = [f"entry{n}" for n in range(14)]
    rows = [
        row("first-chosen", "/gam", 1, 3, ["gamemode", "gamerule"], []),
        row("tab-twice", "/gam", 1, 3, ["gamemode", "gamerule"], ["tab", "tab", "tab"]),
        row("shift-tab", "/gam", 1, 3, ["gamemode", "gamerule"], ["tab", "shift+tab", "tab"]),
        row("arrows", "/gam", 1, 3, ["gamemode", "gamerule", "gametest"], ["down", "down", "up", "up", "up"]),
        row("escape-hides", "/gam", 1, 3, ["gamemode", "gamerule"], ["escape"]),
        row("prefix-first", "/ga", 1, 2, ["zgame", "gamemode", "minecraft:gamerule"], []),
        row("mid-text", "/say hel", 5, 3, ["help", "hello", "hex"], ["tab"]),
        row("scrolling-window", "/e", 1, 1, many, ["down"] * 11 + ["up"] * 3),
        row("wraps-round", "/e", 1, 1, many, ["up", "up", "down"]),
        row("narrow-screen", "/gam", 1, 3, ["gamemode", "gamerule"], ["tab"], width=320, height=240),
    ]
    yield {"case_id": "gui/suggestions", "family": "gui", "operation": "suggestions", "rows": rows}


def gui_chat_draft_cases():
    """The chat box's draft: what a closed box keeps by the way it closed
    (Escape only with the option on, an interruption always, a sent line or
    nothing to send never), a blank text kept as nothing, which box takes
    which draft (a message box any, a command box a command's), a draft left
    as it was when it is not edited, one Backspace wiping it, and a draft
    drawn grey and slanted until it is edited."""
    op = lambda method="message": {"do": "open", "method": method}
    typed = lambda text: {"do": "type", "text": text}
    key = lambda name: {"do": "key", "key": name}
    close = lambda reason: {"do": "close", "reason": reason}
    draw = {"do": "draw"}

    def row(label, events, save=False, width=427, height=240):
        return {"label": label, "width": width, "height": height, "save": save, "events": events}

    kept = [op(), typed("hello"), close("interrupted")]
    rows = [
        row("empty", [op()]), row("empty-command", [op("command")]),
        row("escape-discards", [op(), typed("hello"), close("intentional"), op()]),
        row("escape-saves", [op(), typed("hello"), close("intentional"), op()], save=True),
        row("interrupted-saves", kept + [op()]),
        row("interrupted-saves-with-option", kept + [op()], save=True),
        row("done-discards", [op(), typed("hello"), close("done"), op()]),
        row("done-discards-with-option", [op(), typed("hello"), close("done"), op()], save=True),
        row("command-draft-to-command", [op("command"), typed("gamemode"), close("interrupted"), op("command")]),
        row("command-draft-to-message", [op("command"), typed("gamemode"), close("interrupted"), op()]),
        row("message-draft-to-command", kept + [op("command")]),
        row("message-draft-to-command-then-message", kept + [op("command"), close("done"), op()]),
        row("blank-is-nothing", [op(), typed("   "), close("interrupted"), op()]),
        row("blank-clears-the-draft", kept + [op(), key("backspace"), close("interrupted"), op()]),
        row("slash-only-is-kept", [op("command"), close("interrupted"), op("command")]),
        row("slash-only-blank-is-not", [op("command"), close("interrupted"), op()]),
        row("spaces-kept-as-typed", [op(), typed("  hi  "), close("interrupted"), op()]),
        row("unedited-draft-stays", kept + [op(), close("interrupted"), op()]),
        row("unedited-draft-stays-escape", kept + [op(), close("intentional"), op()], save=True),
        row("unedited-draft-lost-escape", kept + [op(), close("intentional"), op()]),
        row("unedited-draft-lost-done", kept + [op(), close("done"), op()]),
        row("edited-draft-kept", kept + [op(), typed("!"), close("interrupted"), op()]),
        row("edited-draft-lost-on-done", kept + [op(), typed("!"), close("done"), op()]),
        row("backspace-wipes-draft", kept + [op(), key("backspace")]),
        row("backspace-wipes-then-types", kept + [op(), key("backspace"), typed("x")]),
        row("backspace-twice", kept + [op(), key("backspace"), key("backspace")]),
        row("backspace-on-own-text", [op(), typed("abc"), key("backspace")]),
        row("cursor-keys-keep-draft", kept + [op(), key("left"), key("left"), key("home"), key("end")]),
        row("delete-edits-draft", kept + [op(), key("home"), key("delete")]),
        row("typing-ends-draft", kept + [op(), typed("a")]),
        row("draft-then-wiped-then-closed", kept + [op(), key("backspace"), close("intentional"), op()], save=True),
        row("second-draft-replaces", kept + [op(), key("backspace"), typed("other"), close("interrupted"), op()]),
        row("draw-empty", [op(), draw]), row("draw-own-text", [op(), typed("hello"), draw]),
        row("draw-draft", kept + [op(), draw]),
        row("draw-draft-edited", kept + [op(), typed("!"), draw]),
        row("draw-draft-wiped", kept + [op(), key("backspace"), draw]),
        row("draw-draft-cursor-moved", kept + [op(), key("left"), draw]),
        row("draw-draft-small", kept + [op(), draw], width=200, height=120),
        row("draw-long-draft", [op(), typed("a long draft that goes on " * 6), close("interrupted"), op(), draw]),
        row("many-closes", [op(), typed("a"), close("interrupted"), op(), typed("b"), close("interrupted"),
                            op(), typed("c"), close("done"), op()]),
    ]
    yield {"case_id": "gui/chat-draft", "family": "gui", "operation": "chat_draft", "rows": rows}
