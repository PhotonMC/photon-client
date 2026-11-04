# ADR 0022: Backend services

- Status: accepted (2026-11-04); website sign-in is [ADR 0023](0023-website-sign-in.md)

## Context

The platform ([ADR 0018](0018-photon-platform.md)) needs a backend: Photon accounts, the server directory, one-time launch codes ([ADR 0020](0020-accounts-and-launch-links.md)) and later friends, presence, invites and chat. The owner hosts it, and the host may change. Nothing here may hold a player's Microsoft or Minecraft token.

The backend must learn which Minecraft profile (UUID and name) a request comes from, without a token.

## Decision

- **Separate service and repository.** `photon-backend` is its own repository and its own process, not routes inside the Next.js site: presence and chat need long-lived connections, and the site may move hosts. The client repository stays free of server code.
- **Rust, one binary, SQLite first.** The service is Rust (HTTP with `axum`, storage in SQLite through `rusqlite`), so the client team keeps one language, one toolchain and the same lint rules. Launch codes need an atomic single-use consume, which a SQLite transaction gives. Postgres is the path to scale and is a storage change only, behind a small storage trait.
- **Identity proof without a token (client).** The client asks the backend for a challenge and gets a random nonce and a random challenge secret. It calls Mojang's `session/minecraft/join` with the nonce as the server id, exactly as it does for an online-mode server ([ADR 0007](0007-assets-and-accounts.md)), then asks the backend to verify and presents the secret. The backend calls Mojang's `hasJoined` with the profile name and the nonce and receives the UUID and name, then issues a Photon session bound to that UUID to the holder of the secret only. The Minecraft token never leaves the machine. Checked on 2026-11-04 with `photon identity-proof` (a development tool that takes the id from its arguments; the real sign-in asks for its own challenge and never uses outside input): a join for a random 32-hex server id, followed by `hasJoined` for that name and id, returned the profile (HTTP 200). A nonce is single use and lives about 30 seconds. The client always requests the challenge itself and never takes a nonce, a secret or a session from a link or any other outside input; otherwise a hostile page could have a victim's client prove an attacker's nonce and hand the attacker a session for the victim's profile.
- **Website sign-in is an open question.** Nothing here moves a session from the client to the browser, and a naive pairing (the site shows a code, the client binds it) lets a hostile site make the victim's client approve the attacker's pairing. The candidates are a pairing that shows the same short code in the client and the browser and needs a confirmation in the client, or server-side Microsoft OAuth on the site with the tokens discarded after one exchange. It needs the Azure application's redirect settings. It is decided in its own ADR and is not part of the first backend slice.
- **One directory entry.** A server's entry holds the fields of the loading screen ([#220](https://github.com/PhotonMC/photon-client/issues/220)) and of the game page ([ADR 0021](0021-client-ui-technology.md)) once: `id`, `name`, `address`, `description`, `images` (URLs), `players`, `version`. Addresses are never put in a `photon://` link; only the `id` is.
- **M1 contract.** Five calls, JSON over HTTPS: `POST /v1/identity/challenge` (returns the nonce and the secret), `POST /v1/identity/verify` (needs the secret), `GET /v1/servers`, `POST /v1/launch` (issue a code for a server id, session required) and `POST /v1/launch/exchange` (consume a code: returns the address and the bound UUID). A code is single use, valid for 60 seconds, at least 128 bits from a cryptographic random source, and consumed in one atomic storage statement.
- **Development identity.** A `--dev-identity` flag accepts a profile name without Mojang. It exists only in builds with the `dev-identity` Cargo feature, which release builds do not enable, so it cannot be switched on in production.
- **Shared types.** The request and response types live in a `photon-wire` crate of the backend repository. The client reads them by git dependency once the contract is stable; until then the contract in this ADR is the reference. (`photon-api` in the client repository is the mod API of ADR 0006 and is not used for this.)

## Consequences

- The first slices are the backend skeleton with these calls, then the client's sign-in and a directory-fed home screen, then the loading screen fed from an entry.
- Players and invites, friends and chat build on the Photon session and come in phase 2.
- Creating the `photon-backend` repository in the organisation is the owner's call.
