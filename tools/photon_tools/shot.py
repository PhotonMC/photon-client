"""A scripted client run with a screenshot."""

import os
import shutil
import subprocess
import time
from pathlib import Path

from .common import DATA, ROOT, say, tool_environment
from .server import rcon_send


def shot(args):
    """Run the client with a script, send RCON commands once joined, save the first frame."""
    directory = DATA / "runs" / "shots"
    directory.mkdir(parents=True, exist_ok=True)
    prefix = directory / args.name
    for stale in directory.glob(f"{args.name}.*"):
        stale.unlink()
    binary = ROOT / "target" / "release" / ("photon.exe" if os.name == "nt" else "photon")
    environment = tool_environment()
    environment.update({"PHOTON_USER": args.user, "PHOTON_SCRIPT": args.script,
                        "PHOTON_CAPTURE_PPM": str(prefix), "PHOTON_CAPTURE_FRAMES": "1"})
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
