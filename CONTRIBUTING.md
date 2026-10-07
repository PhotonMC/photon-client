# Contributing

Thanks for looking. This is a parity project: the target is the Java client's exact behavior, so the workflow is stricter than usual. Read [`AGENTS.md`](AGENTS.md) first; it is the single rule set for humans and AI agents alike (in Turkish). The essentials are below.

## Ground rules

- **Never commit Mojang's code or assets**, or anything derived from them (images, sounds, dumps, traces, golden captures). Everything under `.data/` stays local. Do not use `git add -f` to get around `.gitignore`.
- **Parity first, optimization later.** Do not "fix" quirks of the Java reference, and do not hide server differences with client-side workarounds.
- **Expected values come from the Java oracle**, never from Rust output.
- Keep slices small and reviewable. One slice, one commit.
- Code follows the [coding standards](docs/coding-standards.md): crate layers, naming and complexity limits. Clippy must stay warning-free.

## Build and test

```sh
cargo build --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib --bins --tests
```

These need no game files and are what CI runs. The full gate additionally compares Rust against the Java reference and needs the local reference data:

```sh
python3 tools/voxel.py setup        # once: fetch the pinned game into .data/
python3 tools/voxel.py check        # prints CHECK OK or CHECK FAILED
python3 tools/voxel.py parity --family gui
```

See the [local setup guide](docs/local-setup.md), [behavior testing](docs/behavior-testing.md) and the [behavior runner](docs/behavior-runner.md). If you changed the oracle sources or the case generators in `tools/behavior/`, run `check --capture`.

## Adding a behavior slice

1. Read the relevant Java source line by line.
2. Write an original scenario and run it through the Java oracle.
3. Capture the Java result, then implement the Rust side.
4. Compare, and record the contract and parity in its own file under [`docs/records/`](docs/records/).

Open work is listed in [`docs/tasks/`](docs/tasks/README.md); each task file carries the context it needs. Do not open a new slice unless the Java source has a distinct branch or an observed difference; a changed constant is a table row, not a slice.

## Commits and pull requests

- Conventional Commits prefix (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`), a title of **72 characters or fewer**, and the detail in the body.
- Commit messages in this repository are written in Turkish; English is fine for outside contributions.
- No `Co-Authored-By` or other AI attribution footers.
- Run the relevant checks before you push and say in the pull request what you verified and what you could not.

By contributing you agree that your work is licensed under the terms in [README.md](README.md#license) (MIT OR Apache-2.0).
