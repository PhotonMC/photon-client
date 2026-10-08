"""Paths, pins and the helpers every subcommand shares."""

import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent.parent
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


def tool_environment():
    environment = dict(os.environ)
    environment["PHOTON_ASSETS"] = str(ASSETS / "jar" / "assets")
    if sys.platform == "darwin" and Path("/usr/local/lib").is_dir():
        environment["DYLD_LIBRARY_PATH"] = "/usr/local/lib"
    return environment


def run_captured(command):
    result = subprocess.run(command, cwd=ROOT, env=tool_environment(), text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    return result.returncode, result.stdout.splitlines()
