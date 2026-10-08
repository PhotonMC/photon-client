"""The `gui` family: settings."""

from cases.helpers import *


def gui_options_screen_cases():
    """The options screen: the field of view, the header's second button
    (online outside a world, world options in one), the grid of settings
    screens (telemetry asleep: the player is offline) and Done, in several
    screen sizes; hover, Tab through every widget and round, the slider
    moved with the mouse and the keys (an arrow key that the slider does not
    take moves the focus, which is not built). The buttons that open other
    screens are not pressed."""
    def row(label, width=427, height=240, mouse=(0, 0), in_world=True, fov=70, steps=()):
        return {"label": label, "width": width, "height": height, "mouse": list(mouse), "in_world": in_world,
                "fov": fov, "steps": list(steps)}

    def key(name):
        return {"do": "key", "key": name}

    click = lambda x, y=38: {"do": "click", "x": x, "y": y}
    drag = lambda x, y=38: {"do": "drag", "x": x, "y": y}
    release = lambda x, y=38: {"do": "release", "x": x, "y": y}
    tab = key("tab")
    rows = [
        row("in-world"), row("out-of-world", in_world=False), row("small", 320, 240), row("large", 640, 360),
        row("tiny", 200, 150), row("wide", 854, 480, in_world=False), row("short", 427, 200),
        row("fov-min", fov=30), row("fov-max", fov=110), row("fov-90", fov=90),
        row("hover-fov", mouse=(100, 38)), row("hover-online", mouse=(300, 38), in_world=False),
        row("hover-world-options", mouse=(300, 38)), row("hover-skin", mouse=(100, 95)),
        row("hover-controls", mouse=(300, 119)), row("hover-telemetry-asleep", mouse=(100, 190)),
        row("hover-done", mouse=(200, 220)), row("hover-gap", mouse=(212, 95)),
        row("hover-elsewhere", mouse=(5, 5)),
        row("tab-1", steps=[tab]), row("tab-2", steps=[tab] * 2), row("tab-3", steps=[tab] * 3),
        row("tab-9", steps=[tab] * 9), row("tab-10-past-telemetry", steps=[tab] * 10),
        row("tab-12-done", steps=[tab] * 12), row("tab-13-round", steps=[tab] * 13),
        row("shift-tab", steps=[key("shift+tab")]), row("shift-tab-2", steps=[key("shift+tab")] * 2),
        row("tab-then-back", steps=[tab, tab, key("shift+tab")]),
        row("tab-back-round", steps=[tab, key("shift+tab")]),
        row("keys-right", steps=[tab, key("right"), key("right"), key("right")]),
        row("keys-left", steps=[tab, key("left")] + [key("left")] * 2, fov=40),
        row("keys-at-max", fov=110, steps=[tab, key("right")]),
        row("keys-enter-twice", steps=[tab, key("enter"), key("enter"), key("right")]),
        row("keys-enter-locks-shown", steps=[tab, key("enter")]),
        row("enter-unfocused", steps=[key("enter")]),
        row("click-fov", steps=[click(130)]), row("click-fov-release", steps=[click(130), release(130)]),
        row("click-fov-left", steps=[click(59), release(59)]),
        row("click-fov-right", steps=[click(208), release(208)]),
        row("drag-fov", steps=[click(70), drag(110), drag(150)]),
        row("drag-fov-release", steps=[click(70), drag(110), drag(150), release(150)]),
        row("drag-past-end", steps=[click(70), drag(900), release(900)]),
        row("drag-without-click", steps=[drag(150), release(150)]),
        row("click-empty", steps=[click(5, 5), release(5, 5)]),
        row("click-telemetry-asleep", steps=[click(100, 190), release(100, 190)]),
        row("click-fov-then-keys", steps=[click(130), release(130), key("right")]),
        row("click-fov-then-tab", steps=[click(130), release(130), tab]),
        row("tab-then-click-fov", steps=[tab, tab, click(130), release(130)]),
        # Arrows the slider does not take move the focus to the neighbour.
        row("arrow-down-from-nothing", steps=[key("down")]),
        row("arrow-up-from-nothing", steps=[key("up")]),
        row("arrow-right-from-nothing", steps=[key("right")]),
        row("arrow-left-from-nothing", steps=[key("left")]),
        row("arrow-down-from-fov", steps=[tab, key("down")]),
        row("arrow-down-twice", steps=[tab, key("down"), key("down")]),
        row("arrow-down-thrice", steps=[tab, key("down"), key("down"), key("down")]),
        row("arrow-down-to-done", steps=[tab] + [key("down")] * 7),
        row("arrow-down-past-done", steps=[tab] + [key("down")] * 9),
        row("arrow-up-from-done", steps=[tab, key("shift+tab"), key("up")]),
        row("arrow-up-from-fov", steps=[tab, key("up")]),
        row("arrow-right-from-button", steps=[tab, key("down"), key("right")]),
        row("arrow-left-from-button", steps=[tab, key("down"), key("right"), key("left")]),
        row("arrow-grid-down", steps=[tab, key("down"), key("down"), key("down"), key("up")]),
        row("arrow-grid-across", steps=[tab, key("down"), key("down"), key("right"), key("down"), key("left")]),
        row("arrow-over-asleep", steps=[tab] + [key("down")] * 6 + [key("right"), key("down")]),
        row("arrow-slider-takes-right", steps=[tab, key("enter"), key("right"), key("right")]),
        row("arrow-slider-left-not-locked", steps=[tab, key("enter"), key("enter"), key("left")]),
        row("arrow-after-click", steps=[click(130), release(130), key("down")]),
        row("arrow-wide-small", 200, 150, steps=[tab, key("down"), key("down"), key("right")]),
        row("arrow-wide-large", 640, 360, steps=[tab, key("down"), key("right"), key("down")]),
        row("arrow-out-of-world", in_world=False, steps=[key("down"), key("down"), key("right")]),
    ]
    yield {"case_id": "gui/options-screen", "family": "gui", "operation": "options_screen", "rows": rows}


