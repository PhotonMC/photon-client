# ADR 0003: `bevy_ecs` for world and entity state

- Status: accepted (2026-10-10), step 1 in progress: the play loop runs as `bevy_ecs` schedules (`crates/photon/src/play/schedule.rs`); its state is still one `Play` held as a non-send resource

## Context

World state lives in large structs, all mutated on the main thread:

- `ClientWorld` has about 40 fields, with chunks in a `BTreeMap`.
- `ClientEntity` has 77 fields plus an `EntityKind` enum.
- `Play` has about 50 fields.

This blocks three goals:

- **Parallel simulation.** Systems that touch disjoint data cannot run on separate cores without hand-written locking.
- **Modding.** Mods need to query and extend state, and to declare which data they touch so that hundreds of them can run in parallel and be unloaded cleanly ([ADR 0006](0006-mod-platform.md)).
- **Engine separation.** Simulation, extraction and rendering should be distinct stages, so rendering can run on its own thread.

## Decision

Adopt `bevy_ecs` as a standalone library, without the Bevy renderer, app framework or plugins. Migrate step by step, with parity green after every step:

1. A fixed 20 TPS tick `Schedule` and a per-frame `Schedule`.
2. `ClientEntity` split into components (`Transform`, `Interpolation`, `LivingState`, `Equipment`, ...); per-type state from `EntityKind` becomes per-type components; a `NetworkId` component maps protocol ids to entities.
3. Chunks as a resource with dense indexing instead of `BTreeMap`.
4. Decoded packets delivered as events that systems consume.
5. An extract stage that copies render state, so rendering can run one frame behind simulation on its own thread.

A custom ECS was rejected: the scheduler, change detection and query machinery are years of work that `bevy_ecs` already provides. Keeping plain structs was rejected because it pushes locking and access control into every mod and system.

## Consequences

- System access declarations become the unit of parallelism for both engine and mod code.
- The order of systems that model one Java method must be explicit (`chain`/ordering constraints), since Java runs them sequentially.
- Parity tests drive systems the same way they drive today's functions; observable output stays the contract.
