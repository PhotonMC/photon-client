# ADR 0026: Join tickets for Photon servers

- Status: proposed (2026-11-12); nothing is built, the owner decides

## Context

[ADR 0019](0019-photon-server.md) gives `photon-bridge` a "backend ticket check" and [ADR 0020](0020-accounts-and-launch-links.md) says Photon servers may later trust a backend ticket, but no document says what a ticket is. Today the backend has no signing key and no call a game server could use ([ADR 0022](0022-backend-services.md) fixes five calls, none for servers), the client sends a game server nothing but `photon:hello` ([ADR 0024](0024-server-delivered-mods.md)), and a Pumpkin plugin has no channel before the play phase.

A plugin can reach the network only when the operator grants it `http.outbound`, has no cryptography in its interface (pure Rust compiled into the plugin works), and can kick a player at any time.

## Proposed decision

- **A ticket is a signed statement, checked offline.** The backend signs `{v, sub, name, aud, iat, exp, jti}` with Ed25519: `sub` is the player's UUID, `aud` the server's directory id, `exp` at most 60 seconds after `iat`. The server verifies the signature with the backend's public key from its own configuration and makes no network call, so it needs no credentials and no `http.outbound` grant.
- **The client asks for it just before it connects.** One new backend call, `POST /v1/ticket` with the session and the server id, returns the ticket. Only servers in the directory have an id, so only they can be named.
- **It travels on `photon:ticket` in the play phase,** sent with the first `photon:hello`. The bridge checks signature, `aud`, `exp` and that `sub` and `name` are the joining player's, and remembers each `jti` until it expires so a ticket is used once.
- **The ticket adds to the session join, it does not replace it.** A Photon server keeps online mode; the ticket proves the player came through Photon and is signed in there.
- **The gate.** A server with `require_ticket` kicks a player whose valid ticket has not arrived within the hello timeout. This is the security boundary that `photon_only` is not.
- **Keys.** The backend holds one signing key and publishes the public key and its id; a ticket names the key id, and servers accept the current and the previous key so a key can be rotated.

## Open questions for the owner

- Whether the check may stay after the join (first seconds unverified, as with mods) until Pumpkin offers a channel in the configuration phase, or whether that needs an upstream change first.
- Whether the signing key is the release key's sibling or its own, and who holds it.
- Whether servers outside the directory can ever ask for tickets.

## Consequences

- Three repositories change in order: the backend (key, call), the client (ask, send), `photon-server` (verify in `bridge-logic`, gate in the plugin).
- A server that loses its clock by more than a few seconds refuses everyone; the bridge allows a small skew and says so in its log.