def gui_video_settings_cases():
    """The video settings screen: the list of display, quality and preference
    options in several window sizes and GUI scales, scrolled with the wheel and
    the scroll bar, hovered, pressed (a cycle button, with and without Shift,
    a slider by a press and a drag), Control and the wheel through the GUI
    scales, and the preset turning to Custom when a quality option changes.
    Nothing presses the options whose reference effect needs a window (full
    screen) or a pipeline (the presets): those are drawn and not pressed."""
    def row(label, framebuffer=(854, 480), gui_scale=2, in_world=True, mouse=(0, 0), steps=(), ms=0):
        return {"label": label, "framebuffer": list(framebuffer), "gui_scale": gui_scale,
                "in_world": in_world, "mouse": list(mouse), "steps": list(steps), "ms": ms}

    click = lambda x, y, shift=False: {"do": "click", "x": x, "y": y, "shift": shift}
    drag = lambda x, y: {"do": "drag", "x": x, "y": y}
    release = lambda x=0, y=0: {"do": "release", "x": x, "y": y}
    wheel = lambda dy, x=200, y=100, ctrl=False: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": ctrl}
    rows = [
        row("in-world"), row("out-of-world", in_world=False),
        row("small", (640, 480)), row("large", (1280, 720)), row("tiny", (640, 480), 3),
        row("auto-large", (1920, 1080), 0), row("short", (854, 300)), row("scale-1", gui_scale=1),
        row("very-short", (640, 400), 4), row("wide", (1708, 480), in_world=False),
        row("hover-gui-scale", mouse=(250, 110)), row("hover-fullscreen-resolution", mouse=(200, 60)),
        row("hover-gamma", mouse=(100, 160)), row("hover-done", mouse=(213, 223)),
        row("hover-gap", mouse=(212, 110)), row("hover-first-pixel-of-last", mouse=(100, 206)),
        row("hover-under-header", mouse=(100, 30), steps=[wheel(-2)]),
        row("hover-header-line", mouse=(100, 40), steps=[wheel(-1)]),
        row("scroll-1", steps=[wheel(-1)]), row("scroll-5", steps=[wheel(-1)] * 5),
        row("scroll-10", steps=[wheel(-1)] * 10), row("scroll-end", steps=[wheel(-100)]),
        row("scroll-up-past-top", steps=[wheel(-3), wheel(10)]),
        row("scroll-fraction", steps=[wheel(-0.5)]), row("scroll-fraction-up", steps=[wheel(-4), wheel(1.5)]),
        row("scroll-over-header", steps=[wheel(-3, 200, 10)]),
        row("scroll-over-widget", steps=[wheel(-2, 250, 110)]),
        row("scroll-end-short", (854, 300), steps=[wheel(-100)]),
        row("scroll-end-very-short", (640, 400), 4, steps=[wheel(-100)]),
        row("scroll-end-large", (1280, 720), steps=[wheel(-100)]),
        row("scroll-end-out-of-world", in_world=False, steps=[wheel(-100)]),
        row("bar-drag", steps=[click(379, 60), drag(379, 90), release(379, 90)]),
        row("bar-drag-held", steps=[click(379, 60), drag(379, 90)]),
        row("bar-drag-past-bottom", steps=[click(379, 60), drag(379, 230)]),
        row("bar-drag-above", steps=[wheel(-5), click(379, 100), drag(379, 20)]),
        row("bar-drag-in-steps", steps=[click(379, 50), drag(379, 70), drag(379, 100), drag(379, 130)]),
        row("bar-click-only", steps=[click(379, 100), release(379, 100)]),
        row("bar-drag-short", (854, 300), steps=[click(379, 50), drag(379, 80)]),
        row("bar-drag-very-short", (640, 400), 4, steps=[click(100, 40), drag(100, 60)]),
        row("bar-edge-left", steps=[click(376, 60), drag(376, 100)]),
        row("bar-edge-right", steps=[click(382, 60), drag(382, 100)]),
        row("bar-beside", steps=[click(383, 60), drag(383, 100)]),
        row("gui-scale-press", steps=[click(250, 110)]),
        row("gui-scale-press-hover", mouse=(250, 110), steps=[click(250, 110)]),
        row("gui-scale-press-twice", steps=[click(250, 110), click(250, 110)]),
        row("gui-scale-press-thrice", steps=[click(250, 110)] * 3),
        row("gui-scale-shift", steps=[click(250, 110, True)]),
        row("gui-scale-from-auto", (1920, 1080), 0, steps=[click(300, 110)]),
        row("gui-scale-large-auto-twice", (1920, 1080), 0, steps=[click(300, 110), click(300, 110)]),
        row("gui-scale-then-scroll", steps=[click(250, 110), wheel(-3, 300, 100)]),
        row("gui-scale-scrolled", steps=[wheel(-5), click(250, 50)]),
        row("gui-scale-then-press-vsync", steps=[click(250, 110), click(250, 85)]),
        row("gui-scale-short", (854, 300), steps=[click(250, 110)]),
        row("gui-scale-shift-wraps", (1920, 1080), 0, steps=[click(300, 110, True)]),
        row("press-vsync", steps=[click(250, 85)]), row("press-vsync-away", mouse=(5, 5), steps=[click(250, 85)]),
        row("press-vsync-twice", steps=[click(250, 85), click(250, 85)]),
        row("press-vsync-shift", steps=[click(250, 85, True)]),
        row("press-inactivity", steps=[click(100, 110)]),
        row("press-exclusive", steps=[click(250, 135)]),
        row("press-backend", steps=[click(250, 160)]), row("press-backend-3", steps=[click(250, 160)] * 3),
        row("press-gap", steps=[click(212, 85)]), row("press-header-line", steps=[click(100, 40)]),
        row("press-outside-row", steps=[click(30, 85)]), row("press-under-header", steps=[click(100, 20)]),
        row("press-two-then-move-focus", steps=[click(250, 85), click(100, 110)]),
        row("press-slider-framerate", steps=[click(100, 85)]),
        row("press-slider-framerate-release", steps=[click(100, 85), release(100, 85)]),
        row("drag-slider-framerate", steps=[click(100, 85), drag(150, 85), release(150, 85)]),
        row("drag-slider-framerate-held", steps=[click(100, 85), drag(190, 85)]),
        row("drag-slider-framerate-end", steps=[click(100, 85), drag(300, 85), release(300, 85)]),
        row("drag-slider-framerate-start", steps=[click(100, 85), drag(0, 85), release(0, 85)]),
        row("press-slider-gamma", steps=[click(70, 160)]), row("drag-slider-gamma", steps=[click(70, 160), drag(60, 160)]),
        row("drag-slider-gamma-100", steps=[click(70, 160), drag(300, 160), release(300, 160)]),
        row("drag-slider-gamma-0", steps=[click(70, 160), drag(10, 160), release(10, 160)]),
        row("press-resolution-slider", steps=[click(100, 60)]),
        row("press-resolution-slider-drag", steps=[click(100, 60), drag(200, 60), release(200, 60)]),
        row("press-preset-last-pixel", steps=[click(170, 206)]),
        row("press-preset-last-pixel-release", steps=[click(170, 206), release(170, 206)]),
        row("press-preset-scrolled-in", steps=[wheel(-9), click(170, 100)]),
        row("press-ao-custom", steps=[wheel(-10), click(100, 170)]),
        row("press-ao-custom-twice", steps=[wheel(-10), click(100, 170), click(100, 170)]),
        row("press-shadows-custom", steps=[wheel(-10), click(100, 180)]),
        row("press-particles-custom", steps=[wheel(-10), click(300, 170)]),
        row("press-vsync-keeps-preset", steps=[wheel(-10), click(250, 20)]),
        row("press-weather-slider", steps=[wheel(-100), click(300, 110)]),
        row("press-weather-slider-drag", steps=[wheel(-100), click(300, 110), drag(250, 110), release(250, 110)]),
        row("press-cloud-range-clipped", steps=[wheel(-100), click(300, 40)]),
        row("press-cutout-leaves", steps=[wheel(-100), click(100, 60)]),
        row("press-transparency", steps=[wheel(-100), click(300, 60)]),
        row("anisotropy-asleep", steps=[wheel(-100)]),
        row("anisotropy-asleep-hover", mouse=(300, 86), steps=[wheel(-100)]),
        row("anisotropy-asleep-press", steps=[wheel(-100), click(300, 86)]),
        row("filtering-rgss", steps=[wheel(-100), click(100, 86)]),
        row("filtering-anisotropic", steps=[wheel(-100), click(100, 86), click(100, 86)]),
        row("filtering-anisotropic-press-slider", steps=[wheel(-100), click(100, 86)] * 2 + [click(300, 86)]),
        row("filtering-anisotropic-drag-slider",
            steps=[wheel(-100), click(100, 86), click(100, 86), click(250, 86), drag(340, 86), release(340, 86)]),
        row("filtering-back-to-none", steps=[wheel(-100)] + [click(100, 86)] * 3),
        row("press-autosave", steps=[wheel(-100), click(100, 165)]),
        row("press-vignette", steps=[wheel(-100), click(300, 165)]),
        row("press-attack", steps=[wheel(-100), click(100, 190)]),
        row("press-attack-shift", steps=[wheel(-100), click(100, 190, True)]),
        row("press-chunk-fade", steps=[wheel(-100), click(300, 190)]),
        row("drag-chunk-fade", steps=[wheel(-100), click(300, 190), drag(240, 190), release(240, 190)]),
        row("drag-chunk-fade-zero", steps=[wheel(-100), click(300, 190), drag(0, 190), release(0, 190)]),
        row("ctrl-wheel-down", steps=[wheel(-1, ctrl=True)]), row("ctrl-wheel-up-at-top", steps=[wheel(1, ctrl=True)]),
        row("ctrl-wheel-auto-down", (1920, 1080), 0, steps=[wheel(-1, ctrl=True)]),
        row("ctrl-wheel-auto-up", (1920, 1080), 0, steps=[wheel(1, ctrl=True)]),
        row("ctrl-wheel-down-twice", (1920, 1080), 3, steps=[wheel(-1, ctrl=True)] * 2),
        row("ctrl-wheel-down-thrice", (1920, 1080), 3, steps=[wheel(-1, ctrl=True)] * 3),
        row("ctrl-wheel-up", (1920, 1080), 2, steps=[wheel(1, ctrl=True)]),
        row("ctrl-wheel-scrolled", steps=[wheel(-5), wheel(-1, ctrl=True)]),
        row("ctrl-wheel-outside-list", steps=[wheel(-1, 200, 10, True)]),
        row("ctrl-wheel-fraction", (1920, 1080), 3, steps=[wheel(-0.5, ctrl=True)]),
        row("ctrl-wheel-focused", steps=[click(250, 85), wheel(-1, ctrl=True)]),
        row("ctrl-wheel-then-wheel", steps=[wheel(-1, ctrl=True), wheel(-3)]),
        row("ctrl-wheel-at-1", gui_scale=1, steps=[wheel(-1, ctrl=True)]),
    ]
    # Tooltips: a moment after the mouse is on the widget (a frame at 0, one at 10).
    frame = lambda ms: {"do": "frame", "ms": ms}
    tip = lambda label, mouse, steps=(), **kw: row("tip-" + label, mouse=mouse, steps=[frame(0)] + list(steps), ms=10, **kw)
    rows += [
        row("tip-first-frame", mouse=(100, 110)),
        tip("inactivity", (100, 110)), tip("exclusive", (250, 135)), tip("backend", (250, 160)),
        tip("gui-scale-none", (250, 110)), tip("resolution-none", (200, 60)),
        tip("preset", (150, 108), [wheel(-9)]), tip("preset-last-pixel", (150, 206)),
        tip("prioritize", (100, 146), [wheel(-10)]),
        tip("ao-none", (100, 170), [wheel(-10)]),
        tip("blurriness", (100, 40), [wheel(-100)]), tip("cutout-leaves", (100, 60), [wheel(-100)]),
        tip("transparency", (300, 60), [wheel(-100)]), tip("filtering", (100, 86), [wheel(-100)]),
        tip("anisotropy-asleep", (300, 86), [wheel(-100)]), tip("weather", (100, 110), [wheel(-100)]),
        tip("vignette", (300, 167), [wheel(-100)]), tip("chunk-fade", (300, 192), [wheel(-100)]),
        tip("attack-none", (100, 192), [wheel(-100)]),
        tip("exclusive-restart", (250, 135), [click(250, 135)]),
        tip("exclusive-back", (250, 135), [click(250, 135), click(250, 135)]),
        tip("backend-restart", (250, 160), [click(250, 160)]),
        tip("backend-vulkan", (250, 160), [click(250, 160), click(250, 160)]),
        tip("backend-default", (250, 160), [click(250, 160)] * 3),
        tip("inactivity-minimized", (100, 110), [click(100, 110)]),
        tip("filtering-rgss", (100, 86), [wheel(-100), click(100, 86)]),
        tip("filtering-anisotropic", (100, 86), [wheel(-100), click(100, 86), click(100, 86)]),
        tip("anisotropy-awake", (300, 86), [wheel(-100), click(100, 86), click(100, 86)]),
        tip("prioritize-by-player", (100, 146), [wheel(-10), click(100, 146)]),
        tip("prioritize-nearby", (100, 146), [wheel(-10), click(100, 146), click(100, 146)]),
        tip("preset-after-custom", (150, 108), [wheel(-10), click(100, 170)]),
        tip("small-screen", (100, 110), [], framebuffer=(640, 480)),
        tip("tiny-screen", (150, 70), [], framebuffer=(640, 480), gui_scale=3),
        tip("out-of-world", (100, 110), [], in_world=False),
        row("tip-late", mouse=(100, 110), steps=[frame(0), frame(300)], ms=1000),
        row("tip-moved-away", mouse=(5, 5), steps=[frame(0)], ms=10),
        row("tip-clipped-hover", mouse=(100, 30), steps=[frame(0), wheel(-2)], ms=10),
        row("tip-marquee-time", mouse=(100, 110), steps=[frame(0), wheel(-100)], ms=1234),
    ]
    yield {"case_id": "gui/video-settings-screen", "family": "gui", "operation": "video_settings_screen",
           "rows": rows}


