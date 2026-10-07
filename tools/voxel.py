#!/usr/bin/env python3
"""Local helpers for working on voxel-client, on Linux, macOS and Windows.

    python3 tools/voxel.py setup                 # fill .data/ from Mojang's servers
    python3 tools/voxel.py server --accept-eula  # set up and start the test server
    python3 tools/voxel.py play                  # build and start the client
    python3 tools/voxel.py rcon "time set noon"  # send commands to the test server
    python3 tools/voxel.py check [--capture]     # the commit gate: prints CHECK OK or CHECK FAILED
    python3 tools/voxel.py parity --family gui   # Java/Rust comparison only, one summary line
    python3 tools/voxel.py shot NAME "SCRIPT"    # scripted client run, screenshot to .data/runs/shots
    python3 tools/voxel.py bench [--repeat N]    # fixed scripted run, frame timing report to .data/runs/bench

Only the Python standard library is used. Everything downloaded goes under
.data/, which stays on this machine and is never committed. Downloads come
from Mojang's official distribution (piston-meta, piston-data, the library
and resource servers) and are checked against the hashes the version file
gives. Run a subcommand with --help for its options.
"""
import argparse
import concurrent.futures
import hashlib
import json
import os
import re
import shutil
import socket
import struct
import subprocess
import sys
import time
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / ".data"
MINECRAFT = DATA / "minecraft"
RUNTIME = MINECRAFT / "runtime"
ASSETS = MINECRAFT / "assets"
SERVER_DIR = DATA / "render" / "walk"
PIN = json.loads((ROOT / "tests/behavior/reference.json").read_text(encoding="utf-8"))
VERSION = PIN["version"]
JAVA_MAJOR = PIN["java_major"]

VERSION_MANIFEST = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
RESOURCES = "https://resources.download.minecraft.net"
VINEFLOWER_VERSION = "1.12.0"
VINEFLOWER_URL = (
    "https://repo1.maven.org/maven2/org/vineflower/vineflower/"
    f"{VINEFLOWER_VERSION}/vineflower-{VINEFLOWER_VERSION}.jar"
)

# The test server's settings: offline, peaceful, seeded, and reachable by
# RCON so scripts can set the scene.
SERVER_PORT = 25600
RCON_PORT = 25601
RCON_PASSWORD = "walk"
SERVER_PROPERTIES = {
    "server-port": str(SERVER_PORT),
    "online-mode": "false",
    "level-name": "walk",
    "level-seed": "776",
    "gamemode": "survival",
    "force-gamemode": "true",
    "difficulty": "peaceful",
    "spawn-monsters": "false",
    "spawn-protection": "0",
    "view-distance": "12",
    "simulation-distance": "6",
    "pause-when-empty-seconds": "60",
    "enable-rcon": "true",
    "rcon.port": str(RCON_PORT),
    "rcon.password": RCON_PASSWORD,
    "broadcast-rcon-to-ops": "true",
    "motd": "voxel-client test server",
}


def say(message):
    print(message, flush=True)


