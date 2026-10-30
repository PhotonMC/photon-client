# ADR 0015: Signed updates and key rotation

- Status: accepted (2026-10-30)

## Context

The client replaces its own files with a downloaded release ([issue 31](https://github.com/PhotonMC/photon-client/issues/31)). A release is trusted only if it carries a valid signature; the public key sits in the client, the private key is a secret of the release workflow. The owner decided where releases live, how the key is made and that the update system has no interface of its own yet.

## Decision

- **Host.** Releases are GitHub Releases of the repository. The client reads `https://github.com/PhotonMC/photon-client/releases/latest/download/<file>` and follows its redirects (a bounded number), over https only, with size caps on every body. `latest` excludes pre-releases, so the only channel today is `stable`.
- **Files of a release.** Per platform (the Rust target triple the client was built for): `photon-<triple>.manifest`, `photon-<triple>.manifest.sig` and the archive `photon-<version>-<triple>.tar.gz` the manifest names. The manifest is `key=value` lines: `version`, `archive`, `sha256`, `size` (all required), `notes` (one line) and `protocols` (the protocol numbers the release can join servers with). Unknown fields are ignored so a later release can add fields without stranding older clients; a repeated field is refused.
- **Signature file.** One hex Ed25519 signature per line, over the manifest's exact bytes. A manifest is accepted if any line verifies under any key of the client's list.
- **Keys.** The client carries a list of public keys (`RELEASE_KEYS`), not one constant. An empty list makes every update refuse, and then the client makes no network call to the release host at all. The private key is made by `cargo xtask release-key`, lives in a secret of the protected `release` environment and in the owner's offline backup, and is read by the workflow only from an environment variable.
- **Rotation.** To change the key: (1) make the new key; (2) publish a release signed with the old key whose build lists old and new; (3) from then on sign every release with both keys (two lines) for as long as clients older than step 2 should still be able to update; (4) a later release drops the old key from the list and is signed with the new key only. A client that skips step 2 and meets a release signed only with the new key refuses it; dual signing in step 3 is what keeps it from being stranded. If the old key is compromised there is no safe rotation: the new key reaches users through a manual download, and the release host's account is the root of trust for that.
- **Download and apply.** A check stages a verified archive; the swap happens at the next start, before any window opens (already built). An update never changes files while the game runs.
- **No interface.** The update system is a service with states, operations and events published through `photon-api`, so any interface (the one that replaces the current screens, or a mod) can bind to it. Until one exists the default behaviour is: check at start and every six hours, download and stage automatically, apply at the next start, log only. Settings live in a config file next to the staged release: `auto_check`, `auto_download`, `channel`.

## Consequences

- Nothing is published until the owner adds the public key to the source and the private key to the `release` environment; the release workflow fails clearly (it does not skip signing) when either is missing or when they do not match.
- Windows and macOS code signing are separate and still open ([issue 57](https://github.com/PhotonMC/photon-client/issues/57)).
