# Contributing

Thanks for looking. Photon is a parity project: the target is the Java client's exact behaviour, so the workflow is stricter than usual. [`AGENTS.md`](AGENTS.md) holds the same rules for AI coding agents.

## Ground rules

- **Never commit Mojang's code or assets**, or anything derived from them: images, sounds, dumps, traces or golden captures. Everything under `.data/` stays local. Do not use `git add -f` to get around `.gitignore`.
- **Expected values come from the Java oracle**, never from Rust output. See [parity](docs/parity/README.md).
- **Parity first.** Do not "fix" quirks of the Java reference, and do not hide server differences with client-side workarounds. Internal structure is free; observable behaviour is not.
- **Small, reviewable changes.** One behaviour slice per commit, with its parity cases.
- **English everywhere:** code, comments, documents and commit messages.

## Setup

```sh
cargo xtask install-hooks   # once per clone
```

The hooks enforce the [coding standards](docs/development/coding-standards.md):

- **pre-commit** runs rustfmt, the lint rules and clippy on affected crates, in seconds.
- **commit-msg** checks the message format.
- **pre-push** runs the tests of affected crates and parity.

Run `cargo xtask check` for the whole gate. Parity needs the local reference; see [development setup](docs/development/setup.md).

## Adding behaviour

1. Read the relevant Java source line by line.
2. Write original scenarios and run them through the Java oracle.
3. Capture Java's result, then implement the Rust side.
4. Compare until equal; if it differs, fix Rust, never the expectation.

A new case set is opened only for a Java branch that runs for the first time or an observed difference; a changed constant is a table row in an existing case.

## Commits and pull requests

- **Format.** [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): summary` with an English, imperative summary of at most 72 characters and details in the body. Types: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `build`, `ci`, `chore`, `style`, `revert`.
- **No tool attribution.** No `Co-Authored-By` or other tool attribution trailers.
- **Pull requests.** State what you verified and what you could not.

Open work is in [issues](https://github.com/PhotonMC/photon-client/issues). By contributing you agree that your work is licensed under the terms in [README.md](README.md#license) (MIT OR Apache-2.0).
