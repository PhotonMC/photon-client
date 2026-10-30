# Releasing

A release is built and signed by the `Release` workflow when a tag `v<version>` is pushed, with `<version>` equal to the workspace version in `Cargo.toml`. The workflow publishes a GitHub Release with, for each platform, the archive, a signed manifest and its signature ([ADR 0015](../adr/0015-signed-updates.md)). No secret value is written anywhere in the repository, in logs or in this page.

## One-time setup (repository owner)

1. Make the key pair on your own machine:

   ```
   cargo xtask release-key ~/photon-release.key
   ```

   The private key goes to that file (readable by you only; an existing file is never overwritten) and is never printed. The command prints the public key.
2. Make an offline backup of the file (a password manager or an encrypted drive that is not connected to the machine). Losing it means rotating the key by a manual download ([ADR 0015](../adr/0015-signed-updates.md)).
3. Put the public key into `RELEASE_KEYS` in `crates/photon-update/src/lib.rs` and commit it. Until then no client trusts any release and the workflow refuses to sign.
4. In the repository settings create an environment named `release`:
   - deployment branches and tags: only tags matching `v*`;
   - required reviewers: you.
5. Add the secret `RELEASE_SIGNING_KEY` to that environment, with the content of the key file (the hex text). Do not add it as a repository secret: only the `release` environment may hold it.
6. Delete or secure the key file as your backup policy says.

## Making a release

1. Set the workspace version, merge it, and make sure CI is green.
2. Push the tag `v<version>`.
3. The workflow builds the six platform archives, then waits in the `release` environment for your approval. After you approve, it signs the manifests with the secret and creates the GitHub Release.

The signing step (`cargo xtask release-sign`) refuses, with a message, to sign when:
- `RELEASE_SIGNING_KEY` is empty or is not a signing key;
- `RELEASE_KEYS` in the source is empty;
- the secret's public key is not in `RELEASE_KEYS` (no client would trust the signature);
- the tag is not `v<version>` of the source.

Nothing is published when the step fails.

## What the client does

The client has no update screen. While `photon menu` or `photon play` runs, one background service asks `https://github.com/PhotonMC/photon-client/releases/latest/download/` for `photon-<triple>.manifest` and its `.sig` at start and every 6 hours, over https only, with a size limit on every body. A newer verified release is downloaded and stored; the next start applies it (see [ADR 0015](../adr/0015-signed-updates.md)). A build whose `RELEASE_KEYS` is empty never contacts the host.

- `<user data>/photon/update/settings.txt` holds `auto_check=true|false`, `auto_download=true|false` and `channel=stable` (the only channel), one per line, `#` for comments. A missing file means the defaults (check and download); a wrong file is logged and ignored.
- `photon update` checks once, downloads what it finds and prints each state.
- The service reports `UpdateState` (Idle, Checking, Available, Downloading, Ready, Failed) and `UpdateChanged` events through `photon-api`; `server_needs_newer_version` fires when a server list ping shows a protocol above every supported one, naming a stored or available release that supports it.
- `restart_into_update` ends the game as a window close does and starts the program again with the same arguments, which applies the stored release first. The dev script step `update_restart` calls it.

## Rotating the key

Follow the four steps of [ADR 0015](../adr/0015-signed-updates.md). In the overlap of step 3 the `release` environment holds the new key as `RELEASE_SIGNING_KEY` and the old one as `RELEASE_SIGNING_KEY_PREVIOUS`: every manifest then gets two signature lines. Remove the second secret when the old key leaves the list.

## Testing without a release

- `cargo xtask package` makes this platform's archive in `target/package`.
- The unit tests of `photon-update` and `xtask` cover signing and verification with throw-away keys.
- Never push a test tag to the real repository: a `v*` tag starts the workflow.
