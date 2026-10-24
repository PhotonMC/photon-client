# Architecture overview

This page describes the code as it is and where it is going. Decisions behind the target state are in the [ADRs](../adr/).

## Crates

All crates live under `crates/`, plus the `xtask` automation crate. Dependencies flow one way; the allowed edges are in [`xtask/lint.toml`](../../xtask/lint.toml) and `cargo xtask lint` rejects any other.

| Crate | Responsibility |
| --- | --- |
| `photon-core` | Java-semantics basics: maths (fdlibm ports, Java float rules), AABB and voxel shapes, wire/binary/NBT/JSON/UTF-8 codecs, identifiers, randomness, timer |
| `photon-assets` | Resource packs: folder and zip sources, `pack.mcmeta` (overlays, filters) and the ordered stack the assets are read through ([ADR 0011](../adr/0011-resource-packs.md)) |
| `photon-data` | Tables generated from the reference, plus typed lookups over them |
| `photon-world` | The client's copy of the world: chunks, block states, light, entities, the local player, movement and particles |
| `photon-game` | Game rules: items and components, menus and inventories, tooltips, options, sound selection, text |
| `photon-protocol` | The Java protocol's wire format: phases, framing, handshake, login, configuration and the play packet codecs (`Decode`) |
| `photon-net` | Connections over sockets: phases driven, the play session (packet `handle` steps, reader thread), a `Recorder` hook that production leaves empty |
| `photon-dev` | Developer tooling, not shipped: live-server probes that watch the client against a real server |
| `photon-render` | Vulkan backend, models, atlases (their sprite lists come from `atlases/<name>.json` of every pack, `atlas_sources`) and meshes, sky and weather, and what the GUI draws with (`GuiFrame`, font, atlas, item icons); knows nothing of game rules; `GuiFrame` is the boundary the screens draw through (immediate-mode calls, with `DrawOp` as the test log) |
| `photon-gui` | Screens, the HUD, chat, options and widgets: layout and logic over game state, turned into draw calls on `GuiFrame` |
| `photon-audio` | Ogg/Vorbis decoding, mixer, the `Channel`/`Pool` contract the sound engine drives, and (feature `device`) `cpal` output |
| `photon` | The application: menu and play loops, input, wiring |
| `photon-oracle` | The Rust side of the parity comparison (JSONL adapter) |
| `xtask` | Lint rules, git hooks, commit gate, parity entry point |

```
core -> assets
core -> data -> world -> game -> protocol -> net -> photon
audio  -> game     (the channel contract only; the `device` feature stays off)
world  -> render   (render knows no game rules)
game + render -> gui -> photon
```

### Reading what the render side owns

`photon-render` evaluates the dimension's environment attributes (sky, fog, light and the clocks' flags) from the data pack, and `photon-world` may not depend on it. Where the world's own code needs one of them (the particle engine asks whether the creaking is about, and what a biome lets drift), `photon-world` defines a trait (`particles::Ambience`, like `TerrainMaterials` before it), the client builds the implementation (`SkyAmbience`) each tick from the sky data and the session's clocks, and the oracle builds the same one. The evaluation itself stays in `photon-render/src/sky`; a block's own reading of a position goes through the trait, not through a copy of the evaluation.

### Known layering debts

- `photon-dev` holds the probes as they were: one 2,200-line function (`probe/mode.rs`) and 121 examples under `crates/photon-dev/examples/` whose size and naming violations are still in the lint baseline. Target: stages and scenario-driven binaries that meet the limits.

These edges are recorded in the lint baseline and must disappear, not grow.

## Threads

| Work | Thread today |
| --- | --- |
| Socket reads, framing, parsing of every clientbound play packet | One reader thread per session |
| Packet handling (`handle`), socket writes | Main thread |
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