def gui_chat_options_cases():
    """The chat settings screen: the eighteen options in a list, with the
    options' own values shown (every slider's label), the ones that hold a
    value on their widget, the narrator asleep (there is none), the list
    scrolled and its bar dragged, and sliders and cycle buttons pressed."""
    def row(label, framebuffer=(854, 480), gui_scale=2, in_world=True, mouse=(0, 0), steps=(),
            options=None, ms=0):
        row = {"label": label, "framebuffer": list(framebuffer), "gui_scale": gui_scale,
               "in_world": in_world, "mouse": list(mouse), "steps": list(steps), "ms": ms}
        if options:
            row["options"] = options
        return row

    click = lambda x, y, shift=False: {"do": "click", "x": x, "y": y, "shift": shift}
    drag = lambda x, y: {"do": "drag", "x": x, "y": y}
    release = lambda x=0, y=0: {"do": "release", "x": x, "y": y}
    wheel = lambda dy, x=200, y=100: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": False}
    opt = lambda **values: values
    rows = [
        row("in-world"), row("out-of-world", in_world=False), row("small", (640, 480)),
        row("large", (1280, 720)), row("tiny", (640, 480), 3), row("short", (854, 300)),
        row("auto-large", (1920, 1080), 0),
        row("hover-visibility", mouse=(100, 45)), row("hover-narrator-asleep", mouse=(100, 195)),
        row("hover-slider", mouse=(100, 95)), row("hover-done", mouse=(213, 223)),
        row("scroll-1", steps=[wheel(-1)]), row("scroll-5", steps=[wheel(-5)]),
        row("scroll-end", steps=[wheel(-100)]), row("scroll-end-short", (854, 300), steps=[wheel(-100)]),
        row("bar-drag", steps=[click(379, 60), drag(379, 90), release(379, 90)]),
        row("bar-drag-end", steps=[click(379, 60), drag(379, 230)]),
        row("opacity-zero", options=opt(opacity=0.0)), row("opacity-full", options=opt(opacity=1.0)),
        row("opacity-mid", options=opt(opacity=0.37)),
        row("background-zero", options=opt(background_opacity=0.0)),
        row("background-mid", options=opt(background_opacity=0.73)),
        row("scale-zero", options=opt(scale=0.0)), row("scale-mid", options=opt(scale=0.55)),
        row("spacing-full", options=opt(line_spacing=1.0)), row("spacing-mid", options=opt(line_spacing=0.25)),
        row("width-zero", options=opt(width=0.0)), row("width-mid", options=opt(width=0.5)),
        row("width-full", options=opt(width=1.0)),
        row("height-focused-low", options=opt(height_focused=0.0)),
        row("height-focused-mid", options=opt(height_focused=0.33)),
        row("height-unfocused-high", options=opt(height_unfocused=1.0)),
        row("everything", options=opt(opacity=0.2, background_opacity=0.9, scale=0.7, line_spacing=0.6,
                                       width=0.3, height_focused=0.8, height_unfocused=0.1)),
        row("press-visibility", steps=[click(100, 45)]),
        row("press-visibility-twice", steps=[click(100, 45)] * 2),
        row("press-visibility-thrice", steps=[click(100, 45)] * 3),
        row("press-visibility-shift", steps=[click(100, 45, True)]),
        row("press-colors", steps=[click(250, 45)]), row("press-links", steps=[click(100, 70)]),
        row("press-links-prompt", steps=[click(250, 70)]),
        row("press-narrator-asleep", steps=[click(100, 195)]),
        row("press-gap", steps=[click(212, 45)]),
        row("press-opacity", steps=[click(100, 95)]), row("press-opacity-end", steps=[click(205, 95)]),
        row("press-opacity-start", steps=[click(60, 95)]),
        row("drag-opacity", steps=[click(80, 95), drag(130, 95), release(130, 95)]),
        row("drag-opacity-held", steps=[click(80, 95), drag(180, 95)]),
        row("drag-opacity-past", steps=[click(80, 95), drag(400, 95), release(400, 95)]),
        row("press-background", steps=[click(300, 95)]),
        row("drag-background", steps=[click(230, 95), drag(300, 95), release(300, 95)]),
        row("press-scale", steps=[click(100, 120)]),
        row("drag-scale-zero", steps=[click(100, 120), drag(0, 120), release(0, 120)]),
        row("drag-scale-full", steps=[click(100, 120), drag(300, 120), release(300, 120)]),
        row("press-spacing", steps=[click(300, 120)]),
        row("press-delay", steps=[click(100, 145)]),
        row("drag-delay", steps=[click(70, 145), drag(120, 145), drag(160, 145), release(160, 145)]),
        row("drag-delay-end", steps=[click(70, 145), drag(300, 145), release(300, 145)]),
        row("press-width", steps=[click(300, 145)]),
        row("drag-width", steps=[click(230, 145), drag(300, 145), release(300, 145)]),
        row("drag-width-zero", steps=[click(300, 145), drag(0, 145), release(0, 145)]),
        row("press-height-focused", steps=[click(100, 170)]),
        row("drag-height-focused", steps=[click(70, 170), drag(150, 170), release(150, 170)]),
        row("press-height-unfocused", steps=[click(300, 170)]),
        row("drag-height-unfocused-zero", steps=[click(300, 170), drag(0, 170), release(0, 170)]),
        row("press-autosuggest", steps=[click(250, 195)]),
        row("scrolled-press-hide-matched", steps=[wheel(-100), click(100, 100)]),
        row("scrolled-press-reduced-debug", steps=[wheel(-100), click(250, 100)]),
        row("scrolled-press-secure", steps=[wheel(-100), click(100, 125)]),
        row("scrolled-press-drafts", steps=[wheel(-100), click(250, 125)]),
        row("scrolled-press-twice", steps=[wheel(-100), click(250, 125), click(250, 125)]),
        row("press-clipped-bottom", steps=[click(100, 206)]),
        row("press-two-in-turn", steps=[click(100, 45), click(250, 70), click(100, 95)]),
        # Tooltips of the widgets that have one.
        row("tip-hide-matched", mouse=(100, 167), steps=[{"do": "frame", "ms": 0}, wheel(-100)], ms=10),
        row("tip-hide-matched-seen", mouse=(100, 167), steps=[wheel(-100), {"do": "frame", "ms": 0}], ms=10),
        row("tip-secure", mouse=(100, 192), steps=[wheel(-100), {"do": "frame", "ms": 0}], ms=10),
        row("tip-drafts", mouse=(300, 192), steps=[wheel(-100), {"do": "frame", "ms": 0}], ms=10),
        row("tip-none-on-opacity", mouse=(100, 95), steps=[{"do": "frame", "ms": 0}], ms=10),
        row("tip-drafts-pressed", mouse=(300, 192), steps=[wheel(-100), {"do": "frame", "ms": 0}, click(300, 192)], ms=10),
    ]
    yield {"case_id": "gui/chat-options-screen", "family": "gui", "operation": "chat_options_screen",
           "rows": rows}


