"""The `sound` family: original synthetic inputs."""

import struct
from cases.helpers import *


def sound_cases():
    yield from sound_registry_cases()
    yield from sound_packet_cases()
    yield from sound_decode_cases()
    yield from sound_engine_cases()
    yield {"case_id": "sound/block-sounds", "family": "sound", "operation": "block_sounds", "rows": [{"label": "all"}]}
    yield {"case_id": "sound/damage-effects", "family": "sound", "operation": "damage_effects", "rows": [{"label": "all"}]}


def sound_engine_cases():
    """The reference's own sound engine, with the sound device replaced by channels that write down their calls:
    sounds played with every kind of parameter, refused for zero volume, no channel or no sound, stopped by
    instance, event and source, ticked through their life, delayed, looping by hand, scripted to move, paused and
    resumed, and followed through the player's volume sliders."""
    registry = ('{"a": {"sounds": [{"name": "x", "volume": 0.5, "pitch": 2.0, "attenuation_distance": 32}]}, '
                '"s": {"sounds": [{"name": "mus", "stream": true, "volume": 0.8}]}, '
                '"wrap": {"sounds": [{"name": "a", "type": "event", "volume": 0.5, "pitch": 0.5}]}, '
                '"multi": {"sounds": ["x", "y", {"name": "z", "weight": 2}]}, '
                '"gone": {"sounds": ["nothing"]}, "intentionally_empty": {"sounds": ["x"]}}')
    files = ["sounds/x.ogg", "sounds/y.ogg", "sounds/z.ogg", "sounds/mus.ogg"]
    counter = [0]

    def play(event="minecraft:a", op="play", id=None, source="BLOCKS", volume=1.0, pitch=1.0, seed=1, looping=False,
             delay=0, attenuation="linear", relative=False, pos=(10.5, 64.0, -3.25), start_silent=False, can_play=True,
             tickable=None, delay_ticks=0):
        counter[0] += 1
        step = {"op": op, "id": counter[0] if id is None else id, "event": event, "source": source,
                "volume": float_bits(volume), "pitch": float_bits(pitch), "seed": str(seed), "looping": looping,
                "delay": delay, "attenuation": attenuation, "relative": relative, "pos": list(pos),
                "start_silent": start_silent, "can_play": can_play, "tickable": tickable}
        if op == "play_delayed":
            step["delay_ticks"] = delay_ticks
        return step

    def scenario(label, steps, volumes=None, static=8, streaming=2):
        counter[0] = 0
        return {"label": label, "json": registry, "files": files, "volumes": volumes or {},
                "static_channels": static, "streaming_channels": streaming, "steps": steps}
    tick = {"op": "tick", "paused": False}
    rows = []
    counter[0] = 0
    rows.append(scenario("plain-sounds", [
        play(), play("minecraft:multi", seed=7, volume=0.75, pitch=1.5), play("minecraft:wrap", seed=3),
        play("minecraft:s", source="MUSIC", seed=9), play("minecraft:s", source="MUSIC", looping=True, seed=9),
        play("minecraft:a", looping=True, seed=2), play("minecraft:a", attenuation="none"),
        play("minecraft:a", relative=True), play("minecraft:a", volume=3.5, pitch=9.0, pos=(-100.5, 5.0, 1e6)),
        play("minecraft:a", volume=0.1, pitch=0.01),
    ]))
    rows.append(scenario("refusals", [
        play("minecraft:unknown"), play("minecraft:gone"), play("minecraft:intentionally_empty"), play(can_play=False),
        play(volume=0.0), play(volume=0.0, start_silent=True), play("minecraft:s", source="MUSIC", volume=0.0),
        play(source="MASTER", volume=0.5), play(volume=-1.0),
    ]))
    rows.append(scenario("volume-sliders", [
        play(source="MUSIC", volume=0.9), play(source="HOSTILE"), play(source="MASTER"),
        {"op": "slider", "source": "MUSIC", "value": 0.4}, {"op": "slider", "source": "MASTER", "value": 0.5},
        {"op": "category_volume", "source": "HOSTILE", "value": float_bits(0.25)}, {"op": "category_volume", "source": "MASTER", "value": float_bits(2.0)},
        play(source="HOSTILE"), play(source="MUSIC"), {"op": "slider", "source": "BLOCKS", "value": 0.0},
    ], volumes={"MASTER": 0.8, "MUSIC": 0.6, "HOSTILE": 0.7, "BLOCKS": 1.0}))
    rows.append(scenario("pools-run-out", [
        play(), play(), play(), play(), play("minecraft:s", source="MUSIC"), play("minecraft:s", source="MUSIC"),
        play("minecraft:s", source="MUSIC"),
    ], static=3, streaming=2))
    rows.append(scenario("stop-by-instance-event-and-source", [
        play(id=1), play("minecraft:multi", id=2, source="HOSTILE"), play("minecraft:wrap", id=3, source="HOSTILE"),
        play("minecraft:s", id=4, source="MUSIC"), {"op": "stop", "id": 1}, {"op": "stop", "id": 1},
        {"op": "stop_named", "event": "minecraft:multi", "source": "HOSTILE"},
        {"op": "stop_named", "event": "minecraft:a", "source": "HOSTILE"}, {"op": "stop_named", "event": "minecraft:wrap", "source": None},
        {"op": "stop_named", "event": None, "source": "MUSIC"}, play(id=5), {"op": "stop_named", "event": None, "source": None},
        {"op": "active", "id": 5}, play(id=6),
    ]))
    rows.append(scenario("lives-end-after-twenty-ticks", [
        play(id=1), {"op": "active", "id": 1}, tick, {"op": "finish", "id": 1}, tick, tick, {"op": "active", "id": 1},
        *[tick] * 20, {"op": "active", "id": 1}, play(id=2), {"op": "stop", "id": 2}, tick, tick,
    ], static=1))
    rows.append(scenario("delay-and-looping-by-hand", [
        play(id=1, op="play_delayed", delay_ticks=3), tick, tick, tick, tick,
        play(id=2, looping=True, delay=5), {"op": "finish", "id": 2}, *[tick] * 24, {"op": "stop", "id": 2},
        {"op": "finish", "id": 2}, *[tick] * 24,
    ]))
    rows.append(scenario("ticking-sounds", [
        play(id=1, tickable={"stop_at": 4, "moves": [[1.0, 2.0, 3.0], [1.5, 2.5, 3.5]], "volumes": [0.9, 0.8], "pitches": [1.1, 1.2]}),
        tick, tick, tick, tick, tick,
        play(id=2, op="queue_ticking", tickable={"stop_at": 2, "moves": [[0.0, 0.0, 0.0]], "volumes": [0.5], "pitches": [0.5]}),
        tick, tick, tick, {"op": "active", "id": 2},
        play(id=3, op="play_delayed", delay_ticks=2, tickable={"stop_at": 9, "moves": [], "volumes": [], "pitches": []}), tick, tick, tick,
    ]))
    rows.append(scenario("pause-and-resume", [
        play(id=1), play("minecraft:s", id=2, source="MUSIC"), play(id=3, source="HOSTILE"),
        {"op": "pause_all_except", "sources": ["MUSIC"]}, {"op": "resume"}, {"op": "pause_all_except", "sources": []},
        {"op": "pause_all_except", "sources": ["MUSIC", "BLOCKS"]}, {"op": "resume"},
        {"op": "finish", "id": 2}, {"op": "tick", "paused": True}, {"op": "finish", "id": 1}, {"op": "tick", "paused": True},
        {"op": "active", "id": 2}, {"op": "active", "id": 1},
    ]))
    yield {"case_id": "sound/engine", "family": "sound", "operation": "engine", "rows": rows}


