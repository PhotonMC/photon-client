# ADR 0010: One product

- Status: accepted (2026-10-22); supersedes [ADR 0008](0008-vanilla-and-enhanced.md)

## Context

[ADR 0008](0008-vanilla-and-enhanced.md) planned two products, Photon Vanilla and Photon Enhanced, as two binaries on a shared core, with the Enhanced-only crates never compiled into Vanilla. Nothing of it was built: there is one binary crate (`photon`), and the split would add a second binary, a second release, a layer rule and a choice the player has to make before anything else, all for features that do not exist yet.

## Decision

- Photon is one client and one binary. There is no Vanilla or Enhanced edition.
- Out of the box it behaves exactly as the Java client does; parity stays the goal for everything the Java client has.
- Features beyond the Java client (mods, [ADR 0006](0006-mod-platform.md); servers of other versions, [ADR 0004](0004-version-support.md); social features and their backend) stay on the roadmap as parts of the same client. Each is off until the player or the server turns it on, and when it is off nothing observable differs from the Java client.
- Such features still live in their own crates behind extension points of the core (the event bus, the mod host interface, UI hooks), so the core does not depend on them.

## Consequences

- The planned split into two binaries on a `photon-app` core is dropped.
- A feature beyond the Java client must ship with a way to turn it off, and the parity families run with it off.
