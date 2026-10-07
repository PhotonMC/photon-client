# Local setup and running

> English translation of [`../local-setup.md`](../local-setup.md). The Turkish original is authoritative.

`tools/voxel.py` is a developer tool that works the same way on Linux, macOS and Windows. It uses only the Python standard library. On Windows, `python` can be written instead of `python3`. Everything it downloads goes into the `.data/` folder outside the repository. This folder never enters Git; the distributed client (the `Cargo.toml` package list) carries neither this tool nor the contents of `.data/`.

## Requirements

- Python 3.9 or newer.
- Rust (`cargo`).
  - GLFW and shaderc (at build time only, to compile the shaders to SPIR-V) are built from source, so CMake and a C/C++ compiler are needed: Visual Studio Build Tools on Windows, the Xcode command line tools on macOS.
- A driver that supports Vulkan. On macOS the Vulkan loader with MoltenVK.
- Java 25 for the server and for decompiling. The tool looks for a Java 25 `java` in `--java`, `JAVA_HOME` and `PATH`, in that order.

## Commands

```sh
python3 tools/voxel.py setup
python3 tools/voxel.py server --accept-eula --background
python3 tools/voxel.py play --user Walker
python3 tools/voxel.py rcon "time set noon" "weather clear"
python3 tools/voxel.py server --stop
```

- **`setup`:** downloads the pinned 26.2 version from Mojang's official distribution and installs it under `.data/minecraft/`:
  - the version file, the client and server jars, all libraries, verified by SHA1;
  - the resources inside the jar (`assets/jar`);
  - the asset index and its objects (sounds, languages; skipped with `--skip-objects`).

  A file that exists and matches its hash is not downloaded again. It stops if the client jar's hash does not match `tests/behavior/reference.json`.

  Options:
  - `--named`: builds a browsable tree of the objects (a copy if links cannot be created).
  - `--decompile`: extracts the client and JOML with Vineflower into `.data/minecraft/source` and `joml-source`. An existing source folder is deleted and produced again.
- **`server`:** sets up the test server under `.data/render/walk`.
  - The Minecraft EULA is accepted only when `--accept-eula` is given.
  - The settings the tools depend on are written every time and the rest of the file is left alone: port 25600, offline, peaceful, seed 776, view distance 12, RCON 25601 (password `walk`).
  - `--background` starts the server as a separate process, writes its log to `server.log` and waits until the game port and RCON are open.
  - `--stop` stops the server over RCON.
- **`play`:** runs `cargo build --release` (`--no-build` skips it) and starts the client with `VOXEL_ASSETS` set. Options: `--user`, `--size`, `--fullscreen`, `--camera back|front`, `--profile`. Other `VOXEL_*` variables are passed through from the environment unchanged.
- **`rcon`:** sends the commands in order and prints the replies.
- **`bench`:** connects to the running test server with a fixed script (standing still and turning through the four directions, 40 s) and writes a frame timing report under `.data/runs/bench/`. Options: `address`, `--repeat N` (consecutive runs are compared by p50; a difference of 5% or more is an error), `--warmup`, `--size`, `--name`, `--vsync on|off`, `--fps-limit N` (unlimited by default, so the work itself is measured), `--validation` (the Vulkan validation layer; its error count goes into the report). It does not start the server itself.

`--decompile` has not been run on this machine. The command line is the same as the old `decompile.sh` that produces the reference sources.

## Open items on macOS (to look at again on Linux)

Seen on macOS arm64 (Apple Silicon, MoltenVK) on 2026-09-29. When back on Linux all families should be captured and compared again.

- **The `math` family does not pass on arm64.** The Rust side was written against NaN bit patterns captured on x86 Linux; on arm64 Java produces a differently signed NaN in some operations (the first difference is `vec3/rotation/subnormal`, expected `7fc00000`, Rust `ffc00000`). Since the comparison stops at the first difference, it is unknown how many cases are affected. The 14 families other than `math` (904 cases) pass on macOS.
- **`input-with-modifiers` is now by platform.** `InputQuirks` chooses the editing shortcut with Cmd on macOS; it passed on macOS. The Ctrl values on Linux did not change but should be verified again.
- **The small families cannot be captured alone.** `binary`, `json`, `nbt`, `timer`, `utf8` and `window` do not compile with a single `--family` (they depend on classes that are not on the shared `Oracle.java` class path); this is the same at HEAD and looks platform independent. They pass when run together with other families.
- **The `libshaderc_shared` problem is gone.** Shaders are compiled at build time (V02), so the binary no longer links shaderc; `DYLD_LIBRARY_PATH` is not needed.
