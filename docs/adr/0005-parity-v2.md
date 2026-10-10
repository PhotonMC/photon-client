# ADR 0005: Parity system v2

- Status: accepted (2026-10-10), in progress

## Context

The parity system compares Rust with the real Java client by running Java code headless as an oracle. It works, but by 2026-10 it had become the slowest and heaviest part of development:

- **Disk.** Nothing was ever cleaned: 67 GB of run directories and captures had piled up.
- **No selection.** Every gate run built and tested the whole workspace in release mode, then ran all 16 families.
- **All-or-nothing cache.** One changed oracle file forced a full recapture.
- **Sequential work.** Java ran twice in sequence (about 26 s each) and the Rust adapter was single-threaded (about 65 s for 1156 cases).
- **Fragile across versions.** 73 of 123 Java oracle files reach private members by reflection and 50 use `Unsafe`.
- **Three languages.** Case generation is 26k lines of Python, the oracle 38k lines of Java, the adapter 34k lines of Rust.

## Decision

- **Location.** The system stays in this repository, as a separate `parity/` layer, so a behaviour change and its test land in one commit.
- **Rust runner.** The runner moves to Rust (`cargo xtask parity`). Case definitions move from Python to Rust. Each case declares its family, its Java target, the Rust crates and modules it covers, its version range and its evidence class.
- **Per-case cache.** Captures are content-addressed per case: `hash(jar, jdk, the family's oracle sources, case input)`. Goldens are stored packed per version and family. Run output is kept only on failure, and garbage collection is automatic.
- **Affected-only runs.** The changed files map to crates, and crates to the cases tagged with them. A full run happens in CI and with `--all`.
- **Parallel execution.** Families are sharded across JVMs, and the repeatability check runs only for newly captured cases. The Rust adapter evaluates independent cases with `rayon`.
- **Thin Java oracle per version.** A version-independent core (`parity/oracle/common`) handles JSONL I/O and dispatch. Each version has its own adapter (`parity/oracle/<version>`). Reflective access is concentrated in one `Access` class per version.
- **No screenshots.** GUI parity compares recorded draw calls; render parity compares meshes, vertices and draw lists.
- **CI.** The runner downloads the official game jar from Mojang into its cache and never uploads original or derived content as artifacts.
- **Records become data.** The 670 Markdown conformance records become metadata on the cases they describe, and a coverage report is generated from it.

## Consequences

Targets:

- Under 30 s for a typical affected-only run.
- Under 2 minutes for a full run.
- Under 300 MB of local parity data.

Java stays only where it has to: calling Minecraft, which runs on the JVM.
