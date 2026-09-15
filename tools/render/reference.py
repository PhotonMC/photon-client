#!/usr/bin/env python3
"""Launch the pinned Java client under a fixed configuration and capture it.

This is the reference half of the render track. It does not compare anything:
it makes a Java run reproducible enough that later slices can compare against
it, and it records what is actually pinned versus what still drifts.

No downloads. Everything comes from `.data/minecraft`, and every capture stays
under `.data/render`.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / ".data"
RUNTIME = DATA / "minecraft" / "runtime"
ASSETS = DATA / "minecraft" / "assets"
WORK = DATA / "render"
PIN = ROOT / "tests/behavior/reference.json"

# Written verbatim before every run so the client never inherits a previous
# session's settings. Values that change what is drawn are pinned here; the
# ones that only silence the environment are marked as such.
PINNED_OPTIONS = {
    # drawing
    "guiScale": "2",
    "fov": "0.0",
    "renderDistance": "8",
    "simulationDistance": "8",
    "graphicsMode": "1",
    "ambientOcclusion": "true",
    "particles": "0",
    "entityShadows": "true",
    "biomeBlendRadius": "2",
    "mipmapLevels": "4",
    "gamma": "0.0",
    "bobView": "false",
    "screenEffectScale": "0.0",
    "fovEffectScale": "0.0",
    "darknessEffectScale": "0.0",
    "glintSpeed": "0.0",
    "glintStrength": "0.0",
    "enableVsync": "false",
    "maxFps": "60",
    "lang": "en_us",
    # environment only: not drawn, kept off so a run is quiet and headless-safe
    "narrator": "0",
    "narratorHotkey": "false",
    "soundCategory_master": "0.0",
    "showSubtitles": "false",
    "autosaveIndicator": "false",
    "pauseOnLostFocus": "false",
    "skipMultiplayerWarning": "true",
    "onboardAccessibility": "false",
    "tutorialStep": "none",
}

READY_PATTERN = re.compile(r"Using graphics backend (\w+), using drivers: (.+)")


class Blocked(Exception):
    pass


class Failed(Exception):
    pass


def digest(path):
    return hashlib.sha1(path.read_bytes()).hexdigest()


def classpath():
    pin = json.loads(PIN.read_text(encoding="utf-8"))
    client = RUNTIME / pin["client_file"]
    if not client.is_file():
        raise Blocked(f"Missing pinned client: {client}")
    if digest(client) != pin["client_sha1"]:
        raise Blocked(f"Client hash mismatch: {client}")
    libs = sorted((RUNTIME / "libs").glob("*.jar"))
    if not libs:
        raise Blocked("No libraries under runtime/libs")
    host = f"natives-{ 'linux' if platform.system() == 'Linux' else platform.system().lower() }"
    usable = [
        jar for jar in libs
        if "natives" not in jar.name or host in jar.name
    ]
    return os.pathsep.join([str(client), *(str(jar) for jar in usable)]), pin


def write_options(game_dir):
    game_dir.mkdir(parents=True, exist_ok=True)
    lines = [f"{key}:{value}" for key, value in sorted(PINNED_OPTIONS.items())]
    (game_dir / "options.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def find_window(timeout):
    """Wait for the client window to exist, returning its X11 id.

    The client runs under XWayland here, which is what makes both key injection
    and per-window capture possible; a native Wayland session would need a
    different route and that is recorded as an open dependency.
    """
    if not shutil.which("xdotool"):
        raise Blocked("xdotool missing; cannot locate the client window")
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        found = subprocess.run(
            ["xdotool", "search", "--name", "--", "Minecraft"],
            capture_output=True, text=True, check=False,
        )
        ids = [line for line in found.stdout.split() if line.strip()]
        if ids:
            return ids[-1]
        time.sleep(0.5)
    raise Failed(f"Client window did not appear within {timeout}s")


def capture_window(window_id, destination):
    if not shutil.which("import"):
        raise Blocked("ImageMagick 'import' missing; cannot capture the window")
    destination.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["import", "-window", window_id, str(destination)],
        capture_output=True, check=True,
    )
    return destination


def launch(args):
    cp, pin = classpath()
    if not ASSETS.is_dir():
        raise Blocked(f"Missing assets: {ASSETS}")
    run_id = time.strftime("%Y%m%d-%H%M%S")
    run_dir = WORK / "runs" / f"{args.backend}-{run_id}"
    game_dir = WORK / "gamedir" / args.backend
    run_dir.mkdir(parents=True, exist_ok=True)
    write_options(game_dir)

    command = [
        args.java, f"-Xmx{args.heap}", "-cp", cp,
        "net.minecraft.client.main.Main",
        "--gameDir", str(game_dir),
        "--assetsDir", str(ASSETS),
        "--assetIndex", args.asset_index,
        "--version", pin["version"],
        "--accessToken", "0",
        "--username", args.username,
        "--offlineDeveloperMode",
        "--width", str(args.width),
        "--height", str(args.height),
        "--graphicsBackend", args.backend,
    ]
    log_path = run_dir / "client.log"
    with log_path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
        try:
            backend, drivers = wait_for_backend(log_path, args.ready_timeout, process)
            window_id = find_window(args.ready_timeout)
            time.sleep(args.settle)
            captures = []
            for index in range(args.captures):
                target = run_dir / f"frame-{index:02d}.png"
                capture_window(window_id, target)
                captures.append(str(target.relative_to(ROOT)))
                if index + 1 < args.captures:
                    time.sleep(args.interval)
        finally:
            process.terminate()
            try:
                process.wait(timeout=20)
            except subprocess.TimeoutExpired:
                process.kill()

    report = {
        "command": "launch",
        "status": "reference_running",
        "minecraft_version": pin["version"],
        "requested_backend": args.backend,
        "reported_backend": backend,
        "drivers": drivers,
        "window_size": f"{args.width}x{args.height}",
        "captures": captures,
        "run_dir": str(run_dir.relative_to(ROOT)),
        "pinned_options": len(PINNED_OPTIONS),
        "scene": "title_screen",
        "deterministic_scene": False,
        "note": "Title screen animates; deterministic scene and frame comparison are the next slice",
    }
    print(json.dumps(report, sort_keys=True))
    return 0


def wait_for_backend(log_path, timeout, process):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise Failed(f"Client exited early with code {process.returncode}; see {log_path}")
        if log_path.is_file():
            for line in log_path.read_text(encoding="utf-8", errors="replace").splitlines():
                found = READY_PATTERN.search(line)
                if found:
                    return found.group(1), found.group(2)
        time.sleep(0.5)
    raise Failed(f"Client did not report a graphics backend within {timeout}s")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    run = sub.add_parser("launch", help="launch the pinned client and capture its window")
    run.add_argument("--backend", choices=("opengl", "vulkan"), default="opengl")
    run.add_argument("--width", type=int, default=854)
    run.add_argument("--height", type=int, default=480)
    run.add_argument("--captures", type=int, default=2)
    run.add_argument("--interval", type=float, default=1.0)
    run.add_argument("--settle", type=float, default=5.0)
    run.add_argument("--ready-timeout", type=float, default=120.0)
    run.add_argument("--heap", default="2G")
    run.add_argument("--java", default="java")
    run.add_argument("--username", default="ParityTest")
    run.add_argument("--asset-index", default="32")
    run.set_defaults(handler=launch)
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
