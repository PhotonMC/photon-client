<div align="center">

<img width="993" height="314" alt="Photon" src="https://github.com/user-attachments/assets/88d45077-7f50-481b-967c-c54290ea7c13" />

# Photon

**A from-scratch Minecraft: Java Edition client in Rust: the same game, on a faster engine.**

[![CI](https://github.com/PhotonMC/photon-client/actions/workflows/ci.yml/badge.svg)](https://github.com/PhotonMC/photon-client/actions/workflows/ci.yml)
![Status](https://img.shields.io/badge/status-pre--alpha-orange)
![Java Edition](https://img.shields.io/badge/Java%20Edition-26.2%20(protocol%20776)-3c8527)
![Rust](https://img.shields.io/badge/rust-1.97-b7410e?logo=rust)
![Vulkan](https://img.shields.io/badge/renderer-Vulkan-a41e22?logo=vulkan)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue)](#license)

[Progress](#progress) · [Priorities](#priorities) · [Build and run](#build-and-run) · [How parity works](#how-parity-works) · [Docs](docs/README.md)

</div>

---

Photon reimplements the Java client's behaviour exactly: packets, physics, timing, camera, animation, UI and sound. Every behaviour is proven against the real Java client by a headless oracle, never by eye. Internally it is a new engine: Vulkan rendering, multi-threaded meshing and a Rust codebase with a strict quality gate.

Photon is one client. Out of the box it behaves as the Java client does; planned extras (mods, joining servers of other versions, social features) are opt-in and change nothing when off ([ADR 0010](docs/adr/0010-one-product.md)).

> Photon is **not affiliated with, endorsed by, or connected to Mojang or Microsoft**. "Minecraft" is a trademark of Mojang Synergies AB.

## At a glance

| | |
| --- | --- |
| **Reference** | Java Edition 26.2, protocol 776 |
| **Parity cases** | Compared against the Java client before every push (pre-push hook); the count is in [Progress](#progress) |
| **Code** | ~375k lines of Rust in 13 crates |
| **Started** | 2026-09-07 |

## What works today

- **Connection.** Handshake, login, configuration and play; the configuration phase entered again, transfers and cookies; online-mode encryption and session join; the title screen, server list with pings and icons, and the disconnected screen.
- **World.** Paletted chunks, a client light engine, smooth lighting and ambient occlusion, sky, clouds, fog, weather, fluids, translucent sorting, animated textures and mipmaps, meshed on worker threads with occlusion culling.
- **Movement.** The whole local player tick, compared tick by tick: collision, step-up, fluids, slime, honey, ladders, powder snow, flight, elytra and rockets, riding, sleeping and pistons.
- **Entities.** Models, animations and layers for the entity types, armor, name tags with teams and scores, glow outlines, fire, shadows and invisible states.
- **Blocks and effects.** Block entities (chests, signs, banners, beacons, portals, spawners, statues and more), block and level events, particles and their ambient effects.
- **Items.** Item models with their per-stack properties, enchantment glint, and use predictions (food, bow, crossbow, spyglass and more).
- **Screens.** HUD, chat, every container and inventory with click prediction, recipe book, advancements, statistics, the F3 debug overlay, options screens, toasts, tutorial, dialogs, and the command, structure and jigsaw block screens.
- **Sound.** Positioned sound, music, subtitles, block, entity and UI sounds.

Not there yet, in the order of the first release ([R1](https://github.com/PhotonMC/photon-client/milestone/16)): Microsoft account sign-in inside the client, secure chat, downloading the game's assets after sign-in, first run and updates inside the client, and packaged releases. Version 1.0 is multiplayer-only: Singleplayer and Realms are out of scope for it and stay inactive on the title screen ([ADR 0014](docs/adr/0014-multiplayer-only-first-release.md)). Later: other protocol versions, mods, fallback fonts (unifont, bidi, IME) and the leftovers in the [parity milestone](https://github.com/PhotonMC/photon-client/milestone/15). Each one has an issue.

## Progress

The table is computed by `cargo xtask progress` and checked in CI. Counts of things in the reference are *named by the client*: the client sources spell the identifier. That is a lower bound for what exists, never proof that it behaves right; parity proves behaviour, and every parity case must pass before a push.

<!-- progress:start -->
| Measure | Now |
| --- | --- |
| Clientbound play packets handled | 121 / 141 |
| Serverbound play packets sent | 58 / 69 |
| Screens named by the client | 70 / 118 |
| Entity types with a renderer, named by the client | 156 / 156 |
| Block entity types with a renderer, named by the client | 24 / 26 |
| Particle types named by the client | 120 / 125 |
| Parity cases | 4,776 in 18 families |
| Lint baseline entries | 966 (at introduction: 1,234) |
<!-- progress:end -->

Features finished before the move to GitHub issues (2026-10-10) are summarised in [history](docs/history.md).

### Where the work stands

- **Done:** the foundation (connection, world, movement, rendering, block actions, items, inventories), containers opened by the server, entities, screens and settings, sound, particles, effects and block entities, and server compatibility (milestones M4, M6, M7, M8 and M9).
- **Now:** the first release, [R1](https://github.com/PhotonMC/photon-client/milestone/16): sign-in, secure chat, asset download, first run and updates, packaging, and the server resource pack flow. The order of work is in [#171](https://github.com/PhotonMC/photon-client/issues/171).
- **In parallel:** the architecture line that makes more versions and mods possible: a pure protocol crate, named schedule sets, a version skeleton, and entities as components ([ADR 0003](docs/adr/0003-bevy-ecs.md), [ADR 0004](docs/adr/0004-version-support.md)).
- **Later:** performance work, the mod platform ([ADR 0006](docs/adr/0006-mod-platform.md)) and the [post-1.0 milestone](https://github.com/PhotonMC/photon-client/milestone/17).

### Issue counts

Closed and total issues per milestone on GitHub; these are issue counts, not completion.

<!-- milestones:start -->
| Milestone | Closed / issues |
| --- | --- |
| [Infra: repository and quality](https://github.com/PhotonMC/photon-client/milestone/1) | 6 / 8 |
| [Infra: parity v2](https://github.com/PhotonMC/photon-client/milestone/2) | 7 / 8 |
| [Infra: versions](https://github.com/PhotonMC/photon-client/milestone/3) | 6 / 8 |
| [Infra: ECS and threading](https://github.com/PhotonMC/photon-client/milestone/4) | 3 / 7 |
| [Infra: rendering](https://github.com/PhotonMC/photon-client/milestone/5) | 0 / 5 |
| [Infra: mod platform](https://github.com/PhotonMC/photon-client/milestone/6) | 0 / 2 |
| [Infra: launcher and accounts](https://github.com/PhotonMC/photon-client/milestone/7) | 1 / 1 |
| [M4 Entities](https://github.com/PhotonMC/photon-client/milestone/8) | 1 / 1 |
| [M5 Text and chat](https://github.com/PhotonMC/photon-client/milestone/9) | 1 / 2 |
| [M6 Screens and settings](https://github.com/PhotonMC/photon-client/milestone/10) | 8 / 8 |
| [M7 Sound](https://github.com/PhotonMC/photon-client/milestone/11) | 2 / 2 |
| [M8 Particles, effects and block entities](https://github.com/PhotonMC/photon-client/milestone/12) | 6 / 6 |
| [M9 Server compatibility](https://github.com/PhotonMC/photon-client/milestone/13) | 5 / 5 |
| [M10 Parity closure and distribution](https://github.com/PhotonMC/photon-client/milestone/14) | 0 / 1 |
| [Parity leftovers](https://github.com/PhotonMC/photon-client/milestone/15) | 7 / 26 |
| [R1 First release](https://github.com/PhotonMC/photon-client/milestone/16) | 1 / 7 |
| [Later (post-1.0)](https://github.com/PhotonMC/photon-client/milestone/17) | 0 / 6 |
<!-- milestones:end -->

## Priorities

Work is picked by the roadmap order in [#171](https://github.com/PhotonMC/photon-client/issues/171), then by priority label:

- **[P0](https://github.com/PhotonMC/photon-client/issues?q=is%3Aissue+is%3Aopen+label%3AP0): playable on real servers.**
- **[P1](https://github.com/PhotonMC/photon-client/issues?q=is%3Aissue+is%3Aopen+label%3AP1): visible bugs, and the infrastructure every new feature is written against.**
- **[P2](https://github.com/PhotonMC/photon-client/issues?q=is%3Aissue+is%3Aopen+label%3AP2): parity leftovers and feature completion.**
- **[P3](https://github.com/PhotonMC/photon-client/issues?q=is%3Aissue+is%3Aopen+label%3AP3): additive work, deferred.** Rendering performance (about 500 fps today in a typical scene on an RX 9060 XT at 1280x720), the mod platform, other versions.

## Build and run

You need:

- Rust (pinned in `rust-toolchain.toml`), CMake and a C/C++ compiler;
- a Vulkan driver (MoltenVK on macOS);
- Java 25 for the reference tooling.

```sh
cargo xtask install-hooks
cargo xtask setup                         # fetch the pinned game into .data/
cargo xtask server --accept-eula --background
cargo xtask play --user Walker            # title screen; Multiplayer lists servers
cargo xtask play --user Walker --direct   # join the local test server at once
```

Details are in [development setup](docs/development/setup.md).

## How parity works

1. Read the Java source of the behaviour.
2. Write original scenarios for it.
3. Run them through the Java client itself, headless (the oracle), and record what it does.
4. Implement the Rust side and compare against the recording, case by case.

Expected values never come from Rust output. The rules are in [parity](docs/parity/README.md).

```sh
cargo xtask lint      # repository rules, ~200 ms
cargo xtask check     # fmt, lint, clippy, tests, cargo-deny, cargo-machete, parity
cargo xtask parity    # Java vs Rust comparison (needs .data/)
```

## Architecture

```mermaid
flowchart LR
  core[photon-core] --> data[photon-data] --> world[photon-world] --> game[photon-game]
  game --> protocol[photon-protocol] --> net[photon-net] --> app[photon]
  audio[photon-audio] --> game
  world --> render[photon-render]
  game --> gui[photon-gui]
  render --> gui --> app
```

| Crate | What it holds |
| --- | --- |
| `photon-core` | Numeric, geometric and wire primitives ported from the reference |
| `photon-data` | Tables generated from the reference and checked on every parity run |
| `photon-world` | The client's copy of the world: blocks, light, entities and the local player |
| `photon-game` | Items, menus and the player's actions on the world copy |
| `photon-protocol` | The protocol's wire format: phases, framing and packet codecs |
| `photon-net` | Connections and the live play session |
| `photon-render` | Window, Vulkan backend, meshing and models |
| `photon-gui` | Screens, the HUD, chat and widgets |
| `photon-audio` | Sound decoding, mixing and output |
| `photon` | The application: session loop, screens and scripted runs |
| `photon-oracle`, `photon-dev` | Parity adapter and live-server probes (development only) |

More in the [architecture overview](docs/architecture/overview.md) and the [decision records](docs/README.md).

## Assets and legal

This repository contains no Mojang code or assets and never will. During development the game is fetched from Mojang's official servers into the git-ignored `.data/` folder and used as a local reference only. Nothing from it is committed, packaged or uploaded. Released builds will download assets from Mojang after the player signs in and ownership is verified ([ADR 0007](docs/adr/0007-assets-and-accounts.md)).

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md). Open work is in [GitHub issues](https://github.com/PhotonMC/photon-client/issues).

## License

Licensed under either the [Apache License 2.0](LICENSE-APACHE) or the [MIT license](LICENSE-MIT), at your option. This covers this repository's own code only and grants no rights to Mojang's code or assets.
