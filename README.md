# voxel-client

A from-scratch reimplementation of the **Minecraft: Java Edition client in Rust**, built to match the Java client's behavior exactly: mechanics, physics, movement, timing, camera, animation, textures, UI and sound.

`voxel-client` is a working title. This project is **not affiliated with, endorsed by, or connected to Mojang or Microsoft**. "Minecraft" is a trademark of Mojang Synergies AB.

## Why

The goal is parity you cannot feel, not a "better" client. Anything that merely feels nicer, plus Bedrock behavior, is out of scope. The server is not part of this project; the long-term compatibility target is [Pumpkin](https://github.com/Pumpkin-Mc/Pumpkin), a Rust server.

Parity is checked mechanically instead of by playing the game. For every behavior slice the Java reference is read line by line, run headless as an *oracle*, and its output is captured. The Rust implementation must then reproduce that capture bit for bit on the same input. Expected values are never derived from the Rust output.

## Status

**Pre-alpha, not a playable client.** The pinned reference is Java Edition **26.2**, protocol **776**, Java **25**.

The `play` mode connects to a server, receives chunks and entities, and lets you walk around the world with rendered terrain, many entity models, several screens (pause, options, inventories) and sound. It does not yet have, among other things, a main menu or server list, particles, online-mode authentication, resource packs, or complete block-entity and screen-overlay rendering. Do not expect to play survival with it.

Two kinds of evidence are kept apart on purpose:

- `client-parity`: the Java client and the Rust code were compared on the same input.
- `server-conformance`: a live server accepted the packets Rust produced. This shows wire compatibility only, not client parity.

Work is tracked as small tasks in [`docs/tasks/`](docs/tasks/README.md); the [roadmap](docs/roadmap.md) and [remaining test plan](docs/remaining-test-plan.md) hold the long-term scope. There is no screenshot in this repository on purpose: every frame contains Mojang's assets, which are never committed.

## Assets and legal

The original game's code and assets are **not included, not downloaded by an installer, and never committed**. They are used locally as test and reference input only, under `.data/`, which is ignored by Git. In the future the client will read an asset folder that you select yourself. Continuous integration never receives the reference or any derived output.

## Build and run

Requirements:

- Rust (the exact toolchain is pinned in `rust-toolchain.toml`), CMake and a C/C++ compiler (GLFW and shaderc are built from source).
- A Vulkan driver (MoltenVK on macOS).
- Python 3.9+ and Java 25 for the local tooling and the reference server.

```sh
cargo build --workspace
python3 tools/voxel.py setup                      # fetch the pinned game into .data/ from Mojang's servers
python3 tools/voxel.py server --accept-eula --background
python3 tools/voxel.py play --user Walker
```

`setup` downloads the game from Mojang's official distribution and checks every file against the published hash. You are responsible for accepting the Minecraft EULA. Details are in the [local setup guide](docs/en/local-setup.md).

## Tests

```sh
cargo test --workspace --lib --bins --tests   # unit tests, no game files needed
python3 tools/voxel.py check                  # the full commit gate: fmt, build, test, clippy, parity, docs
python3 tools/voxel.py parity --family gui    # Java-vs-Rust comparison for one family
```

The full gate and the parity runs need the local `.data/` reference. See [behavior testing](docs/en/behavior-testing.md) and the [behavior runner](docs/en/behavior-runner.md).

## Documentation

The core documents (coding standards, behavior testing, behavior runner, local setup) have English translations under [`docs/en/`](docs/en/); the Turkish originals are authoritative. The slice records and the task files are in Turkish.

- [Document index](docs/index.md), with per-slice contract and parity records under [`docs/records/`](docs/records/).
- [Coding standards](docs/en/coding-standards.md): crate layers, layout, file size and complexity limits.
- [`AGENTS.md`](AGENTS.md): the single rule set for every AI coding agent working here.
- [Önceki Türkçe README](docs/tr/README.md).

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. This covers this repository's own code only; it grants no rights to Mojang's code or assets.
