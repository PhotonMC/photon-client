# Architecture overview

This page describes the code as it is and where it is going. Decisions behind the target state are in the [ADRs](../adr/).

## Crates

All crates live under `crates/`, plus the `xtask` automation crate. Dependencies flow one way; the allowed edges are in [`xtask/lint.toml`](../../xtask/lint.toml) and `cargo xtask lint` rejects any other.

| Crate | Responsibility |
| --- | --- |
| `photon-core` | Java-semantics basics: maths (fdlibm ports, Java float rules), AABB and voxel shapes, wire/binary/NBT/JSON/UTF-8 codecs, identifiers, randomness, timer |
| `photon-data` | Tables generated from the reference, plus typed lookups over them |
| `photon-world` | The client's copy of the world: chunks, block states, light, entities, the local player, movement and particles |
| `photon-game` | Game rules: items and components, menus and inventories, tooltips, options, sound selection, text |
| `photon-net` | Protocol, packet codecs, connection and the play session |
| `photon-render` | Vulkan backend, models, atlases and meshes, sky and weather, GUI drawing |
| `photon-audio` | Ogg/Vorbis decoding, mixer, `cpal` output |
| `photon` | The application: menu and play loops, input, wiring |
| `photon-oracle` | The Rust side of the parity comparison (JSONL adapter) |
| `xtask` | Lint rules, git hooks, commit gate, parity entry point |

```
core -> data -> world -> game -> net ------------> photon
                    \            \-> render -----^
                     \-> audio ------------------^
```

### Known layering debts

- `photon-render` depends on `photon-game`, because GUI screen logic lives in the render crate. Target: a `photon-gui` crate for screen logic and layout, with the renderer only drawing the resulting draw list.
- `photon-audio` depends on `photon-game` for the sound-engine `Channel`/`Pool` types, which belong in the audio crate.
- Live-server probe code (`photon-net/src/connection/probe/`, `crates/photon/examples/`) ships inside library and application crates. Target: a `photon-dev` crate.
- The protocol layer mixes pure codecs with connection IO. Target: `photon-protocol` (pure codecs and version tables) under `photon-net` (IO and session).

These edges are recorded in the lint baseline and must disappear, not grow.

## Threads

| Work | Thread today |
| --- | --- |
| Socket reads, framing, chunk and light packet parsing | One reader thread per session |
| Packet handling, socket writes | Main thread |
| Section meshing | `rayon` pool (`available_parallelism - 2`, 1..8 workers), nearest first within a 1 ms copy budget |
| Occlusion graph flood | Dedicated thread |
| Startup baking (models, atlases, block shapes) | Global `rayon` pool |
| Audio mixing | `cpal` callback thread |
| Light engine, entity ticks, particle and entity vertex generation, GUI, recording Vulkan commands | Main thread |

Target ([ADR 0003](../adr/0003-bevy-ecs.md)):

- Systems run in parallel under the ECS scheduler.
- The light engine works per section on worker threads.
- Entity and particle vertex generation is parallel.
- Socket writes leave the main thread.
- Rendering runs one frame behind simulation on its own thread, fed by an extract stage.

## Main loop

`crates/photon/src/play/schedule.rs` runs the `Frame` schedule (single-threaded, systems chained in this order), and its `run_ticks` system runs the `Tick` schedule once per owed tick. Per frame:

1. Poll window events.
2. Fit the swapchain.
3. Drain packets.
4. Route sounds and particles.
5. Run fixed 50 ms ticks from an accumulator.
6. Draw with the partial tick for interpolation.
7. Apply the frame limiter.

State is held in large structs today:

- `Play` (`crates/photon/src/play.rs`)
- `PlaySession` (`crates/photon-net/src/connection/play.rs`)
- `ClientWorld` (`crates/photon-world/src/lib.rs`)
- `ClientEntity` with its `EntityKind` (`crates/photon-world/src/entity.rs`)

## Versions

Protocol 776 (Java Edition 26.2) is the only version today. The packet ids and generated tables are version-specific. The path to more versions is in [ADR 0004](../adr/0004-version-support.md).
