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
