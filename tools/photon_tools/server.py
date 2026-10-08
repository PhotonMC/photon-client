"""The test server, the client launcher and RCON."""

import os
import socket
import struct
import subprocess
import time
from pathlib import Path

from .common import ASSETS, ROOT, RUNTIME, VERSION, find_java, say


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
    "motd": "photon test server",
}


def server(args):
    directory = Path(args.dir).resolve()
    if args.stop:
        say(rcon_send(["stop"])[0])
        return
    jar = RUNTIME / f"server-{VERSION}.jar"
    if not jar.is_file():
        raise SystemExit(f"{jar} is missing: run `python3 tools/photon.py setup` first")
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


def play(args):
    binary = ROOT / "target" / "release" / ("photon.exe" if os.name == "nt" else "photon")
    if not args.no_build:
        subprocess.run(["cargo", "build", "--release"], cwd=ROOT, check=True)
    assets = ASSETS / "jar" / "assets"
    if not assets.is_dir():
        raise SystemExit(f"{assets} is missing: run `python3 tools/photon.py setup` first")
    environment = dict(os.environ)
    environment["PHOTON_ASSETS"] = str(assets)
    if args.user:
        environment["PHOTON_USER"] = args.user
    if args.fullscreen:
        environment["PHOTON_FULLSCREEN"] = "1"
    if args.camera:
        environment["PHOTON_CAMERA"] = args.camera
    if args.profile:
        environment["PHOTON_PROFILE"] = "1"
    command = [str(binary), "play", args.address]
    if args.size:
        command += args.size
    raise SystemExit(subprocess.run(command, cwd=ROOT, env=environment).returncode)


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
