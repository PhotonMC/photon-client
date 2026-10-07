"""The `window` family: original synthetic inputs."""

from cases.helpers import *


def window_cases():
    """Java `Window` GUI scale selection and scaled-size rounding.

    `calculateScale` walks the scale up while both axes still leave at least the
    320x240 base area, then rounds odd scales up when unicode is enforced.
    `setGuiScale` divides the framebuffer by the scale as a double and rounds the
    result up whenever the division left a remainder, so the interesting inputs
    are the ones that divide exactly, the ones that do not, and the scale
    boundaries on each axis.
    """
    yield {
        "case_id": "window/base-size",
        "family": "window",
        "operation": "base_size",
    }

    yield {
        "case_id": "window/min-size",
        "family": "window",
        "operation": "min_size",
        "rows": [
            {"label": label, "requested": requested}
            for label, requested in [
                ("zero", 0),
                ("one", 1),
                ("negative", -1),
                ("min-int", -2147483648),
                ("default-width", 854),
                ("default-height", 480),
                ("max-int", 2147483647),
            ]
        ],
    }

    rows = [
        # label, framebuffer width, height, max scale, enforce unicode
        ("auto-1920x1080", 1920, 1080, 0, False),
        ("auto-2560x1440", 2560, 1440, 0, False),
        ("auto-3840x2160", 3840, 2160, 0, False),
        ("auto-1366x768", 1366, 768, 0, False),
        ("auto-800x600", 800, 600, 0, False),
        ("auto-1024x768", 1024, 768, 0, False),
        # exact base area and its neighbours drive the loop guard
        ("base-exact", 320, 240, 0, False),
        ("base-minus-one-x", 319, 240, 0, False),
        ("base-minus-one-y", 320, 239, 0, False),
        ("base-double", 640, 480, 0, False),
        ("base-double-minus-one", 639, 479, 0, False),
        # degenerate framebuffers still have to produce a usable scale
        ("tiny-1x1", 1, 1, 0, False),
        ("tiny-2x2", 2, 2, 0, False),
        ("narrow-320x2160", 320, 2160, 0, False),
        ("wide-3840x240", 3840, 240, 0, False),
        # capped scales stop the loop before the area guard does
        ("cap-1", 1920, 1080, 1, False),
        ("cap-2", 1920, 1080, 2, False),
        ("cap-3", 1920, 1080, 3, False),
        ("cap-4", 1920, 1080, 4, False),
        ("cap-above-auto", 1920, 1080, 9, False),
        # unicode rounds an odd result up, including past the cap
        ("unicode-even-result", 1920, 1080, 0, True),
        ("unicode-odd-result", 1000, 700, 0, True),
        ("unicode-cap-1", 1920, 1080, 1, True),
        ("unicode-cap-3", 1920, 1080, 3, True),
        ("unicode-base-exact", 320, 240, 0, True),
        ("unicode-tiny", 1, 1, 0, True),
        # non-divisible sizes exercise the scaled-size rounding branch
        ("remainder-1001x701", 1001, 701, 0, False),
        ("remainder-1920x1081", 1920, 1081, 0, False),
        ("remainder-cap-3", 1001, 701, 3, False),
        ("remainder-cap-7", 1001, 701, 7, False),
        ("divides-exactly", 1280, 720, 4, False),
    ]
    yield {
        "case_id": "window/gui-scale",
        "family": "window",
        "operation": "gui_scale",
        "rows": [
            {
                "label": label,
                "framebuffer_width": width,
                "framebuffer_height": height,
                "max_scale": max_scale,
                "enforce_unicode": enforce_unicode,
            }
            for label, width, height, max_scale, enforce_unicode in rows
        ],
    }
