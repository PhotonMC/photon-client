# Development setup

## Requirements

- **Rust.** The toolchain pinned in `rust-toolchain.toml`. GLFW and shaderc build from source, so CMake and a C/C++ compiler are also needed (Visual Studio Build Tools on Windows, the Xcode command line tools on macOS).
- **Vulkan.** A Vulkan driver; on macOS the Vulkan loader with MoltenVK.
- **Java.** Java 25 for the test server, the parity oracle and `setup --decompile`. The reference setup itself needs only the Rust toolchain ([ADR 0009](../adr/0009-xtask-downloads.md)).

## First steps

```sh
cargo xtask install-hooks       # pre-commit, commit-msg and pre-push hooks
cargo build --workspace
cargo xtask setup               # pinned game into .data/ from Mojang's servers
```

`setup` downloads the pinned 26.2 client and server jars, libraries, the resources inside the jar and the asset objects from Mojang's official distribution into `.data/minecraft/`, and checks every file against its published hash. It stops if the client jar's hash does not match `tests/behavior/reference.json`. Options:

- `--skip-objects` skips sound and language objects.
- `--named` builds a browsable view of the objects.
- `--decompile` extracts readable sources with Vineflower into `.data/minecraft/source`.

`.data/` never enters Git and is never uploaded anywhere.

## Running the client

```sh
cargo xtask server --accept-eula --background   # test server on port 25600
cargo xtask play --user Walker               # title screen; Multiplayer lists servers
cargo xtask play --user Walker --direct      # join the test server at once
cargo xtask rcon "time set noon" "weather clear"
cargo xtask server --stop
```

The test server is offline, peaceful, seed 776, view distance 32, with RCON on 25601. Accepting the Minecraft EULA is your responsibility.

`play` builds in release mode and starts the client with `PHOTON_ASSETS` set. Other `PHOTON_*` variables are passed through:

- `PHOTON_LOG` sets the log level.
- `PHOTON_PROFILE` prints frame timing.
- `PHOTON_VALIDATION` enables the Vulkan validation layer.
- `PHOTON_CULLING=0` disables culling for comparison.

`cargo xtask bench` runs a fixed scripted session against the running server and writes a frame-timing report to `.data/runs/bench/`. `cargo xtask shot` runs a scripted session and saves its first frame as a PNG in `.data/runs/shots/`.

## Checks

| Command | What it runs |
| --- | --- |
| `cargo xtask lint` | Repository rules (~150 ms) |
| `cargo xtask pre-commit` | rustfmt on staged files, lint, clippy on affected crates |
| `cargo xtask pre-push` | Tests of affected crates, parity if engine or reference sources changed |
| `cargo xtask check` | Everything: fmt, lint, clippy, tests, cargo-deny, cargo-machete, parity |
| `cargo xtask affected` | The crates changed since the push base, and their dependents |

The hooks run the same commands. The rules themselves are in the [coding standards](coding-standards.md).

## Joining an online-mode server

`cargo xtask server` runs an offline server, which skips encryption. To exercise the encrypted login, run a vanilla server with `online-mode=true` and `enforce-secure-profile=false`, keep its standard input open (it stops on end of input), and point its session lookups at a stub host with `-Dminecraft.api.session.host=URL -Dminecraft.api.services.host=URL -Dminecraft.api.profiles.host=URL` (all three are needed). The stub answers `POST /session/minecraft/join` by remembering the server hash, `GET /session/minecraft/hasJoined` by returning the profile for a remembered hash (204 otherwise), and the server's `GET /publickeys` at startup with a 204.

The client takes its account from `PHOTON_UUID`, `PHOTON_ACCESS_TOKEN` and, for a host other than Mojang's, `PHOTON_SESSION_HOST` (plain `http://` is only accepted for this machine). The ignored test `cargo test -p photon-net --test online_join -- --ignored` joins with them as well as `PHOTON_LIVE_ADDRESS` and `PHOTON_LIVE_NAME`.

## Platform notes

- **The `math` family** carries NaN bit patterns recorded on x86-64 Linux; arm64 Java produces differently signed NaNs in some operations, so a few `math` cases differ there.
