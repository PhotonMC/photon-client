# ADR 0007: Game assets and accounts

- Status: accepted (2026-10-10), not yet implemented

## Context

The client needs Mojang's assets (textures, sounds, languages) and the version metadata to run. Until now the rule was that an installation neither contains nor downloads original assets, and that the user points the client at a local folder. That conflicts with the goal that a player only opens the game and everything works.

## Decision

- The client signs the player in with a Microsoft account and verifies game ownership, as the official launcher and third-party launchers such as Prism do.
- After verification, it downloads the version metadata and assets from Mojang's official servers (piston-meta and the asset CDN) into a local content-addressed cache. Every file is checked against its published hash.
- Photon never distributes original content: none is embedded in the binary, packaged, committed or uploaded as a CI artifact.
- The launcher lives inside the client (one Rust binary). There is no separate launcher application.
- This also settles the open online-identity decision (encryption, session join, account login): it is required.

## Consequences

- Development keeps using `.data/` as the local reference copy. The production asset cache is a separate, user-level directory.
- Tokens are kept in the operating system keyring, never in plain files.
- Login encryption uses the RustCrypto crates `aes`, `cfb8`, `rsa` and `sha1` plus `getrandom`, in `photon-protocol`. The session join is posted with `ureq` (rustls) from `photon-net`; the keyring joins them when the Microsoft sign-in lands.
