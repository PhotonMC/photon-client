# Photon

<img width="993" height="314" alt="photon-splash-1920x1080(1)" src="https://github.com/user-attachments/assets/88d45077-7f50-481b-967c-c54290ea7c13" />

A from-scratch **Minecraft: Java Edition client in Rust**. It matches the Java client's behaviour exactly (mechanics, physics, timing, camera, animation, UI and sound) and is built to be faster.

Photon is **not affiliated with, endorsed by, or connected to Mojang or Microsoft**. "Minecraft" is a trademark of Mojang Synergies AB.

## Two products

- **Photon Vanilla.** The Java client's behaviour, bit for bit, on a faster engine.
- **Photon Enhanced.** Vanilla plus:
  - mods (data packs, Luau scripts and WebAssembly);
  - its own UI, cosmetics and a community server list;
  - friends, invites and spectating;
  - joining servers of other versions without "Unsupported protocol version" errors.

Both share one engine; see [ADR 0008](docs/adr/0008-vanilla-and-enhanced.md).

## Status

**Pre-alpha.** The reference is Java Edition **26.2** (protocol **776**). Photon connects to a server, renders the world, most entities, block entities, particles and screens, plays sound, and lets you move, ride, build and use containers. Online-mode login, resource pack downloads, the debug overlay, advancements and statistics are still missing.

Every behaviour is proven against the real Java client by a headless oracle, never by eye ([parity](docs/parity/README.md)).

## Roadmap

Open work is tracked in [GitHub issues and milestones](https://github.com/PhotonMC/photon-client/milestones).

| Milestone | State |
| --- | --- |
| Foundation: connection, world copy, movement, world rendering, block actions, items, inventories | Done |
| M3.d Containers opened by the server | Done |
| M4 Entities | Done; coverage audit open |
| M5 Text and chat | Font fallbacks, bidi, IME, text selection and chat details open |
| M6 Screens and settings | Debug overlay, advancements, statistics and the remaining settings open |
| M7 Sound | Event and item sounds, subtitles open |
| M8 Particles, effects and block entities | Beacon/portal block entities, spawners, remaining level events open |
| M9 Server compatibility | Resource pack download, dialog inputs, online login open |
| M10 Parity closure and distribution | Not started |

Infrastructure phase (feature work paused until it lands):

| Track | State | Decision |
| --- | --- | --- |
| Deterministic quality gate (`cargo xtask`) | Done | [Coding standards](docs/development/coding-standards.md) |
| English repository, `main`, issues | In progress | |
| Parity v2: Rust runner, per-case cache, affected-only, parallel | Planned | [ADR 0005](docs/adr/0005-parity-v2.md) |
| Version packs and protocol translation | Planned | [ADR 0004](docs/adr/0004-version-support.md) |
| ECS world (`bevy_ecs`) and multi-threaded simulation | Planned | [ADR 0003](docs/adr/0003-bevy-ecs.md) |
| GPU-driven terrain rendering | Planned | [Rendering](docs/architecture/rendering.md) |
| Mod platform | Planned | [ADR 0006](docs/adr/0006-mod-platform.md) |
| In-client launcher, Microsoft login, asset download | Planned | [ADR 0007](docs/adr/0007-assets-and-accounts.md) |

What has been built so far is summarised in [history](docs/history.md).

## Assets and legal

This repository contains no Mojang code or assets and never will. During development the game is fetched from Mojang's official servers into the git-ignored `.data/` folder and used as a local reference only. Nothing from it is committed, packaged or uploaded. Released builds will download assets from Mojang after the player signs in and ownership is verified ([ADR 0007](docs/adr/0007-assets-and-accounts.md)).

## Build and run

You need:

- Rust (pinned in `rust-toolchain.toml`), CMake and a C/C++ compiler;
- a Vulkan driver (MoltenVK on macOS);
- Python 3.9+ and Java 25 for the reference tooling.

```sh
cargo xtask install-hooks
cargo build --workspace
cargo xtask setup                       # fetch the pinned game into .data/
cargo xtask server --accept-eula --background
cargo xtask play --user Walker
```

Details are in [development setup](docs/development/setup.md).

## Checks

```sh
cargo xtask lint      # repository rules, ~150 ms
cargo xtask check     # fmt, lint, clippy, tests, cargo-deny, cargo-machete, parity
cargo xtask parity    # Java vs Rust comparison (needs .data/)
```

## Documentation

See [docs/](docs/README.md): architecture, decisions, parity, development and history.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).

## License

Licensed under either the [Apache License 2.0](LICENSE-APACHE) or the [MIT license](LICENSE-MIT), at your option. This covers this repository's own code only and grants no rights to Mojang's code or assets.
