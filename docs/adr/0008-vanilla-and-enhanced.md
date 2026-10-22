# ADR 0008: Photon Vanilla and Photon Enhanced

- Status: superseded by [ADR 0010](0010-one-product.md) (2026-10-22); never implemented

## Context

Two products are planned:

- **Photon Vanilla.** The Java client's exact behaviour, only faster.
- **Photon Enhanced.** Everything Vanilla has, plus:
  - its own UI and mod support;
  - cosmetics and player-made skins, maps and modpacks;
  - a community server list;
  - seeing other Photon players, friends, invites, spectating and joining;
  - automatic protocol selection per server.

## Decision

- One repository and two binary crates, `photon-vanilla` and `photon-enhanced`, on a shared application core (`photon-app`).
- Enhanced-only code lives in its own crates (`photon-mods`, `photon-social`, `photon-ui-plus`). The Vanilla binary never compiles them, and the lint's layer table enforces it.
- Online services (friends, invites, cosmetics, community server list) live in a separate backend repository, designed in its own ADR.
- Automatic protocol selection ([ADR 0004](0004-version-support.md)) is part of the shared core, so both products can join servers of any supported version.

## Consequences

- Shared code must not assume Enhanced features. Extension points (the event bus, the mod host interface, UI hooks) are defined in the core and implemented only in the Enhanced crates.
