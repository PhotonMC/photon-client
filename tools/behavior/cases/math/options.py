"""The `math` family: the `options` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    def character_event_case(label, codepoint):
        return {
            "case_id": f"character-event/{label}", "family": "math",
            "operation": "character_event", "codepoint": codepoint,
        }

    character_event_cases = [
        character_event_case("nul", 0),
        character_event_case("space", 32),
        character_event_case("ascii", 65),
        character_event_case("newline", 10),
        character_event_case("delete", 127),
        character_event_case("section-sign", 167),
        character_event_case("accented", 0x00E9),
        character_event_case("surrogate", 0xD800),
        character_event_case("supplementary", 0x1F600),
        character_event_case("maximum", 0x10FFFF),
    ]

    yield from character_event_cases

    def preedit_event_case(label, codepoints, block_sizes, focused_block, caret):
        return {
            "case_id": f"preedit-event/{label}", "family": "math",
            "operation": "preedit_event", "codepoints": codepoints,
            "block_sizes": block_sizes, "focused_block": focused_block, "caret": caret,
        }

    preedit_event_cases = [
        preedit_event_case("empty", [], [0], 0, 0),
        preedit_event_case("single-block", [65, 66], [2], 0, 1),
        preedit_event_case("multiple-blocks", [65, 66, 67], [1, 2], 1, 2),
        preedit_event_case("supplementary-caret", [0x1F600, 0x00E9, 66], [2, 1], 0, 1),
        preedit_event_case("block-boundary-caret", [65, 0x1F600, 66], [1, 2], 1, 1),
        preedit_event_case("end-caret", [65, 0x1F600], [2], 0, 2),
        preedit_event_case("negative-block-size", [65], [-1, 1], 1, 0),
        preedit_event_case("ignored-trailing-codepoints", [65, 66, 67], [1], 0, 2),
    ]

    yield from preedit_event_cases

    def option_values_case(label, minimum, maximum, apply_value_immediately,
                           int_values, sliders, unit_values):
        return {
            "case_id": f"option-values/{label}", "family": "math",
            "operation": "option_values", "min": minimum, "max": maximum,
            "apply_value_immediately": apply_value_immediately, "int_values": int_values,
            "sliders": [float_bits(value, "f64") for value in sliders],
            "unit_values": [float_bits(value, "f64") for value in unit_values],
        }

    option_value_cases = [
        option_values_case(
            "zero-to-ten", 0, 10, True, [-1, 0, 1, 5, 10, 11],
            [-float("inf"), -0.2, 0.0, 0.05, 0.5, 0.99999, 1.0, 1.5, float("inf"), float("nan")],
            [-0.1, -0.0, 0.0, 0.5, 1.0, 1.1, float("inf"), float("nan")]),
        option_values_case(
            "negative-range", -3, 3, False, [-4, -3, -2, 0, 3, 4],
            [-0.5, 0.0, 0.25, 0.5, 0.75, 1.0],
            [0.0, 0.25, 1.0]),
        option_values_case(
            "integer-edge", 2_147_483_646, 2_147_483_647, True,
            [2_147_483_646, 2_147_483_647], [0.0, 0.5, 0.99999, 1.0], [0.0, 1.0]),
    ]

    yield from option_value_cases

    def sliderable_enum_case(label, probe_values, sliders):
        return {
            "case_id": f"sliderable-enum/{label}", "family": "math",
            "operation": "sliderable_enum",
            "probes": [{"value": value} for value in probe_values],
            "sliders": [float_bits(value, "f64") for value in sliders],
        }

    yield sliderable_enum_case(
        "graphics-presets",
        ["FAST", "FANCY", "FABULOUS", "CUSTOM"],
        [
            -float("inf"), -0.0, 0.0, 0.124999999999, 0.125, 0.249999999999,
            0.25, 0.5, 0.75, 0.99999, 1.0, 1.5, float("inf"), float("nan"),
        ],
    )

    def cycleable_enum_case(label, values, probes):
        return {
            "case_id": f"cycleable-enum/{label}", "family": "math",
            "operation": "cycleable_enum", "values": values, "probes": probes,
        }

    yield cycleable_enum_case(
        "membership-and-order",
        ["off", "fast", "fancy", "fabulous"],
        ["off", "fabulous", "missing", ""],
    )

    yield cycleable_enum_case(
        "duplicate-values",
        ["first", "first", "last"],
        ["first", "last", "outside"],
    )

    def alt_enum_case(label, values, alt_values, conditions, probes):
        return {
            "case_id": f"alt-enum/{label}", "family": "math",
            "operation": "alt_enum", "values": values, "alt_values": alt_values,
            "conditions": conditions, "probes": probes,
        }

    yield alt_enum_case(
        "condition-switch",
        ["default", "shared", "only-default"],
        ["alternate", "shared", "only-alternate"],
        [False, True, False, True],
        ["default", "alternate", "shared", "only-alternate", "missing"],
    )

    yield alt_enum_case(
        "duplicates-and-empty-alternative",
        ["first", "first", "last"],
        [],
        [True, False, True, False],
        ["first", "last", "outside"],
    )

    def lazy_enum_case(label, value_lists, valid_values, probes):
        return {
            "case_id": f"lazy-enum/{label}", "family": "math",
            "operation": "lazy_enum", "value_lists": value_lists,
            "valid_values": valid_values, "probes": probes,
        }

    yield lazy_enum_case(
        "dynamic-values",
        [[""], ["", "speakers"], ["", "headphones"]],
        ["", "speakers"],
        ["", "speakers", "headphones", "missing"],
    )

    yield lazy_enum_case(
        "validation-independent-of-list",
        [["default", "alternate"], ["default"]],
        ["default", "external"],
        ["default", "alternate", "external"],
    )

    def clamping_lazy_max_int_range_case(label, minimum, max_values, encodable_max, probes, sliders):
        return {
            "case_id": f"clamping-lazy-max-int-range/{label}", "family": "math",
            "operation": "clamping_lazy_max_int_range", "min": minimum,
            "max_values": max_values, "encodable_max": encodable_max,
            "probes": probes, "sliders": [float_bits(value, "f64") for value in sliders],
        }

    yield clamping_lazy_max_int_range_case(
        "dynamic-upper-bound", -2, [3, 5], 7,
        [-3, -2, -1, 0, 3, 5, 6],
        [-float("inf"), -0.2, 0.0, 0.1, 0.5, 0.99999, 1.0, 1.5, float("inf"), float("nan")],
    )

    def mapped_int_range_case(label, minimum, maximum, discrete, values, sliders):
        return {
            "case_id": f"mapped-int-range/{label}", "family": "math",
            "operation": "mapped_int_range", "min": minimum, "max": maximum,
            "discrete": discrete, "values": values,
            "sliders": [float_bits(value, "f64") for value in sliders],
        }

    yield mapped_int_range_case(
        "discrete",
        -2,
        3,
        True,
        ["-3", "-2", "-1", "0", "2", "3", "4"],
        [-float("inf"), -0.2, 0.0, 0.1, 0.5, 0.99999, 1.0, 1.5, float("inf"), float("nan")],
    )

    yield mapped_int_range_case(
        "continuous",
        0,
        4,
        False,
        ["-1", "0", "2", "4", "5"],
        [-0.25, 0.0, 0.2, 0.5, 1.0],
    )

    def mapped_unit_double_case(label, values, sliders):
        return {
            "case_id": f"mapped-unit-double/{label}", "family": "math",
            "operation": "mapped_unit_double",
            "values": [float_bits(value, "f64") for value in values],
            "sliders": [float_bits(value, "f64") for value in sliders],
        }

    yield mapped_unit_double_case(
        "square-root",
        [-float("inf"), -1.0, -0.0, 0.0, 0.25, 0.5, 1.0, 1.0000000001, float("inf"), float("nan")],
        [-float("inf"), -0.25, -0.0, 0.0, 0.25, 0.5, 1.0, 1.0000000001, float("inf"), float("nan")],
    )

    yield mapped_unit_double_case(
        "rounding-and-subnormal",
        [5e-324, 0.1, 1.0 / 3.0, 0.9999999999999999],
        [5e-324, 0.1, 1.0 / 3.0, 0.9999999999999999],
    )

    def scroll_wheel_case(label, scrolls, selections):
        return {
            "case_id": f"scroll-wheel/{label}", "family": "math",
            "operation": "scroll_wheel",
            "scrolls": [{"x": float_bits(x, "f64"), "y": float_bits(y, "f64")} for x, y in scrolls],
            "selections": [
                {"wheel": float_bits(wheel, "f64"), "current": current, "limit": limit}
                for wheel, current, limit in selections
            ],
        }

    scroll_wheel_cases = [
        scroll_wheel_case(
            "fractional-and-both-axes",
            [(0.4, 0.0), (0.4, 0.6), (0.4, 0.6), (-0.2, -0.6), (-1.0, -1.0)],
            [(1.0, 0, 9), (-1.0, 0, 9), (0.0, 4, 9), (float("nan"), 4, 9),
             (float("inf"), 0, 9), (float("-inf"), 0, 9)],
        ),
        scroll_wheel_case(
            "negative-and-selection-wrap",
            [(-0.6, -0.6), (-0.6, -0.6), (-0.8, -0.8), (0.1, 0.1), (1.0, 0.0)],
            [(1.0, 0, 9), (-1.0, 0, 9), (1.0, 8, 9), (-1.0, 0, 9),
             (float("nan"), -1, 9), (float("inf"), 8, 9)],
        ),
    ]

    yield from scroll_wheel_cases

    def smooth_double_case(label, steps):
        return {
            "case_id": f"smooth-double/{label}", "family": "math",
            "operation": "smooth_double", "steps": steps,
        }

    def smooth_delta(target_delta, time):
        return {
            "op": "delta", "target_delta": float_bits(target_delta, "f64"),
            "time": float_bits(time, "f64"),
        }

    smooth_double_cases = [
        smooth_double_case("positive-negative", [
            smooth_delta(1.0, 1.0), smooth_delta(1.0, 0.5), smooth_delta(-0.25, 1.0),
            smooth_delta(0.0, 2.0), smooth_delta(-2.0, 0.25),
        ]),
        smooth_double_case("reset", [
            smooth_delta(4.0, 0.25), {"op": "reset"}, smooth_delta(-1.0, 1.0),
            smooth_delta(0.5, 0.0), {"op": "reset"}, smooth_delta(0.0, 1.0),
        ]),
        smooth_double_case("signed-zero", [
            smooth_delta(-0.0, 1.0), smooth_delta(0.0, -0.0), smooth_delta(-1.0, -0.5),
        ]),
        smooth_double_case("special-values", [
            smooth_delta(float("nan"), 1.0), smooth_delta(1.0, 1.0),
            {"op": "reset"}, smooth_delta(float("inf"), 0.5),
            smooth_delta(float("-inf"), 1.0),
        ]),
    ]

    yield from smooth_double_cases

    def input_event_case(label, key, scancode, modifiers, mouse_button, mouse_x, mouse_y):
        return {
            "case_id": f"input-with-modifiers/{label}", "family": "math",
            "operation": "input_with_modifiers", "key": key, "scancode": scancode,
            "modifiers": modifiers, "mouse_button": mouse_button,
            "mouse_x": float_bits(mouse_x, "f64"), "mouse_y": float_bits(mouse_y, "f64"),
        }

    input_event_cases = [
        input_event_case("enter-selection", 257, 1, 0, 0, 1.25, -2.5),
        input_event_case("space-selection", 32, 2, 0, 1, 0.0, 0.0),
        input_event_case("keypad-enter-selection", 335, 3, 0, 2, -0.0, 3.5),
        input_event_case("escape", 256, 4, 0, 3, 4.5, -0.0),
        input_event_case("left", 263, 5, 0, 4, 5.5, 6.5),
        input_event_case("right", 262, 6, 0, 5, 7.5, 8.5),
        input_event_case("up", 265, 7, 0, 6, 9.5, 10.5),
        input_event_case("down", 264, 8, 0, 7, 11.5, 12.5),
        input_event_case("cycle-focus", 258, 9, 0, 0, 13.5, 14.5),
        input_event_case("digit-zero", 48, 10, 0, 1, 15.5, 16.5),
        input_event_case("digit-nine", 57, 11, 0, 2, 17.5, 18.5),
        input_event_case("non-digit", 58, 12, 0, 3, 19.5, 20.5),
        input_event_case("select-all", 65, 13, 2, 4, 21.5, 22.5),
        input_event_case("copy", 67, 14, 2, 5, 23.5, 24.5),
        input_event_case("paste", 86, 15, 2, 6, 25.5, 26.5),
        input_event_case("cut", 88, 16, 2, 7, 27.5, 28.5),
        input_event_case("shortcut-with-shift", 65, 17, 3, 0, 29.5, 30.5),
        input_event_case("modifier-bits", 0, 18, 15, 1, 31.5, 32.5),
        input_event_case("super-only", 65, 19, 8, 2, 33.5, 34.5),
        input_event_case("integer-underflow", -2147483648, 20, 0, 3, 35.5, 36.5),
    ]

    yield from input_event_cases

    def key_spec(kind, value):
        return {"type": kind, "value": value}

    def key_mapping_case(label, primary, secondary, other, rebind, actions,
                         key_event, unknown_key_event, mouse_button):
        return {
            "case_id": f"key-mapping/state/{label}", "family": "math",
            "operation": "key_mapping", "primary": primary, "secondary": secondary,
            "other": other, "rebind": rebind, "actions": actions,
            "key_event": key_event, "unknown_key_event": unknown_key_event,
            "mouse_button": mouse_button,
        }

    key_mapping_cases = []

    base = 1000

    key_mapping_cases.append(key_mapping_case(
        "shared-press-release",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("keysym", base + 1),
        key_spec("scancode", base + 2),
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "set", "key": "primary", "value": False},
         {"op": "click", "key": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "secondary"},
         {"op": "set_down", "target": "other", "value": True},
         {"op": "release_all"},
         {"op": "consume_click", "target": "secondary"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 20))

    base += 100

    key_mapping_cases.append(key_mapping_case(
        "repeated-clicks",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 1),
        key_spec("scancode", base + 2),
        [{"op": "click", "key": "primary"},
         {"op": "click", "key": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "secondary"},
         {"op": "consume_click", "target": "secondary"},
         {"op": "consume_click", "target": "secondary"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 1))

    base += 100

    key_mapping_cases.append(key_mapping_case(
        "rebind-keeps-old-table",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 10),
        key_spec("scancode", base + 20),
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "set_key", "target": "primary", "key": "rebind"},
         {"op": "set", "key": "primary_default", "value": False},
         {"op": "set", "key": "primary", "value": True},
         {"op": "click", "key": "primary"},
         {"op": "click", "key": "primary_default"},
         {"op": "consume_click", "target": "primary"},
         {"op": "consume_click", "target": "secondary"},
         {"op": "release_all"},
         {"op": "set_key", "target": "primary", "key": "rebind"}],
        {"key": base, "scancode": base + 20},
        {"key": -1, "scancode": base + 20}, base + 10))

    base += 100

    key_mapping_cases.append(key_mapping_case(
        "default-and-unbound",
        key_spec("keysym", -1), key_spec("keysym", -1), key_spec("mouse", base),
        key_spec("keysym", base + 1),
        [{"op": "set_key", "target": "primary", "key": "rebind"}],
        {"key": -1, "scancode": base + 2},
        {"key": -1, "scancode": base + 2}, base))

    base += 100

    key_mapping_cases.append(key_mapping_case(
        "event-matching",
        key_spec("keysym", base), key_spec("scancode", base + 1), key_spec("mouse", base + 2),
        key_spec("keysym", base + 3), [],
        {"key": base, "scancode": base + 1},
        {"key": -1, "scancode": base + 1}, base + 2))

    key_mapping_cases.append(key_mapping_case(
        "known-key-names",
        key_spec("keysym", 87), key_spec("keysym", 65), key_spec("mouse", 0),
        key_spec("keysym", 32), [],
        {"key": 87, "scancode": 65}, {"key": -1, "scancode": 65}, 0))

    def toggle_key_mapping_case(label, primary, secondary, other, settings, actions,
                                key_event, unknown_key_event, mouse_button):
        return {
            "case_id": f"toggle-key-mapping/state/{label}", "family": "math",
            "operation": "toggle_key_mapping", "primary": primary, "secondary": secondary,
            "other": other, **settings, "actions": actions,
            "key_event": key_event, "unknown_key_event": unknown_key_event,
            "mouse_button": mouse_button,
        }

    toggle_key_mapping_cases = []

    base = 1500

    toggle_key_mapping_cases.append(toggle_key_mapping_case(
        "toggle-and-restore",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 1),
        {"primary_needs_toggle": True, "primary_should_restore": True,
         "secondary_needs_toggle": True, "secondary_should_restore": False,
         "other_needs_toggle": False, "other_should_restore": True},
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "set_down", "target": "primary", "value": True},
         {"op": "set", "key": "primary", "value": True},
         {"op": "click", "key": "primary"},
         {"op": "consume_click", "target": "primary"},
         {"op": "release_all"},
         {"op": "restore_toggle_states"},
         {"op": "should_restore", "target": "primary"},
         {"op": "reset_toggle_keys"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 1))

    base += 100

    toggle_key_mapping_cases.append(toggle_key_mapping_case(
        "scancode-does-not-restore",
        key_spec("scancode", base), key_spec("scancode", base), key_spec("keysym", base + 1),
        {"primary_needs_toggle": True, "primary_should_restore": True,
         "secondary_needs_toggle": True, "secondary_should_restore": True,
         "other_needs_toggle": True, "other_should_restore": True},
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "release_all"},
         {"op": "restore_toggle_states"},
         {"op": "should_restore", "target": "primary"},
         {"op": "reset_toggle_keys"}],
        {"key": -1, "scancode": base},
        {"key": -1, "scancode": base}, base + 1))

    base += 100

    toggle_key_mapping_cases.append(toggle_key_mapping_case(
        "direct-set-mode",
        key_spec("keysym", base), key_spec("keysym", base), key_spec("mouse", base + 1),
        {"primary_needs_toggle": False, "primary_should_restore": True,
         "secondary_needs_toggle": False, "secondary_should_restore": True,
         "other_needs_toggle": False, "other_should_restore": True},
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "set_down", "target": "primary", "value": False},
         {"op": "set", "key": "primary", "value": True},
         {"op": "release_all"},
         {"op": "restore_toggle_states"},
         {"op": "should_restore", "target": "primary"},
         {"op": "reset_toggle_keys"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 1))

    base += 100

    toggle_key_mapping_cases.append(toggle_key_mapping_case(
        "toggle-without-restore",
        key_spec("keysym", base), key_spec("keysym", base + 1), key_spec("mouse", base + 2),
        {"primary_needs_toggle": True, "primary_should_restore": False,
         "secondary_needs_toggle": True, "secondary_should_restore": False,
         "other_needs_toggle": True, "other_should_restore": False},
        [{"op": "set_down", "target": "primary", "value": True},
         {"op": "release_all"},
         {"op": "restore_toggle_states"},
         {"op": "should_restore", "target": "primary"},
         {"op": "reset_toggle_keys"}],
        {"key": base, "scancode": base + 10},
        {"key": -1, "scancode": base + 10}, base + 2))

    yield from key_mapping_cases

    yield from toggle_key_mapping_cases

    keyboard_inputs = {
        "empty": {},
        "forward": {"up": True},
        "backward": {"down": True},
        "left": {"left": True},
        "right": {"right": True},
        "forward-left": {"up": True, "left": True},
        "forward-right": {"up": True, "right": True},
        "backward-left": {"down": True, "left": True},
        "backward-right": {"down": True, "right": True},
        "forward-backward": {"up": True, "down": True},
        "left-right": {"left": True, "right": True},
        "all-movement": {"up": True, "down": True, "left": True, "right": True},
        "jump": {"jump": True},
        "shift": {"shift": True},
        "sprint": {"sprint": True},
        "movement-and-flags": {
            "up": True, "right": True, "jump": True, "shift": True, "sprint": True,
        },
    }

    keyboard_defaults = {
        "up": False, "down": False, "left": False, "right": False,
        "jump": False, "shift": False, "sprint": False,
    }

    for label, values in keyboard_inputs.items():
        keys = {**keyboard_defaults, **values}
        yield {"case_id": f"keyboard-input/tick/{label}", "family": "math",
               "operation": "keyboard_input", **keys}

    coords = [0, 1, 15, 16, 17, -1, -15, -16, -17, 31, 32, -32, -33,
              2147483647, -2147483648, 33554431, -33554432, 2097151, -2097152]

    for fn in ("block_to_section", "section_to_block", "section_relative"):
        for value in coords:
            yield {"case_id": f"coord/{fn}/{value}", "family": "math", "operation": "coord",
                   "fn": fn, "value": str(value)}

    for width in ("f32", "f64"):
        specials = {float_bits(x, width) for x in (0.0, -0.0, float("inf"), -float("inf"), float("nan"))}
        specials.update({"00000001", "80000001"} if width == "f32"
                        else {"0000000000000001", "8000000000000001"})
        for operation, centers in [("floor", (-2, -1, 1, 2, -(2**31), 2**31)),
                                   ("wrap_degrees", (-720, -360, -180, 180, 360, 720, 100000))]:
            values = specials | {bits for value in centers for bits in neighbors(value, width)}
            for bits in sorted(values):
                yield {"case_id": f"math/{operation}/{width}/{bits}", "family": "math",
                       "operation": operation, "type": width, "value": bits}
        for low, high in [(-1.0, 1.0), (0.0, 1.0)]:
            values = specials | {bits for value in (1.0, -1.0) for bits in neighbors(value, width)}
            for bits in sorted(values):
                yield {"case_id": f"math/clamp/{width}/{low}/{bits}", "family": "math",
                       "operation": "clamp", "type": width, "value": bits,
                       "min": float_bits(low, width), "max": float_bits(high, width)}
