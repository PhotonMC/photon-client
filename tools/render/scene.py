#!/usr/bin/env python3
"""Build the fixed scene the render comparison runs against.

A reference frame is only meaningful if the same frame comes back every time.
The title screen never stops moving, and an ordinary world keeps clouds, sky,
water and entities in motion, so this builds the opposite: a sealed, lit stone
room with no sky, no weather, no ticking and nothing alive in it. What varies
between two runs of that scene is what the environment itself contributes.

The world is generated once by the pinned dedicated server and kept as a
fixture under `.data/`; client runs copy it in fresh so a run can never mutate
the reference.
"""

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / ".data"
RUNTIME = DATA / "minecraft" / "runtime"
WORK = DATA / "render"
SERVER_JAR = RUNTIME / "server-26.2.jar"
SERVER_SHA1 = "823e2250d24b3ddac457a60c92a6a941943fcd6a"

SEED = "776"

# Two scenes on purpose. `room` has a single light source and lit-by-propagation
# walls, which is where the cross-run difference lives. `emissive` makes every
# visible surface a light source, so there is nothing left to propagate; what
# still differs there is independent of the lighting bake.
SCENES = {
    "room": {
        "shell": "minecraft:stone",
        "floor": "minecraft:glowstone",
    },
    "emissive": {
        "shell": "minecraft:glowstone",
        "floor": "minecraft:glowstone",
    },
}
DEFAULT_SCENE = "room"

# A sealed stone shell with a lit floor. No sky, no weather, no water, no
# entities: the only thing left that can differ between runs is the renderer
# and the environment it runs on.
BUILD_COMMANDS = [
    "gamerule doDaylightCycle false",
    "gamerule doWeatherCycle false",
    "gamerule doMobSpawning false",
    "gamerule doFireTick false",
    "gamerule mobGriefing false",
    "gamerule randomTickSpeed 0",
    "gamerule doEntityDrops false",
    "gamerule doTileDrops false",
    "gamerule announceAdvancements false",
    "gamerule doPatrolSpawning false",
    "gamerule doTraderSpawning false",
    "gamerule doInsomnia false",
    "gamerule doVinesSpread false",
    "gamerule doWardenSpawning false",
    "gamerule spawnChunkRadius 2",
    "difficulty peaceful",
    # The build area has to be resident before it can be written to; a fresh
    # server only keeps the spawn chunks loaded.
    "forceload add -6 -6 6 6",
    "time set noon",
    "weather clear",
    "defaultgamemode spectator",
    "fill -6 94 -6 6 106 6 {shell} hollow",
    "fill -5 95 -5 5 95 5 {floor}",
    "fill -5 96 -5 5 96 5 minecraft:air",
    "setblock -3 97 -3 minecraft:oak_planks",
    "setblock 3 97 3 minecraft:iron_block",
    "setblock -3 97 3 minecraft:bookshelf",
    "setblock 3 97 -3 minecraft:red_wool",
    "setworldspawn 0 96 0",
    "kill @e[type=!player]",
    "forceload remove all",
    "save-all flush",
]

SERVER_PROPERTIES = {
    "level-seed": SEED,
    "online-mode": "false",
    "spawn-protection": "0",
    "view-distance": "8",
    "simulation-distance": "8",
    "gamemode": "spectator",
    "force-gamemode": "true",
    "difficulty": "peaceful",
    "allow-nether": "false",
    "spawn-monsters": "false",
    "generate-structures": "false",
    "max-tick-time": "-1",
    "sync-chunk-writes": "true",
}


class Blocked(Exception):
    pass


class Failed(Exception):
    pass


def level_name(scene):
    return f"parity-{scene}"


def build_commands(scene):
    materials = SCENES[scene]
    return [line.format(**materials) for line in BUILD_COMMANDS]


def server_properties(scene, port=None):
    properties = dict(SERVER_PROPERTIES)
    properties["level-name"] = level_name(scene)
    if port is not None:
        properties["server-port"] = str(port)
    return "\n".join(f"{key}={value}" for key, value in sorted(properties.items())) + "\n"


def check_server():
    if not SERVER_JAR.is_file():
        raise Blocked(f"Missing pinned server jar: {SERVER_JAR}")
    actual = hashlib.sha1(SERVER_JAR.read_bytes()).hexdigest()
    if actual != SERVER_SHA1:
        raise Blocked(f"Server hash mismatch: {actual}")