def sound_decode_cases():
    """Real sound files decoded by the reference's own Ogg/Vorbis stream and 16-bit conversion: short and long, the
    kinds of sound the game plays (a click, steps, a pig, a cave ambience, a voice, a track of music)."""
    objects = [
        ("click", "random/click"), ("dig-stone", "dig/stone1"), ("step-stone", "step/stone1"),
        ("pig", "mob/pig/say1"), ("cave", "ambient/cave/cave1"), ("villager", "mob/villager/idle1"),
        ("tiny-chicken-step", "mob/chicken/baby_chicken/step"), ("suspicious-gravel", "block/suspicious_gravel/break1"),
        ("music", "music/game/creative/dreiton"),
    ]
    yield {"case_id": "sound/decode", "family": "sound", "operation": "decode",
           "rows": [{"label": label, "object": f"minecraft/sounds/{name}.ogg"} for label, name in objects]}


def sound_registry_cases():
    """The reference's own sound registry: small `sounds.json` texts (weights, borrowed events, missing
    files, the things a file can get wrong), picked from with seeded random sources, and the game's real
    `sounds.json` with its real file list."""
    files = ["a", "b", "c", "dir/d"]

    def small(label, text, picks=(), events=None, listed=files):
        row = {"label": label, "json": text, "files": [f"sounds/{name}.ogg" for name in listed]}
        row["picks"] = [{"event": event, "seed": str(seed), "count": count} for event, seed, count in picks]
        if events is not None:
            row["events"] = events
        return row
    rows = [
        small("plain-names", '{"one": {"sounds": ["a", "b", "c"]}}', [("minecraft:one", 7, 12)]),
        small("weights", '{"w": {"sounds": [{"name": "a", "weight": 5}, {"name": "b"}, {"name": "c", "weight": 2}]}}',
              [("minecraft:w", 1, 30), ("minecraft:w", -42, 30), ("minecraft:w", 123456789, 30)]),
        small("fields", '{"f": {"subtitle": "subtitles.f", "sounds": [{"name": "a", "volume": 0.75, "pitch": 1.5, '
              '"stream": true, "preload": true, "attenuation_distance": 48}, {"name": "dir/d", "volume": 0.3}]}}',
              [("minecraft:f", 3, 10)]),
        small("event-entries", '{"base": {"sounds": [{"name": "a", "volume": 0.5, "weight": 3}, {"name": "b", "pitch": 2.0}]}, '
              '"wrap": {"sounds": [{"name": "base", "type": "event", "volume": 0.5, "pitch": 0.5, "weight": 9}, "c"]}, '
              '"wrap2": {"sounds": [{"name": "wrap", "type": "event", "stream": true}]}}',
              [("minecraft:wrap", 5, 24), ("minecraft:wrap2", 11, 24)]),
        small("missing-event-entry", '{"w": {"sounds": [{"name": "nowhere", "type": "event"}, "a"]}}',
              [("minecraft:w", 2, 12)]),
        small("missing-files", '{"m": {"sounds": ["gone", "a", {"name": "also/gone"}]}}', [("minecraft:m", 4, 8)]),
        small("empty-event", '{"e": {"sounds": []}, "n": {}}', [("minecraft:e", 1, 3), ("minecraft:n", 1, 3)]),
        small("replace", '{"r": {"sounds": ["a"], "replace": true}, "s": {"subtitle": "x"}}', [("minecraft:r", 1, 4)]),
        small("namespaced-names", '{"n": {"sounds": [{"name": "minecraft:a"}, "other:thing"]}}', [("minecraft:n", 9, 6)],
              listed=files + ["thing"]),
        small("zero-volume", '{"good": {"sounds": ["a"]}, "bad": {"sounds": [{"name": "a", "volume": 0}]}}'),
        small("negative-pitch", '{"bad": {"sounds": [{"name": "a", "pitch": -1.0}]}}'),
        small("zero-weight", '{"bad": {"sounds": [{"name": "a", "weight": 0}]}}'),
        small("bad-type", '{"bad": {"sounds": [{"name": "a", "type": "loop"}]}}'),
        small("no-name", '{"bad": {"sounds": [{"volume": 1.0}]}}'),
        small("not-an-object", '{"bad": ["a"]}'),
        small("sounds-not-array", '{"bad": {"sounds": "a"}}'),
        small("subtitle-not-string", '{"bad": {"subtitle": 3, "sounds": ["a"]}}'),
        small("bad-bool", '{"bad": {"replace": 1, "sounds": ["a"]}}'),
        small("float-weight", '{"w": {"sounds": [{"name": "a", "weight": 2.9}, "b"]}}', [("minecraft:w", 6, 12)]),
        small("big-volume", '{"v": {"sounds": [{"name": "a", "volume": 3.5}]}}', [("minecraft:v", 1, 2)]),
    ]
    real = {"label": "real", "real": True, "picks": []}
    for event, seeds in (("minecraft:block.stone.break", (1, 2, 3)), ("minecraft:entity.pig.ambient", (4, 5)),
                         ("minecraft:ambient.cave", (6, 7)), ("minecraft:entity.cow.step", (8,)),
                         ("minecraft:block.note_block.harp", (9,)), ("minecraft:ui.button.click", (10,))):
        real["picks"] += [{"event": event, "seed": str(seed), "count": 40} for seed in seeds]
    yield {"case_id": "sound/registry", "family": "sound", "operation": "registry", "rows": rows + [real]}


