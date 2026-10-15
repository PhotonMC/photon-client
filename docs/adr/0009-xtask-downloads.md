# ADR 0009: Downloads and archives in xtask

- Status: accepted (2026-10-15)

## Context

`cargo xtask` is meant to need nothing beyond the Rust toolchain ([ADR 0005](0005-parity-v2.md)). The reference setup (the pinned game, its libraries and assets), the test server, the launcher, RCON and the bench and screenshot runs were still `tools/photon.py` and a Python package. Python was therefore a development and CI requirement, and the same commands could not be shared with the later in-client downloader ([ADR 0007](0007-assets-and-accounts.md)).

Two capabilities were missing from the workspace: fetching files over HTTPS and reading the client jar, which is a zip archive.

## Decision

- **HTTPS: `ureq` 3 with its default features.** It is a blocking client with a small dependency tree, uses `rustls` with `ring` and the bundled `webpki-roots`, and needs no system TLS library or CMake step. The downloader is blocking and parallel with `rayon`; an async runtime is not warranted for a few hundred small files.
- **Archives: `zip` with `deflate-flate2` only.** The jar is read with the `flate2` backend the workspace already uses. The other compression backends are off.
- **Hashes.** SHA-1 and SHA-256 come from the `sha1` and `sha2` crates xtask already has. Every download is checked against the hash Mojang publishes before it replaces a file.
- **Scope.** Both crates are dependencies of `xtask` only. The game client does not link them; its own downloader stays a separate decision (#30).
- **License.** `webpki-roots` is CDLA-Permissive-2.0, which `deny.toml` now allows.
- **Commands.** `cargo xtask setup|server|play|rcon|bench|shot` replace `tools/photon.py` with the same options and the same `.data/` layout. The Python tools are deleted.

## Consequences

- Python is no longer needed for development or for the `setup` step in CI.
- A first `cargo xtask` build compiles a TLS stack. The dev profile builds xtask with optimisation already, so the hooks stay fast once it is built.
- The generators in `tools/generate/` and the render scripts in `tools/render/` are still Python; they move with #14 and the render track.
