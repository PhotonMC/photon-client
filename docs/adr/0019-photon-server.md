# ADR 0019: photon-server on an unmodified Pumpkin

- Status: accepted (2026-11-04)

## Context

Photon's server features (the handshake with the Photon client, loading-screen data, mod delivery, the backend check of a joining player) need a server. The server is [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin), which changes daily. A fork that edits Pumpkin's sources would conflict with upstream on every update.

Pumpkin has a WebAssembly plugin host with a versioned WIT interface: `pumpkin-plugin-wit` ships `v0.1` and `v0.2`, and both hosts are compiled in. Seen at upstream `a83abee` (2026-11-03):

- A plugin can send a custom payload to a player (`player.send-custom-payload`) and receives the player's payloads (`player-custom-payload-event`). It also gets login, pre-login and join events, and can transfer a player to another address.
- The custom payload event fires only in the play phase, so a plugin has no channel during configuration.

## Decision

- **Separate repository.** `photon-server` holds Photon's server work. It has its own AGENTS.md. The client repository's "no server" rule does not apply there.
- **Pumpkin stays unmodified.** `photon-server` depends on Pumpkin at a pinned upstream commit and adds features as WebAssembly plugins against the plugin WIT. The main plugin is `photon-bridge`, which handles the Photon handshake, loading-screen data and the backend ticket check.
- **Patches are the exception.** When the WIT lacks something (first known case: a custom channel in the configuration phase, needed to deliver the mod manifest of [ADR 0006](0006-mod-platform.md) before play), we first propose the change upstream. Until upstream accepts it, the change lives as a small, named patch file applied at build time, never as edits on a long-lived fork branch.
- **Following upstream.** A scheduled CI job (daily) moves the pinned commit to upstream's latest commit, builds, and runs the plugin tests plus a join test with the Photon client. If everything passes, it opens a pull request. If something fails, it opens an issue with the failing step.
- **Plugin interface version.** Plugins target the newest WIT version that the pinned Pumpkin supports. Moving to a new WIT version is a normal change with its own pull request.

## Consequences

- Pumpkin is GPL-3.0. Builds and images of `photon-server` that contain Pumpkin are distributed under GPL-3.0. The plugin API crates are MIT/Apache, so `photon-bridge` and other plugins may use their own license.
- Mod delivery before play (ADR 0006) waits on the configuration-phase channel, upstream or as a patch. The first milestone does not need it, because loading-screen data comes from the backend before the connection is opened.
- Vanilla and third-party servers keep working with the client. Photon-only features stay inactive there.
