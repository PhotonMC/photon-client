"""Downloading and decompiling the reference game."""

import concurrent.futures
import json
import os
import shutil
import subprocess
import zipfile
from pathlib import Path

from .common import ASSETS, MINECRAFT, PIN, RESOURCES, RUNTIME, VERSION, VERSION_MANIFEST, fetch, fetch_json, find_java, say, sha1_of
from .server import server


VINEFLOWER_VERSION = "1.12.0"
VINEFLOWER_URL = (
    "https://repo1.maven.org/maven2/org/vineflower/vineflower/"
    f"{VINEFLOWER_VERSION}/vineflower-{VINEFLOWER_VERSION}.jar"
)


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