def sha1_of(path):
    digest = hashlib.sha1()
    with open(path, "rb") as file:
        for block in iter(lambda: file.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def fetch(url, path, sha1=None, attempts=3):
    """Download `url` to `path` unless a file with the right hash is there."""
    path = Path(path)
    if path.is_file() and (sha1 is None or sha1_of(path) == sha1):
        return False
    path.parent.mkdir(parents=True, exist_ok=True)
    partial = path.with_name(path.name + ".part")
    for attempt in range(attempts):
        try:
            with urllib.request.urlopen(url, timeout=60) as response, open(partial, "wb") as out:
                shutil.copyfileobj(response, out)
            if sha1 is not None and sha1_of(partial) != sha1:
                raise OSError(f"hash mismatch for {url}")
            os.replace(partial, path)
            return True
        except OSError:
            if attempt == attempts - 1:
                raise
            time.sleep(1 + attempt)
    return False


def fetch_json(url):
    with urllib.request.urlopen(url, timeout=60) as response:
        return json.load(response)


def find_java(explicit=None):
    """A `java` of the pinned major version: --java, then JAVA_HOME, then PATH."""
    candidates = []
    if explicit:
        candidates.append(Path(explicit))
    if os.environ.get("JAVA_HOME"):
        name = "java.exe" if os.name == "nt" else "java"
        candidates.append(Path(os.environ["JAVA_HOME"]) / "bin" / name)
    on_path = shutil.which("java")
    if on_path:
        candidates.append(Path(on_path))
    for candidate in candidates:
        if not candidate.is_file():
            continue
        result = subprocess.run([str(candidate), "-version"], capture_output=True, text=True)
        text = result.stdout + result.stderr
        for token in text.replace('"', " ").split():
            head = token.split(".")[0]
            if head.isdigit() and int(head) >= 8:
                if int(head) == JAVA_MAJOR:
                    return str(candidate)
                break
    raise SystemExit(
        f"Java {JAVA_MAJOR} not found: install it, or pass --java PATH or set JAVA_HOME"
    )


# ---------------------------------------------------------------- setup


def setup(args):
    RUNTIME.mkdir(parents=True, exist_ok=True)
    say(f"Minecraft {VERSION}: version file")
    manifest = fetch_json(VERSION_MANIFEST)
    entry = next((v for v in manifest["versions"] if v["id"] == VERSION), None)
    if entry is None:
        raise SystemExit(f"version {VERSION} is not in Mojang's manifest")
    version_path = RUNTIME / f"{VERSION}.json"
    fetch(entry["url"], version_path, entry.get("sha1"))
    version = json.loads(version_path.read_text(encoding="utf-8"))

    say("client and server jars")
    client = RUNTIME / f"client-{VERSION}.jar"
    fetch(version["downloads"]["client"]["url"], client, version["downloads"]["client"]["sha1"])
    if client.name != PIN["client_file"] or sha1_of(client) != PIN["client_sha1"]:
        raise SystemExit("the client jar does not match tests/behavior/reference.json")
    server = RUNTIME / f"server-{VERSION}.jar"
    fetch(version["downloads"]["server"]["url"], server, version["downloads"]["server"]["sha1"])

    # Every library the version lists, natives for every platform included,
    # as the behaviour runner's pin expects them by file name.
    say("libraries")
    libraries = []
    for library in version["libraries"]:
        artifact = library.get("downloads", {}).get("artifact")
        if artifact:
            libraries.append(artifact)
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        list(pool.map(
            lambda a: fetch(a["url"], RUNTIME / "libs" / Path(a["path"]).name, a["sha1"]),
            libraries,
        ))
    (RUNTIME / "lib-urls.txt").write_text(
        "".join(f"{a['url']}\n" for a in sorted(libraries, key=lambda a: a["url"])),
        encoding="utf-8",
    )

    # What the client reads from the jar: textures, models, shaders, data.
    say("jar resources -> .data/minecraft/assets/jar")
    target = ASSETS / "jar"
    with zipfile.ZipFile(client) as jar:
        for member in jar.infolist():
            if member.is_dir() or member.filename.endswith(".class"):
                continue
            destination = target / member.filename
            if destination.is_file() and destination.stat().st_size == member.file_size:
                continue
            destination.parent.mkdir(parents=True, exist_ok=True)
            with jar.open(member) as source, open(destination, "wb") as out:
                shutil.copyfileobj(source, out)

    if not args.skip_objects:
        index_info = version["assetIndex"]
        index_path = ASSETS / "indexes" / f"{index_info['id']}.json"
        fetch(index_info["url"], index_path, index_info["sha1"])
        objects = json.loads(index_path.read_text(encoding="utf-8"))["objects"]
        hashes = sorted({meta["hash"] for meta in objects.values()})
        say(f"asset objects: {len(hashes)} (sounds and languages)")

        def fetch_object(digest):
            return fetch(f"{RESOURCES}/{digest[:2]}/{digest}",
                         ASSETS / "objects" / digest[:2] / digest, digest)

        with concurrent.futures.ThreadPoolExecutor(16) as pool:
            fetched = sum(pool.map(fetch_object, hashes))
        say(f"  downloaded {fetched}, the rest were already there")
        if args.named:
            build_named(objects)

    if args.decompile:
        decompile(args)
    say("done")


def build_named(objects):
    """assets/named/<logical path>: links (copies where links are refused)
    into the hash-named object store, for browsing."""
    say("named asset tree")
    named = ASSETS / "named"
    for logical, meta in objects.items():
        digest = meta["hash"]
        source = ASSETS / "objects" / digest[:2] / digest
        destination = named / logical
        if destination.exists():
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        try:
            os.symlink(source, destination)
        except OSError:
            shutil.copyfile(source, destination)


def decompile(args):
    """The client jar to .data/minecraft/source with Vineflower, libraries
    resolved, as the reference sources were made."""
    java = find_java(args.java)
    tool = RUNTIME / "tools" / f"vineflower-{VINEFLOWER_VERSION}.jar"
    fetch(VINEFLOWER_URL, tool)
    source = MINECRAFT / "source"
    if source.exists():
        shutil.rmtree(source)
    source.mkdir(parents=True)
    libraries = sorted((RUNTIME / "libs").glob("*.jar"))
    say(f"decompiling with Vineflower {VINEFLOWER_VERSION} (this takes minutes)")
    command = [
        java, "-Xmx10G", "-jar", str(tool),
        f"--thread-count={os.cpu_count() or 4}",
        "--indent-string=    ",
        "--include-runtime=current",
        "--decompile-generics=true",
        "--skip-extra-files=true",
        "--dump-bytecode-on-error=true",
        *[f"-e={library}" for library in libraries],
        str(RUNTIME / f"client-{VERSION}.jar"),
        str(source),
    ]
    with open(MINECRAFT / "decompile.log", "w", encoding="utf-8") as log:
        subprocess.run(command, check=True, stdout=log, stderr=subprocess.STDOUT)
    joml = next((library for library in libraries if library.name.startswith("joml-")), None)
    if joml is not None:
        target = MINECRAFT / "joml-source"
        shutil.rmtree(target, ignore_errors=True)
        target.mkdir(parents=True)
        subprocess.run(
            [java, "-jar", str(tool), "--indent-string=    ", "--skip-extra-files=true",
             str(joml), str(target)],
            check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )


# ---------------------------------------------------------------- server


def server(args):
    directory = Path(args.dir).resolve()
    if args.stop:
        say(rcon_send(["stop"])[0])
        return
    jar = RUNTIME / f"server-{VERSION}.jar"
    if not jar.is_file():
        raise SystemExit(f"{jar} is missing: run `python3 tools/voxel.py setup` first")
    directory.mkdir(parents=True, exist_ok=True)

    eula = directory / "eula.txt"
    if "eula=true" not in (eula.read_text(encoding="utf-8") if eula.is_file() else ""):
        if not args.accept_eula:
            raise SystemExit(
                "The server needs the Minecraft EULA (https://aka.ms/MinecraftEULA) "
                "accepted. Read it, then run again with --accept-eula."
            )
        eula.write_text("eula=true\n", encoding="utf-8")

    # Settings the tools rely on are written over whatever is there; the
    # rest of the file is left alone.
    properties_path = directory / "server.properties"
    lines = properties_path.read_text(encoding="utf-8").splitlines() if properties_path.is_file() else []
    wanted = dict(SERVER_PROPERTIES)
    kept = []
    for line in lines:
        key = line.split("=", 1)[0] if "=" in line and not line.startswith("#") else None
        if key in wanted:
            kept.append(f"{key}={wanted.pop(key)}")
        else:
            kept.append(line)
    kept.extend(f"{key}={value}" for key, value in wanted.items())
    properties_path.write_text("\n".join(kept) + "\n", encoding="utf-8")

    java = find_java(args.java)
    command = [java, f"-Xmx{args.memory}", "-jar", str(jar), "nogui"]
    if not args.background:
        say(f"starting the server on port {SERVER_PORT} (stop with `stop` or Ctrl+C)")
        subprocess.run(command, cwd=directory)
        return
    log = open(directory / "server.log", "w", encoding="utf-8")
    options = {"cwd": directory, "stdout": log, "stderr": subprocess.STDOUT,
               "stdin": subprocess.DEVNULL}
    if os.name == "nt":
        options["creationflags"] = (subprocess.DETACHED_PROCESS
                                    | subprocess.CREATE_NEW_PROCESS_GROUP)
    else:
        options["start_new_session"] = True
    process = subprocess.Popen(command, **options)
    say(f"server started in the background (pid {process.pid}), log: {directory / 'server.log'}")
    # Ready once both the game port and RCON, which opens a little later,
    # take connections.
    waiting = [SERVER_PORT, RCON_PORT]
    for _ in range(180):
        if process.poll() is not None:
            raise SystemExit("the server exited; see the log")
        try:
            with socket.create_connection(("127.0.0.1", waiting[0]), timeout=1):
                waiting.pop(0)
        except OSError:
            time.sleep(1)
        if not waiting:
            say(f"ready: game on 127.0.0.1:{SERVER_PORT}, RCON on {RCON_PORT}")
            return
    say("the server is not ready yet; see the log")


# ---------------------------------------------------------------- play


def play(args):
    binary = ROOT / "target" / "release" / ("voxel.exe" if os.name == "nt" else "voxel")
    if not args.no_build:
        subprocess.run(["cargo", "build", "--release"], cwd=ROOT, check=True)
    assets = ASSETS / "jar" / "assets"
    if not assets.is_dir():
        raise SystemExit(f"{assets} is missing: run `python3 tools/voxel.py setup` first")
    environment = dict(os.environ)
    environment["VOXEL_ASSETS"] = str(assets)
    if args.user:
        environment["VOXEL_USER"] = args.user
    if args.fullscreen:
        environment["VOXEL_FULLSCREEN"] = "1"
    if args.camera:
        environment["VOXEL_CAMERA"] = args.camera
    if args.profile:
        environment["VOXEL_PROFILE"] = "1"
    command = [str(binary), "play", args.address]
    if args.size:
        command += args.size
    raise SystemExit(subprocess.run(command, cwd=ROOT, env=environment).returncode)


# ---------------------------------------------------------------- rcon


def rcon_send(commands, host="127.0.0.1", port=RCON_PORT, password=RCON_PASSWORD):
    """Send commands over RCON and return the replies."""

    def packet(connection, request_id, kind, body):
        data = struct.pack("<ii", request_id, kind) + body.encode("utf-8") + b"\0\0"
        connection.sendall(struct.pack("<i", len(data)) + data)
        length = struct.unpack("<i", connection.recv(4))[0]
        payload = b""
        while len(payload) < length:
            payload += connection.recv(length - len(payload))
        return struct.unpack("<i", payload[:4])[0], payload[8:-2].decode("utf-8", "replace")

    try:
        connection = socket.create_connection((host, port), timeout=10)
    except OSError as error:
        raise SystemExit(f"no RCON on {host}:{port} ({error}); is the server running?")
    with connection:
        answered, _ = packet(connection, 1, 3, password)
        if answered == -1:
            raise SystemExit("RCON refused the password")
        return [packet(connection, 2, 2, command)[1] for command in commands]


def rcon(args):
    for command, reply in zip(args.commands, rcon_send(args.commands)):
        say(f"{command} -> {reply}")


# ---------------------------------------------------------------- check


# Every behavior family except `math` (which only runs on x86-64 Linux).
PARITY_FAMILIES = ["atlas", "binary", "block", "entity", "gui", "image", "json", "model",
                   "movement", "nbt", "resources", "sound", "timer", "utf8", "window", "wire"]
NOISE = ("block v0.1.6", "future-incompat")


def tool_environment():
    environment = dict(os.environ)
    environment["VOXEL_ASSETS"] = str(ASSETS / "jar" / "assets")
    if sys.platform == "darwin" and Path("/usr/local/lib").is_dir():
        environment["DYLD_LIBRARY_PATH"] = "/usr/local/lib"
    return environment


def run_captured(command):
    result = subprocess.run(command, cwd=ROOT, env=tool_environment(), text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    return result.returncode, result.stdout.splitlines()


def diagnostics(lines, limit=60):
    """Warning and error blocks of a cargo run, without the known noise."""
    shown, context = [], 0
    for line in lines:
        if line.startswith(("warning", "error")):
            context = 10
            keep = not any(noise in line for noise in NOISE)
        elif context:
            context -= 1
        else:
            keep = False
            continue
        if keep:
            shown.append(line)
    return shown[:limit]


def parity_summary(capture, families):
    """Run the Java/Rust comparison and return (ok, one summary line)."""
    families = families or PARITY_FAMILIES
    command = [sys.executable, "tools/behavior/run.py", "capture" if capture else "compare"]
    for family in families:
        command += ["--family", family]
    adapter = "target/release/examples/behavior_adapter"
    if capture:
        _, lines = run_captured(command)
        last = parse_report(lines)
        if last.get("status") != "reference_ready":
            return False, f"capture: {summary_of(last, lines)}"
    _, lines = run_captured([sys.executable, "tools/behavior/run.py", "compare",
                             *command[3:], "--adapter", adapter])
    report = parse_report(lines)
    return report.get("status") == "parity_passed", f"parity: {summary_of(report, lines)}"


def parse_report(lines):
    for line in reversed(lines):
        try:
            report = json.loads(line)
        except ValueError:
            continue
        if isinstance(report, dict):
            return report
    return {}


def summary_of(report, lines):
    if not report:
        return " | ".join(lines[-3:])
    difference = str(report.get("difference"))[:600]
    return f"{report.get('status')} {report.get('case_count')} {difference} {report.get('reason') or ''}".strip()


def check(args):
    ok = True

    def step(label, command, fatal_only_on_output=False):
        nonlocal ok
        code, lines = run_captured(command)
        problems = diagnostics(lines)
        failed = code != 0 or (fatal_only_on_output and problems)
        if failed or problems:
            ok = False
            say(f"{label}: problems")
            for line in (problems or lines[-40:]):
                print(line)
        return lines

    run_captured(["cargo", "fmt", "--all"])
    step("build", ["cargo", "build", "--release", "--workspace", "--all-targets"], True)
    code, lines = run_captured(["cargo", "test", "--release", "--workspace", "--all-targets"])
    passed = failed = 0
    for line in lines:
        found = re.match(r"test result: \w+\. (\d+) passed; (\d+) failed", line)
        if found:
            passed += int(found.group(1))
            failed += int(found.group(2))
        elif "FAILED" in line or "panicked" in line:
            print(line)
    print(f"tests passed {passed} failed {failed}")
    if code != 0 or failed:
        ok = False
        if not failed:
            print("\n".join(lines[-30:]))
    step("clippy", ["cargo", "clippy", "--release", "--workspace", "--all-targets"], True)
    good, line = parity_summary(args.capture, args.family)
    print(line)
    ok = ok and good
    for script in ("generate-doc-index.py", "check-doc-cells.py", "check-doc-links.py"):
        code, lines = run_captured([sys.executable, f"tools/{script}"])
        if code != 0:
            ok = False
            print("\n".join(lines[-15:]))
    print("CHECK OK" if ok else "CHECK FAILED")
    raise SystemExit(0 if ok else 1)


def parity(args):
    good, line = parity_summary(args.capture, args.family)
    print(line)
    raise SystemExit(0 if good else 1)


# ---------------------------------------------------------------- shot


def shot(args):
    """Run the client with a script, send RCON commands once joined, save the first frame."""
    directory = DATA / "runs" / "shots"
    directory.mkdir(parents=True, exist_ok=True)
    prefix = directory / args.name
    for stale in directory.glob(f"{args.name}.*"):
        stale.unlink()
    binary = ROOT / "target" / "release" / ("voxel.exe" if os.name == "nt" else "voxel")
    environment = tool_environment()
    environment.update({"VOXEL_USER": args.user, "VOXEL_SCRIPT": args.script,
                        "VOXEL_CAPTURE_PPM": str(prefix), "VOXEL_CAPTURE_FRAMES": "1"})
    log = open(f"{prefix}.log", "w", encoding="utf-8")
    process = subprocess.Popen([str(binary), "play", args.address, "854", "480"], cwd=ROOT,
                               env=environment, stdout=log, stderr=subprocess.STDOUT)
    if args.rcon:
        time.sleep(args.wait)
        for command, reply in zip(args.rcon, rcon_send(args.rcon)):
            say(f"{command} -> {reply[:80]}")
    process.wait()
    log.close()
    frame = Path(f"{prefix}.000.ppm")
    if frame.exists() and shutil.which("sips"):
        subprocess.run(["sips", "-s", "format", "png", str(frame), "--out", f"{prefix}.png"],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    say(f"frame: {prefix}.png" if Path(f"{prefix}.png").exists() else f"frame: {frame}")
    for line in Path(f"{prefix}.log").read_text(encoding="utf-8").splitlines()[-2:]:
        print(line)


# ---------------------------------------------------------------- bench

# Standing still and turning through the four directions, so every run draws
# the same world from the same place. 40 seconds of ticks; the first seconds
# (VOXEL_BENCH_WARMUP) are left out while the sections mesh.
BENCH_SCRIPT = "none=200;turn:90;none=100;turn:90;none=100;turn:90;none=100;turn:90;none=300"
# A user of its own, so its saved position is the world's spawn and stays there.
BENCH_USER = "Bench"


def server_listening(address):
    host, _, port = address.rpartition(":")
    try:
        with socket.create_connection((host, int(port)), timeout=2):
            return True
    except OSError:
        return False


def bench_summary(report):
    """One line: the frame's work and the GPU's scene time, as p50/p95/p99 milliseconds."""
    def triple(stats):
        return f"{stats['p50']:.2f}/{stats['p95']:.2f}/{stats['p99']:.2f}"

    gpu = report["gpu_ms"]
    if not gpu:
        gpu_text = " gpu=unavailable"
    else:
        passes = "" if gpu["pass_breakdown_resolved"] else " (no per-pass breakdown on this driver)"
        gpu_text = f" gpu_scene={triple(gpu['scene'])}{passes}"
    return (f"frames={report['frames']} work_ms={triple(report['work_ms'])} "
            f"work_fps={report['work_fps']:.0f}{gpu_text}")


def bench(args):
    """Run the client on a fixed script against the test server and keep its timing report."""
    if not server_listening(args.address):
        raise SystemExit(f"no server on {args.address}: run `python3 tools/voxel.py server --accept-eula --background`")
    assets = ASSETS / "jar" / "assets"
    if not assets.is_dir():
        raise SystemExit(f"{assets} is missing: run `python3 tools/voxel.py setup` first")
    if not args.no_build:
        subprocess.run(["cargo", "build", "--release", "-p", "voxel"], cwd=ROOT, check=True)
    binary = ROOT / "target" / "release" / ("voxel.exe" if os.name == "nt" else "voxel")
    directory = DATA / "runs" / "bench"
    directory.mkdir(parents=True, exist_ok=True)
    reports = []
    for number in range(1, args.repeat + 1):
        path = directory / f"{args.name}-{number}.json"
        path.unlink(missing_ok=True)
        environment = tool_environment()
        environment.update({"VOXEL_USER": BENCH_USER, "VOXEL_SCRIPT": BENCH_SCRIPT,
                            "VOXEL_BENCH": str(path), "VOXEL_BENCH_WARMUP": str(args.warmup)})
        result = subprocess.run([str(binary), "play", args.address, *args.size], cwd=ROOT,
                                env=environment, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if result.returncode != 0 or not path.exists():
            print("\n".join(result.stdout.splitlines()[-15:]))
            raise SystemExit(f"bench run {number} failed (exit {result.returncode}); no report at {path}")
        reports.append(json.loads(path.read_text(encoding="utf-8")))
        say(f"run {number}: {bench_summary(reports[-1])}")
    first = reports[0]
    say(f"{first['device']} {first['extent'][0]}x{first['extent'][1]} render_distance={first['render_distance']} "
        f"chunks={first['loaded_chunks']} entities={first['entities']}")
    # The frame is pinned at the display's refresh time under FIFO, so its p50
    # alone proves little: the waits, the meshing span and the GPU's scene time
    # are compared too.
    measures = {
        "work_ms": lambda report: report["work_ms"]["p50"],
        "fence_wait": lambda report: report["cpu_ms"]["fence_wait"]["p50"],
        "mesh": lambda report: report["cpu_ms"]["mesh"]["p50"],
        "gpu_scene": lambda report: report["gpu_ms"]["scene"]["p50"] if report["gpu_ms"] else None,
    }
    ok = True
    for before, after in zip(reports, reports[1:]):
        changes = {}
        for name, pick in measures.items():
            old, new = pick(before), pick(after)
            if old and new is not None:
                changes[name] = abs(new - old) / old * 100
        say("p50 change between consecutive runs: " + ", ".join(f"{name} {change:.1f}%" for name, change in changes.items()))
        ok = ok and all(change < 5 for change in changes.values())
    if not ok:
        raise SystemExit("a p50 moved 5% or more between runs: the machine was not quiet enough to compare")


# ---------------------------------------------------------------- main


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)

    setup_parser = commands.add_parser("setup", help="download the reference game into .data/")
    setup_parser.add_argument("--skip-objects", action="store_true",
                              help="leave out the asset objects (sounds, languages; ~470 MB)")
    setup_parser.add_argument("--named", action="store_true",
                              help="also build assets/named, a browsable tree of the objects")
    setup_parser.add_argument("--decompile", action="store_true",
                              help=f"decompile the client (and JOML) with Vineflower; needs Java {JAVA_MAJOR}")
    setup_parser.add_argument("--java", help=f"path to a Java {JAVA_MAJOR} `java`")
    setup_parser.set_defaults(run=setup)

    server_parser = commands.add_parser("server", help="set up and start the test server")
    server_parser.add_argument("--accept-eula", action="store_true",
                               help="accept the Minecraft EULA (https://aka.ms/MinecraftEULA)")
    server_parser.add_argument("--background", action="store_true",
                               help="start detached, logging to server.log, and wait for the port")
    server_parser.add_argument("--stop", action="store_true", help="stop a running server over RCON")
    server_parser.add_argument("--dir", default=str(SERVER_DIR), help="server directory")
    server_parser.add_argument("--memory", default="2G", help="Java heap size (default 2G)")
    server_parser.add_argument("--java", help=f"path to a Java {JAVA_MAJOR} `java`")
    server_parser.set_defaults(run=server)

    bench_parser = commands.add_parser("bench", help="fixed scripted run, frame timing report in .data/runs/bench")
    bench_parser.add_argument("address", nargs="?", default=f"127.0.0.1:{SERVER_PORT}")
    bench_parser.add_argument("--name", default="bench", help="report name (default bench)")
    bench_parser.add_argument("--repeat", type=int, default=1, help="runs in a row, compared by p50")
    bench_parser.add_argument("--warmup", type=float, default=10, help="seconds left out at the start")
    bench_parser.add_argument("--size", nargs=2, default=["1280", "720"], metavar=("WIDTH", "HEIGHT"))
    bench_parser.add_argument("--no-build", action="store_true", help="skip `cargo build --release`")
    bench_parser.set_defaults(run=bench)

    play_parser = commands.add_parser("play", help="build and start the client")
    play_parser.add_argument("address", nargs="?", default=f"127.0.0.1:{SERVER_PORT}")
    play_parser.add_argument("--user", help="player name (default VoxelClient)")
    play_parser.add_argument("--size", nargs=2, metavar=("WIDTH", "HEIGHT"), help="window size")
    play_parser.add_argument("--fullscreen", action="store_true")
    play_parser.add_argument("--camera", choices=["back", "front"], help="start in third person")
    play_parser.add_argument("--profile", action="store_true", help="print frame timings")
    play_parser.add_argument("--no-build", action="store_true", help="skip `cargo build --release`")
    play_parser.set_defaults(run=play)

    rcon_parser = commands.add_parser("rcon", help="send commands to the test server")
    rcon_parser.add_argument("commands", nargs="+")
    rcon_parser.set_defaults(run=rcon)

    check_parser = commands.add_parser("check", help="commit gate: fmt, build, test, clippy, parity, docs")
    check_parser.add_argument("--capture", action="store_true",
                              help="capture the Java reference first (needed after oracle source changes)")
    check_parser.add_argument("--family", action="append", help="repeat to limit the parity families")
    check_parser.set_defaults(run=check)

    parity_parser = commands.add_parser("parity", help="Java/Rust comparison, one summary line")
    parity_parser.add_argument("--capture", action="store_true", help="capture the Java reference first")
    parity_parser.add_argument("--family", action="append", help="repeat to select families")
    parity_parser.set_defaults(run=parity)

    shot_parser = commands.add_parser("shot", help="scripted client run with a screenshot")
    shot_parser.add_argument("name")
    shot_parser.add_argument("script", help="VOXEL_SCRIPT steps, e.g. 'none=60;escape=1;none=5'")
    shot_parser.add_argument("--user", default="Miner")
    shot_parser.add_argument("--address", default=f"127.0.0.1:{SERVER_PORT}")
    shot_parser.add_argument("--rcon", action="append", help="command to send after --wait seconds")
    shot_parser.add_argument("--wait", type=float, default=9.0)
    shot_parser.set_defaults(run=shot)

    args = parser.parse_args()
    args.run(args)


if __name__ == "__main__":
    main()