def gui_mouse_cases():
    """The mouse settings screen and the controls screen it opens from: their
    lists in several sizes, the sensitivity and wheel sliders at their ends
    and middle (with their special labels), the inversions, and presses and
    drags on every kind of widget. The buttons that open another screen are
    drawn and hovered, not pressed."""
    def row(label, framebuffer=(854, 480), gui_scale=2, in_world=True, mouse=(0, 0), steps=(),
            options=None, ms=0):
        row = {"label": label, "framebuffer": list(framebuffer), "gui_scale": gui_scale,
               "in_world": in_world, "mouse": list(mouse), "steps": list(steps), "ms": ms}
        if options:
            row["options"] = options
        return row

    click = lambda x, y, shift=False: {"do": "click", "x": x, "y": y, "shift": shift}
    drag = lambda x, y: {"do": "drag", "x": x, "y": y}
    release = lambda x=0, y=0: {"do": "release", "x": x, "y": y}
    wheel = lambda dy, x=200, y=100: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": False}
    opt = lambda **values: values
    mouse = [
        row("in-world"), row("out-of-world", in_world=False), row("small", (640, 480)),
        row("large", (1280, 720)), row("tiny", (640, 480), 3), row("short", (854, 240)),
        row("sensitivity-zero", options=opt(sensitivity=0.0)),
        row("sensitivity-full", options=opt(sensitivity=1.0)),
        row("sensitivity-mid", options=opt(sensitivity=0.37)),
        row("wheel-low", options=opt(wheel=0.01)), row("wheel-high", options=opt(wheel=10.0)),
        row("wheel-odd", options=opt(wheel=2.5)),
        row("inverted", options=opt(invert_x=True, invert_y=True)),
        row("inverted-x", options=opt(invert_x=True)),
        row("hover-sensitivity", mouse=(100, 45)), row("hover-done", mouse=(213, 223)),
        row("hover-gap", mouse=(212, 45)),
        row("press-sensitivity", steps=[click(100, 45)]),
        row("press-sensitivity-start", steps=[click(60, 45)]),
        row("press-sensitivity-end", steps=[click(205, 45)]),
        row("drag-sensitivity", steps=[click(80, 45), drag(150, 45), release(150, 45)]),
        row("drag-sensitivity-held", steps=[click(80, 45), drag(190, 45)]),
        row("drag-sensitivity-zero", steps=[click(100, 45), drag(0, 45), release(0, 45)]),
        row("drag-sensitivity-full", steps=[click(100, 45), drag(400, 45), release(400, 45)]),
        row("press-wheel", steps=[click(300, 45)]),
        row("drag-wheel", steps=[click(230, 45), drag(260, 45), release(260, 45)]),
        row("drag-wheel-low", steps=[click(300, 45), drag(0, 45), release(0, 45)]),
        row("drag-wheel-high", steps=[click(300, 45), drag(500, 45), release(500, 45)]),
        row("press-discrete", steps=[click(100, 70)]), row("press-discrete-twice", steps=[click(100, 70)] * 2),
        row("press-invert-x", steps=[click(250, 70)]), row("press-invert-y", steps=[click(100, 95)]),
        row("press-cursor", steps=[click(250, 95)]), row("press-shift", steps=[click(250, 70, True)]),
        row("press-gap", steps=[click(212, 70)]),
        row("scroll-short", (854, 240), steps=[wheel(-100)]),
        row("press-in-turn", steps=[click(250, 70), click(100, 95), click(100, 45)]),
        row("tip-cursor-changes", mouse=(250, 97), steps=[{"do": "frame", "ms": 0}], ms=10),
        row("tip-cursor-first", mouse=(250, 97), ms=0),
        row("tip-none-on-sensitivity", mouse=(100, 45), steps=[{"do": "frame", "ms": 0}], ms=10),
    ]
    controls = [
        row("in-world"), row("out-of-world", in_world=False), row("small", (640, 480)),
        row("large", (1280, 720)), row("short", (854, 240)), row("tiny", (640, 480), 3),
        row("hover-mouse-settings", mouse=(100, 45)), row("hover-key-binds", mouse=(300, 45)),
        row("hover-toggle", mouse=(100, 70)), row("hover-done", mouse=(213, 223)),
        row("press-toggle-sneak", steps=[click(100, 70)]),
        row("press-toggle-sneak-twice", steps=[click(100, 70)] * 2),
        row("press-toggle-sprint", steps=[click(250, 70)]),
        row("press-toggle-attack", steps=[click(100, 95)]),
        row("press-toggle-use", steps=[click(250, 95)]),
        row("press-shift-toggle", steps=[click(250, 95, True)]),
        row("press-auto-jump", steps=[click(100, 120)]),
        row("press-sprint-window", steps=[click(250, 120)]),
        row("drag-sprint-window", steps=[click(230, 120), drag(300, 120), release(300, 120)]),
        row("drag-sprint-window-zero", steps=[click(230, 120), drag(0, 120), release(0, 120)]),
        row("drag-sprint-window-end", steps=[click(230, 120), drag(400, 120), release(400, 120)]),
        row("press-operator-tab", steps=[click(100, 145)]),
        row("press-gap", steps=[click(212, 70)]),
        row("scroll-short", (854, 240), steps=[wheel(-100)]),
        row("press-in-turn", steps=[click(100, 70), click(250, 95), click(100, 145)]),
        row("tip-sprint-window", mouse=(250, 120), steps=[{"do": "frame", "ms": 0}], ms=10),
        row("tip-sprint-window-dragged", mouse=(250, 120),
            steps=[{"do": "frame", "ms": 0}, click(230, 120), drag(300, 120), release(300, 120)], ms=10),
        row("tip-none-on-buttons", mouse=(100, 45), steps=[{"do": "frame", "ms": 0}], ms=10),
    ]
    yield {"case_id": "gui/mouse-settings-screen", "family": "gui", "operation": "mouse_settings_screen",
           "rows": mouse}
    yield {"case_id": "gui/controls-screen", "family": "gui", "operation": "controls_screen",
           "rows": controls}