# Where the camera sits for every reference frame. Inside the sealed room,
# two blocks above the lit floor, facing due south with no pitch.
# Feet exactly on the lit floor: a spectator left floating drifts down,
# which quietly makes every later capture a different camera.
PLAYER_POS = (0.5, 96.0, 0.5)
PLAYER_ROTATION = (0.0, 0.0)
SPECTATOR_GAME_TYPE = 3

TAG_END, TAG_BYTE, TAG_SHORT, TAG_INT, TAG_LONG = 0, 1, 2, 3, 4
TAG_FLOAT, TAG_DOUBLE, TAG_BYTE_ARRAY, TAG_STRING = 5, 6, 7, 8
TAG_LIST, TAG_COMPOUND, TAG_INT_ARRAY, TAG_LONG_ARRAY = 9, 10, 11, 12

PAYLOAD_SIZES = {
    TAG_BYTE: 1, TAG_SHORT: 2, TAG_INT: 4, TAG_LONG: 8,
    TAG_FLOAT: 4, TAG_DOUBLE: 8,
}


def scan_payload(data, offset, tag):
    """Return the offset just past one NBT payload, recording nothing."""
    if tag in PAYLOAD_SIZES:
        return offset + PAYLOAD_SIZES[tag]
    if tag == TAG_BYTE_ARRAY:
        count = int.from_bytes(data[offset:offset + 4], "big", signed=True)
        return offset + 4 + count
    if tag == TAG_STRING:
        length = int.from_bytes(data[offset:offset + 2], "big")
        return offset + 2 + length
    if tag == TAG_INT_ARRAY:
        count = int.from_bytes(data[offset:offset + 4], "big", signed=True)
        return offset + 4 + count * 4
    if tag == TAG_LONG_ARRAY:
        count = int.from_bytes(data[offset:offset + 4], "big", signed=True)
        return offset + 4 + count * 8
    if tag == TAG_LIST:
        element = data[offset]
        count = int.from_bytes(data[offset + 1:offset + 5], "big", signed=True)
        offset += 5
        for _ in range(max(count, 0)):
            offset = scan_payload(data, offset, element)
        return offset
    if tag == TAG_COMPOUND:
        return scan_compound(data, offset)[1]
    raise Failed(f"Unsupported NBT tag {tag}")


def scan_compound(data, offset, wanted=(), found=None):
    """Walk one compound, recording where the named top-level tags start."""
    found = {} if found is None else found
    while True:
        tag = data[offset]
        offset += 1
        if tag == TAG_END:
            return found, offset
        length = int.from_bytes(data[offset:offset + 2], "big")
        name = data[offset + 2:offset + 2 + length].decode("utf-8")
        offset += 2 + length
        if name in wanted:
            found[name] = (tag, offset)
        offset = scan_payload(data, offset, tag)


def patch_player(path):
    """Write the pinned camera into a saved player file.

    The client's spawn search walks up to the highest solid block, so a world
    spawn inside a sealed shell still puts the player on its roof. Saved player
    data wins over that search. Every value written here is a plain coordinate
    the game itself would store; the payload sizes are unchanged, so the file
    keeps its original layout.
    """
    raw = gzip.decompress(path.read_bytes())
    if raw[0] != TAG_COMPOUND:
        raise Failed(f"Unexpected player root tag in {path}")
    name_length = int.from_bytes(raw[1:3], "big")
    wanted = ("Pos", "Motion", "Rotation", "playerGameType", "FallDistance", "OnGround")
    found, _ = scan_compound(raw, 3 + name_length, wanted=wanted)
    for key in ("Pos", "Motion", "Rotation", "playerGameType"):
        if key not in found:
            raise Failed(f"{key} missing from {path}")

    data = bytearray(raw)
    tag, offset = found["Pos"]
    if tag != TAG_LIST or data[offset] != TAG_DOUBLE:
        raise Failed("Pos is not a list of doubles")
    for index, value in enumerate(PLAYER_POS):
        at = offset + 5 + index * 8
        data[at:at + 8] = struct.pack(">d", value)

    tag, offset = found["Motion"]
    if tag != TAG_LIST or data[offset] != TAG_DOUBLE:
        raise Failed("Motion is not a list of doubles")
    for index in range(3):
        at = offset + 5 + index * 8
        data[at:at + 8] = struct.pack(">d", 0.0)

    if "FallDistance" in found:
        tag, offset = found["FallDistance"]
        if tag == TAG_FLOAT:
            data[offset:offset + 4] = struct.pack(">f", 0.0)
    if "OnGround" in found:
        tag, offset = found["OnGround"]
        if tag == TAG_BYTE:
            data[offset] = 1

    tag, offset = found["Rotation"]
    if tag != TAG_LIST or data[offset] != TAG_FLOAT:
        raise Failed("Rotation is not a list of floats")
    for index, value in enumerate(PLAYER_ROTATION):
        at = offset + 5 + index * 4
        data[at:at + 4] = struct.pack(">f", value)

    tag, offset = found["playerGameType"]
    if tag != TAG_INT:
        raise Failed("playerGameType is not an int")
    data[offset:offset + 4] = struct.pack(">i", SPECTATOR_GAME_TYPE)

    path.write_bytes(gzip.compress(bytes(data)))


