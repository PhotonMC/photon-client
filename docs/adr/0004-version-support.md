# ADR 0004: Multi-version support through translation

- Status: accepted (2026-10-10); version packs, the extractor and protocol translation are replaced by [ADR 0013](0013-multi-version-content-model.md)

## Context

Players want to join servers of other Minecraft versions without "Unsupported protocol version" errors. Version-specific code is spread through the client today:

- `PROTOCOL_VERSION = 776` is hard-coded.
- Packet ids are hand-written (`photon-net/src/connection/protocol/ids.rs`).
- Registry tables in `photon-data/src/generated/` exist for one version.
- Some code compares raw registry numbers.

## Decision

- **One branch.** Every supported version lives on `main`; there are no per-version branches, because fixes would have to be cherry-picked across all of them.
- **Canonical model.** The client lives in the newest supported version. Its internal model is that version's meaning.
- **Version packs.** Each supported version has a data pack: packet ids, registries, block states and other tables. A data extractor run against that version's game jar produces the pack, and `cargo xtask gen <version>` generates code from it. Hand-written tables are replaced, and the planned hand-writing of generated tables (D02-D07) is cancelled.
- **Protocol translation.** Older (and newer) protocols are translated to and from canonical packets in `photon-protocol::translate`, the approach ViaVersion and PommeMC take.
- **Behaviour differences.** Movement constants, attack cooldown, sprint rules and break formula variants live in one `VersionProfile`. Scattered `if version < X` checks are not allowed (enforced by the lint).
- **Selection at connect time.** The protocol number from the status ping picks the version pack, the translator and the profile.
- **Registry ids.** Code resolves ids by name through the version's tables; comparing an id with a number literal is a lint error.

## Consequences

- The newest version is held to full `client-parity`. Older versions are held to parity of their difference table: each `VersionProfile` row is checked against that version's oracle.
- The lowest supported version is still open. The design does not assume a floor. Pre-1.14.4 versions need community mappings, since Mojang's official mappings start at 1.14.4.
