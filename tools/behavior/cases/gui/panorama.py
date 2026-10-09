"""The `gui` family: the title screen's panorama."""

from cases.helpers import float_bits


def gui_panorama_cases():
    """The cube's model-view at the title screen's pitch and a spread of spins, and the spin one
    frame adds at several frame lengths and speeds."""
    for label, spin in [("zero", 0.0), ("first-tick", -0.1), ("quarter", -90.0),
                        ("fraction", -45.37), ("wrapped-limit", 179.95), ("negative-limit", -180.0),
                        ("small", 0.0123)]:
        yield {"case_id": f"gui/panorama-view/model-view/{label}", "family": "gui",
               "operation": "panorama_view", "rot_x": float_bits(10.0, "f32"),
               "rot_y": float_bits(spin, "f32"), "spin": float_bits(0.0, "f32"),
               "ticks": float_bits(1.0, "f32"), "speed": float_bits(1.0, "f64")}
    for label, spin, ticks, speed in [("one-tick", 0.0, 1.0, 1.0), ("half-speed", 12.5, 0.4, 0.5),
                                      ("zero-speed", 33.0, 1.0, 0.0), ("wraps-up", 179.9, 2.0, 1.0),
                                      ("wraps-down", -179.9, 3.0, -1.0), ("fractional", 7.77, 0.37, 0.83)]:
        yield {"case_id": f"gui/panorama-view/spin/{label}", "family": "gui",
               "operation": "panorama_view", "rot_x": float_bits(10.0, "f32"),
               "rot_y": float_bits(0.0, "f32"), "spin": float_bits(spin, "f32"),
               "ticks": float_bits(ticks, "f32"), "speed": float_bits(speed, "f64")}