def seed_player(args, fixture):
    """Let the server create the player file by joining once.

    Only the file's existence matters here: the camera fields are rewritten
    afterwards, so nothing depends on what the connection manages to do before
    it drops.
    """
    if not args.probe.is_file():
        raise Blocked(f"Missing probe binary: {args.probe}; run cargo build --release")
    staging = WORK / f"scene-seed-{args.scene}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir(parents=True)
    (staging / "eula.txt").write_text("eula=true\n", encoding="utf-8")
    (staging / "server.properties").write_text(
        server_properties(args.scene, args.port), encoding="utf-8",
    )
    shutil.copytree(fixture, staging / level_name(args.scene))

    log_path = staging / "seed.log"
    with log_path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen(
            [args.java, f"-Xmx{args.heap}", "-jar", str(SERVER_JAR), "nogui"],
            cwd=staging, stdin=subprocess.PIPE, stdout=log,
            stderr=subprocess.STDOUT, text=True,
        )
        try:
            wait_for(log_path, "Done (", args.timeout, process)
            subprocess.run(
                [str(args.probe), f"127.0.0.1:{args.port}", args.user, "1", "30"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
                timeout=args.timeout,
            )
            wait_for(log_path, "left the game", args.timeout, process)
            process.stdin.write("save-all flush\nstop\n")
            process.stdin.flush()
            process.wait(timeout=args.timeout)
        except Exception:
            process.kill()
            raise

    produced = staging / level_name(args.scene)
    shutil.rmtree(fixture)
    shutil.copytree(produced, fixture)


def place(args):
    """Pin the player's exact position, rotation and game mode in the fixture.

    The server writes a player file on first join; this rewrites the camera
    fields in it so every later run starts from the identical viewpoint.
    """
    check_server()
    fixture = WORK / "scene" / level_name(args.scene)
    if not fixture.is_dir():
        raise Blocked(f"Missing fixture: {fixture}; run build first")
    players = sorted(fixture.rglob("players/data/*.dat"))
    if not players:
        seed_player(args, fixture)
        players = sorted(fixture.rglob("players/data/*.dat"))
    if not players:
        raise Failed(f"No player file produced in {fixture}")
    for player in players:
        patch_player(player)
    report = {
        "command": "place",
        "status": "player_pinned",
        "level_name": level_name(args.scene),
        "position": list(PLAYER_POS),
        "motion": [0.0, 0.0, 0.0],
        "rotation": list(PLAYER_ROTATION),
        "game_type": SPECTATOR_GAME_TYPE,
        "player_files": [str(path.relative_to(fixture)) for path in players],
    }
    print(json.dumps(report, sort_keys=True))
    return 0


def build(args):
    check_server()
    name = level_name(args.scene)
    fixture = WORK / "scene" / name
    if fixture.exists() and not args.force:
        raise Blocked(f"Fixture already exists: {fixture} (use --force to rebuild)")

    staging = WORK / f"scene-build-{args.scene}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir(parents=True)
    (staging / "eula.txt").write_text("eula=true\n", encoding="utf-8")
    (staging / "server.properties").write_text(
        server_properties(args.scene), encoding="utf-8",
    )

    command = [args.java, f"-Xmx{args.heap}", "-jar", str(SERVER_JAR), "nogui"]
    log_path = staging / "build.log"
    with log_path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen(
            command, cwd=staging, stdin=subprocess.PIPE,
            stdout=log, stderr=subprocess.STDOUT, text=True,
        )
        try:
            wait_for(log_path, 'Done (', args.timeout, process)
            time.sleep(args.settle)
            for line in build_commands(args.scene):
                process.stdin.write(line + "\n")
                process.stdin.flush()
                time.sleep(args.step_delay)
            process.stdin.write("stop\n")
            process.stdin.flush()
            process.wait(timeout=args.timeout)
        except Exception:
            process.kill()
            raise

    if process.returncode != 0:
        raise Failed(f"Server exited {process.returncode}; see {log_path}")

    generated = staging / name
    if not generated.is_dir():
        raise Failed(f"Server produced no world at {generated}")
    fixture.parent.mkdir(parents=True, exist_ok=True)
    if fixture.exists():
        shutil.rmtree(fixture)
    shutil.copytree(generated, fixture)

    report = {
        "command": "build",
        "status": "scene_ready",
        "level_name": name,
        "scene": args.scene,
        "seed": SEED,
        "fixture": str(fixture.relative_to(ROOT)),
        "commands": len(build_commands(args.scene)),
        "region_files": len(list(fixture.rglob("region/*.mca"))),
        "fixture_bytes": sum(f.stat().st_size for f in fixture.rglob("*") if f.is_file()),
    }
    print(json.dumps(report, sort_keys=True))
    return 0


def wait_for(log_path, needle, timeout, process, poll=0.5):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise Failed(f"Server exited early with code {process.returncode}; see {log_path}")
        if log_path.is_file() and needle in log_path.read_text(encoding="utf-8", errors="replace"):
            return
        time.sleep(poll)
    raise Failed(f"Server never printed {needle!r} within {timeout}s")


def serve(args):
    """Run the pinned server on the built scene and leave it up.

    Nothing about the world is generated here; the fixture is copied in so the
    server serves exactly the scene `build` produced. Offline mode is already
    on in the pinned properties, so a client joins without an account."""
    check_server()
    name = level_name(args.scene)
    fixture = WORK / "scene" / name
    if not fixture.is_dir():
        raise Blocked(f"No fixture for {args.scene}; run `build` first")

    run = WORK / f"serve-{args.scene}"
    if run.exists():
        shutil.rmtree(run)
    run.mkdir(parents=True)
    (run / "eula.txt").write_text("eula=true\n", encoding="utf-8")
    (run / "server.properties").write_text(
        server_properties(args.scene, args.port), encoding="utf-8",
    )
    shutil.copytree(fixture, run / name)

    log_path = run / "serve.log"
    with log_path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen(
            [args.java, f"-Xmx{args.heap}", "-jar", str(SERVER_JAR), "nogui"],
            cwd=run, stdin=subprocess.PIPE,
            stdout=log, stderr=subprocess.STDOUT, text=True,
        )
        try:
            wait_for(log_path, 'Done (', args.timeout, process)
            print(json.dumps({
                "command": "serve",
                "status": "listening",
                "scene": args.scene,
                "level_name": name,
                "port": args.port,
                "pid": process.pid,
                "log": str(log_path.relative_to(ROOT)),
            }, sort_keys=True), flush=True)
            if args.seconds:
                time.sleep(args.seconds)
            else:
                process.wait()
        finally:
            if process.poll() is None:
                try:
                    process.stdin.write("stop\n")
                    process.stdin.flush()
                    process.wait(timeout=60)
                except Exception:
                    process.kill()
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    make = sub.add_parser("build", help="generate the fixed scene fixture")
    make.add_argument("--scene", choices=sorted(SCENES), default=DEFAULT_SCENE)
    make.add_argument("--force", action="store_true")
    make.add_argument("--java", default="java")
    make.add_argument("--heap", default="2G")
    make.add_argument("--timeout", type=float, default=300.0)
    make.add_argument("--settle", type=float, default=3.0)
    make.add_argument("--step-delay", type=float, default=0.15)
    make.set_defaults(handler=build)

    put = sub.add_parser("place", help="pin the player camera inside the built scene")
    put.add_argument("--scene", choices=sorted(SCENES), default=DEFAULT_SCENE)
    put.add_argument("--user", default="ParityTest")
    put.add_argument("--port", type=int, default=25599)
    put.add_argument("--probe", type=Path, default=ROOT / "target/release/voxel")
    put.add_argument("--java", default="java")
    put.add_argument("--heap", default="2G")
    put.add_argument("--timeout", type=float, default=180.0)
    put.set_defaults(handler=place)

    up = sub.add_parser("serve", help="run the pinned server on the built scene")
    up.add_argument("--scene", choices=sorted(SCENES), default=DEFAULT_SCENE)
    up.add_argument("--port", type=int, default=25599)
    up.add_argument("--java", default="java")
    up.add_argument("--heap", default="2G")
    up.add_argument("--timeout", type=float, default=300.0)
    up.add_argument("--seconds", type=float, default=0.0,
                    help="stop after this long; zero runs until killed")
    up.set_defaults(handler=serve)

    args = parser.parse_args()
    try:
        return args.handler(args)
    except Blocked as error:
        print(json.dumps({"status": "blocked", "reason": str(error)}), file=sys.stderr)
        return 2
    except Failed as error:
        print(json.dumps({"status": "failed", "reason": str(error)}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