def sound_packet_cases():
    """The sound event registry in id order, and the three sound packets decoded by the reference's own codecs."""
    yield {"case_id": "sound/events", "family": "sound", "operation": "sound_events", "rows": [{"label": "all"}]}

    def varint(value):
        out = bytearray()
        value &= 0xFFFFFFFF
        while True:
            part = value & 0x7F
            value >>= 7
            out.append(part | (0x80 if value else 0))
            if not value:
                return bytes(out)

    def identifier(text):
        data = text.encode()
        return varint(len(data)) + data

    def holder(registered=None, direct=None, range_=None):
        if registered is not None:
            return varint(registered + 1)
        out = varint(0) + identifier(direct)
        return out + (bytes([1]) + struct.pack(">f", range_) if range_ is not None else bytes([0]))

    def sound(label, sound_holder, source, at, volume, pitch, seed, extra=b"", cut=None):
        data = (sound_holder + varint(source) + b"".join(struct.pack(">i", int(v)) for v in at)
                + struct.pack(">f", volume) + struct.pack(">f", pitch) + struct.pack(">q", seed) + extra)
        return {"label": f"sound-{label}", "kind": "sound", "hex": (data[:cut] if cut is not None else data).hex()}

    def entity(label, sound_holder, source, entity_id, volume, pitch, seed):
        data = (sound_holder + varint(source) + varint(entity_id) + struct.pack(">f", volume)
                + struct.pack(">f", pitch) + struct.pack(">q", seed))
        return {"label": f"entity-{label}", "kind": "sound_entity", "hex": data.hex()}

    def stop(label, flags, source=None, name=None, extra=b""):
        data = bytes([flags]) + (varint(source) if source is not None else b"") + (identifier(name) if name else b"") + extra
        return {"label": f"stop-{label}", "kind": "stop_sound", "hex": data.hex()}
    rows = [
        sound("registered-first", holder(registered=0), 4, (80, 520, -24), 1.0, 1.0, 0),
        sound("registered-later", holder(registered=1234), 0, (1, 2, 3), 0.5, 1.25, 123456789012345),
        sound("every-source", holder(registered=7), 10, (0, 0, 0), 2.0, 0.5, -1),
        sound("direct", holder(direct="my:sound.custom"), 2, (-8, 8, 80000), 1.5, 2.0, 9),
        sound("direct-range", holder(direct="minecraft:block.stone.break", range_=24.5), 1, (3, -5, 7), 1.0, 1.0, 5),
        sound("big-coordinates", holder(registered=3), 4, (2147483647, -2147483648, 16777217), 1.0, 1.0, 1),
        sound("source-too-big", holder(registered=3), 11, (0, 0, 0), 1.0, 1.0, 1),
        sound("registered-too-big", holder(registered=99999), 4, (0, 0, 0), 1.0, 1.0, 1),
        sound("truncated", holder(registered=3), 4, (0, 0, 0), 1.0, 1.0, 1, cut=20),
        sound("trailing", holder(registered=3), 4, (0, 0, 0), 1.0, 1.0, 1, extra=b"\x00\x01"),
        sound("nan-volume", holder(registered=3), 4, (0, 0, 0), float("nan"), -0.0, 1),
        entity("registered", holder(registered=15), 5, 77, 0.8, 1.1, 42),
        entity("direct", holder(direct="a:b", range_=0.0), 8, 100000, 1.0, 1.0, -77),
        entity("bad-source", holder(registered=15), 12, 1, 1.0, 1.0, 1),
        stop("everything", 0), stop("source", 1, source=4), stop("name", 2, name="minecraft:ui.button.click"),
        stop("both", 3, source=9, name="minecraft:block.anvil.land"), stop("bad-source", 1, source=11),
        stop("trailing", 0, extra=b"\x00"), stop("ignored-bits", 4),
    ]
    yield {"case_id": "sound/packets", "family": "sound", "operation": "packets", "rows": rows}
