"""The elytra state cases."""

from cases.helpers import *


def elytra_state_cases():
    """The real `ElytraAnimationState` of a real remote player ticked through scenarios: standing,
    sneaking, gliding level, diving steeply and shallowly, rising, a glide that ends, sneaking while
    gliding, and no motion."""
    def tick(glide=False, sneak=False, motion=(0.0, 0.0, 0.0)):
        return {"fall_flying": glide, "crouching": sneak, "x": float_bits(motion[0], "f64"),
                "y": float_bits(motion[1], "f64"), "z": float_bits(motion[2], "f64")}

    def scenario(label, ticks):
        return {"label": label, "ticks": ticks}

    level = (0.0, 0.0, 1.2)
    scenarios = [
        scenario("standing", [tick() for _ in range(12)]),
        scenario("sneaking", [tick(sneak=True) for _ in range(12)]),
        scenario("sneak-then-stand", [tick(sneak=True)] * 8 + [tick()] * 8),
        scenario("glide-level", [tick(True, motion=level) for _ in range(14)]),
        scenario("glide-dive-steep", [tick(True, motion=(0.0, -1.5, 0.3)) for _ in range(14)]),
        scenario("glide-dive-shallow", [tick(True, motion=(0.0, -0.2, 1.5)) for _ in range(14)]),
        scenario("glide-rising", [tick(True, motion=(0.0, 0.8, 1.0)) for _ in range(14)]),
        scenario("glide-no-motion", [tick(True) for _ in range(8)]),
        scenario("glide-tiny-motion", [tick(True, motion=(0.0, -1.0e-6, 0.0)) for _ in range(8)]),
        scenario("glide-ends", [tick(True, motion=(0.0, -0.7, 1.0))] * 6 + [tick()] * 10),
        scenario("glide-sneaking", [tick(True, sneak=True, motion=(0.5, -0.5, 0.5)) for _ in range(10)]),
        scenario("glide-varying", [tick(True, motion=(0.1 * i, -0.1 * i, 1.0 - 0.05 * i)) for i in range(16)]),
        scenario("glide-turning-down", [tick(True, motion=(0.0, -0.05 * i, 1.0)) for i in range(16)]),
    ]
    yield {"case_id": "movement/elytra-state", "family": "movement", "operation": "elytra_state", "rows": scenarios}
