# Development setup

## Requirements

- **Rust.** The toolchain pinned in `rust-toolchain.toml`. GLFW and shaderc build from source, so CMake and a C/C++ compiler are also needed (Visual Studio Build Tools on Windows, the Xcode command line tools on macOS).
- **Vulkan.** A Vulkan driver; on macOS the Vulkan loader with MoltenVK.
- **Reference tooling.** Python 3.9+ and Java 25 for the reference setup, the test server and the parity oracle. Python is being replaced by `cargo xtask` ([ADR 0005](../adr/0005-parity-v2.md)).

## First steps

```sh
cargo xtask install-hooks       # pre-commit, commit-msg and pre-push hooks
cargo build --workspace
python3 tools/photon.py setup   # pinned game into .data/ from Mojang's servers
```

`setup` downloads the pinned 26.2 client and server jars, libraries, the resources inside the jar and the asset objects from Mojang's official distribution into `.data/minecraft/`, and checks every file against its published hash. It stops if the client jar's hash does not match `tests/behavior/reference.json`. Options:

- `--skip-objects` skips sound and language objects.
- `--named` builds a browsable view of the objects.
- `--decompile` extracts readable sources with Vineflower into `.data/minecraft/source`.

`.data/` never enters Git and is never uploaded anywhere.

## Running the client

```sh
python3 tools/photon.py server --accept-eula --background   # test server on port 25600
python3 tools/photon.py play --user Walker
python3 tools/photon.py rcon "time set noon" "weather clear"
python3 tools/photon.py server --stop
```

The test server is offline, peaceful, seed 776, view distance 12, with RCON on 25601. Accepting the Minecraft EULA is your responsibility.

`play` builds in release mode and starts the client with `PHOTON_ASSETS` set. Other `PHOTON_*` variables are passed through:

- `PHOTON_LOG` sets the log level.
- `PHOTON_PROFILE` prints frame timing.
- `PHOTON_VALIDATION` enables the Vulkan validation layer.
- `PHOTON_CULLING=0` disables culling for comparison.

`python3 tools/photon.py bench` runs a fixed scripted session against the running server and writes a frame-timing report to `.data/runs/bench/`. `python3 tools/photon.py shot` runs a scripted session and saves a screenshot to `.data/runs/shots/`.

## Checks

| Command | What it runs |
| --- | --- |
| `cargo xtask lint` | Repository rules (~150 ms) |
| `cargo xtask pre-commit` | rustfmt on staged files, lint, clippy on affected crates |
| `cargo xtask pre-push` | Tests of affected crates, parity if engine or reference sources changed |
| `cargo xtask check` | Everything: fmt, lint, clippy, tests, cargo-deny, cargo-machete, parity |
| `cargo xtask affected` | The crates changed since the push base, and their dependents |

The hooks run the same commands. The rules themselves are in the [coding standards](coding-standards.md).

## Platform notes

- **The `math` family** is not in the default set. Its NaN bit patterns were recorded on x86-64 Linux (arm64 Java produces differently signed NaNs), and it currently has known differences after the oracle stopped using HotSpot's `Math` intrinsics. Run it with `cargo xtask parity --family math`.
