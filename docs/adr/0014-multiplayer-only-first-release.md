# ADR 0014: Multiplayer-only first release

- Status: accepted (2026-10-28)

## Context

Photon does not write a server ([AGENTS.md](../../AGENTS.md)). The Java client's Singleplayer button starts an integrated server, and "Open to LAN" publishes it; Realms is a hosted service. A first release that offered either would need a server of ours or a service we do not run.

## Decision

- Photon 1.0 is multiplayer-only. Singleplayer, opening a world to LAN and Realms are out of scope for it.
- The title screen shows the Singleplayer and Realms buttons, but inactive, with a tooltip that says why. The wording is Photon's own (the translation key `photon.title.multiplayer_only`, with an English fallback in the code), not the reference's.
- Joining LAN worlds that other clients open stays as it is: the server list's LAN scan is unchanged.
- The parity cases build the title screen as the reference does, with both buttons active. The inactive state is a separate step applied by the application (`TitleScreen::multiplayer_only`), so the reference behaviour stays tested and the difference stays one visible call.
- An embedded server can be reconsidered after 1.0 in its own ADR.

## Consequences

- The title screen differs from the Java client in these two buttons on purpose; this is the only observable difference of the title screen.
- Nothing is removed: re-enabling the buttons later means dropping the call, not rebuilding the screen.
