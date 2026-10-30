# ADR 0003: `bevy_ecs` for world and entity state

- Status: accepted (2026-10-10), step 1 in progress: the play loop runs as `bevy_ecs` schedules (`crates/photon/src/play/schedule.rs`); its state is still one `Play` held as a non-send resource. Step 2 has begun: `ClientWorld` owns a `bevy_ecs::World` in which every entity is an ECS entity holding a `ClientEntity` component, with an ordered index by network id (id order is tick order, which parity observes, so nothing walks entities with a query where order matters). This inner World is meant to become the simulation World, so the chunk, weather and level resources of step 3 go there too. Step 3 has begun: chunks live in a `Chunks` store that is the Java client's chunk cache itself (a ring of slots, indexed by position) and each chunk holds its sections in a dense array, instead of two ordered maps

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

## Addendum: entity views and their cost (2026-10-30)

Step 2 moved the entity state into components, and reading it through `EntityView` cost more than the plain struct had: a view fetched each component with its own table lookup, and a renderer built one view per entity for every pass over `entities().drawn()` (about forty per entity per frame). With 1591 `NoAI` pigs in view the mean work per frame went from 3.5-4.6 ms before the migration to 14-17 ms.

Two rules follow from the fix, and apply to any later entity code:

- **Views come from stored queries.** `Entities` holds one `QueryState` for reading and one for writing, built once and refreshed when an entity is inserted or removed, so a view is one archetype-table fetch and not a series of `World::get` calls. Code that needs a view must not create its own query state per call.
- **Filter before you fetch.** A set that leaves entities out (out of view, invisible) is applied to the ids first; a view is built only for an entity that is kept.

Measured on the same world (1591 pigs, still route, render distance 20): 14.4-16.8 ms with the per-call lookups, 5.7 ms with the stored queries, 3.1 ms with the ids filtered first. Passes that every entity needs and that do not depend on each other's order (the `Query` systems, `par_iter`, change detection) remain open work in #173; each of them must be measured against this table before it is kept.
