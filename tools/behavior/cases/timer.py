"""The `timer` family: original synthetic inputs."""

from cases.helpers import *


def timer_cases():
    def trace(label, steps, start=0, tps=20, target=50):
        return {"case_id": f"timer/{label}", "family": "timer", "operation": "trace",
                "start_ms": str(start), "tps": float_bits(tps), "target_mspt": float_bits(target),
                "steps": steps}

    def time(action, ms):
        return {"action": action, "ms": str(ms)}

    for delta in (0, 1, 49, 50, 51, 1000):
        yield trace(f"boundary-{delta}", [time("game", delta), time("real", delta)])
    yield trace("residual", [time("game", ms) for ms in (17, 34, 49, 50, 51, 99, 100, 100, 101)])
    for tps, target in [(10, 100), (20, 25), (30, 1000 / 30), (60, 1000 / 60)]:
        yield trace(f"rate-{tps}-{target}", [step for ms in (1, 17, 33, 50, 100, 1000)
                                            for step in (time("game", ms), time("real", ms))],
                    tps=tps, target=target)
    yield trace("rate-change", [time("game", 25), {"action": "target", "value": float_bits(100)},
                               time("game", 100), {"action": "target", "value": float_bits(25)},
                               time("game", 125), time("real", 125)])
    yield trace("pause-residual", [time("game", 25), {"action": "pause", "value": True},
                                  time("game", 50), {"action": "pause", "value": True},
                                  time("real", 75), time("game", 75),
                                  {"action": "pause", "value": False}, time("game", 100),
                                  {"action": "pause", "value": False}])
    for pause in (False, True):
        for freeze in (False, True):
            yield trace(f"pause-{pause}-freeze-{freeze}", [time("game", 25),
                        {"action": "pause", "value": pause}, {"action": "freeze", "value": freeze},
                        time("game", 75), time("real", 100), {"action": "freeze", "value": False},
                        {"action": "pause", "value": False}, time("game", 100)])
    for delta in (349, 350, 351, 5000):
        yield trace(f"real-limit-{delta}", [time("real", delta), time("real", delta + 1)])
    start = 2**53 + 1
    yield trace("large-clock", [time("game", start + 49), time("real", start + 50),
                                time("game", start + 50), time("game", start + 51)], start=start)