def gui_sound_options_cases():
    """The sound settings screen: its list in several sizes, the sliders at
    their ends and middle (with the "Off" label), the device, the subtitles,
    the directional audio and the music choices at their values and pressed,
    and the tooltips of the last three."""
    def row(label, framebuffer=(854, 480), gui_scale=2, in_world=True, mouse=(0, 0), steps=(),
            options=None, ms=0):
        row = {"label": label, "framebuffer": list(framebuffer), "gui_scale": gui_scale,
               "in_world": in_world, "mouse": list(mouse), "steps": list(steps), "ms": ms}
        if options:
            row["options"] = options
        return row

    click = lambda x, y, shift=False: {"do": "click", "x": x, "y": y, "shift": shift}
    drag = lambda x, y: {"do": "drag", "x": x, "y": y}
    release = lambda x=0, y=0: {"do": "release", "x": x, "y": y}
    wheel = lambda dy, x=200, y=100: {"do": "scroll", "x": x, "y": y, "dy": dy, "ctrl": False}
    frame = {"do": "frame", "ms": 0}
    opt = lambda **values: values
    rows = [
        row("in-world"), row("out-of-world", in_world=False), row("small", (640, 480)),
        row("large", (1280, 720)), row("tiny", (640, 480), 3), row("short", (854, 240)),
        row("tall", (854, 720)),
        row("master-off", options=opt(volumes={"master": 0.0})),
        row("master-mid", options=opt(volumes={"master": 0.37})),
        row("all-off", options=opt(volumes={n: 0.0 for n in (
            "master", "music", "record", "weather", "block", "hostile", "neutral", "player",
            "ambient", "voice", "ui")})),
        row("mixed", options=opt(volumes={"music": 0.5, "record": 0.25, "weather": 0.0, "block": 0.75,
                                          "hostile": 0.1, "neutral": 0.9, "player": 0.33})),
        row("subtitles-on", options=opt(subtitles=True)),
        row("directional-on", options=opt(directional=True)),
        row("frequent", options=opt(music_frequency="FREQUENT")),
        row("constant", options=opt(music_frequency="CONSTANT")),
        row("toast-pause", options=opt(music_toast="pause")),
        row("toast-both", options=opt(music_toast="pause_and_toast")),
        row("hover-master", mouse=(100, 45)), row("hover-music", mouse=(100, 70)),
        row("hover-gap", mouse=(212, 70)), row("hover-done", mouse=(213, 223)),
        row("press-master", steps=[click(100, 45)]),
        row("press-master-start", steps=[click(60, 45)]),
        row("press-master-end", steps=[click(365, 45)]),
        row("drag-master", steps=[click(100, 45), drag(200, 45), release(200, 45)]),
        row("drag-master-held", steps=[click(100, 45), drag(300, 45)]),
        row("drag-master-zero", steps=[click(100, 45), drag(0, 45), release(0, 45)]),
        row("drag-master-full", steps=[click(100, 45), drag(500, 45), release(500, 45)]),
        row("press-music", steps=[click(100, 70)]), row("press-record", steps=[click(300, 70)]),
        row("drag-weather", steps=[click(100, 95), drag(60, 95), release(60, 95)]),
        row("drag-block", steps=[click(300, 95), drag(250, 95), release(250, 95)]),
        row("press-hostile", steps=[click(100, 120)]), row("press-neutral", steps=[click(300, 120)]),
        row("press-player", steps=[click(100, 145)]), row("press-ambient", steps=[click(300, 145)]),
        row("press-voice", steps=[click(100, 170)]), row("press-ui", steps=[click(300, 170)]),
        row("press-gap", steps=[click(212, 70)]),
        row("scroll-bottom", steps=[wheel(-100)]),
        row("scroll-short", (854, 240), steps=[wheel(-100)]),
        row("scroll-short-hover", (854, 240), mouse=(100, 60), steps=[wheel(-100)]),
        row("scroll-tall-up", (854, 240), steps=[wheel(-100), wheel(100)]),
        row("press-device-scrolled", steps=[wheel(-100), click(100, 140)]),
        row("press-subtitles", steps=[wheel(-100), click(100, 165)]),
        row("press-directional", steps=[wheel(-100), click(300, 165)]),
        row("press-frequency", steps=[wheel(-100), click(100, 190)]),
        row("press-frequency-twice", steps=[wheel(-100), click(100, 190), click(100, 190)]),
        row("press-frequency-shift", steps=[wheel(-100), click(100, 190, True)]),
        row("press-toast", steps=[wheel(-100), click(300, 190)]),
        row("press-toast-thrice", steps=[wheel(-100), click(300, 190), click(300, 190), click(300, 190)]),
        row("tip-subtitles", mouse=(100, 165), steps=[wheel(-100), frame], ms=10),
        row("tip-directional", mouse=(300, 165), steps=[wheel(-100), frame], ms=10),
        row("tip-directional-on", mouse=(300, 165), steps=[wheel(-100), frame],
            options=opt(directional=True), ms=10),
        row("tip-frequency", mouse=(100, 190), steps=[wheel(-100), frame], ms=10),
        row("tip-toast", mouse=(300, 190), steps=[wheel(-100), frame], ms=10),
        row("tip-toast-both", mouse=(300, 190), steps=[wheel(-100), frame],
            options=opt(music_toast="pause_and_toast"), ms=10),
        row("tip-none-on-slider", mouse=(100, 45), steps=[frame], ms=10),
        row("tip-none-on-device", mouse=(100, 140), steps=[wheel(-100), frame], ms=10),
        row("tab-first", steps=[{"do": "key", "key": "tab"}]),
        row("tab-twice", steps=[{"do": "key", "key": "tab"}] * 2),
        row("tab-back", steps=[{"do": "key", "key": "shift+tab"}]),
        row("arrow-right", steps=[{"do": "key", "key": "tab"}, {"do": "key", "key": "right"}]),
        row("arrow-left", steps=[{"do": "key", "key": "tab"}, {"do": "key", "key": "left"}]),
        row("arrow-down-twice", steps=[{"do": "key", "key": "tab"}, {"do": "key", "key": "down"},
                                       {"do": "key", "key": "down"}]),
        row("press-in-turn", steps=[click(100, 45), click(300, 70), click(100, 95)]),
    ]
    yield {"case_id": "gui/sound-options-screen", "family": "gui", "operation": "sound_options_screen",
           "rows": rows}


def gui_key_mappings_cases():
    """The key mappings, their categories and the keys' names, as a fresh `Options` has them."""
    yield {"case_id": "gui/key-mappings", "family": "gui", "operation": "key_mappings"}
