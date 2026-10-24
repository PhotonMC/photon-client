# ADR 0011: Resource packs and the asset stack

- Status: accepted (2026-10-23)

## Context

Server resource packs ([#51](https://github.com/PhotonMC/photon-client/issues/51)) put a downloaded zip on top of the game's own assets and reload everything that was read from them. Until now every consumer took one `assets_root: &Path` and read files from it directly, so there was nothing a second pack could be layered over.

## Decision

- **A crate for the stack.** `photon-assets` sits directly above `photon-core` and holds the pack sources, the reading of `pack.mcmeta` and the ordered stack. Language, text, atlas, font and sound code read through it instead of through a root path; that migration goes one consumer per commit.
- **Behaviour of the reference's managers.** The topmost pack that has a file wins (`getResource`). `stack` returns every version, lowest first. `list` returns the topmost version of each file, sorted by path and then namespace. A pack's `filter.block` hides files of the packs below it, but never its own. The namespace of a filter pattern and the path of another one are tested separately, as the reference does. Overlays that name the running pack format are searched before the pack's own files, the last listed first.
- **Sources.** A pack is a folder or a zip archive behind a small `Source` trait, so the game's own assets (the client jar's folder and the indexed objects) can be layered as sources too.
- **Dependencies.** `zip` (with only the `flate2` backend, which the workspace already uses) reads the packs. `regex` evaluates the `filter` patterns; the reference's patterns are Java regular expressions and the common subset is the same in both engines.
- **Server pack order.** The packs a server names go on top of the stack in the order the server lists them, the last one topmost.

- **Sprite sources.** The atlas definitions' `filter` source uses the same `IdentifierPattern`, so `photon-assets` exports it and `photon-render` (and the oracle binary) may depend on `photon-assets`. The crate depends on `photon-core` alone, so no cycle appears.

## Consequences

- The client now links a zip reader, which [ADR 0009](0009-xtask-downloads.md) kept out of it.
- Syntax that Java's regular expressions have and `regex` lacks (look-around, back-references) makes a filter pattern invalid, and the pack is read as if it had no filter.
- The stack is tested against packs built in test code. It is not yet compared with the Java managers through the oracle.
