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


def send_keys(window_id, keys):
    """Send a real key chord to the client window.

    The client runs under XWayland, so XTEST input reaches it the way a user's
    would. Synthetic XSendEvent keys are ignored by GLFW, which is why the
    window is activated first and the chord goes through the test extension.
    """
    subprocess.run(
        ["xdotool", "windowactivate", "--sync", window_id],
        capture_output=True, check=True,
    )
    time.sleep(0.4)
    for hold, tap in keys:
        # Held as a real chord: sending F3 and A as separate taps lets the game
        # read A as strafe-left, which moves a spectator and silently changes
        # the camera the whole measurement depends on.
        subprocess.run(["xdotool", "keydown", hold], capture_output=True, check=True)
        time.sleep(0.2)
        subprocess.run(["xdotool", "key", tap], capture_output=True, check=True)
        time.sleep(0.2)
        subprocess.run(["xdotool", "keyup", hold], capture_output=True, check=True)
        time.sleep(0.4)


def capture_window(window_id, destination):
    if not shutil.which("import"):
        raise Blocked("ImageMagick 'import' missing; cannot capture the window")
    destination.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["import", "-window", window_id, str(destination)],
        capture_output=True, check=True,
    )
    return destination


def install_scene(game_dir, name):
    """Copy the fixed scene in fresh so a run can never mutate the reference."""
    fixture = WORK / "scene" / name
    if not fixture.is_dir():
        raise Blocked(f"Missing scene fixture: {fixture}; run tools/render/scene.py build")
    saves = game_dir / "saves" / name
    if saves.exists():
        shutil.rmtree(saves)
    saves.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(fixture, saves)
    return saves


def launch(args):
    cp, pin = classpath()
    if not ASSETS.is_dir():
        raise Blocked(f"Missing assets: {ASSETS}")
    run_id = time.strftime("%Y%m%d-%H%M%S")
    run_dir = WORK / "runs" / f"{args.backend}-{run_id}"
    game_dir = WORK / "gamedir" / args.backend
    run_dir.mkdir(parents=True, exist_ok=True)
    write_options(game_dir)
    if args.scene:
        install_scene(game_dir, args.scene)

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
    if args.scene:
        command += ["--quickPlaySingleplayer", args.scene]
    log_path = run_dir / "client.log"
    with log_path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
        try:
            backend, drivers = wait_for_backend(log_path, args.ready_timeout, process)
            window_id = find_window(args.ready_timeout)
            time.sleep(args.settle)
            camera_before = None
            if args.reload_chunks:
                # F3+A forces every loaded chunk to re-mesh. Activating the
                # window to deliver it can rotate the view, so the camera is
                # fingerprinted on both sides and the run is rejected if it
                # moved - a moved camera silently invalidates the measurement.
                probe_frame = run_dir / "camera-before.png"
                capture_window(window_id, probe_frame)
                camera_before = camera_signature(probe_frame)
                send_keys(window_id, [("F3", "a")])
                time.sleep(args.reload_settle)
            if camera_before is not None:
                probe_frame = run_dir / "camera-after.png"
                capture_window(window_id, probe_frame)
                camera_after = camera_signature(probe_frame)
                camera_stable = camera_before == camera_after
            else:
                camera_stable = True
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
        "scene": args.scene or "title_screen",
        "deterministic_scene": bool(args.scene),
        "reload_chunks": bool(args.reload_chunks),
        "camera_stable": camera_stable,
    }
    print(json.dumps(report, sort_keys=True))
    if not camera_stable:
        raise Failed(
            "Camera moved while forcing the chunk re-mesh; this run cannot be "
            "compared against others"
        )
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


def read_png(path):
    """Decode a capture to raw RGB rows via ImageMagick, which is already a
    dependency of the capture path."""
    if not shutil.which("magick"):
        raise Blocked("ImageMagick 'magick' missing; cannot decode captures")
    raw = subprocess.run(
        ["magick", str(path), "-depth", "8", "ppm:-"],
        capture_output=True, check=True,
    ).stdout
    fields, index = [], 0
    while len(fields) < 4:
        while raw[index:index + 1].isspace():
            index += 1
        if raw[index:index + 1] == b"#":
            while raw[index:index + 1] != b"\n":
                index += 1
            continue
        end = index
        while not raw[end:end + 1].isspace():
            end += 1
        fields.append(raw[index:end])
        index = end
    return int(fields[1]), int(fields[2]), raw[index + 1:]


def camera_signature(path, row=400, threshold=200):
    """Positions of the bright texture peaks along one scanline.

    Any camera movement changes where the floor pattern lands, including the
    sub-degree rotation a window activation can inject. Region averages and the
    horizon row both survive such a rotation, so this phase is what actually
    detects it.
    """
    width, _, pixels = read_png(path)
    row_values = [pixels[(row * width + x) * 3] for x in range(width)]
    return [
        x for x in range(2, width - 2)
        if row_values[x] > threshold
        and row_values[x] >= row_values[x - 1]
        and row_values[x] > row_values[x + 1]
    ]


def compare(args):
    width, height, first = read_png(args.first)
    other_width, other_height, second = read_png(args.second)
    if (width, height) != (other_width, other_height):
        raise Failed(f"Size mismatch: {width}x{height} vs {other_width}x{other_height}")
    pixels = width * height
    differing = 0
    worst = 0
    total = 0
    for offset in range(0, pixels * 3, 3):
        left = first[offset:offset + 3]
        right = second[offset:offset + 3]
        if left != right:
            differing += 1
            for channel in range(3):
                delta = abs(left[channel] - right[channel])
                total += delta
                worst = max(worst, delta)
    report = {
        "command": "compare",
        "status": "identical" if differing == 0 else "differs",
        "first": str(args.first),
        "second": str(args.second),
        "size": f"{width}x{height}",
        "pixels": pixels,
        "differing_pixels": differing,
        "differing_fraction": round(differing / pixels, 6),
        "max_channel_delta": worst,
        "mean_channel_delta": round(total / (pixels * 3), 6),
    }
    print(json.dumps(report, sort_keys=True))
    return 0


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
    run.add_argument("--reload-chunks", action="store_true",
                     help="force a chunk re-mesh (F3+A) before capturing")
    run.add_argument("--reload-settle", type=float, default=6.0)
    run.add_argument("--scene", default=None,
                     help="fixed scene fixture to load instead of the title screen")
    run.set_defaults(handler=launch)
    diff = sub.add_parser("compare", help="compare two captures pixel by pixel")
    diff.add_argument("first", type=Path)
    diff.add_argument("second", type=Path)
    diff.set_defaults(handler=compare)

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
