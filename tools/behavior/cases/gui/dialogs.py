"""Server dialogs: the notice and confirmation screen, and the waiting screen."""


def gui_dialog_screen_cases():
    """`SimpleDialogScreen` for a notice and a confirmation: the title with its warning button, the
    body in a scrolling box (wheel, scrollbar, Tab into view), the buttons and what they tell the
    connection, Escape, and the screen about custom screens; and `WaitingForResponseScreen`."""
    def message(text, width=200):
        return {"text": text, "width": width}

    def item(width=16, height=16, description=None):
        body = {"item": True, "width": width, "height": height}
        if description:
            body["description"] = description
        return body

    def button(label="OK", width=150, tooltip=None, action=None):
        data = {"label": label, "width": width}
        if tooltip:
            data["tooltip"] = tooltip
        if action:
            data["action"] = action
        return data

    def row(label, width=427, height=240, mouse=(0, 0), in_world=True, steps=(), ms=0, body=(message("Hello there."),),
            buttons=None, title="Notice", can_close=True, after="CLOSE", kind=None):
        buttons = buttons if buttons is not None else [button()]
        dialog = {"kind": kind or ("confirmation" if len(buttons) == 2 else "notice"), "title": title, "body": list(body),
                  "buttons": buttons, "can_close": can_close, "after": after}
        return {"label": label, "width": width, "height": height, "mouse": list(mouse), "in_world": in_world,
                "dialog": dialog, "steps": list(steps), "ms": ms}

    key = lambda name: {"do": "key", "key": name}
    click = lambda x, y: {"do": "click", "x": x, "y": y}
    scroll = lambda x, y, amount: {"do": "scroll", "x": x, "y": y, "amount": amount}
    command = lambda text: {"type": "run_command", "value": text}
    custom = lambda name: {"type": "custom", "value": name}
    paragraphs = [message(f"Paragraph {n}: the body of this dialog runs on for a while.") for n in range(14)]
    confirm = [button("Yes", 100, action=command("say yes")), button("No", 100, action=custom("minecraft:no"))]
    long_text = ("This message is long enough that it has to wrap onto several lines of the dialog "
                 "before the button, and it keeps going so that more lines are needed too.")
    rows = [
        row("notice"), row("notice-out-of-world", in_world=False), row("no-body", body=()),
        row("two-bodies", body=(message("First."), message("Second, a little longer than the first.")))
        , row("wrapped", body=(message(long_text),)), row("wrapped-narrow", body=(message(long_text, 100),)),
        row("wide-message", body=(message(long_text, 400),)), row("small", 320, 240), row("large", 854, 480),
        row("tiny", 200, 150, body=(message(long_text),)), row("empty-message", body=(message(""),)),
        row("long-title", title="A title that is long enough to push the warning button out of view"),
        row("confirmation", buttons=confirm),
        row("confirmation-wide", buttons=[button("Accept", 200), button("Decline", 120)]),
        row("button-tooltip-hover", mouse=(200, 224), steps=[{"do": "frame", "ms": 0}], ms=700,
            buttons=[button("OK", 150, tooltip="Closes the dialog.")]),
        row("hover-button", mouse=(200, 224)), row("hover-warning", mouse=(230, 15)),
        row("hover-warning-tooltip", mouse=(230, 15), steps=[{"do": "frame", "ms": 0}], ms=800),
        row("item", body=(item(),)), row("item-large", body=(item(64, 48),)),
        row("item-description", body=(item(16, 16, message("A shiny diamond.", 120)),)),
        row("item-and-text", body=(message("Look:"), item(), message("That one."))),
        row("scrolling", body=paragraphs), row("scrolling-small", 320, 180, body=paragraphs),
        row("scroll-down", body=paragraphs, steps=[scroll(213, 120, -1)]),
        row("scroll-down-far", body=paragraphs, steps=[scroll(213, 120, -50)]),
        row("scroll-up-clamped", body=paragraphs, steps=[scroll(213, 120, 3)]),
        row("scroll-outside", body=paragraphs, steps=[scroll(5, 120, -3)]),
        row("scroll-no-room", steps=[scroll(213, 100, -3)]),
        row("drag-bar", body=paragraphs, steps=[click(305, 80), {"do": "drag", "x": 305, "y": 140, "dx": 0, "dy": 60}]),
        row("drag-bar-released", body=paragraphs, steps=[click(305, 80), {"do": "drag", "x": 305, "y": 140, "dx": 0, "dy": 60},
                                                         {"do": "release", "x": 305, "y": 140},
                                                         {"do": "drag", "x": 305, "y": 200, "dx": 0, "dy": 60}]),
        row("drag-above", body=paragraphs, steps=[scroll(213, 120, -5), click(305, 80), {"do": "drag", "x": 305, "y": 10, "dx": 0, "dy": -60}]),
        row("tab", steps=[key("tab")]), row("tab-2", steps=[key("tab")] * 2), row("tab-3", steps=[key("tab")] * 3),
        row("tab-4-round", steps=[key("tab")] * 4), row("shift-tab", steps=[key("shift+tab")]),
        row("shift-tab-2", steps=[key("shift+tab")] * 2),
        row("tab-confirmation", buttons=confirm, steps=[key("tab")] * 4),
        row("tab-item", body=(item(), message("Text")), steps=[key("tab")] * 2),
        row("tab-into-view", body=paragraphs, steps=[key("tab")] * 8),
        row("tab-into-view-end", body=paragraphs, steps=[key("tab")] * 15),
        row("tab-back-into-view", body=paragraphs, steps=[key("shift+tab")] * 3),
        row("click-text", body=paragraphs[:2], mouse=(213, 100), steps=[click(213, 100)]),
        row("click-gap", steps=[click(10, 10)]),
        row("enter-unfocused", steps=[key("enter")]),
        row("enter-button", steps=[key("tab")] * 3 + [key("enter")], buttons=[button("Run", 150, action=command("say hi"))]),
        row("space-button", steps=[key("tab")] * 3 + [key("space")], buttons=[button("Run", 150, action=command("/say hi"))]),
        row("enter-no-action", steps=[key("tab")] * 3 + [key("enter")]),
        row("enter-on-text", steps=[key("tab"), key("enter")]),
        row("enter-warning", steps=[key("tab"), key("enter")], body=()),
        row("click-button-run", mouse=(200, 224), steps=[click(200, 224)], buttons=[button("Run", 150, action=command("say hi"))]),
        row("click-button-custom", mouse=(200, 224), steps=[click(200, 224)], buttons=[button("Go", 150, action=custom("minecraft:go"))]),
        row("click-button-none", mouse=(200, 224), steps=[click(200, 224)], after="NONE",
            buttons=[button("Run", 150, action=command("say hi"))]),
        row("click-button-wait", mouse=(200, 224), steps=[click(200, 224)], after="WAIT_FOR_RESPONSE",
            buttons=[button("Run", 150, action=command("say hi"))]),
        row("click-button-wait-no-action", mouse=(200, 224), steps=[click(200, 224)], after="WAIT_FOR_RESPONSE"),
        row("click-button-none-no-action", mouse=(200, 224), steps=[click(200, 224)], after="NONE"),
        row("click-yes", mouse=(160, 224), steps=[click(160, 224)], buttons=confirm),
        row("click-no", mouse=(268, 224), steps=[click(268, 224)], buttons=confirm),
        row("click-warning", mouse=(230, 15), steps=[click(230, 15)]),
        row("escape", steps=[key("escape")]), row("escape-action", steps=[key("escape")],
                                                   buttons=[button("Run", 150, action=command("say hi"))]),
        row("escape-confirmation", steps=[key("escape")], buttons=confirm),
        row("escape-shut", steps=[key("escape")], can_close=False),
        row("escape-wait", steps=[key("escape")], after="WAIT_FOR_RESPONSE", buttons=confirm),
        row("escape-none", steps=[key("escape")], after="NONE", buttons=confirm),
    ]
    waiting = lambda label, steps=(), mouse=(0, 0), in_world=True, width=427, height=240: {
        "label": label, "width": width, "height": height, "mouse": list(mouse), "in_world": in_world,
        "dialog": {"kind": "waiting"}, "steps": list(steps), "ms": 0}
    ticks = lambda count: {"do": "tick", "count": count}
    rows += [
        waiting("waiting"), waiting("waiting-out-of-world", in_world=False), waiting("waiting-19", [ticks(19)]),
        waiting("waiting-20", [ticks(20)]), waiting("waiting-21", [ticks(21)]), waiting("waiting-40", [ticks(40)]),
        waiting("waiting-60", [ticks(60)]), waiting("waiting-80", [ticks(80)]), waiting("waiting-99", [ticks(99)]),
        waiting("waiting-100", [ticks(100)]), waiting("waiting-101", [ticks(101)]), waiting("waiting-300", [ticks(300)]),
        waiting("waiting-small", [ticks(100)], width=320, height=240), waiting("waiting-short", [ticks(100)], height=60),
        waiting("waiting-hover", [ticks(100)], mouse=(213, 73)),
        waiting("waiting-click-asleep", [ticks(40), click(213, 73)], mouse=(213, 73)),
        waiting("waiting-click-awake", [ticks(100), click(213, 73)], mouse=(213, 73)),
        waiting("waiting-escape-asleep", [ticks(40), key("escape")]),
        waiting("waiting-escape-awake", [ticks(100), key("escape")]),
    ]
    yield {"case_id": "gui/dialog-screen", "family": "gui", "operation": "dialog_screen", "rows": rows}
