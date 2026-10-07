"""A fixed scripted run with a frame timing report."""

import json
import os
import socket
import subprocess

from .common import ASSETS, DATA, ROOT, say, tool_environment


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
            f"work_fps={report['work_fps']:.0f} wall_fps={report['wall_fps']:.1f}{gpu_text}")


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
        if args.validation:
            environment["VOXEL_VALIDATION"] = "1"
        environment["VOXEL_VSYNC"] = "0" if args.vsync == "off" else "1"
        if args.fps_limit:
            environment["VOXEL_MAX_FPS"] = str(args.fps_limit)
        result = subprocess.run([str(binary), "play", args.address, *args.size], cwd=ROOT,
                                env=environment, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if result.returncode != 0 or not path.exists():
            print("\n".join(result.stdout.splitlines()[-15:]))
            raise SystemExit(f"bench run {number} failed (exit {result.returncode}); no report at {path}")
        path.with_suffix(".log").write_text(result.stdout, encoding="utf-8")
        reports.append(json.loads(path.read_text(encoding="utf-8")))
        say(f"run {number}: {bench_summary(reports[-1])}")
        if args.validation:
            findings = reports[-1]["validation"]
            say(f"validation: {findings['errors']} errors, {findings['warnings']} warnings (see {path.with_suffix('.log')})")
    first = reports[0]
    say(f"{first['device']} {first['extent'][0]}x{first['extent'][1]} present={first['present_mode']} "
        f"render_distance={first['render_distance']} "
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
