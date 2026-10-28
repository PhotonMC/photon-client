# ADR 0013: Multi-version content model

- Status: accepted (2026-10-28); replaces the version packs, extractor and translation layer of [ADR 0004](0004-version-support.md)

## Context

[ADR 0004](0004-version-support.md) planned several supported versions through generated data packs, an extractor run against each version's jar and a protocol translation layer. None of it was built. The generated tables in `photon-data/src/generated/` cover one version, and the extractor would be a second tool to keep correct for every release.

The goal is unchanged: one client that joins servers of the newest version and of a few older ones, with the newest version held to full parity.

## Decision

- **One binary, no launcher.** The version is a runtime value, chosen at connect time from the protocol number in the status ping. A per-server manual override in the server list can follow later. There are no cargo features per version.
- **Union content model.** Behaviour defaults to the newest version. Content (blocks, items, entity types, sound events, particle types, menu types and so on) is the union of all supported versions. Content that a newer version removed stays defined with `until(..)`; it is never mapped to a "closest" replacement, because that breaks parity.
- **Code, not data.** Version differences are Rust code. There are no per-version data packs and no extractor. The generated tables are a transitional state and move to hand-written definitions.
- **Two id spaces.** Game code uses canonical ids: the index in the union list, known at compile time, so `named(..)` constants keep working. Wire ids are per version and exist only at the protocol boundary, as separate types (`WireItemId`, `WireBlockStateId` and so on). A per-session `Registries`, built once at connect, translates between the two.
- **Ids are computed, not stored.** A version's wire ids come from walking the union list filtered to that version, the way the block registry already numbers states: registry order, cartesian product of properties, last property fastest. A property added in a newer version carries `since(..)`; older versions map its states to the property's default value.
- **Availability.** `registries.has(x)` filters every list the client builds itself (creative tabs, recipe book, command suggestions), as the reference's feature flag set does.
- **Where differences live.** Each crate has a `versions/` module with one file per older version (for example `photon-world/src/versions/v26_1.rs`). Canonical code never names an older version.
  - Behaviour differences go through a `Rules` trait with canonical defaults and per-version overrides: generic dispatch, chosen once per session, for hot paths; `dyn` for cold ones.
  - Replaced systems are swapped by name in a per-version schedule builder, which needs the named system sets of [#165](https://github.com/PhotonMC/photon-client/issues/165).
- **Lint.** `Version` comparisons appear only in `versions/` modules and in a short list of dispatch points. Wire id types appear only in `photon-protocol` and `photon-net`. Wrapping a raw read into a canonical id outside the translation point (`ItemId(read_varint(..))`) is an error.
- **Support window.** The newest version plus the two before it. Dropping a version deletes its `versions/` modules and its `until` definitions.
- **Second version: 26.1**, chosen to test the design cheaply ([#17](https://github.com/PhotonMC/photon-client/issues/17)).
- **Verification.** Java stays a development-time reference only. Each version gets a registry order case, a definition properties case, and a difference table with a case for every override. As in ADR 0004, older versions are held to parity of their difference table; the newest version is held to full parity.

## Consequences

- The ADR 0004 decisions on version packs, the extractor, `cargo xtask gen <version>` and `photon-protocol::translate` no longer apply. Its one-branch rule, the canonical newest-version model, selection at connect time and name-based id resolution stay.
- The work is split into the version skeleton ([#161](https://github.com/PhotonMC/photon-client/issues/161)), the registry order case ([#162](https://github.com/PhotonMC/photon-client/issues/162)) and retiring the generated tables ([#163](https://github.com/PhotonMC/photon-client/issues/163)). Until they land, protocol 776 is the only version.
- A version costs hand-written definitions and an override per difference. In exchange there is no data format to maintain and canonical code stays free of version checks.
- The lowest supported version is bounded by the window, not by the availability of mappings.
