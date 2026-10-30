# ADR 0006: Mod platform

- Status: accepted direction (2026-10-10), design and prototype pending

## Context

Photon will support mods, off until the player or a server turns them on ([ADR 0010](0010-one-product.md)). The goals:

- **Three audiences.** People who cannot code add assets. People who code a little change behaviour and see the result at once. Experienced developers build complex systems.
- **Safety without slowdown.** Mods run sandboxed, with permissions and protection against malicious code, and still perform well with 300 or more mods loaded.
- **Multi-core.** Mods can use many cores rather than one constrained thread.
- **Live toggling.** Mods can be turned off and on while the client runs.
- **Zero setup.** Servers deliver their mods automatically, with a permission prompt and a cache. No mods folder, no loader, no third-party launcher.
- **Choice of language.** Mods can be written in several programming languages.

## Decision

Three tiers on one host:

| Audience | Tier |
| --- | --- |
| No code | Data packs: assets, models, sounds, translations, block/item definitions, GUI layouts (JSON/TOML). A browser editor (Rust compiled to WebAssembly) can produce them later. |
| Some code | Luau scripts (`mlua` with the `luau` feature). One VM per mod, instant hot reload, typed `.d.luau` API definitions. |
| Experienced | WebAssembly components (`wasmtime`) with a WIT-defined API, written in Rust, C/C++, Go, Zig, TypeScript or anything that targets the component model. |

How it works:

- **Scheduling.** Mod systems register with the ECS scheduler ([ADR 0003](0003-bevy-ecs.md)) and declare the components they read and write. Non-conflicting mods run in parallel, and every mod is its own instance or VM, so no global lock exists.
- **Budgets.** `wasmtime`'s pooling allocator and epoch interruption enforce per-mod time and memory budgets.
- **Capabilities.** A mod's manifest declares capabilities (`world.read`, `world.write`, `network.server`, `fs.mod_data`, `ui.overlay`, ...). A host function cannot be imported without its capability, and WASI is limited to what was granted.
- **Signing.** Packages carry an ed25519 signature and a content hash.
- **Unloading.** Every entity, component, system and resource a mod creates is tagged with its owner, so disabling a mod removes all of it and drops the instance.
- **Server delivery.** The Photon fork of Pumpkin sends a manifest of mod ids, hashes and capabilities on a custom channel during configuration. The client fetches missing content into a content-addressed cache, asks for permissions, and loads the mods. The next join only checks hashes.
- **Vanilla servers.** Only client-side mods allowed by the server's policy run.

## Consequences

- The ECS migration and the event bus come before the mod host.
- The prototype must measure the empty-tick cost per mod (target under 1 us) and show that a call without its capability is refused.

## Addendum: first deliverable is a native `photon-api` crate (2026-10-30)

The roadmap review of 2026-10-28 (decision recorded in #26) changed the order of work: before any WebAssembly or Luau host exists, an in-process crate `photon-api` defines what a mod can see and change, and Photon's own opt-in features use it first. The WIT world is generated from that crate afterwards, so the two cannot drift. Luau is deferred until one runtime works.

What the crate holds:

- **Events.** Typed events a feature subscribes to, named after Fabric API where that helps porting (for example a client tick start and end, join and disconnect, a screen opening, a chat message received). An event has a registration order and an owner.
- **Ordering.** The named system sets of the frame and tick schedules (`FrameSet`, `TickSet` in `photon::play::schedule`, today private to that module) become public through the crate, so a feature can place a system before or after a part, or replace one. A test fixes the set order, as it does today.
- **Registries.** Lookups of the game's registries (items, blocks, entity types, sounds) by name, over the wire-id boundary of [ADR 0013](0013-multi-version-content-model.md), so a feature never sees a numeric id of one version.
- **Ownership.** Everything a feature registers (event handlers, systems, registry additions) is tagged with the feature that made it, so turning the feature off removes all of it. This is the unload rule above, applied to native features first.

Not decided here, and to be settled in the first code slice with a measurement or a test: the event list beyond the examples, whether handlers run inside the schedule as systems or from a dispatcher, and the capability names a native feature declares. The empty-tick cost target (under 1 microsecond per feature) applies to native features too and is checked the same way.

## Addendum: capabilities, manifest, data, delivery (2026-10-30)

Settled after the first code slices of `photon-api`. The mod host itself ([#27]) is not started.

- **Capabilities.** Named `area.verb`, mirroring what `photon-api` offers. A mod gets only what its manifest asks for and the user grants. The first set is `events.tick` and `events.frame` (observe only) and `registry.read`. Later, each added together with the API it guards and never ahead of it: `render.hud`, `input.keys`, `chat.read`, `chat.send`, and `net.packets` (the last two only after the packet types of [ADR 0013](0013-multi-version-content-model.md) and secure chat settle). The first version has no file, network or process capability.
- **Manifest.** One `mod.toml` per mod: its id (the owner name used for unloading), its version, the `photon-api` version it was built for, the capabilities it asks for and a hash of its payload. The API version follows the crate's semver; a major difference refuses the mod instead of loading it wrongly.
- **Data.** A mod carries a vanilla-format data pack and resource pack beside its code. The client already reads both formats, so nothing new is specified.
- **WIT world.** Generated from `photon-api` when the first runtime prototype needs it, and not before. Until then there is no WIT file.
- **Signing.** Open. Mods a user installs by hand are not signed in the first version; the key policy for anything delivered depends on the release and self-update keys and waits for the owner.
- **Server delivery.** Accepted in this shape, to be built after 1.0: a server can only suggest a mod by id and hash on a custom payload channel; the client shows the capabilities and asks, loads nothing without a click, and loads nothing in singleplayer or offline play. It follows the multiplayer-only decision of [ADR 0014](0014-multiplayer-only-first-release.md) and the trust model above.

[#27]: https://github.com/PhotonMC/photon-client/issues/27
