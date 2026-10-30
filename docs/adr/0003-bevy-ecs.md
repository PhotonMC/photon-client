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

## Addendum: rendering on its own thread (2026-10-30)

Step 5 of the decision is taken in two parts. What exists now is the first part; the second moves to [#25](https://github.com/PhotonMC/photon-client/issues/25). The performance program is tracked on [#98](https://github.com/PhotonMC/photon-client/issues/98), the slices in #22.

**What was built.**

- **The backend sits in a `Handoff`.** `VulkanBackend` is `Send`, and the client keeps it in a `Handoff<VulkanBackend>` (`crates/photon-render/src/vulkan/handoff.rs`). `submit` gives a job to a render thread, which runs it on the backend while the simulation thread goes on. Every other access (`Deref`, `DerefMut`, `finish`, drop) first waits until the job in flight is done, so no call site needs to know that a thread exists, and the backend is never seen half way through a frame.
- **Only the end of the frame moves.** Recording, submitting and presenting the scene (`draw_scene`) run on the render thread. Everything that builds the frame (overlays, sections, sky, GUI, the uploads) still runs on the simulation thread, and so does everything that changes game state: packets, the sound listener, the out-of-view set, forgotten chunks, skins.
- **Errors.** An error of a job is kept and returned by the next frame's submit, a frame late. A panic on the render thread is raised again on the first access that waits for it.
- **The setting.** `render.pipelined` in `photon.toml` of the game directory, on unless the file says `false`; there is no settings screen. With `false` the same job runs inline on the calling thread, which is the serial reference. The file reads a small subset of TOML written for the purpose; no TOML crate is added.
- **Captures.** A screenshot, a finished script and `cargo xtask shot` read the backend through `Deref`, so they wait for the frame in flight and show the frame that was asked for.
- **Measuring.** With two threads the main thread's work time falls whether or not frames get faster, so a pipelined result is judged on the present-to-present interval (`interval_ms`), with the render thread's job time and the time the main thread spent waiting for it beside it (`render_thread`) and the GPU clock range of the run (`gpu_clock_mhz`). The bench records a frame once its backend timings are final, and takes the presentation time from the render thread. Input-to-present latency keeps its definition (input poll to present return), and it is unchanged: the render thread starts the frame as soon as it is submitted, so no frame sits between simulation and recording.

**What it gives.** The render thread's job is about 0.2 ms of a frame of about 1.1 ms, and the main thread waits for almost all of it, because it touches the backend again at the top of the next frame's overlays. The measured gain is a few percent of the interval, from runs that alternate the two modes so that the GPU's changing clock reaches both alike (still route 1.07-1.08 against 1.13-1.14 ms in three pairs, about 5 %; pig route 2.40 against 2.45 ms). The render job takes 0.19-0.20 ms per frame and the main thread waits 0.18-0.19 ms of it, which is the most that moving the first backend touch later could win back. The overlap is only the next tick and the work before the first backend call.

**What moved to #25.** Moving more of the frame off the simulation thread needs every overlay pass to finish building its vertices before the frame's first backend call, so that the render thread can run while the next frame is built, and the per-entity extract (transforms and appearance as plain data, in network id order, with no shared random state in parallel tasks) that instancing needs anyway. Doing that restructuring here and again in #25 would be the same work twice. Two things the first plan named are therefore not built: vertex building on the render thread, and a late camera that applies the newest mouse look just before recording. Latency shows no frame to win back today; the late camera is revisited when a design puts a frame between simulation and recording. The test that serial and pipelined draw the same frame is a pixel comparison of pinned screenshots (0.05 to 0.12 % of the pixels differ between two runs of either mode, and the same share between the two modes; a blue particle was seen in one of the differences, the others were not traced), not a draw-list test.

