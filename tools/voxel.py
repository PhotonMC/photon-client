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

from voxel_tools.bench import bench
from voxel_tools.common import JAVA_MAJOR, SERVER_DIR
from voxel_tools.gate import check, parity
from voxel_tools.reference import setup
from voxel_tools.server import SERVER_PORT, play, rcon, server
from voxel_tools.shot import shot


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
    bench_parser.add_argument("--route", choices=["still", "run"], default="still",
                              help="stand and turn (default), or sprint out and back while chunks stream")
    bench_parser.add_argument("--no-build", action="store_true", help="skip `cargo build --release`")
    bench_parser.add_argument("--vsync", choices=["on", "off"], default="on",
                              help="wait for the display (default on)")
    bench_parser.add_argument("--fps-limit", type=int,
                              help="hold to this frame rate (default: no limit, to measure the work)")
    bench_parser.add_argument("--validation", action="store_true",
                              help="run with the Vulkan validation layer and report what it finds")
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
