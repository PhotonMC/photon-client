"""The book and quill's editing screen."""


def gui_book_edit_screen_cases():
    """The book and quill's screen: its pages typed, wrapped and broken by Enter up to the page's
    fourteen lines, the cursor moved by line, word and click and blinking, selections over one
    line and several, pages turned and added by the buttons and Page Up/Down, the focus moved by
    Tab, a page longer than the box scrolled to its cursor."""
    def row(label, pages=(), width=427, height=240, mouse=(0, 0), steps=(), ms=0):
        return {"label": label, "pages": list(pages), "width": width, "height": height,
                "mouse": list(mouse), "steps": list(steps), "ms": ms}

    type_ = lambda text: {"do": "type", "text": text}
    key = lambda name: {"do": "key", "key": name}
    click = lambda x, y, shift=False: {"do": "click", "x": x, "y": y, **({"shift": True} if shift else {})}
    two = ("Hello world", "Second page")
    long_text = "The quick brown fox jumps over the lazy dog and keeps on running far away"
    rows = [
        row("empty"), row("empty small", width=320), row("empty wide", width=640, height=360),
        row("written", two), row("written blink", two, ms=300), row("written blink back", two, ms=650),
        row("typed", steps=[type_("Hi there")]),
        row("wrapped", steps=[type_(long_text)]),
        row("one long word", steps=[type_("W" * 40)]),
        row("new lines", steps=[type_("one"), key("enter"), key("enter"), type_("three")]),
        row("line limit", steps=[key("enter")] * 16 + [type_("x")]),
        row("select all", two, steps=[key("ctrl+a")]),
        row("select left", two, steps=[key("shift+left"), key("shift+left"), key("shift+left")]),
        row("select lines", steps=[type_(long_text), key("shift+up"), key("shift+up")]),
        row("select typed over", two, steps=[key("ctrl+a"), type_("new")]),
        row("cursor inside", two, steps=[key("left"), key("left")]),
        row("cursor up", steps=[type_(long_text), key("up")]),
        row("cursor up down", steps=[type_(long_text), key("up"), key("up"), key("down")]),
        row("home", steps=[type_(long_text), key("home")]),
        row("ctrl home", steps=[type_(long_text), key("ctrl+home")]),
        row("end", steps=[type_(long_text), key("ctrl+home"), key("end")]),
        row("word left", two, steps=[key("ctrl+left"), type_("X")]),
        row("word right", two, steps=[key("ctrl+home"), key("ctrl+right"), type_("X")]),
        row("word erased", two, steps=[key("ctrl+backspace")]),
        row("backspace", two, steps=[key("backspace"), key("backspace")]),
        row("delete", two, steps=[key("ctrl+home"), key("delete")]),
        row("section sign refused", steps=[type_("a§b")]),
        row("page down new", two, steps=[key("pagedown"), key("pagedown")]),
        row("page down up", two, steps=[key("pagedown"), key("pageup")]),
        row("page up first", two, steps=[key("pageup")]),
        row("forward clicked", two, steps=[click(240, 164)]),
        row("forward clicked twice", two, steps=[click(240, 164), click(240, 164), click(240, 164)]),
        row("back clicked", two, steps=[click(240, 164), click(167, 164)]),
        row("hover forward", two, mouse=(240, 164)),
        row("hover back", two, mouse=(167, 164), steps=[key("pagedown")]),
        row("hover sign", two, mouse=(150, 205)), row("hover done", two, mouse=(260, 205)),
        row("tab", two, steps=[key("tab")]), row("tab twice", two, steps=[key("tab"), key("tab")]),
        row("tab thrice", two, steps=[key("tab")] * 3),
        row("tab round", two, steps=[key("tab")] * 4),
        row("shift tab", two, steps=[key("shift+tab")]),
        row("tab back", two, steps=[key("pagedown"), key("tab")]),
        row("tab enter", two, steps=[key("tab"), key("enter")]),
        row("tab space", two, steps=[key("tab"), key("space")]),
        row("tab typed ignored", two, steps=[key("tab"), type_("zz")]),
        row("click text", two, steps=[click(165, 34)]),
        row("click second line", steps=[type_(long_text), click(170, 43)]),
        row("click below", two, steps=[click(170, 120)]),
        row("shift click", two, steps=[key("ctrl+home"), click(175, 34, True)]),
        row("click back focus", two, steps=[key("tab"), key("tab"), click(165, 34)], ms=100),
        row("overflowing page", ["line\n" * 20 + "end"]),
        row("overflowing page up", ["line\n" * 20 + "end"], steps=[key("ctrl+home")]),
    ]
    yield {"case_id": "gui/book-edit-screen", "family": "gui", "operation": "book_edit_screen",
           "rows": rows}
