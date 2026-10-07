"""The sky flash cases."""

from cases.helpers import *


def sky_flash_cases():
    """The sky colour and the sky light factor with and without a lightning flash, with the
    option that hides it, from the real client level's attribute system."""
    rows = []
    for flash in (0, 1, 2):
        for hide in (False, True):
            rows.append({"label": f"flash-{flash}-hide-{hide}", "blocks": [], "chunk_radius": 0,
                         "flash": flash, "hide": hide, "x": float_bits(8.5, "f64"),
                         "y": float_bits(64.0, "f64"), "z": float_bits(8.5, "f64")})
    for y in (-60.0, 200.0, 320.0):
        rows.append({"label": f"flash-2-y-{y}", "blocks": [], "chunk_radius": 0, "flash": 2,
                     "hide": False, "x": float_bits(-300.25, "f64"), "y": float_bits(y, "f64"),
                     "z": float_bits(1000.5, "f64")})
    yield {"case_id": "movement/sky-flash", "family": "movement", "operation": "sky_flash",
           "rows": rows}
